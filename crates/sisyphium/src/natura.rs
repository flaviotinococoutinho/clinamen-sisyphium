//! `natura`: nature (level L1). The Galois generator that paces every uncertain time and place,
//! the rule-30 ring that shapes slope and wind, stateless 1/f noise, the logistic and Hénon maps,
//! and the mixing function behind every dispersion of a key.

use crate::spatium::{Fx, KNUTH64};
use crate::verbum::SEMEN_DEFAULT;

/// Feedback taps of the 16-bit Galois generator: x¹⁶ + x¹⁴ + x¹³ + x¹¹ + 1 (REQ-L1-01).
pub const GALOIS_POLY: u16 = 0xB400;

/// One step of the Galois generator; the period is 65 535 for every non-zero state, and the
/// zero state is read as [`SEMEN_DEFAULT`] so that the sequence never freezes.
#[must_use]
pub const fn galois(semen: u16) -> u16 {
    let semen = if semen == 0 { SEMEN_DEFAULT } else { semen };
    let lsb = semen & 1;
    (semen >> 1) ^ (lsb.wrapping_neg() & GALOIS_POLY)
}

/// One generation of rule 30 on a ring of eight cells: cell′ = left ⊕ (cell ∨ right), with
/// bit 7 leftmost so that the left neighbour of bit i is bit i + 1 (Wolfram, 1983; REQ-L1-02).
#[must_use]
pub const fn rule30(cells: u8) -> u8 {
    let left = cells.rotate_right(1);
    let right = cells.rotate_left(1);
    left ^ (cells | right)
}

/// Wind blows against the ascent when six or more cells are alive (REQ-L1-08).
#[must_use]
pub const fn wind(cells: u8) -> bool {
    cells.count_ones() >= 6
}

/// Mixes a 64-bit key: two rounds of xor-shift and multiplication by [`KNUTH64`], offset so
/// that the zero key does not map to zero.
#[must_use]
pub const fn mix(x: u64) -> u64 {
    let x = x ^ KNUTH64;
    let x = (x ^ (x >> 32)).wrapping_mul(KNUTH64);
    let x = (x ^ (x >> 29)).wrapping_mul(KNUTH64);
    x ^ (x >> 32)
}

/// Value of octave `k` at time `t`: constant over each interval of 2ᵏ units, as in the
/// octave-holding construction of 1/f noise (Voss and Clarke, 1978; Gardner, 1978).
const fn octave(k: u32, t: u64) -> u8 {
    (mix((t >> (k & 63)) ^ ((k as u64) << 58)) & 0xFF) as u8
}

/// Stateless 1/f noise at time `t` (1/32 s units): the mean of eight octaves, where octave k
/// holds its value for 2ᵏ units (REQ-L1-03).
#[must_use]
pub const fn pink(t: u64) -> u8 {
    let mut sum = 0u32;
    let mut k = 0u32;
    while k < 8 {
        sum = sum.wrapping_add(octave(k, t) as u32);
        k = k.wrapping_add(1);
    }
    (sum >> 3) as u8
}

/// The accumulation point of the period-doubling cascade, r∞ ≈ 3.569 945 (Feigenbaum, 1978).
pub const R_INF: Fx = Fx(0x391E8);

/// The logistic map x′ = r·x·(1 − x) (May, 1976; REQ-L1-04).
#[must_use]
pub const fn logistic(x: Fx, r: Fx) -> Fx {
    r.mul(x).mul(Fx::ONE.sub(x))
}

/// Hénon's a = 1.4 in Q16.16.
const HENON_A: Fx = Fx(0x16666);
/// Hénon's b = 0.3 in Q16.16.
const HENON_B: Fx = Fx(0x4CCD);

