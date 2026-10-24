//! `spatium`: space (level L2). Fixed-point arithmetic, trigonometry without a floating-point
//! unit, multiplicative dispersion, the curvature projection, parallax strata, cyclic stratum
//! dominance, the dimetric projection with its null vector, the closed staircase, and the
//! composition of the frame from the golden and silver ratios.

/// Q16.16 fixed-point number: 16 integer bits, 16 fractional bits, in an `i32`.
///
/// Products and quotients are computed in `i64` and saturate to the `i32` range (REQ-L2-01).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Fx(pub i32);

/// Saturates an `i64` to the `i32` range.
const fn sat(v: i64) -> i32 {
    if v > i32::MAX as i64 {
        i32::MAX
    } else if v < i32::MIN as i64 {
        i32::MIN
    } else {
        v as i32
    }
}

impl Fx {
    /// Zero.
    pub const ZERO: Self = Self(0);
    /// One (`1 << 16`).
    pub const ONE: Self = Self(1 << 16);

    /// The integer `i`, saturating beyond ±32 767.
    #[must_use]
    pub const fn from_int(i: i32) -> Self {
        Self(sat((i as i64).wrapping_shl(16)))
    }

    /// The ratio `num / den`, exact to the unit for every representable result and saturating
    /// beyond; a zero denominator saturates by the sign of `num`.
    #[must_use]
    pub const fn from_ratio(num: i32, den: i32) -> Self {
        match (num as i64).wrapping_shl(16).checked_div(den as i64) {
            Some(q) => Self(sat(q)),
            None if num > 0 => Self(i32::MAX),
            None if num < 0 => Self(i32::MIN),
            None => Self::ZERO,
        }
    }

    /// The floor of the value as an integer.
    #[must_use]
    pub const fn to_int(self) -> i32 {
        self.0.wrapping_shr(16)
    }

    /// Saturating product.
    #[must_use]
    pub const fn mul(self, other: Self) -> Self {
        Self(sat((self.0 as i64)
            .wrapping_mul(other.0 as i64)
            .wrapping_shr(16)))
    }

    /// Saturating quotient; division by zero saturates by the sign of the dividend.
    #[must_use]
    pub const fn div(self, other: Self) -> Self {
        let num = (self.0 as i64).wrapping_shl(16);
        match num.checked_div(other.0 as i64) {
            Some(q) => Self(sat(q)),
            None if self.0 > 0 => Self(i32::MAX),
            None if self.0 < 0 => Self(i32::MIN),
            None => Self::ZERO,
        }
    }

    /// Saturating sum.
    #[must_use]
    pub const fn add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Saturating difference.
    #[must_use]
    pub const fn sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    /// Saturating negation.
    #[must_use]
    pub const fn neg(self) -> Self {
        Self(self.0.saturating_neg())
    }

    /// Saturating absolute value.
    #[must_use]
    pub const fn abs(self) -> Self {
        Self(self.0.saturating_abs())
    }
}

/// φ = (1 + √5)/2.
pub const PHI: Fx = Fx(0x19E37);
/// φ⁻¹.
pub const PHI_INV: Fx = Fx(0x9E37);
/// φ⁻².
pub const PHI_INV2: Fx = Fx(0x61C9);
/// φ⁻³.
pub const PHI_INV3: Fx = Fx(0x3C6F);
/// φ⁻⁴.
pub const PHI_INV4: Fx = Fx(0x255A);
/// The silver ratio δS = 1 + √2.
pub const SILVER: Fx = Fx(0x26A0A);
/// π.
pub const PI: Fx = Fx(0x3243F);
/// Multiplicative dispersion constant: ⌊2³²/φ⌋, odd (Knuth, 1973, §6.4).
pub const KNUTH32: u32 = 0x9E37_79B9;
/// Multiplicative dispersion constant: ⌊2⁶⁴/φ⌋, odd.
pub const KNUTH64: u64 = 0x9E37_79B9_7F4A_7C15;
/// Parallax velocities of the four strata: φ⁻¹, φ⁻², φ⁻³, φ⁻⁴ (REQ-L2-07).
pub const STRATA: [Fx; 4] = [PHI_INV, PHI_INV2, PHI_INV3, PHI_INV4];

/// Sine of an angle measured in 1/65 536 of a turn, by Bhaskara I's rational approximation
/// (REQ-L2-02): on the half turn `H`, sin(x) ≈ 16x(H − x) / (5H² − 4x(H − x)).
#[must_use]
pub const fn sin(angle: u16) -> Fx {
    const H: u64 = 32_768;
    let x = (angle as u64) & 0x7FFF;
    let hx = x.wrapping_mul(H.wrapping_sub(x));
    let num = hx.wrapping_mul(16).wrapping_shl(16);
    let den = (5 * H * H).wrapping_sub(hx.wrapping_mul(4));
    let s = match num.checked_div(den) {
        Some(q) => q as i32,
        None => 0,
    };
    if angle >= 32_768 {
        Fx(s.wrapping_neg())
    } else {
        Fx(s)
    }
}

