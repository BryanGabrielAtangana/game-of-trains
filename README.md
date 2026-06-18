# 🚂 Game of Trains

Route every train to the station that matches its number by flipping the track
switches before it arrives. Miss, and it ends up at the wrong house — or worse, a
dead-end.

This is a **2026 rebuild** of an old jQuery/CSS browser game (preserved under
[`Wise train/`](./Wise%20train)) into an **all-Rust**, mobile-first PWA with
**cheat-resistant global leaderboards and streaks**.

> **Status:** Phase 1 complete — the deterministic engine (`train-core`) is built,
> fully tested, and compiles to both native and WebAssembly. The game client and
> backend are scaffolded and come next. See [the roadmap](#roadmap).

---

## Why it's built this way

The whole project is organised around one idea: **a single, deterministic game
engine shared by the browser and the server.**

```
            ┌──────────────────────────────────────────────┐
            │                 train-core                    │
            │  map-gen · simulation · scoring · verify()    │
            │     pure, deterministic, no I/O, no render     │
            └───────────────┬───────────────┬──────────────┘
                            │ (wasm32)      │ (native)
                ┌───────────▼──────┐  ┌─────▼─────────────────┐
                │   train-client   │  │     train-server      │
                │ macroquad → WASM │  │ Axum · SQLx · Postgres│
                │  renders & plays │  │ leaderboards · verify │
                └──────────────────┘  └───────────────────────┘
```

Because every outcome is a pure function of `(seed, level, inputs)`, the server
can **re-simulate any submitted run** and trust only the score *it* computes.
Forged scores simply don't reproduce. That's the engineering centrepiece — and
it's only practical because client and server run the *exact same Rust code*.

## Workspace layout

| Crate / dir       | What it is                                                                 |
| ----------------- | ------------------------------------------------------------------------- |
| `crates/train-core`   | The engine: RNG, map generation, simulation, scoring, replay/verify. No I/O, no rendering. **Done & tested.** |
| `crates/train-client` | The game: a `macroquad` renderer for the engine. Runs natively and as WASM in the PWA. **Playable.** |
| `crates/train-server` | The backend. Phase 4 adds Axum + SQLx + Postgres on Shuttle.rs. *(CLI stub for now.)* |
| `web/`            | The installable PWA shell (manifest, service worker, icon) the WASM client mounts into. |
| `Wise train/`     | The original 2019-era game, kept for posterity.                           |

## Running it today

```bash
# Run the engine test suite (invariants, scoring, replay verification)
cargo test -p train-core

# Play the game natively (opens a desktop window)
cargo run -p train-client

# Watch the server accept an honest run and reject a forged one
cargo run -p train-server
```

### Play it in the browser (the real target)

```bash
./scripts/build-web.sh                 # builds the wasm + assembles web/
(cd web && python3 -m http.server 8080)
# open http://localhost:8080  — tap switches to route the trains
```

On Linux, the native client needs a few system libraries to link
(`libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev`); the WebAssembly build
does not.

## Roadmap

- [x] **Phase 0 — Scaffold:** Cargo workspace, CI (fmt + clippy + test + wasm), PWA shell.
- [x] **Phase 1 — `train-core`:** deterministic map-gen, simulation, scoring, replay verification, full unit tests.
- [x] **Phase 2 — `train-client`:** `macroquad` rendering, tap-to-switch, level progression, native + WASM builds, GitHub Pages deploy.
- [ ] **Phase 3 — Polish & PWA:** art, sound, haptics, share card, offline install.
- [ ] **Phase 4 — `train-server`:** Axum + SQLx + Postgres, daily seeds, score verification, leaderboards, streaks.
- [ ] **Phase 5 — Integrate & deploy:** wire client ↔ server, deploy backend (Shuttle.rs) + client (CDN).
- [ ] **Phase 6 — Growth:** analytics, share images, onboarding, accessibility.

## Learning Rust alongside this project

This repo doubles as a **hands-on Rust course**: see
[`docs/learning-rust/`](./docs/learning-rust/README.md). Each chapter teaches a
Rust concept using *real code from this codebase*, with exercises.

## Where it's headed

Beyond the solo daily puzzle, the design for a strategy battler — **Rail Royale**
(simultaneous, hidden commit-and-resolve turns on a shared switchable rail
network, async online PvP, server-verified for fairness) — lives in
[`docs/design/rail-royale.md`](./docs/design/rail-royale.md).

## License

MIT.
