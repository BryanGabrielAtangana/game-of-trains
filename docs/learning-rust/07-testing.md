# Chapter 7 — Testing

Testing is built into the language and `cargo`. The engine's correctness *is* the
product — a bug in combat or determinism would corrupt every match — so it ships
with a focused suite.

## Unit tests live next to the code

The convention is a `tests` submodule at the bottom of a file, compiled only for
tests. See [`battle/mod.rs`](../../crates/train-core/src/battle/mod.rs):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_state_is_sane() {
        let s = BattleState::new(BattleConfig::default());
        assert_eq!(s.king_hp(Faction::A), s.cfg.king_hp);
        assert!(s.trains.is_empty());
        assert_eq!(s.status, Status::Ongoing);
    }
}
```

- `#[cfg(test)]` excludes the module from normal builds — zero cost shipped.
- `use super::*;` imports the parent module's items, including **private** ones.
- `#[test]` marks a test; run them with `cargo test`.

## The assertion macros

`assert!(cond)`, `assert_eq!(a, b)`, `assert_ne!(a, b)` (the `_eq`/`_ne` forms
print both sides on failure, so types need `Debug`), and `panic!("msg")`. Messages
can be customised:

```rust
assert!(won, "rockets should break through an idle B");
```

## Testing determinism directly

Determinism is a *requirement*, so it's a test (`mod.rs`):

```rust
#[test]
fn deterministic_replay() {
    let script = |s: &mut BattleState| {
        for _ in 0..6 {
            resolve_turn(s, &Orders::new().deploy(TrainKind::Express, 0),
                            &Orders::new().deploy(TrainKind::Armored, 0));
        }
    };
    let mut s1 = BattleState::new(duel_cfg());
    let mut s2 = BattleState::new(duel_cfg());
    script(&mut s1);
    script(&mut s2);
    assert_eq!(s1, s2);   // whole states compared
}
```

Comparing entire `BattleState`s with `==` is only possible because every type in
the tree derives `PartialEq` (Chapter 3). Derives and tests reinforce each other.

## Testing behaviour, not just values

Other tests assert *mechanics* over a real resolution — the win condition, the
counter-triangle, and resource limits:

```rust
// rockets out-range the King and shell it down
fn rockets_destroy_the_king_and_win() { /* loop turns until Status::Won(A) */ }

// a fragile Express loses a head-on to an Armored
fn head_on_collision_favors_armored() { /* assert A has 0 trains, B has 1 */ }

// steam caps how many trains spawn in a turn
fn steam_caps_deploys() { /* assert <= 2 Armored from 10 steam */ }
```

Each test *is* a small specification: if someone breaks combat or routing, the
relevant one goes red. Several use a shared `duel_cfg()` helper (a tiny single-lane
arena) to keep outcomes easy to reason about — helpers aren't marked `#[test]`,
they're just functions.

## Doctests: examples that can't rot

The example in `lib.rs`'s doc comment is compiled and run by `cargo test`:

````rust
//! ```
//! use train_core::{BattleConfig, BattleState, Orders, TrainKind};
//! let mut state = BattleState::new(BattleConfig::default());
//! let a = Orders::new().deploy(TrainKind::Armored, 0);
//! train_core::resolve_turn(&mut state, &a, &Orders::new());
//! assert!(state.turn == 1);
//! ```
````

If the public API changes and this example stops compiling, the suite fails — your
docs can't silently drift.

## Running tests

```bash
cargo test -p train-core          # everything
cargo test -p train-core head_on  # only matching names
cargo test -p train-core -- --nocapture  # show println! output
cargo test --doc -p train-core    # just doctests
```

## Exercises

1. Add a test proving an `Armored` train survives more tower fire than an
   `Express` (deploy each into the same lane vs an idle enemy; compare survival).
2. Break combat on purpose — halve `Rocket` damage in `unit.rs` — and watch which
   test fails and how the message localises it. Revert.
3. Add a doctest to `daily_seed_from_unix` in `lib.rs` showing same-day equality.

Next: [Chapter 8 — Determinism & WebAssembly →](./08-determinism-and-wasm.md)
