//! `corpus`: the body (level L3). The gait, the posture as a pure function of the state, the
//! procedural figure drawn from a skeleton, and the boulder as the direct reading of the
//! `Verbum` (REQ-L3-01..05).

use crate::signum::pixel::LABEL_NONE;
use crate::signum::raster::{Modo, Pictor};
use crate::spatium::{isqrt, sin};
use crate::verbum::{Modus, Verbum};

/// Posture of the figure (REQ-L3-02).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Postura {
    /// Upright, leaning into the boulder.
    Erecta,
    /// Bent under the burden.
    Curva,
    /// Kneeling, exhausted.
    Genua,
    /// Standing still, arms down.
    Contemplans,
    /// Lying at the foot of the slope, the head uphill against the boulder.
    Dormiens,
    /// Turned toward the observer.
    Spectans,
}

/// Amplitude of the first sine of the gait, in Q16.16 frames per step per minute of base
/// cadence: 0.03·(4/60 000)·7 500/(2π)·65 536 = 156.46, rounded (REQ-L3-01). The sine of
/// period 7 500·2ᵏ ms carries 2ᵏ times this amplitude, so each of the four sines adds ±3 % to
/// the base rate.
pub const GAIT_SWAY: i64 = 156;

/// Frames of the gait in Q16.16 within the minute (REQ-L3-01): with t = `epoch_ms` mod 60 000
/// and c = `cadentia`, 4·c·t/60 000 plus Σₖ Aₖ·sin(2π·t/Pₖ + k·π/2) for k = 0..=3, with
/// Pₖ = 7 500·2ᵏ ms and Aₖ = 2ᵏ·c·[`GAIT_SWAY`] (Bhaskara's sine). The rate stays within ±12 %
/// of the base, so the count never decreases; every period divides the minute, so at the turn
/// of the minute the count drops by exactly 4·c, a whole number of steps of four frames.
#[must_use]
pub const fn gait_frames(epoch_ms: i64, cadentia: u8) -> i64 {
    let t = epoch_ms.rem_euclid(60_000);
    let c = cadentia as i64;
    let mut frames = c
        .wrapping_mul(4)
        .wrapping_mul(t)
        .wrapping_mul(65_536)
        .wrapping_div(60_000);
    let mut k = 0u32;
    while k < 4 {
        let period = 7_500i64.wrapping_shl(k);
        let within = match t.checked_rem_euclid(period) {
            Some(r) => r,
            None => 0,
        };
        let turn = match within.wrapping_mul(65_536).checked_div(period) {
            Some(q) => q,
            None => 0,
        }
        .wrapping_add((k as i64).wrapping_mul(16_384));
        let amplitude = c.wrapping_shl(k).wrapping_mul(GAIT_SWAY);
        let sine = sin(turn as u16).0 as i64;
        frames = frames.wrapping_add(amplitude.wrapping_mul(sine).wrapping_shr(16));
        k = k.wrapping_add(1);
    }
    frames
}

/// Phase of the gait, 0..=3: the whole frames of [`gait_frames`] modulo 4 (REQ-L3-01); a still
/// world (`cadentia` 0) stays at frame 0.
#[must_use]
pub const fn gait_phase(epoch_ms: i64, cadentia: u8) -> u8 {
    (gait_frames(epoch_ms, cadentia).wrapping_shr(16) & 3) as u8
}

/// The posture as a pure function of the state and of the moment (REQ-L3-02), in this
/// precedence: facing the observer, asleep, contemplating (or Quies), kneeling below 48 of
/// effort, bent from 128 of weight, upright otherwise.
#[must_use]
pub const fn postura(verbum: Verbum, contemplating: bool, asleep: bool, leaving: bool) -> Postura {
    if leaving {
        Postura::Spectans
    } else if asleep {
        Postura::Dormiens
    } else if contemplating || matches!(verbum.modus(), Modus::Quies) {
        Postura::Contemplans
    } else if verbum.conatus() < 48 {
        Postura::Genua
    } else if verbum.onus() >= 128 {
        Postura::Curva
    } else {
        Postura::Erecta
    }
}

/// The 8×8 texture of the boulder: the word itself, rolled by eight bits per step of ascent
/// (REQ-L3-03).
#[must_use]
pub const fn boulder_texture(verbum: Verbum) -> u64 {
    verbum
        .bits()
        .rotate_left(((verbum.ascensus() & 7) as u32) << 3)
}

