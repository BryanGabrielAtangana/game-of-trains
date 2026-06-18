# Chapter 5 — Collections & iterators

Idiomatic Rust leans on `Vec` and on **iterators** — lazy, composable pipelines
as fast as hand loops but far easier to read.

## `Vec<T>`: the workhorse

The battle state is mostly `Vec`s (`state.rs`):

```rust
pub struct BattleState {
    pub towers: Vec<Tower>,
    pub trains: Vec<Train>,
    pub switches: [Vec<u8>; 2], // routing choice per faction per node
    // ...
}
```

`arena.rs` builds the graph by pushing into a `Vec`:

```rust
let mut nodes: Vec<NodeData> = Vec::new();
for r in 0..rows {
    for c in 0..cols {
        nodes.push(NodeData { pos: Pos::new(c as i32 * 2, (r as i32 + 1) * 2), exits: [Vec::new(), Vec::new()] });
    }
}
```

## Iterators: describe *what*, not *how*

`apply_damage` in
[`resolve.rs`](../../crates/train-core/src/battle/resolve.rs) snapshots every
train's occupied node with a single chain:

```rust
let occ: Vec<Option<NodeId>> = state
    .trains
    .iter()
    .map(|t| if t.alive() { Some(node_of(t)) } else { None })
    .collect();
```

- `.iter()` borrows each element as `&Train`.
- `.map(...)` transforms each with a closure (`|t| ...` is an anonymous function).
- `.collect()` gathers into the annotated `Vec<Option<NodeId>>`.

## `enumerate`, `filter`, `find`, `max`

Targeting and combat use the toolbox heavily:

```rust
// index + value together
for (i, t) in state.trains.iter().enumerate() { /* ... */ }

// the spawn lists are built by mapping a range
let spawns_a: Vec<NodeId> = (0..cols).map(|c| id(0, c)).collect();

// "nearest enemy in range" keeps a running best
if d <= r2 && best.is_none_or(|(bd, _)| d < bd) { best = Some((d, j)); }

// melee: the biggest enemy damage sharing this node
foe = foe.max(u.kind.stats().damage);
```

The test suite is iterator-heavy too — counting a side's survivors:

```rust
let a = s.trains.iter().filter(|t| t.faction == Faction::A).count();
```

## `retain`: filter in place

Dead trains are removed each tick with one call (`resolve.rs`):

```rust
state.trains.retain(|t| t.alive());
```

`retain` keeps only the elements matching the predicate, dropping the rest —
no manual index juggling.

## `iter()` vs `iter_mut()` vs `into_iter()`

| Call | Yields | Use when |
| ---- | ------ | -------- |
| `v.iter()` | `&T` | read only (most common) |
| `v.iter_mut()` | `&mut T` | modify in place |
| `v.into_iter()` | `T` | consume the `Vec` |

`apply_damage` uses `iter_mut()` to subtract the accumulated damage:

```rust
for (i, t) in state.trains.iter_mut().enumerate() {
    if train_dmg[i] > 0 && t.alive() { t.hp -= train_dmg[i]; }
}
```

## Exercises

1. Add `BattleState::alive_count(&self, f: Faction) -> usize` using
   `.iter().filter(...).count()`.
2. In `arena.rs`, build a `Vec<NodeId>` of every node that is a switch for faction
   A using `(0..nodes.len()).filter(...).collect()`.
3. Replace the manual "nearest" loop in the tower-fire pass with
   `.filter(...).min_by_key(...)`. Does it stay deterministic? (Think about
   tie-breaking — `min_by_key` keeps the *first* minimum.)

Next: [Chapter 6 — Modules, crates & features →](./06-modules-crates-features.md)
