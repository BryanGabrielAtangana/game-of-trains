//! The battle arena: a shared rail graph with per-faction routing toward the
//! enemy, plus towers.
//!
//! Layout is a symmetric "ladder": `rows` rows of `cols` lane-nodes. Faction `A`
//! travels from row 0 up to the top and into B's King; faction `B` travels the
//! mirror. Both factions share the same node *positions* (so their trains can
//! collide), but each has its own routing edges pointing at the enemy — at a node
//! with two exits, that faction's owner chooses the lane (a switch).

use super::Faction;
use crate::geometry::Pos;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Index into [`Arena::nodes`].
pub type NodeId = usize;

/// Tunables for a match. Fully determines arena generation and pacing.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BattleConfig {
    pub seed: u64,
    /// Lanes across.
    pub cols: usize,
    /// Junction rows between the two home rows (depth of the field).
    pub rows: usize,
    /// Simulation ticks executed per resolved turn.
    pub ticks_per_turn: u32,
    /// Steam available on turn 1 and the per-turn regen / cap.
    pub steam_start: u32,
    pub steam_per_turn: u32,
    pub steam_cap: u32,
    pub king_hp: i32,
    pub side_tower_hp: i32,
    /// King tower fire (squared range is compared; damage is per tick).
    pub king_range: i32,
    pub king_damage: i32,
    /// Side tower fire.
    pub side_range: i32,
    pub side_damage: i32,
    /// Hard cap on turns before sudden-death/draw handling (kept simple here).
    pub max_turns: u32,
}

impl Default for BattleConfig {
    fn default() -> Self {
        BattleConfig {
            seed: 1,
            cols: 3,
            rows: 4,
            ticks_per_turn: 30,
            steam_start: 6,
            steam_per_turn: 8,
            steam_cap: 24,
            king_hp: 100,
            side_tower_hp: 50,
            // Towers defend but no longer melt single-file streams instantly
            // (balance pass; see docs/design/rail-royale.md). Rockets (range 5)
            // keep their 1-tile siege edge over the King (range 4).
            king_range: 4,
            king_damage: 2,
            side_range: 5,
            side_damage: 2,
            max_turns: 24,
        }
    }
}

/// What a tower is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TowerKind {
    /// Destroy it to win.
    King,
    /// Gates a lane; defensive.
    Side,
}

/// A tower belonging to a faction, sitting on a node and shooting nearby enemies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Tower {
    pub faction: Faction,
    pub kind: TowerKind,
    pub node: NodeId,
    pub hp: i32,
    pub range: i32,
    pub damage: i32,
}

impl Tower {
    pub fn alive(&self) -> bool {
        self.hp > 0
    }
}

/// A node in the graph: a position plus, per faction, the exits toward the enemy.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NodeData {
    pub pos: Pos,
    /// `exits[faction.index()]` = nodes this faction may route to from here.
    /// 0 exits = terminal (an enemy King node); 2 = a switchable junction.
    pub exits: [Vec<NodeId>; 2],
}

/// The generated arena.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Arena {
    pub nodes: Vec<NodeData>,
    pub towers: Vec<Tower>,
    /// Entry nodes per faction (one per lane); deploys pick a lane.
    pub spawns: [Vec<NodeId>; 2],
    /// Each faction's own King node (the enemy aims here).
    pub kings: [NodeId; 2],
    pub cols: usize,
    pub rows: usize,
}

