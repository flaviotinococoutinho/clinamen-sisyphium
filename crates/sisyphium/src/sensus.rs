//! `sensus`: perception (level L6). The mutation of the ring of cells under occlusion
//! (REQ-L6-01), the liminal anomaly after the return of the observer (REQ-L6-02), the gate of
//! the rare event (REQ-L6-03) and the inhibition of return that places the sky node of each
//! event (REQ-L6-04).
//!
//! No law of this module reads a count of interactions, and none makes an event more frequent
//! with interaction (REQ-L6-05): the inputs are the word, the regularity of the interaction and
//! the time since a return as the host measures them, the clock and the memory of the world.

use crate::natura::mix;
use crate::verbum::Verbum;

/// Weight above which an occlusion flips cells instead of rotating them: 2⁷, half of the 256
/// values of `onus` (REQ-L6-01).
const OCCLUSION_WEIGHT: u8 = 128;

/// Mask of the cells that an occlusion flips: the alternate cells of the ring of eight, bits 1, 3,
/// 5 and 7, 2 + 8 + 32 + 128 = 170 = 0xAA (REQ-L6-01).
const OCCLUSION_MASK: u8 = 0xAA;

/// Regularity of the interaction from which the liminal anomaly appears: 200 of 255, 78 % of the
/// range of `automatism` (REQ-L6-02).
const LIMINAL_AUTOMATISM: u8 = 200;

/// Milliseconds after a return during which the liminal anomaly lasts: three seconds
/// (REQ-L6-02).
const LIMINAL_WINDOW_MS: u32 = 3000;

/// Length of a window of the rare gate in milliseconds: four seconds, 2² s (REQ-L6-03).
const RARE_WINDOW_MS: i64 = 4000;

/// Trailing zero bits of the mixed key from which a window opens: 13, a chance of 2⁻¹³ = 1/8192
/// per window for a uniform key, which opens about 2.6 of the 21 600 windows of a day
/// (REQ-L6-03).
const RARE_ZEROS: u32 = 13;

/// Bit of the key where `clausurae` starts: 40, so that the window keeps the 40 low bits and the
/// 16 bits of the `clausurae` argument end where `eventus` starts (REQ-L6-03).
const RARE_CLAUSURAE_SHIFT: u32 = 40;

/// Bit of the key where `eventus` starts: 56, so that its 8 bits end at bit 63 (REQ-L6-03).
const RARE_EVENTUS_SHIFT: u32 = 56;

/// Step of the quadrant of the sky node from one event to the next: 3, a unit of the integers
/// mod 4 (3·3 = 9 ≡ 1), so that the quadrant changes at every event and the four quadrants come
/// round before one repeats (REQ-L6-04).
const QUADRANT_STEP: u8 = 3;

/// Mask of a quadrant: the two bits of `quadrans`, the residue mod 4 (REQ-L6-04).
const QUADRANT_MASK: u8 = 3;

/// The mutation of the ring of cells under occlusion (REQ-L6-01; Rensink, O'Regan and Clark,
/// 1997): above `onus` 128 the ring is xored with 0xAA, which flips the alternate cells;
/// otherwise it rotates left by one bit. The change happens while the work is covered, where the
/// observer cannot compare the frames before and after it. No other field changes. The caller
/// applies the law in a tick that carries `occluded`.
#[must_use]
pub const fn occlusion(v: Verbum) -> Verbum {
    let ring = v.automatum();
    v.with_automatum(if v.onus() > OCCLUSION_WEIGHT {
        ring ^ OCCLUSION_MASK
    } else {
        ring.rotate_left(1)
    })
}

/// The liminal anomaly (REQ-L6-02): the observer interacts with mechanical regularity,
/// `automatism` ≥ 200, within three seconds of the return of the work, `returned_ms` < 3000. In
/// this window the shadow of the figure points away from the light: a prediction error (Friston,
/// 2010) offered to automatic interaction (Kahneman, 2011) at the moment of the return.
/// `returned_ms` is `u32::MAX` when the work never returned, which lies outside the window.
#[must_use]
pub const fn liminal(automatism: u8, returned_ms: u32) -> bool {
    automatism >= LIMINAL_AUTOMATISM && returned_ms < LIMINAL_WINDOW_MS
}

/// The gate of the rare event (REQ-L6-03; Schultz, Dayan and Montague, 1997): an event that
/// nothing in the state announces, a prediction error for the observer. Time is cut into windows
/// of four seconds, w = ⌊`epoch_ms` / 4000⌋ by Euclidean division, so that the verdict holds for
/// the whole window and negative epochs have windows too. A window opens when the key
/// w ⊕ (`clausurae` ≪ 40) ⊕ (`eventus` ≪ 56), mixed by [`mix`], ends in at least 13 zero bits:
/// a chance of 2⁻¹³ per window. The key reads the clock and the memory of the world and nothing
/// that the observer does (REQ-L6-05).
#[must_use]
pub const fn rare(epoch_ms: i64, clausurae: u16, eventus: u8) -> bool {
    let window = epoch_ms.div_euclid(RARE_WINDOW_MS) as u64;
    let key = window
        ^ ((clausurae as u64) << RARE_CLAUSURAE_SHIFT)
        ^ ((eventus as u64) << RARE_EVENTUS_SHIFT);
    mix(key).trailing_zeros() >= RARE_ZEROS
}

