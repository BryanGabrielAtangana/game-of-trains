//! Sound effects.
//!
//! On the web we don't pull in macroquad's audio backend (that would mean
//! re-bundling the quad-snd JS plugin and the version-matched loader we worked
//! hard to avoid). Instead we declare a single host import, `play_sfx(id)`,
//! which a tiny Web Audio shim in `web/index.html` synthesizes. Missing imports
//! are auto-stubbed by the loader, so this can never break startup.
//!
//! Native builds (used for tests / local dev) are a no-op — the web PWA is the
//! shipping target.

/// The sound effects the game can request.
#[derive(Clone, Copy, Debug)]
pub enum Sfx {
    Good,
    Bad,
    Ugly,
    Switch,
    Win,
    Lose,
}

impl Sfx {
    /// Stable id shared with the JS shim. Do not renumber.
    fn id(self) -> u32 {
        match self {
            Sfx::Good => 0,
            Sfx::Bad => 1,
            Sfx::Ugly => 2,
            Sfx::Switch => 3,
            Sfx::Win => 4,
            Sfx::Lose => 5,
        }
    }
}

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn play_sfx(id: u32);
    fn safe_area_top() -> f32;
}

/// Play a sound effect (no-op on native).
pub fn play(sfx: Sfx) {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: `play_sfx` is provided by the host JS plugin; it only reads `id`.
    unsafe {
        play_sfx(sfx.id());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = sfx.id();
    }
}

/// Safe-area top inset in CSS pixels (the notch), from the host. 0 on native.
pub fn safe_area_top_px() -> f32 {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: provided by the host JS plugin; returns a plain number.
    unsafe {
        safe_area_top()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0.0
    }
}
