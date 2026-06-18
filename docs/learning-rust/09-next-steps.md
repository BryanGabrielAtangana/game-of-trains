# Chapter 9 — Where to go next

You've seen ownership, enums/matching, traits, `Option`, iterators, modules,
testing, and determinism — all through the real battle engine. The next phases of
Rail Royale (see [`docs/design/rail-royale.md`](../design/rail-royale.md)) are
where you apply them to new domains.

## Phase 2 — single-player vs an AI ✅ (shipped)

A heuristic opponent that picks `Orders` each turn, so you can tune feel and
balance entirely offline (and it doubles as onboarding). This now lives in
[`crates/train-core/src/battle/ai.rs`](../../crates/train-core/src/battle/ai.rs)
as a **pure, deterministic** function:

```rust
use train_core::{ai_orders, AiLevel, BattleState, BattleConfig, Faction};

let state = BattleState::new(BattleConfig::default());
let plan = ai_orders(&state, Faction::A, AiLevel::Normal);
```

It reads the board, **counter-picks** the kind the enemy fields most (the
counter-triangle from Chapter 2's enums), **defends** the lane where a train is
nearest its King, and **spends steam** without ever going over budget. `AiLevel`
(`Easy`/`Normal`/`Hard`) tunes greed and whether it routes switches.

The Rust this exercises, chapter by chapter:

- **Pattern matching** (Ch. 2) over `TrainKind` / `AiLevel` to choose a counter.
- **Iterators** (Ch. 5) — `filter` + `min_by_key` to find the closest threat and
  the cheapest affordable unit, with **no allocation**.
- **`Option`** (Ch. 4) — `threat_lane` returns `Option<usize>`; the caller uses
  `unwrap_or` / `if let`.
- **Testing & determinism** (Ch. 7–8): the module's tests assert the AI never
  overspends, never deploys while broke, is reproducible, and that *two AIs
  playing each other always reach the same terminal `Status`* — the capstone.

Because it's deterministic, the example
[`examples/selfplay.rs`](../../crates/train-core/examples/selfplay.rs) pits every
difficulty pairing against each other and prints the results:

```text
cargo run -p train-core --example selfplay
```

That table is your **balance dashboard**: tweak `TrainKind::stats()` or
`BattleConfig`, re-run, and watch outcomes shift — pure functions make balance a
spreadsheet, not a guessing game. It already paid off: the harness exposed a
draw-heavy default (towers were melting single-file streams), which a balance pass
fixed by making tower stats part of `BattleConfig` and tuning them — now every
asymmetric matchup is decisive and a test pins the `Hard > Normal > Easy` ladder.

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

## Phase 4 — the web client ✅ (vs-AI slice shipped)

[`crates/train-client`](../../crates/train-client) compiles to
`wasm32-unknown-unknown` as a `cdylib` and renders the battle on a 2D `<canvas>` —
menu → plan → resolve → win/lose — playable vs the Phase-2 AI. It deliberately
uses **no game framework and no `wasm-bindgen`**: all logic *and* rendering is Rust
that emits a small JSON **display list**, and a ~60-line JS loader paints it.

New Rust this introduces:

- **FFI / `#[no_mangle] extern "C"`** — the only boundary to JS. The whole game is
  one thread-local `Client`; functions like `rr_pointer`, `rr_tick`, `rr_render`
  drive it. The wasm exports its `memory`, so JS reads the scene JSON straight out
  of WASM linear memory at `(ptr, len)`.
- **`thread_local!` + `RefCell`** — safe global mutable state on a single-threaded
  wasm target, without `static mut` or `unsafe`.
- **`serde` serialization in anger** (Chapter 6) — the `Scene`/`DrawOp` display
  list is `Serialize`d to JSON every frame.
- **The engine grew `resolve_turn_frames`** — returns a per-tick snapshot `Vec` so
  the client can *animate* a turn; the same data will drive replays later.

Try it: `./scripts/build-web.sh` then serve `web/`. Still ahead (full Phase 4):
richer animation/juice, polished switch-routing UI, decks, MMR, replays, live turns.

## Recommended resources

- **[The Rust Book](https://doc.rust-lang.org/book/)** — the canonical tutorial.
- **[Rust by Example](https://doc.rust-lang.org/rust-by-example/)** — runnable snippets.
- **[Rustlings](https://github.com/rust-lang/rustlings)** — fix-the-code drills.
- **[`std` docs](https://doc.rust-lang.org/std/)** — live in `Option`, `Result`,
  `Iterator`, `Vec`.
- **`cargo clippy`** — your in-repo mentor; read every lint.

## A capstone exercise

The deterministic AI (Phase 2 above) is done, including the test that two AIs
playing each other from the same config always reach the same `Status` — a real
use of determinism, pattern matching, iterators, and testing together. Your turn:
**make the AI smarter or the game more decisive.** Pick one:

- Teach `counter_pick` to weigh enemy *steam* and tower HP, not just unit counts.
- Add an `AiLevel::Insane` that simulates a turn ahead with `resolve_turn` on a
  cloned state and keeps the plan that does the most King damage (`BattleState`
  is `Clone` precisely so you can do this).
- Re-tune `BattleConfig`/`TrainKind::stats()` until `selfplay` stops drawing, then
  add a test pinning the matchup you want.

From there, wiring it into the match server is "just" I/O around the engine you
understand.

---

That's the course. The engine is real, tested, and waiting for a face. Go build
Phase 2. 🚂⚔️
