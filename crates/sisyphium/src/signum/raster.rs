//! Raster primitives over a `Tabula`: plotting through the curvature projection, in painter's
//! order or under the cyclic dominance of stratum classes; lines, discs, dithered fills and the
//! packed Bayer matrix (REQ-L2-06, REQ-L2-08, REQ-L5-03, REQ-H-02).

use crate::hospes::Tabula;
use crate::signum::pixel::{lum_of, pixel, stratum_of};
use crate::spatium::{WARP_R2_MAX, dominates, isqrt, warp_block, warp_factor, warp_scale};

/// The 4×4 ordered-dither matrix, four bits per threshold, index `((y & 3) << 2) | (x & 3)`
/// (Bayer, 1973; REQ-L5-03).
pub const BAYER_4X4: u64 = 0x5D7F_91B3_6E4C_A280;

/// Threshold of the dither matrix at `(x, y)`, 0..=15.
#[must_use]
pub const fn bayer_threshold(x: u32, y: u32) -> u8 {
    let index = ((y & 3) << 2) | (x & 3);
    ((BAYER_4X4 >> (index << 2)) & 0xF) as u8
}

/// Whether a cell of luminance `lum` (0..=15) is lit at `(x, y)`: exactly `lum` of every 16
/// cells light up.
#[must_use]
pub const fn bayer(x: u32, y: u32, lum: u8) -> bool {
    lum > bayer_threshold(x, y)
}

/// How a plot interacts with the pixel already there.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Modo {
    /// Painter's order: always writes.
    Super,
    /// Cyclic dominance of stratum classes (REQ-L2-08): writes when the new class dominates or
    /// the old pixel is void; XORs the luminance when the classes are equal; otherwise leaves.
    Dominans,
}

/// A painter over a `Tabula`: every coordinate passes through the curvature projection around
/// the frame center before it is written, and every write is bounds-checked (REQ-H-02).
#[derive(Debug)]
pub struct Pictor<'t, 'p> {
    /// The surface.
    tab: &'t mut Tabula<'p>,
    /// Curvature applied to every coordinate.
    kappa: i8,
    /// Frame center.
    cx: i32,
    /// Frame center.
    cy: i32,
    /// Half of the larger dimension: the unit of the normalized coordinates.
    half: i32,
}

impl<'t, 'p> Pictor<'t, 'p> {
    /// A painter with curvature `kappa` over `tab`.
    pub fn new(tab: &'t mut Tabula<'p>, kappa: i8) -> Self {
        let w = i32::from(tab.width);
        let h = i32::from(tab.height);
        let half = if w > h { w } else { h }.wrapping_shr(1).max(1);
        Self {
            tab,
            kappa,
            cx: w.wrapping_shr(1),
            cy: h.wrapping_shr(1),
            half,
        }
    }

    /// A painter without curvature.
    pub fn flat(tab: &'t mut Tabula<'p>) -> Self {
        Self::new(tab, 0)
    }

    /// The farthest logical coordinate that can still project inside the frame: the strongest
    /// contraction of REQ-L2-06 is `warp_factor(WARP_R2_MAX, −128)` = 21 845/65 536 > 1/8, so no
    /// point beyond ±(8·half + 256) lands in the frame.
    fn reach(&self) -> i32 {
        self.half.saturating_mul(8).saturating_add(256)
    }

    /// Clamps a logical coordinate to the reachable range.
    fn clamp(&self, v: i32) -> i32 {
        let r = self.reach();
        v.clamp(r.wrapping_neg(), r)
    }

    /// Projects a logical coordinate through the curvature (REQ-L2-06): the squared radius
    /// normalized to the half of the larger dimension, r2 = (dx² + dy²)·65 536/half² in Q16.16
    /// (saturating), gives the factor of [`warp_factor`], which scales the displacement in pixels
    /// with full precision ([`warp_scale`]). Returns the projected point and the side of the
    /// block that covers it ([`warp_block`]).
    fn project(&self, x: i32, y: i32) -> (i64, i64, i64) {
        if self.kappa == 0 {
            return (i64::from(x), i64::from(y), 1);
        }
        let (cx, cy) = (i64::from(self.cx), i64::from(self.cy));
        let dx = i64::from(x).wrapping_sub(cx);
        let dy = i64::from(y).wrapping_sub(cy);
        let half = i64::from(self.half);
        let r2 = dx
            .saturating_mul(dx)
            .saturating_add(dy.saturating_mul(dy))
            .saturating_mul(65_536)
            .checked_div(half.wrapping_mul(half))
            .unwrap_or(WARP_R2_MAX);
        let factor = warp_factor(r2, self.kappa);
        (
            cx.wrapping_add(warp_scale(dx, factor)),
            cy.wrapping_add(warp_scale(dy, factor)),
            warp_block(r2, self.kappa),
        )
    }