/// A chip of the boulder: under `seism` ≥ 240, when the generator's six low bits are zero, one
/// bit in 48..=61 chosen by the generator is flipped; the seed is never touched (REQ-L3-04).
#[must_use]
pub const fn chip(verbum: Verbum, seism: u8) -> Option<Verbum> {
    if seism < 240 || verbum.semen() & 0x3F != 0 {
        return None;
    }
    let bit = 48u32.wrapping_add(((verbum.semen() >> 6) % 14) as u32);
    Some(Verbum::new(verbum.bits() ^ (1u64 << bit)))
}

/// The figure to draw: its foot on the slope, height, integer scale, posture, gait frame and
/// line weight.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Figura {
    /// Ground point under the figure, in frame pixels.
    pub foot: (i32, i32),
    /// Height in pixels.
    pub h: u16,
    /// Integer scale of the frame.
    pub s: u16,
    /// Posture.
    pub postura: Postura,
    /// Gait frame 0..=3.
    pub phase: u8,
    /// Line weight in units of the scale.
    pub linea: u8,
}

/// Joint positions in units of h/24, y upward from the foot, x uphill: hip, shoulder, head
/// top-left, back foot, front foot, back hand, front hand.
struct Ossa {
    /// Hip.
    hip: (i32, i32),
    /// Shoulder.
    shoulder: (i32, i32),
    /// Top-left corner of the head block.
    head: (i32, i32),
    /// Feet.
    feet: [(i32, i32); 2],
    /// Hands.
    hands: [(i32, i32); 2],
}

/// The skeleton of a posture; the gait frame swings the feet of walking postures.
const fn ossa(postura: Postura, phase: u8) -> Ossa {
    let stride: i32 = match phase & 3 {
        1 => 2,
        3 => -2,
        _ => 0,
    };
    match postura {
        Postura::Erecta => Ossa {
            hip: (0, 10),
            shoulder: (3, 19),
            head: (2, 23),
            feet: [
                ((-3i32).wrapping_sub(stride), 0),
                (3i32.wrapping_add(stride), 0),
            ],
            hands: [(9, 14), (9, 12)],
        },
        Postura::Curva => Ossa {
            hip: (0, 9),
            shoulder: (7, 14),
            head: (7, 18),
            feet: [
                ((-3i32).wrapping_sub(stride), 0),
                (3i32.wrapping_add(stride), 0),
            ],
            hands: [(12, 10), (12, 9)],
        },
        Postura::Genua => Ossa {
            hip: (0, 5),
            shoulder: (4, 13),
            head: (3, 17),
            feet: [(-5, 0), (4, 0)],
            hands: [(9, 9), (9, 8)],
        },
        Postura::Contemplans => Ossa {
            hip: (0, 10),
            shoulder: (0, 19),
            head: (-2, 23),
            feet: [(-2, 0), (2, 0)],
            hands: [(-3, 11), (3, 11)],
        },
        Postura::Dormiens => Ossa {
            hip: (0, 2),
            shoulder: (8, 3),
            head: (9, 5),
            feet: [(-9, 0), (-10, 1)],
            hands: [(6, 1), (4, 1)],
        },
        Postura::Spectans => Ossa {
            hip: (0, 10),
            shoulder: (0, 19),
            head: (-2, 23),
            feet: [(-3, 0), (3, 0)],
            hands: [(-5, 11), (5, 11)],
        },
    }
}

/// A skeleton coordinate of `k` units of h/24 in pixels: round(k·h/24), half away from zero.
const fn joint(k: i32, h: u16) -> i32 {
    let n = (k as i64).wrapping_mul(h as i64);
    let q = (n.unsigned_abs().wrapping_add(12) / 24) as i32;
    if n < 0 { q.wrapping_neg() } else { q }
}

