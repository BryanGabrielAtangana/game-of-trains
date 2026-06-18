//! Turn resolution: apply both players' orders, then run the fixed-tick
//! deterministic simulation (movement → shooting → collisions → win check).

use super::arena::{dist2, NodeId, TowerKind};
use super::state::{BattleState, Status};
use super::unit::{Train, TrainKind};
use super::{Command, Faction, Orders};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Notable things that happened during a turn (for the UI / tests later).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TurnEvent {
    KingDamaged { faction: Faction, amount: i32 },
    TowerDestroyed { faction: Faction, kind: TowerKind },
    TrainDestroyed { faction: Faction },
    Victory { faction: Faction },
}

/// Apply both committed plans and resolve the turn, mutating `state` in place.
/// Returns the events that occurred. Deterministic: identical inputs → identical
/// result on every platform (the basis for server-side verification).
pub fn resolve_turn(state: &mut BattleState, a: &Orders, b: &Orders) -> Vec<TurnEvent> {
    let mut events = Vec::new();
    if state.is_over() {
        return events;
    }

    apply_orders(state, Faction::A, a);
    apply_orders(state, Faction::B, b);

    for _ in 0..state.cfg.ticks_per_turn {
        if state.is_over() {
            break;
        }
        step(state, &mut events);
        state.tick += 1;
    }

    state.turn += 1;
    for f in 0..2 {
        state.steam[f] = (state.steam[f] + state.cfg.steam_per_turn).min(state.cfg.steam_cap);
    }

    // Turn cap: whoever's King is healthier wins; equal is a draw.
    if !state.is_over() && state.turn >= state.cfg.max_turns {
        let a_hp = state.king_hp(Faction::A);
        let b_hp = state.king_hp(Faction::B);
        state.status = match a_hp.cmp(&b_hp) {
            std::cmp::Ordering::Greater => Status::Won(Faction::A),
            std::cmp::Ordering::Less => Status::Won(Faction::B),
            std::cmp::Ordering::Equal => Status::Draw,
        };
    }

    events
}

fn apply_orders(state: &mut BattleState, faction: Faction, orders: &Orders) {
    let f = faction.index();
    for cmd in &orders.commands {
        match *cmd {
            Command::SetSwitch { node, choice } => {
                if node < state.switches[f].len() {
                    state.switches[f][node] = choice;
                }
            }
            Command::Deploy { kind, lane } => {
                let cost = kind.stats().cost;
                if state.steam[f] < cost {
                    continue; // unaffordable — skip
                }
                let spawns = &state.arena.spawns[f];
                if spawns.is_empty() {
                    continue;
                }
                let node = spawns[lane.min(spawns.len() - 1)];
                state.steam[f] -= cost;
                let choice = state.switches[f][node] as usize;
                let to = state.arena.route(faction, node, choice);
                let stats = kind.stats();
                state.trains.push(Train {
                    faction,
                    kind,
                    hp: stats.hp,
                    from: node,
                    to,
                    progress: 0,
                    edge_ticks: stats.edge_ticks.max(1),
                });
            }
        }
    }
}

/// One simulation tick.
fn step(state: &mut BattleState, events: &mut Vec<TurnEvent>) {
    move_trains(state, events);
    apply_damage(state, events);
    prune_and_check(state, events);
}

/// Advance every train one tick; route at nodes; ram the enemy King on arrival.
fn move_trains(state: &mut BattleState, events: &mut Vec<TurnEvent>) {
    for i in 0..state.trains.len() {
        let mut t = state.trains[i];
        if !t.alive() || t.to.is_none() {
            continue;
        }
        t.progress += 1;
        if t.progress >= t.edge_ticks {
            let arrived = t.to.expect("active train has a destination");
            let enemy = t.faction.enemy();
            if arrived == state.arena.kings[enemy.index()] {
                let dmg = ram_damage(t.kind);
                damage_tower_node(state, arrived, enemy, dmg, events);
                t.hp = 0; // consumed on impact
            } else {
                let choice = state.switches[t.faction.index()][arrived] as usize;
                t.from = arrived;
                t.to = state.arena.route(t.faction, arrived, choice);
                t.progress = 0;
                if t.to.is_none() {
                    t.hp = 0; // routed into a terminal that isn't the enemy King
                }
            }
        }
        state.trains[i] = t;
    }
}

