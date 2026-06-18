//! A player's committed plan for one turn.

use super::arena::NodeId;
use super::unit::TrainKind;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One instruction within a turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Command {
    /// Spawn a train of `kind` at lane `lane` (clamped to available lanes).
    /// Costs steam; skipped if unaffordable when processed.
    Deploy { kind: TrainKind, lane: usize },
    /// Set this faction's routing choice at a junction (persists across turns).
    SetSwitch { node: NodeId, choice: u8 },
}

/// Everything one player commits for a turn, applied in order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Orders {
    pub commands: Vec<Command>,
}

impl Orders {
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder helper: deploy a train in a lane.
    pub fn deploy(mut self, kind: TrainKind, lane: usize) -> Self {
        self.commands.push(Command::Deploy { kind, lane });
        self
    }

    /// Builder helper: set a switch.
    pub fn switch(mut self, node: NodeId, choice: u8) -> Self {
        self.commands.push(Command::SetSwitch { node, choice });
        self
    }
}
