//! The whole client game: state machine (menu → plan → resolve → game over),
//! responsive layout, tap hit-testing, and rendering into a [`Scene`]. All of it
//! is plain Rust driven by [`train_core`]; the JS shim only forwards taps/ticks
//! and paints the resulting display list.

use train_core::{
    ai_orders, resolve_turn_frames, AiLevel, BattleConfig, BattleState, Faction, Orders, Status,
    TowerKind, Train, TrainKind,
};

use crate::scene::Scene;

// --- palette (kept close to the placeholder page) -------------------------
const BG: &str = "#16222b";
const PANEL: &str = "#1f3340";
const INK: &str = "#0e171d";
const WHITE: &str = "#f4f9fb";
const MUTE: &str = "#9fb3bd";
const TRACK: &str = "#3a5260";
const RAIL_ACTIVE: &str = "#ffce54";
const STEAM: &str = "#ffce54";
const A_MAIN: &str = "#4fc1e9"; // you
const A_DEEP: &str = "#2f8fb8";
const B_MAIN: &str = "#ed5565"; // enemy AI
const B_DEEP: &str = "#c2384a";
const GOOD: &str = "#a0d468";

const FRAME_MS: f32 = 38.0; // animation speed: ms per simulated tick

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Menu,
    Planning,
    Resolving,
    GameOver,
}

#[derive(Clone, Copy)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Rect {
    fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }
    fn cx(&self) -> f32 {
        self.x + self.w / 2.0
    }
    fn cy(&self) -> f32 {
        self.y + self.h / 2.0
    }
}

struct Layout {
    board: Rect,
    node_pos: Vec<(f32, f32)>,
    node_r: f32,
    top: Rect,
    bottom: Rect,
    kind_btns: [Rect; 3],
    go: Rect,
    centre_btns: Vec<Rect>, // menu difficulty / play-again
}

pub struct Client {
    cfg: BattleConfig,
    state: BattleState,
    difficulty: AiLevel,
    phase: Phase,
    w: f32,
    h: f32,
    // planning
    selected: usize, // index into TrainKind::ALL
    plan: Vec<(TrainKind, usize)>,
    switches_a: Vec<u8>,
    preview_steam: u32,
    // resolving
    frames: Vec<BattleState>,
    anim_t: f32,
    // world bounds
    wx: (f32, f32),
    wy: (f32, f32),
}

impl Client {
    pub fn new(w: f32, h: f32) -> Self {
        let cfg = BattleConfig::default();
        let state = BattleState::new(cfg.clone());
        let (wx, wy) = world_bounds(&state);
        Client {
            cfg,
            state,
            difficulty: AiLevel::Normal,
            phase: Phase::Menu,
            w,
            h,
            selected: 1, // Armored
            plan: Vec::new(),
            switches_a: Vec::new(),
            preview_steam: 0,
            frames: Vec::new(),
            anim_t: 0.0,
            wx,
            wy,
        }
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        self.w = w;
        self.h = h;
    }

    fn start_match(&mut self) {
        self.state = BattleState::new(self.cfg.clone());
        let (wx, wy) = world_bounds(&self.state);
        self.wx = wx;
        self.wy = wy;
        self.begin_planning();
    }

    fn begin_planning(&mut self) {
        self.phase = Phase::Planning;
        self.selected = 1;
        self.plan.clear();
        self.switches_a = self.state.switches[Faction::A.index()].clone();
        self.preview_steam = self.state.steam[Faction::A.index()];
    }

    fn commit(&mut self) {
        // Switches first (so deploys read the chosen route), then deploys.
        let mut orders = Orders::new();
        for node in 0..self.state.arena.nodes.len() {
            if self.state.arena.is_switch(Faction::A, node) {
                orders = orders.switch(node, self.switches_a[node]);
            }
        }
        for &(kind, lane) in &self.plan {
            orders = orders.deploy(kind, lane);
        }
        let ai = ai_orders(&self.state, Faction::B, self.difficulty);
        let (_events, frames) = resolve_turn_frames(&mut self.state, &orders, &ai);
        self.frames = frames;
        self.anim_t = 0.0;
        if self.frames.is_empty() {
            self.settle();
        } else {
            self.phase = Phase::Resolving;
        }
    }

