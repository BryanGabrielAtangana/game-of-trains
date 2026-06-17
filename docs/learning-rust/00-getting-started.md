# Chapter 0 — Getting started

## Install the toolchain

Rust is installed via `rustup`, which manages compiler versions and targets:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

This repo pins its toolchain in [`rust-toolchain.toml`](../../rust-toolchain.toml):

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
targets = ["wasm32-unknown-unknown"]
```

When you `cd` into the project, `rustup` automatically uses this exact setup —
the same `stable` channel, with `rustfmt`, `clippy`, and the WebAssembly target
already installed. Reproducible environments for free.

## The tools you'll use constantly

| Command | What it does |
| ------- | ------------ |
| `cargo build` | Compile. |
| `cargo test` | Compile and run tests. |
| `cargo run -p train-client` | Run a specific crate's binary. |
| `cargo fmt --all` | Auto-format (no style debates, ever). |
| `cargo clippy --all-targets` | Lint — catches bugs and un-idiomatic code. |
| `cargo doc --open` | Build and view the API docs from `///` comments. |

`cargo` is the build tool, package manager, test runner, and doc generator in
one. You will rarely call `rustc` directly.

## What a "crate" and a "workspace" are

- A **crate** is the unit of compilation — either a library (`lib.rs`) or a
  binary (`main.rs`).
- A **workspace** is several crates that share one `Cargo.lock` and `target/`
  directory.

Open the top-level [`Cargo.toml`](../../Cargo.toml). It has no `[package]` — it's
a pure workspace manifest:

```toml
[workspace]
members = [
    "crates/train-core",
    "crates/train-client",
    "crates/train-server",
]
default-members = ["crates/train-core"]
```

`default-members` is why `cargo test` (with no `-p`) runs only the fast,
dependency-free engine tests. To touch a specific crate, name it: `cargo run -p
train-client`.

## Your first run

```bash
cargo test --workspace        # 39 tests should pass
cargo run -p train-client     # prints today's generated map and an auto-play
cargo run -p train-server     # accepts an honest run, rejects a forged one
```

If those work, your environment is correct and you understand the project's
shape. 

## Anatomy of a tiny Rust program

Look at [`crates/train-client/src/main.rs`](../../crates/train-client/src/main.rs):

```rust
use train_core::{daily_seed, GameConfig, NodeKind, Simulation};

fn main() {
    let seed = daily_seed(2026, 6, 17);
    // ...
}
```

- `use` brings names into scope (like `import`).
- `fn main()` is the entry point of a binary.
- `let seed = ...` binds a variable. **Bindings are immutable by default** —
  you'd write `let mut seed` to reassign it. This default is one of the first
  ways Rust nudges you toward safer code.

## Exercises

1. Run `cargo doc --open` and browse the `train_core` docs. Notice every public
   item has documentation — that comes from the `///` comments in the source.
2. In `train-client`'s `main.rs`, change the date passed to `daily_seed`. Re-run
   and confirm the generated map changes. Why does it change? (Chapter 8 explains
   the determinism behind it.)
3. Run `cargo fmt --all` after deliberately mis-indenting a line. Watch it fix
   itself.

Next: [Chapter 1 — Ownership & borrowing →](./01-ownership-and-borrowing.md)
