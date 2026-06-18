//! # train-client — the Rail Royale web client (WASM)
//!
//! Compiled to `wasm32-unknown-unknown` as a `cdylib`. It holds the whole game
//! in a thread-local [`Client`] and exposes a tiny C ABI the JS loader drives:
//! feed it size/taps/time, then read back a JSON [display list](scene) to paint.
//!
//! There is **no `wasm-bindgen`, no game framework, and no required JS imports** —
//! just `std`, `serde_json`, and the deterministic [`train_core`] engine. That
//! keeps the web boot bulletproof (the lesson learned from the retired puzzle
//! client) while keeping all game and rendering logic in Rust.
//!
//! ## Protocol
//! - `rr_init(w, h)` — create/reset the client at a CSS-pixel size.
//! - `rr_resize(w, h)` — viewport changed.
//! - `rr_pointer(x, y)` — a tap/click at CSS-pixel coords.
//! - `rr_tick(dt_ms)` — advance animation by `dt_ms` milliseconds.
//! - `rr_render() -> len` then `rr_render_ptr() -> *const u8` — the JSON scene
//!   lives in WASM memory at `[ptr, ptr+len)`; decode and execute it.
//!
//! (The `#[no_mangle]` exports below are the crate's only FFI surface; everything
//! else is ordinary safe Rust.)

mod game;
mod scene;

use std::cell::RefCell;

use game::Client;

thread_local! {
    static CLIENT: RefCell<Option<Client>> = const { RefCell::new(None) };
    /// Reusable buffer holding the most recently rendered scene JSON.
    static BUF: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn with_client<R>(f: impl FnOnce(&mut Client) -> R) -> Option<R> {
    CLIENT.with(|c| c.borrow_mut().as_mut().map(f))
}

/// Create or reset the game at the given CSS-pixel size.
#[no_mangle]
pub extern "C" fn rr_init(w: f32, h: f32) {
    CLIENT.with(|c| *c.borrow_mut() = Some(Client::new(w, h)));
}

/// The viewport size changed.
#[no_mangle]
pub extern "C" fn rr_resize(w: f32, h: f32) {
    with_client(|c| c.resize(w, h));
}

/// A pointer/tap at CSS-pixel coordinates.
#[no_mangle]
pub extern "C" fn rr_pointer(x: f32, y: f32) {
    with_client(|c| c.pointer(x, y));
}

/// Advance time-based animation by `dt_ms` milliseconds.
#[no_mangle]
pub extern "C" fn rr_tick(dt_ms: f32) {
    with_client(|c| c.tick(dt_ms));
}

/// Render the current frame into the shared buffer; returns its byte length.
/// Call [`rr_render_ptr`] for the pointer (valid until the next `rr_render`).
#[no_mangle]
pub extern "C" fn rr_render() -> usize {
    let json =
        with_client(|c| serde_json::to_vec(&c.render()).unwrap_or_default()).unwrap_or_default();
    BUF.with(|b| {
        let mut b = b.borrow_mut();
        *b = json;
        b.len()
    })
}

/// Pointer to the buffer filled by the last [`rr_render`].
#[no_mangle]
pub extern "C" fn rr_render_ptr() -> *const u8 {
    BUF.with(|b| b.borrow().as_ptr())
}
