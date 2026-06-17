# Learning Rust with Game of Trains 🦀🚂

A hands-on Rust course that teaches the language using the **real code in this
repository**. Every concept is shown the way it's actually used in `train-core`,
`train-client`, and `train-server` — not toy snippets — so you learn Rust *and*
understand the project at the same time.

## Who this is for

You know how to program in some language and want to **master Rust** by reading
and writing real code. You don't need prior Rust experience; Chapter 0 starts
from zero.

## How to use it

1. Read a chapter.
2. Open the file it points to (e.g. `crates/train-core/src/map.rs`) and find the
   symbol it discusses. Reading real code is the point.
3. Do the **Exercises** at the end. Most ask you to change code and re-run
   `cargo test`.
4. Keep the [Rust Book](https://doc.rust-lang.org/book/) open as a reference; this
   course is the *applied* companion to it.

> Tip: after any change, run `cargo test --workspace`, `cargo clippy --workspace
> --all-targets`, and `cargo fmt --all`. The compiler and Clippy are the best
> Rust teachers you have — read their messages carefully.

## The learning path

| #  | Chapter | Concepts | Anchored in |
| -- | ------- | -------- | ----------- |
| 0  | [Getting started](./00-getting-started.md) | toolchain, cargo, workspaces, running code | whole repo |
| 1  | [Ownership & borrowing](./01-ownership-and-borrowing.md) | move/borrow, `&`/`&mut`, lifetimes | `rng.rs`, `map.rs`, `sim.rs` |
| 2  | [Structs, enums & pattern matching](./02-structs-enums-matching.md) | `struct`, `enum`, `match`, exhaustiveness | `geometry.rs`, `map.rs`, `score.rs` |
| 3  | [Traits, generics & derive](./03-traits-generics-derive.md) | traits, `derive`, `Copy`/`Clone`, `From` | across `train-core` |
| 4  | [Error handling: `Option` & `Result`](./04-error-handling.md) | `Option`, `Result`, `?`, custom error enums | `map.rs`, `replay.rs` |
| 5  | [Collections & iterators](./05-collections-and-iterators.md) | `Vec`, iterator adapters, closures | `map.rs`, `sim.rs` |
| 6  | [Modules, crates & features](./06-modules-crates-features.md) | module tree, `pub`, workspaces, cargo features | `lib.rs`, `Cargo.toml` |
| 7  | [Testing](./07-testing.md) | unit tests, doctests, property-style testing | every `mod tests` |
| 8  | [Determinism & WebAssembly](./08-determinism-and-wasm.md) | integer math, `wrapping_*`, `cfg`, wasm target | `rng.rs`, `sim.rs` |
| 9  | [Where to go next](./09-next-steps.md) | the path into Phase 2 (macroquad) & Phase 4 (Axum) | client/server crates |

## A note on philosophy

Rust front-loads difficulty: the compiler refuses to build code that other
languages would happily let you run (and crash). That up-front friction is the
language doing your debugging *before* the program runs. This course leans into
that — when something won't compile, the goal is to understand *why* the rule
exists, not just how to silence it.

Start with [Chapter 0 →](./00-getting-started.md)
