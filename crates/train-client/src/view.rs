//! Layout (grid → screen mapping) and all the drawing primitives.

use crate::theme;
use macroquad::prelude::*;
use train_core::{Map, NodeKind, Pos, Simulation, Train};

/// Maps the engine's integer grid coordinates onto pixels, fitting the whole
/// map into a given screen rectangle with a uniform scale.
pub struct Layout {
    cell: f32,
    ox: f32,
    oy: f32,
}

impl Layout {
    pub fn fit(map: &Map, area: Rect) -> Layout {
        let span_x = (map.width - 1).max(1) as f32;
        let span_y = (map.height - 1).max(1) as f32;
        // Leave a margin so houses/labels near the edges aren't clipped.
        let margin = 48.0;
        let avail_w = (area.w - 2.0 * margin).max(1.0);
        let avail_h = (area.h - 2.0 * margin).max(1.0);
        let cell = (avail_w / span_x).min(avail_h / span_y).clamp(18.0, 130.0);
        let content_w = span_x * cell;
        let content_h = span_y * cell;
        Layout {
            cell,
            ox: area.x + (area.w - content_w) / 2.0,
            oy: area.y + (area.h - content_h) / 2.0,
        }
    }

    pub fn cell(&self) -> f32 {
        self.cell
    }

    /// Screen position of a grid point.
    pub fn pt(&self, p: Pos) -> Vec2 {
        vec2(
            self.ox + p.x as f32 * self.cell,
            self.oy + p.y as f32 * self.cell,
        )
    }
}

/// Draw the static world: tracks, switches, stations, dead-ends, the root.
pub fn draw_map(sim: &Simulation, layout: &Layout, time: f32) {
    let map = sim.map();
    let cell = layout.cell();

    // --- edges (tracks) ---
    for (i, node) in map.nodes.iter().enumerate() {
        let from = layout.pt(node.pos);
        let active_child = map.next(i, sim.switches()[i]);
        for &child in &node.children {
            let to = layout.pt(map.nodes[child].pos);
            let is_active = active_child == Some(child);
            // Inactive branch of a switch is drawn dim; everything else solid.
            let (color, thick) = if node.is_switch() && !is_active {
                (theme::TRACK_DIM, cell * 0.10)
            } else {
                (theme::TRACK, cell * 0.16)
            };
            draw_line(from.x, from.y, to.x, to.y, thick, color);
            if is_active {
                // A thin bright rail on top of the active route.
                draw_line(
                    from.x,
                    from.y,
                    to.x,
                    to.y,
                    (cell * 0.05).max(2.0),
                    theme::RAIL_ACTIVE,
                );
            }
        }
    }

    // --- nodes ---
    for (i, node) in map.nodes.iter().enumerate() {
        let p = layout.pt(node.pos);
        match node.kind {
            NodeKind::Station { label } => draw_house(p, cell, &label.to_string()),
            NodeKind::DeadEnd => draw_dead_end(p, cell),
            NodeKind::Track => {
                if i == map.root {
                    // Gently pulsing origin.
                    let r = cell * 0.22 * (1.0 + 0.12 * (time * 3.0).sin());
                    draw_circle(p.x, p.y, r, theme::ROOT);
                    draw_circle_lines(p.x, p.y, r, 2.0, theme::WHITE);
                } else if node.is_switch() {
                    draw_switch(p, cell, layout, sim, i);
                }
            }
        }
    }
}

fn draw_switch(p: Vec2, cell: f32, layout: &Layout, sim: &Simulation, i: usize) {
    let map = sim.map();
    let active = sim.switches()[i];
    let color = if active {
        theme::SWITCH_ALT
    } else {
        theme::SWITCH
    };
    let r = cell * 0.20;
    draw_circle(p.x, p.y, r, color);
    draw_circle_lines(p.x, p.y, r, 2.0, theme::WHITE);
    // A short stub pointing at the currently selected child, hinting direction.
    if let Some(child) = map.next(i, active) {
        let c = layout.pt(map.nodes[child].pos);
        let dir = (c - p).normalize_or_zero();
        let tip = p + dir * (r + cell * 0.12);
        draw_line(p.x, p.y, tip.x, tip.y, 3.0, theme::WHITE);
    }
}

fn draw_house(p: Vec2, cell: f32, label: &str) {
    let w = cell * 0.6;
    let body = w * 0.8;
    let x = p.x - w / 2.0;
    let y = p.y - body / 2.0;
    // roof
    draw_triangle(
        vec2(x - w * 0.08, y),
        vec2(x + w + w * 0.08, y),
        vec2(p.x, y - w * 0.5),
        theme::HOUSE_ROOF,
    );
    // body
    draw_rectangle(x, y, w, body, theme::HOUSE);
    draw_rectangle_lines(x, y, w, body, 2.0, theme::HOUSE_ROOF);
    centered_text(
        label,
        p.x,
        y + body * 0.5,
        (cell * 0.42) as u16,
        theme::WHITE,
    );
}

fn draw_dead_end(p: Vec2, cell: f32) {
    let r = cell * 0.22;
    draw_circle(p.x, p.y, r, theme::DEAD_END);
    let d = r * 0.55;
    draw_line(p.x - d, p.y - d, p.x + d, p.y + d, 3.0, theme::WHITE);
    draw_line(p.x - d, p.y + d, p.x + d, p.y - d, 3.0, theme::WHITE);
}

/// Draw the live trains, interpolated along their current edge.
pub fn draw_trains(sim: &Simulation, layout: &Layout) {
    let map = sim.map();
    let cell = layout.cell();
    for t in sim.trains() {
        let pos = train_pos(map, layout, t);
        let s = cell * 0.34;
        draw_rectangle(pos.x - s / 2.0, pos.y - s / 2.0, s, s, theme::TRAIN);
        draw_rectangle_lines(pos.x - s / 2.0, pos.y - s / 2.0, s, s, 2.0, theme::INK);
        centered_text(
            &t.dest.to_string(),
            pos.x,
            pos.y,
            (cell * 0.30) as u16,
            theme::WHITE,
        );
    }
}

fn train_pos(map: &Map, layout: &Layout, t: &Train) -> Vec2 {
    let from = layout.pt(map.nodes[t.from].pos);
    match t.to {
        Some(to) => from.lerp(layout.pt(map.nodes[to].pos), t.fraction()),
        None => from,
    }
}

/// Draw text horizontally and vertically centred on `(cx, cy)`.
pub fn centered_text(text: &str, cx: f32, cy: f32, size: u16, color: Color) {
    let dims = measure_text(text, None, size, 1.0);
    draw_text(
        text,
        cx - dims.width / 2.0,
        cy + dims.height / 2.0,
        size as f32,
        color,
    );
}
