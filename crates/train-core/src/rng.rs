//! A tiny, fully deterministic pseudo-random number generator.
//!
//! We deliberately avoid the `rand` crate here: the whole anti-cheat story
//! depends on the client (WASM) and the server (native) producing *bit-for-bit
//! identical* sequences from the same seed. A small, self-contained generator
//! with no platform-dependent behaviour is the safest way to guarantee that.
//!
//! The algorithm is SplitMix64 — used to seed/expand — feeding xoshiro256**.
//! Both are well-known, fast, and use only wrapping integer arithmetic, so the
//! output is identical on every target.

/// Deterministic RNG. Cloneable so a run can be re-simulated from any point.
#[derive(Clone, Debug)]
pub struct Rng {
    state: [u64; 4],
}

impl Rng {
    /// Create an RNG from a 64-bit seed.
    pub fn new(seed: u64) -> Self {
        // Expand the single seed into 256 bits of state with SplitMix64.
        let mut sm = seed;
        let mut next = || {
            sm = sm.wrapping_add(0x9E3779B97F4A7C15);
            let mut z = sm;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            z ^ (z >> 31)
        };
        Rng {
            state: [next(), next(), next(), next()],
        }
    }

    /// Raw next 64-bit value (xoshiro256**).
    pub fn next_u64(&mut self) -> u64 {
        let s = &mut self.state;
        let result = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = s[1] << 17;

        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);

        result
    }

    /// Uniform integer in `[0, n)`. Returns 0 when `n == 0`.
    ///
    /// Uses Lemire's multiply-shift to avoid modulo bias while staying integer-only.
    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        let n = n as u64;
        let mut x = self.next_u64() as u32 as u64;
        let mut m = x.wrapping_mul(n);
        let mut low = m as u32 as u64;
        if low < n {
            let threshold = n.wrapping_neg() % n;
            while low < threshold {
                x = self.next_u64() as u32 as u64;
                m = x.wrapping_mul(n);
                low = m as u32 as u64;
            }
        }
        (m >> 32) as u32
    }

    /// Inclusive range `[lo, hi]`.
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        let span = (hi - lo) as u32 + 1;
        lo + self.below(span) as i32
    }

    /// Returns true with probability `percent`/100. `chance(0)`=never, `chance(100)`=always.
    pub fn chance(&mut self, percent: u32) -> bool {
        if percent == 0 {
            return false;
        }
        if percent >= 100 {
            return true;
        }
        self.below(100) < percent
    }

    /// Pick a random element index from a slice of length `len`.
    /// Returns `None` for an empty slice.
    pub fn pick(&mut self, len: usize) -> Option<usize> {
        if len == 0 {
            None
        } else {
            Some(self.below(len as u32) as usize)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Rng::new(1);
        let mut b = Rng::new(2);
        // It is astronomically unlikely for the first value to collide.
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn below_is_in_range_and_unbiased_enough() {
        let mut r = Rng::new(7);
        let mut counts = [0u32; 6];
        for _ in 0..60_000 {
            let v = r.below(6);
            assert!(v < 6);
            counts[v as usize] += 1;
        }
        // Each bucket should be near 10_000; allow generous slack.
        for c in counts {
            assert!(c > 9_000 && c < 11_000, "bucket out of balance: {c}");
        }
    }

    #[test]
    fn below_zero_and_one() {
        let mut r = Rng::new(99);
        assert_eq!(r.below(0), 0);
        for _ in 0..100 {
            assert_eq!(r.below(1), 0);
        }
    }

    #[test]
    fn range_inclusive() {
        let mut r = Rng::new(123);
        for _ in 0..10_000 {
            let v = r.range(-3, 3);
            assert!((-3..=3).contains(&v));
        }
        assert_eq!(r.range(5, 5), 5);
        assert_eq!(r.range(10, 2), 10); // degenerate -> lo
    }

    #[test]
    fn chance_bounds() {
        let mut r = Rng::new(5);
        assert!(!r.chance(0));
        assert!(r.chance(100));
        assert!(r.chance(1000));
    }
}
