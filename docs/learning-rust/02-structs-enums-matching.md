# Chapter 2 — Structs, enums & pattern matching

Rust models data with **structs** ("has-a") and **enums** ("is-one-of"). With
`match`, they make illegal states hard to represent and force you to handle every
case.

## Structs

A struct groups fields. `TrainStats` in
[`unit.rs`](../../crates/train-core/src/battle/unit.rs):

```rust
pub struct TrainStats {
    pub hp: i32,
    pub damage: i32,
    pub range: i32,
    pub edge_ticks: u32,
    pub cost: u32,
}

impl TrainStats {
    pub const fn new(hp: i32, damage: i32, range: i32, edge_ticks: u32, cost: u32) -> Self {
        TrainStats { hp, damage, range, edge_ticks, cost }
    }
}
```

- `impl TrainStats { ... }` holds methods/associated functions, separate from the data.
- `new` is an **associated function** (no `self`) — `TrainStats::new(...)`.
- `Self` aliases the type; `const fn` means it can run at compile time.

## Plain enums with behaviour

`TrainKind` (also `unit.rs`) is a simple enumeration that earns its keep with
methods:

```rust
pub enum TrainKind { Express, Armored, Rocket }

impl TrainKind {
    pub const ALL: [TrainKind; 3] = [TrainKind::Express, TrainKind::Armored, TrainKind::Rocket];

    pub fn stats(self) -> TrainStats {
        match self {
            TrainKind::Express => TrainStats::new(20, 8, 0, 6, 3),
            TrainKind::Armored => TrainStats::new(80, 10, 0, 12, 5),
            TrainKind::Rocket  => TrainStats::new(24, 18, 5, 10, 5),
        }
    }
}
```

`Faction` (`mod.rs`), `TowerKind` (`arena.rs`), and `Status` (`state.rs`) are the
same shape.

## Enums that carry data (sum types)

A Rust enum variant can hold different data. `Command` in
[`orders.rs`](../../crates/train-core/src/battle/orders.rs):

```rust
pub enum Command {
    Deploy { kind: TrainKind, lane: usize },
    SetSwitch { node: NodeId, choice: u8 },
}
```

A `Command` is *exactly one* of these. You can't read a `lane` from a `SetSwitch`
— the type makes the meaningless combination unrepresentable. `TurnEvent` in
`resolve.rs` is a richer example (`KingDamaged { faction, amount }`, `Victory {
faction }`, …).

## `match`: exhaustive by force

`apply_orders` in `resolve.rs` destructures commands:

```rust
match *cmd {
    Command::SetSwitch { node, choice } => { /* set routing */ }
    Command::Deploy { kind, lane } => { /* pay steam, spawn */ }
}
```

The `match` must cover **every** variant. Add a new `Command` and this stops
compiling until you handle it — the compiler keeps your logic in sync with your
data. `Command::Deploy { kind, lane }` **destructures** the variant, binding its
fields in one step.

`match` is also an *expression* — `resolve_turn` computes the end-of-match result
straight into `state.status`:

```rust
state.status = match a_hp.cmp(&b_hp) {
    std::cmp::Ordering::Greater => Status::Won(Faction::A),
    std::cmp::Ordering::Less    => Status::Won(Faction::B),
    std::cmp::Ordering::Equal   => Status::Draw,
};
```

## `matches!` and tuple matching

For "is it over?", `state.rs` uses the `matches!` macro:

```rust
pub fn is_over(&self) -> bool {
    !matches!(self.status, Status::Ongoing)
}
```

And `prune_and_check` decides victory by matching a **tuple** of booleans:

```rust
state.status = match (a_dead, b_dead) {
    (true, true)  => Status::Draw,
    (true, false) => Status::Won(Faction::B),
    (false, true) => Status::Won(Faction::A),
    (false, false) => return,
};
```

## Exercises

1. Add a `TrainKind::Saboteur` variant. `cargo build` and let the compiler list
   every `match` you must update (start with `stats`). That list *is* the work.
   Then revert.
2. Write `TowerKind::is_king(self) -> bool` two ways — with `matches!` and with a
   full `match` — and decide which reads better here.
3. In `orders.rs`, add a builder method `Orders::deploy_many(kind, n)` that pushes
   `n` `Deploy` commands. (Structs + methods + a loop.)

Next: [Chapter 3 — Traits, generics & derive →](./03-traits-generics-derive.md)
