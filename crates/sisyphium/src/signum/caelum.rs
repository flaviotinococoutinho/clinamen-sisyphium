//! The sky: the constellation scattered from the automaton, the hexagram of serenity, the moon
//! by its phase and the ray of a rare event (REQ-L5-07, REQ-L5-08, REQ-L6-03).

use crate::natura::mix;
use crate::signum::margo::harmony;
use crate::signum::pixel::LABEL_NONE;
use crate::signum::raster::{Modo, Pictor, bayer};
use crate::spatium::{Compositio, Rect, isqrt, part, scatter64};

/// The region of the sky under the harmony of the burden (REQ-L5-12): the sky of the
/// composition while `onus` ≤ 200; above, its right edge is ⌊`width`·harmony(`onus`)⌋ in place of
/// ⌊`width`·φ⁻¹⌋, by the formula of the composition (REQ-L2-11).
#[must_use]
pub fn sky_region(comp: &Compositio, width: u16, onus: u8) -> Rect {
    if onus <= 200 {
        return comp.sky;
    }
    Rect {
        w: part(width, harmony(onus)).saturating_sub(comp.sky.x),
        ..comp.sky
    }
}

/// The six lines of the hexagram from the six bits of serenity, bit 0 first (the bottom line);
/// `true` is an unbroken line (REQ-L5-07).
#[must_use]
pub const fn hexagram_lines(tranquillitas: u8) -> [bool; 6] {
    let t = tranquillitas;
    [
        t & 1 != 0,
        t & 2 != 0,
        t & 4 != 0,
        t & 8 != 0,
        t & 16 != 0,
        t & 32 != 0,
    ]
}

/// Draws the hexagram with its top-left corner at `(x, y)`: six lines of 12·`s` pixels, two
/// scale units apart, the bottom line being bit 0; a broken line is two halves of 5·`s` with a
/// gap of 2·`s`.
pub fn hexagram(
    pictor: &mut Pictor,
    x: i32,
    y: i32,
    s: u16,
    tranquillitas: u8,
    lum: u8,
    stratum: u8,
) {
    let unit = i32::from(s.max(1));
    let lines = hexagram_lines(tranquillitas);
    for (k, unbroken) in (0i32..).zip(lines) {
        let row = y.wrapping_add(unit.wrapping_mul(10i32.wrapping_sub(k.wrapping_mul(2))));
        let right = x.wrapping_add(unit.wrapping_mul(12)).wrapping_sub(1);
        if unbroken {
            pictor.line(x, row, right, row, lum, stratum, s as u8, Modo::Super);
        } else {
            let gap_start = x.wrapping_add(unit.wrapping_mul(5)).wrapping_sub(1);
            let gap_end = x.wrapping_add(unit.wrapping_mul(7));
            pictor.line(x, row, gap_start, row, lum, stratum, s as u8, Modo::Super);
            pictor.line(gap_end, row, right, row, lum, stratum, s as u8, Modo::Super);
        }
    }
}

/// Scatters twelve stars over `sky` from the automaton: star j sits at the dispersion of
/// `mix(automatum | j << 8)`.
pub fn constellation(pictor: &mut Pictor, sky: Rect, automatum: u8, lum: u8, stratum: u8) {
    let (w, h) = (i64::from(sky.w), i64::from(sky.h));
    for j in 0u64..12 {
        let key = mix(u64::from(automatum) | (j << 8));
        let fx = i64::from(scatter64(key, 16) as u16);
        let fy = i64::from(scatter64(key >> 32, 16) as u16);
        let px = i64::from(sky.x).wrapping_add(fx.wrapping_mul(w).wrapping_shr(16));
        let py = i64::from(sky.y).wrapping_add(fy.wrapping_mul(h).wrapping_shr(16));
        if w > 0 && h > 0 {
            pictor.plot(px as i32, py as i32, lum, stratum, LABEL_NONE, Modo::Super);
        }
    }
}

/// Tone of the lit part of the moon where the ordered dither does not reach the moon's tone
/// (REQ-L5-08).
pub const PUNCTA_LUNAE: u8 = 9;

