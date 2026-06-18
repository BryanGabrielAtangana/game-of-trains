# Learning Rust with Rail Royale 🦀🚂⚔️

A hands-on Rust course that teaches the language using the **real code in this
repository** — the deterministic battle engine in `crates/train-core`. Every
concept is shown the way it's actually used, so you learn Rust *and* understand
the engine at the same time.

## Who this is for

You can program in some language and want to **master Rust** by reading and
writing real code. No prior Rust needed; Chapter 0 starts from zero.

## How to use it

1. Read a chapter.
2. Open the file it points to (e.g. `crates/train-core/src/battle/resolve.rs`) and
   find the symbol it discusses. Reading real code is the point.
3. Do the **Exercises** — most ask you to change code and re-run `cargo test`.
4. Keep the [Rust Book](https://doc.rust-lang.org/book/) open as a reference; this
   course is its *applied* companion.

> After any change run `cargo test -p train-core`, `cargo clippy --workspace
> --all-targets`, and `cargo fmt --all`. The compiler and Clippy are the best
> Rust teachers you have — read their messages carefully.

## The learning path

| #  | Chapter | Concepts | Anchored in |
| -- | ------- | -------- | ----------- |
| 0  | [Getting started](./00-getting-started.md) | toolchain, cargo, workspaces | whole repo |
| 1  | [Ownership & borrowing](./01-ownership-and-borrowing.md) | move/borrow, `&`/`&mut`, `Copy` | `resolve.rs`, `arena.rs` |
| 2  | [Structs, enums & matching](./02-structs-enums-matching.md) | `struct`, `enum`, `match` | `mod.rs`, `unit.rs`, `orders.rs` |
| 3  | [Traits, generics & derive](./03-traits-generics-derive.md) | traits, `derive`, `Default` | across `battle/` |
| 4  | [`Option` & error handling](./04-error-handling.md) | `Option`, `?`, future `Result` | `arena.rs`, `resolve.rs` |
| 5  | [Collections & iterators](./05-collections-and-iterators.md) | `Vec`, iterator adapters, closures | `resolve.rs`, `arena.rs` |
| 6  | [Modules, crates & features](./06-modules-crates-features.md) | module tree, `pub use`, features | `mod.rs`, `lib.rs`, `Cargo.toml` |
| 7  | [Testing](./07-testing.md) | unit tests, doctests, property-style | `battle/mod.rs`, `lib.rs` |
| 8  | [Determinism & WebAssembly](./08-determinism-and-wasm.md) | integer math, `wrapping_*`, wasm | `resolve.rs`, `rng.rs`, `lib.rs` |
| 9  | [Where to go next](./09-next-steps.md) | the path to vs-AI & the match server | the roadmap |

## Philosophy

Rust front-loads difficulty: the compiler refuses code other languages would let
you run (and crash). That friction is the language debugging your program *before*
it runs. When something won't compile, the goal is to understand *why* the rule
exists — not just to silence it.

Start with [Chapter 0 →](./00-getting-started.md)
