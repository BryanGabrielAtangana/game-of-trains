# Chapter 8 — Determinism & WebAssembly

This chapter ties Rust features to the project's defining constraint: the engine
must produce **bit-for-bit identical results** in the browser (WebAssembly) and on
the server (native x86). That's what lets the server re-simulate a submitted run
and catch cheats.

## Why determinism is hard (and how we get it)

Two traps break cross-platform reproducibility:

1. **Floating-point math.** `f32`/`f64` results can differ subtly across
   compilers, optimisation levels, and CPUs. So all *scoring-relevant*
   computation in `train-core` is **integer-only**. Ticks are `u32`. Scores are
   `i32`. The simulation never makes a gameplay decision based on a float. The one
   `f32` in the engine, `Train::fraction()`, exists *only* for smooth rendering
   and never feeds back into outcomes:

   ```rust
   pub fn fraction(&self) -> f32 {           // for the renderer's eyes only
       self.progress as f32 / self.edge_ticks as f32
   }
   ```

2. **Undefined integer overflow behaviour.** In release builds, Rust integer
   overflow wraps rather than panicking, but relying on accidental wrapping is a
   bug. When wrapping is *intended* — as in a PRNG — you say so explicitly.

## Explicit wrapping arithmetic in the RNG

Open [`crates/train-core/src/rng.rs`](../../crates/train-core/src/rng.rs). The
generator is built entirely from `wrapping_*` operations:

```rust
sm = sm.wrapping_add(0x9E3779B97F4A7C15);
let mut z = sm;
z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
```

`wrapping_add` / `wrapping_mul` define overflow to wrap modulo 2⁶⁴ on *every*
platform and build profile. There's no UB, no platform variance, and no debug-vs-
release difference. The same seed therefore yields the same `u64` stream in WASM
and native — verified by the `same_seed_same_sequence` test.

Rust gives you a whole family of overflow strategies, each explicit:

| Method | On overflow |
| ------ | ----------- |
| `a + b` | panics in debug, wraps in release (use only when overflow is impossible) |
| `a.wrapping_add(b)` | wraps, always (intentional, as in the RNG) |
| `a.checked_add(b)` | returns `Option` (`None` on overflow) |
| `a.saturating_add(b)` | clamps to the type's max/min |

You can see `saturating_*` used for difficulty in `config.rs`:

```rust
let base = 240u32.saturating_sub(level * 12).max(96); // never underflows past 0
```

Choosing the right one is a Rust habit worth building: it documents your intent
and removes a class of bugs.

## Deterministic ordering

Determinism isn't only arithmetic — *order of operations* must be fixed too.
`Simulation::step` documents and enforces a strict within-tick order (apply inputs
→ spawn → advance), and `verify` replays inputs in sorted tick order:

```rust
let mut inputs = run.inputs.clone();
inputs.sort_by_key(|i| i.tick);
```

So even a payload that arrives shuffled re-simulates identically. (There's a test
for exactly this: `unsorted_inputs_still_verify`.)

## Compiling to WebAssembly

WebAssembly is a portable binary instruction format that runs in every modern
browser at near-native speed. Rust targets it as a first-class platform. The
engine builds for it with one command:

```bash
cargo build -p train-core --target wasm32-unknown-unknown
```

`wasm32-unknown-unknown` is a **target triple**: 32-bit WASM, unknown vendor,
unknown OS (i.e. no operating system at all — just the WASM sandbox). Because
`train-core` does **no I/O** and uses only `core`/`alloc`-level functionality, it
compiles to this bare target with no changes. That "no I/O, no platform code" rule
in the crate's design is precisely what makes it portable.

> The reason it's listed in [`rust-toolchain.toml`](../../rust-toolchain.toml)'s
> `targets` is so this build works on a fresh checkout and in CI without extra
> setup.

## Target-specific code with `cfg`

When you *do* need platform-specific code (you will in the Phase 2 client), the
same `#[cfg(...)]` mechanism from Chapter 6 selects it at compile time:

```rust
#[cfg(target_arch = "wasm32")]
fn now_ms() -> f64 { /* call into JS performance.now() */ }

#[cfg(not(target_arch = "wasm32"))]
fn now_ms() -> f64 { /* use std::time::Instant */ }
```

Only the matching version is compiled. The engine itself avoids needing this by
staying platform-agnostic, but the client will use it for timing and input.

## The payoff, restated

Put together — integer-only gameplay math, explicit `wrapping_*` in the RNG, fixed
operation ordering, and a no-I/O core — these choices mean:

> `verify(run)` on the server reaches the **exact same score** the player saw in
> their browser. Identical code, identical inputs, identical result. Cheating
> requires breaking determinism, which the test suite guards.

This is the clearest example in the project of Rust's philosophy paying off:
careful, explicit choices at the type and arithmetic level buy you a guarantee
that would be fragile or impossible in a language that hides these details.

## Exercises

1. Find every `f32`/`f64` in `train-core` (try `grep -rn "f32\|f64" crates/train-core/src`).
   Confirm each is rendering-only and never influences an `Outcome` or score.
2. Change one `wrapping_mul` in `rng.rs` to a plain `*`. Build in release
   (`cargo build --release`) — it works. Now reason about why that's a latent bug
   even though it didn't panic. Revert.
3. Add a `#[cfg(test)]` test that builds a `Run`, serializes it to JSON with
   `serde_json::to_string`, deserializes it back, and asserts the round-trip is
   equal. (This is the data that will cross the WASM/native boundary.)

Next: [Chapter 9 — Where to go next →](./09-next-steps.md)
