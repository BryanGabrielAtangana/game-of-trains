# Chapter 3 — Traits, generics & derive

A **trait** is a set of behaviours a type can implement (Rust's interfaces).
**Generics** let code work over any type satisfying the traits it needs.
**`derive`** auto-implements common traits.

## `derive`: traits for free

Almost every battle type starts like this (`BattleState` in
[`state.rs`](../../crates/train-core/src/battle/state.rs)):

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BattleState { /* ... */ }
```

| Trait | Gives you | Used by |
| ----- | --------- | ------- |
| `Clone` | `.clone()` — explicit deep copy | re-simulating a turn from a snapshot |
| `Copy`  | implicit bitwise copy | `Train`, `Tower`, `TrainStats` (small, all-`Copy` fields) |
| `Debug` | `{:?}` formatting | every test assertion's failure message |
| `PartialEq, Eq` | `==` | the determinism test `assert_eq!(s1, s2)` |

`Train` adds `Copy` because all its fields are `Copy`; `BattleState` owns `Vec`s,
so it's `Clone` but **not** `Copy` — a useful distinction to read off the derives.

## Conditional derive with `cfg_attr`

```rust
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
```

"*If* the `serde` feature is on, also derive `Serialize`/`Deserialize`." That's how
`Orders` and `BattleState` will turn into JSON to travel between the client and the
match server — but only when serialization is wanted (Chapter 6).

## `Default`: sensible starting values

`BattleConfig` hand-writes `Default` to encode the standard match settings
(`arena.rs`):

```rust
impl Default for BattleConfig {
    fn default() -> Self {
        BattleConfig { seed: 1, cols: 3, rows: 4, ticks_per_turn: 24, /* ... */ }
    }
}
```

`Orders` instead *derives* `Default` (an empty command list) and offers a builder:

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Orders { pub commands: Vec<Command> }

let plan = Orders::new().switch(node, 1).deploy(TrainKind::Rocket, 0);
```

Each builder method takes `self` by value and returns `Self`, so calls chain.

## Generics you already rely on

The engine uses mostly concrete types, but leans on generic std types everywhere:

- `Vec<Train>`, `Vec<Tower>`, `[Vec<NodeId>; 2]` — `Vec<T>` is generic over `T`.
- `Option<NodeId>` (a train's next node) — `Option<T>`.
- `resolve_turn` returns `Vec<TurnEvent>`.

When you write your *own* generic code you constrain types with **trait bounds**:

```rust
fn describe<T: std::fmt::Debug>(x: T) -> String { format!("{x:?}") }
// describe(TrainKind::Rocket) and describe(Status::Draw) both work.
```

Generics are monomorphised — the compiler stamps out a specialised copy per
concrete type, so they're zero-cost at runtime.

## Exercises

1. Add `Hash` to `Faction`'s derives (it already has it) and use a
   `std::collections::HashMap<Faction, i32>` in a scratch test to tally each side's
   trains. Which other derive does `Hash` lean on? (`Eq`.)
2. Remove `Debug` from `TurnEvent` and build — find which test/`println!` breaks
   and explain why `Debug` was needed. Revert.
3. Write `fn total_cost<I: IntoIterator<Item = TrainKind>>(kinds: I) -> u32` that
   sums `kind.stats().cost`. Your first generic function with a bound.

Next: [Chapter 4 — `Option` & error handling →](./04-error-handling.md)
