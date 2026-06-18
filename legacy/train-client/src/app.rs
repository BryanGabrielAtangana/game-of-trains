//! Game state machine, input handling and the per-frame update/draw.

use crate::audio::{self, Sfx};
use crate::fx::{smoothstep, Effects};
use crate::theme;
use crate::view::{self, Layout};
use macroquad::prelude::*;
use train_core::{
    daily_seed, daily_seed_from_unix, GameConfig, Input, Outcome, Simulation, TICKS_PER_SECOND,
};

/// Accuracy (percent) needed to advance to the next level.
const PASS_ACCURACY: u32 = 80;
/// Base HUD bar height (safe-area inset is added on top).
const HUD_BASE: f32 = 56.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// Map is shown, waiting for the player to begin.
    Ready,
    /// Trains are running.
    Playing,
    /// Round finished; results shown.
    RoundOver,
}

pub struct App {
    seed: u64,
    level: u32,
    sim: Simulation,
    phase: Phase,
    /// Leftover real time not yet consumed by fixed-timestep stepping.
    accumulator: f32,
    /// Player switch toggles this round (seed of a future server-verified Run).
    inputs: Vec<Input>,
    layout: Layout,
    last_passed: bool,
    effects: Effects,
    /// `get_time()` at the last phase change — drives banner entrance.
    phase_since: f32,
    /// Seconds since the last smoke puff was emitted.
    smoke_timer: f32,
    /// Safe-area top inset (px), so the HUD clears a notch.
    safe_top: f32,
}

impl App {
    pub fn new() -> App {
        let seed = today_seed();
        let level = 1;
        let sim = Simulation::new(&GameConfig::new(seed, level));
        let safe_top = sanitize_inset(audio::safe_area_top_px());
        let layout = Layout::fit(sim.map(), playfield(safe_top));
        App {
            seed,
            level,
            sim,
            phase: Phase::Ready,
            accumulator: 0.0,
            inputs: Vec::new(),
            layout,
            last_passed: false,
            effects: Effects::new(),
            phase_since: 0.0,
            smoke_timer: 0.0,
            safe_top,
        }
    }

    fn start_round(&mut self, level: u32) {
        self.level = level;
        self.sim = Simulation::new(&GameConfig::new(self.seed, level));
        self.accumulator = 0.0;
        self.inputs.clear();
        self.phase = Phase::Ready;
        self.phase_since = get_time() as f32;
    }

    pub fn update(&mut self) {
        let now = get_time() as f32;
        // Layout depends on the live window size; recompute each frame.
        self.layout = Layout::fit(self.sim.map(), playfield(self.safe_top));
        self.effects.update(now);

        let tap = tap_position();

        match self.phase {
            Phase::Ready => {
                if tap.is_some() {
                    self.phase = Phase::Playing;
                    self.phase_since = now;
                    self.accumulator = 0.0;
                }
            }
            Phase::Playing => {
                if let Some(p) = tap {
                    self.handle_tap(p);
                }
                self.advance_sim();
                self.emit_smoke(now);
                if self.sim.is_finished() {
                    self.last_passed = self.sim.scorer().accuracy() >= PASS_ACCURACY;
                    self.phase = Phase::RoundOver;
                    self.phase_since = now;
                    audio::play(if self.last_passed {
                        Sfx::Win
                    } else {
                        Sfx::Lose
                    });
                }
            }
            Phase::RoundOver => {
                if tap.is_some() {
                    let next = if self.last_passed {
                        self.level + 1
                    } else {
                        self.level
                    };
                    self.start_round(next);
                }
            }
        }
    }

    fn advance_sim(&mut self) {
        self.accumulator += get_frame_time();
        let dt = 1.0 / TICKS_PER_SECOND as f32;
        // Cap catch-up so a long stall (e.g. a backgrounded tab) can't freeze
        // the game by running thousands of steps at once.
        let mut budget = 600;
        while self.accumulator >= dt && budget > 0 {
            self.step_with_effects();
            self.accumulator -= dt;
            budget -= 1;
        }
    }

