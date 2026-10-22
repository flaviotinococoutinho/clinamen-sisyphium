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

/// Amplitudes of the three harmonics in Q16.16: ⌊0.55·2¹⁶⌋ = 36 044, ⌊0.30·2¹⁶⌋ = 19 660 and
/// ⌊0.15·2¹⁶⌋ = 9 830; decreasing with the harmonic and summing to 65 534 ≤ 2¹⁶, so that the swing
/// stays in −1..=1 (REQ-L5-05).
const AMPLITUDES: [Fx; 3] = [Fx(36_044), Fx(19_660), Fx(9_830)];

/// A point of the whiplash curve at parameter `t` (0 at the bottom corner, 255 at the top):
/// `(swing, height)` (REQ-L5-05). The swing, in −1..=1, is the sum of three sines (Bhaskara's)
/// with amplitudes 0.55, 0.30 and 0.15: harmonic k at k·`t`/256 of a turn plus the phase `semen`
/// rotated left by 5k bits, read in 1/65 536 of a turn, plus k quarter turns on the right side;
/// it wilts by (1024 − `onus`)/1024 and by 3/4 below 77 of energy. The height is `t`/255.
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
    #![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
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

    /// The swing is the sum of three sines in the horizontal axis, harmonic k at k·t/256 of a
    /// turn plus the phase `semen` rotated left by 5k bits and, on the right side, k quarter
    /// turns; it wilts by (1024 − onus)/1024 and by 3/4 below 77 of energy; the height rises
    /// linearly as t/255 (REQ-L5-05). A reference sum with the exact sine agrees within the
    /// error of Bhaskara's sine.
    #[test]
    fn whiplash_harmonics() {
        let amplitudes = [0.55, 0.30, 0.15];
        for semen in [0xACE1u16, 0x1234] {
            for right in [false, true] {
                for (onus, energy) in [(0u8, 255u8), (255, 255), (90, 50)] {
                    for t in [0u8, 64, 128, 191, 255] {
                        let mut swing = 0.0;
                        for (k, a) in (1u32..).zip(amplitudes) {
                            let phase = f64::from(semen.rotate_left(5 * k));
                            let side = if right { f64::from(k) * 16_384.0 } else { 0.0 };
                            let turn = f64::from(k) * f64::from(t) * 256.0 + phase + side;
                            swing += a * (turn / 65_536.0 * core::f64::consts::TAU).sin();
                        }
                        swing *= (1024.0 - f64::from(onus)) / 1024.0;
                        if energy < 77 {
                            swing *= 0.75;
                        }
                        let (x, y) = whiplash(t, onus, energy, semen, right);
                        let got = f64::from(x.0) / 65_536.0;
                        assert!(
                            (got - swing).abs() <= 0.003,
                            "semen {semen:#x} right {right} onus {onus} t {t}: {got} vs {swing}"
                        );
                        assert_eq!(y, Fx::from_ratio(i32::from(t), 255));
                    }
                }
            }
        }
    }

    /// The amplitudes are ⌊0.55·2¹⁶⌋, ⌊0.30·2¹⁶⌋ and ⌊0.15·2¹⁶⌋: decreasing, summing to at most
    /// one so that the swing never leaves −1..=1 (REQ-L5-05).
    #[test]
    fn whiplash_amplitudes() {
        assert_eq!(AMPLITUDES, [Fx(36_044), Fx(19_660), Fx(9_830)]);
        for (a, share) in AMPLITUDES.iter().zip([0.55, 0.30, 0.15]) {
            assert_eq!(f64::from(a.0), (share * 65_536.0f64).floor());
        }
        assert!(AMPLITUDES[0] > AMPLITUDES[1] && AMPLITUDES[1] > AMPLITUDES[2]);
        let sum: i32 = AMPLITUDES.iter().map(|a| a.0).sum();
        assert!(sum <= Fx::ONE.0 && sum >= Fx::ONE.0 - 3, "sum {sum}");
    }

    /// φ⁻¹ ⊕ 0x1337 above a burden of 200 (REQ-L5-12).
    #[test]
    fn corrupted_harmony_constant() {
        assert_eq!(harmony(200), PHI_INV);
        assert_eq!(harmony(201), Fx(PHI_INV.0 ^ 0x1337));
        assert_eq!(harmony(255), Fx(0x8D00));
    }
}