/// Draws the figure from its skeleton: joints at round(k·h/24) pixels from the foot, lines of
/// `linea`·`s` pixels and a solid head block of side max(1, round(4·h/24)) without features
/// (REQ-L2-11, REQ-L3-05). Returns the head block `(x, y, size)` in frame pixels.
pub fn draw_figure(pictor: &mut Pictor, figura: &Figura, lum: u8, stratum: u8) -> (i32, i32, u16) {
    let thick = figura.linea.max(1).saturating_mul(figura.s.max(1) as u8);
    let bones = ossa(figura.postura, figura.phase);
    let at = |p: (i32, i32)| {
        (
            figura.foot.0.wrapping_add(joint(p.0, figura.h)),
            figura.foot.1.wrapping_sub(joint(p.1, figura.h)),
        )
    };
    let hip = at(bones.hip);
    let shoulder = at(bones.shoulder);
    pictor.line(
        hip.0,
        hip.1,
        shoulder.0,
        shoulder.1,
        lum,
        stratum,
        thick,
        Modo::Super,
    );
    for foot in bones.feet {
        let f = at(foot);
        pictor.line(hip.0, hip.1, f.0, f.1, lum, stratum, thick, Modo::Super);
    }
    for hand in bones.hands {
        let h = at(hand);
        pictor.line(
            shoulder.0,
            shoulder.1,
            h.0,
            h.1,
            lum,
            stratum,
            thick,
            Modo::Super,
        );
    }
    let size = joint(4, figura.h).max(1);
    let head = at(bones.head);
    pictor.fill(head.0, head.1, size, size, lum, stratum, false);
    (head.0, head.1, size as u16)
}

