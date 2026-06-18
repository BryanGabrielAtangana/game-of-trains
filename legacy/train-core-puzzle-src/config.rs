//! Difficulty configuration.
//!
//! A [`GameConfig`] is *fully determined* by `(seed, level)`. The client and the
//! server both build it the same way, so the server only needs those two numbers
//! (plus the player's switch inputs) to reconstruct and verify an entire run.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Number of simulation ticks per second. Rendering interpolates between ticks.
pub const TICKS_PER_SECOND: u32 = 60;

/// Everything needed to generate a map and a train schedule, deterministically.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GameConfig {
    /// Master seed. Map, dead-ends, train destinations and timings all derive from it.
    pub seed: u64,
    /// Difficulty level (1 = gentlest). Endless mode increments this over time.
    pub level: u32,

    // --- Map generation ---
    /// Target depth of the track tree (more depth = bigger map, more stations).
    pub tree_height: u32,
    /// Percent chance (0..=100) that a branch extends straight before splitting.
    pub extend_probability: u32,
    /// Percent chance (0..=100) that a station is replaced by a dead-end trap.
    pub dead_end_probability: u32,
    /// Hard cap on intentional dead-ends so a map is never unfair.
    pub max_dead_ends: u32,

    // --- Train schedule ---
    /// How many trains will be dispatched this round.
    pub trains: u32,
    /// Allowed per-edge traversal times in ticks (smaller = faster). Picked per train.
    pub edge_ticks_choices: Vec<u32>,
    /// Allowed gaps (in edges) between consecutive train dispatches.
    pub interval_choices: Vec<u32>,
}

impl GameConfig {
    /// Build a config for a given seed and level using a smooth difficulty curve.
    ///
    /// The curve mirrors the spirit of the original game (bigger maps, more and
    /// faster trains as you climb) but is continuous, so endless mode keeps
    /// escalating instead of stopping at a hand-authored level 9.
    pub fn new(seed: u64, level: u32) -> Self {
        let level = level.max(1);

        // Map grows in discrete steps; trains scale faster than the map.
        // Both are capped so endless mode stays playable instead of exploding
        // (a binary tree of height 7 already has up to 128 stations).
        let tree_height = (2 + level / 2).min(7); // 2,2,3,3,4,4,...,7
        let extend_probability = 30 + (level.min(6) * 8); // 38..=78
        let dead_end_probability = if level <= 2 { 0 } else { (level - 2).min(15) };
        let max_dead_ends = (level / 3).min(3);
        let trains = (8 + level * 2).min(30);

        // Base traversal time shrinks with level (trains speed up), with a floor.
        let base = 240u32.saturating_sub(level * 12).max(96); // ticks per edge
        let edge_ticks_choices = if level <= 1 {
            vec![base]
        } else {
            vec![base, (base * 4 / 5).max(90)]
        };

        let interval_choices = if level <= 2 {
            vec![3, 2]
        } else {
            vec![3, 2, 2, 1]
        };

        GameConfig {
            seed,
            level,
            tree_height,
            extend_probability,
            dead_end_probability,
            max_dead_ends,
            trains,
            edge_ticks_choices,
            interval_choices,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_from_seed_and_level() {
        assert_eq!(GameConfig::new(123, 5), GameConfig::new(123, 5));
    }

    #[test]
    fn difficulty_increases_monotonically() {
        let easy = GameConfig::new(1, 1);
        let hard = GameConfig::new(1, 10);
        assert!(hard.tree_height >= easy.tree_height);
        assert!(hard.trains > easy.trains);
        // Hardest available edge time should not be slower than the easy one.
        let easy_fast = *easy.edge_ticks_choices.iter().min().unwrap();
        let hard_fast = *hard.edge_ticks_choices.iter().min().unwrap();
        assert!(hard_fast <= easy_fast);
    }

    #[test]
    fn level_zero_is_clamped() {
        assert_eq!(GameConfig::new(9, 0), GameConfig::new(9, 1));
    }

    #[test]
    fn edge_times_have_a_floor() {
        for level in 1..200 {
            let c = GameConfig::new(0, level);
            assert!(c.edge_ticks_choices.iter().all(|&t| t >= 90));
        }
    }
}
