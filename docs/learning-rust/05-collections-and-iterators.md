# Chapter 5 — Collections & iterators

Idiomatic Rust leans hard on `Vec` and on **iterators** — lazy, composable
sequence pipelines that are as fast as hand-written loops but far easier to read.

## `Vec<T>`: the workhorse

The entire map is a `Vec<Node>` with relationships stored as indices (the arena
pattern from Chapter 1). From `map.rs`:

```rust
pub struct Map {
    pub nodes: Vec<Node>,
    pub root: usize,
    pub stations: Vec<usize>,
    pub labels: Vec<u32>,
    // ...
}
```

Building a `Vec` imperatively (also `map.rs`):

```rust
let mut stations = Vec::new();
let mut labels = Vec::new();
for &leaf in &leaves {
    // ...
    stations.push(leaf);
    labels.push(next_label);
}
```

Note `for &leaf in &leaves`: iterating `&leaves` yields `&usize`, and the `&leaf`
pattern *destructures* the reference to copy out a plain `usize`. Small `Copy`
values are nicer to handle by value.

## Iterators: describe *what*, not *how*

Compare the imperative loop above with the declarative style used to find all
leaves:

```rust
let mut leaves: Vec<usize> = (0..nodes.len())
    .filter(|&i| nodes[i].is_terminal())
    .collect();
```

Read it left to right: take the range of indices `0..nodes.len()`, keep those
whose node `is_terminal()`, and `collect()` the result into a `Vec<usize>`. No
index bookkeeping, no manual `push`. The closure `|&i| ...` is an anonymous
function; `&i` again destructures the `&usize` the iterator yields.

`collect()` is generic over the target collection — the type annotation
`: Vec<usize>` tells it what to build.

## Counting and searching

`train-client/src/main.rs` summarises a map with iterator chains:

```rust
let switches = map.nodes.iter().filter(|n| n.is_switch()).count();
let dead_ends = map
    .nodes
    .iter()
    .filter(|n| n.kind == NodeKind::DeadEnd)
    .count();
```

- `.iter()` borrows each element as `&Node`.
- `.filter(...)` keeps matches.
- `.count()` consumes the iterator and returns how many remained.

To find *one* element, use `.find(...)` or `.position(...)`. From `sim.rs`:

```rust
let sw = m.nodes.iter().position(|n| n.is_switch()).unwrap();
```

`.position(...)` returns `Option<usize>` — the index of the first match, or
`None`. (Iterators and `Option` go hand in hand.)

## Adjacent pairs with `windows`

A schedule's spawn ticks must be non-decreasing. The test in `sim.rs` checks every
adjacent pair at once:

```rust
assert!(sched.windows(2).all(|w| w[1].tick >= w[0].tick));
```

`.windows(2)` yields overlapping slices `[a, b]`, `[b, c]`, …; `.all(...)` returns
`true` only if the predicate holds for every one. Expressing "is this sequence
sorted?" in a single line is peak iterator style.

## Sorting by a key

Labels must be reproducible, so leaves are sorted before labelling (`map.rs`):

```rust
leaves.sort_by_key(|&i| (nodes[i].pos.x, nodes[i].pos.y));
```

`sort_by_key` sorts in place by a derived key — here a `(x, y)` tuple, which sorts
lexicographically (by `x`, then `y`). Tuples implementing `Ord` for free is what
makes this one-liner work.

## Building a `Vec` with known capacity

When you know roughly how many items you'll push, pre-allocate (`sim.rs`):

```rust
let mut schedule = Vec::with_capacity(config.trains as usize);
```

`with_capacity` avoids repeated reallocation as the `Vec` grows. A small but real
performance habit.

## `iter()` vs `into_iter()` vs `iter_mut()`

Three ways to iterate, differing in ownership:

| Call | Yields | Use when |
| ---- | ------ | -------- |
| `v.iter()` | `&T` | you only need to read (most common) |
| `v.iter_mut()` | `&mut T` | you need to modify in place |
| `v.into_iter()` | `T` | you're done with the `Vec` and want to consume it |

The codebase mostly uses `.iter()` (read-only summaries and checks). When you
write the renderer in Phase 2, you'll reach for `.iter()` on `sim.trains()` every
frame.

## Exercises

1. Rewrite the imperative station/label loop in `map.rs` to compute `labels` with
   an iterator chain over `stations`. (Hint: `(1..=stations.len() as u32)` or
   `.enumerate()`.) Keep the tests green.
2. In `train-client/main.rs`, add a line that prints the *deepest* node's `x`
   coordinate using `map.nodes.iter().map(|n| n.pos.x).max()`. Why does `max()`
   return an `Option`?
3. Use `.filter(...).map(...).collect::<Vec<_>>()` to build a `Vec` of all station
   labels that are even. What does the `_` in `Vec<_>` ask the compiler to do?

Next: [Chapter 6 — Modules, crates & features →](./06-modules-crates-features.md)
