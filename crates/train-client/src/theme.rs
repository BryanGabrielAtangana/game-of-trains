//! Colour palette, tuned to echo the original game's flat, sunny look.

use macroquad::prelude::Color;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}

pub const SKY: Color = rgb(79, 193, 233);
pub const GRASS: Color = rgb(160, 212, 104);
pub const INK: Color = rgb(43, 58, 66);
pub const TRACK: Color = rgb(120, 100, 78);
pub const TRACK_DIM: Color = rgb(150, 140, 128);
pub const RAIL_ACTIVE: Color = rgb(255, 206, 84);
pub const SWITCH: Color = rgb(79, 156, 236);
pub const SWITCH_ALT: Color = rgb(252, 110, 81);
pub const ROOT: Color = rgb(140, 193, 82);
pub const HOUSE: Color = rgb(233, 87, 63);
pub const HOUSE_ROOF: Color = rgb(59, 47, 47);
pub const TRAIN: Color = rgb(237, 85, 101);
pub const DEAD_END: Color = rgb(43, 58, 66);
pub const PANEL: Color = Color::new(0.0, 0.0, 0.0, 0.45);
pub const WHITE: Color = rgb(255, 255, 255);
pub const GOOD: Color = rgb(140, 193, 82);
pub const BAD: Color = rgb(237, 85, 101);
