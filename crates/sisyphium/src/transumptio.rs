//! `transumptio`: the levels of narration (level L7). The oracle with which the figure models the
//! touches of the observer, a Markov chain of order 1 over the four quadrants of the screen
//! (REQ-L7-01), the trap that a dominant row sets (REQ-L7-02), the metalepsis in which the
//! predicted touch inverts agency (REQ-L7-03) and the sampling of the boulder in abyme
//! (REQ-L7-04). The name is Quintilian's Latin for metalepsis (VIII, 6, 37).
//!
//! The oracle is the `oraculum` of [`Memoria`]: four rows of four 2-bit weights. Row `r` holds,
//! at bits 8r..=8r + 7, the evidence on the quadrant of the next touch given that the last touch
//! fell in quadrant `r`; its cell `c` sits at bits 8r + 2c.

use crate::arbitrium::METALEPSIS;
use crate::hospes::Nuntius;
use crate::memoria::Memoria;
use crate::spatium::KNUTH64;
use crate::verbum::{Modus, Verbum};

/// Number of quadrants of the screen, which is also the number of rows of the oracle and of cells
/// in a row: 4 rows × 4 cells × 2 bits fill the 32 bits of `oraculum` (REQ-L7-01).
const QUADRANTS: u8 = 4;

/// Greatest weight of a cell: two bits, 2² − 1 = 3 (REQ-L7-01).
const WEIGHT_MAX: u32 = 3;

/// Row sum above which a touch lowers the other cells: 6, half of the greatest sum, 4·3 = 12
/// (REQ-L7-01).
const DECAY_SUM: u32 = 6;

/// Greatest row sum of a ready trap: 5, the largest sum for which the maximum 3 holds a strict
/// majority of the row (2·3 > 5) (REQ-L7-02).
const TRAP_SUM: u32 = 5;

/// The eight bits of a row: four cells of two bits.
const ROW_MASK: u32 = 0xFF;

/// A row of four cells of weight 1, 0b01 four times, the row to which the metalepsis returns the
/// oracle (REQ-L7-03); as a mask it selects the low bit of each cell.
const ROW_UNIFORM: u32 = 0x55;

/// The high bit of each cell of a row: [`ROW_UNIFORM`] shifted left by one.
const ROW_HIGH: u32 = ROW_UNIFORM << 1;

/// Rotation, in bits, that turns the word into the micro word of the sampling (REQ-L7-04).
const MICRO_ROTATION: u32 = 13;

/// Fractional bits of the zoom, which is counted in sixteenths, 2⁴ (REQ-L7-04).
const ZOOM_BITS: u32 = 4;

/// Mask of a coordinate of the 8×8 grid that a word draws: its three low bits (REQ-L7-04).
const GRID_MASK: u32 = 7;

/// Offset in bits of the row `row` in the oracle: 8·`row`.
const fn row_shift(row: u8) -> u32 {
    (row as u32).wrapping_shl(3)
}

/// The eight bits of the row `row` of the oracle; the caller keeps the row inside the grid.
const fn row_bits(oraculum: u32, row: u8) -> u32 {
    oraculum.wrapping_shr(row_shift(row)) & ROW_MASK
}

/// The oracle with the row `row` replaced by the eight low bits of `bits`; the caller keeps the
/// row inside the grid.
const fn with_row_bits(oraculum: u32, row: u8, bits: u32) -> u32 {
    let shift = row_shift(row);
    (oraculum & !ROW_MASK.wrapping_shl(shift)) | (bits & ROW_MASK).wrapping_shl(shift)
}

/// Sum of the four weights of a row of eight bits: a weight is its low bit plus twice its high
/// bit, so the sum is the count of ones plus the count of high bits.
const fn row_sum(bits: u32) -> u32 {
    bits.count_ones()
        .wrapping_add((bits & ROW_HIGH).count_ones())
}

/// The low bit of each cell of a row that holds the greatest weight, 3: both bits of the cell
/// are set.
const fn row_full(bits: u32) -> u32 {
    bits & (bits >> 1) & ROW_UNIFORM
}

