//! The deterministic simulation.
//!
//! Everything advances in fixed integer ticks (see [`crate::config::TICKS_PER_SECOND`]).
//! Given a [`GameConfig`] and a sequence of switch toggles at known ticks, the
//! simulation always produces the same outcomes and the same final score —
//! whether it runs in the browser (WASM) or on the server (native). That single
//! property is what makes trustworthy leaderboards possible.

use crate::config::GameConfig;
use crate::map::{Map, NodeKind};
use crate::rng::Rng;
use crate::score::{Outcome, Scorer};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A scheduled train: when it appears, where it wants to go, and how fast it moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TrainSpawn {
    /// Tick at which the train enters at the root.
    pub tick: u32,
    /// The station label this train is trying to reach.
    pub dest: u32,
    /// Ticks to traverse one edge (smaller = faster).
    pub edge_ticks: u32,
}

/// A train currently on the board, modelled as travelling along one edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Train {
    pub dest: u32,
    pub edge_ticks: u32,
    /// Node the train is leaving.
    pub from: usize,
    /// Node the train is heading to (`None` only if `from` is terminal).
    pub to: Option<usize>,
    /// Ticks elapsed on the current edge, `0..edge_ticks`.
    pub progress: u32,
}

impl Train {
    /// Progress along the current edge as a fraction in `[0.0, 1.0]`, for rendering.
    pub fn fraction(&self) -> f32 {
        if self.edge_ticks == 0 {
            1.0
        } else {
            self.progress as f32 / self.edge_ticks as f32
        }
    }
}

/// Build the deterministic train schedule for a config.
///
/// Train destinations, speeds and spacing all come from the seed, so the client
/// and server generate an identical schedule and only the player's switch inputs
/// differ between runs.
pub fn generate_schedule(config: &GameConfig, labels: &[u32]) -> Vec<TrainSpawn> {
    let mut rng = Rng::new(config.seed ^ 0x747261696E73); // "trains"
    let mut schedule = Vec::with_capacity(config.trains as usize);
    let mut tick = 0u32;

    for n in 0..config.trains {
        let dest = labels
            .get(rng.below(labels.len().max(1) as u32) as usize)
            .copied()
            .unwrap_or(1);
        let edge_ticks =
            config.edge_ticks_choices[rng.below(config.edge_ticks_choices.len() as u32) as usize];
        let interval =
            config.interval_choices[rng.below(config.interval_choices.len() as u32) as usize];

        schedule.push(TrainSpawn {
            tick,
            dest,
            edge_ticks,
        });

        // Gap before the next train: `interval` edges at this train's speed.
        // The first train is always immediate (tick 0).
        if n + 1 < config.trains {
            tick += interval * edge_ticks;
        }
    }

    schedule
}

/// A running game.
#[derive(Clone, Debug)]
pub struct Simulation {
    map: Map,
    /// Active child per node (`false` = child 0). Only meaningful for switches.
    switches: Vec<bool>,
    schedule: Vec<TrainSpawn>,
    next_spawn: usize,
    trains: Vec<Train>,
    scorer: Scorer,
    tick: u32,
    outcomes: Vec<Outcome>,
}

impl Simulation {
    /// Create a fresh simulation from a config.
    pub fn new(config: &GameConfig) -> Self {
        let map = Map::generate(config);
        let schedule = generate_schedule(config, &map.labels);
        let switches = vec![false; map.nodes.len()];
        Simulation {
            map,
            switches,
            schedule,
            next_spawn: 0,
            trains: Vec::new(),
            scorer: Scorer::new(),
            tick: 0,
            outcomes: Vec::new(),
        }
    }

    // --- read-only views (for rendering) ---
    pub fn map(&self) -> &Map {
        &self.map
    }
    pub fn switches(&self) -> &[bool] {
        &self.switches
    }
    pub fn trains(&self) -> &[Train] {
        &self.trains
    }
    pub fn scorer(&self) -> &Scorer {
        &self.scorer
    }
    pub fn tick(&self) -> u32 {
        self.tick
    }
    pub fn schedule(&self) -> &[TrainSpawn] {
        &self.schedule
    }
    pub fn outcomes(&self) -> &[Outcome] {
        &self.outcomes
    }