/// Cosine, as the sine shifted by a quarter turn.
#[must_use]
pub const fn cos(angle: u16) -> Fx {
    sin(angle.wrapping_add(16_384))
}

/// ⌊√n⌋ by the digit-by-digit method, two bits per step (REQ-L2-03).
#[must_use]
pub const fn isqrt(n: u32) -> u32 {
    let mut rest = n;
    let mut root = 0u32;
    let mut bit = 1u32 << 30;
    while bit > rest {
        bit = bit.wrapping_shr(2);
    }
    while bit != 0 {
        let trial = root.wrapping_add(bit);
        if rest >= trial {
            rest = rest.wrapping_sub(trial);
            root = root.wrapping_shr(1).wrapping_add(bit);
        } else {
            root = root.wrapping_shr(1);
        }
        bit = bit.wrapping_shr(2);
    }
    root
}

/// arctan(2⁻ⁱ) for i in 0..16, in 1/2²⁴ of a turn (eight fractional bits beyond the angle unit).
const ATAN: [i64; 16] = [
    2_097_152, 1_238_021, 654_136, 332_050, 166_669, 83_416, 41_718, 20_860, 10_430, 5_215, 2_608,
    1_304, 652, 326, 163, 81,
];
/// CORDIC gain compensation K ≈ 0.607 253 in Q8.24.
const CORDIC_K: i64 = 10_188_012;

/// Rotates `(x, y)` by `angle` (1/65 536 of a turn) with 16 CORDIC iterations: shifts and
/// additions only, with eight guard bits on the vector and on the angle (Volder, 1959; REQ-L2-04).
/// Total over the whole type: the error is at most 4 units plus 2⁻¹⁵ of the magnitude, and the
/// result saturates to the `i32` range.
#[must_use]
pub fn cordic(x: Fx, y: Fx, angle: u16) -> (Fx, Fx) {
    let mut z = i64::from(angle);
    if z > 32_767 {
        z = z.wrapping_sub(65_536);
    }
    let (mut px, mut py) = (
        i64::from(x.0).wrapping_shl(8),
        i64::from(y.0).wrapping_shl(8),
    );
    if z > 16_384 {
        px = px.wrapping_neg();
        py = py.wrapping_neg();
        z = z.wrapping_sub(32_768);
    } else if z < -16_384 {
        px = px.wrapping_neg();
        py = py.wrapping_neg();
        z = z.wrapping_add(32_768);
    }
    z = z.wrapping_shl(8);
    for (i, atan) in (0u32..).zip(ATAN) {
        let (dx, dy) = (px.wrapping_shr(i), py.wrapping_shr(i));
        if z >= 0 {
            px = px.wrapping_sub(dy);
            py = py.wrapping_add(dx);
            z = z.wrapping_sub(atan);
        } else {
            px = px.wrapping_add(dy);
            py = py.wrapping_sub(dx);
            z = z.wrapping_add(atan);
        }
    }
    (
        Fx(sat(px
            .wrapping_shr(8)
            .wrapping_mul(CORDIC_K)
            .wrapping_shr(24))),
        Fx(sat(py
            .wrapping_shr(8)
            .wrapping_mul(CORDIC_K)
            .wrapping_shr(24))),
    )
}

/// Disperses `k` over `bits` output bits by multiplication with [`KNUTH32`] (REQ-L2-05).
#[must_use]
pub const fn scatter32(k: u32, bits: u32) -> u32 {
    if bits == 0 {
        return 0;
    }
    let bits = if bits > 32 { 32 } else { bits };
    k.wrapping_mul(KNUTH32)
        .wrapping_shr(32u32.wrapping_sub(bits))
}

/// Disperses `k` over `bits` output bits by multiplication with [`KNUTH64`] (REQ-L2-05).
#[must_use]
pub const fn scatter64(k: u64, bits: u32) -> u64 {
    if bits == 0 {
        return 0;
    }
    let bits = if bits > 64 { 64 } else { bits };
    k.wrapping_mul(KNUTH64)
        .wrapping_shr(64u32.wrapping_sub(bits))
}

/// Largest squared radius of the curvature projection, in Q16.16: r² = 2, the corner of a square
/// frame (REQ-L2-06). Beyond it the factor stays constant and the map stays radial and linear.
pub const WARP_R2_MAX: i64 = 2 << 16;

