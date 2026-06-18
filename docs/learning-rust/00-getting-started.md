# Chapter 0 — Getting started

## Install the toolchain

Rust is installed via `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

This repo pins its toolchain in [`rust-toolchain.toml`](../../rust-toolchain.toml)
(`stable` + `rustfmt`, `clippy`, and the `wasm32-unknown-unknown` target), so a
fresh checkout uses the exact same setup automatically.

## The tools you'll use constantly

| Command | What it does |
| ------- | ------------ |
| `cargo build` | Compile. |
| `cargo test -p train-core` | Compile and run the engine's tests. |
| `cargo fmt --all` | Auto-format (no style debates). |
| `cargo clippy --all-targets` | Lint — catches bugs and un-idiomatic code. |
| `cargo doc --open` | Build and view docs from `///` comments. |

## Crates and the workspace

- A **crate** is the unit of compilation — a library (`lib.rs`) or binary (`main.rs`).
- A **workspace** groups crates under one `Cargo.lock` / `target/`.

Open the top-level [`Cargo.toml`](../../Cargo.toml):

```toml
[workspace]
resolver = "2"
members = ["crates/train-core"]
# The retired daily-puzzle crates live under legacy/ (kept for history, not built).
exclude = ["legacy"]
```

Today the workspace is a single crate, **`train-core`** — the deterministic battle
engine. (The earlier puzzle game is archived under `legacy/`.)

## Your first run

```bash
cargo test -p train-core    # ~20 tests should pass
cargo doc -p train-core --open
```

If those work, your environment is correct.

## Anatomy of the engine's entry points

`train-core` is a *library*, not a binary — there's no `main()`. Its public API
lives in [`crates/train-core/src/lib.rs`](../../crates/train-core/src/lib.rs),
which re-exports the battle types and includes a runnable doc example:

```rust
use train_core::{BattleConfig, BattleState, Orders, TrainKind};

let mut state = BattleState::new(BattleConfig::default());
let a = Orders::new().deploy(TrainKind::Rocket, 0);
let b = Orders::new();
train_core::resolve_turn(&mut state, &a, &b);
```

- `use` brings names into scope (like `import`).
- `let state = ...` binds a variable; **bindings are immutable by default** —
  `let mut state` is needed to mutate it. That default is one of the first ways
  Rust nudges you toward safer code.

## Exercises

1. Run `cargo doc -p train-core --open` and browse the API. Notice every public
   item is documented from its `///` comments.
2. In the doc example, change `TrainKind::Rocket` to `TrainKind::Armored`. Re-run
   `cargo test --doc -p train-core`. Does it still pass?
3. Run `cargo fmt --all` after mis-indenting a line and watch it fix itself.

Next: [Chapter 1 — Ownership & borrowing →](./01-ownership-and-borrowing.md)
