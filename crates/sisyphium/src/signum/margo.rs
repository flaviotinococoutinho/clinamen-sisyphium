//! The organic frame: whiplash curves of three harmonics in fixed point, wilting with the burden
//! and with scarce energy, thickened by agitation (REQ-L5-05); the corrupted harmony constant
//! (REQ-L5-12).

use crate::signum::raster::{Modo, Pictor};
use crate::spatium::{Compositio, Fx, PHI_INV, sin};

/// The harmony constant of the sky: φ⁻¹, or φ⁻¹ ⊕ 0x1337 above a burden of 200 (REQ-L5-12).
#[must_use]
pub const fn harmony(onus: u8) -> Fx {
    if onus > 200 {
        Fx(PHI_INV.0 ^ 0x1337)
    } else {
        PHI_INV
    }
}

/// Amplitudes of the three harmonics in Q16.16: 0.55, 0.30, 0.15.
const AMPLITUDES: [Fx; 3] = [Fx(36_044), Fx(19_660), Fx(9_830)];

/// A point of the whiplash curve at parameter `t` (0 at the bottom corner, 255 at the top):
/// `(swing, height)` with the swing in −1..=1 as the sum of three harmonics phased by the seed,
/// wilted by the burden (×(1024 − onus)/1024) and by scarce energy (×3/4 below 77), and the
/// height in 0..=1 (REQ-L5-05). The right side carries a quarter-turn phase per harmonic.
#[must_use]
pub fn whiplash(t: u8, onus: u8, energy: u8, semen: u16, right: bool) -> (Fx, Fx) {
    let angle = u32::from(t).wrapping_mul(256);
    let mut swing = Fx::ZERO;
    for (k, amplitude) in (1u32..).zip(AMPLITUDES) {
        let phase = u32::from(semen.rotate_left(k.wrapping_mul(5)));
        let side = if right { k.wrapping_mul(16_384) } else { 0 };
        let a = angle.wrapping_mul(k).wrapping_add(phase).wrapping_add(side) as u16;
        swing = swing.add(amplitude.mul(sin(a)));
    }
    let wilt = Fx((1024i32.wrapping_sub(i32::from(onus))) << 6);
    swing = swing.mul(wilt);
    if energy < 77 {
        swing = swing.mul(Fx::from_ratio(3, 4));
    }
    (swing, Fx::from_ratio(i32::from(t), 255))
}

/// Draws the organic frame: one whiplash curve per side, growing from the bottom corners to the
/// top margin, with the line weight `linea`·`s`.
#[allow(clippy::too_many_arguments)]
pub fn draw_frame(
    pictor: &mut Pictor,
    comp: &Compositio,
    onus: u8,
    energy: u8,
    semen: u16,
    linea: u8,
    lum: u8,
    stratum: u8,
) {
    let width = i32::from(pictor.width());
    let height = i32::from(pictor.height());
    let margin = i32::from(comp.m).max(2);
    let half = margin.wrapping_shr(1);
    let thick = linea.max(1).saturating_mul(comp.s.max(1) as u8);
    for right in [false, true] {
        let mut prev: Option<(i32, i32)> = None;
        let mut t = 0u16;
        while t <= 255 {
            let (swing, h) = whiplash(t as u8, onus, energy, semen, right);
            let dx = swing.mul(Fx::from_int(half.wrapping_sub(1))).to_int();
            let x = if right {
                width.wrapping_sub(half).wrapping_sub(dx)
            } else {
                half.wrapping_add(dx)
            };
            let y = height.wrapping_sub(margin).wrapping_sub(
                h.mul(Fx::from_int(height.wrapping_sub(margin.wrapping_mul(2))))
                    .to_int(),
            );
            if let Some((px, py)) = prev {
                pictor.line(px, py, x, y, lum, stratum, thick, Modo::Super);
            }
            prev = Some((x, y));
            t = t.wrapping_add(3);
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::arithmetic_side_effects)]
    use super::*;
    use crate::spatium::{Fx, PHI_INV};

    /// Amplitude falls with the burden and with scarce energy (REQ-L5-05).
    #[test]
    fn whiplash_wilt() {
        let swing = |onus: u8, energy: u8| {
            (0..=255u8)
                .map(|t| whiplash(t, onus, energy, 0xACE1, false).0.abs())
                .max()
                .unwrap_or(Fx::ZERO)
        };
        let light = swing(0, 255);
        let heavy = swing(255, 255);
        let tired = swing(0, 50);
        assert!(heavy < light, "burden wilts: {heavy:?} < {light:?}");
        assert!(tired < light, "scarcity wilts: {tired:?} < {light:?}");
        assert!(light.0 <= Fx::ONE.0, "normalized amplitude");
        assert!(light.0 > Fx::ONE.0 / 2, "visible swing");
        let (left_x, _) = whiplash(100, 0, 255, 0xACE1, false);
        let (right_x, _) = whiplash(100, 0, 255, 0xACE1, true);
        assert_ne!(left_x, right_x, "the two sides are asymmetric");
        assert_eq!(whiplash(0, 0, 255, 1, false).1, Fx::ZERO);
        assert_eq!(whiplash(255, 0, 255, 1, false).1, Fx::ONE);
    }

    /// φ⁻¹ ⊕ 0x1337 above a burden of 200 (REQ-L5-12).
    #[test]
    fn corrupted_harmony_constant() {
        assert_eq!(harmony(200), PHI_INV);
        assert_eq!(harmony(201), Fx(PHI_INV.0 ^ 0x1337));
        assert_eq!(harmony(255), Fx(0x8D00));
    }
}
