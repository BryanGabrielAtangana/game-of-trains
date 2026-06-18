# Chapter 4 — `Option` & error handling

Rust has no `null` and no exceptions. "Might be absent" is `Option<T>`; "might
fail" is `Result<T, E>`. Both are ordinary enums (Chapter 2) with special syntax
support. The battle engine leans heavily on `Option`; `Result` arrives with the
match server.

## `Option<T>`: a value, or nothing

`Option<T>` is `Some(T)` or `None`. A train's next node is optional — it's `None`
once the train has reached a terminal (`unit.rs`):

```rust
pub struct Train {
    // ...
    pub to: Option<NodeId>, // None once it has reached the enemy King
}
```

And `Arena::route` (`arena.rs`) returns `None` at a terminal node:

```rust
pub fn route(&self, faction: Faction, node: NodeId, choice: usize) -> Option<NodeId> {
    let exits = &self.nodes[node].exits[faction.index()];
    match exits.len() {
        0 => None,
        n => Some(exits[choice.min(n - 1)]),
    }
}
```

The caller *cannot* use a missing node as if it existed — the type is
`Option<NodeId>`, not `NodeId`.

## Consuming an `Option`

In `move_trains` (`resolve.rs`) the engine asserts an invariant it knows holds:

```rust
let arrived = t.to.expect("active train has a destination");
```

- `.expect(msg)` unwraps `Some` or panics with `msg` — use it for invariants you
  *know* are true; the message documents the assumption.
- Others: `.unwrap_or(default)`, `.map(...)`, `.and_then(...)`, and `if let
  Some(x) = opt { ... }`.

`let ... else` handles the absent case by bailing early (also `resolve.rs`):

```rust
let Some(node) = occ[i] else { continue }; // skip dead/!placed trains
```

## `is_none_or` / `is_some_and`: test the inside

Targeting code keeps the closest in-range candidate without unwrapping:

```rust
fn consider(best: &mut Option<(i64, Target)>, d: i64, r2: i64, target: Target) {
    if d <= r2 && best.as_ref().is_none_or(|(bd, _)| d < *bd) {
        *best = Some((d, target));
    }
}
```

`best.is_none_or(pred)` is `true` when there's no best yet *or* the new candidate
is closer — exactly "take it if it's the first or the nearest".

## `Result<T, E>`: success or a typed failure (coming with the server)

The engine itself can't "fail" — a turn always resolves. But the **match server**
(Phase 3) will validate submissions and return typed errors, e.g.:

```rust
enum SubmitError { NotYourTurn, IllegalOrders, OutOfSync }

fn submit(state: &mut BattleState, who: Faction, orders: &Orders)
    -> Result<Vec<TurnEvent>, SubmitError>
{
    if !is_players_turn(state, who) {
        return Err(SubmitError::NotYourTurn);
    }
    let events = train_core::resolve_turn(state, /* ... */);
    Ok(events)
}
```

- `Result<T, E>` forces every caller to consider failure; the failure modes are
  documented in the type.
- The **`?` operator** unwraps `Ok`/`Some` or returns the error from the enclosing
  function — turning nested matches into linear code:

```rust
let verified = verify(&run)?;   // on Err, return it immediately
```

That `verify(&run)?` pattern is exactly how the old puzzle's replay checker worked
(now archived under `legacy/`), and how the battle server will re-simulate and
validate each turn.

## Exercises

1. Change `move_trains`'s `.expect(...)` to `let Some(arrived) = t.to else {
   continue; }`. Does the logic still hold? Which communicates intent better here?
2. Write `fn first_switch(arena: &Arena, f: Faction) -> Option<NodeId>` returning
   the first node where `arena.is_switch(f, node)` — using `.find(...)` (Chapter 5)
   and returning `Option`.
3. Sketch a `Result`-returning `parse_lane(s: &str) -> Result<usize, SubmitError>`
   using `str::parse` and `.map_err(...)` + `?`.

Next: [Chapter 5 — Collections & iterators →](./05-collections-and-iterators.md)
