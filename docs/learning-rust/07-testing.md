# Chapter 7 — Testing

Testing is built into the language and `cargo`. `train-core` ships with 39 tests
(and counting) because the engine's correctness is the whole product — a bug in
scoring or determinism would corrupt every leaderboard.

## Unit tests live next to the code

The convention is a `tests` submodule at the bottom of each file, compiled only
for tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;   // bring the module's items into scope

    #[test]
    fn basic_points() {
        let mut s = Scorer::new();
        assert_eq!(s.apply(Outcome::Good), 10);
        assert_eq!(s.apply(Outcome::Bad), -10);
    }
}
```

- `#[cfg(test)]` means the whole module is excluded from normal builds — zero cost
  in the shipped binary.
- `use super::*;` imports everything from the parent module (the file you're
  testing), including its **private** items. Unit tests can reach internals.
- `#[test]` marks a function as a test. Run them all with `cargo test`.

## The assertion macros

| Macro | Checks |
| ----- | ------ |
| `assert!(cond)` | `cond` is true |
| `assert_eq!(a, b)` | `a == b` (prints both on failure — needs `Debug`) |
| `assert_ne!(a, b)` | `a != b` |
| `panic!("msg")` | fail immediately (used in unreachable match arms) |

Failure messages can be customised, as in `map.rs`:

```rust
assert!(seen.iter().all(|&s| s), "some nodes unreachable from root");
```

## Property-style testing without a framework

You don't always need fixed inputs. Several `train-core` tests assert
*invariants* over many generated cases — closer to property testing. From
`sim.rs`:

```rust
#[test]
fn finishes_and_delivers_every_train() {
    for level in 1..12 {
        let cfg = GameConfig::new(1000 + level as u64, level);
        let mut sim = Simulation::new(&cfg);
        sim.run_to_end(5_000_000);
        assert!(sim.is_finished(), "level {level} did not finish");
        assert_eq!(sim.scorer().total, cfg.trains, "level {level} lost trains");
    }
}
```

The claim isn't about one map — it's "for *every* level, the tree structure
guarantees every train reaches a leaf." That's a structural property, checked
across a dozen seeds in one test.

## Testing determinism

Determinism is a *correctness requirement* here, so it's tested directly. From
`rng.rs` and `map.rs`:

```rust
#[test]
fn same_seed_same_sequence() {
    let mut a = Rng::new(42);
    let mut b = Rng::new(42);
    for _ in 0..1000 {
        assert_eq!(a.next_u64(), b.next_u64());
    }
}

#[test]
fn deterministic() {
    assert_eq!(map(5), map(5));   // works because Map derives PartialEq
}
```

Comparing whole `Map`s with `==` is only possible because every type in the tree
derives `PartialEq` (Chapter 3). Derives and tests reinforce each other.

## Testing the error path

Good test suites prove failures fail *correctly*. The anti-cheat guarantee is
encoded as a test in `replay.rs`:

```rust
#[test]
fn cheating_is_rejected() {
    let mut run = honest_default_run(7, 5);
    run.claimed_score += 1000;          // forge the score
    match verify(&run) {
        Err(RejectReason::ScoreMismatch { claimed, actual }) => {
            assert_ne!(actual, claimed);
        }
        other => panic!("expected ScoreMismatch, got {other:?}"),
    }
}
```

This test *is* the specification of the feature: "a forged score is rejected with
a `ScoreMismatch`." If someone later breaks verification, this goes red.

## Test helpers

Helper functions inside the `tests` module keep tests readable. `sim.rs` has
`path_switches_to(...)` and `open_path_to(...)` that compute the switch settings
needed to route a train to a chosen station — reused across several tests. Helpers
aren't marked `#[test]`; they're just functions.

## Doctests: examples that can't rot

The example in `lib.rs`'s top doc comment is compiled and run by `cargo test`:

````rust
//! ```
//! use train_core::{GameConfig, Simulation};
//!
//! let config = GameConfig::new(1234, 3);
//! let mut sim = Simulation::new(&config);
//! sim.run_to_end(5_000_000);
//! println!("score: {}", sim.scorer().score);
//! ```
````

If the public API changes and this example stops compiling, the test suite fails.
Your documentation can never silently drift out of date — a uniquely Rust feature.

## Running tests

```bash
cargo test --workspace            # everything
cargo test -p train-core          # one crate
cargo test same_seed              # only tests whose name contains "same_seed"
cargo test -- --nocapture         # show println! output from tests
```

## Exercises

1. Add a test to `score.rs` proving that a `Good` after two `Bad`s scores exactly
   10 (no leftover combo). Run just it with `cargo test combo`.
2. Break something on purpose: change `GOOD_POINTS` to `9` in `score.rs` and run
   the suite. Note *which* tests fail and how the messages help you localise the
   bug. Revert.
3. Add a doctest to `daily_seed` in `lib.rs` showing that the same date yields the
   same seed. Confirm `cargo test --doc` runs it.

Next: [Chapter 8 — Determinism & WebAssembly →](./08-determinism-and-wasm.md)
