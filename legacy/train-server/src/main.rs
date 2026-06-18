//! Temporary CLI stand-in for the backend.
//!
//! Phase 4 turns this into an Axum service (daily seeds, score submission,
//! leaderboards, streaks) backed by SQLx + Postgres and deployed on Shuttle.rs.
//! For now it demonstrates the core server responsibility: taking a submitted
//! [`Run`] and verifying it by re-simulating with the shared engine.

use train_core::{verify, GameConfig, Input, Run, Simulation};

fn main() {
    // Pretend a client just submitted this run. We rebuild an honest one here.
    let seed = train_core::daily_seed(2026, 6, 17);
    let level = 4;

    let cfg = GameConfig::new(seed, level);
    let mut sim = Simulation::new(&cfg);
    sim.run_to_end(5_000_000);
    let honest = Run {
        seed,
        level,
        inputs: Vec::<Input>::new(),
        claimed_score: sim.scorer().score,
    };

    println!("Verifying an honest run...");
    match verify(&honest) {
        Ok(v) => println!(
            "  accepted: score={} correct={}/{}",
            v.score, v.correct, v.total
        ),
        Err(e) => println!("  rejected: {e:?}"),
    }

    println!("Verifying a forged run (claimed_score inflated)...");
    let forged = Run {
        claimed_score: honest.claimed_score + 9000,
        ..honest
    };
    match verify(&forged) {
        Ok(v) => println!("  accepted (unexpected!): {v:?}"),
        Err(e) => println!("  rejected as expected: {e:?}"),
    }
}