/// Draws the boulder: a disc of diameter `d` whose interior reads the 8×8 `texture` in blocks
/// of `scale` pixels (bit set → `lum_on`, clear → `lum_off`) and whose rim is `lum_edge`. The
/// rim is drawn after the interior, so that it stays the outline where a contraction lands
/// several logical pixels on one frame pixel (REQ-L2-06); without curvature every logical pixel
/// has its own frame pixel and the order does not change the frame. Each pass visits only the
/// rows and columns that can reach the frame.
#[allow(clippy::too_many_arguments)]
pub fn draw_boulder(
    pictor: &mut Pictor,
    center: (i32, i32),
    d: u16,
    scale: u16,
    texture: u64,
    lum_on: u8,
    lum_off: u8,
    lum_edge: u8,
) {
    let radius = i64::from(i32::from(d).wrapping_shr(1).max(1));
    let scale = i64::from(scale.max(1));
    let r2 = (radius as u32).saturating_mul(radius as u32);
    let inner = radius.wrapping_sub(1).max(0) as u32;
    let inner2 = inner.saturating_mul(inner);
    let (cx, cy) = (i64::from(center.0), i64::from(center.1));
    for rim in [false, true] {
        let (top, bottom) = pictor.clip_rows(cy.wrapping_sub(radius), cy.wrapping_add(radius));
        for y in top..=bottom {
            let dy = y.wrapping_sub(cy);
            let dy2 = dy.wrapping_mul(dy) as u32;
            let span = i64::from(isqrt(r2.saturating_sub(dy2)));
            let (left, right) = pictor.clip_columns(cx.wrapping_sub(span), cx.wrapping_add(span));
            for x in left..=right {
                let dx = x.wrapping_sub(cx);
                let dist2 = dy2.saturating_add(dx.wrapping_mul(dx) as u32);
                if (dist2 > inner2) == rim {
                    let lum = if rim {
                        lum_edge
                    } else {
                        let col =
                            (dx.wrapping_add(radius).checked_div(scale).unwrap_or(0) & 7) as u32;
                        let row =
                            (dy.wrapping_add(radius).checked_div(scale).unwrap_or(0) & 7) as u32;
                        if (texture >> (row.wrapping_mul(8).wrapping_add(col))) & 1 == 1 {
                            lum_on
                        } else {
                            lum_off
                        }
                    };
                    pictor.plot(x as i32, y as i32, lum, 0, LABEL_NONE, Modo::Super);
                }
            }
        }
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
    use crate::hospes::Tabula;
    use crate::signum::pixel::{VOID, lum_of};
    use crate::signum::raster::Pictor;
    use crate::verbum::{Modus, Verbum};
    use std::vec::Vec;

    /// Noon of the 2026 June solstice at −03:00.
    const NOON: i64 = 1_782_054_000_000;

    /// Four frames per step at the base cadence (REQ-L3-01): across a minute the count grows by
    /// 4·c frames, a still world stays at frame 0, and every frame of the step appears.
    #[test]
    fn gait_cadence() {
        for c in [48u8, 64] {
            for minute in [0i64, NOON - NOON % 60_000, -120_000] {
                let span = gait_frames(minute + 59_999, c) - gait_frames(minute, c);
                let expected = 4 * i64::from(c) * 65_536;
                assert!(
                    (expected - 2 * 65_536..=expected).contains(&span),
                    "c {c}: a minute spans {span} of {expected}"
                );
            }
            let mut seen = [false; 4];
            for ms in (0..4000).step_by(50) {
                seen[usize::from(gait_phase(NOON + ms, c))] = true;
            }
            assert!(seen.iter().all(|&s| s), "c {c}: frames seen {seen:?}");
        }
        for ms in [0i64, 937, 5_000, NOON, -1] {
            assert_eq!(gait_phase(ms, 0), 0, "a still world has no gait");
        }
        let sway = 0.03 * (4.0 / 60_000.0) * 7_500.0 / core::f64::consts::TAU * 65_536.0;
        assert!(
            (GAIT_SWAY as f64 - sway).abs() <= 0.5,
            "{GAIT_SWAY} vs {sway}"
        );
    }

    /// The phase never steps back and never skips: ten minutes at 24 Hz advance 0 or 1 frame per
    /// sample, at 4 Hz 0..=2, exactly 4·c frames in sixty seconds, and the rate breathes: some
    /// window of two seconds runs at least 5 % above the base rate and some at least 5 % below
    /// (REQ-L3-01).
    #[test]
    fn gait_monotone() {
        for c in [48u8, 64] {
            let at = |n: i64, hz: i64| NOON + n * 1000 / hz;
            for (hz, allowed) in [(24i64, 0..=1u8), (4, 0..=2)] {
                for n in 0..600 * hz {
                    let step =
                        gait_phase(at(n + 1, hz), c).wrapping_sub(gait_phase(at(n, hz), c)) & 3;
                    assert!(
                        allowed.contains(&step),
                        "c {c} at {hz} Hz, sample {n}: {step}"
                    );
                }
            }
            let start = NOON + 12_345;
            let advance: u32 = (0..1440)
                .map(|n| {
                    let (a, b) = (start + n * 1000 / 24, start + (n + 1) * 1000 / 24);
                    u32::from(gait_phase(b, c).wrapping_sub(gait_phase(a, c)) & 3)
                })
                .sum();
            assert_eq!(advance, 4 * u32::from(c), "c {c}: frames in sixty seconds");
            let minute = NOON - NOON % 60_000;
            let base = 4.0 * f64::from(c) * 2_000.0 * 65_536.0 / 60_000.0;
            let rates: Vec<f64> = (0..58_000)
                .step_by(250)
                .map(|t| {
                    (gait_frames(minute + t + 2_000, c) - gait_frames(minute + t, c)) as f64 / base
                })
                .collect();
            assert!(rates.iter().any(|&r| r >= 1.05), "c {c}: never 5 % faster");
            assert!(rates.iter().any(|&r| r <= 0.95), "c {c}: never 5 % slower");
            assert!(
                rates.iter().all(|&r| r > 0.85 && r < 1.15),
                "c {c}: rate out of ±15 %"
            );
        }
    }

    /// Posture is a pure function of the state (REQ-L3-02).
    #[test]
    fn posture_table() {
        let v = |conatus: u8, onus: u8, modus: Modus| {
            Verbum::new(1)
                .with_conatus(conatus)
                .with_onus(onus)
                .with_modus(modus)
        };
        let vigil = Modus::Vigilia;
        assert_eq!(
            postura(v(200, 40, vigil), false, false, false),
            Postura::Erecta
        );
        assert_eq!(
            postura(v(200, 128, vigil), false, false, false),
            Postura::Curva
        );
        assert_eq!(
            postura(v(47, 40, vigil), false, false, false),
            Postura::Genua
        );
        assert_eq!(
            postura(v(47, 200, vigil), false, false, false),
            Postura::Genua
        );
        assert_eq!(
            postura(v(200, 40, Modus::Quies), false, false, false),
            Postura::Contemplans
        );
        assert_eq!(
            postura(v(200, 40, vigil), true, false, false),
            Postura::Contemplans
        );
        assert_eq!(
            postura(v(200, 40, vigil), false, true, false),
            Postura::Dormiens
        );
        assert_eq!(
            postura(v(200, 40, vigil), true, true, true),
            Postura::Spectans
        );
        assert_eq!(
            postura(v(10, 255, Modus::Quies), false, false, true),
            Postura::Spectans
        );
    }

    /// The boulder texture is the word rolled by eight bits per step of ascent (REQ-L3-03).
    #[test]
    fn boulder_is_verbum() {
        let v = Verbum::new(0x0123_4567_89AB_CDEF);
        assert_eq!(
            boulder_texture(v.with_ascensus(0)),
            v.with_ascensus(0).bits()
        );
        assert_eq!(
            boulder_texture(v.with_ascensus(1)),
            v.with_ascensus(1).bits().rotate_left(8)
        );
        assert_eq!(
            boulder_texture(v.with_ascensus(9)),
            v.with_ascensus(9).bits().rotate_left(8)
        );
        assert_eq!(
            boulder_texture(v.with_ascensus(8)),
            v.with_ascensus(8).bits()
        );
        assert_eq!(
            boulder_texture(v.with_ascensus(7)),
            v.with_ascensus(7).bits().rotate_left(56)
        );
    }

    /// A chip flips exactly one bit in 48..=61, only under strong seism at the generator's gate,
    /// and never touches the seed (REQ-L3-04).
    #[test]
    fn chip_flips_one_bit() {
        let gate = Verbum::new(0x40).with_automatum(0xA5).with_tranquillitas(5);
        assert_eq!(chip(gate, 239), None, "seism below 240");
        let closed = Verbum::new(0x41);
        assert_eq!(
            chip(closed, 255),
            None,
            "gate closed when semen & 0x3F != 0"
        );
        let flipped = chip(gate, 255).unwrap();
        let diff = flipped.bits() ^ gate.bits();
        assert_eq!(diff.count_ones(), 1);
        let bit = diff.trailing_zeros();
        assert!((48..=61).contains(&bit), "bit {bit}");
        assert_eq!(flipped.semen(), gate.semen());
        let mut bits_seen = 0u64;
        for k in 0..1024u16 {
            let v = Verbum::new(u64::from(k) << 6);
            if let Some(f) = chip(v, 255) {
                bits_seen |= f.bits() ^ v.bits();
            }
        }
        assert_eq!(
            bits_seen, 0x3FFF_0000_0000_0000,
            "every chippable bit is reachable"
        );
    }

    /// The figure fits the box [foot.x − h, foot.x + h] × [foot.y − h, foot.y] at every height in
    /// every posture and gait frame; upright, the top of the head stands round(23·h/24) above the
    /// foot within one pixel (REQ-L2-11).
    #[test]
    fn figure_fits_box() {
        for h in [4u16, 10, 24, 43, 58, 146] {
            for postura in [
                Postura::Erecta,
                Postura::Curva,
                Postura::Genua,
                Postura::Contemplans,
                Postura::Dormiens,
                Postura::Spectans,
            ] {
                for phase in 0..4u8 {
                    let mut buf = std::vec![VOID; 320 * 200];
                    let mut tab = Tabula::new(320, 200, &mut buf).unwrap();
                    let mut pictor = Pictor::flat(&mut tab);
                    let foot = (160i32, 180i32);
                    let f = Figura {
                        foot,
                        h,
                        s: 1,
                        postura,
                        phase,
                        linea: 1,
                    };
                    draw_figure(&mut pictor, &f, 14, 0);
                    let lit: std::vec::Vec<(i32, i32)> = (0..320 * 200)
                        .filter(|&i| lum_of(buf[i]) == 14)
                        .map(|i| ((i % 320) as i32, (i / 320) as i32))
                        .collect();
                    assert!(!lit.is_empty(), "h {h} {postura:?} {phase}");
                    let reach = i32::from(h);
                    for &(x, y) in &lit {
                        assert!(
                            (foot.0 - reach..=foot.0 + reach).contains(&x)
                                && (foot.1 - reach..=foot.1).contains(&y),
                            "h {h} {postura:?} {phase}: ({x}, {y}) outside the box"
                        );
                    }
                    if postura == Postura::Erecta {
                        let top = lit.iter().map(|&(_, y)| y).min().unwrap();
                        let rise = foot.1 - top;
                        let law = (23.0 * f64::from(h) / 24.0).round() as i32;
                        assert!((rise - law).abs() <= 1, "h {h}: height {rise} vs {law}");
                    }
                }
            }
        }
    }

    /// The face is a solid block of 4×4 pixels without features at the logical height of 24
    /// pixels, in every posture (REQ-L3-05).
    #[test]
    fn sprite_face_blank() {
        for postura in [
            Postura::Erecta,
            Postura::Curva,
            Postura::Genua,
            Postura::Contemplans,
            Postura::Dormiens,
            Postura::Spectans,
        ] {
            for phase in 0..4u8 {
                let mut buf = [VOID; 64 * 64];
                let mut tab = Tabula::new(64, 64, &mut buf).unwrap();
                let mut pictor = Pictor::flat(&mut tab);
                let f = Figura {
                    foot: (32, 56),
                    h: 24,
                    s: 1,
                    postura,
                    phase,
                    linea: 1,
                };
                let (hx, hy, hs) = draw_figure(&mut pictor, &f, 14, 0);
                assert_eq!(hs, 4, "{postura:?}: head is 4×4 at the logical height");
                for dy in 0..hs {
                    for dx in 0..hs {
                        let (x, y) = ((hx + i32::from(dx)) as usize, (hy + i32::from(dy)) as usize);
                        assert_eq!(
                            lum_of(buf[x + y * 64]),
                            14,
                            "{postura:?} head hole at {x},{y}"
                        );
                    }
                }
            }
        }
    }

    /// Asleep after the recognition, the figure lies with the head uphill, toward the boulder
    /// that stands ahead on the slope (`docs/07-narrativa.md` §6).
    #[test]
    fn dormiens_head_toward_boulder() {
        for h in [24u16, 58] {
            let mut buf = std::vec![VOID; 200 * 100];
            let mut tab = Tabula::new(200, 100, &mut buf).unwrap();
            let mut pictor = Pictor::flat(&mut tab);
            let f = Figura {
                foot: (100, 90),
                h,
                s: 1,
                postura: Postura::Dormiens,
                phase: 0,
                linea: 1,
            };
            let (hx, _, hs) = draw_figure(&mut pictor, &f, 14, 0);
            assert!(
                hx > 100,
                "h {h}: head at x {hx}, the boulder lies to the right"
            );
            assert!(
                hx + i32::from(hs) <= 100 + i32::from(h),
                "h {h}: head inside the box"
            );
        }
    }

    /// Under a contraction several logical pixels land on one frame pixel; the rim stays the
    /// outline of the boulder: at least nine in ten drawn pixels next to an undrawn one carry the
    /// rim tone, at five places of the frame (REQ-L2-06, REQ-L3-03).
    #[test]
    fn boulder_rim_survives_contraction() {
        for kappa in [-64i8, -48, -32, -16, 0] {
            for center in [(50, 14), (14, 50), (50, 50), (14, 14), (32, 10)] {
                let mut buf = [VOID; 64 * 64];
                let mut tab = Tabula::new(64, 64, &mut buf).unwrap();
                let mut pictor = Pictor::new(&mut tab, kappa);
                draw_boulder(&mut pictor, center, 15, 2, 0x00FF_00FF_00FF_00FF, 12, 4, 15);
                let drawn = |x: i32, y: i32| {
                    (0..64).contains(&x)
                        && (0..64).contains(&y)
                        && buf[(x + y * 64) as usize] != VOID
                };
                let (mut outline, mut rim) = (0, 0);
                for y in 0..64 {
                    for x in 0..64 {
                        let open = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                            .iter()
                            .any(|&(ex, ey)| !drawn(x + ex, y + ey));
                        if drawn(x, y) && open {
                            outline += 1;
                            rim += usize::from(lum_of(buf[(x + y * 64) as usize]) == 15);
                        }
                    }
                }
                assert!(outline >= 20, "κ {kappa} at {center:?}: outline {outline}");
                assert!(
                    rim * 10 >= outline * 9,
                    "κ {kappa} at {center:?}: rim {rim} of {outline}"
                );
            }
        }
    }

    /// The boulder draws its texture bits inside a disc of the given diameter.
    #[test]
    fn boulder_disc_and_texture() {
        let mut buf = [VOID; 32 * 32];
        let mut tab = Tabula::new(32, 32, &mut buf).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        draw_boulder(
            &mut pictor,
            (16, 16),
            15,
            2,
            0xFFFF_FFFF_0000_0000,
            12,
            4,
            15,
        );
        let lit = |l: u8| (0..32 * 32).filter(|&i| lum_of(buf[i]) == l).count();
        assert!(
            lit(12) > 40 && lit(4) > 40,
            "both halves present: {} {}",
            lit(12),
            lit(4)
        );
        assert!(lit(15) > 20, "outline");
        for i in 0..32 * 32 {
            let (x, y) = ((i % 32) as i32 - 16, (i / 32) as i32 - 16);
            if lum_of(buf[i]) != 1 {
                assert!(x * x + y * y <= 9 * 9, "outside the disc at {x},{y}");
            }
        }
    }
}