    /// Step the simulation one tick, spawning delivery effects/sounds for any
    /// trains that resolve. We predict which trains will resolve *this* tick
    /// (the engine advances progress by 1 then resolves a terminal `to`), which
    /// lets us recover the delivery station without changing the engine.
    fn step_with_effects(&mut self) {
        let resolving: Vec<usize> = {
            let map = self.sim.map();
            self.sim
                .trains()
                .iter()
                .filter(|t| {
                    t.progress + 1 >= t.edge_ticks
                        && t.to.is_some_and(|to| map.nodes[to].is_terminal())
                })
                .filter_map(|t| t.to)
                .collect()
        };

        let before = self.sim.outcomes().len();
        let score_before = self.sim.scorer().score;
        self.sim.step();
        let new: Vec<Outcome> = self.sim.outcomes()[before..].to_vec();
        if new.is_empty() {
            return;
        }
        let delta = self.sim.scorer().score - score_before;
        let cell = self.layout.cell();
        let size = (cell * 0.36).max(16.0) as u16;

        for (k, outcome) in new.iter().enumerate() {
            let pos = resolving
                .get(k)
                .map(|&n| self.layout.pt(self.sim.map().nodes[n].pos));
            let (text, color, sfx) = match outcome {
                Outcome::Good => (
                    if new.len() == 1 {
                        format!("+{}", delta.max(0))
                    } else {
                        "GOOD".to_string()
                    },
                    theme::GOOD,
                    Sfx::Good,
                ),
                Outcome::Bad => (
                    if new.len() == 1 {
                        delta.to_string()
                    } else {
                        "WRONG".to_string()
                    },
                    theme::BAD,
                    Sfx::Bad,
                ),
                Outcome::Ugly => (
                    if new.len() == 1 {
                        delta.to_string()
                    } else {
                        "OOPS".to_string()
                    },
                    theme::BAD,
                    Sfx::Ugly,
                ),
            };
            if let Some(p) = pos {
                self.effects.popup(p, text, color, size);
                self.effects.burst(p, color, cell * 0.5);
            }
            audio::play(sfx);
        }
    }

    fn emit_smoke(&mut self, now: f32) {
        if self.sim.trains().is_empty() {
            return;
        }
        if now - self.smoke_timer < 0.12 {
            return;
        }
        self.smoke_timer = now;
        let root = self.sim.map().root;
        let p = self.layout.pt(self.sim.map().nodes[root].pos);
        let cell = self.layout.cell();
        self.effects.smoke(p + vec2(0.0, -cell * 0.25), cell * 0.12);
    }