    /// Writes one logical pixel: the block of REQ-L2-06 whose top-left corner is the projected
    /// point (one pixel unless the curvature expands), clipped to the frame.
    pub fn plot(&mut self, x: i32, y: i32, lum: u8, stratum: u8, label: u8, modo: Modo) {
        let (px, py, side) = self.project(x, y);
        let (w, h) = (i64::from(self.tab.width), i64::from(self.tab.height));
        let (right, bottom) = (
            px.saturating_add(side).min(w),
            py.saturating_add(side).min(h),
        );
        for row in py.max(0)..bottom {
            for column in px.max(0)..right {
                self.put(column, row, lum, stratum, label, modo);
            }
        }
    }

    /// Writes one frame pixel at `(x, y)`, inside the frame, under `modo`.
    fn put(&mut self, x: i64, y: i64, lum: u8, stratum: u8, label: u8, modo: Modo) {
        let index = (y as usize)
            .wrapping_mul(usize::from(self.tab.width))
            .wrapping_add(x as usize);
        let Some(slot) = self.tab.pixels.get_mut(index) else {
            return;
        };
        let old = *slot;
        let new = pixel(lum, stratum, label);
        *slot = match modo {
            Modo::Super => new,
            Modo::Dominans => {
                let old_class = stratum_of(old);
                if dominates(stratum & 3, old_class) {
                    new
                } else if old_class == (stratum & 3) {
                    pixel(lum_of(old) ^ (lum & 0xF), old_class, label)
                } else {
                    old
                }
            }
        };
    }

    /// Writes a `thick`×`thick` block whose top-left corner is offset by ⌊(thick − 1)/2⌋.
    #[allow(clippy::too_many_arguments)]
    fn blot(&mut self, x: i32, y: i32, lum: u8, stratum: u8, label: u8, thick: u8, modo: Modo) {
        let t = i32::from(thick.max(1));
        let off = t.wrapping_sub(1).wrapping_shr(1);
        let mut dy = 0;
        while dy < t {
            let mut dx = 0;
            while dx < t {
                self.plot(
                    x.wrapping_sub(off).wrapping_add(dx),
                    y.wrapping_sub(off).wrapping_add(dy),
                    lum,
                    stratum,
                    label,
                    modo,
                );
                dx = dx.wrapping_add(1);
            }
            dy = dy.wrapping_add(1);
        }
    }

    /// A line by Bresenham's algorithm (1965) with a block of `thick` pixels per step.
    #[allow(clippy::too_many_arguments)]
    pub fn line(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        lum: u8,
        stratum: u8,
        thick: u8,
        modo: Modo,
    ) {
        let (mut x, mut y) = (self.clamp(x0), self.clamp(y0));
        let (x1, y1) = (self.clamp(x1), self.clamp(y1));
        let dx = x1.wrapping_sub(x).abs();
        let dy = y1.wrapping_sub(y).abs().wrapping_neg();
        let sx = if x < x1 { 1 } else { -1 };
        let sy = if y < y1 { 1 } else { -1 };
        let mut err = dx.wrapping_add(dy);
        loop {
            self.blot(x, y, lum, stratum, 3, thick, modo);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = err.wrapping_shl(1);
            if e2 >= dy {
                err = err.wrapping_add(dy);
                x = x.wrapping_add(sx);
            }
            if e2 <= dx {
                err = err.wrapping_add(dx);
                y = y.wrapping_add(sy);
            }
        }
    }

    /// A filled disc, solid or dithered by the Bayer matrix at luminance `lum`.
    pub fn disc(&mut self, cx: i32, cy: i32, r: i32, lum: u8, stratum: u8, dither: bool) {
        let r = self.clamp(r.abs());
        let r2 = (r as u32).saturating_mul(r as u32);
        let mut dy = r.wrapping_neg();
        while dy <= r {
            let dy2 = (dy as i64).wrapping_mul(dy as i64) as u32;
            let span = isqrt(r2.saturating_sub(dy2)) as i32;
            let mut dx = span.wrapping_neg();
            while dx <= span {
                let (x, y) = (cx.wrapping_add(dx), cy.wrapping_add(dy));
                if !dither || bayer(x as u32, y as u32, lum) {
                    self.plot(x, y, lum, stratum, 3, Modo::Super);
                }
                dx = dx.wrapping_add(1);
            }
            dy = dy.wrapping_add(1);
        }
    }

