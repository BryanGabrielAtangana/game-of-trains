//! Deterministic map generation.
//!
//! A map is a tree of track segments rooted at a single node. Trains enter at
//! the root and flow outward. Internal nodes are either *straight* track (one
//! child) or *switches* (two children, one of which is active at a time —
//! the only thing the player can change). Leaves are *stations* (numbered
//! houses) or, occasionally, *dead-ends* (traps).
//!
//! Geometry is a simple "tidy tree": depth maps to the x axis, and each leaf
//! occupies its own row on the y axis. The renderer reads positions directly;
//! the simulation only cares about the parent/child graph.

use crate::config::GameConfig;
use crate::geometry::Pos;
use crate::rng::Rng;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Vertical spacing multiplier between adjacent leaf rows.
const ROW_SCALE: i32 = 2;

/// A node's role in the map.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum NodeKind {
    /// Internal track. With one child it is straight; with two it is a switch.
    Track,
    /// A numbered destination. Trains want to reach the station whose label matches.
    Station { label: u32 },
    /// A trap. Any train that arrives here is lost.
    DeadEnd,
}

/// A single node in the map tree.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Node {
    /// Grid position, for rendering.
    pub pos: Pos,
    /// Parent node index, or `None` for the root.
    pub parent: Option<usize>,
    /// Child node indices (0 = leaf, 1 = straight, 2 = switch).
    pub children: Vec<usize>,
    pub kind: NodeKind,
}

impl Node {
    /// A switch is an internal node with two children: the player can flip which
    /// child is active.
    pub fn is_switch(&self) -> bool {
        matches!(self.kind, NodeKind::Track) && self.children.len() == 2
    }

    /// True for stations and dead-ends (no children).
    pub fn is_terminal(&self) -> bool {
        self.children.is_empty()
    }
}

/// A generated map: the node arena plus convenient indices.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Map {
    pub nodes: Vec<Node>,
    pub root: usize,
    /// Indices of every station node (dead-ends excluded).
    pub stations: Vec<usize>,
    /// All station labels, in station order — the pool a train picks a destination from.
    pub labels: Vec<u32>,
    pub width: i32,
    pub height: i32,
}

impl Map {
    /// Generate the map for a config. Pure function of `(seed, level)`.
    pub fn generate(config: &GameConfig) -> Map {
        // Salt the seed so the map stream is independent of the train stream.
        let mut rng = Rng::new(config.seed ^ 0x6D61705F73656564); // "map_seed"
        let mut b = Builder {
            nodes: Vec::new(),
            rng: &mut rng,
            extend_probability: config.extend_probability,
            next_row: 0,
        };
        let root = b.grow(None, 0, config.tree_height);
        let mut nodes = b.nodes;

        // --- Turn leaves into stations / dead-ends ---
        let mut leaves: Vec<usize> = (0..nodes.len())
            .filter(|&i| nodes[i].is_terminal())
            .collect();
        // Stable order by position so labels are reproducible.
        leaves.sort_by_key(|&i| (nodes[i].pos.x, nodes[i].pos.y));

        let mut dead_ends = 0u32;
        let mut next_label = 1u32;
        let mut stations = Vec::new();
        let mut labels = Vec::new();
        for &leaf in &leaves {
            let make_dead_end = dead_ends < config.max_dead_ends
                && rng.chance(config.dead_end_probability)
                // never let the very last potential station be a trap
                && stations.len() + 1 < leaves.len();
            if make_dead_end {
                nodes[leaf].kind = NodeKind::DeadEnd;
                dead_ends += 1;
            } else {
                nodes[leaf].kind = NodeKind::Station { label: next_label };
                stations.push(leaf);
                labels.push(next_label);
                next_label += 1;
            }
        }

        // --- Bounds / normalisation (coords are already >= 0) ---
        let width = nodes.iter().map(|n| n.pos.x).max().unwrap_or(0) + 1;
        let height = nodes.iter().map(|n| n.pos.y).max().unwrap_or(0) + 1;

        Map {
            nodes,
            root,
            stations,
            labels,
            width,
            height,
        }
    }

    /// The active next node from `node`, given the current switch states.
    ///
    /// * straight track -> its only child
    /// * switch -> the child selected by `active` (0 or 1)
    /// * terminal -> `None`
    pub fn next(&self, node: usize, active: bool) -> Option<usize> {
        let n = &self.nodes[node];
        match n.children.len() {
            0 => None,
            1 => Some(n.children[0]),
            _ => Some(n.children[usize::from(active)]),
        }
    }
}

