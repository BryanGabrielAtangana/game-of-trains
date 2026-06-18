//! Offline balance harness: pit the heuristic AIs against each other and print
//! how each matchup resolves. Because [`ai_orders`] and [`resolve_turn`] are pure
//! and deterministic, every run is reproducible — handy for tuning train stats and
//! the steam economy in `BattleConfig` without a UI.
//!
//! Run it with:
//! ```text
//! cargo run -p train-core --example selfplay
//! ```

use train_core::{ai_orders, resolve_turn, AiLevel, BattleConfig, BattleState, Faction, Status};

/// Play one match to its terminal state and report how it ended.
fn play(cfg: BattleConfig, a: AiLevel, b: AiLevel) -> (Status, u32, i32, i32) {
    let mut s = BattleState::new(cfg);
    while !s.is_over() {
        let oa = ai_orders(&s, Faction::A, a);
        let ob = ai_orders(&s, Faction::B, b);
        resolve_turn(&mut s, &oa, &ob);
    }
    (
        s.status,
        s.turn,
        s.king_hp(Faction::A),
        s.king_hp(Faction::B),
    )
}

fn label(status: Status) -> &'static str {
    match status {
        Status::Won(Faction::A) => "A wins",
        Status::Won(Faction::B) => "B wins",
        Status::Draw => "draw  ",
        Status::Ongoing => "??    ",
    }
}

fn main() {
    let cfg = BattleConfig::default();
    println!(
        "Rail Royale — AI self-play (default config: {} cols x {} rows, {} max turns)\n",
        cfg.cols, cfg.rows, cfg.max_turns
    );
    println!("  A (Hard plays switches) vs B   | result  | turns | A.king | B.king");
    println!("  ------------------------------+---------+-------+--------+-------");

    for a in AiLevel::ALL {
        for b in AiLevel::ALL {
            let (status, turns, ahp, bhp) = play(cfg.clone(), a, b);
            println!(
                "  A:{a:<6?} vs B:{b:<6?}        | {} | {turns:^5} | {ahp:^6} | {bhp:^6}",
                label(status),
            );
        }
    }

    println!("\nTip: tweak TrainKind::stats() or BattleConfig and re-run to see balance shift.");
}
