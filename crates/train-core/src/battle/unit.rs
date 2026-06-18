//! Train types, their stats, and live train instances.

use super::arena::NodeId;
use super::Faction;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The deployable train types. The counter-triangle (see the design doc) keeps
/// any single pick from dominating.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TrainKind {
    /// Fast, fragile; first to a junction, good for contesting routes.
    Express,
    /// Slow, very tanky; wins collisions and soaks tower fire.
    Armored,
    /// Slow, fragile, hits at range; softens tanks and towers from safety.
    Rocket,
}

impl TrainKind {
    /// Every selectable kind, in a stable order.
    pub const ALL: [TrainKind; 3] = [TrainKind::Express, TrainKind::Armored, TrainKind::Rocket];

    /// Base stats for this kind. All integers, for cross-platform determinism.
    pub fn stats(self) -> TrainStats {
        match self {
            // hp, dmg, range(0 = melee), edge_ticks(lower = faster), cost
            TrainKind::Express => TrainStats::new(20, 8, 0, 6, 3),
            TrainKind::Armored => TrainStats::new(80, 10, 0, 12, 5),
            TrainKind::Rocket => TrainStats::new(24, 18, 5, 10, 5),
        }
    }
}

/// Fixed per-kind stats. `range` is in grid units (squared distance is compared);
/// `range == 0` means melee (damage only on collision / at a tower node).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TrainStats {
    pub hp: i32,
    pub damage: i32,
    pub range: i32,
    pub edge_ticks: u32,
    pub cost: u32,
}

impl TrainStats {
    pub const fn new(hp: i32, damage: i32, range: i32, edge_ticks: u32, cost: u32) -> Self {
        TrainStats {
            hp,
            damage,
            range,
            edge_ticks,
            cost,
        }
    }
}

/// A train on the board. Movement mirrors the (retired) puzzle model: it occupies
/// the edge `from -> to`, advancing one tick at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Train {
    pub faction: Faction,
    pub kind: TrainKind,
    pub hp: i32,
    pub from: NodeId,
    /// Next node, or `None` once it has reached a terminal (enemy King).
    pub to: Option<NodeId>,
    pub progress: u32,
    pub edge_ticks: u32,
}

impl Train {
    /// Progress along the current edge in `[0.0, 1.0]`, for the renderer.
    pub fn fraction(&self) -> f32 {
        if self.edge_ticks == 0 {
            1.0
        } else {
            self.progress as f32 / self.edge_ticks as f32
        }
    }

    pub fn alive(&self) -> bool {
        self.hp > 0
    }
}