/// The quadrant of a touch (REQ-L7-01; Markov, 1906): ((`y` ≫ 7) ≪ 1) | (`x` ≫ 7). The screen,
/// normalized to 0..=255 on each axis, is cut in two at the middle of each axis: bit 0 is the
/// half of the horizontal axis and bit 1 the half of the vertical axis, and 127 and 128 fall on
/// either side of each cut. The four quadrants are the states of the chain of the oracle.
#[must_use]
pub const fn quadrant(x: u8, y: u8) -> u8 {
    ((y >> 7) << 1) | (x >> 7)
}

/// The weight of the cell `col` of the row `row` of the oracle (REQ-L7-01; Markov, 1906): the two
/// bits at 8·`row` + 2·`col` of `oraculum`, the evidence that a touch in quadrant `col` follows a
/// touch in quadrant `row`. Outside the 4×4 grid there is no cell and the weight is 0.
#[must_use]
pub const fn weight(oraculum: u32, row: u8, col: u8) -> u8 {
    if row < QUADRANTS && col < QUADRANTS {
        (row_bits(oraculum, row).wrapping_shr((col as u32).wrapping_shl(1)) & WEIGHT_MAX) as u8
    } else {
        0
    }
}

/// The update of the oracle by a touch (REQ-L7-01; Markov, 1906; von Foerster, 1974): `row` is
/// the quadrant of the previous touch and `q` the quadrant of this one. The cell `q` of the row
/// receives min(weight + 1, 3); the other cells of the row receive max(weight − 1, 0) when the
/// sum of the row, with the cell `q` already raised, exceeds 6. The other rows do not change. A
/// row or a quadrant outside the grid leaves the oracle unchanged.
#[must_use]
pub const fn oracle_update(oraculum: u32, row: u8, q: u8) -> u32 {
    if row >= QUADRANTS || q >= QUADRANTS {
        return oraculum;
    }
    let cells = row_bits(oraculum, row);
    let lane = 1u32.wrapping_shl((q as u32).wrapping_shl(1));
    let raised = cells.wrapping_add(lane & !row_full(cells));
    let updated = if row_sum(raised) > DECAY_SUM {
        let present = (raised | (raised >> 1)) & ROW_UNIFORM;
        raised.wrapping_sub(present & !lane)
    } else {
        raised
    };
    with_row_bits(oraculum, row, updated)
}

/// The trap of the row `row` (REQ-L7-02; Markov, 1906): the quadrant that the oracle predicts
/// when the row is ready, that is, when its greatest weight is 3 and the sum of its weights is at
/// most 5. The quadrant is the first cell of the greatest weight; a ready row has only one such
/// cell, since two cells of weight 3 already sum 6. A row outside the grid has no trap.
#[must_use]
pub const fn trap(oraculum: u32, row: u8) -> Option<u8> {
    if row >= QUADRANTS {
        return None;
    }
    let cells = row_bits(oraculum, row);
    let full = row_full(cells);
    if full != 0 && row_sum(cells) <= TRAP_SUM {
        Some((full.trailing_zeros() >> 1) as u8)
    } else {
        None
    }
}

/// The touch of the observer (REQ-L7-01, REQ-L7-03; Genette, 1972): the quadrant of the touch is
/// confronted with the prediction of the row `quadrans`, the quadrant of the previous touch.
/// Outside `Modus::Somnium`, when the trap of that row is ready ([`trap`]) and predicts the
/// quadrant of the touch, the touch is the metalepsis, a transgression of the border between the
/// level of the observer and the level of the figure: the mode becomes `Modus::AmorFati`, the
/// event becomes [`METALEPSIS`] with `ramus` 0 and the age 0, the row returns to uniform weights
/// and the step announces [`Nuntius::METALEPSIS`]. Any other touch updates the row
/// ([`oracle_update`]) and announces nothing. In `Modus::Somnium` no touch is prophesied: the
/// oracle only learns, and the law of the mode wakes the dreamer (REQ-L4-10, REQ-L4-11). In
/// every case `quadrans` becomes the quadrant of the touch. The touch is a measurement: the law
/// takes no capability of the host and every touch counts.
#[must_use]
pub const fn touch(v: Verbum, m: Memoria, x: u8, y: u8) -> (Verbum, Memoria, Nuntius) {
    let touched = quadrant(x, y);
    let row = m.quadrans();
    let prophesied = match trap(m.oraculum(), row) {
        Some(predicted) => predicted == touched,
        None => false,
    };
    if prophesied && !matches!(v.modus(), Modus::Somnium) {
        (
            v.with_modus(Modus::AmorFati),
            m.with_eventus(METALEPSIS)
                .with_ramus(0)
                .with_gradus(0)
                .with_oraculum(with_row_bits(m.oraculum(), row, ROW_UNIFORM))
                .with_quadrans(touched),
            Nuntius::METALEPSIS,
        )
    } else {
        (
            v,
            m.with_oraculum(oracle_update(m.oraculum(), row, touched))
                .with_quadrans(touched),
            Nuntius::NIHIL,
        )
    }
}