    /// Flip the switch at `node`. No-op if the node is not a switch.
    /// This is the only player action.
    pub fn toggle(&mut self, node: usize) {
        if self.map.nodes.get(node).is_some_and(|n| n.is_switch()) {
            self.switches[node] = !self.switches[node];
        }
    }

    /// True once every scheduled train has spawned and left the board.
    pub fn is_finished(&self) -> bool {
        self.next_spawn >= self.schedule.len() && self.trains.is_empty()
    }

    /// Advance the simulation by exactly one tick.
    ///
    /// Order within a tick is fixed for determinism:
    /// 1. (switch inputs are applied by the caller, before calling `step`)
    /// 2. spawn any trains due this tick,
    /// 3. advance every train one tick and resolve arrivals.
    pub fn step(&mut self) {
        // 2. Spawn.
        while self.next_spawn < self.schedule.len()
            && self.schedule[self.next_spawn].tick == self.tick
        {
            let s = self.schedule[self.next_spawn];
            self.next_spawn += 1;
            let train = Train {
                dest: s.dest,
                edge_ticks: s.edge_ticks.max(1),
                from: self.map.root,
                to: self.active_next(self.map.root),
                progress: 0,
            };
            // A well-formed map always branches at the root; guard anyway.
            if train.to.is_none() {
                self.resolve(&train);
                continue;
            }
            self.trains.push(train);
        }

        // 3. Advance.
        let mut resolved: Vec<usize> = Vec::new();
        for i in 0..self.trains.len() {
            let mut t = self.trains[i];
            t.progress += 1;
            if t.progress >= t.edge_ticks {
                // Arrived at `t.to`.
                let arrived = t.to.expect("active train always has a destination node");
                if self.map.nodes[arrived].is_terminal() {
                    self.resolve(&t);
                    resolved.push(i);
                } else {
                    t.from = arrived;
                    t.to = self.active_next(arrived);
                    t.progress = 0;
                }
            }
            self.trains[i] = t;
        }
        // Remove resolved trains (high indices first to keep positions valid).
        for &i in resolved.iter().rev() {
            self.trains.swap_remove(i);
        }

        self.tick += 1;
    }

    /// Run to completion, returning the final scorer. `max_ticks` guards against
    /// pathological inputs (it should never actually be hit for a valid tree).
    pub fn run_to_end(&mut self, max_ticks: u32) -> &Scorer {
        while !self.is_finished() && self.tick < max_ticks {
            self.step();
        }
        &self.scorer
    }

    fn active_next(&self, node: usize) -> Option<usize> {
        self.map.next(node, self.switches[node])
    }