/// Draws the moon of radius `r` centered at `(cx, cy)` in the sky's class 2: nothing at new moon,
/// a full disc at 128, and in between the disc minus a shadow disc displaced by the illuminated
/// fraction, to the left while waxing and to the right while waning in the northern hemisphere,
/// mirrored in the southern one (`australis`). The lit part is dotted: a pixel takes `lum` where
/// the Bayer matrix lights it at `lum`, and [`PUNCTA_LUNAE`] elsewhere (REQ-L5-08). Only the rows
/// and columns that can reach the frame are visited.
pub fn moon(pictor: &mut Pictor, cx: i32, cy: i32, r: i32, luna: u8, lum: u8, australis: bool) {
    if luna == 0 {
        return;
    }
    let r = r.saturating_abs().min(4096);
    let waxing = luna <= 128;
    let fraction = if waxing {
        i32::from(luna)
    } else {
        i32::from(255u8.wrapping_sub(luna).wrapping_add(1))
    };
    let offset = r.wrapping_mul(2).wrapping_mul(fraction).wrapping_shr(7);
    let shadow_x = if waxing == australis {
        offset
    } else {
        offset.wrapping_neg()
    };
    let r2 = (r as u32).saturating_mul(r as u32);
    let (cx, cy, radius) = (i64::from(cx), i64::from(cy), i64::from(r));
    let (top, bottom) = pictor.clip_rows(cy.wrapping_sub(radius), cy.wrapping_add(radius));
    for y in top..=bottom {
        let dy = y.wrapping_sub(cy);
        let dy2 = dy.wrapping_mul(dy) as u32;
        let span = i64::from(isqrt(r2.saturating_sub(dy2)));
        let (left, right) = pictor.clip_columns(cx.wrapping_sub(span), cx.wrapping_add(span));
        for x in left..=right {
            let sx = x.wrapping_sub(cx).wrapping_sub(i64::from(shadow_x));
            let shadow2 = sx.wrapping_mul(sx).wrapping_add(dy.wrapping_mul(dy));
            if shadow2 > i64::from(r2) {
                let tone = if bayer(x as u32, y as u32, lum) {
                    lum
                } else {
                    PUNCTA_LUNAE
                };
                pictor.plot(x as i32, y as i32, tone, 2, LABEL_NONE, Modo::Super);
            }
        }
    }
}

