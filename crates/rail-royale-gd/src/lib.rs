//! Rail Royale — Godot 4 / gdext entry point.
//!
//! This crate is the **presentation layer only**. All game rules live in
//! [`train_core`] (the deterministic engine, shared with the future match
//! server); Godot owns rendering, input, scenes, animation and cross-platform
//! export. This file is the toolchain **spike**: it proves the bridge
//! Godot ⇄ Rust ⇄ `train-core` compiles and links into a GDExtension, before we
//! port the prototype's UI into Godot scenes.

use godot::prelude::*;

/// Registers this crate as a GDExtension. `entry_symbol = "gdext_rust_init"` in
/// `rail_royale.gdextension` points Godot here.
struct RailRoyaleExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RailRoyaleExtension {}

/// A throwaway probe node: on `ready()` it spins up a `train_core` battle and
/// logs it, proving the engine is linked and callable from inside Godot. It will
/// be replaced by the real board/HUD nodes as we build the UI.
#[derive(GodotClass)]
#[class(base=Node)]
struct EngineProbe {
    base: Base<Node>,
}

#[godot_api]
impl INode for EngineProbe {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }

    fn ready(&mut self) {
        use train_core::{BattleConfig, BattleState, Faction, TrainKind};

        let state = BattleState::new(BattleConfig::default());
        godot_print!(
            "[Rail Royale] train-core online — board {}x{} lanes, King HP {}",
            state.arena.cols,
            state.arena.rows,
            state.king_hp(Faction::A),
        );
        for kind in TrainKind::ALL {
            let s = kind.stats();
            godot_print!(
                "  unit {:?}: cost {}  hp {}  dmg {}  range {}",
                kind,
                s.cost,
                s.hp,
                s.damage,
                s.range
            );
        }
    }
}

/// Exposed to GDScript so a scene can show "engine is alive" without Rust UI yet.
#[derive(GodotClass)]
#[class(base=RefCounted)]
struct RailEngine {
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for RailEngine {
    fn init(base: Base<RefCounted>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl RailEngine {
    /// A one-line status string a Label can display to confirm the Rust ⇄ Godot
    /// bridge works end to end.
    #[func]
    fn status(&self) -> GString {
        use train_core::{BattleConfig, BattleState, Faction};
        let state = BattleState::new(BattleConfig::default());
        format!(
            "train-core online · {} lanes · King {} HP",
            state.arena.cols,
            state.king_hp(Faction::A)
        )
        .as_str()
        .into()
    }
}
