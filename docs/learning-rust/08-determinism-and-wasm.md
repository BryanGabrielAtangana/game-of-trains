# Chapter 8 — Determinism & WebAssembly

This chapter ties Rust features to the engine's defining constraint: a turn must
resolve to **bit-for-bit identical results** in the browser (WebAssembly) and on
the server (native). That's what lets the match server re-simulate a submitted
turn and catch cheats.

## Two traps, and how we avoid them

1. **Floating-point math** can differ across compilers/CPUs. So all
   *gameplay* math in `train-core` is **integer-only**: HP, damage, steam and
   ticks are `i32`/`u32`; ranges compare **squared integer distances** (`arena.rs`):

   ```rust
   pub fn dist2(a: Pos, b: Pos) -> i64 {
       let dx = (a.x - b.x) as i64;
       let dy = (a.y - b.y) as i64;
       dx * dx + dy * dy           // compared against range*range — no sqrt, no floats
   }
   ```

   The only `f32` in the engine is `Train::fraction()` — purely for the renderer's
   eyes, never feeding a gameplay decision.

2. **Order of operations.** `resolve_turn` fixes the order *within* a tick:
   movement → shooting → collisions → prune/win (`resolve.rs`'s `step`). And all
   damage in a tick is computed from **one snapshot** before any of it is applied:

   ```rust
   let occ: Vec<Option<NodeId>> = /* every train's node, captured up front */;
   let mut train_dmg = vec![0i32; n];
   // ... accumulate ranged + tower + melee damage into train_dmg ...
   for (i, t) in state.trains.iter_mut().enumerate() {
       if train_dmg[i] > 0 && t.alive() { t.hp -= train_dmg[i]; }
   }
   ```

   So "who shot first" can't bias the result — the tick is a pure function of its
   start state and the orders.

## Explicit wrapping arithmetic

The shared PRNG and the seed mixer use `wrapping_*` so overflow is defined the same
on every platform (`lib.rs`):

```rust
fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}
```

Rust gives a whole family — `wrapping_*` (wrap, intentional), `checked_*` (returns
`Option`), `saturating_*` (clamp). Choosing the right one documents intent and
removes a class of bugs.

## Compiling the engine to WebAssembly

```bash
cargo build -p train-core --target wasm32-unknown-unknown
```

`wasm32-unknown-unknown` is a **target triple**: 32-bit WASM, no vendor, no OS —
just the sandbox. Because `train-core` does **no I/O** and uses only
`core`/`alloc`-level functionality, it compiles to this bare target unchanged.
That "no I/O, no platform code" rule is exactly what makes the *same engine* run in
the browser and on the server.

## Target-specific code with `cfg`

When the client needs platform-specific glue (timing, input), the same
`#[cfg(...)]` mechanism from Chapter 6 selects it:

```rust
#[cfg(target_arch = "wasm32")]
fn now_ms() -> f64 { /* call into JS */ }

#[cfg(not(target_arch = "wasm32"))]
fn now_ms() -> f64 { /* std::time::Instant */ }
```

The engine itself stays platform-agnostic, so it needs none of this.

## The payoff, restated

Integer-only gameplay math + a fixed within-tick order + explicit `wrapping_*` +
a no-I/O core means:

> `resolve_turn` on the server reaches the **exact same state** the player saw in
> their browser — identical code, identical orders, identical result. Cheating
> would require breaking determinism, which the `deterministic_replay` test guards.

This is Rust's philosophy paying off: careful, explicit choices at the type and
arithmetic level buy a guarantee that would be fragile in a language that hides
these details.

## Exercises

1. `grep -rn "f32\|f64" crates/train-core/src`. Confirm the only hit is
   rendering-facing (`Train::fraction`) and never affects a gameplay outcome.
2. Change a `wrapping_mul` in `splitmix64` to `*` and build `--release`. It works —
   now reason about why it's a latent cross-platform bug anyway. Revert.
3. Add a `#[cfg(test)]` test that serializes `BattleState` to JSON with
   `serde_json`, deserializes it, and asserts the round-trip is equal. (That's the
   data that will cross the client/server boundary.)

Next: [Chapter 9 — Where to go next →](./09-next-steps.md)