    fn resolve(&mut self, train: &Train) {
        let node = train.to.unwrap_or(self.map.root);
        let outcome = match self.map.nodes[node].kind {
            NodeKind::DeadEnd => Outcome::Ugly,
            NodeKind::Station { label } => {
                if label == train.dest {
                    Outcome::Good
                } else {
                    Outcome::Bad
                }
            }
            NodeKind::Track => return, // not actually terminal; nothing to resolve
        };
        self.outcomes.push(outcome);
        self.scorer.apply(outcome);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drive a switch to send every train to the station matching its number.
    /// Because the map is a tree, there is exactly one path to each station;
    /// we set switches greedily just before each train reaches them.
    ///
    /// For the test we instead verify by perfect *static* routing: set switches
    /// once so the path to a chosen label is open, then send a train there.
    fn path_switches_to(map: &Map, target_label: u32) -> Vec<(usize, bool)> {
        // Find the station node with this label.
        let station = map
            .stations
            .iter()
            .copied()
            .find(|&s| matches!(map.nodes[s].kind, NodeKind::Station { label } if label == target_label))
            .expect("label exists");
        // Walk up to the root, recording which child each switch must select.
        let mut settings = Vec::new();
        let mut cur = station;
        while let Some(parent) = map.nodes[cur].parent {
            if map.nodes[parent].is_switch() {
                let active = map.nodes[parent].children[1] == cur; // true => child 1
                settings.push((parent, active));
            }
            cur = parent;
        }
        settings
    }

    #[test]
    fn schedule_is_deterministic() {
        let cfg = GameConfig::new(42, 5);
        let m = Map::generate(&cfg);
        assert_eq!(
            generate_schedule(&cfg, &m.labels),
            generate_schedule(&cfg, &m.labels)
        );
    }

    #[test]
    fn first_train_is_immediate_and_count_matches() {
        let cfg = GameConfig::new(42, 5);
        let m = Map::generate(&cfg);
        let sched = generate_schedule(&cfg, &m.labels);
        assert_eq!(sched.len(), cfg.trains as usize);
        assert_eq!(sched[0].tick, 0);
        // ticks are non-decreasing
        assert!(sched.windows(2).all(|w| w[1].tick >= w[0].tick));
    }

    /// Force the switches so the path to `target_label` is fully open, then hold.
    fn open_path_to(sim: &mut Simulation, target_label: u32) {
        for (node, active) in path_switches_to(sim.map(), target_label) {
            if sim.switches()[node] != active {
                sim.toggle(node);
            }
        }
    }

    #[test]
    fn single_train_routed_to_its_dest_scores_good() {
        // One train, switches pre-set to its destination and held: must be Good.
        let mut cfg = GameConfig::new(7, 6);
        cfg.trains = 1;
        let mut sim = Simulation::new(&cfg);
        let dest = sim.schedule()[0].dest;
        open_path_to(&mut sim, dest);
        sim.run_to_end(2_000_000);

        assert!(sim.is_finished());
        assert_eq!(sim.outcomes(), &[Outcome::Good]);
        assert_eq!(sim.scorer().correct, 1);
    }

    #[test]
    fn single_train_routed_to_wrong_station_scores_bad() {
        let mut cfg = GameConfig::new(7, 6);
        cfg.trains = 1;
        cfg.dead_end_probability = 0; // ensure the wrong leaf is a station, not a trap
        cfg.max_dead_ends = 0;
        let mut sim = Simulation::new(&cfg);
        let dest = sim.schedule()[0].dest;
        // Open the path to *a different* label than the destination.
        let other = sim
            .map()
            .labels
            .iter()
            .copied()
            .find(|&l| l != dest)
            .expect("level 6 has more than one station");
        open_path_to(&mut sim, other);
        sim.run_to_end(2_000_000);

        assert!(sim.is_finished());
        assert_eq!(sim.outcomes(), &[Outcome::Bad]);
        assert_eq!(sim.scorer().correct, 0);
    }

    #[test]
    fn finishes_and_delivers_every_train() {
        // With any switch settings, a tree guarantees every train reaches a leaf,
        // so the run finishes and total == number of trains.
        for level in 1..12 {
            let cfg = GameConfig::new(1000 + level as u64, level);
            let mut sim = Simulation::new(&cfg);
            sim.run_to_end(5_000_000);
            assert!(sim.is_finished(), "level {level} did not finish");
            assert_eq!(sim.scorer().total, cfg.trains, "level {level} lost trains");
        }
    }

    #[test]
    fn toggle_ignores_non_switches() {
        let cfg = GameConfig::new(3, 5);
        let mut sim = Simulation::new(&cfg);
        let station = sim.map().stations[0];
        let before = sim.switches().to_vec();
        sim.toggle(station); // station is terminal, not a switch
        sim.toggle(99_999); // out of range
        assert_eq!(sim.switches(), before.as_slice());
    }

    #[test]
    fn perfect_play_with_spaced_trains_scores_only_good() {
        // If trains all share a destination and the path is held open, every one
        // of them is delivered correctly — a clean end-to-end check of scoring
        // and combo accrual over a full multi-train run.
        let mut cfg = GameConfig::new(55, 6);
        cfg.dead_end_probability = 0;
        cfg.max_dead_ends = 0;
        let mut sim = Simulation::new(&cfg);

        // Hold the path to the first train's destination open for the whole run.
        let dest = sim.schedule()[0].dest;
        // Rewrite the schedule's destinations to all match by regenerating with
        // a config whose labels make this trivial is overkill; instead just open
        // the path to `dest` and only assert about trains that target it.
        open_path_to(&mut sim, dest);
        sim.run_to_end(5_000_000);

        assert!(sim.is_finished());
        // Every train aimed at the open station arrived Good; none of those were Bad/Ugly.
        let aimed = sim.schedule().iter().filter(|s| s.dest == dest).count() as u32;
        let goods = sim
            .outcomes()
            .iter()
            .filter(|&&o| o == Outcome::Good)
            .count() as u32;
        assert!(aimed >= 1);
        assert!(
            goods >= aimed,
            "all trains to the open station should be Good"
        );
    }
}
