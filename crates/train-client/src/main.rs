//! Temporary CLI front-end for the engine.
//!
//! Phase 2 replaces this with a `macroquad` game compiled to WebAssembly. Until
//! then this binary proves the engine works end-to-end: it generates today's
//! daily map and plays it with no switching, printing a summary.

use train_core::{daily_seed, GameConfig, NodeKind, Simulation};

fn main() {
    // A stand-in "today"; the real client reads the device clock / server date.
    let seed = daily_seed(2026, 6, 17);
    let level = 4;
    let config = GameConfig::new(seed, level);

    let mut sim = Simulation::new(&config);
    let map = sim.map();
    let stations = map.stations.len();
    let switches = map.nodes.iter().filter(|n| n.is_switch()).count();
    let dead_ends = map
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::DeadEnd)
        .count();

    println!("Game of Trains — daily map for 2026-06-17");
    println!("  seed        : {seed}");
    println!("  level       : {level}");
    println!("  grid        : {}x{}", map.width, map.height);
    println!("  nodes       : {}", map.nodes.len());
    println!("  stations    : {stations}");
    println!("  switches    : {switches}");
    println!("  dead-ends   : {dead_ends}");
    println!("  trains      : {}", config.trains);

    sim.run_to_end(5_000_000);
    let s = sim.scorer();
    println!("\nAuto-play with no switching:");
    println!("  delivered   : {}/{}", s.correct, s.total);
    println!("  accuracy    : {}%", s.accuracy());
    println!("  score       : {}", s.score);
}
