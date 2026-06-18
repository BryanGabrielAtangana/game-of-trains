//! The mutable battle state carried between turns.

use super::arena::{Arena, BattleConfig, Tower, TowerKind};
use super::unit::Train;
use super::Faction;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Match status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Status {
    Ongoing,
    Won(Faction),
    Draw,
}

/// The full, deterministic state of a battle. Cloneable so a turn can be
/// re-simulated from any point (the basis for server verification).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BattleState {
    pub arena: Arena,
    /// Live towers (hp mutates); starts as a copy of `arena.towers`.
    pub towers: Vec<Tower>,
    pub trains: Vec<Train>,
    /// Steam available to each faction next plan phase.
    pub steam: [u32; 2],
    /// Persisted routing choice per faction per node (`switches[f][node]`).
    pub switches: [Vec<u8>; 2],
    pub tick: u32,
    pub turn: u32,
    pub status: Status,
    pub cfg: BattleConfig,
}

impl BattleState {
    /// Fresh battle from a config.
    pub fn new(cfg: BattleConfig) -> Self {
        let arena = Arena::generate(&cfg);
        let n = arena.nodes.len();
        let towers = arena.towers.clone();
        BattleState {
            arena,
            towers,
            trains: Vec::new(),
            steam: [cfg.steam_start, cfg.steam_start],
            switches: [vec![0u8; n], vec![0u8; n]],
            tick: 0,
            turn: 0,
            status: Status::Ongoing,
            cfg,
        }
    }

    /// HP of a faction's King tower (0 if somehow missing).
    pub fn king_hp(&self, faction: Faction) -> i32 {
        self.towers
            .iter()
            .find(|t| t.faction == faction && t.kind == TowerKind::King)
            .map(|t| t.hp)
            .unwrap_or(0)
    }

    pub fn is_over(&self) -> bool {
        !matches!(self.status, Status::Ongoing)
    }
}
