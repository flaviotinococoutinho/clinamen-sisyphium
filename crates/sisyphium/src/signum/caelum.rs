//! The sky: the constellation scattered from the automaton, the hexagram of serenity, the moon
//! by its phase and the ray of a rare event (REQ-L5-07, REQ-L5-08, REQ-L6-03).

use crate::natura::mix;
use crate::signum::pixel::LABEL_NONE;
use crate::signum::raster::{Modo, Pictor};
use crate::spatium::{Rect, isqrt, scatter64};

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

/// Draws the moon of radius `r` centered at `(cx, cy)`: nothing at new moon, a full disc at
/// 128, and in between the disc minus a shadow disc displaced by the illuminated fraction, to
/// the left while waxing and to the right while waning (REQ-L5-08).
pub fn moon(pictor: &mut Pictor, cx: i32, cy: i32, r: i32, luna: u8, lum: u8) {
    if luna == 0 {
        return;
    }
    let r = r.abs().min(4096);
    let waxing = luna <= 128;
    let fraction = if waxing {
        i32::from(luna)
    } else {
        i32::from(255u8.wrapping_sub(luna).wrapping_add(1))
    };
    let offset = r.wrapping_mul(2).wrapping_mul(fraction).wrapping_shr(7);
    let shadow_x = if waxing {
        offset.wrapping_neg()
    } else {
        offset
    };
    let r2 = (r as u32).saturating_mul(r as u32);
    let mut dy = r.wrapping_neg();
    while dy <= r {
        let dy2 = (dy as i64).wrapping_mul(dy as i64) as u32;
        let span = isqrt(r2.saturating_sub(dy2)) as i32;
        let mut dx = span.wrapping_neg();
        while dx <= span {
            let sx = dx.wrapping_sub(shadow_x);
            let shadow2 = (sx as i64)
                .wrapping_mul(sx as i64)
                .wrapping_add((dy as i64).wrapping_mul(dy as i64));
            if shadow2 > i64::from(r2) {
                pictor.plot(
                    cx.wrapping_add(dx),
                    cy.wrapping_add(dy),
                    lum,
                    2,
                    LABEL_NONE,
                    Modo::Super,
                );
            }
            dx = dx.wrapping_add(1);
        }
        dy = dy.wrapping_add(1);
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
    fn hexagram_lines_from_bits() {
        assert_eq!(
            hexagram_lines(0b10_1010),
            [false, true, false, true, false, true]
        );
        assert_eq!(hexagram_lines(0b11_1111), [true; 6]);
        assert_eq!(hexagram_lines(0), [false; 6]);
        assert_eq!(
            hexagram_lines(0b100_0001),
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

    /// New moon draws nothing; full moon fills the disc; half moon lights about half (REQ-L5-08).
    #[test]
    fn moon_phase_fill() {
        let render = |luna: u8| {
            let mut buf = [VOID; 32 * 32];
            let mut tab = Tabula::new(32, 32, &mut buf).unwrap();
            let mut pictor = Pictor::flat(&mut tab);
            moon(&mut pictor, 16, 16, 8, luna, 12);
            (0..32 * 32).filter(|&i| lum_of(buf[i]) == 12).count()
        };
        assert_eq!(render(0), 0);
        let full = render(128);
        assert!((160..=230).contains(&full), "full {full}");
        let half = render(64);
        assert!(
            half > full / 4 && half < full * 3 / 4,
            "half {half} of {full}"
        );
        let waning = render(192);
        assert!(
            waning > full / 4 && waning < full * 3 / 4,
            "waning {waning} of {full}"
        );
    }
}
