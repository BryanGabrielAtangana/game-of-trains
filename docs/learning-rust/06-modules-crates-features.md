# Chapter 6 — Modules, crates & features

How Rust code is *organised*: modules inside a crate, crates inside a workspace,
and Cargo **features** to toggle optional functionality.

## Modules and `pub`

A crate's code is a tree of modules. `train-core`'s root is
[`crates/train-core/src/lib.rs`](../../crates/train-core/src/lib.rs), which
declares its child modules:

```rust
pub mod config;
pub mod geometry;
pub mod map;
pub mod replay;
pub mod rng;
pub mod score;
pub mod sim;
```

Each `pub mod foo;` pulls in `src/foo.rs` as a module. **Everything is private by
default** — a struct, field, function, or module is only visible outside its
module if marked `pub`. This is the inverse of many languages, and it means your
public API is exactly what you chose to expose, nothing leaks by accident.

You can see private-by-default at work in `map.rs`:

```rust
struct Builder<'r> { /* ... */ }   // no `pub`: an internal helper
```

`Builder` is an implementation detail of map generation — leaving off `pub` means
no other module (or crate) can even name it.

## Re-exports: curating a prelude

Forcing callers to write `train_core::sim::Simulation` is noisy. `lib.rs`
re-exports the important types at the crate root:

```rust
pub use config::{GameConfig, TICKS_PER_SECOND};
pub use map::{Map, Node, NodeKind};
pub use replay::{verify, Input, RejectReason, Run, Verified};
pub use sim::{generate_schedule, Simulation, Train, TrainSpawn};
```

Now `train-client` and `train-server` write `use train_core::{GameConfig,
Simulation};` — clean and stable, even if you later move things between modules.
This is a deliberate API-design technique: the internal module layout can change
without breaking callers.

## Crates and the workspace

A **crate** compiles to one library or binary; a **workspace** groups crates. The
root [`Cargo.toml`](../../Cargo.toml) lists members and shares settings:

```toml
[workspace]
members = ["crates/train-core", "crates/train-client", "crates/train-server"]

[workspace.package]
version = "0.1.0"
edition = "2021"

[workspace.dependencies]
train-core = { path = "crates/train-core" }
serde = { version = "1", features = ["derive"] }
```

Then each member inherits with `.workspace = true`. From
`crates/train-client/Cargo.toml`:

```toml
[package]
name = "train-client"
version.workspace = true        # inherit the workspace version
edition.workspace = true

[dependencies]
train-core = { workspace = true }   # depend on our own engine crate
```

Benefits: one place to bump versions, one shared `Cargo.lock`, one `target/`
cache, and crates depend on each other by path (`train-client` → `train-core`).

## Cargo features: optional functionality

A **feature** is a named, optional piece of a crate. `train-core` defines one:

```toml
# crates/train-core/Cargo.toml
[features]
default = ["serde"]
serde = ["dep:serde"]

[dependencies]
serde = { workspace = true, optional = true }
```

- `optional = true` means `serde` is only compiled if requested.
- The `serde` feature, when on, activates the `serde` dependency (`dep:serde`).
- `default = ["serde"]` turns it on unless a consumer opts out.

In the source, code guarded by the feature uses `cfg`:

```rust
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Run { /* ... */ }
```

This is why a `Run` can serialize to JSON for the network — but a build that
doesn't need serialization (say, a size-conscious WASM experiment) could disable
the feature with `default-features = false` and shed the dependency entirely.

`#[cfg(...)]` is **conditional compilation**: the attributed item only exists in
builds where the condition holds. You'll meet it again for target-specific code in
Chapter 8.

## Visibility levels beyond `pub`

`pub` isn't all-or-nothing. You'll encounter:

- `pub(crate)` — visible anywhere in *this* crate, but not to outside crates.
- `pub(super)` — visible to the parent module.
- plain (no `pub`) — visible only within the current module and its children.

`train-core` keeps its surface small by exposing only what clients/servers need
and leaving helpers private.

## Exercises

1. Try to use `train_core::map::Builder` from `train-client`. It won't compile —
   explain the error in terms of `pub`.
2. Build the engine without serde: `cargo build -p train-core --no-default-features`.
   It should still compile (serialization just isn't available). Now run
   `cargo test -p train-core --no-default-features` — do the JSON-related dev
   needs still hold? (Look at what `serde_json` is used for.)
3. Add a new module `crates/train-core/src/util.rs` with a `pub fn` of your own,
   declare it in `lib.rs` with `pub mod util;`, re-export the function, and call
   it from `train-client`.

Next: [Chapter 7 — Testing →](./07-testing.md)
