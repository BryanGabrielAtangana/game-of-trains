//! The display list: a tiny, serializable set of 2D draw primitives. The Rust
//! side builds a `Scene`; the JS loader executes each op on a `<canvas>` 2D
//! context. Keeping the protocol this small is what lets the whole game — logic,
//! layout, *and* rendering decisions — live in Rust with a ~60-line JS shim.
//!
//! All coordinates are in CSS pixels (the loader applies the device-pixel-ratio
//! transform), so Rust thinks in plain on-screen units.

use serde::Serialize;

/// One primitive to draw, in back-to-front order.
#[derive(Serialize)]
#[serde(tag = "op")]
pub enum DrawOp {
    /// Filled (optionally rounded) rectangle.
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        fill: String,
        round: f32,
    },
    /// Filled circle with an optional stroke.
    Circle {
        x: f32,
        y: f32,
        r: f32,
        fill: String,
        stroke: String,
        lw: f32,
    },
    /// Straight line segment.
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        stroke: String,
        lw: f32,
    },
    /// A single line of text. `align` is `left|center|right`; `baseline` is
    /// `top|middle|alphabetic`.
    Text {
        x: f32,
        y: f32,
        text: String,
        size: f32,
        fill: String,
        align: &'static str,
        baseline: &'static str,
        weight: &'static str,
    },
}

/// An ordered list of draw operations for one frame.
#[derive(Serialize, Default)]
pub struct Scene {
    pub ops: Vec<DrawOp>,
}

impl Scene {
    pub fn new() -> Self {
        Scene { ops: Vec::new() }
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, fill: &str, round: f32) {
        self.ops.push(DrawOp::Rect {
            x,
            y,
            w,
            h,
            fill: fill.to_string(),
            round,
        });
    }

    pub fn circle(&mut self, x: f32, y: f32, r: f32, fill: &str, stroke: &str, lw: f32) {
        self.ops.push(DrawOp::Circle {
            x,
            y,
            r,
            fill: fill.to_string(),
            stroke: stroke.to_string(),
            lw,
        });
    }

    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, stroke: &str, lw: f32) {
        self.ops.push(DrawOp::Line {
            x1,
            y1,
            x2,
            y2,
            stroke: stroke.to_string(),
            lw,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn text(
        &mut self,
        x: f32,
        y: f32,
        text: &str,
        size: f32,
        fill: &str,
        align: &'static str,
        baseline: &'static str,
        weight: &'static str,
    ) {
        self.ops.push(DrawOp::Text {
            x,
            y,
            text: text.to_string(),
            size,
            fill: fill.to_string(),
            align,
            baseline,
            weight,
        });
    }
}