/// The quadrant of the sky node of the event `eventus` (REQ-L6-04; Posner and Cohen, 1984):
/// (3·`eventus`) mod 4. Three is a unit of the integers mod 4, so consecutive events never share
/// a quadrant and four consecutive events visit the four quadrants before one repeats: attention
/// is never led back to the place it has just left. The product wraps in `u8`, which keeps its
/// residue mod 4.
#[must_use]
pub const fn quadrant_of(eventus: u8) -> u8 {
    eventus.wrapping_mul(QUADRANT_STEP) & QUADRANT_MASK
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memoria::Memoria;
    use std::vec::Vec;

    /// The eight bits of the ring of cells in a word.
    const RING: u64 = 0xFF << 48;

    /// Windows of four seconds that open for `clausurae` and `eventus`, among the `count`
    /// windows that start at `first`.
    fn open_windows(first: i64, count: i64, clausurae: u16, eventus: u8) -> Vec<i64> {
        (first..first + count)
            .filter(|&w| rare(w * 4000, clausurae, eventus))
            .collect()
    }

    /// ⊕ 0xAA under weight, rotation otherwise (REQ-L6-01).
    #[test]
    fn occlusion_mutation() {
        let v = Verbum::new(0).with_automatum(0b1000_0001);
        assert_eq!(occlusion(v.with_onus(129)).automatum(), 0b0010_1011);
        assert_eq!(occlusion(v.with_onus(128)).automatum(), 0b0000_0011);
    }

    /// The mutation rewrites the ring and no other bit, for every `onus` and 256 words from the
    /// mixing function; the threshold lies between `onus` 128 and 129; a flip undoes itself and a
    /// rotation returns after eight applications (REQ-L6-01).
    #[test]
    fn occlusion_confined_to_the_ring() {
        let mut x: u64 = 0x2545_F491_4F6C_DD1D;
        for _ in 0..256u32 {
            x = mix(x);
            for onus in 0..=u8::MAX {
                let v = Verbum::new(x).with_onus(onus);
                let mutated = occlusion(v);
                let expected = if onus > 128 {
                    v.automatum() ^ 0xAA
                } else {
                    v.automatum().rotate_left(1)
                };
                assert_eq!(mutated.automatum(), expected, "onus {onus}");
                assert_eq!(mutated.bits() & !RING, v.bits() & !RING, "onus {onus}");
            }
            let heavy = Verbum::new(x).with_onus(200);
            assert_eq!(occlusion(occlusion(heavy)), heavy);
            let light = Verbum::new(x).with_onus(100);
            let turned = (0..8).fold(light, |word, _| occlusion(word));
            assert_eq!(turned, light);
        }
    }

    /// Regular interaction just after a return (REQ-L6-02).
    #[test]
    fn liminal_anomaly_window() {
        assert!(liminal(200, 2999));
        assert!(!liminal(199, 0));
        assert!(!liminal(255, 3000));
        assert!(!liminal(255, u32::MAX));
    }

    /// The anomaly holds exactly on the corner of the two thresholds: `automatism` from 200 and
    /// `returned_ms` below 3000, over both whole domains (REQ-L6-02).
    #[test]
    fn liminal_corner_of_the_thresholds() {
        for automatism in 0..=u8::MAX {
            assert!(!liminal(automatism, 3000) && !liminal(automatism, u32::MAX));
            assert_eq!(liminal(automatism, 0), automatism >= 200, "{automatism}");
            assert_eq!(liminal(automatism, 2999), automatism >= 200, "{automatism}");
        }
        for returned_ms in (0..6000).chain([u32::MAX - 1, u32::MAX]) {
            assert_eq!(
                liminal(255, returned_ms),
                returned_ms < 3000,
                "{returned_ms}"
            );
            assert!(!liminal(0, returned_ms) && !liminal(199, returned_ms));
        }
    }

    /// About one window in 2¹³ opens; the gate holds for four seconds (REQ-L6-03).
    #[test]
    fn rare_gate_probability() {
        let hits = (0..(1i64 << 20))
            .filter(|w| rare(w * 4000, 0b101, 3))
            .count();
        assert!((90..=170).contains(&hits), "{hits}");
        for w in [0i64, 7, 1 << 30, -5] {
            for d in [0, 1, 3999] {
                assert_eq!(rare(w * 4000 + d, 1, 1), rare(w * 4000, 1, 1));
            }
        }
        let _ = (rare(i64::MIN, u16::MAX, 15), rare(i64::MAX, 0, 0));
    }

    /// Known answers of the gate, computed from the formula of REQ-L6-03 by an independent
    /// reference: the windows among the 2¹⁶ from −2¹⁵ that open for three pairs of `clausurae`
    /// and `eventus`, the extremes of both fields included.
    #[test]
    fn rare_gate_known_windows() {
        let cases: [(u16, u8, &[i64]); 3] = [
            (
                0b101,
                3,
                &[
                    -30939, -12827, 6412, 9102, 15124, 18524, 24000, 29731, 30150,
                ],
            ),
            (
                0x3FFF,
                15,
                &[-31140, -25431, 678, 5428, 13144, 20112, 31334],
            ),
            (
                0,
                0,
                &[
                    -29614, -26972, -16948, -7939, -5227, 876, 2819, 16776, 23226, 24553,
                ],
            ),
        ];
        for (clausurae, eventus, windows) in cases {
            assert_eq!(
                open_windows(-(1 << 15), 1 << 16, clausurae, eventus),
                windows
            );
        }
    }

    /// The windows are the intervals [4000·w, 4000·w + 3999] of the Euclidean quotient, for
    /// negative epochs as for positive ones: every window that opens, in 2¹⁸ windows on each
    /// side of zero, is open at its first and at its last millisecond, and the milliseconds
    /// that border it belong to the neighbouring windows (REQ-L6-03).
    #[test]
    fn rare_window_edges() {
        for side in [-(1i64 << 18), 0] {
            let open = open_windows(side, 1 << 18, 0b101, 3);
            assert!(!open.is_empty(), "no window opens from {side}");
            for w in open {
                assert!(rare(w * 4000, 0b101, 3), "start of {w}");
                assert!(rare(w * 4000 + 3999, 0b101, 3), "end of {w}");
                assert_eq!(rare(w * 4000 - 1, 0b101, 3), rare((w - 1) * 4000, 0b101, 3));
                assert_eq!(
                    rare(w * 4000 + 4000, 0b101, 3),
                    rare((w + 1) * 4000, 0b101, 3)
                );
            }
        }
    }

    /// The key reads `clausurae` and `eventus`: the windows that open change with either of them
    /// (REQ-L6-03).
    #[test]
    fn rare_gate_reads_the_memory() {
        let base = open_windows(0, 1 << 17, 0b101, 3);
        assert!(!base.is_empty());
        assert_ne!(base, open_windows(0, 1 << 17, 0b100, 3));
        assert_ne!(base, open_windows(0, 1 << 17, 0b101, 2));
        assert_ne!(base, open_windows(0, 1 << 17, 0b101 | 1 << 13, 3));
    }

    /// The node of event k sits in quadrant 3k mod 4 (REQ-L6-04).
    #[test]
    fn inhibition_of_return_cycle() {
        assert_eq!(
            [0u8, 1, 2, 3, 4, 5, 6, 7].map(quadrant_of),
            [0, 3, 2, 1, 0, 3, 2, 1]
        );
        for k in 0..16u8 {
            let mut seen = [false; 4];
            for j in 0..4u8 {
                seen[usize::from(quadrant_of((k + j) & 15))] = true;
            }
            assert!(seen.iter().all(|&s| s));
        }
    }

    /// The quadrant is (3·k) mod 4 over the whole domain of `eventus`, including the wrap of the
    /// product in `u8`, and consecutive events never share a quadrant (REQ-L6-04).
    #[test]
    fn quadrant_follows_the_modular_law() {
        for k in 0..=u8::MAX {
            assert_eq!(u32::from(quadrant_of(k)), 3 * u32::from(k) % 4, "{k}");
            assert_ne!(quadrant_of(k), quadrant_of(k.wrapping_add(1)), "{k}");
        }
    }

    /// The numbers of the laws are the derivations recorded beside them (REQ-L6-01..04).
    #[test]
    fn law_constants() {
        assert_eq!(OCCLUSION_WEIGHT, 1 << 7);
        assert_eq!(OCCLUSION_MASK, (0..4).map(|i| 2u8 << (2 * i)).sum::<u8>());
        assert_eq!(OCCLUSION_MASK, 2 * (u8::MAX / 3));
        assert_eq!((LIMINAL_AUTOMATISM, LIMINAL_WINDOW_MS), (200, 3 * 1000));
        assert_eq!(RARE_WINDOW_MS, 4 * 1000);
        assert_eq!(1u32 << RARE_ZEROS, 8192);
        assert_eq!(24 * 3600 * 1000 / RARE_WINDOW_MS, 21_600);
        assert_eq!(10 * 21_600 / 8192, 26);
        assert_eq!(RARE_CLAUSURAE_SHIFT + u16::BITS, RARE_EVENTUS_SHIFT);
        assert_eq!(RARE_EVENTUS_SHIFT + u8::BITS, u64::BITS);
        assert_eq!(QUADRANT_STEP * QUADRANT_STEP % 4, 1);
        assert_eq!(
            QUADRANT_MASK,
            Memoria::new(0).with_quadrans(u8::MAX).quadrans()
        );
    }
}
