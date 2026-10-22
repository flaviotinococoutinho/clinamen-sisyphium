//! The mountain: the slope profile grown from the rule-30 ring, the parallax strata, and the
//! ruins counted from the closures (REQ-L2-07, REQ-L8-01).

use crate::natura::rule30;
use crate::signum::pixel::LABEL_NONE;
use crate::signum::raster::{Modo, Pictor};
use crate::spatium::{Compositio, STRATA, scatter32};

/// The ring after `generations` steps of rule 30 (at most 255).
const fn evolved(automatum: u8, generations: u32) -> u8 {
    let mut cells = automatum;
    let mut g = 0u32;
    while g < (generations & 0xFF) {
        cells = rule30(cells);
        g = g.wrapping_add(1);
    }
    cells
}

/// Relief of the slope at `column`, 0..=`amplitude`: the live cells of the ring evolved one
/// generation every four columns, interpolated linearly between generations.
#[must_use]
pub const fn slope_height(automatum: u8, column: u32, amplitude: u16) -> u16 {
    let generation = column.wrapping_shr(2);
    let within = column & 3;
    let here = evolved(automatum, generation).count_ones();
    let next = evolved(automatum, generation.wrapping_add(1)).count_ones();
    let blend = here
        .wrapping_mul(4u32.wrapping_sub(within))
        .wrapping_add(next.wrapping_mul(within));
    (blend.wrapping_mul(amplitude as u32) >> 5) as u16
}

/// Vertical position of the slope line at `x`, between its foot and its summit.
fn slope_y(comp: &Compositio, x: i32) -> i32 {
    let (a, b) = comp.slope;
    let dx = b.x.wrapping_sub(a.x);
    if dx <= 0 {
        return a.y;
    }
    let t = x.wrapping_sub(a.x).clamp(0, dx);
    a.y.wrapping_add(
        b.y.wrapping_sub(a.y)
            .wrapping_mul(t)
            .checked_div(dx)
            .unwrap_or(0),
    )
}

/// Draws `strata` layers of the mountain, farthest first, each displaced by its parallax
/// velocity times the launcher offset and relieved by rule 30; the farthest layer is class 1,
/// the others class 2; luminance rises toward the nearest layer.
#[allow(clippy::too_many_arguments)]
pub fn draw_slope(
    pictor: &mut Pictor,
    comp: &Compositio,
    automatum: u8,
    strata: u8,
    offset: u8,
    lum_far: u8,
    lum_near: u8,
    dither: bool,
) {
    let count = strata.clamp(1, 4);
    let base_y = i32::from(comp.scene_h);
    let amplitude = (comp.figure_h >> 1).max(2);
    let shift = i32::from(offset).wrapping_sub(128);
    let width = i32::from(pictor.width());
    let mut k = count;
    while k > 0 {
        k = k.wrapping_sub(1);
        let velocity = STRATA.get(usize::from(k)).copied().unwrap_or(STRATA[3]);
        let dx = shift.wrapping_mul(velocity.0).wrapping_shr(16);
        let lift = i32::from(k).wrapping_mul(i32::from(amplitude));
        let class = if k == count.wrapping_sub(1) && count > 1 {
            1
        } else {
            2
        };
        let lum = if count <= 1 {
            lum_near
        } else {
            let span = i32::from(lum_near).wrapping_sub(i32::from(lum_far));
            let step = span
                .wrapping_mul(i32::from(count.wrapping_sub(1).wrapping_sub(k)))
                .checked_div(i32::from(count.wrapping_sub(1)))
                .unwrap_or(0);
            (i32::from(lum_far).wrapping_add(step)).clamp(0, 15) as u8
        };
        let seed = automatum.rotate_left(u32::from(k));
        let mut x = 0i32;
        while x < width {
            let relief = i32::from(slope_height(
                seed,
                x.wrapping_add(dx).max(0) as u32,
                amplitude,
            ));
            let top = slope_y(comp, x.wrapping_add(dx))
                .wrapping_sub(lift)
                .wrapping_sub(relief);
            if top < base_y {
                pictor.line(x, top, x, base_y, lum, class, 1, Modo::Super);
                if !dither {
                    pictor.plot(
                        x,
                        top,
                        lum.saturating_add(3).min(15),
                        class,
                        LABEL_NONE,
                        Modo::Super,
                    );
                }
            }
            x = x.wrapping_add(1);
        }
    }
}

