# Chapter 1 — Ownership & borrowing

This is *the* chapter. Ownership is the idea that makes Rust memory-safe without a
garbage collector. Once it clicks, the rest of the language follows.

## The three rules

1. Every value has exactly one **owner**.
2. When the owner goes out of scope, the value is **dropped** (freed).
3. You can **borrow** a value instead of taking ownership:
   - any number of **shared** borrows `&T` (read-only), **or**
   - exactly one **mutable** borrow `&mut T` (read-write),
   - but never both at the same time.

That last rule — "shared XOR mutable" — is what prevents data races *at compile
time*.

## Borrowing in the wild: returning views

Open [`crates/train-core/src/sim.rs`](../../crates/train-core/src/sim.rs) and find
the read-only accessors on `Simulation`:

```rust
pub fn map(&self) -> &Map {
    &self.map
}
pub fn trains(&self) -> &[Train] {
    &self.trains
}
```

`&self` means "I'm borrowing the `Simulation`, not consuming it." The return type
`&Map` hands the caller a *shared borrow* into the simulation's own data — no copy
is made. The renderer can read `sim.trains()` every frame essentially for free.

Because these return shared borrows, the borrow checker guarantees nobody can be
mutating the simulation while you're reading those slices. You get zero-cost
read access *and* a safety proof.

## `&mut self`: the one who can change things

The only method that changes a `Simulation` mid-game is:

```rust
pub fn toggle(&mut self, node: usize) {
    if self.map.nodes.get(node).is_some_and(|n| n.is_switch()) {
        self.switches[node] = !self.switches[node];
    }
}
```

`&mut self` is an *exclusive* borrow. While `toggle` runs, nothing else can touch
that `Simulation`. The single player action that mutates state is clearly marked
in the type system — you can find every mutation point by searching for
`&mut self`.

## Moves vs. copies

Look at `step()` in the same file:

```rust
for i in 0..self.trains.len() {
    let mut t = self.trains[i];   // copy out
    t.progress += 1;
    // ...
    self.trains[i] = t;           // write back
}
```

This works because `Train` derives `Copy` (see its definition: `#[derive(Clone,
Copy, ...)]`). For `Copy` types, `let mut t = self.trains[i]` makes a bitwise
**copy**; the original stays put. If `Train` held a `Vec` or `String`, it could
*not* be `Copy`, and that line would try to **move** out of a slice — which the
borrow checker forbids (you can't leave a hole in a `Vec`). The fix would be
`self.trains[i].clone()` or indexing differently.

This is a great mental model: **`Copy` = cheap, duplicated automatically;
non-`Copy` = moved, and the compiler tracks who owns it.**

## Lifetimes: borrows that need names

Most borrows are anonymous. Sometimes the compiler needs you to *name* how long a
borrow lasts. See the `Builder` in
[`crates/train-core/src/map.rs`](../../crates/train-core/src/map.rs):

```rust
struct Builder<'r> {
    nodes: Vec<Node>,
    rng: &'r mut Rng,        // a mutable borrow of someone else's Rng
    extend_probability: u32,
    next_row: i32,
}
```

`'r` is a **lifetime parameter**. It says: "a `Builder<'r>` holds a mutable borrow
of an `Rng` that lives at least as long as `'r`." The `Builder` doesn't *own* the
`Rng` — `Map::generate` does — it just borrows it for the duration of generation:

```rust
let mut rng = Rng::new(/* ... */);   // generate() owns rng
let mut b = Builder { rng: &mut rng, /* ... */ };
let root = b.grow(None, 0, config.tree_height);
```

When `generate` returns, `rng` is dropped. The lifetime `'r` guarantees the
`Builder` can't outlive the `Rng` it points at — a dangling pointer is simply not
expressible.

> You'll write explicit lifetimes far less often than you'd fear. The compiler
> *elides* them in the common cases (that's why the accessor methods above didn't
> need one). You write them when a struct stores a borrow, as here.

## Why the recursive builder takes `&mut self`

`grow` is recursive and mutates the shared node arena:

```rust
fn grow(&mut self, parent: Option<usize>, col: i32, depth_left: u32) -> usize {
    let id = self.nodes.len();
    self.nodes.push(Node { /* ... */ });
    // ...
    let a = self.grow(Some(id), col + 1, depth_left - 1);
    let bch = self.grow(Some(id), col + 1, depth_left - 1);
    // ...
}
```

Notice it returns a `usize` **index**, not a `&Node`. This is the classic Rust
pattern for graphs and trees: store nodes in a `Vec` and refer to them by index
("arena allocation"). If `grow` tried to return `&mut Node` references into
`self.nodes` while also pushing new nodes, the borrows would conflict — pushing
can reallocate the `Vec` and invalidate references. Indices sidestep the whole
problem and are how you build trees/graphs ergonomically in Rust.

## Exercises

1. In `sim.rs`, try changing an accessor to return the data by value, e.g.
   `pub fn trains(&self) -> Vec<Train>` returning `self.trains.clone()`. It
   compiles — but why is returning `&[Train]` better for a per-frame renderer?
2. Remove `Copy` from `Train`'s derive list and run `cargo build`. Read the
   error on the `let mut t = self.trains[i];` line. Then put it back.
3. In `map.rs`, imagine `Builder` owned the `Rng` (`rng: Rng`) instead of
   borrowing it. What would `Map::generate` have to change? (Hint: who creates
   the `Rng`, and who needs it afterward?) Try it.

Next: [Chapter 2 — Structs, enums & pattern matching →](./02-structs-enums-matching.md)