    fn settle(&mut self) {
        self.frames.clear();
        if self.state.is_over() {
            self.phase = Phase::GameOver;
        } else {
            self.begin_planning();
        }
    }

    pub fn tick(&mut self, dt_ms: f32) {
        if self.phase != Phase::Resolving {
            return;
        }
        self.anim_t += dt_ms;
        let idx = (self.anim_t / FRAME_MS) as usize;
        if idx >= self.frames.len() {
            self.settle();
        }
    }

    pub fn pointer(&mut self, px: f32, py: f32) {
        let l = self.layout();
        match self.phase {
            Phase::Menu => {
                for (i, r) in l.centre_btns.iter().enumerate() {
                    if r.contains(px, py) {
                        self.difficulty = AiLevel::ALL[i];
                        self.start_match();
                        return;
                    }
                }
            }
            Phase::GameOver => {
                if let Some(r) = l.centre_btns.first() {
                    if r.contains(px, py) {
                        self.phase = Phase::Menu;
                    }
                }
            }
            Phase::Planning => self.pointer_planning(px, py, &l),
            Phase::Resolving => {} // taps ignored while the turn plays out
        }
    }

    fn pointer_planning(&mut self, px: f32, py: f32, l: &Layout) {
        // 1. Kind buttons.
        for (i, r) in l.kind_btns.iter().enumerate() {
            if r.contains(px, py) {
                self.selected = i;
                return;
            }
        }
        // 2. GO.
        if l.go.contains(px, py) {
            self.commit();
            return;
        }
        // 3. Board: a switch node toggles its route; otherwise deploy in a lane.
        if l.board.contains(px, py) {
            if let Some(node) = self.nearest_switch_node(px, py, l) {
                let count = self.state.arena.nodes[node].exits[Faction::A.index()].len() as u8;
                if count > 1 {
                    self.switches_a[node] = (self.switches_a[node] + 1) % count;
                    return;
                }
            }
            self.try_deploy(px, l);
        }
    }

