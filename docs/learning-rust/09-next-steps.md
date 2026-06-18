# Chapter 9 — Where to go next

You've seen ownership, enums/matching, traits, `Option`, iterators, modules,
testing, and determinism — all through the real battle engine. The next phases of
Rail Royale (see [`docs/design/rail-royale.md`](../design/rail-royale.md)) are
where you apply them to new domains.

## Phase 2 — single-player vs an AI

A heuristic opponent that picks `Orders` each turn, so you can tune feel and
balance entirely offline (and it doubles as onboarding). A sketch you could add to
`train-core`:

```rust
/// Pick a plan for `me` given the current state (pure, deterministic, testable).
pub fn ai_orders(state: &BattleState, me: Faction) -> Orders {
    let mut o = Orders::new();
    if state.steam[me.index()] >= TrainKind::Rocket.stats().cost {
        o = o.deploy(TrainKind::Rocket, 0); // naive: shell down a lane
    }
    o
}
```

Rust you'll exercise: pattern matching over state, iterators to score options,
and **unit tests** that assert the AI never issues an unaffordable plan.

## Phase 3 — async online PvP (the match server)

An Axum + SQLx + Postgres service (deployable on Shuttle.rs). A match is
`config + the ordered list of each turn's orders`; on submission the server
**re-runs `resolve_turn` to validate and advance** — the cheat-proof core.

```rust
async fn submit(
    State(db): State<PgPool>,
    Json(turn): Json<TurnSubmission>,   // serde (Chapter 6) decodes the orders
) -> Result<Json<BattleState>, ApiError> {
    let mut state = load_match(&db, turn.match_id).await?;   // ? (Chapter 4)
    train_core::resolve_turn(&mut state, &turn.a, &turn.b);  // the shared engine
    save_match(&db, &state).await?;
    Ok(Json(state))
}
```

Rust you'll exercise: `Result`/`?` end to end, traits & generics (Axum's
`State<T>`/`Json<T>`), `async`/`.await`, and the `serde` feature carrying
`Orders`/`BattleState` over the wire.

## Phase 4 — client, polish & ladder

A `macroquad` (or similar) client compiled to WebAssembly renders the arena,
towers, HP bars and two-colour factions, and provides the hidden plan-phase UI;
then decks, MMR, replays and live turns.

## Recommended resources

- **[The Rust Book](https://doc.rust-lang.org/book/)** — the canonical tutorial.
- **[Rust by Example](https://doc.rust-lang.org/rust-by-example/)** — runnable snippets.
- **[Rustlings](https://github.com/rust-lang/rustlings)** — fix-the-code drills.
- **[`std` docs](https://doc.rust-lang.org/std/)** — live in `Option`, `Result`,
  `Iterator`, `Vec`.
- **`cargo clippy`** — your in-repo mentor; read every lint.

## A capstone exercise

Add a **deterministic AI** (Phase 2 above) *and* prove with a test that two AIs
playing each other from the same config always reach the same `Status` — a real
use of determinism, pattern matching, iterators, and testing together. From there,
wiring it into the match server is "just" I/O around the engine you understand.

---

That's the course. The engine is real, tested, and waiting for a face. Go build
Phase 2. 🚂⚔️