/// The cell (`u`, `w`) of the 8×8 grid that a word draws, bit 8·`w` + `u`; both coordinates wrap
/// at 8, as the texture of the boulder is read.
const fn grid_bit(word: u64, u: u32, w: u32) -> bool {
    word.wrapping_shr((w & GRID_MASK).wrapping_shl(3).wrapping_add(u & GRID_MASK)) & 1 == 1
}

/// A coordinate of the grid scaled by the zoom `z`/16: (`a`·`z`) ≫ 4, which the reading of the
/// grid wraps at 8.
const fn zoomed(a: u8, z: u8) -> u32 {
    (a as u32).wrapping_mul(z as u32) >> ZOOM_BITS
}

/// The sampling of the boulder in abyme (REQ-L7-04; Hofstadter, 1979; Avatamsaka Sutra, the net
/// of Indra): the boulder is the word, and the word holds a second boulder. The macro bit is the
/// cell (`u`, `w`) of the 8×8 grid that the word draws, bit 8·`w` + `u`; the micro bit is the cell
/// (u′, w′) = ((`u`·`z`) ≫ 4, (`w`·`z`) ≫ 4) ∧ 7 of the grid of the micro word, the word rotated
/// left by 13 bits and xored with [`KNUTH64`]: the part holds a transformed copy of the whole,
/// seen at the zoom `z`/16. Coordinates wrap at 8.
#[must_use]
pub const fn abyme(v: Verbum, u: u8, w: u8, z: u8) -> (bool, bool) {
    let micro = v.bits().rotate_left(MICRO_ROTATION) ^ KNUTH64;
    (
        grid_bit(v.bits(), u as u32, w as u32),
        grid_bit(micro, zoomed(u, z), zoomed(w, z)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mundus::Mundus;
    use crate::natura::{galois, mix};

    /// The four weights of the row `row` of the oracle.
    fn weights_of(oraculum: u32, row: u8) -> [u32; 4] {
        [0u8, 1, 2, 3].map(|c| u32::from(weight(oraculum, row, c)))
    }

    /// The oracle with the row `row` replaced by `bits` over the base word `base`.
    fn with_row(base: u32, row: u8, bits: u32) -> u32 {
        (base & !(0xFF << (8 * row))) | (bits << (8 * row))
    }

    /// REQ-L7-01 on plain weights: the touched cell rises to at most 3 and, when the row then
    /// sums above 6, the others fall by one to at least 0.
    fn reference_update(mut cells: [u32; 4], q: usize) -> [u32; 4] {
        cells[q] = (cells[q] + 1).min(3);
        if cells.iter().sum::<u32>() > 6 {
            for (c, cell) in cells.iter_mut().enumerate() {
                if c != q {
                    *cell = cell.saturating_sub(1);
                }
            }
        }
        cells
    }

    /// One touch raises its cell and, above a row sum of 6, lowers the others (REQ-L7-01).
    #[test]
    fn oracle_update() {
        let uniform = 0x5555_5555u32;
        let o = super::oracle_update(uniform, 2, 1);
        assert_eq!(weight(o, 2, 1), 2);
        assert_eq!(o & !0x00FF_0000, uniform & !0x00FF_0000);
        let o = super::oracle_update(o, 2, 1);
        assert_eq!([0, 1, 2, 3].map(|c| weight(o, 2, c)), [1, 3, 1, 1]);
        let o = super::oracle_update(o, 2, 1);
        assert_eq!(weight(o, 2, 1), 3);
        let o = super::oracle_update(o, 2, 0);
        assert_eq!([0, 1, 2, 3].map(|c| weight(o, 2, c)), [2, 2, 0, 0]);
        assert_eq!(
            [
                quadrant(0, 0),
                quadrant(127, 127),
                quadrant(128, 0),
                quadrant(0, 128),
                quadrant(255, 255)
            ],
            [0, 0, 1, 2, 3]
        );
    }

    /// The packed update equals the update on plain weights for every row of the oracle, every
    /// content of the row and every touched quadrant, and leaves the other rows alone
    /// (REQ-L7-01).
    #[test]
    fn oracle_update_matches_reference() {
        for row in 0..4u8 {
            for bits in 0..=0xFFu32 {
                let before = with_row(0xA5C3_1E87, row, bits);
                for q in 0..4u8 {
                    let after = super::oracle_update(before, row, q);
                    assert_eq!(
                        weights_of(after, row),
                        reference_update(weights_of(before, row), usize::from(q)),
                        "row {row} bits {bits:#04x} q {q}"
                    );
                    for other in (0..4u8).filter(|&r| r != row) {
                        assert_eq!(weights_of(after, other), weights_of(before, other));
                    }
                }
            }
        }
    }

    /// The weight is the pair of bits at 8·row + 2·col, and outside the 4×4 grid there is no
    /// cell: the weight is 0, the oracle does not change and no trap is ready (REQ-L7-01,
    /// REQ-L7-02).
    #[test]
    fn oracle_grid_edges() {
        let o = 0xE4D2_B198u32;
        for row in 0..4u8 {
            for col in 0..4u8 {
                let shift = 8 * u32::from(row) + 2 * u32::from(col);
                assert_eq!(u32::from(weight(o, row, col)), (o >> shift) & 3);
            }
        }
        for outside in 4..=u8::MAX {
            assert_eq!(weight(u32::MAX, outside, 0), 0);
            assert_eq!(weight(u32::MAX, 0, outside), 0);
            for q in [0, 3, outside] {
                assert_eq!(super::oracle_update(o, outside, q), o);
                assert_eq!(super::oracle_update(o, q.min(3), outside), o);
            }
            assert_eq!(trap(0x0303_0303, outside), None);
        }
    }

    /// Ready when the row's maximum is 3 and its sum at most 5 (REQ-L7-02).
    #[test]
    fn trap_ready_condition() {
        let row = |w: [u32; 4]| w[0] | w[1] << 2 | w[2] << 4 | w[3] << 6;
        assert_eq!(trap(row([3, 1, 1, 0]) << 8, 1), Some(0));
        assert_eq!(trap(row([3, 1, 1, 1]) << 8, 1), None);
        assert_eq!(trap(row([2, 2, 1, 0]) << 8, 1), None);
        assert_eq!(trap(row([1, 3, 0, 0]) << 8, 1), Some(1));
        assert_eq!(trap(row([3, 0, 0, 0]), 0), Some(0));
        assert_eq!(trap(0x5555_5555, 0), None);
    }

    /// Over every row and every content of the row, the trap is ready exactly when the greatest
    /// weight is 3 and the sum at most 5, and it names the first cell of the greatest weight
    /// (REQ-L7-02).
    #[test]
    fn trap_matches_reference() {
        let mut ready = 0u32;
        for row in 0..4u8 {
            for bits in 0..=0xFFu32 {
                let oraculum = with_row(0x9C27_5B01, row, bits);
                let weights = weights_of(oraculum, row);
                let greatest = weights.iter().copied().max().unwrap();
                let expected = (greatest == 3 && weights.iter().sum::<u32>() <= 5)
                    .then(|| weights.iter().position(|&c| c == greatest).unwrap() as u8);
                assert_eq!(trap(oraculum, row), expected, "row {row} bits {bits:#04x}");
                ready += u32::from(expected.is_some());
            }
        }
        assert_eq!(ready, 4 * 40);
    }

    /// The predicted touch inverts agency; elsewhere the oracle learns; the dreamer is never
    /// prophesied (REQ-L7-03).
    #[test]
    fn metalepsis_fires() {
        let v = Verbum::new(0);
        let o = (0x5555_5555u32 & !0xFF00) | (0x07 << 8);
        let m = Memoria::new(0)
            .with_oraculum(o)
            .with_quadrans(1)
            .with_eventus(5)
            .with_ramus(3);
        let (v1, m1, n) = touch(v, m, 10, 10);
        assert_eq!(n, Nuntius::METALEPSIS);
        assert_eq!(
            (v1.modus(), m1.eventus(), m1.ramus(), m1.gradus()),
            (Modus::AmorFati, 11, 0, 0)
        );
        assert_eq!((m1.oraculum() >> 8) & 0xFF, 0x55);
        assert_eq!(m1.quadrans(), 0);
        let (v2, m2, n2) = touch(v, m, 200, 10);
        assert_eq!(
            (n2, v2.modus(), m2.quadrans()),
            (Nuntius::NIHIL, Modus::Vigilia, 1)
        );
        assert_eq!(weight(m2.oraculum(), 1, 1), 2);
        let (_, m3, n3) = touch(v.with_modus(Modus::Somnium), m, 10, 10);
        assert_eq!((n3, m3.eventus(), m3.quadrans()), (Nuntius::NIHIL, 5, 0));
    }

    /// Over the borders of the quadrants, the four modes and 2 048 memories from the generator,
    /// the touch writes the quadrant of the touch into `quadrans`, announces only the
    /// metalepsis, and changes only the fields that the law names: the metalepsis in the three
    /// modes that are not a dream, the update of the oracle everywhere else (REQ-L7-01,
    /// REQ-L7-03).
    #[test]
    fn touch_confined_to_its_fields() {
        let edges = [0u8, 1, 126, 127, 128, 129, 254, 255];
        let (mut semen, mut seed) = (1u16, 0x2545_F491_4F6C_DD1Du64);
        let (mut fired, mut learned, mut dreamed) = (0u32, 0u32, 0u32);
        for _ in 0..2048u32 {
            semen = galois(semen);
            seed = mix(seed ^ u64::from(semen));
            let memoria = Memoria::new(seed);
            let row = memoria.quadrans();
            let kept = !(0xFFu32 << (8 * row));
            for modus in [
                Modus::Vigilia,
                Modus::Quies,
                Modus::Somnium,
                Modus::AmorFati,
            ] {
                let verbum = Verbum::new(seed.rotate_left(17)).with_modus(modus);
                for px in edges {
                    for py in edges {
                        let target = quadrant(px, py);
                        let (after_v, after_m, nuntius) = touch(verbum, memoria, px, py);
                        let ready = trap(memoria.oraculum(), row) == Some(target);
                        assert_eq!(after_m.quadrans(), target);
                        assert_eq!(after_m.clausurae(), memoria.clausurae());
                        if ready && modus != Modus::Somnium {
                            fired += 1;
                            assert_eq!(nuntius, Nuntius::METALEPSIS);
                            assert_eq!(after_v, verbum.with_modus(Modus::AmorFati));
                            assert_eq!(
                                (after_m.eventus(), after_m.ramus(), after_m.gradus()),
                                (11, 0, 0)
                            );
                            assert_eq!(weights_of(after_m.oraculum(), row), [1, 1, 1, 1]);
                            assert_eq!(after_m.oraculum() & kept, memoria.oraculum() & kept);
                        } else {
                            learned += 1;
                            dreamed += u32::from(ready);
                            assert_eq!(nuntius, Nuntius::NIHIL);
                            assert_eq!(after_v, verbum);
                            assert_eq!(
                                (after_m.eventus(), after_m.ramus(), after_m.gradus()),
                                (memoria.eventus(), memoria.ramus(), memoria.gradus())
                            );
                            assert_eq!(
                                after_m.oraculum(),
                                super::oracle_update(memoria.oraculum(), row, target)
                            );
                        }
                    }
                }
            }
        }
        assert!(fired > 1000 && learned > 1000 && dreamed > 100);
    }

    /// Macro bit from the word; micro bit from the rotated and mixed word at zoom z/16 (REQ-L7-04).
    #[test]
    fn abyme_sampling() {
        let v = Verbum::new(0x0123_4567_89AB_CDEF);
        let micro = v.bits().rotate_left(13) ^ 0x9E37_79B9_7F4A_7C15;
        for u in 0..8u8 {
            for w in 0..8u8 {
                for z in [0u8, 16, 37, 255] {
                    let (a, b) = abyme(v, u, w, z);
                    assert_eq!(a, (v.bits() >> (8 * u32::from(w) + u32::from(u))) & 1 == 1);
                    let (u2, w2) = (
                        (u32::from(u) * u32::from(z)) >> 4 & 7,
                        (u32::from(w) * u32::from(z)) >> 4 & 7,
                    );
                    assert_eq!(b, (micro >> (8 * w2 + u2)) & 1 == 1);
                }
            }
        }
        assert!(abyme(Verbum::new(1), 0, 0, 16).0);
    }

    /// Over the whole byte domain of the coordinates and for the zooms that matter, the macro
    /// bit wraps at 8 as the texture of the boulder is read, the micro bit follows the zoom,
    /// zoom 0 collapses the micro grid onto its first cell and zoom 16 reads the same cell of
    /// the micro word (REQ-L7-04).
    #[test]
    fn abyme_total_over_the_bytes() {
        for bits in [0x0123_4567_89AB_CDEFu64, 0xFEDC_BA98_7654_3210, 1, u64::MAX] {
            let boulder = Verbum::new(bits);
            let micro = boulder.bits().rotate_left(13) ^ 0x9E37_79B9_7F4A_7C15;
            for col in 0..=u8::MAX {
                for row in 0..=u8::MAX {
                    let macro_shift = 8 * u32::from(row & 7) + u32::from(col & 7);
                    let macro_bit = (boulder.bits() >> macro_shift) & 1 == 1;
                    for zoom in [0u8, 1, 15, 16, 17, 255] {
                        let (outer, inner) = abyme(boulder, col, row, zoom);
                        assert_eq!(outer, macro_bit);
                        let micro_col = ((u32::from(col) * u32::from(zoom)) >> 4) & 7;
                        let micro_row = ((u32::from(row) * u32::from(zoom)) >> 4) & 7;
                        assert_eq!(inner, (micro >> (8 * micro_row + micro_col)) & 1 == 1);
                    }
                    assert_eq!(abyme(boulder, col, row, 0).1, micro & 1 == 1);
                    if col < 8 && row < 8 {
                        let same = (micro >> (8 * u32::from(row) + u32::from(col))) & 1 == 1;
                        assert_eq!(abyme(boulder, col, row, 16).1, same);
                    }
                }
            }
        }
    }

    /// The numbers of the laws are the derivations recorded beside them, and the uniform row is
    /// the row of the first world (REQ-L7-01..04).
    #[test]
    fn law_constants() {
        assert_eq!(u32::from(QUADRANTS) * u32::from(QUADRANTS) * 2, u32::BITS);
        assert_eq!(
            QUADRANTS - 1,
            Memoria::new(0).with_quadrans(u8::MAX).quadrans()
        );
        assert_eq!(WEIGHT_MAX, (1 << 2) - 1);
        assert_eq!(DECAY_SUM * 2, 4 * WEIGHT_MAX);
        assert_eq!(TRAP_SUM, 2 * WEIGHT_MAX - 1);
        assert_eq!(ROW_MASK, (1 << (2 * u32::from(QUADRANTS))) - 1);
        assert_eq!(ROW_UNIFORM, 0b0101_0101);
        assert_eq!(ROW_HIGH, 0b1010_1010);
        assert_eq!(weights_of(ROW_UNIFORM, 0), [1, 1, 1, 1]);
        assert_eq!(
            Mundus::genesis(1).memoria.oraculum(),
            ROW_UNIFORM * 0x0101_0101
        );
        assert_eq!(MICRO_ROTATION, 13);
        assert_eq!(1u32 << ZOOM_BITS, 16);
        assert_eq!(GRID_MASK + 1, 8);
        assert_eq!(KNUTH64, 0x9E37_79B9_7F4A_7C15);
    }
}
