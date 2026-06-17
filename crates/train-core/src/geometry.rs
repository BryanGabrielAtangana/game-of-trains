//! Grid directions and coordinates.
//!
//! The map is laid out on an integer grid. `(0, 0)` is the top-left; `x` grows
//! east (right), `y` grows south (down) — matching screen coordinates so the
//! renderer can use positions directly.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The four cardinal directions a track can travel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    /// All four directions, in a stable order. Used by generation so that the
    /// same seed always considers directions in the same sequence.
    pub const ALL: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    /// The opposite direction.
    pub fn flip(self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }

    /// Unit step on the grid for this direction.
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        }
    }
}

/// An integer grid coordinate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub const fn new(x: i32, y: i32) -> Self {
        Pos { x, y }
    }

    /// The neighbouring position one step in `dir`.
    pub fn step(self, dir: Direction) -> Pos {
        let (dx, dy) = dir.delta();
        Pos::new(self.x + dx, self.y + dy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_is_involution() {
        for d in Direction::ALL {
            assert_eq!(d.flip().flip(), d);
            assert_ne!(d.flip(), d);
        }
    }

    #[test]
    fn step_then_step_back_returns() {
        let p = Pos::new(3, 4);
        for d in Direction::ALL {
            assert_eq!(p.step(d).step(d.flip()), p);
        }
    }

    #[test]
    fn deltas_are_unit() {
        for d in Direction::ALL {
            let (dx, dy) = d.delta();
            assert_eq!(dx.abs() + dy.abs(), 1);
        }
    }
}
