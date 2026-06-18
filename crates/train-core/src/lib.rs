//! # train-core — the Rail Royale engine
//!
//! A deterministic, integer-tick battle engine. Two factions commit [`Orders`]
//! each turn (deploy trains, set routing switches); [`resolve_turn`] runs a fixed
//! number of ticks — movement, shooting, collisions — and returns the next
//! [`BattleState`]. Destroy the enemy King tower to win.
//!
//! It contains **no I/O, no rendering, and no platform-specific code**, so the
//! same logic compiles to WebAssembly for the client *and* runs natively in the
//! match server. Because every result is a pure function of the config and the
//! committed orders, a server can re-simulate a match to validate it — outcomes
//! can't be faked. (The previous daily-routing puzzle is archived under
//! [`legacy/`](https://github.com/BryanGabrielAtangana/game-of-trains/tree/main/legacy).)
//!
//! ## Tour
//! ```
//! use train_core::{BattleConfig, BattleState, Orders, TrainKind};
//!
//! let mut state = BattleState::new(BattleConfig::default());
//! // Faction A sends an Armored train up lane 0; B holds.
//! let a = Orders::new().deploy(TrainKind::Armored, 0);
//! let b = Orders::new();
//! train_core::resolve_turn(&mut state, &a, &b);
//! assert!(state.turn == 1);
//! ```

#![forbid(unsafe_code)]

pub mod battle;
pub mod geometry;
pub mod rng;

pub use battle::{
    resolve_turn, Arena, BattleConfig, BattleState, Command, Faction, NodeId, Orders, Status,
    Tower, TowerKind, Train, TrainKind, TrainStats, TurnEvent,
};
pub use geometry::{Direction, Pos};

/// Derive the deterministic daily seed for a calendar date (UTC).
///
/// Used to hand both players the same fair, mirrored arena for a day's matches.
/// SplitMix64 over a packed `YYYYMMDD`; stable across platforms.
pub fn daily_seed(year: i32, month: u32, day: u32) -> u64 {
    let packed = (year as u64) << 16 | (month as u64) << 8 | day as u64;
    splitmix64(packed)
}

/// Derive the deterministic daily seed from a Unix timestamp (seconds): keys on
/// the integer day index `floor(secs / 86400)`, so everything in the same UTC day
/// shares a seed and consecutive days differ.
pub fn daily_seed_from_unix(unix_seconds: u64) -> u64 {
    let day_index = unix_seconds / 86_400;
    splitmix64(day_index ^ 0x6461795F696E6478) // "day_indx"
}

/// SplitMix64 finalizer — stable across platforms (wrapping integer math only).
fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daily_seed_is_stable_and_date_specific() {
        assert_eq!(daily_seed(2026, 6, 17), daily_seed(2026, 6, 17));
        assert_ne!(daily_seed(2026, 6, 17), daily_seed(2026, 6, 18));
    }

    #[test]
    fn daily_seed_from_unix_is_per_day() {
        let day = 1_750_000_000u64;
        assert_eq!(daily_seed_from_unix(day), daily_seed_from_unix(day + 3600));
        assert_ne!(
            daily_seed_from_unix(day),
            daily_seed_from_unix(day + 86_400)
        );
    }

    #[test]
    fn end_to_end_via_prelude() {
        let mut state = BattleState::new(BattleConfig::default());
        let a = Orders::new().deploy(TrainKind::Armored, 0);
        let b = Orders::new();
        resolve_turn(&mut state, &a, &b);
        assert_eq!(state.turn, 1);
        assert!(matches!(state.status, Status::Ongoing | Status::Won(_)));
    }
}
