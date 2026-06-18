# 🚂⚔️ Rail Royale

A **turn-based strategy battler on rails**. Two players push **armed trains**
across a shared, switchable rail network to destroy each other's King tower. Play
is **commit-and-resolve**: each turn both sides secretly plan (deploy trains, set
routing switches), then the turn resolves deterministically.

> **Status:** **playable vs the AI in your browser** → **[play it here](https://bryangabrielatangana.github.io/game-of-trains/)**.
> The deterministic battle **engine** (`train-core`) is built and unit-tested; a
> heuristic **AI opponent** (Phase 2) and a WebAssembly **client** (Phase 4, vs-AI
> slice) are live. The online match server (Phase 3) is next. Full design:
> [`docs/design/rail-royale.md`](./docs/design/rail-royale.md).
>
> This project began as a solo daily routing puzzle (a 2019 jQuery game,
> rebuilt in Rust). We pivoted to the battler; the puzzle is archived under
> [`legacy/`](./legacy) and in git history.

## Why it's built this way

Everything hangs off one idea: a **single, deterministic engine** that both the
browser client and the match server run.

```
                     train-core  (the battle engine)
        arena · trains · orders · resolve_turn · combat · win check
              pure · deterministic · no I/O · unit-tested
                       │ wasm32            │ native
              ┌────────▼────────┐  ┌───────▼─────────────────┐
              │   game client   │  │     match server        │
              │  (renders the   │  │  Axum · SQLx · Postgres │
              │   battle, sends │  │  resolves + verifies    │
              │   your orders)  │  │  each submitted turn    │
              └─────────────────┘  └─────────────────────────┘
```

Because a turn is a **pure function of `(config, both players' orders)`**, the
server can **re-simulate any match to validate it** — no result can be faked.
That's the foundation for a trustworthy competitive ladder, and it's only
practical because client and server share the exact same Rust code.

## The engine (Phase 1, today)

`crates/train-core`:
- **`battle/arena.rs`** — the symmetric rail graph (lanes, junctions, towers),
  generated from a config; per-faction routing toward the enemy.
- **`battle/unit.rs`** — train types (`Express`, `Armored`, `Rocket`) and their
  integer stats (the counter-triangle).
- **`battle/orders.rs`** — a player's committed plan (`Deploy`, `SetSwitch`).
- **`battle/state.rs`** — the deterministic `BattleState` carried between turns.
- **`battle/resolve.rs`** — `resolve_turn`: movement → shooting → collisions →
  win check, in fixed ticks.

Deterministic primitives (`rng.rs`, `geometry.rs`) and the daily-seed helpers are
shared from the puzzle era.

## The client (Phase 4 vs-AI slice, today)

`crates/train-client` compiles to `wasm32-unknown-unknown` and renders the battle
on a 2D `<canvas>` — **menu → plan → watch it resolve → win/lose**, mobile-first.
Deliberately **no game framework and no `wasm-bindgen`** (that fragility sank the
old puzzle client): all game *and rendering* logic is Rust that emits a small JSON
**display list**, and a ~60-line JS loader (`web/index.html`) just paints it and
forwards taps. The module exposes a handful of `#[no_mangle]` functions; the wasm
has **zero imports**, so the browser boot can't fail on linking.

## Running it

```bash
cargo test --workspace      # engine invariants, combat, determinism, AI, win
cargo doc -p train-core --open

# Build + play the web client locally:
./scripts/build-web.sh
python3 -m http.server -d web 8080   # then open http://localhost:8080

# Tune balance offline (AI vs AI across difficulties):
cargo run -p train-core --example selfplay
```

A tiny taste:

```rust
use train_core::{BattleConfig, BattleState, Orders, TrainKind};

let mut state = BattleState::new(BattleConfig::default());
let a = Orders::new().deploy(TrainKind::Rocket, 0); // shell the enemy King
let b = Orders::new();                              // opponent holds
train_core::resolve_turn(&mut state, &a, &b);
```

## Roadmap (from the design doc)

- [x] **Phase 1 — engine slice:** arena, orders, `resolve_turn`, towers, train
      types, win check. Deterministic, unit-tested. *No UI.*
- [x] **Phase 2 — vs-AI single-player:** a heuristic opponent (`battle::ai`) to
      tune feel/balance offline (doubles as onboarding), plus a balance pass.
- [x] **Phase 4 (vs-AI slice) — web client:** WASM + canvas, playable vs the AI.
- [ ] **Phase 3 — async online PvP:** match server (Axum + SQLx + Postgres on
      Shuttle.rs) — create/join, submit a turn's orders, server resolves + verifies.
- [ ] **Phase 4 (full) — polish & ladder:** full roster, decks, MMR, replays, live turns.

## Learning Rust alongside this project

This repo doubles as a **hands-on Rust course** built around the engine — see
[`docs/learning-rust/`](./docs/learning-rust/README.md). Each chapter teaches a
Rust concept using *real code from `train-core`*, with exercises.

## License

MIT.