/// The radial factor of the curvature projection in Q16.16 for a squared radius `r2` in Q16.16,
/// with κ = `kappa`/128 and `r2` clamped to 0..=[`WARP_R2_MAX`] (REQ-L2-06):
/// 65 536 + (`kappa`·r2) ≫ 7, that is 1 + κr², for κ ≥ 0; 2³²/(65 536 + (|`kappa`|·r2) ≫ 7),
/// that is 1/(1 + |κ|r²), for κ < 0. The contraction is monotone while |κ|r² ≤ 1, which holds over
/// the whole operational domain (|`curvatura`| ≤ 64, r² ≤ 2), and equals 1 + κr² to the second
/// order (the Poincaré disk).
#[must_use]
pub const fn warp_factor(r2: i64, kappa: i8) -> i64 {
    let r2 = if r2 < 0 {
        0
    } else if r2 > WARP_R2_MAX {
        WARP_R2_MAX
    } else {
        r2
    };
    let k = kappa as i64;
    if k >= 0 {
        65_536i64.wrapping_add(k.wrapping_mul(r2).wrapping_shr(7))
    } else {
        let den = 65_536i64.wrapping_add(k.wrapping_neg().wrapping_mul(r2).wrapping_shr(7));
        match (1i64 << 32).checked_div(den) {
            Some(f) => f,
            None => 65_536,
        }
    }
}

/// Integer upper estimate of radial expansion: 1 + ⌈3·`kappa`·r2/2²³⌉, at most 8, from
/// the radial derivative 1 + 3κr² of [`warp_factor`], with `r2` in Q16.16 clamped to
/// [`WARP_R2_MAX`]; one without expansion. The raster coverage of REQ-L2-06 is instead
/// determined by adjacent projected coordinates in [`crate::signum::raster::Pictor`].
#[must_use]
pub const fn warp_block(r2: i64, kappa: i8) -> i64 {
    if kappa <= 0 {
        return 1;
    }
    let r2 = if r2 < 0 {
        0
    } else if r2 > WARP_R2_MAX {
        WARP_R2_MAX
    } else {
        r2
    };
    let spread = 3i64
        .wrapping_mul(kappa as i64)
        .wrapping_mul(r2)
        .wrapping_add((1 << 23) - 1)
        .wrapping_shr(23);
    let side = 1i64.wrapping_add(spread);
    if side > 8 { 8 } else { side }
}

/// `d`·`f`/65 536 rounded half away from zero: a displacement scaled by a Q16.16 factor of
/// [`warp_factor`] (REQ-L2-06). Saturates beyond the `i64` range.
#[must_use]
pub const fn warp_scale(d: i64, f: i64) -> i64 {
    let n = d.saturating_mul(f);
    let q = (n.unsigned_abs().saturating_add(32_768) >> 16) as i64;
    if n < 0 { q.wrapping_neg() } else { q }
}

/// Curvature projection of a point in coordinates normalized to the frame center in Q8.8
/// (256 = half of the larger dimension), REQ-L2-06: r² = x² + y² in Q16.16, the factor of
/// [`warp_factor`], and each coordinate scaled by [`warp_scale`] and clamped to the `i16` range.
/// Zero curvature is the identity; negative contracts toward the center without folding;
/// positive expands.
#[must_use]
pub const fn warp(x: i16, y: i16, kappa: i8) -> (i16, i16) {
    let (xi, yi) = (x as i64, y as i64);
    let r2 = xi.wrapping_mul(xi).wrapping_add(yi.wrapping_mul(yi));
    let factor = warp_factor(r2, kappa);
    (
        clamp_i16(warp_scale(xi, factor)),
        clamp_i16(warp_scale(yi, factor)),
    )
}

/// Clamps to the `i16` range.
const fn clamp_i16(v: i64) -> i16 {
    if v > i16::MAX as i64 {
        i16::MAX
    } else if v < i16::MIN as i64 {
        i16::MIN
    } else {
        v as i16
    }
}

/// Cyclic dominance of stratum classes packed as one bit per (new, old) pair, index
/// `(new << 2) | old`: 0 ≻ 2, 2 ≻ 1, 1 ≻ 0; class 3 (void) is always overwritten; equal
/// classes do not dominate (REQ-L2-08).
pub const DOMINANCE_LUT: u16 = 0x0A9C;

/// Whether a pixel of class `new` overwrites a pixel of class `old` (REQ-L2-08).
#[must_use]
pub const fn dominates(new: u8, old: u8) -> bool {
    let index = (((new & 3) as u32) << 2) | ((old & 3) as u32);
    (DOMINANCE_LUT as u32).wrapping_shr(index) & 1 == 1
}

/// Dimetric 2:1 projection u = x − z, v = y + ⌊(x + z)/2⌋; its kernel is spanned by
/// (2, −2, 2) (REQ-L2-09).
#[must_use]
pub const fn dimetric(x: i32, y: i32, z: i32) -> (i32, i32) {
    (
        x.wrapping_sub(z),
        y.wrapping_add(x.wrapping_add(z).wrapping_shr(1)),
    )
}

