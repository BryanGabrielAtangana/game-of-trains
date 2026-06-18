# Rail Royale on Godot 4 + Rust (gdext)

The cross-platform target (**browser + iOS + Android + desktop**) is **Godot 4**
with the game logic in **Rust** via [gdext](https://github.com/godot-rust/gdext).
Godot owns rendering, input, scenes, animation, audio and exports; Rust owns the
rules. The deterministic engine in [`train-core`](../../crates/train-core) is the
single source of truth and is **reused as a dependency** of the Godot extension
(`crates/rail-royale-gd`) — and, later, natively on the match server.

```
  crates/train-core   ──►  crates/rail-royale-gd  ──►  godot/  (scenes, UI)
  (rules, deterministic)   (gdext: Godot ⇄ Rust)       │ export
                                                        ├─ Web (WASM)
                                                        ├─ Android / iOS
                                                        └─ Desktop
```

## Why this stack

A turn-based 2D game doesn't need Rust for FPS; Rust is here for **deterministic,
cheat-proof rules shared between client and server**. Godot gives mature
cross-platform export + a real 2D/UI toolchain, so we don't hand-roll a renderer.
The Claude-designed prototype (`design/prototype/rail-royale.dc.html`) is the
**visual spec** we port into Godot scenes.

## Status — toolchain spike

✅ **Verified here:** `crates/rail-royale-gd` compiles and links `train-core` into
a GDExtension cdylib on Linux desktop (`cargo build -p rail-royale-gd`). The
bridge **Godot ⇄ Rust ⇄ train-core** is real: `EngineProbe` (a `Node`) and
`RailEngine` (callable from GDScript) spin up a `BattleState` and report it.

⏳ **Needs your machine (not available in CI):** the Godot **editor** and the
**web/Android/iOS export templates + SDKs**. This box has no Godot, no Android
SDK/NDK, and no macOS/Xcode, so the *export* half of the spike runs locally.

> gdext's web + mobile support is **experimental** ("documentation and tooling
> still lacking"). Treat the first web/Android exports as a de-risking step before
> we port the full UI.

## Prerequisites

- **Rust** (stable) + `libclang` (for gdext codegen): `apt install libclang-dev`
  (Linux) / `brew install llvm` (macOS).
- **Godot 4.3+** (4.3 is the floor for web GDExtension, esp. Firefox).
- Per target: emscripten (web), Android SDK/NDK (Android), Xcode on macOS (iOS).

## Run on desktop (the spike)

```bash
cargo build -p rail-royale-gd          # produces target/debug/librail_royale_gd.*
# open the godot/ folder in Godot 4.3+, then press Play (F5)
```

You should see the title + a status line like `train-core online · 3 lanes ·
King 100 HP`, and the unit roster printed to the Godot console (from `EngineProbe`).
If Godot says the extension type is missing, build the lib first, then reopen.

## Export — web (WASM)

gdext web needs an **emscripten** build of the extension and a host that sets
**Cross-Origin Isolation** headers (COOP/COEP) — so **GitHub Pages won't work**
for the threaded build. Use Itch.io, Cloudflare Pages, or Netlify (which can set
headers), or a no-threads build.

```bash
rustup target add wasm32-unknown-emscripten
# install & activate emsdk (matching the version Godot expects), then:
cargo build -p rail-royale-gd --release --target wasm32-unknown-emscripten
# In Godot: Project → Export → Web (enable Thread Support), Export Project.
```

Required headers when self-hosting:
```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

## Export — Android

```bash
rustup target add aarch64-linux-android
# with Android SDK/NDK installed and ANDROID_NDK_HOME set, build per-arch:
cargo build -p rail-royale-gd --release --target aarch64-linux-android
# Godot: install Android build template + export preset, then Export.
```
The `.gdextension` already lists the `android.*.arm64` library paths.

## Export — iOS (needs macOS + Xcode)

```bash
rustup target add aarch64-apple-ios
cargo build -p rail-royale-gd --release --target aarch64-apple-ios
# Godot on a Mac: iOS export preset → Xcode project → TestFlight.
```

## Project layout

- `godot/project.godot` — Godot project (portrait, GL Compatibility for wide
  device/browser support).
- `godot/rail_royale.gdextension` — maps each platform/arch to the built Rust lib
  under `../target/...`.
- `godot/main.tscn` + `main.gd` — the spike scene (replaced as we port the UI).
- `crates/rail-royale-gd/src/lib.rs` — the GDExtension entry + probe classes.

## What's next

1. Port the prototype's **Match screen** into Godot scenes (board, towers, switch
   dials, river, wood control deck, Wind-Up meter, card hand) — design tokens in
   [`docs/design/mvp-brief.md`](../design/mvp-brief.md).
2. Expand `train-core` to the **5-card + states** model (Kamikaze / Engineer /
   Saboteur / Flak / Heavy; Shield / Jam / Overload) and drive the scenes from it.
3. Wire the Rust **AI** (`battle::ai`) as the offline opponent.
4. Later: the Axum/Shuttle match server reusing the same `train-core` for
   server-side verification.
