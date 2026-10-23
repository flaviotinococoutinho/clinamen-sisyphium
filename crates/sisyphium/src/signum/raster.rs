//! Raster primitives over a `Tabula`: plotting through the curvature projection, in painter's
//! order or under the cyclic dominance of stratum classes; lines, discs, dithered fills and the
//! packed Bayer matrix (REQ-L2-06, REQ-L2-08, REQ-L5-03, REQ-H-02).

use crate::hospes::Tabula;
use crate::signum::pixel::{lum_of, pixel, stratum_of};
use crate::spatium::{WARP_R2_MAX, dominates, isqrt, warp_factor, warp_scale};

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
    /// The inclusive logical rectangle `[left, top, right, bottom]` outside which no point
    /// reaches the frame ([`fines`]).
    fines: [i64; 4],
    /// Exclusive projected row limit for the ground (REQ-L2-06).
    finis_y: u16,
    /// Highest luminance of a ground mark (REQ-L5-11).
    lumen_maximum: u8,
}

/// Distance from the center beyond which a logical displacement cannot land within `extent`
/// pixels of the center under a contraction whose factor never falls below `floor` (Q16.16):
/// round(d·f/65 536) ≤ `extent` requires d·`floor`/65 536 < `extent` + 1/2, that is
/// d < (2·`extent` + 1)·32 768/`floor`, so ⌊(2·`extent` + 1)·32 768/`floor`⌋ bounds d.
const fn contracted_reach(extent: i64, floor: i64) -> i64 {
    extent
        .wrapping_mul(2)
        .wrapping_add(1)
        .wrapping_mul(32_768)
        .div_euclid(if floor > 0 { floor } else { 1 })
}

/// The inclusive logical rectangle `[left, top, right, bottom]` outside which no point reaches a
/// frame of `width`×`height` centered at `(cx, cy)` under `kappa` (REQ-L2-06). Flat: the frame
/// itself. Contraction: the factor never falls below f = `warp_factor(WARP_R2_MAX, kappa)`, so
/// a displacement d ≥ 0 projects to at least round(d·f/65 536), and the bound on each side is
/// [`contracted_reach`] of E, the frame's extent from the center on that side (W − 1 − cx to
/// the right, cx to the left; likewise in height). Expansion: the factor is at least 1, so a
/// projected point is never closer to the center than its logical point. The adjacent endpoint
/// can cross the frame only from the preceding logical coordinate; a seven-pixel
/// margin is conservative for the neighbour coverage (REQ-L2-06).
const fn fines(width: u16, height: u16, cx: i64, cy: i64, kappa: i8) -> [i64; 4] {
    let (right, bottom) = (
        (width as i64).wrapping_sub(1),
        (height as i64).wrapping_sub(1),
    );
    if kappa > 0 {
        [-7, -7, right, bottom]
    } else if kappa == 0 {
        [0, 0, right, bottom]
    } else {
        let floor = warp_factor(WARP_R2_MAX, kappa);
        [
            cx.wrapping_sub(contracted_reach(cx, floor)),
            cy.wrapping_sub(contracted_reach(cy, floor)),
            cx.wrapping_add(contracted_reach(right.wrapping_sub(cx), floor)),
            cy.wrapping_add(contracted_reach(bottom.wrapping_sub(cy), floor)),
        ]
    }
}

impl<'t, 'p> Pictor<'t, 'p> {
    /// A painter with curvature `kappa` over `tab`.
    pub fn new(tab: &'t mut Tabula<'p>, kappa: i8) -> Self {
        let w = i32::from(tab.width);
        let h = i32::from(tab.height);
        let half = if w > h { w } else { h }.wrapping_shr(1).max(1);
        let (cx, cy) = (w.wrapping_shr(1), h.wrapping_shr(1));
        let fines = fines(tab.width, tab.height, i64::from(cx), i64::from(cy), kappa);
        Self {
            tab,
            kappa,
            cx,
            cy,
            half,
            fines,
            finis_y: h as u16,
            lumen_maximum: 15,
        }
    }