/// Position on the closed staircase of four flights of `span` steps around `(cx, cy)`, with
/// the stratum class that lets the last flight pass behind the first (REQ-L2-10). Returns
/// `(x, y, class)`; the whole triple is periodic: `position(t + 4·span) = position(t)`. The
/// class is 0 on the first flight and 2 on the last for every `span`; `span` is clamped to
/// 1..=16 384.
#[must_use]
pub const fn penrose(step: u32, cx: i32, cy: i32, span: u32) -> (i32, i32, u8) {
    let span = if span == 0 {
        1
    } else if span > 16_384 {
        16_384
    } else {
        span
    };
    let lap_len = span.wrapping_mul(4);
    let lap = match step.checked_rem(lap_len) {
        Some(r) => r,
        None => 0,
    };
    let phase = match lap.checked_div(span) {
        Some(q) => q & 3,
        None => 0,
    };
    let local = match lap.checked_rem(span) {
        Some(r) => r as i32,
        None => 0,
    };
    let s = span as i32;
    let half = local.wrapping_shr(1);
    let (u, v) = match phase {
        0 => (local, half.wrapping_neg()),
        1 => (
            s.wrapping_sub(local),
            s.wrapping_shr(1).wrapping_neg().wrapping_sub(half),
        ),
        2 => (local.wrapping_neg(), s.wrapping_neg().wrapping_add(half)),
        _ => (
            s.wrapping_neg().wrapping_add(local),
            s.wrapping_shr(1).wrapping_neg().wrapping_add(half),
        ),
    };
    let class = match lap.wrapping_mul(3).checked_div(lap_len) {
        Some(q) => q as u8,
        None => 0,
    };
    (cx.wrapping_add(u), cy.wrapping_add(v), class)
}

/// An axis-aligned rectangle in frame pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rect {
    /// Left edge.
    pub x: u16,
    /// Top edge.
    pub y: u16,
    /// Width.
    pub w: u16,
    /// Height.
    pub h: u16,
}

/// A point in frame pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Pt {
    /// Horizontal coordinate.
    pub x: i32,
    /// Vertical coordinate, downward.
    pub y: i32,
}

/// The composition of a frame from the golden and silver ratios (REQ-L2-11).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Compositio {
    /// Integer scale of figures and lines.
    pub s: u16,
    /// Margin.
    pub m: u16,
    /// Height of the plinth of chronicles at the base.
    pub plinth: u16,
    /// Height of the scene above the plinth.
    pub scene_h: u16,
    /// Region of the sky graph.
    pub sky: Rect,
    /// The slope, from its foot to its summit.
    pub slope: (Pt, Pt),
    /// Region of the chronicles.
    pub annales: Rect,
    /// Glyph cells per chronicle line.
    pub glyphs_per_line: u16,
    /// Height of the figure.
    pub figure_h: u16,
    /// Diameter of the boulder.
    pub boulder_d: u16,
    /// Integer scale of the boulder's 8×8 texture.
    pub boulder_scale: u16,
}

/// ⌊`len` · `ratio`⌋ for a pixel length and a ratio in 0..=1, exact for every `u16`.
#[must_use]
pub const fn part(len: u16, ratio: Fx) -> u16 {
    ((len as u64).wrapping_mul(ratio.0 as u64) >> 16) as u16
}

/// 8·δS in Q16.16: the divisor of the margin law m = ⌊W/(8·δS)⌋.
const SILVER_OCTUPLE: u64 = (SILVER.0 as u64) * 8;

