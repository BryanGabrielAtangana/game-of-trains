//! Scoring and combo logic.
//!
//! Kept separate from the simulation so it can be reasoned about (and tested) on
//! its own. The rules echo the original game: correct deliveries build a combo
//! that pays an escalating bonus; mistakes reset it.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// What happened when a train reached the end of its track.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Outcome {
    /// Delivered to the station matching its number.
    Good,
    /// Delivered to the wrong station.
    Bad,
    /// Ran into a dead-end.
    Ugly,
}

const GOOD_POINTS: i32 = 10;
const BAD_POINTS: i32 = -10;
const UGLY_POINTS: i32 = -20;
const COMBO_STARTS_AT: u32 = 3;
const COMBO_STEP: i32 = 3;
const MAX_COMBO_BONUS: i32 = 15;

/// Running score state. Deterministic: feed it the same outcomes in the same
/// order and you get the same totals — which is exactly what server-side
/// verification relies on.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Scorer {
    pub score: i32,
    /// Current streak of consecutive correct deliveries.
    pub combo: u32,
    /// Longest streak reached this run.
    pub best_combo: u32,
    pub correct: u32,
    pub total: u32,
}

impl Scorer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply one delivery outcome and return the points it scored (incl. bonus).
    pub fn apply(&mut self, outcome: Outcome) -> i32 {
        self.total += 1;
        let points = match outcome {
            Outcome::Good => {
                self.combo += 1;
                self.best_combo = self.best_combo.max(self.combo);
                self.correct += 1;
                GOOD_POINTS + self.combo_bonus()
            }
            Outcome::Bad => {
                self.combo = 0;
                BAD_POINTS
            }
            Outcome::Ugly => {
                self.combo = 0;
                UGLY_POINTS
            }
        };
        self.score += points;
        points
    }

    /// Accuracy as a whole-number percentage (0 when nothing delivered yet).
    pub fn accuracy(&self) -> u32 {
        if self.total == 0 {
            0
        } else {
            self.correct * 100 / self.total
        }
    }

    fn combo_bonus(&self) -> i32 {
        if self.combo >= COMBO_STARTS_AT {
            (((self.combo - COMBO_STARTS_AT + 1) as i32) * COMBO_STEP).min(MAX_COMBO_BONUS)
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_points() {
        let mut s = Scorer::new();
        assert_eq!(s.apply(Outcome::Good), 10);
        assert_eq!(s.apply(Outcome::Bad), -10);
        assert_eq!(s.apply(Outcome::Ugly), -20);
        assert_eq!(s.score, -20);
        assert_eq!(s.correct, 1);
        assert_eq!(s.total, 3);
    }

    #[test]
    fn combo_kicks_in_on_third_correct() {
        let mut s = Scorer::new();
        assert_eq!(s.apply(Outcome::Good), 10); // combo 1
        assert_eq!(s.apply(Outcome::Good), 10); // combo 2
        assert_eq!(s.apply(Outcome::Good), 13); // combo 3 -> +3
        assert_eq!(s.apply(Outcome::Good), 16); // combo 4 -> +6
        assert_eq!(s.best_combo, 4);
    }

    #[test]
    fn combo_bonus_is_capped() {
        let mut s = Scorer::new();
        for _ in 0..20 {
            s.apply(Outcome::Good);
        }
        // bonus maxes at 15 => 25 points per delivery at the cap.
        assert_eq!(s.apply(Outcome::Good), 25);
    }

    #[test]
    fn mistakes_reset_combo() {
        let mut s = Scorer::new();
        s.apply(Outcome::Good);
        s.apply(Outcome::Good);
        s.apply(Outcome::Good);
        assert_eq!(s.combo, 3);
        s.apply(Outcome::Bad);
        assert_eq!(s.combo, 0);
        assert_eq!(s.apply(Outcome::Good), 10); // back to base
        assert_eq!(s.best_combo, 3);
    }

    #[test]
    fn accuracy_math() {
        let mut s = Scorer::new();
        assert_eq!(s.accuracy(), 0);
        s.apply(Outcome::Good);
        s.apply(Outcome::Good);
        s.apply(Outcome::Bad);
        s.apply(Outcome::Bad);
        assert_eq!(s.accuracy(), 50);
    }
}