    fn nearest_switch_node(&self, px: f32, py: f32, l: &Layout) -> Option<usize> {
        let mut best: Option<(f32, usize)> = None;
        for node in 0..self.state.arena.nodes.len() {
            if !self.state.arena.is_switch(Faction::A, node) {
                continue;
            }
            let (nx, ny) = l.node_pos[node];
            let d = (nx - px) * (nx - px) + (ny - py) * (ny - py);
            if d <= (l.node_r * 1.6) * (l.node_r * 1.6) && best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, node));
            }
        }
        best.map(|(_, n)| n)
    }

    fn try_deploy(&mut self, px: f32, l: &Layout) {
        let kind = TrainKind::ALL[self.selected];
        let cost = kind.stats().cost;
        if self.preview_steam < cost {
            return;
        }
        // Nearest spawn lane by x.
        let spawns = &self.state.arena.spawns[Faction::A.index()];
        let mut lane = 0usize;
        let mut best = f32::MAX;
        for (i, &node) in spawns.iter().enumerate() {
            let dx = (l.node_pos[node].0 - px).abs();
            if dx < best {
                best = dx;
                lane = i;
            }
        }
        self.plan.push((kind, lane));
        self.preview_steam -= cost;
    }

    // --- layout ----------------------------------------------------------
    fn layout(&self) -> Layout {
        let (w, h) = (self.w, self.h);
        let top_h = (h * 0.12).clamp(50.0, 96.0);
        let bot_h = (h * 0.22).clamp(132.0, 240.0);
        let board = Rect {
            x: 8.0,
            y: top_h,
            w: w - 16.0,
            h: (h - top_h - bot_h).max(80.0),
        };
        let arena = &self.state.arena;
        let cols = arena.cols.max(1) as f32;
        let rows = arena.rows.max(1) as f32;
        let cell = (board.w / cols).min(board.h / (rows + 1.0));
        let node_r = (cell * 0.20).clamp(7.0, 26.0);
        let pad = node_r * 1.9;

        let node_pos = arena
            .nodes
            .iter()
            .map(|n| {
                let fx = if self.wx.1 > self.wx.0 {
                    (n.pos.x as f32 - self.wx.0) / (self.wx.1 - self.wx.0)
                } else {
                    0.5
                };
                let fy = if self.wy.1 > self.wy.0 {
                    (n.pos.y as f32 - self.wy.0) / (self.wy.1 - self.wy.0)
                } else {
                    0.5
                };
                let sx = board.x + pad + fx * (board.w - 2.0 * pad);
                let sy = board.y + pad + (1.0 - fy) * (board.h - 2.0 * pad);
                (sx, sy)
            })
            .collect();

        // Bottom controls.
        let m = 10.0;
        let bottom = Rect {
            x: 0.0,
            y: h - bot_h,
            w,
            h: bot_h,
        };
        let steam_h = 22.0;
        let row_y = bottom.y + m + steam_h + m;
        let row_h = bottom.y + bot_h - m - row_y;
        let left_w = (w - 2.0 * m) * 0.72;
        let go_w = (w - 2.0 * m) * 0.24;
        let gap = 8.0;
        let btn_w = (left_w - 2.0 * gap) / 3.0;
        let kind_btns = [0, 1, 2].map(|i| Rect {
            x: m + i as f32 * (btn_w + gap),
            y: row_y,
            w: btn_w,
            h: row_h,
        });
        let go = Rect {
            x: w - m - go_w,
            y: row_y,
            w: go_w,
            h: row_h,
        };

        // Centre buttons (menu / game over), stacked.
        let cb_w = (w * 0.7).min(320.0);
        let cb_h = 56.0;
        let centre_btns = (0..3)
            .map(|i| Rect {
                x: (w - cb_w) / 2.0,
                y: h * 0.42 + i as f32 * (cb_h + 14.0),
                w: cb_w,
                h: cb_h,
            })
            .collect();

        Layout {
            board,
            node_pos,
            node_r,
            top: Rect {
                x: 0.0,
                y: 0.0,
                w,
                h: top_h,
            },
            bottom,
            kind_btns,
            go,
            centre_btns,
        }
    }

    // --- rendering -------------------------------------------------------
    pub fn render(&self) -> Scene {
        let mut s = Scene::new();
        let l = self.layout();
        s.rect(0.0, 0.0, self.w, self.h, BG, 0.0);

        match self.phase {
            Phase::Menu => self.render_menu(&mut s, &l),
            Phase::Planning | Phase::Resolving => {
                self.render_board(&mut s, &l);
                self.render_hud(&mut s, &l);
            }
            Phase::GameOver => {
                self.render_board(&mut s, &l);
                self.render_hud(&mut s, &l);
                self.render_gameover(&mut s, &l);
            }
        }
        s
    }

    fn view_state(&self) -> &BattleState {
        if self.phase == Phase::Resolving && !self.frames.is_empty() {
            let idx = ((self.anim_t / FRAME_MS) as usize).min(self.frames.len() - 1);
            &self.frames[idx]
        } else {
            &self.state
        }
    }

    fn render_board(&self, s: &mut Scene, l: &Layout) {
        let view = self.view_state();
        s.rect(l.board.x, l.board.y, l.board.w, l.board.h, PANEL, 14.0);

        // Track segments (dedup undirected pairs).
        let mut seen: Vec<(usize, usize)> = Vec::new();
        for n in 0..view.arena.nodes.len() {
            for f in 0..2 {
                for &m in &view.arena.nodes[n].exits[f] {
                    let pair = (n.min(m), n.max(m));
                    if !seen.contains(&pair) {
                        seen.push(pair);
                        let (ax, ay) = l.node_pos[pair.0];
                        let (bx, by) = l.node_pos[pair.1];
                        s.line(ax, ay, bx, by, TRACK, (l.node_r * 0.45).max(3.0));
                    }
                }
            }
        }

        // Player's chosen routes (planning feedback) highlighted.
        if self.phase == Phase::Planning {
            for n in 0..view.arena.nodes.len() {
                if view.arena.is_switch(Faction::A, n) {
                    let c = self.switches_a[n] as usize;
                    let m = view.arena.nodes[n].exits[Faction::A.index()][c];
                    let (ax, ay) = l.node_pos[n];
                    let (bx, by) = l.node_pos[m];
                    s.line(ax, ay, bx, by, RAIL_ACTIVE, (l.node_r * 0.35).max(2.0));
                }
            }
        }

        // Junction nodes.
        for n in 0..view.arena.nodes.len() {
            let (x, y) = l.node_pos[n];
            let is_sw = view.arena.is_switch(Faction::A, n);
            let r = if is_sw {
                l.node_r * 0.5
            } else {
                l.node_r * 0.34
            };
            let fill = if is_sw && self.phase == Phase::Planning {
                RAIL_ACTIVE
            } else {
                "#4a6675"
            };
            s.circle(x, y, r, fill, INK, 1.5);
        }

        // Towers.
        for tow in &view.towers {
            let (x, y) = l.node_pos[tow.node];
            let (deep, main) = faction_colors(tow.faction);
            let size = if tow.kind == TowerKind::King {
                l.node_r * 2.1
            } else {
                l.node_r * 1.4
            };
            s.rect(x - size / 2.0, y - size / 2.0, size, size, deep, 6.0);
            s.rect(
                x - size / 2.0 + 3.0,
                y - size / 2.0 + 3.0,
                size - 6.0,
                size - 6.0,
                main,
                4.0,
            );
            if tow.kind == TowerKind::King {
                s.text(x, y, "♚", size * 0.6, WHITE, "center", "middle", "bold");
            }
            // HP bar above.
            let max_hp = if tow.kind == TowerKind::King {
                view.cfg.king_hp
            } else {
                view.cfg.side_tower_hp
            };
            hp_bar(
                s,
                x,
                y - size / 2.0 - 7.0,
                size,
                tow.hp,
                max_hp,
                tow.faction,
            );
        }

        // Trains.
        for t in &view.trains {
            if !t.alive() {
                continue;
            }
            let (x, y) = train_xy(l, t);
            let (deep, main) = faction_colors(t.faction);
            let r = l.node_r * 0.62;
            s.circle(x, y, r, main, deep, 2.0);
            s.text(
                x,
                y,
                kind_glyph(t.kind),
                r * 1.15,
                INK,
                "center",
                "middle",
                "bold",
            );
        }

        // Planned deploys (ghosts) above each lane.
        if self.phase == Phase::Planning {
            let spawns = &self.state.arena.spawns[Faction::A.index()];
            let mut per_lane = vec![0u32; spawns.len()];
            for &(kind, lane) in &self.plan {
                let (sx, sy) = l.node_pos[spawns[lane]];
                let n = per_lane[lane];
                per_lane[lane] += 1;
                let gy = sy + l.node_r * 1.6 + n as f32 * (l.node_r * 1.1);
                s.circle(sx, gy, l.node_r * 0.5, A_MAIN, "#0e171d", 1.5);
                s.text(
                    sx,
                    gy,
                    kind_glyph(kind),
                    l.node_r * 0.6,
                    INK,
                    "center",
                    "middle",
                    "bold",
                );
            }
        }
    }

    fn render_hud(&self, s: &mut Scene, l: &Layout) {
        let view = self.view_state();
        // Top: enemy info.
        s.text(
            12.0,
            l.top.cy() - 9.0,
            &format!("Enemy AI · {}", level_name(self.difficulty)),
            16.0,
            WHITE,
            "left",
            "middle",
            "bold",
        );
        let bhp = view.king_hp(Faction::B);
        s.text(
            12.0,
            l.top.cy() + 11.0,
            &format!("King {} / {}", bhp.max(0), view.cfg.king_hp),
            13.0,
            B_MAIN,
            "left",
            "middle",
            "normal",
        );
        s.text(
            self.w - 12.0,
            l.top.cy(),
            &format!("Turn {} / {}", view.turn + 1, view.cfg.max_turns),
            14.0,
            MUTE,
            "right",
            "middle",
            "bold",
        );

        // Bottom panel.
        s.rect(l.bottom.x, l.bottom.y, l.bottom.w, l.bottom.h, PANEL, 14.0);

        // Steam preview bar.
        let steam = if self.phase == Phase::Planning {
            self.preview_steam
        } else {
            self.state.steam[Faction::A.index()]
        };
        let cap = self.cfg.steam_cap;
        let bx = 12.0;
        let bw = self.w - 24.0;
        let by = l.bottom.y + 10.0;
        s.rect(bx, by, bw, 16.0, "#0e171d", 6.0);
        let frac = (steam as f32 / cap.max(1) as f32).clamp(0.0, 1.0);
        s.rect(bx, by, bw * frac, 16.0, STEAM, 6.0);
        s.text(
            bx + 8.0,
            by + 8.0,
            &format!("⚙ Steam {steam}/{cap}"),
            12.0,
            INK,
            "left",
            "middle",
            "bold",
        );

        // Kind buttons.
        for (i, r) in l.kind_btns.iter().enumerate() {
            let kind = TrainKind::ALL[i];
            let cost = kind.stats().cost;
            let affordable = self.preview_steam >= cost || self.phase != Phase::Planning;
            let selected = i == self.selected && self.phase == Phase::Planning;
            let fill = if selected {
                A_DEEP
            } else if affordable {
                "#26414f"
            } else {
                "#1a2a33"
            };
            s.rect(r.x, r.y, r.w, r.h, fill, 10.0);
            if selected {
                s.rect(r.x, r.y, r.w, 4.0, A_MAIN, 10.0);
            }
            let txt = if affordable { WHITE } else { MUTE };
            s.text(
                r.cx(),
                r.cy() - 12.0,
                kind_glyph(kind),
                22.0,
                txt,
                "center",
                "middle",
                "bold",
            );
            s.text(
                r.cx(),
                r.cy() + 12.0,
                kind_name(kind),
                12.0,
                txt,
                "center",
                "middle",
                "normal",
            );
            s.text(
                r.cx(),
                r.cy() + 28.0,
                &format!("⚙{cost}"),
                12.0,
                STEAM,
                "center",
                "middle",
                "bold",
            );
        }

        // GO button.
        let go_fill = if self.phase == Phase::Planning {
            GOOD
        } else {
            "#3a5260"
        };
        s.rect(l.go.x, l.go.y, l.go.w, l.go.h, go_fill, 10.0);
        let go_label = if self.phase == Phase::Resolving {
            "…"
        } else {
            "GO ▶"
        };
        s.text(
            l.go.cx(),
            l.go.cy() - 8.0,
            go_label,
            18.0,
            INK,
            "center",
            "middle",
            "bold",
        );
        s.text(
            l.go.cx(),
            l.go.cy() + 14.0,
            &format!("{} planned", self.plan.len()),
            11.0,
            INK,
            "center",
            "middle",
            "normal",
        );
    }

    fn render_menu(&self, s: &mut Scene, l: &Layout) {
        s.text(
            self.w / 2.0,
            self.h * 0.20,
            "🚂⚔️",
            54.0,
            WHITE,
            "center",
            "middle",
            "bold",
        );
        s.text(
            self.w / 2.0,
            self.h * 0.20 + 60.0,
            "RAIL ROYALE",
            34.0,
            WHITE,
            "center",
            "middle",
            "bold",
        );
        s.text(
            self.w / 2.0,
            self.h * 0.20 + 96.0,
            "Route armed trains. Wreck the enemy King.",
            14.0,
            MUTE,
            "center",
            "middle",
            "normal",
        );
        s.text(
            self.w / 2.0,
            self.h * 0.38,
            "Choose your opponent",
            14.0,
            WHITE,
            "center",
            "middle",
            "bold",
        );
        for (i, r) in l.centre_btns.iter().enumerate() {
            let lvl = AiLevel::ALL[i];
            s.rect(r.x, r.y, r.w, r.h, A_DEEP, 12.0);
            s.text(
                r.cx(),
                r.cy() - 8.0,
                level_name(lvl),
                20.0,
                WHITE,
                "center",
                "middle",
                "bold",
            );
            s.text(
                r.cx(),
                r.cy() + 14.0,
                level_blurb(lvl),
                11.0,
                "#cfe6f1",
                "center",
                "middle",
                "normal",
            );
        }
    }

    fn render_gameover(&self, s: &mut Scene, l: &Layout) {
        s.rect(0.0, 0.0, self.w, self.h, "rgba(8,14,18,0.78)", 0.0);
        let (msg, color) = match self.state.status {
            Status::Won(Faction::A) => ("VICTORY", GOOD),
            Status::Won(Faction::B) => ("DEFEAT", B_MAIN),
            _ => ("DRAW", STEAM),
        };
        s.text(
            self.w / 2.0,
            self.h * 0.34,
            msg,
            46.0,
            color,
            "center",
            "middle",
            "bold",
        );
        s.text(
            self.w / 2.0,
            self.h * 0.34 + 40.0,
            &format!(
                "Your King {} · Enemy King {}",
                self.state.king_hp(Faction::A).max(0),
                self.state.king_hp(Faction::B).max(0)
            ),
            14.0,
            MUTE,
            "center",
            "middle",
            "normal",
        );
        if let Some(r) = l.centre_btns.first() {
            s.rect(r.x, r.y, r.w, r.h, A_DEEP, 12.0);
            s.text(
                r.cx(),
                r.cy(),
                "Play again",
                20.0,
                WHITE,
                "center",
                "middle",
                "bold",
            );
        }
    }
}