/// One iteration of the Hénon map x′ = 1 − a·x² + y, y′ = b·x (Hénon, 1976; REQ-L1-05).
#[must_use]
pub const fn henon(x: Fx, y: Fx) -> (Fx, Fx) {
    (Fx::ONE.sub(HENON_A.mul(x.mul(x))).add(y), HENON_B.mul(x))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
    use super::*;
    use crate::spatium::Fx;

    /// The 16-bit Galois generator with taps `0xB400` has the full period 65 535 (REQ-L1-01).
    #[test]
    fn galois_full_period() {
        assert_eq!(GALOIS_POLY, 0xB400);
        for start in [1u16, 0xACE1, 0x8000, 0xFFFF] {
            let mut s = galois(start);
            let mut n = 1u32;
            while s != start {
                s = galois(s);
                n += 1;
            }
            assert_eq!(n, 65_535, "start {start:#x}");
        }
        assert_eq!(galois(0), galois(crate::verbum::SEMEN_DEFAULT));
    }

    /// Rule 30 on a ring of eight cells matches a bit-by-bit reference (REQ-L1-02).
    #[test]
    fn rule30_known_sequence() {
        fn reference(cells: u8) -> u8 {
            let mut next = 0u8;
            for i in 0..8u32 {
                let left = (cells >> ((i + 1) % 8)) & 1;
                let center = (cells >> i) & 1;
                let right = (cells >> ((i + 7) % 8)) & 1;
                next |= (left ^ (center | right)) << i;
            }
            next
        }
        let mut cells = 0x10u8;
        for _ in 0..64 {
            let next = rule30(cells);
            assert_eq!(next, reference(cells), "from {cells:#010b}");
            cells = next;
        }
        assert_eq!(rule30(0x10), 0x38);
        assert_eq!(rule30(0), 0);
    }

    /// Wind blows when six or more cells are alive (REQ-L1-08).
    #[test]
    fn wind_threshold() {
        assert!(!wind(0b0001_1111));
        assert!(wind(0b0011_1111));
        assert!(wind(0xFF));
    }

    /// Octave k changes value exactly when `t >> k` changes (REQ-L1-03).
    #[test]
    fn pink_octave_rates() {
        for k in 0..8u32 {
            for t in 0..4096u64 {
                let same = octave(k, t) == octave(k, t + 1);
                let boundary = (t >> k) != ((t + 1) >> k);
                if !boundary {
                    assert!(same, "octave {k} changed inside its interval at t {t}");
                }
            }
            let mut changes = 0u32;
            for t in 0..4096u64 {
                changes += u32::from(octave(k, t) != octave(k, t + 1));
            }
            assert!(
                changes * 3 >= 4096 >> k,
                "octave {k} changes too rarely: {changes}"
            );
        }
    }

    /// The mean of 4 096 samples lies in 96..=160 and every sample fits a byte (REQ-L1-03).
    #[test]
    fn pink_mean_bounds() {
        let sum: u64 = (0..4096u64).map(|t| u64::from(pink(t))).sum();
        let mean = sum / 4096;
        assert!((96..=160).contains(&mean), "mean {mean}");
    }

    /// x' = r·x·(1 − x) in Q16.16 and the Feigenbaum threshold (REQ-L1-04).
    #[test]
    fn logistic_fixed_point() {
        assert_eq!(R_INF, Fx(0x391E8));
        assert_eq!(logistic(Fx::from_ratio(1, 2), Fx::from_int(4)), Fx::ONE);
        assert_eq!(logistic(Fx::ZERO, Fx::from_int(4)), Fx::ZERO);
        assert_eq!(logistic(Fx::ONE, Fx::from_int(4)), Fx::ZERO);
        let x = logistic(Fx::from_ratio(1, 4), Fx::from_int(2));
        assert_eq!(x, Fx::from_ratio(3, 8));
    }

    /// The Hénon attractor stays inside |x| ≤ 1.5, |y| ≤ 0.5 for 4 096 iterations (REQ-L1-05).
    #[test]
    fn henon_bounded() {
        let (mut x, mut y) = (Fx::ZERO, Fx::ZERO);
        let (mut min_x, mut max_x) = (Fx::ZERO, Fx::ZERO);
        for _ in 0..4096 {
            let (nx, ny) = henon(x, y);
            x = nx;
            y = ny;
            assert!(x.0.abs() <= 3 * 32768, "x {}", x.0);
            assert!(y.0.abs() <= 32768, "y {}", y.0);
            min_x = min_x.min(x);
            max_x = max_x.max(x);
        }
        assert!(
            max_x.0 > 60_000 && min_x.0 < -60_000,
            "orbit did not spread: {min_x:?} {max_x:?}"
        );
    }

    /// Flipping one input bit of `mix` flips on average at least 24 output bits (avalanche).
    #[test]
    fn mix_avalanche() {
        let mut flipped = 0u64;
        let mut trials = 0u64;
        for k in 0..512u64 {
            let x = mix(k ^ 0x5DEE_CE66_D1CE_4E5B);
            for bit in 0..64u32 {
                flipped += u64::from((mix(x) ^ mix(x ^ (1u64 << bit))).count_ones());
                trials += 1;
            }
        }
        let average = flipped / trials;
        assert!(average >= 24, "average flipped bits {average}");
        assert_ne!(mix(0), 0);
    }
}