    /// A painter without curvature.
    pub fn flat(tab: &'t mut Tabula<'p>) -> Self {
        Self::new(tab, 0)
    }

    /// A ground painter with the full frame's projection, writing only projected rows below
    /// `scene_h` and tones at most `lum` (REQ-L2-06; `docs/05-estetica.md` §10). Existing
    /// constructors retain the whole frame and the full sixteen-tone range.
    pub(crate) fn ground(tab: &'t mut Tabula<'p>, kappa: i8, scene_h: u16, lum: u8) -> Self {
        let mut painter = Self::new(tab, kappa);
        painter.finis_y = scene_h.min(painter.tab.height);
        painter.lumen_maximum = lum.min(15);
        painter
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

    /// The part of the logical rows `from..=to` that can reach the frame (REQ-L2-06): loops over
    /// areas skip only rows whose points all land outside it.
    #[must_use]
    pub fn clip_rows(&self, from: i64, to: i64) -> (i64, i64) {
        let [_, top, _, bottom] = self.fines;
        (from.max(top), to.min(bottom))
    }

    /// The part of the logical columns `from..=to` that can reach the frame (REQ-L2-06).
    #[must_use]
    pub fn clip_columns(&self, from: i64, to: i64) -> (i64, i64) {
        let [left, _, right, _] = self.fines;
        (from.max(left), to.min(right))
    }

    /// Projects a logical coordinate using the full-frame radial law (REQ-L2-06).
    /// Widened coordinates also represent the neighbours of either i32 endpoint.
    fn project(&self, x: i64, y: i64) -> (i64, i64) {
        if self.kappa == 0 {
            return (x, y);
        }
        let (cx, cy) = (i64::from(self.cx), i64::from(self.cy));
        let dx = x.saturating_sub(cx);
        let dy = y.saturating_sub(cy);
        let half = i64::from(self.half);
        let r2 = dx
            .saturating_mul(dx)
            .saturating_add(dy.saturating_mul(dy))
            .saturating_mul(65_536)
            .checked_div(half.wrapping_mul(half))
            .unwrap_or(WARP_R2_MAX);
        let factor = warp_factor(r2, self.kappa);
        (
            cx.saturating_add(warp_scale(dx, factor)),
            cy.saturating_add(warp_scale(dy, factor)),
        )
    }

    /// The uncut rectangular footprint of one logical pixel (REQ-L2-06). Expansion reaches
    /// the next projected column and row independently, with a minimum extent of one pixel.
    fn coverage(&self, x: i32, y: i32) -> [i64; 4] {
        let (x, y) = (i64::from(x), i64::from(y));
        let (px, py) = self.project(x, y);
        let (mut right, mut bottom) = (px.saturating_add(1), py.saturating_add(1));
        if self.kappa > 0 {
            right = right.max(self.project(x.saturating_add(1), y).0);
            bottom = bottom.max(self.project(x, y.saturating_add(1)).1);
        }
        [px, py, right, bottom]
    }

    /// Conservative projected bounds `[left, top, right, bottom]` with exclusive ends for a
    /// logical rectangle, including each block written by `plot` (REQ-L2-06, REQ-L5-06).
    /// In the operational curvature range ±64, expansion reaches its extrema at corners;
    /// contraction may also reach an extremum at an edge point nearest either central axis.
    /// These nine samples reuse the independent column and row ends of `plot`, including the
    /// neighbour beyond the final logical pixel. Empty rectangles have no bounds.
    pub(crate) fn projected_bounds(&self, x: i32, y: i32, w: i32, h: i32) -> Option<[i64; 4]> {
        if w <= 0 || h <= 0 {
            return None;
        }
        let right = x.saturating_add(w.saturating_sub(1));
        let bottom = y.saturating_add(h.saturating_sub(1));
        let columns = [x, self.cx.clamp(x, right), right];
        let rows = [y, self.cy.clamp(y, bottom), bottom];
        let mut bounds = [i64::MAX, i64::MAX, i64::MIN, i64::MIN];
        for column in columns {
            for row in rows {
                let [px, py, qx, qy] = self.coverage(column, row);
                let [left, top, right, bottom] = &mut bounds;
                *left = (*left).min(px);
                *top = (*top).min(py);
                *right = (*right).max(qx);
                *bottom = (*bottom).max(qy);
            }
        }
        Some(bounds)
    }

    /// Whether an entire logical rectangle fits the projected writing region (REQ-L2-06,
    /// REQ-L5-06), using the same bounds as projected symbol separation. Empty rectangles do not fit.
    pub(crate) fn contains_projection(&self, x: i32, y: i32, w: i32, h: i32) -> bool {
        self.projected_bounds(x, y, w, h)
            .is_some_and(|[left, top, right, bottom]| {
                left >= 0
                    && top >= 0
                    && right <= i64::from(self.tab.width)
                    && bottom <= i64::from(self.finis_y)
            })
    }

    /// Writes one logical pixel: the block of REQ-L2-06 whose top-left corner is the projected
    /// point (one pixel unless the curvature expands), clipped to the frame.
    pub fn plot(&mut self, x: i32, y: i32, lum: u8, stratum: u8, label: u8, modo: Modo) {
        let [px, py, qx, qy] = self.coverage(x, y);
        let (w, h) = (i64::from(self.tab.width), i64::from(self.tab.height));
        let (right, bottom) = (qx.min(w), qy.min(h).min(i64::from(self.finis_y)));
        for row in py.max(0)..bottom {
            for column in px.max(0)..right {
                self.put(
                    column,
                    row,
                    lum.min(self.lumen_maximum),
                    stratum,
                    label,
                    modo,
                );
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
        self.stroke(x0, y0, x1, y1, lum, stratum, 3, thick, modo);
    }

    /// A one-pixel Bresenham edge carrying a flight label (REQ-L2-08, REQ-L2-10).
    #[allow(clippy::too_many_arguments)]
    pub fn edge(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        lum: u8,
        stratum: u8,
        label: u8,
        modo: Modo,
    ) {
        self.stroke(x0, y0, x1, y1, lum, stratum, label, 1, modo);
    }

    /// Bresenham's integer recurrence shared by labelled edges and thick lines (1965).
    #[allow(clippy::too_many_arguments)]
    fn stroke(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        lum: u8,
        stratum: u8,
        label: u8,
        thick: u8,
        modo: Modo,
    ) {
        let (mut x, mut y) = (self.clamp(x0), self.clamp(y0));
        let (x1, y1) = (self.clamp(x1), self.clamp(y1));
        let dx = x1.wrapping_sub(x).saturating_abs();
        let dy = y1.wrapping_sub(y).saturating_abs().wrapping_neg();
        let sx = if x < x1 { 1 } else { -1 };
        let sy = if y < y1 { 1 } else { -1 };
        let mut err = dx.wrapping_add(dy);
        loop {
            self.blot(x, y, lum, stratum, label, thick, modo);
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

    /// A filled disc, solid or dithered by the Bayer matrix at luminance `lum`, row by row from
    /// the top and left to right, visiting only the rows and columns that can reach the frame.
    pub fn disc(&mut self, cx: i32, cy: i32, r: i32, lum: u8, stratum: u8, dither: bool) {
        let r = self.clamp(r.saturating_abs());
        let r2 = (r as u32).saturating_mul(r as u32);
        let (cx, cy, radius) = (i64::from(cx), i64::from(cy), i64::from(r));
        let (top, bottom) = self.clip_rows(cy.wrapping_sub(radius), cy.wrapping_add(radius));
        for y in top..=bottom {
            let dy = y.wrapping_sub(cy);
            let span = i64::from(isqrt(r2.saturating_sub(dy.wrapping_mul(dy) as u32)));
            let (left, right) = self.clip_columns(cx.wrapping_sub(span), cx.wrapping_add(span));
            for x in left..=right {
                if !dither || bayer(x as u32, y as u32, lum) {
                    self.plot(x as i32, y as i32, lum, stratum, 3, Modo::Super);
                }
            }
        }
    }

    /// A filled rectangle, solid or dithered by the Bayer matrix at luminance `lum`, row by row,
    /// visiting only the rows and columns that can reach the frame.
    #[allow(clippy::too_many_arguments)]
    pub fn fill(&mut self, x: i32, y: i32, w: i32, h: i32, lum: u8, stratum: u8, dither: bool) {
        let (x0, y0) = (self.clamp(x), self.clamp(y));
        let (x1, y1) = (
            self.clamp(x.saturating_add(w)),
            self.clamp(y.saturating_add(h)),
        );
        let (top, bottom) = self.clip_rows(i64::from(y0), i64::from(y1).wrapping_sub(1));
        let (left, right) = self.clip_columns(i64::from(x0), i64::from(x1).wrapping_sub(1));
        for py in top..=bottom {
            for px in left..=right {
                if !dither || bayer(px as u32, py as u32, lum) {
                    self.plot(px as i32, py as i32, lum, stratum, 3, Modo::Super);
                }
            }
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
    /// Neighbours of i32 endpoints are formed after widening; expansion dimensions differ.
    #[test]
    fn coverage_at_coordinate_extremes() {
        let mut pixels = [VOID; 320 * 180];
        let mut tab = Tabula::new(320, 180, &mut pixels).unwrap();
        let painter = Pictor::new(&mut tab, 64);
        for x in [i32::MIN, i32::MAX] {
            let [px, py, right, bottom] = painter.coverage(x, 40);
            assert_eq!(right, (px + 1).max(painter.project(i64::from(x) + 1, 40).0));
            assert_eq!(bottom, (py + 1).max(painter.project(i64::from(x), 41).1));
            assert_eq!(right - px, 2);
        }
        assert!((0..320).any(|x| {
            let [px, py, right, bottom] = painter.coverage(x, 40);
            right - px != bottom - py
        }));
    }

    /// The least expansion retains a one-pixel stroke (REQ-L2-06).
    #[test]
    fn expansion_keeps_stroke_width() {
        for (kappa, limit) in [(1, 1), (64, 3)] {
            let mut pixels = [VOID; 320 * 180];
            let mut tab = Tabula::new(320, 180, &mut pixels).unwrap();
            Pictor::new(&mut tab, kappa).line(0, 40, 319, 40, 15, 0, 1, Modo::Super);
            let mut reached = 0;
            for x in 0..320 {
                let count = (0..180)
                    .filter(|&y| lum_of(pixels[y * 320 + x]) == 15)
                    .count();
                assert!(count <= limit, "kappa {kappa}, column {x}: {count}");
                reached += usize::from(count > 0);
            }
            assert_eq!(reached, 320);
        }
    }

    /// Ground restrictions are applied after the full frame projection, including expanded
    /// blocks, without touching the caller's suffix (REQ-L2-06, REQ-H-02).
    #[test]
    fn ground_limits_projected_pixels() {
        for kappa in [-64, 0, 64] {
            let mut ordinary = [0xA5; 64 * 48 + 8];
            let mut bounded = ordinary;
            for (buf, ground) in [(&mut ordinary, false), (&mut bounded, true)] {
                let mut tab = crate::hospes::Tabula::new(64, 48, buf).unwrap();
                let mut p = if ground {
                    Pictor::ground(&mut tab, kappa, 32, 4)
                } else {
                    Pictor::new(&mut tab, kappa)
                };
                for y in -16..80 {
                    for x in -16..80 {
                        p.plot(x, y, 12, 2, 0, Modo::Super);
                    }
                }
            }
            for (at, (&normal, &ground)) in ordinary[..64 * 48].iter().zip(&bounded).enumerate() {
                if at / 64 >= 32 || normal == 0xA5 {
                    assert_eq!(ground, 0xA5);
                } else {
                    assert_eq!(ground, crate::signum::pixel::pixel(4, 2, 0));
                }
            }
            assert_eq!(&bounded[64 * 48..], &[0xA5; 8]);
        }
    }

    /// Projected bounds contain every written block in portrait, square and landscape frames,
    /// and containment agrees with visiting every logical pixel throughout the operational
    /// curvature range (REQ-L2-06, REQ-L5-06).
    #[test]
    fn projected_rectangle_containment() {
        for (frame_w, frame_h) in [(64u16, 48u16), (48, 64), (64, 64), (65, 65)] {
            let mut buffer = std::vec![0; usize::from(frame_w) * usize::from(frame_h)];
            let mut tab = Tabula::new(frame_w, frame_h, &mut buffer).unwrap();
            let scene_h = frame_h - 8;
            for kappa in (-64i8..=64).step_by(8) {
                let painter = Pictor::ground(&mut tab, kappa, scene_h, 15);
                for x in (-16..80).step_by(11) {
                    for y in (-12..76).step_by(9) {
                        for (w, h) in [(1, 1), (7, 11), (21, 9), (33, 25)] {
                            let [left, top, right, bottom] =
                                painter.projected_bounds(x, y, w, h).unwrap();
                            let mut contained = true;
                            for column in x..x + w {
                                for row in y..y + h {
                                    let [px, py, qx, qy] = painter.coverage(column, row);
                                    assert!(
                                        left <= px && top <= py && right >= qx && bottom >= qy,
                                        "{frame_w}x{frame_h}, curvature {kappa}, rectangle {x},{y},{w},{h}: ({column},{row}) projects to {px},{py},{qx},{qy} outside {left},{top},{right},{bottom}"
                                    );
                                    contained &= px >= 0
                                        && py >= 0
                                        && qx <= i64::from(frame_w)
                                        && qy <= i64::from(scene_h);
                                }
                            }
                            assert_eq!(
                                painter.contains_projection(x, y, w, h),
                                contained,
                                "{frame_w}x{frame_h}, curvature {kappa}, rectangle {x},{y},{w},{h}"
                            );
                        }
                    }
                }
                assert_eq!(painter.projected_bounds(0, 0, 0, 1), None);
                assert_eq!(painter.projected_bounds(0, 0, 1, 0), None);
            }
        }
    }

    /// The shared bounds enclose every projected block of representative annotation and
    /// symbol rectangles at operational resolutions and curvature extremes (REQ-L2-06, REQ-L5-06).
    #[test]
    fn projected_bounds_cover_operational_rectangles() {
        for (w, h) in [
            (320u16, 180u16),
            (320, 320),
            (321, 321),
            (360, 640),
            (640, 640),
            (1920, 1080),
        ] {
            let mut buffer = std::vec![0; usize::from(w) * usize::from(h)];
            let mut tab = Tabula::new(w, h, &mut buffer).unwrap();
            for kappa in [-64, 0, 64] {
                let painter = Pictor::new(&mut tab, kappa);
                let (fw, fh) = (i32::from(w), i32::from(h));
                for (x, y, rw, rh) in [
                    (0, 0, fw / 2, fh / 2),
                    (fw / 4, fh / 4, fw / 2, fh / 2),
                    (fw / 2, fh / 2, fw / 2, fh / 2),
                    (fw / 3, fh / 8, fw / 3, 12 * (fw.min(fh) / 180).max(1)),
                ] {
                    let [left, top, right, bottom] =
                        painter.projected_bounds(x, y, rw, rh).unwrap();
                    for column in x..x + rw {
                        for row in y..y + rh {
                            let [px, py, qx, qy] = painter.coverage(column, row);
                            assert!(
                                left <= px && top <= py && right >= qx && bottom >= qy,
                                "{w}x{h}, curvature {kappa}, rectangle {x},{y},{rw},{rh}, pixel {column},{row}"
                            );
                        }
                    }
                }
            }
        }
    }

    use crate::hospes::Tabula;
    use crate::natura::galois;
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

    /// The primitives write only the prefix of width·height pixels: lines, discs, rectangles and
    /// points with wild coordinates under every curvature, over frame sizes from the Galois
    /// generator, leave the bytes after the frame intact (REQ-H-02).
    #[test]
    fn pictor_writes_only_the_prefix() {
        let mut buf = [0u8; 4096 + 64];
        let mut semen = 1u16;
        let mut next = || {
            semen = galois(semen);
            semen
        };
        for _ in 0..512 {
            let w = 1 + (next() & 63);
            let hh = 1 + (next() & 63);
            let kappa = next() as i8;
            let area = usize::from(w) * usize::from(hh);
            buf.fill(0xA5);
            let mut tab = Tabula::new(w, hh, &mut buf).unwrap();
            let mut pictor = Pictor::new(&mut tab, kappa);
            let mut c = || i32::from(next() & 0xFF) - 128;
            let (x0, y0, x1, y1) = (c(), c(), c(), c());
            let thick = 1 + (next() & 1) as u8;
            pictor.line(x0, y0, x1, y1, 12, 0, thick, Modo::Super);
            pictor.disc(x0, y0, i32::from(next() & 31), 9, 1, true);
            pictor.fill(x1, y1, 40, 40, 5, 2, false);
            pictor.plot(i32::MAX, i32::MIN, 1, 0, 0, Modo::Dominans);
            pictor.plot(w.into(), hh.into(), 15, 0, 3, Modo::Super);
            assert!(
                buf[area..].iter().all(|&b| b == 0xA5),
                "{w}×{hh} κ {kappa}: past the frame"
            );
        }
    }

    /// Density is measured inside the continuous radial image, excluding the exterior corners
    /// of its enclosing rectangle. Ray intersection with the source square is independent of
    /// the integer projection and coverage implementation (REQ-L2-06).
    #[test]
    fn curvature_density() {
        for (w, h) in [(320u16, 180u16), (640, 360), (1920, 1080)] {
            for kappa in [-64i8, -1, 1, 64] {
                let mut buf = std::vec![VOID; usize::from(w) * usize::from(h)];
                let mut tab = Tabula::new(w, h, &mut buf).unwrap();
                let side = i32::from(h) / 2;
                let (cx, cy) = (i32::from(w) / 2, i32::from(h) / 2);
                let (x0, y0) = (cx - side / 2, cy - side / 2);
                Pictor::new(&mut tab, kappa).fill(x0, y0, side, side, 15, 0, false);
                let mut required = 0;
                let mut covered = 0;
                for y in 0..i32::from(h) {
                    for x in 0..i32::from(w) {
                        let dx = f64::from(x - cx) + 0.5;
                        let dy = f64::from(y - cy) + 0.5;
                        let tx = f64::from(if dx > 0.0 { x0 + side - cx } else { x0 - cx }) / dx;
                        let ty = f64::from(if dy > 0.0 { y0 + side - cy } else { y0 - cy }) / dy;
                        let t = tx.min(ty);
                        let half = f64::from(w.max(h) / 2);
                        let radius = (t * t * (dx * dx + dy * dy) / (half * half)).min(2.0);
                        let factor = if kappa >= 0 {
                            1.0 + f64::from(kappa) * radius / 128.0
                        } else {
                            1.0 / (1.0 - f64::from(kappa) * radius / 128.0)
                        };
                        if t * factor > 1.0 {
                            required += 1;
                            covered += usize::from(
                                lum_of(buf[y as usize * usize::from(w) + x as usize]) == 15,
                            );
                        }
                    }
                }
                assert!(required > 0);
                assert!(
                    covered * 100 >= required * 95,
                    "{w}x{h}, kappa {kappa}: {covered}/{required}"
                );
            }
        }
    }

    /// No logical point outside the reachable area reaches the frame: for small frames and the
    /// whole range of curvature, every point whose block touches the frame lies inside `fines`.
    /// The scanned window reaches 4·half + 16 beyond the frame, past any point that the
    /// strongest contraction (21 845/65 536 > 1/4) can bring inside (REQ-L2-06).
    #[test]
    fn reachable_area_bounds_the_projection() {
        for (w, h) in [(7u16, 5u16), (16, 9), (1, 9), (9, 1), (33, 17), (64, 64)] {
            for kappa in [-128i8, -64, -48, -1, 0, 1, 64, 127] {
                let mut buf = std::vec![VOID; usize::from(w) * usize::from(h)];
                let mut tab = Tabula::new(w, h, &mut buf).unwrap();
                let pictor = Pictor::new(&mut tab, kappa);
                let [left, top, right, bottom] = pictor.fines;
                let margin = i64::from(pictor.half) * 4 + 16;
                let (wl, hl) = (i64::from(w), i64::from(h));
                let mut touching = 0;
                for y in -margin..hl + margin {
                    for x in -margin..wl + margin {
                        let [px, py, qx, qy] = pictor.coverage(x as i32, y as i32);
                        if qx > 0 && px < wl && qy > 0 && py < hl {
                            touching += 1;
                            assert!(
                                (left..=right).contains(&x) && (top..=bottom).contains(&y),
                                "{w}×{h} κ {kappa}: ({x}, {y}) lands outside {:?}",
                                pictor.fines
                            );
                        }
                    }
                }
                assert!(touching > 0, "{w}×{h} κ {kappa}: nothing reaches the frame");
            }
        }
    }

    /// Bounding the loops to the reachable area changes no pixel: discs and rectangles drawn
    /// across the borders equal the same shapes plotted point by point, in the same order.
    #[test]
    fn bounded_loops_match_every_point() {
        let (w, h) = (24u16, 16u16);
        for kappa in [-128i8, -64, 0, 64, 127] {
            for (cx, cy, r) in [
                (0, 0, 9),
                (23, 15, 12),
                (-10, 8, 14),
                (12, 30, 20),
                (12, 8, 40),
            ] {
                let mut bounded = [VOID; 24 * 16];
                let mut tab = Tabula::new(w, h, &mut bounded).unwrap();
                let mut pictor = Pictor::new(&mut tab, kappa);
                pictor.disc(cx, cy, r, 9, 1, true);
                pictor.fill(cx - r, cy - r / 2, r, r, 13, 2, false);
                let mut every = [VOID; 24 * 16];
                let mut tab = Tabula::new(w, h, &mut every).unwrap();
                let mut pictor = Pictor::new(&mut tab, kappa);
                for dy in -r..=r {
                    let span = isqrt((r * r - dy * dy) as u32) as i32;
                    for dx in -span..=span {
                        let (x, y) = (cx + dx, cy + dy);
                        if bayer(x as u32, y as u32, 9) {
                            pictor.plot(x, y, 9, 1, 3, Modo::Super);
                        }
                    }
                }
                for y in cy - r / 2..cy - r / 2 + r {
                    for x in cx - r..cx {
                        pictor.plot(x, y, 13, 2, 3, Modo::Super);
                    }
                }
                assert_eq!(bounded, every, "κ {kappa} disc at ({cx}, {cy}) r {r}");
            }
        }
    }

    /// Discs and moons of extreme radius never panic: the absolute value of the radius saturates
    /// instead of overflowing (REQ-L0-06).
    #[test]
    fn primitives_total_at_extremes() {
        use crate::signum::caelum::moon;
        for r in [i32::MIN, i32::MIN + 1, -1, 0, i32::MAX] {
            for kappa in [-128i8, 0, 127] {
                let mut buf = [VOID; 16 * 16];
                let mut tab = Tabula::new(16, 16, &mut buf).unwrap();
                let mut pictor = Pictor::new(&mut tab, kappa);
                pictor.disc(8, 8, r, 9, 1, true);
                moon(&mut pictor, 8, 8, r, 64, 12, true);
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
