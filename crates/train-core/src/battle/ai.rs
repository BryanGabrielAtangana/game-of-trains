//! A heuristic opponent — Phase 2 of Rail Royale (see `docs/design/rail-royale.md`).
//!
//! [`ai_orders`] looks at a [`BattleState`] and returns the [`Orders`] a computer
//! player commits this turn. It is **pure and deterministic**: the same state +
//! difficulty always yields the same plan, so it can drive offline balance
//! testing, replays, and an onboarding tutorial — and two AIs playing each other
//! always reach the same result (see the tests).
//!
//! The heuristic is small on purpose:
//! 1. **Counter-pick** the kind the enemy is fielding most (the counter-triangle:
//!    Armored beats Express, Rocket beats Armored, Express beats Rocket).
//! 2. **Defend** the lane where an enemy train is closest to our King.
//! 3. **Spend steam** down (never over budget), deploying the counter across lanes.
//! 4. On [`AiLevel::Hard`], also **route switches toward the enemy King**.

use super::arena::NodeId;
use super::state::BattleState;
use super::unit::TrainKind;
use super::{Faction, Orders};
use crate::geometry::Pos;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// How strong (and how greedy) the computer opponent plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AiLevel {
    /// Deploys a single cheap unit per turn; ignores the counter-triangle. A
    /// gentle sparring partner for onboarding.
    Easy,
    /// Counter-picks, defends the threatened lane, and spends most of its steam.
    Normal,
    /// Like Normal, and also routes its switches toward the enemy King.
    Hard,
}

impl AiLevel {
    /// Every level, in increasing strength.
    pub const ALL: [AiLevel; 3] = [AiLevel::Easy, AiLevel::Normal, AiLevel::Hard];
}

/// Choose `me`'s plan for the current turn. Pure: no I/O, no randomness, no clock.
pub fn ai_orders(state: &BattleState, me: Faction, level: AiLevel) -> Orders {
    if state.is_over() {
        return Orders::new();
    }

    let f = me.index();
    let mut budget = state.steam[f];
    let lanes = state.arena.spawns[f].len().max(1);
    let mut orders = Orders::new();

    // Hard play first commits routing: send every lane toward the enemy King.
    if level == AiLevel::Hard {
        for &spawn in &state.arena.spawns[f] {
            if state.arena.is_switch(me, spawn) {
                orders = orders.switch(spawn, route_toward_king(state, me, spawn));
            }
        }
    }

    let counter = counter_pick(state, me, level);
    let cost = counter.stats().cost;

    // Easy: one cheap unit and done.
    if level == AiLevel::Easy {
        let cheap = cheapest_affordable(budget);
        if let Some(kind) = cheap {
            let lane = threat_lane(state, me).unwrap_or(0);
            orders = orders.deploy(kind, lane);
        }
        return orders;
    }

    // Normal / Hard: defend the threatened lane first, then pressure across lanes.
    let mut next_lane = 0usize;
    if let Some(lane) = threat_lane(state, me) {
        if budget >= cost {
            orders = orders.deploy(counter, lane);
            budget -= cost;
        }
    }
    while budget >= cost {
        orders = orders.deploy(counter, next_lane % lanes);
        budget -= cost;
        next_lane += 1;
    }

    orders
}

/// The kind that best counters what the enemy is fielding most right now.
/// With an empty board, opens with a solid Armored (Easy is handled separately).
fn counter_pick(state: &BattleState, me: Faction, _level: AiLevel) -> TrainKind {
    let enemy = me.enemy();
    let (mut express, mut armored, mut rocket) = (0u32, 0u32, 0u32);
    for t in &state.trains {
        if t.faction != enemy || !t.alive() {
            continue;
        }
        match t.kind {
            TrainKind::Express => express += 1,
            TrainKind::Armored => armored += 1,
            TrainKind::Rocket => rocket += 1,
        }
    }

    // No enemy units yet: open with the value unit.
    if express == 0 && armored == 0 && rocket == 0 {
        return TrainKind::Armored;
    }

    // Counter-triangle, ties broken toward the more common threat.
    let max = express.max(armored).max(rocket);
    if armored == max {
        TrainKind::Rocket // artillery shells the tank
    } else if express == max {
        TrainKind::Armored // tank stops the rush
    } else {
        TrainKind::Express // speed runs down the fragile rockets
    }
}

/// The cheapest train kind affordable within `budget`, if any.
fn cheapest_affordable(budget: u32) -> Option<TrainKind> {
    TrainKind::ALL
        .into_iter()
        .filter(|k| k.stats().cost <= budget)
        .min_by_key(|k| k.stats().cost)
}

/// Lane (index into our spawns) of the enemy train closest to our King, if any —
/// i.e. where to put a defender.
fn threat_lane(state: &BattleState, me: Faction) -> Option<usize> {
    let enemy = me.enemy();
    let king_pos = state.arena.pos(state.arena.kings[me.index()]);

    let threat = state
        .trains
        .iter()
        .filter(|t| t.faction == enemy && t.alive())
        .map(|t| occupied_pos(state, t.from, t.to, t.progress, t.edge_ticks))
        .min_by_key(|p| dist2(*p, king_pos))?;

    let spawns = &state.arena.spawns[me.index()];
    (0..spawns.len()).min_by_key(|&i| {
        let dx = (state.arena.pos(spawns[i]).x - threat.x) as i64;
        dx * dx
    })
}

/// On a switch, the choice index whose target column is nearest the enemy King's
/// column — "head for the throne".
fn route_toward_king(state: &BattleState, me: Faction, node: NodeId) -> u8 {
    let king_x = state.arena.pos(state.arena.kings[me.enemy().index()]).x;
    let exits = &state.arena.nodes[node].exits[me.index()];
    let mut best = 0u8;
    let mut best_dx = i64::MAX;
    for (i, &target) in exits.iter().enumerate() {
        let dx = (state.arena.pos(target).x - king_x) as i64;
        let d = dx * dx;
        if d < best_dx {
            best_dx = d;
            best = i as u8;
        }
    }
    best
}