/// Lays out a frame of `w`×`h` pixels with `lines` chronicle lines (REQ-L2-11).
#[must_use]
pub const fn compositio(width: u16, height: u16, lines: u8) -> Compositio {
    let by_width = (width as u32).wrapping_div(320);
    let by_height = (height as u32).wrapping_div(180);
    let scale0 = if by_width < by_height {
        by_width
    } else {
        by_height
    };
    let scale = if scale0 == 0 { 1u16 } else { scale0 as u16 };
    let margin = ((width as u64) << 16).wrapping_div(SILVER_OCTUPLE) as u16;
    let plinth = margin.saturating_add((lines as u16).saturating_mul(10).saturating_mul(scale));
    let scene_h = height.saturating_sub(plinth);
    let sky_right = part(width, PHI_INV);
    let sky_bottom = part(scene_h, PHI_INV2);
    let sky = Rect {
        x: margin,
        y: margin,
        w: sky_right.saturating_sub(margin),
        h: sky_bottom.saturating_sub(margin),
    };
    let slope = (
        Pt {
            x: part(width, PHI_INV2) as i32,
            y: scene_h as i32,
        },
        Pt {
            x: width.saturating_sub(margin) as i32,
            y: sky_bottom as i32,
        },
    );
    let inner = width.saturating_sub(margin.saturating_mul(2));
    let annales = Rect {
        x: margin,
        y: scene_h,
        w: inner,
        h: height.saturating_sub(margin).saturating_sub(scene_h),
    };
    let glyphs_per_line = match (inner as u32).checked_div(6u32.wrapping_mul(scale as u32)) {
        Some(q) => q as u16,
        None => 0,
    };
    let figure_h = part(scene_h, PHI_INV3);
    let boulder_d = ((figure_h as u64)
        .wrapping_mul(PHI_INV.0 as u64)
        .wrapping_add(0xFFFF)
        >> 16) as u16;
    let boulder_scale = {
        let q = boulder_d.saturating_add(4).wrapping_div(8);
        if q == 0 { 1 } else { q }
    };
    Compositio {
        s: scale,
        m: margin,
        plinth,
        scene_h,
        sky,
        slope,
        annales,
        glyphs_per_line,
        figure_h,
        boulder_d,
        boulder_scale,
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::arithmetic_side_effects,
        clippy::indexing_slicing,
        clippy::float_arithmetic,
        clippy::float_cmp,
        clippy::cast_precision_loss
    )]
    use super::*;

    /// Products and quotients saturate instead of overflowing (REQ-L2-01).
    #[test]
    fn fixed_mul_div_saturate() {
        let max = Fx(i32::MAX);
        let min = Fx(i32::MIN);
        assert_eq!(max.mul(max), max);
        assert_eq!(min.mul(max), min);
        assert_eq!(max.div(Fx(1)), max);
        assert_eq!(min.div(Fx(1)), min);
        assert_eq!(Fx::ONE.div(Fx::ZERO), max);
        assert_eq!(Fx(-1).div(Fx::ZERO), min);
        assert_eq!(Fx::ZERO.div(Fx::ZERO), Fx::ZERO);
        assert_eq!(
            Fx::from_int(3).mul(Fx::from_ratio(1, 2)),
            Fx::from_ratio(3, 2)
        );
        assert_eq!(Fx::from_int(3).div(Fx::from_int(2)), Fx::from_ratio(3, 2));
        assert_eq!(Fx::from_int(40_000), max);
        assert_eq!(Fx::from_int(-40_000), min);
        assert_eq!(Fx::from_ratio(-7, 2).to_int(), -4);
        assert_eq!(Fx::from_ratio(40_000, 2), Fx(1_310_720_000));
        assert_eq!(Fx::from_ratio(1, 100_000), Fx(0));
        assert_eq!(Fx::from_ratio(3, 0), max);
        assert_eq!(Fx::from_ratio(-3, 0), min);
        assert_eq!(Fx::from_ratio(0, 0), Fx::ZERO);
        assert_eq!(Fx::from_ratio(i32::MAX, 1), max);
        assert_eq!(max.add(Fx::ONE), max);
        assert_eq!(min.sub(Fx::ONE), min);
    }

    /// Bhaskara I's sine stays within 110 units of Q16.16 of the true sine (REQ-L2-02).
    #[test]
    fn bhaskara_max_error() {
        let mut worst = 0i64;
        for angle in 0..=u16::MAX {
            let exact = (f64::from(angle) / 65536.0 * core::f64::consts::TAU).sin() * 65536.0;
            let got = i64::from(sin(angle).0);
            worst = worst.max((got - exact.round() as i64).abs());
        }
        assert!(worst <= 110, "worst error {worst}");
        assert_eq!(sin(0), Fx::ZERO);
        assert_eq!(sin(16384), Fx::ONE);
        assert_eq!(sin(32768), Fx::ZERO);
        assert_eq!(sin(49152), Fx(-65536));
        assert_eq!(cos(0), Fx::ONE);
        assert_eq!(cos(16384), Fx::ZERO);
    }

    /// `isqrt` is the floor of the square root over a dispersed sweep and the extremes (REQ-L2-03).
    #[test]
    fn isqrt_floor() {
        for n in [0u32, 1, 2, 3, 4, 15, 16, 17, u32::MAX - 1, u32::MAX] {
            let r = isqrt(n);
            assert!(u64::from(r) * u64::from(r) <= u64::from(n));
            assert!((u64::from(r) + 1) * (u64::from(r) + 1) > u64::from(n));
        }
        for k in 0..65_535u32 {
            let n = scatter32(k, 32);
            let r = isqrt(n);
            assert!(u64::from(r) * u64::from(r) <= u64::from(n));
            assert!((u64::from(r) + 1) * (u64::from(r) + 1) > u64::from(n));
        }
    }

    /// CORDIC rotates unit vectors with at most 4 units of error per component (REQ-L2-04).
    #[test]
    fn cordic_unit_rotation() {
        for angle in (0..=u16::MAX).step_by(97) {
            let (x, y) = cordic(Fx::ONE, Fx::ZERO, angle);
            let a = f64::from(angle) / 65536.0 * core::f64::consts::TAU;
            let ex = (a.cos() * 65536.0).round() as i32;
            let ey = (a.sin() * 65536.0).round() as i32;
            assert!((x.0 - ex).abs() <= 4, "angle {angle}: x {} vs {ex}", x.0);
            assert!((y.0 - ey).abs() <= 4, "angle {angle}: y {} vs {ey}", y.0);
        }
    }

    /// Rotations by zero and by a half turn keep any vector within 2⁻¹⁵ of its magnitude plus 4
    /// units, even at the extremes of the type.
    #[test]
    fn cordic_total_over_i32() {
        for (x, y) in [
            (i32::MAX, i32::MAX),
            (i32::MIN, 0),
            (0, i32::MIN),
            (1 << 30, -(1 << 30)),
        ] {
            let (rx, ry) = cordic(Fx(x), Fx(y), 0);
            let tol = (i64::from(x).abs() + i64::from(y).abs()) / 32_768 + 4;
            assert!(
                (i64::from(rx.0) - i64::from(x)).abs() <= tol,
                "x {x} -> {}",
                rx.0
            );
            assert!(
                (i64::from(ry.0) - i64::from(y)).abs() <= tol,
                "y {y} -> {}",
                ry.0
            );
            let (hx, hy) = cordic(Fx(x), Fx(y), 32_768);
            assert!(
                (i64::from(hx.0) + i64::from(x)).abs() <= tol,
                "half turn x {x} -> {}",
                hx.0
            );
            assert!(
                (i64::from(hy.0) + i64::from(y)).abs() <= tol,
                "half turn y {y} -> {}",
                hy.0
            );
        }
    }

    /// The dispersion constants are the golden-ratio reciprocals of 2³² and 2⁶⁴ (REQ-L2-05).
    #[test]
    fn knuth_constants() {
        assert_eq!(KNUTH32, 0x9E37_79B9);
        assert_eq!(KNUTH64, 0x9E37_79B9_7F4A_7C15);
        assert_eq!(scatter32(1, 8), 0x9E);
        assert_eq!(scatter32(1, 32), KNUTH32);
        assert_eq!(scatter32(1, 0), 0);
        assert_eq!(scatter64(1, 16), 0x9E37);
        let mut seen = [false; 256];
        let mut distinct = 0u32;
        for k in 0..256u32 {
            let p = scatter32(k, 8) as usize;
            distinct += u32::from(!seen[p]);
            seen[p] = true;
        }
        assert!(distinct >= 160, "dispersion too clustered: {distinct}");
    }

    /// Zero curvature is the identity projection (REQ-L2-06).
    #[test]
    fn curvature_identity_at_zero() {
        for (x, y) in [
            (0i16, 0i16),
            (256, 0),
            (-256, 256),
            (100, -37),
            (-256, -256),
        ] {
            assert_eq!(warp(x, y, 0), (x, y));
        }
    }

    /// Negative curvature contracts toward the center; positive expands (REQ-L2-06).
    #[test]
    fn curvature_sign_direction() {
        for (x, y) in [(256i16, 0i16), (-200, 100), (37, -37), (0, 256)] {
            let (hx, hy) = warp(x, y, -64);
            let (ex, ey) = warp(x, y, 64);
            assert!(
                i32::from(hx).abs() + i32::from(hy).abs() < i32::from(x).abs() + i32::from(y).abs()
            );
            assert!(
                i32::from(ex).abs() + i32::from(ey).abs() > i32::from(x).abs() + i32::from(y).abs()
            );
        }
        // 256/(1 + 0.5·1) = 170.67, rounded half away from zero.
        assert_eq!(warp(256, 0, -64), (171, 0));
        // 256·(1 + 0.5·1) = 384.
        assert_eq!(warp(256, 0, 64), (384, 0));
        assert_eq!(warp(i16::MIN, i16::MIN, 127), (i16::MIN, i16::MIN));
        assert_eq!(warp(i16::MAX, 0, 127), (i16::MAX, 0));
        let (cx, cy) = warp(i16::MIN, i16::MIN, -128);
        assert!(cx > i16::MIN / 2 && cy > i16::MIN / 2 && cx < 0 && cy < 0);
        assert_eq!(warp(0, 0, -64), (0, 0));
    }

    /// The projection never folds: along a radius the projected coordinate never decreases, for
    /// every operational curvature, from the center to the corner of a square frame
    /// (256·√2 ≈ 362 in Q8.8) (REQ-L2-06).
    #[test]
    fn curvature_monotone() {
        for kappa in -64i8..=64 {
            let mut previous = i16::MIN;
            for x in 0i16..=362 {
                let (px, py) = warp(x, 0, kappa);
                assert!(px >= previous, "κ {kappa}: x {x} → {px} after {previous}");
                assert_eq!(py, 0);
                previous = px;
            }
        }
    }

    /// The factor is 1 + κr² under expansion and 1/(1 + |κ|r²) under contraction, in Q16.16, with
    /// r² clamped to 0..=2 (REQ-L2-06).
    #[test]
    fn curvature_factor_law() {
        assert_eq!(WARP_R2_MAX, 2 << 16);
        for kappa in [-128i8, -64, -1, 0, 1, 64, 127] {
            assert_eq!(warp_factor(0, kappa), 1 << 16, "κ {kappa} at the center");
            assert_eq!(
                warp_factor(WARP_R2_MAX, kappa),
                warp_factor(i64::MAX, kappa),
                "κ {kappa} beyond the corner"
            );
            assert_eq!(warp_factor(-5, kappa), warp_factor(0, kappa));
            for r2 in (0..=WARP_R2_MAX).step_by(997) {
                let k = f64::from(kappa) / 128.0;
                let r = r2 as f64 / 65_536.0;
                let exact = if kappa >= 0 {
                    1.0 + k * r
                } else {
                    1.0 / (1.0 - k * r)
                } * 65_536.0;
                let got = warp_factor(r2, kappa) as f64;
                assert!(
                    (got - exact).abs() <= 1.0,
                    "κ {kappa} r2 {r2}: {got} vs {exact}"
                );
            }
        }
        assert_eq!(warp_factor(1 << 16, -64), 43_690);
        assert_eq!(warp_factor(1 << 16, 64), 98_304);
        assert_eq!(warp_factor(WARP_R2_MAX, -128), 21_845);
        assert_eq!(warp_factor(WARP_R2_MAX, 127), 195_584);
    }

    /// Under expansion a point is covered by a block of side ⌈1 + 3κr²⌉, the radial derivative
    /// of the map, at most 8; without expansion the block is one pixel (REQ-L2-06).
    #[test]
    fn curvature_block_law() {
        for kappa in [-128i8, -1, 0] {
            assert_eq!(warp_block(WARP_R2_MAX, kappa), 1);
        }
        assert_eq!(warp_block(0, 64), 1);
        assert_eq!(warp_block(1, 1), 2, "any expansion off the center");
        assert_eq!(warp_block(WARP_R2_MAX, 64), 4, "1 + 3·0.5·2");
        assert_eq!(warp_block(WARP_R2_MAX, 127), 7);
        assert_eq!(warp_block(i64::MAX, 127), 7);
        for kappa in 1i8..=127 {
            for r2 in (0..=WARP_R2_MAX).step_by(4099) {
                let derivative = 1.0 + 3.0 * (f64::from(kappa) / 128.0) * (r2 as f64 / 65_536.0);
                assert_eq!(
                    warp_block(r2, kappa),
                    (derivative.ceil() as i64).min(8),
                    "κ {kappa} r2 {r2}"
                );
            }
        }
    }

    /// The strata velocities are φ⁻¹..φ⁻⁴ within one unit of Q16.16 (REQ-L2-07).
    #[test]
    fn strata_velocities_phi() {
        let phi = f64::midpoint(1.0, 5f64.sqrt());
        for (n, v) in STRATA.iter().enumerate() {
            let exact = (phi.powi(-(n as i32 + 1)) * 65536.0).round() as i32;
            assert!((v.0 - exact).abs() <= 1, "stratum {n}: {} vs {exact}", v.0);
        }
        assert_eq!(STRATA, [Fx(0x9E37), Fx(0x61C9), Fx(0x3C6F), Fx(0x255A)]);
        assert_eq!(PHI, Fx(0x19E37));
        assert_eq!(SILVER, Fx(0x26A0A));
        assert_eq!(PI, Fx(0x3243F));
    }

    /// Dominance is cyclic: 0 ≻ 2, 2 ≻ 1, 1 ≻ 0; the void is always overwritten (REQ-L2-08).
    #[test]
    fn stratum_dominance_cycle() {
        assert!(dominates(0, 2) && dominates(2, 1) && dominates(1, 0));
        assert!(!dominates(2, 0) && !dominates(1, 2) && !dominates(0, 1));
        for new in 0..3u8 {
            assert!(dominates(new, 3));
            assert!(!dominates(new, new));
        }
        assert!(!dominates(3, 0) && !dominates(3, 3));
    }

    /// The packed table equals the rule evaluated cell by cell (REQ-L2-08).
    #[test]
    fn stratum_lut_matches_rule() {
        assert_eq!(DOMINANCE_LUT, 0x0A9C);
        for new in 0..4u8 {
            for old in 0..4u8 {
                let rule = new != 3 && (old == 3 || [(0, 2), (1, 0), (2, 1)].contains(&(new, old)));
                assert_eq!(dominates(new, old), rule, "new {new} old {old}");
            }
        }
    }

    /// The vector (2, −2, 2) lies in the kernel of the dimetric projection (REQ-L2-09).
    #[test]
    fn isometric_kernel() {
        for k in -40i32..40 {
            for (x, y, z) in [
                (0i32, 0i32, 0i32),
                (5, -3, 7),
                (-11, 2, 9),
                (100, 100, -100),
            ] {
                assert_eq!(dimetric(x + 2 * k, y - 2 * k, z + 2 * k), dimetric(x, y, z));
            }
        }
        assert_eq!(dimetric(3, 1, 1), (2, 3));
    }

    /// Four flights of `span` steps close on themselves; the first flight passes before the last (REQ-L2-10).
    #[test]
    fn penrose_closed_loop() {
        let span = 24u32;
        for t in 0..4 * span {
            assert_eq!(
                penrose(t, 100, 100, span),
                penrose(t + 4 * span, 100, 100, span)
            );
        }
        let (x0, y0, l0) = penrose(0, 100, 100, span);
        let (x3, y3, l3) = penrose(4 * span - 1, 100, 100, span);
        assert!(
            (x0 - x3).abs() <= 2 && (y0 - y3).abs() <= 2,
            "({x0},{y0}) vs ({x3},{y3})"
        );
        assert_eq!((l0, l3), (0, 2));
        assert!(dominates(l0, l3));
        for span in [1u32, 2, 3, 7, 23, 24, 25, 100] {
            for t in 0..8 * span {
                assert_eq!(
                    penrose(t, 0, 0, span),
                    penrose(t + 4 * span, 0, 0, span),
                    "span {span} t {t}"
                );
                let lap = t % (4 * span);
                let (_, _, class) = penrose(t, 0, 0, span);
                assert_eq!(u32::from(class), lap * 3 / (4 * span), "span {span} t {t}");
                let first = penrose(lap % span, 0, 0, span).2;
                let last = penrose(3 * span + lap % span, 0, 0, span).2;
                assert!(
                    dominates(first, last),
                    "span {span} lap {lap}: {first} vs {last}"
                );
            }
        }
        assert_eq!(penrose(5, 0, 0, u32::MAX), penrose(5, 0, 0, 16_384));
        for flight in 0..2u32 {
            let (_, ya, _) = penrose(flight * span, 100, 100, span);
            let (_, yb, _) = penrose(flight * span + span - 1, 100, 100, span);
            assert!(yb < ya, "flight {flight} does not rise on screen");
        }
        for flight in 0..3u32 {
            let (xa, ya, _) = penrose(flight * span + span - 1, 100, 100, span);
            let (xb, yb, _) = penrose((flight + 1) * span, 100, 100, span);
            assert!(
                (xa - xb).abs() <= 2 && (ya - yb).abs() <= 2,
                "flights {flight} and next are not joined"
            );
        }
    }

    /// The composition at 320×180 with six lines (REQ-L2-11).
    #[test]
    fn composition_constants() {
        let c = compositio(320, 180, 6);
        assert_eq!((c.s, c.m, c.plinth, c.scene_h), (1, 16, 76, 104));
        assert_eq!(
            c.sky,
            Rect {
                x: 16,
                y: 16,
                w: 181,
                h: 23
            }
        );
        assert_eq!(c.slope, (Pt { x: 122, y: 104 }, Pt { x: 304, y: 39 }));
        assert_eq!(
            c.annales,
            Rect {
                x: 16,
                y: 104,
                w: 288,
                h: 60
            }
        );
        assert_eq!(c.glyphs_per_line, 48);
        assert_eq!((c.figure_h, c.boulder_d, c.boulder_scale), (24, 15, 2));
        let big = compositio(1920, 1080, 6);
        assert_eq!((big.s, big.m), (6, 99));
        let huge = compositio(40_000, 22_500, 6);
        assert_eq!(
            (huge.s, huge.m, huge.plinth, huge.scene_h),
            (125, 2071, 9571, 12_929)
        );
        assert_eq!(
            (huge.sky.w, huge.sky.h, huge.slope.0.x, huge.figure_h),
            (22_650, 2867, 15_278, 3052)
        );
        let tall = compositio(360, 800, 6);
        assert_eq!(tall.s, 1);
        assert_eq!(tall.glyphs_per_line, 54);
    }

    /// Small widths and zero lines keep every region inside the frame with non-negative sizes.
    #[test]
    fn composition_small_widths() {
        for (w, h, lines) in [
            (200u16, 120u16, 6u8),
            (272, 60, 0),
            (96, 96, 6),
            (1, 1, 6),
            (320, 180, 0),
        ] {
            let c = compositio(w, h, lines);
            assert!(c.s >= 1);
            assert!(u32::from(c.sky.x) + u32::from(c.sky.w) <= u32::from(w));
            assert!(u32::from(c.sky.y) + u32::from(c.sky.h) <= u32::from(h));
            assert!(u32::from(c.annales.x) + u32::from(c.annales.w) <= u32::from(w));
            assert!(u32::from(c.annales.y) + u32::from(c.annales.h) <= u32::from(h));
            assert!(c.slope.0.y <= i32::from(h) && c.slope.1.y >= 0);
            assert!(c.slope.1.x <= i32::from(w));
            assert!(c.boulder_scale >= 1);
        }
    }
}
