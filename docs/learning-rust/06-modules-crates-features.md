# Chapter 6 — Modules, crates & features

How Rust code is *organised*: modules in a crate, crates in a workspace, and Cargo
**features** to toggle optional functionality.

## Modules and `pub`

The engine groups its battle code in a module tree. `lib.rs` declares the top
level, and [`battle/mod.rs`](../../crates/train-core/src/battle/mod.rs) declares
its children:

```rust
// battle/mod.rs
mod arena;
mod orders;
mod resolve;
mod state;
mod unit;
```

**Everything is private by default** — only `pub` items are visible outside their
module. Notice the submodules are declared *without* `pub`: their contents are
exposed deliberately through re-exports (below), not by leaking the module paths.
Internal helpers stay private:

```rust
// resolve.rs — an implementation detail, no `pub`
fn node_of(t: &Train) -> NodeId { /* ... */ }
```

## Re-exports: a curated API

Forcing callers to write `train_core::battle::resolve::resolve_turn` would be
noisy. `battle/mod.rs` lifts the useful types to the module root, and `lib.rs`
lifts them to the crate root:

```rust
// battle/mod.rs
pub use arena::{Arena, BattleConfig, NodeId, Tower, TowerKind};
pub use resolve::{resolve_turn, TurnEvent};
pub use state::{BattleState, Status};
pub use unit::{Train, TrainKind, TrainStats};

// lib.rs
pub use battle::{resolve_turn, BattleConfig, BattleState, Orders, TrainKind /* … */};
```

Now consumers write `use train_core::{BattleState, Orders};`. The internal layout
can change without breaking callers — a deliberate API-design technique.

## Crates and the workspace

The top-level [`Cargo.toml`](../../Cargo.toml) is a pure workspace manifest:

```toml
[workspace]
resolver = "2"
members = ["crates/train-core"]
exclude = ["legacy"]   # the retired puzzle crates, kept for history, not built
```

One member today (`train-core`). When the game client and match server arrive
they'll be added here and depend on `train-core` by path — the same engine
compiled to wasm for the client and natively for the server.

## Cargo features: optional functionality

`crates/train-core/Cargo.toml` defines a `serde` feature:

```toml
[features]
default = ["serde"]
serde = ["dep:serde"]

[dependencies]
serde = { workspace = true, optional = true }
```

- `optional = true` — `serde` compiles only if requested.
- The `serde` feature activates that dependency (`dep:serde`).
- `default = ["serde"]` turns it on unless a consumer opts out.

In the source, feature-gated code uses `cfg`:

```rust
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Orders { /* ... */ }
```

That's why `Orders`/`BattleState` can serialize to JSON to cross the wire — but a
size-conscious build could drop the feature with `--no-default-features`.
`#[cfg(...)]` is **conditional compilation**: the item only exists in builds where
the condition holds (you'll meet it again for the wasm target in Chapter 8).

## Visibility beyond `pub`

You'll also see `pub(crate)` (visible crate-wide, not outside) and `pub(super)`
(visible to the parent module). The engine keeps its surface small by exposing
only what clients/servers need.

## Exercises

1. Try to use `train_core::battle::resolve::node_of` from a test. It won't compile
   — explain the error in terms of `pub`.
2. Build without serde: `cargo build -p train-core --no-default-features`. It
   should still compile (serialization just isn't available).
3. Add a `util` submodule under `battle/` with a `pub fn`, declare it in `mod.rs`,
   re-export it, and call it from a test.

Next: [Chapter 7 — Testing →](./07-testing.md)
