# Chapter 4 — Error handling: `Option` & `Result`

Rust has no `null` and no exceptions. Instead, "might be absent" is `Option<T>`
and "might fail" is `Result<T, E>`. Both are ordinary enums you've already met in
Chapter 2 — the language just gives them special syntax support.

## `Option<T>`: a value, or nothing

`Option<T>` is `Some(T)` or `None`. The map's navigation returns one — a terminal
node has no next step:

```rust
// crates/train-core/src/map.rs
pub fn next(&self, node: usize, active: bool) -> Option<usize> {
    let n = &self.nodes[node];
    match n.children.len() {
        0 => None,                                   // station / dead-end: nowhere to go
        1 => Some(n.children[0]),                    // straight track
        _ => Some(n.children[usize::from(active)]),  // switch: follow the active child
    }
}
```

The caller *cannot* accidentally use a missing value as if it were present — the
type is `Option<usize>`, not `usize`, so they must deal with the `None` case. This
is how Rust abolishes null-pointer bugs.

## Consuming an `Option`

The simulation stores each train's target as `to: Option<usize>` (a train at a
terminal has none). Several idioms appear in `sim.rs`:

```rust
// In step(): a train mid-board is guaranteed to have a destination, so we assert
// the invariant with a clear message if it's ever violated.
let arrived = t.to.expect("active train always has a destination node");

// In resolve(): fall back to the root if somehow absent (defensive).
let node = train.to.unwrap_or(self.map.root);
```

- `.expect(msg)` unwraps `Some`, or panics with `msg`. Use it to assert
  invariants you *know* hold — the message documents the assumption.
- `.unwrap_or(default)` unwraps `Some`, or returns a fallback. No panic.
- There's also `.unwrap_or_else(|| ...)` (lazy default), `.map(...)`,
  `.and_then(...)`, and `if let Some(x) = opt { ... }` for conditional use.

## `is_some_and`: test the inside without unwrapping

From `Simulation::toggle`:

```rust
if self.map.nodes.get(node).is_some_and(|n| n.is_switch()) {
    self.switches[node] = !self.switches[node];
}
```

Two safety nets in one line:

- `self.map.nodes.get(node)` returns `Option<&Node>` — indexing that can't panic
  even if `node` is out of range (compare to `self.map.nodes[node]`, which would).
- `.is_some_and(|n| n.is_switch())` is `true` only when the node exists *and*
  passes the test. An out-of-range or non-switch node is silently ignored — which
  is exactly the behaviour we documented.

## `Result<T, E>`: success or a typed failure

The crown jewel of the engine is run verification in
[`crates/train-core/src/replay.rs`](../../crates/train-core/src/replay.rs). Its
signature tells the whole story:

```rust
pub fn verify(run: &Run) -> Result<Verified, RejectReason> {
```

A caller gets back either `Ok(Verified)` (the authoritative score to store) or
`Err(RejectReason)` explaining *why* it was rejected. The error type is a custom
enum that enumerates the failure modes:

```rust
pub enum RejectReason {
    ScoreMismatch { claimed: i32, actual: i32 },
    MalformedInput,
}
```

Returning errors as values (not exceptions) means the compiler forces every
caller to consider failure, and the failure modes are documented in the type.

## Producing errors early with `return Err(...)`

`verify` bails out the moment something is wrong:

```rust
if run.inputs.len() > MAX_INPUTS {
    return Err(RejectReason::MalformedInput);
}
// ...
if next < inputs.len() && inputs[next].tick < now {
    return Err(RejectReason::MalformedInput);
}
// ...
if final_score.score != run.claimed_score {
    return Err(RejectReason::ScoreMismatch {
        claimed: run.claimed_score,
        actual: final_score.score,
    });
}
Ok(Verified { /* ... */ })
```

## The `?` operator: propagate failures concisely

The server (Phase 4) will chain fallible calls. `?` unwraps an `Ok`/`Some` or
returns the error from the enclosing function. A sketch of how a handler will use
`verify`:

```rust
fn handle_submit(run: &Run) -> Result<Verified, RejectReason> {
    let verified = verify(run)?;   // on Err, returns it immediately
    // ... store verified.score in the database ...
    Ok(verified)
}
```

`?` is the single most common error-handling tool in real Rust; it turns nested
match pyramids into linear, readable code.

## How the tests pin this down

In `replay.rs`'s tests, `cheating_is_rejected` matches on the exact error:

```rust
match verify(&run) {
    Err(RejectReason::ScoreMismatch { claimed, actual }) => {
        assert_eq!(claimed, run.claimed_score);
        assert_ne!(actual, claimed);
    }
    other => panic!("expected ScoreMismatch, got {other:?}"),
}
```

Typed errors make assertions precise: the test doesn't just check "it failed," it
checks *how*.

## Exercises

1. Add a third `RejectReason`, e.g. `UnknownLevel`, and make `verify` return it
   when `run.level == 0`. Watch the test `match` in `cheating_is_rejected` still
   compile (its `other =>` arm catches new variants) — then add a dedicated test.
2. Replace the `.expect(...)` in `step()` with `if let Some(arrived) = t.to {
   ... }`. Does the logic still hold? Which communicates intent better here?
3. Write a function returning `Result<u32, RejectReason>` that parses a level from
   a `&str` (use `str::parse`) and uses `?` to propagate the parse error mapped
   into `RejectReason::MalformedInput`. (Hint: `.map_err(...)`.)

Next: [Chapter 5 — Collections & iterators →](./05-collections-and-iterators.md)