/// The position a train currently occupies (its near node along the edge).
fn occupied_pos(
    state: &BattleState,
    from: NodeId,
    to: Option<NodeId>,
    progress: u32,
    edge_ticks: u32,
) -> Pos {
    match to {
        Some(to) if progress * 2 >= edge_ticks => state.arena.pos(to),
        _ => state.arena.pos(from),
    }
}

/// Squared distance (integer, deterministic).
fn dist2(a: Pos, b: Pos) -> i64 {
    let dx = (a.x - b.x) as i64;
    let dy = (a.y - b.y) as i64;
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::{resolve_turn, BattleConfig, Command, Status};

    fn cfg() -> BattleConfig {
        BattleConfig::default()
    }

    /// Total steam an order set would spend.
    fn spend(orders: &Orders) -> u32 {
        orders
            .commands
            .iter()
            .map(|c| match c {
                Command::Deploy { kind, .. } => kind.stats().cost,
                Command::SetSwitch { .. } => 0,
            })
            .sum()
    }

    #[test]
    fn never_overspends() {
        for level in AiLevel::ALL {
            let s = BattleState::new(cfg());
            let o = ai_orders(&s, Faction::A, level);
            assert!(
                spend(&o) <= s.steam[Faction::A.index()],
                "{level:?} overspent"
            );
        }
    }

    #[test]
    fn broke_ai_does_not_deploy() {
        let mut s = BattleState::new(cfg());
        s.steam = [0, 0];
        for level in AiLevel::ALL {
            let o = ai_orders(&s, Faction::A, level);
            assert!(
                !o.commands
                    .iter()
                    .any(|c| matches!(c, Command::Deploy { .. })),
                "{level:?} deployed with no steam"
            );
        }
    }

    #[test]
    fn deterministic_same_state_same_plan() {
        let s = BattleState::new(cfg());
        for level in AiLevel::ALL {
            assert_eq!(
                ai_orders(&s, Faction::A, level),
                ai_orders(&s, Faction::A, level)
            );
        }
    }

    #[test]
    fn counters_an_armored_push_with_rockets() {
        // Give B a couple of Armored trains; Normal AI for A should answer Rocket.
        let mut s = BattleState::new(cfg());
        for _ in 0..2 {
            s.trains.push(super::super::unit::Train {
                faction: Faction::B,
                kind: TrainKind::Armored,
                hp: TrainKind::Armored.stats().hp,
                from: s.arena.spawns[Faction::B.index()][0],
                to: None,
                progress: 0,
                edge_ticks: TrainKind::Armored.stats().edge_ticks,
            });
        }
        let o = ai_orders(&s, Faction::A, AiLevel::Normal);
        assert!(
            o.commands.iter().any(|c| matches!(
                c,
                Command::Deploy {
                    kind: TrainKind::Rocket,
                    ..
                }
            )),
            "should counter Armored with Rocket"
        );
    }

    #[test]
    fn easy_is_no_greedier_than_normal() {
        let s = BattleState::new(cfg());
        let easy = spend(&ai_orders(&s, Faction::A, AiLevel::Easy));
        let normal = spend(&ai_orders(&s, Faction::A, AiLevel::Normal));
        assert!(
            easy <= normal,
            "Easy ({easy}) should not outspend Normal ({normal})"
        );
    }

    #[test]
    fn hard_sets_routing_switches() {
        let s = BattleState::new(cfg()); // default arena has switches at spawns
        let o = ai_orders(&s, Faction::A, AiLevel::Hard);
        assert!(
            o.commands
                .iter()
                .any(|c| matches!(c, Command::SetSwitch { .. })),
            "Hard should route its switches"
        );
    }

    #[test]
    fn two_ais_reach_a_deterministic_terminal() {
        // The capstone: AIs playing each other always finish, and identically.
        let play = || {
            let mut s = BattleState::new(cfg());
            while !s.is_over() {
                let a = ai_orders(&s, Faction::A, AiLevel::Hard);
                let b = ai_orders(&s, Faction::B, AiLevel::Normal);
                resolve_turn(&mut s, &a, &b);
            }
            s.status
        };
        let r1 = play();
        let r2 = play();
        assert_eq!(r1, r2, "self-play must be deterministic");
        assert!(
            matches!(r1, Status::Won(_) | Status::Draw),
            "must terminate"
        );
    }

    /// Play a full match (A = `a`, B = `b`) to its terminal status.
    fn play_match(a: AiLevel, b: AiLevel) -> Status {
        let mut s = BattleState::new(cfg());
        while !s.is_over() {
            let oa = ai_orders(&s, Faction::A, a);
            let ob = ai_orders(&s, Faction::B, b);
            resolve_turn(&mut s, &oa, &ob);
        }
        s.status
    }

    #[test]
    fn stronger_difficulty_wins_and_mirrors_draw() {
        // The difficulty ladder is real on the (balanced) default config, and a
        // guard against future stat/economy regressions making games stall.
        assert_eq!(
            play_match(AiLevel::Normal, AiLevel::Easy),
            Status::Won(Faction::A)
        );
        assert_eq!(
            play_match(AiLevel::Hard, AiLevel::Easy),
            Status::Won(Faction::A)
        );
        assert_eq!(
            play_match(AiLevel::Hard, AiLevel::Normal),
            Status::Won(Faction::A)
        );
        // Symmetric play between equals is a draw.
        assert_eq!(play_match(AiLevel::Normal, AiLevel::Normal), Status::Draw);
    }
}