impl Arena {
    /// Build the symmetric ladder arena for a config.
    pub fn generate(cfg: &BattleConfig) -> Arena {
        let cols = cfg.cols.max(1);
        let rows = cfg.rows.max(1);
        let mut nodes: Vec<NodeData> = Vec::new();

        // Grid nodes: id = r*cols + c.
        let id = |r: usize, c: usize| r * cols + c;
        for r in 0..rows {
            for c in 0..cols {
                nodes.push(NodeData {
                    pos: Pos::new(c as i32 * 2, (r as i32 + 1) * 2),
                    exits: [Vec::new(), Vec::new()],
                });
            }
        }
        // King nodes, one row beyond each home row.
        let a_king = nodes.len();
        nodes.push(NodeData {
            pos: Pos::new(cols as i32 - 1, 0),
            exits: [Vec::new(), Vec::new()],
        });
        let b_king = nodes.len();
        nodes.push(NodeData {
            pos: Pos::new(cols as i32 - 1, (rows as i32 + 1) * 2),
            exits: [Vec::new(), Vec::new()],
        });

        let a = Faction::A.index();
        let b = Faction::B.index();
        // Faction A goes row r -> r+1 (and into B's king at the top).
        for r in 0..rows {
            for c in 0..cols {
                let exits = if r + 1 < rows {
                    dedup_two(id(r + 1, c), id(r + 1, (c + 1) % cols))
                } else {
                    vec![b_king]
                };
                nodes[id(r, c)].exits[a] = exits;
            }
        }
        // Faction B goes row r -> r-1 (and into A's king at the bottom).
        for r in 0..rows {
            for c in 0..cols {
                let exits = if r >= 1 {
                    dedup_two(id(r - 1, c), id(r - 1, (c + 1) % cols))
                } else {
                    vec![a_king]
                };
                nodes[id(r, c)].exits[b] = exits;
            }
        }

        // Towers: a King each, plus a side tower guarding each home row's flanks.
        let mut towers = vec![
            Tower {
                faction: Faction::A,
                kind: TowerKind::King,
                node: a_king,
                hp: cfg.king_hp,
                range: cfg.king_range,
                damage: cfg.king_damage,
            },
            Tower {
                faction: Faction::B,
                kind: TowerKind::King,
                node: b_king,
                hp: cfg.king_hp,
                range: cfg.king_range,
                damage: cfg.king_damage,
            },
        ];
        if cols >= 2 {
            for (f, r) in [(Faction::A, 0usize), (Faction::B, rows - 1)] {
                towers.push(Tower {
                    faction: f,
                    kind: TowerKind::Side,
                    node: id(r, 0),
                    hp: cfg.side_tower_hp,
                    range: cfg.side_range,
                    damage: cfg.side_damage,
                });
            }
        }

        let spawns_a: Vec<NodeId> = (0..cols).map(|c| id(0, c)).collect();
        let spawns_b: Vec<NodeId> = (0..cols).map(|c| id(rows - 1, c)).collect();

        Arena {
            nodes,
            towers,
            spawns: [spawns_a, spawns_b],
            kings: [a_king, b_king],
            cols,
            rows,
        }
    }

    /// The exit a faction takes from `node` given a switch `choice`, or `None` at
    /// a terminal. `choice` is clamped to the available exits.
    pub fn route(&self, faction: Faction, node: NodeId, choice: usize) -> Option<NodeId> {
        let exits = &self.nodes[node].exits[faction.index()];
        match exits.len() {
            0 => None,
            n => Some(exits[choice.min(n - 1)]),
        }
    }

    /// Whether a faction can choose a route at `node` (a real switch).
    pub fn is_switch(&self, faction: Faction, node: NodeId) -> bool {
        self.nodes[node].exits[faction.index()].len() >= 2
    }

    pub fn pos(&self, node: NodeId) -> Pos {
        self.nodes[node].pos
    }
}

/// Two distinct exits, deduped to one when they coincide (e.g. a single lane).
fn dedup_two(x: NodeId, y: NodeId) -> Vec<NodeId> {
    if x == y {
        vec![x]
    } else {
        vec![x, y]
    }
}

/// Squared grid distance between two nodes — integer, for deterministic ranges.
pub fn dist2(a: Pos, b: Pos) -> i64 {
    let dx = (a.x - b.x) as i64;
    let dy = (a.y - b.y) as i64;
    dx * dx + dy * dy
}