    fn handle_tap(&mut self, p: Vec2) {
        // Toggle the nearest switch within a finger-friendly radius.
        let radius = (self.layout.cell() * 0.6).max(40.0);
        let mut best: Option<(usize, f32)> = None;
        for (i, node) in self.sim.map().nodes.iter().enumerate() {
            if !node.is_switch() {
                continue;
            }
            let d = self.layout.pt(node.pos).distance(p);
            if d <= radius && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((i, d));
            }
        }
        if let Some((node, _)) = best {
            self.sim.toggle(node);
            self.inputs.push(Input {
                tick: self.sim.tick(),
                node,
            });
            let pos = self.layout.pt(self.sim.map().nodes[node].pos);
            self.effects
                .flash(pos, theme::RAIL_ACTIVE, self.layout.cell() * 0.5);
            audio::play(Sfx::Switch);
        }
    }

    pub fn draw(&self) {
        // Sky over grass.
        clear_background(theme::SKY);
        let horizon = screen_height() * 0.42;
        draw_rectangle(
            0.0,
            horizon,
            screen_width(),
            screen_height() - horizon,
            theme::GRASS,
        );

        let time = get_time() as f32;
        view::draw_map(&self.sim, &self.layout, time);
        view::draw_trains(&self.sim, &self.layout);
        self.effects.draw();

        self.draw_hud();

        match self.phase {
            Phase::Ready => self.draw_center_banner(
                "GAME OF TRAINS",
                "Tap a switch to change the tracks. Tap anywhere to start.",
            ),
            Phase::RoundOver => self.draw_results(),
            Phase::Playing => {}
        }
    }

    /// 0→1 eased progress of the current phase's entrance animation.
    fn entrance(&self) -> f32 {
        smoothstep((get_time() as f32 - self.phase_since) / 0.3)
    }

    fn draw_hud(&self) {
        let s = self.sim.scorer();
        let h = HUD_BASE + self.safe_top;
        draw_rectangle(0.0, 0.0, screen_width(), h, theme::PANEL);
        let y = self.safe_top + 36.0;
        draw_text(format!("Score {}", s.score), 16.0, y, 30.0, theme::WHITE);
        draw_text(
            format!("{}/{}", s.correct, s.total),
            screen_width() * 0.5 - 30.0,
            y,
            30.0,
            theme::WHITE,
        );
        if s.combo >= 2 {
            // Pulse the combo label so streaks feel alive.
            let pulse = 24.0 + 4.0 * (get_time() as f32 * 8.0).sin();
            view::centered_text(
                &format!("x{} combo", s.combo),
                screen_width() * 0.5,
                h + 18.0,
                pulse as u16,
                theme::RAIL_ACTIVE,
            );
        }
        let lvl = format!("Level {}", self.level);
        let dims = measure_text(&lvl, None, 30, 1.0);
        draw_text(
            &lvl,
            screen_width() - dims.width - 16.0,
            y,
            30.0,
            theme::WHITE,
        );
    }

    fn draw_center_banner(&self, title: &str, subtitle: &str) {
        let e = self.entrance();
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0 - (1.0 - e) * 16.0;
        draw_rectangle(0.0, cy - 70.0, screen_width(), 140.0, fade(theme::PANEL, e));
        view::centered_text(
            title,
            cx,
            cy - 16.0,
            (56.0 * (0.8 + 0.2 * e)) as u16,
            theme::WHITE,
        );
        view::centered_text(subtitle, cx, cy + 34.0, 26, fade(theme::WHITE, e));
    }

    fn draw_results(&self) {
        let e = self.entrance();
        let s = self.sim.scorer();
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0 - (1.0 - e) * 16.0;
        let (headline, color) = if self.last_passed {
            ("GREAT JOB", theme::GOOD)
        } else {
            ("TRY AGAIN", theme::BAD)
        };
        draw_rectangle(
            0.0,
            cy - 100.0,
            screen_width(),
            200.0,
            fade(theme::PANEL, e),
        );
        view::centered_text(
            headline,
            cx,
            cy - 46.0,
            (60.0 * (0.8 + 0.2 * e)) as u16,
            color,
        );
        view::centered_text(
            &format!("Accuracy {}%   Score {}", s.accuracy(), s.score),
            cx,
            cy + 6.0,
            30,
            fade(theme::WHITE, e),
        );
        let next = if self.last_passed {
            "Tap for the next level"
        } else {
            "Tap to retry this level"
        };
        view::centered_text(next, cx, cy + 52.0, 26, fade(theme::WHITE, e));
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Today's daily seed from the device clock (UTC day), with a fixed fallback.
fn today_seed() -> u64 {
    let secs = macroquad::miniquad::date::now();
    if secs.is_finite() && secs > 0.0 {
        daily_seed_from_unix(secs as u64)
    } else {
        daily_seed(2026, 6, 17)
    }
}

fn sanitize_inset(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 200.0)
    } else {
        0.0
    }
}

fn fade(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, c.a * a.clamp(0.0, 1.0))
}

/// The play area, below the HUD bar (which includes the safe-area inset).
fn playfield(safe_top: f32) -> Rect {
    let top = HUD_BASE + safe_top;
    Rect::new(0.0, top, screen_width(), (screen_height() - top).max(1.0))
}

/// A single tap/click this frame, if any (mouse or first touch).
fn tap_position() -> Option<Vec2> {
    if is_mouse_button_pressed(MouseButton::Left) {
        let (x, y) = mouse_position();
        return Some(vec2(x, y));
    }
    touches()
        .iter()
        .find(|t| t.phase == TouchPhase::Started)
        .map(|t| t.position)
}