/// Number of ruins: interrupted cycles among the thirteen closed ones (bits 1..=13 of the
/// closures; the present cycle, bit 0, is not a ruin yet) (REQ-L8-01).
#[must_use]
pub const fn ruins(clausurae: u16) -> u8 {
    (clausurae & 0x3FFE).count_ones() as u8
}

/// Position of ruin `j` on the slope line, by dispersion of `j + 1`.
#[must_use]
pub fn ruin_position(comp: &Compositio, index: u8) -> (i32, i32) {
    let along = i64::from(scatter32(u32::from(index).wrapping_add(1), 16));
    let (foot, summit) = comp.slope;
    let px = i64::from(foot.x).wrapping_add(
        i64::from(summit.x.wrapping_sub(foot.x))
            .wrapping_mul(along)
            .wrapping_shr(16),
    );
    let py = i64::from(foot.y).wrapping_add(
        i64::from(summit.y.wrapping_sub(foot.y))
            .wrapping_mul(along)
            .wrapping_shr(16),
    );
    (px as i32, py as i32)
}

/// Draws the ruins as broken columns of half the figure's height standing on the slope.
pub fn draw_ruins(pictor: &mut Pictor, comp: &Compositio, clausurae: u16, lum: u8, s: u16) {
    let count = ruins(clausurae);
    let height = i32::from(comp.figure_h >> 1).max(2);
    let cap = i32::from(s.max(1)).wrapping_mul(2);
    let mut j = 0u8;
    while j < count {
        let (x, y) = ruin_position(comp, j);
        let top = y.wrapping_sub(height);
        pictor.line(x, y, x, top, lum, 2, s as u8, Modo::Super);
        pictor.line(
            x.wrapping_sub(cap),
            top,
            x.wrapping_add(cap),
            top,
            lum,
            2,
            s as u8,
            Modo::Super,
        );
        j = j.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::arithmetic_side_effects,
        clippy::indexing_slicing,
        clippy::unwrap_used
    )]
    use super::*;
    use crate::spatium::compositio;

    /// The profile is deterministic, bounded by the amplitude and sensitive to the automaton.
    #[test]
    fn slope_profile_from_rule30() {
        for column in 0..320u32 {
            let h = slope_height(0x10, column, 12);
            assert!(h <= 12, "column {column}: {h}");
            assert_eq!(h, slope_height(0x10, column, 12));
        }
        let a: std::vec::Vec<u16> = (0..64).map(|c| slope_height(0x10, c, 12)).collect();
        let b: std::vec::Vec<u16> = (0..64).map(|c| slope_height(0x3C, c, 12)).collect();
        assert_ne!(a, b);
        assert!(
            a.iter().any(|&h| h > 0) && a.iter().any(|&h| h < 12),
            "profile has relief"
        );
        assert_eq!(slope_height(0x10, 5, 0), 0);
    }

    /// Ruins count the interrupted closed cycles: bits 1..=13, never bit 0 (REQ-L4-08, REQ-L8-01).
    #[test]
    fn ruins_count() {
        assert_eq!(ruins(0), 0);
        assert_eq!(ruins(0b1), 0, "the present cycle is not a ruin yet");
        assert_eq!(ruins(0b10), 1);
        assert_eq!(ruins(0x3FFF), 13);
        assert_eq!(ruins(0xFFFF), 13, "bits above 13 ignored");
    }

    /// Ruin positions lie on the slope, inside the frame, and repeat for the same index.
    #[test]
    fn ruins_positions_deterministic() {
        let comp = compositio(320, 180, 6);
        for j in 0..13u8 {
            let (x, y) = ruin_position(&comp, j);
            assert_eq!((x, y), ruin_position(&comp, j));
            assert!(x >= comp.slope.0.x && x <= comp.slope.1.x, "x {x}");
            assert!(y >= comp.slope.1.y && y <= comp.slope.0.y, "y {y}");
        }
        assert_ne!(ruin_position(&comp, 0), ruin_position(&comp, 1));
    }
}