// --- free helpers --------------------------------------------------------

fn world_bounds(state: &BattleState) -> ((f32, f32), (f32, f32)) {
    let mut minx = f32::MAX;
    let mut maxx = f32::MIN;
    let mut miny = f32::MAX;
    let mut maxy = f32::MIN;
    for n in &state.arena.nodes {
        minx = minx.min(n.pos.x as f32);
        maxx = maxx.max(n.pos.x as f32);
        miny = miny.min(n.pos.y as f32);
        maxy = maxy.max(n.pos.y as f32);
    }
    ((minx, maxx), (miny, maxy))
}

fn train_xy(l: &Layout, t: &Train) -> (f32, f32) {
    let (fx, fy) = l.node_pos[t.from];
    match t.to {
        Some(to) => {
            let (tx, ty) = l.node_pos[to];
            let f = t.fraction();
            (fx + (tx - fx) * f, fy + (ty - fy) * f)
        }
        None => (fx, fy),
    }
}

fn hp_bar(s: &mut Scene, cx: f32, y: f32, w: f32, hp: i32, max: i32, f: Faction) {
    let w = w.max(22.0);
    let h = 4.5;
    let x = cx - w / 2.0;
    s.rect(x, y, w, h, "#0e171d", 2.0);
    let frac = (hp.max(0) as f32 / max.max(1) as f32).clamp(0.0, 1.0);
    let (_, main) = faction_colors(f);
    s.rect(x, y, w * frac, h, main, 2.0);
}

fn faction_colors(f: Faction) -> (&'static str, &'static str) {
    match f {
        Faction::A => (A_DEEP, A_MAIN),
        Faction::B => (B_DEEP, B_MAIN),
    }
}

fn kind_glyph(k: TrainKind) -> &'static str {
    match k {
        TrainKind::Express => "E",
        TrainKind::Armored => "A",
        TrainKind::Rocket => "R",
    }
}

fn kind_name(k: TrainKind) -> &'static str {
    match k {
        TrainKind::Express => "Express",
        TrainKind::Armored => "Armored",
        TrainKind::Rocket => "Rocket",
    }
}

fn level_name(l: AiLevel) -> &'static str {
    match l {
        AiLevel::Easy => "Easy",
        AiLevel::Normal => "Normal",
        AiLevel::Hard => "Hard",
    }
}

fn level_blurb(l: AiLevel) -> &'static str {
    match l {
        AiLevel::Easy => "one cheap unit a turn",
        AiLevel::Normal => "counters & defends",
        AiLevel::Hard => "counters, defends, routes",
    }
}
