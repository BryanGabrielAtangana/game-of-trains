//! Rail Royale — the deterministic battle engine (Phase 1, no UI).
//!
//! Two factions push **armed trains** across a shared rail graph to destroy the
//! enemy King tower. Play is **commit-and-resolve**: each turn both players submit
//! [`Orders`] (deploy trains, set their routing switches), then
//! [`resolve_turn`](resolve::resolve_turn) runs a fixed number of integer ticks
//! and returns the next [`BattleState`]. Like the rest of `train-core` it is pure,
//! deterministic and unit-tested, so a server can re-simulate a match to validate
//! it — no outcome can be faked.
//!
//! Phase-1 simplifications (tracked in `docs/design/rail-royale.md`): each faction
//! routes *its own* trains (switch ownership is per-faction, not yet contested),
//! and combat math is intentionally small so it stays easy to balance.

mod ai;
mod arena;
mod orders;
mod resolve;
mod state;
mod unit;

pub use ai::{ai_orders, AiLevel};
pub use arena::{Arena, BattleConfig, NodeId, Tower, TowerKind};
pub use orders::{Command, Orders};
pub use resolve::{resolve_turn, resolve_turn_frames, TurnEvent};
pub use state::{BattleState, Status};
pub use unit::{Train, TrainKind, TrainStats};

/// The two sides. `A` starts at the bottom of the arena, `B` at the top.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Faction {
    A,
    B,
}

impl Faction {
    /// The opposing faction.
    pub fn enemy(self) -> Faction {
        match self {
            Faction::A => Faction::B,
            Faction::B => Faction::A,
        }
    }

    /// Stable index `0`/`1` for per-faction arrays.
    pub fn index(self) -> usize {
        match self {
            Faction::A => 0,
            Faction::B => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small single-lane arena makes outcomes easy to reason about.
    fn duel_cfg() -> BattleConfig {
        BattleConfig {
            cols: 1,
            rows: 3,
            ticks_per_turn: 30,
            steam_start: 10,
            steam_per_turn: 10,
            steam_cap: 20,
            king_hp: 40,
            max_turns: 30,
            ..BattleConfig::default()
        }
    }

    #[test]
    fn frames_match_plain_resolve_and_are_per_tick() {
        // resolve_turn_frames must leave the state identical to resolve_turn, and
        // hand back one frame per tick plus the initial post-deploy snapshot.
        let cfg = duel_cfg();
        let a = Orders::new().deploy(TrainKind::Express, 0);
        let b = Orders::new().deploy(TrainKind::Armored, 0);

        let mut plain = BattleState::new(cfg.clone());
        resolve_turn(&mut plain, &a, &b);

        let mut traced = BattleState::new(cfg.clone());
        let (_events, frames) = resolve_turn_frames(&mut traced, &a, &b);

        assert_eq!(plain, traced, "frames variant must not change the outcome");
        assert_eq!(
            frames.len() as u32,
            cfg.ticks_per_turn + 1,
            "one frame per tick + the initial post-deploy frame"
        );
        // The first frame is before movement: trains sit at progress 0.
        assert!(frames[0].trains.iter().all(|t| t.progress == 0));
    }

    #[test]
    fn fresh_state_is_sane() {
        let s = BattleState::new(BattleConfig::default());
        assert_eq!(s.king_hp(Faction::A), s.cfg.king_hp);
        assert_eq!(s.king_hp(Faction::B), s.cfg.king_hp);
        assert!(s.trains.is_empty());
        assert_eq!(s.status, Status::Ongoing);
    }

    #[test]
    fn deterministic_replay() {
        let script = |s: &mut BattleState| {
            for _ in 0..6 {
                let a = Orders::new().deploy(TrainKind::Express, 0);
                let b = Orders::new().deploy(TrainKind::Armored, 0);
                resolve_turn(s, &a, &b);
            }
        };
        let mut s1 = BattleState::new(duel_cfg());
        let mut s2 = BattleState::new(duel_cfg());
        script(&mut s1);
        script(&mut s2);
        assert_eq!(s1, s2);
    }

    #[test]
    fn rockets_destroy_the_king_and_win() {
        // Rockets out-range the King tower and shell it down -> Won(A).
        let mut s = BattleState::new(duel_cfg());
        let start = s.king_hp(Faction::B);
        let mut won = false;
        for _ in 0..s.cfg.max_turns {
            let a = Orders::new().deploy(TrainKind::Rocket, 0);
            resolve_turn(&mut s, &a, &Orders::new());
            if s.status == Status::Won(Faction::A) {
                won = true;
                break;
            }
        }
        assert!(won, "rockets should break through an idle B");
        assert!(s.king_hp(Faction::B) < start);
    }

    #[test]
    fn head_on_collision_favors_armored() {
        // Meet head-on in the midfield (out of tower range) in a single short
        // turn: the fragile Express dies, the Armored survives the collision.
        let cfg = BattleConfig {
            cols: 1,
            rows: 5,
            ticks_per_turn: 18,
            steam_start: 10,
            ..duel_cfg()
        };
        let mut s = BattleState::new(cfg);
        resolve_turn(
            &mut s,
            &Orders::new().deploy(TrainKind::Express, 0),
            &Orders::new().deploy(TrainKind::Armored, 0),
        );
        let a = s.trains.iter().filter(|t| t.faction == Faction::A).count();
        let b = s.trains.iter().filter(|t| t.faction == Faction::B).count();
        assert_eq!(a, 0, "Express should lose the head-on");
        assert_eq!(b, 1, "Armored should survive the collision");
    }

    #[test]
    fn steam_caps_deploys() {
        // steam_start 10; Armored costs 5 -> at most 2 spawn on turn 1.
        let cfg = BattleConfig {
            steam_start: 10,
            steam_per_turn: 0,
            ..duel_cfg()
        };
        let mut s = BattleState::new(cfg);
        let a = Orders::new()
            .deploy(TrainKind::Armored, 0)
            .deploy(TrainKind::Armored, 0)
            .deploy(TrainKind::Armored, 0);
        // Resolve a turn with no enemy and read how many actually spawned by
        // counting A trains still alive immediately (1 tick of travel can't kill).
        resolve_turn(&mut s, &a, &Orders::new());
        let a_trains = s.trains.iter().filter(|t| t.faction == Faction::A).count();
        assert!(a_trains <= 2, "spawned {a_trains}, steam should allow <= 2");
    }

    #[test]
    fn switch_sets_initial_route() {
        // Default arena has 3 lanes -> spawn node (0,0) is a real switch for A.
        // Use ticks_per_turn = 0 so no movement/combat: just inspect the route.
        let cfg = BattleConfig {
            ticks_per_turn: 0,
            ..BattleConfig::default()
        };
        let spawn = BattleState::new(cfg.clone()).arena.spawns[Faction::A.index()][0];
        assert!(BattleState::new(cfg.clone())
            .arena
            .is_switch(Faction::A, spawn));

        let mut s0 = BattleState::new(cfg.clone());
        resolve_turn(
            &mut s0,
            &Orders::new().switch(spawn, 0).deploy(TrainKind::Express, 0),
            &Orders::new(),
        );
        let mut s1 = BattleState::new(cfg);
        resolve_turn(
            &mut s1,
            &Orders::new().switch(spawn, 1).deploy(TrainKind::Express, 0),
            &Orders::new(),
        );
        assert_ne!(
            s0.trains[0].to, s1.trains[0].to,
            "different switch choices should pick different first nodes"
        );
    }
}