    /// A filled rectangle, solid or dithered by the Bayer matrix at luminance `lum`.
    #[allow(clippy::too_many_arguments)]
    pub fn fill(&mut self, x: i32, y: i32, w: i32, h: i32, lum: u8, stratum: u8, dither: bool) {
        let (x0, y0) = (self.clamp(x), self.clamp(y));
        let (x1, y1) = (
            self.clamp(x.saturating_add(w)),
            self.clamp(y.saturating_add(h)),
        );
        let mut py = y0;
        while py < y1 {
            let mut px = x0;
            while px < x1 {
                if !dither || bayer(px as u32, py as u32, lum) {
                    self.plot(px, py, lum, stratum, 3, Modo::Super);
                }
                px = px.wrapping_add(1);
            }
            py = py.wrapping_add(1);
        }
    }

    /// Width of the surface.
    #[must_use]
    pub fn width(&self) -> u16 {
        self.tab.width
    }

    /// Height of the surface.
    #[must_use]
    pub fn height(&self) -> u16 {
        self.tab.height
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::arithmetic_side_effects,
        clippy::indexing_slicing,
        clippy::unwrap_used,
        clippy::naive_bytecount
    )]
    use super::*;
    use crate::hospes::Tabula;
    use crate::natura::mix;
    use crate::signum::pixel::{VOID, lum_of, pixel, stratum_of};
    use std::vec::Vec;

    /// The packed matrix is the classic 4×4 ordered-dither matrix (REQ-L5-03).
    #[test]
    fn bayer_packed_matrix() {
        assert_eq!(BAYER_4X4, 0x5D7F_91B3_6E4C_A280);
        let classic = [
            [0u8, 8, 2, 10],
            [12, 4, 14, 6],
            [3, 11, 1, 9],
            [15, 7, 13, 5],
        ];
        for y in 0..4 {
            for x in 0..4 {
                assert_eq!(bayer_threshold(x, y), classic[y as usize][x as usize]);
            }
        }
        for lum in 0..16u8 {
            let on = (0..16u32).filter(|i| bayer(i % 4, i / 4, lum)).count();
            assert_eq!(
                on,
                usize::from(lum),
                "luminance {lum} lights {on} of 16 cells"
            );
        }
    }

    /// `Super` always writes; `Dominans` follows the cyclic rule and XORs equal classes.
    #[test]
    fn plot_respects_dominance() {
        let mut buf = [VOID; 4];
        let mut tab = Tabula::new(2, 2, &mut buf).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        pictor.plot(0, 0, 9, 2, 3, Modo::Super);
        assert_eq!((lum_of(buf[0]), stratum_of(buf[0])), (9, 2));
        let mut tab = Tabula::new(2, 2, &mut buf).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        pictor.plot(0, 0, 5, 1, 3, Modo::Dominans);
        assert_eq!(
            (lum_of(buf[0]), stratum_of(buf[0])),
            (9, 2),
            "1 does not dominate 2"
        );
        let mut tab = Tabula::new(2, 2, &mut buf).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        pictor.plot(0, 0, 5, 0, 3, Modo::Dominans);
        assert_eq!(
            (lum_of(buf[0]), stratum_of(buf[0])),
            (5, 0),
            "0 dominates 2"
        );
        let mut tab = Tabula::new(2, 2, &mut buf).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        pictor.plot(0, 0, 3, 0, 3, Modo::Dominans);
        assert_eq!(
            (lum_of(buf[0]), stratum_of(buf[0])),
            (5 ^ 3, 0),
            "equal classes XOR"
        );
        let mut tab = Tabula::new(2, 2, &mut buf).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        pictor.plot(1, 1, 7, 1, 3, Modo::Dominans);
        pictor.plot(5, 5, 7, 1, 3, Modo::Super);
        pictor.plot(-1, 0, 7, 1, 3, Modo::Super);
        assert_eq!(
            (lum_of(buf[3]), stratum_of(buf[3])),
            (7, 1),
            "void is overwritten"
        );
        assert_eq!(buf[1], VOID);
    }

    /// A line touches both endpoints and has max(|dx|, |dy|) + 1 pixels at thickness 1.
    #[test]
    fn line_endpoints_and_length() {
        let mut buf = [VOID; 64];
        let mut tab = Tabula::new(8, 8, &mut buf).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        pictor.line(1, 1, 6, 4, 12, 0, 1, Modo::Super);
        assert_eq!(lum_of(buf[1 + 8]), 12);
        assert_eq!(lum_of(buf[6 + 4 * 8]), 12);
        let lit = buf.iter().filter(|&&p| lum_of(p) == 12).count();
        assert_eq!(lit, 6);
        let mut buf2 = [VOID; 64];
        let mut tab = Tabula::new(8, 8, &mut buf2).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        pictor.line(0, 0, 7, 7, 12, 0, 2, Modo::Super);
        let lit2 = buf2.iter().filter(|&&p| lum_of(p) == 12).count();
        assert!(lit2 > 8 && lit2 <= 32, "thick diagonal lit {lit2}");
    }

    /// A dithered disc of luminance 8 lights about half of its interior; a solid one lights all.
    #[test]
    fn disc_and_fill() {
        let mut buf = [VOID; 16 * 16];
        let mut tab = Tabula::new(16, 16, &mut buf).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        pictor.disc(8, 8, 5, 8, 1, true);
        let lit = buf
            .iter()
            .filter(|&&p| stratum_of(p) == 1 && lum_of(p) == 8)
            .count();
        assert!((20..=60).contains(&lit), "dithered disc lit {lit}");
        let mut buf2 = [VOID; 16 * 16];
        let mut tab = Tabula::new(16, 16, &mut buf2).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        pictor.fill(2, 2, 4, 3, 13, 2, false);
        let lit2 = buf2.iter().filter(|&&p| p == pixel(13, 2, 3)).count();
        assert_eq!(lit2, 12);
    }

    /// Drawing never writes outside the slice, for dispersed frame sizes and wild coordinates.
    #[test]
    fn render_never_exceeds_slice() {
        let mut buf = [0u8; 4096 + 7];
        for k in 0..512u64 {
            let h = mix(k);
            let w = 1 + (h & 63) as u16;
            let hh = 1 + ((h >> 8) & 63) as u16;
            let area = usize::from(w) * usize::from(hh);
            buf.fill(VOID);
            buf[area..].fill(0xA5);
            let mut tab = Tabula::new(w, hh, &mut buf[..area]).unwrap();
            let kappa = (h >> 16) as i8;
            let mut pictor = Pictor::new(&mut tab, kappa);
            let c = |s: u32| ((h >> s) & 0xFF) as i32 - 128;
            pictor.line(
                c(24),
                c(32),
                c(40),
                c(48),
                12,
                0,
                1 + (h & 1) as u8,
                Modo::Super,
            );
            pictor.disc(c(24), c(32), (h >> 56) as i32 & 31, 9, 1, true);
            pictor.fill(c(40), c(48), 40, 40, 5, 2, false);
            pictor.plot(i32::MAX, i32::MIN, 1, 0, 0, Modo::Dominans);
            assert!(
                buf[area..].iter().all(|&b| b == 0xA5),
                "overflow at frame {w}×{hh}"
            );
        }
    }

    /// A filled square of side H/2 at the center stays a solid area under curvature: inside the
    /// bounding box of the lit pixels at least 95 % are lit, at three resolutions and under
    /// contraction and expansion (REQ-L2-06).
    #[test]
    fn curvature_density() {
        for (w, h) in [(320u16, 180u16), (640, 360), (1920, 1080)] {
            for kappa in [-64i8, -1, 1, 64] {
                let (wu, hu) = (usize::from(w), usize::from(h));
                let mut buf = std::vec![VOID; wu * hu];
                let mut tab = Tabula::new(w, h, &mut buf).unwrap();
                let mut pictor = Pictor::new(&mut tab, kappa);
                let side = i32::from(h) / 2;
                let (x0, y0) = (i32::from(w) / 2 - side / 2, i32::from(h) / 2 - side / 2);
                pictor.fill(x0, y0, side, side, 15, 0, false);
                let (mut left, mut right, mut top, mut bottom) = (wu, 0, hu, 0);
                let mut lit = 0usize;
                for (i, &p) in buf.iter().enumerate() {
                    if lum_of(p) == 15 {
                        let (x, y) = (i % wu, i / wu);
                        left = left.min(x);
                        right = right.max(x);
                        top = top.min(y);
                        bottom = bottom.max(y);
                        lit += 1;
                    }
                }
                let area = (right - left + 1) * (bottom - top + 1);
                assert!(
                    lit * 100 >= area * 95,
                    "{w}×{h} κ {kappa}: {lit} of {area} lit in the bounding box"
                );
            }
        }
    }

    /// Curvature bends the plotted position: a point at the edge moves inward under κ < 0.
    #[test]
    fn plot_follows_curvature() {
        let mut buf = [VOID; 64 * 64];
        let mut tab = Tabula::new(64, 64, &mut buf).unwrap();
        let mut pictor = Pictor::new(&mut tab, -64);
        pictor.plot(63, 32, 15, 0, 3, Modo::Super);
        assert_eq!(buf[63 + 32 * 64], VOID, "edge point moved");
        let lit: Vec<usize> = buf
            .iter()
            .enumerate()
            .filter(|&(_, &p)| lum_of(p) == 15)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(lit.len(), 1);
        let (x, y) = (lit[0] % 64, lit[0] / 64);
        assert!(x < 63 && x > 40 && y == 32, "moved to ({x}, {y})");
    }
}