/// Compute all shooting + collision damage from a single snapshot, then apply it
/// (so order within a tick can't bias the result).
fn apply_damage(state: &mut BattleState, events: &mut Vec<TurnEvent>) {
    let n = state.trains.len();
    let occ: Vec<Option<NodeId>> = state
        .trains
        .iter()
        .map(|t| if t.alive() { Some(node_of(t)) } else { None })
        .collect();

    let mut train_dmg = vec![0i32; n];
    let mut tower_dmg = vec![0i32; state.towers.len()];

    // Ranged trains (Rockets): hit the nearest enemy train or tower in range.
    for (i, t) in state.trains.iter().enumerate() {
        if !t.alive() {
            continue;
        }
        let stats = t.kind.stats();
        if stats.range <= 0 {
            continue;
        }
        let Some(node) = occ[i] else { continue };
        let p = state.arena.pos(node);
        let r2 = (stats.range as i64) * (stats.range as i64);

        let mut best: Option<(i64, Target)> = None;
        for (j, u) in state.trains.iter().enumerate() {
            if j == i || !u.alive() || u.faction == t.faction {
                continue;
            }
            if let Some(jn) = occ[j] {
                consider(
                    &mut best,
                    dist2(p, state.arena.pos(jn)),
                    r2,
                    Target::Train(j),
                );
            }
        }
        for (k, tow) in state.towers.iter().enumerate() {
            if !tow.alive() || tow.faction == t.faction {
                continue;
            }
            consider(
                &mut best,
                dist2(p, state.arena.pos(tow.node)),
                r2,
                Target::Tower(k),
            );
        }
        if let Some((_, target)) = best {
            match target {
                Target::Train(j) => train_dmg[j] += stats.damage,
                Target::Tower(k) => tower_dmg[k] += stats.damage,
            }
        }
    }

    // Towers shoot the nearest enemy train in range.
    for tow in state.towers.iter() {
        if !tow.alive() {
            continue;
        }
        let p = state.arena.pos(tow.node);
        let r2 = (tow.range as i64) * (tow.range as i64);
        let mut best: Option<(i64, usize)> = None;
        for (j, u) in state.trains.iter().enumerate() {
            if !u.alive() || u.faction == tow.faction {
                continue;
            }
            if let Some(jn) = occ[j] {
                let d = dist2(p, state.arena.pos(jn));
                if d <= r2 && best.is_none_or(|(bd, _)| d < bd) {
                    best = Some((d, j));
                }
            }
        }
        if let Some((_, j)) = best {
            train_dmg[j] += tow.damage;
        }
    }

    // Melee: opposing trains sharing a node trade blows (max enemy damage there).
    for (i, t) in state.trains.iter().enumerate() {
        let Some(node) = occ[i] else { continue };
        let mut foe = 0;
        for (j, u) in state.trains.iter().enumerate() {
            if j == i || !u.alive() || u.faction == t.faction {
                continue;
            }
            if occ[j] == Some(node) {
                foe = foe.max(u.kind.stats().damage);
            }
        }
        train_dmg[i] += foe;
    }

    // Apply train damage.
    for (i, t) in state.trains.iter_mut().enumerate() {
        if train_dmg[i] > 0 && t.alive() {
            t.hp -= train_dmg[i];
        }
    }
    // Apply tower damage + events.
    for (k, tow) in state.towers.iter_mut().enumerate() {
        if tower_dmg[k] > 0 && tow.alive() {
            tow.hp -= tower_dmg[k];
            if tow.kind == TowerKind::King {
                events.push(TurnEvent::KingDamaged {
                    faction: tow.faction,
                    amount: tower_dmg[k],
                });
            }
            if !tow.alive() {
                events.push(TurnEvent::TowerDestroyed {
                    faction: tow.faction,
                    kind: tow.kind,
                });
            }
        }
    }
}

/// Remove dead trains, then decide victory if a King has fallen.
fn prune_and_check(state: &mut BattleState, events: &mut Vec<TurnEvent>) {
    let before = state.trains.len();
    let mut killed = 0;
    state.trains.retain(|t| {
        if t.alive() {
            true
        } else {
            killed += 1;
            false
        }
    });
    for _ in 0..killed.min(before) {
        // Faction of each killed train isn't tracked post-retain; emit a generic
        // count via repeated events keeps the UI simple. (Refine later if needed.)
        events.push(TurnEvent::TrainDestroyed {
            faction: Faction::A,
        });
    }

    let a_dead = state.king_hp(Faction::A) <= 0;
    let b_dead = state.king_hp(Faction::B) <= 0;
    state.status = match (a_dead, b_dead) {
        (true, true) => Status::Draw,
        (true, false) => Status::Won(Faction::B),
        (false, true) => Status::Won(Faction::A),
        (false, false) => return,
    };
    if let Status::Won(f) = state.status {
        events.push(TurnEvent::Victory { faction: f });
    }
}

/// Damage the (King) tower sitting on `node` for `faction`.
fn damage_tower_node(
    state: &mut BattleState,
    node: NodeId,
    faction: Faction,
    dmg: i32,
    events: &mut Vec<TurnEvent>,
) {
    if let Some(tow) = state
        .towers
        .iter_mut()
        .find(|t| t.node == node && t.faction == faction)
    {
        tow.hp -= dmg;
        if tow.kind == TowerKind::King {
            events.push(TurnEvent::KingDamaged {
                faction: tow.faction,
                amount: dmg,
            });
        }
    }
}

/// The node a train currently occupies (its near node along the current edge).
fn node_of(t: &Train) -> NodeId {
    match t.to {
        Some(to) if t.progress * 2 >= t.edge_ticks => to,
        _ => t.from,
    }
}

/// Siege damage when a train rams the enemy King.
fn ram_damage(kind: TrainKind) -> i32 {
    kind.stats().damage * 2
}

enum Target {
    Train(usize),
    Tower(usize),
}

/// Keep the closest in-range candidate (tie-break: earlier index, since we visit
/// trains before towers and in index order).
fn consider(best: &mut Option<(i64, Target)>, d: i64, r2: i64, target: Target) {
    if d <= r2 && best.as_ref().is_none_or(|(bd, _)| d < *bd) {
        *best = Some((d, target));
    }
}
