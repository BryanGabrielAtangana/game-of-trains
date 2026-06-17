# Chapter 2 — Structs, enums & pattern matching

Rust models data with two building blocks: **structs** ("has-a") and **enums**
("is-one-of"). Combined with `match`, they make illegal states hard to represent
and force you to handle every case.

## Structs

A struct groups related fields. See `Pos` in
[`crates/train-core/src/geometry.rs`](../../crates/train-core/src/geometry.rs):

```rust
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub const fn new(x: i32, y: i32) -> Self {
        Pos { x, y }
    }

    pub fn step(self, dir: Direction) -> Pos {
        let (dx, dy) = dir.delta();
        Pos::new(self.x + dx, self.y + dy)
    }
}
```

Things to notice:

- `impl Pos { ... }` is where methods and associated functions live, separate from
  the data definition.
- `new` is an **associated function** (no `self`) — called as `Pos::new(1, 2)`.
- `step` is a **method** (takes `self`) — called as `p.step(dir)`. It takes `self`
  *by value*, which is fine because `Pos` is a small `Copy` type.
- `Self` is an alias for the type the `impl` block is for.
- `const fn` means `new` can run at compile time.

## Enums that carry data

Rust enums are *sum types* — each variant can hold different data. Look at
`NodeKind` in [`crates/train-core/src/map.rs`](../../crates/train-core/src/map.rs):

```rust
pub enum NodeKind {
    Track,                  // internal track (straight or switch)
    Station { label: u32 }, // a numbered destination
    DeadEnd,                // a trap
}
```

`Station` carries a `u32` label; the other variants carry nothing. A `NodeKind` is
*exactly one* of these, and you can't read a `label` without first proving you're
looking at a `Station`. Compare that to a language where you'd have a `type` field
plus a `label` that's meaningless for non-stations — Rust makes the meaningless
combinations unrepresentable.

## `match`: exhaustive by force

The simulation decides a train's fate in `resolve()` in
[`crates/train-core/src/sim.rs`](../../crates/train-core/src/sim.rs):

```rust
let outcome = match self.map.nodes[node].kind {
    NodeKind::DeadEnd => Outcome::Ugly,
    NodeKind::Station { label } => {
        if label == train.dest {
            Outcome::Good
        } else {
            Outcome::Bad
        }
    }
    NodeKind::Track => return, // not actually terminal; nothing to resolve
};
```

Key ideas:

- The `match` must cover **every** variant. If you add a new `NodeKind`, this
  stops compiling until you handle it. That's the compiler keeping your logic in
  sync with your data — a refactoring superpower.
- `NodeKind::Station { label }` **destructures** the variant, binding its inner
  `u32` to `label` in one step.
- `match` is an *expression*: the whole thing evaluates to a value assigned to
  `outcome`.

## `matches!` for quick boolean checks

When you only need "is it this variant?", the `matches!` macro is concise. From
`Node` in `map.rs`:

```rust
pub fn is_switch(&self) -> bool {
    matches!(self.kind, NodeKind::Track) && self.children.len() == 2
}
```

And with a guard, from the test helper in `sim.rs`:

```rust
.find(|&s| matches!(map.nodes[s].kind, NodeKind::Station { label } if label == target_label))
```

The `if label == target_label` is a **match guard** — extra condition on top of
the pattern.

## A simple C-like enum

Not every enum carries data. `Direction` in `geometry.rs` is a plain enumeration,
but it still earns its keep with methods:

```rust
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn flip(self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}
```

`Outcome` in [`score.rs`](../../crates/train-core/src/score.rs) (`Good`/`Bad`/`Ugly`)
is the same shape, and `Scorer::apply` `match`es on it to update the score.

## Exercises

1. Add a new `NodeKind` variant, say `Bridge`, and run `cargo build`. List every
   `match`/`matches!` the compiler flags. That list *is* the set of places your
   game logic must decide what a bridge does. Then revert.
2. In `geometry.rs`, write a method `Direction::turn_right(self) -> Direction`
   using `match`, and a unit test that `North.turn_right() == East`, etc.
3. Rewrite `Node::is_switch` without `matches!`, using a full `match` that returns
   `true`/`false`. Which reads better here, and why?

Next: [Chapter 3 — Traits, generics & derive →](./03-traits-generics-derive.md)
