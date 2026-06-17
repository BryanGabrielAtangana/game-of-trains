# Chapter 3 — Traits, generics & derive

A **trait** is a set of behaviours a type can implement — Rust's version of
interfaces. **Generics** let code work over any type that implements the traits it
needs. **`derive`** auto-implements common traits for you.

## `derive`: the traits you get for free

Almost every type in `train-core` starts like this (`Train` in `sim.rs`):

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Train { /* ... */ }
```

What each one buys you:

| Trait | Gives you | Used by |
| ----- | --------- | ------- |
| `Clone` | `.clone()` — explicit deep copy | cloning a `Map` in tests |
| `Copy`  | implicit bitwise copy on assignment/pass | `let mut t = self.trains[i];` in `step()` |
| `Debug` | `{:?}` formatting | every `println!("{e:?}")`, test failure messages |
| `PartialEq, Eq` | `==` comparison | `assert_eq!` in tests, `is_finished` logic |

`Copy` requires that all fields are themselves `Copy`. `Train`'s fields are
`u32`/`usize`/`Option<usize>` — all `Copy` — so it qualifies. `Simulation` is
*not* `Copy` (it owns `Vec`s) and only derives `Clone, Debug`.

## Conditional derive with `cfg_attr`

The second attribute is doing something clever:

```rust
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
```

This reads: "*if* the `serde` feature is enabled, also derive `Serialize` and
`Deserialize`." That's how a `Run` becomes JSON to travel from the browser to the
server — but only when serialization is actually wanted. (Cargo features are
Chapter 6.)

## `Default`: sensible zero values

`Scorer` in [`score.rs`](../../crates/train-core/src/score.rs) derives `Default`:

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scorer {
    pub score: i32,
    pub combo: u32,
    pub best_combo: u32,
    pub correct: u32,
    pub total: u32,
}

impl Scorer {
    pub fn new() -> Self {
        Self::default()   // all fields zeroed
    }
}
```

`#[derive(Default)]` generates a `default()` that zero-initialises every field
(because each field's type is itself `Default`). `Scorer::new()` just forwards to
it — a common, tidy idiom.

## Implementing a trait yourself

Sometimes derive isn't enough and you write the impl. Although `train-core` leans
on derives, the standard library's `From` trait shows up implicitly. In `map.rs`:

```rust
_ => Some(n.children[usize::from(active)]),
```

`usize::from(active)` converts a `bool` into `0` or `1` via the standard library's
`impl From<bool> for usize`. Calling `usize::from(...)` is using a trait method.
This is also why you'll see `.into()` around the codebase ecosystem — `into()` is
the mirror image of `from()`, both from the `From`/`Into` trait pair.

## Generics (where you'll meet them next)

`train-core` mostly uses *concrete* types, but you already rely on generic code
from the standard library everywhere:

- `Vec<Train>`, `Vec<bool>`, `Option<usize>` — `Vec<T>` and `Option<T>` are
  generic over the element type `T`.
- `Result<Verified, RejectReason>` in `replay.rs` — `Result<T, E>` is generic over
  both the success and error types.

When you write your own generic function, you constrain the type with **trait
bounds**. A sketch you could add to `train-core`:

```rust
/// Pick the element with the highest score from any iterator of scored items.
fn best_by_score<T, I>(items: I) -> Option<T>
where
    I: IntoIterator<Item = T>,
    T: Copy,
    // ... plus some way to read a score
{ /* ... */ }
```

The `where` clause says "this works for any `T` and any iterator `I`, *as long as*
they satisfy these traits." The function is monomorphised — the compiler stamps
out a specialised version for each concrete type you call it with, so generics are
zero-cost at runtime.

## Exercises

1. Add `Hash` to a type's derive list (try `Pos`) and use it as a key in a
   `std::collections::HashMap<Pos, usize>` in a scratch test. Which other derives
   does `Hash` interact with? (Hint: the compiler will tell you about `Eq`.)
2. Remove `Debug` from `RejectReason` in `replay.rs` and build. Find which
   `println!("{e:?}")` or `assert` breaks, and explain why `Debug` was needed.
3. Write a free function `fn describe<T: std::fmt::Debug>(x: T) -> String` that
   returns `format!("{x:?}")`, and call it on a `NodeKind` and an `Outcome`. You
   just wrote your first generic function with a trait bound.

Next: [Chapter 4 — Error handling →](./04-error-handling.md)
