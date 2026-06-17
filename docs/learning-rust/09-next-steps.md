# Chapter 9 — Where to go next

You've seen ownership, enums, traits, error handling, iterators, modules,
testing, and determinism — all through a real engine. The next phases of this
project are where you'll apply them to *new* domains: real-time rendering and a
networked backend.

## Phase 2 — the game client (`macroquad` → WASM)

[`macroquad`](https://github.com/not-fl3/macroquad) is a tiny, cross-platform 2D
game framework that compiles to WASM with no glue code. The skeleton you'll grow
into `crates/train-client`:

```rust
use macroquad::prelude::*;
use train_core::{daily_seed, GameConfig, Simulation, TICKS_PER_SECOND};

#[macroquad::main("Game of Trains")]
async fn main() {
    let cfg = GameConfig::new(daily_seed(2026, 6, 17), 4);
    let mut sim = Simulation::new(&cfg);
    let mut accumulator = 0.0_f32;

    loop {
        // Fixed-timestep simulation, interpolated rendering.
        accumulator += get_frame_time();
        let dt = 1.0 / TICKS_PER_SECOND as f32;
        while accumulator >= dt {
            sim.step();
            accumulator -= dt;
        }

        // Input: tap a switch to toggle it.
        if is_mouse_button_pressed(MouseButton::Left) {
            // hit-test against sim.map() node positions -> sim.toggle(node)
        }

        // Render: clear, draw tracks/houses from sim.map(), draw trains using
        // train.fraction() to interpolate between node positions.
        clear_background(SKYBLUE);
        next_frame().await;
    }
}
```

Rust concepts you'll exercise here:

- **The fixed-timestep loop** uses the integer-tick engine you already
  understand — `sim.step()` is the same function the server calls.
- **Borrowing per frame**: `sim.map()` and `sim.trains()` return shared borrows
  (Chapter 1) you read while drawing.
- **`async`/`.await`**: `next_frame().await` yields to the browser each frame.
  This is your introduction to async Rust, in the gentlest possible setting.
- **`cfg`** (Chapter 8) for any timing/input differences between native and WASM.

Build it for the web with:

```bash
cargo build -p train-client --target wasm32-unknown-unknown --release
# then drop the .wasm next to web/index.html and uncomment the loader script
```

## Phase 4 — the backend (`Axum` + `SQLx` + Postgres on Shuttle.rs)

The server's whole job is built on `verify()` from Chapter 4. A sketch of the
submit handler:

```rust
async fn submit(
    State(db): State<PgPool>,
    Json(run): Json<Run>,            // serde (Chapter 6) turns JSON into a Run
) -> Result<Json<Verified>, ApiError> {
    let verified = verify(&run)?;     // re-simulate; `?` propagates rejections
    sqlx::query!(
        "INSERT INTO scores (seed, level, score, best_combo) VALUES ($1,$2,$3,$4)",
        run.seed as i64, run.level as i32, verified.score, verified.best_combo as i32,
    )
    .execute(&db)
    .await?;
    Ok(Json(verified))
}
```

Rust concepts you'll exercise here:

- **`Result` and `?`** (Chapter 4) end to end: parse → verify → store, any step
  can fail and propagate cleanly.
- **Traits & generics** (Chapter 3): Axum extractors like `State<T>` and
  `Json<T>` are generic; `T: DeserializeOwned` ties back to serde.
- **`async`/`.await`** again, now for I/O (database, HTTP) rather than frames.
- **Features** (Chapter 6): the server build turns on `train-core`'s `serde`
  feature to accept `Run`s over the wire.

[Shuttle.rs](https://www.shuttle.rs/) deploys an Axum app and provisions Postgres
from annotations in code — a Rust-native deployment story that fits the
all-Rust theme.

## Recommended external resources

- **[The Rust Book](https://doc.rust-lang.org/book/)** — the canonical tutorial;
  this course is its applied companion.
- **[Rust by Example](https://doc.rust-lang.org/rust-by-example/)** — runnable
  snippets per concept.
- **[Rustlings](https://github.com/rust-lang/rustlings)** — small fix-the-code
  exercises; great muscle memory.
- **[The `std` docs](https://doc.rust-lang.org/std/)** — searchable; learn to live
  in `Option`, `Result`, `Iterator`, and `Vec`.
- **`cargo clippy`** — your in-repo mentor. Run it often and read every lint.

## A capstone exercise

Implement an **optimal solver** in `train-core`: given a `GameConfig`, compute the
switch inputs that route every train correctly, and prove with a test that the
resulting `Run` verifies with 100% accuracy. You'll touch the map graph
(Chapter 1's arena), pattern matching (Chapter 2), iterators (Chapter 5), and
testing (Chapter 7) — and you'll have written an AI for the game using only what
this course covered.

---

That's the course. The engine is real, tested, and waiting for a face. Go build
Phase 2. 🚂
