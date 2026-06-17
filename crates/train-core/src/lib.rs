//! # train-core
//!
//! The deterministic engine behind **Game of Trains**. It knows how to:
//!
//! * generate a map from a seed ([`map::Map`]),
//! * schedule and simulate trains tick-by-tick ([`sim::Simulation`]),
//! * score deliveries and combos ([`score::Scorer`]), and
//! * record and **verify** a whole play-through ([`replay::Run`], [`replay::verify`]).
//!
//! It contains **no I/O, no rendering, and no platform-specific code**, so the
//! very same logic compiles to WebAssembly for the browser client *and* runs
//! natively inside the server. Because every result is a pure function of
//! `(seed, level, inputs)`, the server can re-simulate any submitted run and
//! trust only the score it computes itself — the foundation for cheat-resistant
//! global leaderboards.
//!
//! ## Tour
//! ```
//! use train_core::{GameConfig, Simulation};
//!
//! let config = GameConfig::new(/* seed */ 1234, /* level */ 3);
//! let mut sim = Simulation::new(&config);
//!
//! // A real client would render between steps and toggle switches on tap.
//! sim.run_to_end(5_000_000);
//! println!("score: {}", sim.scorer().score);
//! ```

#![forbid(unsafe_code)]

pub mod config;
pub mod geometry;
pub mod map;
pub mod replay;
pub mod rng;
pub mod score;
pub mod sim;

// A small, curated prelude of the types most callers need.
pub use config::{GameConfig, TICKS_PER_SECOND};
pub use geometry::{Direction, Pos};
pub use map::{Map, Node, NodeKind};
pub use replay::{verify, Input, RejectReason, Run, Verified};
pub use score::{Outcome, Scorer};
pub use sim::{generate_schedule, Simulation, Train, TrainSpawn};

/// Derive the deterministic daily seed for a given calendar date (UTC).
///
/// Everyone who plays the daily challenge on the same date gets the same map and
/// train schedule, which is what makes the daily leaderboard and streaks fair.
/// The mixing is just SplitMix64 over a packed `YYYYMMDD`, so it is stable across
/// platforms and trivially reproducible.
pub fn daily_seed(year: i32, month: u32, day: u32) -> u64 {
    let packed = (year as u64) << 16 | (month as u64) << 8 | day as u64;
    let mut z = packed.wrapping_add(0x9E3779B97F4A7C15);
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
        assert_ne!(daily_seed(2026, 6, 17), daily_seed(2025, 6, 17));
    }

    #[test]
    fn end_to_end_via_prelude() {
        let cfg = GameConfig::new(daily_seed(2026, 6, 17), 4);
        let mut sim = Simulation::new(&cfg);
        sim.run_to_end(5_000_000);
        assert!(sim.is_finished());
        assert_eq!(sim.scorer().total, cfg.trains);
    }
}
