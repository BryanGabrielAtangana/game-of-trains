//! Game state machine, input handling and the per-frame update/draw.

use crate::theme;
use crate::view::{self, Layout};
use macroquad::prelude::*;
use train_core::{daily_seed, GameConfig, Input, Simulation, TICKS_PER_SECOND};

/// Accuracy (percent) needed to advance to the next level.
const PASS_ACCURACY: u32 = 80;

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
}

impl App {
    pub fn new() -> App {
        // Phase 4 will fetch the real date from the server; for now a fixed
        // daily seed keeps every launch reproducible.
        let seed = daily_seed(2026, 6, 17);
        let level = 1;
        let sim = Simulation::new(&GameConfig::new(seed, level));
        let layout = Layout::fit(sim.map(), playfield());
        App {
            seed,
            level,
            sim,
            phase: Phase::Ready,
            accumulator: 0.0,
            inputs: Vec::new(),
            layout,
            last_passed: false,
        }
    }

    fn start_round(&mut self, level: u32) {
        self.level = level;
        self.sim = Simulation::new(&GameConfig::new(self.seed, level));
        self.accumulator = 0.0;
        self.inputs.clear();
        self.phase = Phase::Ready;
    }

    pub fn update(&mut self) {
        // Layout depends on the live window size; recompute each frame.
        self.layout = Layout::fit(self.sim.map(), playfield());

        let tap = tap_position();

        match self.phase {
            Phase::Ready => {
                if tap.is_some() {
                    self.phase = Phase::Playing;
                    self.accumulator = 0.0;
                }
            }
            Phase::Playing => {
                if let Some(p) = tap {
                    self.handle_tap(p);
                }
                self.advance_sim();
                if self.sim.is_finished() {
                    self.last_passed = self.sim.scorer().accuracy() >= PASS_ACCURACY;
                    self.phase = Phase::RoundOver;
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
            self.sim.step();
            self.accumulator -= dt;
            budget -= 1;
        }
    }

    fn handle_tap(&mut self, p: Vec2) {
        // Toggle the nearest switch within a finger-friendly radius.
        let map = self.sim.map();
        let radius = (self.layout.cell() * 0.5).max(26.0);
        let mut best: Option<(usize, f32)> = None;
        for (i, node) in map.nodes.iter().enumerate() {
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

    fn draw_hud(&self) {
        let s = self.sim.scorer();
        draw_rectangle(0.0, 0.0, screen_width(), 56.0, theme::PANEL);
        let y = 36.0;
        draw_text(format!("Score {}", s.score), 16.0, y, 30.0, theme::WHITE);
        draw_text(
            format!("{}/{}", s.correct, s.total),
            screen_width() * 0.5 - 30.0,
            y,
            30.0,
            theme::WHITE,
        );
        if s.combo >= 2 {
            view::centered_text(
                &format!("x{} combo", s.combo),
                screen_width() * 0.5,
                72.0,
                26,
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
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;
        draw_rectangle(0.0, cy - 70.0, screen_width(), 140.0, theme::PANEL);
        view::centered_text(title, cx, cy - 16.0, 56, theme::WHITE);
        view::centered_text(subtitle, cx, cy + 34.0, 26, theme::WHITE);
    }

    fn draw_results(&self) {
        let s = self.sim.scorer();
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;
        let (headline, color) = if self.last_passed {
            ("GREAT JOB", theme::GOOD)
        } else {
            ("TRY AGAIN", theme::BAD)
        };
        draw_rectangle(0.0, cy - 100.0, screen_width(), 200.0, theme::PANEL);
        view::centered_text(headline, cx, cy - 46.0, 60, color);
        view::centered_text(
            &format!("Accuracy {}%   Score {}", s.accuracy(), s.score),
            cx,
            cy + 6.0,
            30,
            theme::WHITE,
        );
        let next = if self.last_passed {
            "Tap for the next level"
        } else {
            "Tap to retry this level"
        };
        view::centered_text(next, cx, cy + 52.0, 26, theme::WHITE);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// The play area, below the HUD bar.
fn playfield() -> Rect {
    Rect::new(0.0, 56.0, screen_width(), (screen_height() - 56.0).max(1.0))
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
