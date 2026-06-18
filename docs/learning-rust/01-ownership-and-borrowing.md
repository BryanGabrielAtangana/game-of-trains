# Chapter 1 — Ownership & borrowing

This is *the* chapter. Ownership makes Rust memory-safe without a garbage
collector. Once it clicks, the rest follows.

## The three rules

1. Every value has exactly one **owner**.
2. When the owner goes out of scope, the value is **dropped** (freed).
3. You can **borrow** instead of taking ownership:
   - any number of shared borrows `&T` (read-only), **or**
   - exactly one mutable borrow `&mut T` (read-write),
   - never both at once.

"Shared XOR mutable" is what prevents data races *at compile time*.

## `&mut` marks who can change state

The turn resolver in
[`crates/train-core/src/battle/resolve.rs`](../../crates/train-core/src/battle/resolve.rs)
takes the world by exclusive reference:

```rust
pub fn resolve_turn(state: &mut BattleState, a: &Orders, b: &Orders) -> Vec<TurnEvent>
```

`&mut BattleState` means *only* `resolve_turn` can mutate the battle while it runs;
`&Orders` are read-only views of each player's plan — they aren't copied or
consumed, just borrowed. You can find every place the world changes by searching
for `&mut BattleState`.

## Moves vs. copies

Look at `move_trains` in `resolve.rs`:

```rust
for i in 0..state.trains.len() {
    let mut t = state.trains[i];   // copy out
    t.progress += 1;
    // ...
    state.trains[i] = t;           // write back
}
```

This works because `Train` derives `Copy` (see `unit.rs`:
`#[derive(Clone, Copy, ...)]`). For `Copy` types `let mut t = state.trains[i]`
makes a bitwise **copy**; the original stays in the `Vec`. If `Train` held a `Vec`
or `String` it could *not* be `Copy`, and that line would try to **move** out of a
slice — which the borrow checker forbids (you can't leave a hole in a `Vec`).

Mental model: **`Copy` = cheap, duplicated automatically; non-`Copy` = moved, and
the compiler tracks who owns it.**

## Borrowing fields without conflict

Inside that same loop the code reads other parts of `state` while preparing to
write `state.trains`:

```rust
let arrived = t.to.expect("active train has a destination");
let choice = state.switches[t.faction.index()][arrived] as usize;
t.to = state.arena.route(t.faction, arrived, choice);  // borrows state.arena
state.trains[i] = t;                                   // then writes state.trains
```

`t` is a local **copy**, so the read of `state.arena`/`state.switches` and the
later write to `state.trains` don't overlap in time — no aliasing, no borrow
error. Copying small values to sidestep borrow conflicts is a common, idiomatic move.

## Shared borrows: reading the graph

`Arena::route` in
[`arena.rs`](../../crates/train-core/src/battle/arena.rs) takes `&self`:

```rust
pub fn route(&self, faction: Faction, node: NodeId, choice: usize) -> Option<NodeId> {
    let exits = &self.nodes[node].exits[faction.index()];
    // ...
}
```

`&self` borrows the arena read-only and `&self.nodes[node].exits[...]` hands back a
*shared borrow into* the arena's own data — no copy. Many callers can read the
arena at once; none can mutate it while they do.

## The arena pattern (graphs without lifetime pain)

Notice the graph stores relationships as **indices** (`NodeId = usize`), not
references:

```rust
pub struct NodeData { pub pos: Pos, pub exits: [Vec<NodeId>; 2] }
```

A node refers to its neighbours by index. If nodes held `&NodeData` references
instead, pushing to the `Vec` could reallocate and invalidate them — a classic
fight with the borrow checker. Indices ("arena allocation") sidestep it entirely
and are how you build graphs ergonomically in Rust.

## Exercises

1. Remove `Copy` from `Train`'s derive in `unit.rs` and run `cargo build`. Read
   the error on `let mut t = state.trains[i];`, then put it back.
2. In `move_trains`, try to keep a reference `let t = &mut state.trains[i];` and
   *also* call `state.arena.route(...)` in the same scope. Watch the borrow
   checker object, and explain why the copy-based version avoids it.
3. Why does `route` return `Option<NodeId>` instead of `NodeId`? (Chapter 4.)

Next: [Chapter 2 — Structs, enums & matching →](./02-structs-enums-matching.md)
