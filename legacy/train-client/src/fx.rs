//! Lightweight, time-based visual effects (pure rendering — no game state).
//!
//! Everything is keyed off a single `now` clock (seconds, from `get_time()`),
//! mirroring the pulsing-root pattern in [`crate::view`]. Effects store screen
//! positions captured at spawn; they live well under a second, so a resize
//! mid-effect is not worth tracking.

use crate::theme;
use crate::view;
use macroquad::prelude::*;

/// Smoothstep easing on `t in [0,1]`. Reused for train motion and effect curves.
pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

enum Effect {
    /// Floating text that rises and fades (e.g. "+10").
    Popup {
        pos: Vec2,
        text: String,
        color: Color,
        size: u16,
        born: f32,
        life: f32,
    },
    /// Expanding ring that fades (delivery impact).
    Burst {
        pos: Vec2,
        color: Color,
        radius: f32,
        born: f32,
        life: f32,
    },
    /// Quick filled-circle pop (switch tap feedback).
    Flash {
        pos: Vec2,
        color: Color,
        radius: f32,
        born: f32,
        life: f32,
    },
    /// Small rising puff that fades (smoke).
    Smoke {
        pos: Vec2,
        drift: Vec2,
        radius: f32,
        born: f32,
        life: f32,
    },
}

impl Effect {
    fn dead(&self, now: f32) -> bool {
        let (born, life) = match self {
            Effect::Popup { born, life, .. }
            | Effect::Burst { born, life, .. }
            | Effect::Flash { born, life, .. }
            | Effect::Smoke { born, life, .. } => (*born, *life),
        };
        now - born >= life
    }
}

/// A small pool of active effects.
#[derive(Default)]
pub struct Effects {
    items: Vec<Effect>,
    now: f32,
}

impl Effects {
    pub fn new() -> Self {
        Self::default()
    }

    /// Advance the clock and drop finished effects. Call once per frame.
    pub fn update(&mut self, now: f32) {
        self.now = now;
        let t = now;
        self.items.retain(|e| !e.dead(t));
    }

    pub fn popup(&mut self, pos: Vec2, text: impl Into<String>, color: Color, size: u16) {
        self.items.push(Effect::Popup {
            pos,
            text: text.into(),
            color,
            size,
            born: self.now,
            life: 0.9,
        });
    }

    pub fn burst(&mut self, pos: Vec2, color: Color, radius: f32) {
        self.items.push(Effect::Burst {
            pos,
            color,
            radius,
            born: self.now,
            life: 0.45,
        });
    }

    pub fn flash(&mut self, pos: Vec2, color: Color, radius: f32) {
        self.items.push(Effect::Flash {
            pos,
            color,
            radius,
            born: self.now,
            life: 0.25,
        });
    }

    pub fn smoke(&mut self, pos: Vec2, radius: f32) {
        // A touch of deterministic-ish jitter from the clock so puffs spread.
        let j = (self.now * 53.0).sin();
        self.items.push(Effect::Smoke {
            pos,
            drift: vec2(j * 8.0, -22.0),
            radius,
            born: self.now,
            life: 0.7,
        });
    }

    pub fn draw(&self) {
        let now = self.now;
        for e in &self.items {
            match e {
                Effect::Popup {
                    pos,
                    text,
                    color,
                    size,
                    born,
                    life,
                } => {
                    let p = ((now - born) / life).clamp(0.0, 1.0);
                    let y = pos.y - smoothstep(p) * 36.0;
                    let c = fade(*color, 1.0 - p);
                    view::centered_text(text, pos.x, y, *size, c);
                }
                Effect::Burst {
                    pos,
                    color,
                    radius,
                    born,
                    life,
                } => {
                    let p = ((now - born) / life).clamp(0.0, 1.0);
                    let r = radius * (0.3 + smoothstep(p) * 1.4);
                    draw_circle_lines(pos.x, pos.y, r, 3.0, fade(*color, 1.0 - p));
                }
                Effect::Flash {
                    pos,
                    color,
                    radius,
                    born,
                    life,
                } => {
                    let p = ((now - born) / life).clamp(0.0, 1.0);
                    let r = radius * (1.0 + 0.6 * p);
                    draw_circle(pos.x, pos.y, r, fade(*color, (1.0 - p) * 0.8));
                }
                Effect::Smoke {
                    pos,
                    drift,
                    radius,
                    born,
                    life,
                } => {
                    let p = ((now - born) / life).clamp(0.0, 1.0);
                    let c = *pos + *drift * p;
                    let r = radius * (0.6 + p);
                    draw_circle(c.x, c.y, r, fade(theme::SMOKE, (1.0 - p) * 0.5));
                }
            }
        }
    }
}

/// Copy a colour with a new alpha multiplier.
fn fade(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, c.a * a.clamp(0.0, 1.0))
}