/// The ray of a rare event: a line of the brightest tone from a point of the sky to the figure.
pub fn ray(pictor: &mut Pictor, from: (i32, i32), to: (i32, i32), lum: u8) {
    pictor.line(from.0, from.1, to.0, to.1, lum, 2, 1, Modo::Super);
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::arithmetic_side_effects,
        clippy::indexing_slicing,
        clippy::unwrap_used
    )]
    use super::*;
    use crate::hospes::Tabula;
    use crate::signum::pixel::{VOID, lum_of};
    use crate::signum::raster::Pictor;
    use crate::spatium::Rect;

    /// Six lines from the six bits, bit 0 at the bottom; 1 = unbroken (REQ-L5-07).
    #[test]
    fn hexagram_lines() {
        assert_eq!(
            super::hexagram_lines(0b10_1010),
            [false, true, false, true, false, true]
        );
        assert_eq!(super::hexagram_lines(0b11_1111), [true; 6]);
        assert_eq!(super::hexagram_lines(0), [false; 6]);
        assert_eq!(
            super::hexagram_lines(0b100_0001),
            [true, false, false, false, false, false],
            "bit 6 ignored"
        );
        let mut buf = [VOID; 16 * 16];
        let mut tab = Tabula::new(16, 16, &mut buf).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        hexagram(&mut pictor, 2, 2, 1, 0b00_0001, 9, 2);
        let row = |y: usize| (0..16).filter(|&x| lum_of(buf[x + y * 16]) == 9).count();
        assert_eq!(row(12), 12, "bottom line unbroken over 12 px");
        assert_eq!(row(10), 10, "second line broken: a gap of 2 px");
    }

    /// The harmony of the burden shapes the sky (REQ-L5-12): up to 200 the region of the
    /// composition; above, its right edge is ⌊W·(φ⁻¹ ⊕ 0x1337)⌋ by the composition's formula.
    #[test]
    fn harmony_shapes_sky() {
        use crate::signum::margo::harmony;
        use crate::spatium::compositio;
        for (w, h) in [(320u16, 180u16), (1920, 1080), (272, 60)] {
            let comp = compositio(w, h, 6);
            assert_eq!(sky_region(&comp, w, 0), comp.sky);
            assert_eq!(sky_region(&comp, w, 200), comp.sky);
            let corrupted = sky_region(&comp, w, 201);
            let right = ((u64::from(w) * harmony(201).0 as u64) >> 16) as u16;
            assert_eq!(
                corrupted,
                Rect {
                    w: right.saturating_sub(comp.sky.x),
                    ..comp.sky
                }
            );
            assert_eq!(sky_region(&comp, w, 255), corrupted);
            if w >= 320 {
                assert_ne!(corrupted.w, comp.sky.w, "{w}×{h}");
            }
        }
    }

    /// The same automaton scatters the same constellation; a different one differs.
    #[test]
    fn constellation_deterministic() {
        let sky = Rect {
            x: 4,
            y: 4,
            w: 56,
            h: 24,
        };
        let render = |automatum: u8| {
            let mut buf = [VOID; 64 * 32];
            let mut tab = Tabula::new(64, 32, &mut buf).unwrap();
            let mut pictor = Pictor::flat(&mut tab);
            constellation(&mut pictor, sky, automatum, 7, 1);
            buf
        };
        assert_eq!(render(0x10), render(0x10));
        assert_ne!(render(0x10), render(0x11));
        let buf = render(0x10);
        let lit = (0..64 * 32).filter(|&i| lum_of(buf[i]) == 7).count();
        assert!(lit >= 12 && lit <= 12 * 5, "lit {lit}");
        for i in 0..64 * 32 {
            if lum_of(buf[i]) == 7 {
                let (x, y) = (i % 64, i / 64);
                assert!(
                    (4..60).contains(&x) && (4..28).contains(&y),
                    "outside the sky at {x},{y}"
                );
            }
        }
    }

    /// New moon draws nothing; full moon fills the disc; half moon lights about half; the lit
    /// part is dotted, the cells of the dither in the moon's tone and the others in tone 9
    /// (REQ-L5-08).
    #[test]
    fn moon_phase_fill() {
        let render = |luna: u8| {
            let mut buf = [VOID; 32 * 32];
            let mut tab = Tabula::new(32, 32, &mut buf).unwrap();
            let mut pictor = Pictor::flat(&mut tab);
            moon(&mut pictor, 16, 16, 8, luna, 12, false);
            let tone = |t: u8| (0..32 * 32).filter(|&i| lum_of(buf[i]) == t).count();
            (tone(12), tone(PUNCTA_LUNAE))
        };
        assert_eq!(render(0), (0, 0));
        let (bright, dim) = render(128);
        let full = bright + dim;
        assert!((160..=230).contains(&full), "full {full}");
        assert!(dim > 0, "the lit part is dotted");
        assert!(
            bright * 16 >= full * 10 && bright * 16 <= full * 14,
            "twelve in sixteen cells bright: {bright} of {full}"
        );
        let (b, d) = render(64);
        assert!(
            b + d > full / 4 && b + d < full * 3 / 4,
            "half {} of {full}",
            b + d
        );
        let (b, d) = render(192);
        assert!(
            b + d > full / 4 && b + d < full * 3 / 4,
            "waning {} of {full}",
            b + d
        );
    }

    /// The moon of the southern hemisphere is the mirror image of the northern one about its
    /// vertical axis: the waxing moon is lit on the right in the north and on the left in the
    /// south (REQ-L5-08).
    #[test]
    fn moon_hemisphere_mirror() {
        let lit = |luna: u8, australis: bool| {
            let mut buf = [VOID; 33 * 33];
            let mut tab = Tabula::new(33, 33, &mut buf).unwrap();
            let mut pictor = Pictor::flat(&mut tab);
            moon(&mut pictor, 16, 16, 10, luna, 12, australis);
            buf.map(|p| p != VOID)
        };
        for luna in [20u8, 64, 100, 160, 200, 240] {
            let (north, south) = (lit(luna, false), lit(luna, true));
            for y in 0..33 {
                for x in 0..33 {
                    assert_eq!(
                        north[x + y * 33],
                        south[32 - x + y * 33],
                        "luna {luna} at {x},{y}"
                    );
                }
            }
            assert_ne!(north, south, "luna {luna}");
        }
        let side = |m: &[bool; 33 * 33], right: bool| {
            (0..33 * 33)
                .filter(|&i| m[i] && if right { i % 33 > 16 } else { i % 33 < 16 })
                .count()
        };
        let (north, south) = (lit(64, false), lit(64, true));
        assert!(
            side(&north, true) > side(&north, false),
            "waxing north lit on the right"
        );
        assert!(
            side(&south, false) > side(&south, true),
            "waxing south lit on the left"
        );
    }
}