/// Internal recursive builder.
struct Builder<'r> {
    nodes: Vec<Node>,
    rng: &'r mut Rng,
    extend_probability: u32,
    next_row: i32,
}

impl Builder<'_> {
    /// Grow a subtree and return its node index. `depth_left` is the number of
    /// *branch* levels remaining (straight extensions don't consume it).
    fn grow(&mut self, parent: Option<usize>, col: i32, depth_left: u32) -> usize {
        let id = self.nodes.len();
        self.nodes.push(Node {
            pos: Pos::new(col, 0), // y is fixed up below
            parent,
            children: Vec::new(),
            kind: NodeKind::Track, // leaves are re-typed after generation
        });

        if depth_left == 0 {
            // Leaf: claim the next row.
            let row = self.next_row;
            self.next_row += 1;
            self.nodes[id].pos.y = row * ROW_SCALE;
            return id;
        }

        // Optionally lay a straight extension before branching (one child),
        // making the map lankier and giving the player more time to switch.
        if self.rng.chance(self.extend_probability) {
            let child = self.grow(Some(id), col + 1, depth_left);
            let y = self.nodes[child].pos.y;
            self.nodes[id].pos.y = y;
            self.nodes[id].children = vec![child];
            return id;
        }

        // Branch into two children (a switch).
        let a = self.grow(Some(id), col + 1, depth_left - 1);
        let bch = self.grow(Some(id), col + 1, depth_left - 1);
        let ya = self.nodes[a].pos.y;
        let yb = self.nodes[bch].pos.y;
        self.nodes[id].pos.y = (ya + yb) / 2;
        self.nodes[id].children = vec![a, bch];
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(level: u32) -> Map {
        Map::generate(&GameConfig::new(0xABCD, level))
    }

    #[test]
    fn deterministic() {
        assert_eq!(map(5), map(5));
        assert_eq!(
            Map::generate(&GameConfig::new(1, 3)),
            Map::generate(&GameConfig::new(1, 3))
        );
    }

    #[test]
    fn different_seeds_usually_differ() {
        let a = Map::generate(&GameConfig::new(1, 4));
        let b = Map::generate(&GameConfig::new(2, 4));
        assert_ne!(a, b);
    }

    #[test]
    fn is_a_valid_tree() {
        let m = map(6);
        assert_eq!(m.nodes[m.root].parent, None);
        // exactly one root
        assert_eq!(m.nodes.iter().filter(|n| n.parent.is_none()).count(), 1);
        // every child points back to its parent; no node is its own ancestor (acyclic)
        for (i, n) in m.nodes.iter().enumerate() {
            for &c in &n.children {
                assert_eq!(m.nodes[c].parent, Some(i));
            }
        }
    }

    #[test]
    fn all_nodes_reachable_from_root() {
        let m = map(6);
        let mut seen = vec![false; m.nodes.len()];
        let mut stack = vec![m.root];
        while let Some(i) = stack.pop() {
            if seen[i] {
                continue;
            }
            seen[i] = true;
            stack.extend(m.nodes[i].children.iter().copied());
        }
        assert!(seen.iter().all(|&s| s), "some nodes unreachable from root");
    }

    #[test]
    fn stations_have_unique_sequential_labels() {
        let m = map(6);
        assert!(!m.stations.is_empty());
        assert_eq!(m.labels.len(), m.stations.len());
        let expected: Vec<u32> = (1..=m.stations.len() as u32).collect();
        assert_eq!(m.labels, expected);
        for &s in &m.stations {
            assert!(matches!(m.nodes[s].kind, NodeKind::Station { .. }));
        }
    }

    #[test]
    fn dead_ends_respect_the_cap() {
        // High level => dead-ends possible; must never exceed the configured cap.
        for level in 1..30 {
            let cfg = GameConfig::new(777, level);
            let m = Map::generate(&cfg);
            let de = m
                .nodes
                .iter()
                .filter(|n| n.kind == NodeKind::DeadEnd)
                .count() as u32;
            assert!(de <= cfg.max_dead_ends, "level {level}: {de} dead-ends");
            // There is always at least one real station to aim for.
            assert!(!m.stations.is_empty(), "level {level} has no stations");
        }
    }

    #[test]
    fn next_follows_switch_state() {
        let m = map(5);
        // find a switch
        let sw = m.nodes.iter().position(|n| n.is_switch()).unwrap();
        assert_eq!(m.next(sw, false), Some(m.nodes[sw].children[0]));
        assert_eq!(m.next(sw, true), Some(m.nodes[sw].children[1]));
        // a station goes nowhere
        let st = m.stations[0];
        assert_eq!(m.next(st, false), None);
    }
}
