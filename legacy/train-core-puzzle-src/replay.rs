//! Replays and server-side verification.
//!
//! A [`Run`] is the *complete, compact* record of a play-through: the config it
//! was played on plus the ordered list of switch toggles the player made and the
//! tick each happened. It is everything the server needs to **recompute the
//! score from scratch** and reject anything that doesn't add up.
//!
//! This is the payoff of sharing one engine between client and server: the
//! browser plays the game and submits a `Run`; the server re-runs the identical
//! simulation and trusts only the score *it* computed.

use crate::config::GameConfig;
use crate::score::Scorer;
use crate::sim::Simulation;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single player action: "at tick T, the player toggled switch `node`".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Input {
    pub tick: u32,
    pub node: usize,
}

/// A full, replayable record of one game.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Run {
    pub seed: u64,
    pub level: u32,
    /// Switch toggles in the order they were made (sorted by tick when verified).
    pub inputs: Vec<Input>,
    /// The score the client claims it achieved. The server never trusts this
    /// directly — it compares it against a fresh simulation.
    pub claimed_score: i32,
}

/// Why a submitted run was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RejectReason {
    /// The claimed score does not match the authoritative re-simulation.
    ScoreMismatch { claimed: i32, actual: i32 },
    /// An input referenced a tick beyond the run, or inputs were absurdly many.
    MalformedInput,
}

/// The authoritative result of verifying a run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Verified {
    /// The score the server computed and will store.
    pub score: i32,
    pub correct: u32,
    pub total: u32,
    pub best_combo: u32,
}

/// A generous safety bound so verification of malicious input can't run forever.
const MAX_TICKS: u32 = 50_000_000;
/// No legitimate session needs more toggles than this.
const MAX_INPUTS: usize = 1_000_000;

/// Re-simulate a run from scratch and return the authoritative score, or a reason
/// to reject it. The server stores `Verified.score`, never `run.claimed_score`.
pub fn verify(run: &Run) -> Result<Verified, RejectReason> {
    if run.inputs.len() > MAX_INPUTS {
        return Err(RejectReason::MalformedInput);
    }

    let cfg = GameConfig::new(run.seed, run.level);
    let mut sim = Simulation::new(&cfg);

    // Apply inputs in tick order. We sort defensively so a shuffled payload
    // still verifies deterministically.
    let mut inputs = run.inputs.clone();
    inputs.sort_by_key(|i| i.tick);
    let mut next = 0usize;

    while !sim.is_finished() && sim.tick() < MAX_TICKS {
        let now = sim.tick();
        // Apply every toggle scheduled for the current tick *before* stepping,
        // matching the ordering the client used while playing.
        while next < inputs.len() && inputs[next].tick == now {
            sim.toggle(inputs[next].node);
            next += 1;
        }
        // Inputs for ticks that already passed mean a malformed/forged payload.
        if next < inputs.len() && inputs[next].tick < now {
            return Err(RejectReason::MalformedInput);
        }
        sim.step();
    }

    let final_score: &Scorer = sim.scorer();
    if final_score.score != run.claimed_score {
        return Err(RejectReason::ScoreMismatch {
            claimed: run.claimed_score,
            actual: final_score.score,
        });
    }

    Ok(Verified {
        score: final_score.score,
        correct: final_score.correct,
        total: final_score.total,
        best_combo: final_score.best_combo,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::Simulation;

    /// Play a run honestly (recording inputs) and return the Run + true score.
    /// Strategy: leave all switches at default and never toggle — a valid, if
    /// not optimal, play-through. This exercises the full verify path.
    fn honest_default_run(seed: u64, level: u32) -> Run {
        let cfg = GameConfig::new(seed, level);
        let mut sim = Simulation::new(&cfg);
        sim.run_to_end(MAX_TICKS);
        Run {
            seed,
            level,
            inputs: Vec::new(),
            claimed_score: sim.scorer().score,
        }
    }

    #[test]
    fn honest_run_verifies() {
        for level in 1..10 {
            let run = honest_default_run(2024, level);
            let v = verify(&run).expect("honest run should verify");
            assert_eq!(v.score, run.claimed_score);
            assert_eq!(v.total, GameConfig::new(2024, level).trains);
        }
    }

    #[test]
    fn run_with_real_inputs_verifies() {
        // Record an actual sequence of toggles and confirm it re-simulates.
        let cfg = GameConfig::new(99, 6);
        let mut sim = Simulation::new(&cfg);
        let mut inputs = Vec::new();

        // Toggle the root switch a few times at known ticks (if root is a switch).
        let root = sim.map().root;
        let toggle_ticks = [0u32, 5, 30, 100];
        let mut ti = 0;
        while !sim.is_finished() && sim.tick() < 1_000_000 {
            if ti < toggle_ticks.len() && sim.tick() == toggle_ticks[ti] {
                if sim.map().nodes[root].is_switch() {
                    sim.toggle(root);
                    inputs.push(Input {
                        tick: sim.tick(),
                        node: root,
                    });
                }
                ti += 1;
            }
            sim.step();
        }

        let run = Run {
            seed: 99,
            level: 6,
            inputs,
            claimed_score: sim.scorer().score,
        };
        let v = verify(&run).expect("recorded inputs should verify");
        assert_eq!(v.score, run.claimed_score);
    }

    #[test]
    fn cheating_is_rejected() {
        let mut run = honest_default_run(7, 5);
        run.claimed_score += 1000; // inflate the score
        match verify(&run) {
            Err(RejectReason::ScoreMismatch { claimed, actual }) => {
                assert_eq!(claimed, run.claimed_score);
                assert_ne!(actual, claimed);
            }
            other => panic!("expected ScoreMismatch, got {other:?}"),
        }
    }

    #[test]
    fn unsorted_inputs_still_verify() {
        // Build a valid run, then shuffle its inputs; verify must sort and accept.
        let cfg = GameConfig::new(11, 6);
        let mut sim = Simulation::new(&cfg);
        let root = sim.map().root;
        let mut inputs = Vec::new();
        let ticks = [2u32, 8, 8, 20];
        let mut ti = 0;
        while !sim.is_finished() && sim.tick() < 1_000_000 {
            while ti < ticks.len() && ticks[ti] == sim.tick() {
                if sim.map().nodes[root].is_switch() {
                    sim.toggle(root);
                    inputs.push(Input {
                        tick: sim.tick(),
                        node: root,
                    });
                }
                ti += 1;
            }
            sim.step();
        }
        let score = sim.scorer().score;
        inputs.reverse(); // out of order on the wire
        let run = Run {
            seed: 11,
            level: 6,
            inputs,
            claimed_score: score,
        };
        assert!(verify(&run).is_ok());
    }

    #[test]
    fn too_many_inputs_rejected() {
        let run = Run {
            seed: 1,
            level: 1,
            inputs: vec![Input { tick: 0, node: 0 }; MAX_INPUTS + 1],
            claimed_score: 0,
        };
        assert_eq!(verify(&run), Err(RejectReason::MalformedInput));
    }
}
