//! `concentus`: the chord (level L5). One vector of tuning, computed before any drawing and read
//! by every channel: tone, contrast, line weight, cadence, text register, curvature and the sound
//! index. No channel reads the state directly to decide its form (REQ-L5-11).

use crate::arbitrium::visible_annales;
use crate::ephemeris::ephemeris;
use crate::hospes::{Facultates, Indicia, Lingua, Regimen};
use crate::mundus::Mundus;
use crate::spatium::isqrt;
use crate::verbum::Modus;

pub use crate::arbitrium::Registrum;

/// Family of the palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tonus {
    /// Slate and chalk: the default.
    Ardesia,
    /// Warm, oxidized: moral weight above 200.
    Sepia,
    /// Cold blue: the dream.
    Frigus,
    /// Gold in the brightest tone: affirmation.
    Aurum,
}

/// The chord: every channel of form reads only this (REQ-L5-11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Concentus {
    /// Palette family.
    pub tonus: Tonus,
    /// Tint strength 0..=64: each tone mixes `chroma`/256 of the tint target into its color, at most
    /// 64/256 (a quarter) with no serenity; serenity reduces it.
    pub chroma: u8,
    /// Minimum tone difference between figure and ground, 8..=15; rises as energy falls.
    pub contrastus: u8,
    /// Line weight in units of the integer scale: 1 serene, 2 agitated.
    pub linea: u8,
    /// Base cadence of the gait in steps per minute (REQ-L3-01): 64, 48 below 48 of effort, 0
    /// when the world ticks once a second or slower.
    pub cadentia: u8,
    /// Register of the visible text.
    pub registrum: Registrum,
    /// Curvature applied to the frame.
    pub curvatura: i8,
    /// Sound index: 2040 / isqrt(64 + onus), from 255 to 120; mass includes the boulder
    /// (`docs/05-estetica.md` §10).
    pub sonus: u8,
    /// Language of the text.
    pub lingua: Lingua,
    /// Astronomical festival day.
    pub festum: bool,
    /// Night.
    pub nocturnal: bool,
    /// Lunar phase.
    pub luna: u8,
    /// Southern hemisphere: the moon is drawn mirrored (REQ-L5-08).
    pub australis: bool,
    /// Circadian phase.
    pub phase: u8,
}

/// Computes the chord from the world, the host and the regimen (REQ-L5-11): a pure function.
#[must_use]
pub fn concentus(
    mundus: &Mundus,
    facultates: &Facultates,
    indicia: &Indicia,
    regimen: &Regimen,
) -> Concentus {
    let v = mundus.verbum;
    let m = mundus.memoria;
    let eph = ephemeris(
        indicia.epoch_ms,
        facultates.tz_offset_min,
        facultates.australis,
    );
    let tonus = match v.modus() {
        Modus::AmorFati => Tonus::Aurum,
        Modus::Somnium => Tonus::Frigus,
        _ if v.onus() > 200 => Tonus::Sepia,
        _ => Tonus::Ardesia,
    };
    let tranquillitas = u16::from(v.tranquillitas());
    let chroma = (63u16.saturating_sub(tranquillitas).wrapping_mul(64) / 63) as u8;
    let contrastus = if v.modus() == Modus::Quies {
        15
    } else {
        match indicia.energy {
            179..=255 => 8,
            77..=178 => 10,
            39..=76 => 12,
            _ => 15,
        }
    };
    let linea = if v.tranquillitas() < 32 { 2 } else { 1 };
    let cadentia = if regimen.tick_ms >= 1000 {
        0
    } else if v.conatus() < 48 {
        48
    } else {
        64
    };
    let rows = if facultates.cells {
        facultates.height
    } else {
        u16::MAX
    };
    let registrum = visible_annales(m, facultates.lingua, indicia.leaving, regimen, rows).registrum;
    Concentus {
        tonus,
        chroma,
        contrastus,
        linea,
        cadentia,
        registrum,
        curvatura: v.curvatura(),
        sonus: 2040u32
            .checked_div(isqrt(64u32.wrapping_add(u32::from(v.onus()))))
            .unwrap_or(255) as u8,
        lingua: facultates.lingua,
        festum: eph.festum,
        nocturnal: eph.nocturnal,
        luna: eph.luna,
        australis: facultates.australis,
        phase: eph.phase,
    }
}

/// Slate, the darkest tone.
const ARDESIA: [u8; 3] = [18, 20, 24];
/// Chalk, the brightest tone.
const CRETA: [u8; 3] = [240, 240, 245];
/// Gold, the brightest tone in affirmation.
const AURUM: [u8; 3] = [255, 214, 102];
/// Sepia tint target: HSL(37.5°, 8/19, 32/51), a warm, half-saturated light brown; each
/// channel rounded to a byte gives (200, 170, 120).
const SEPIA: [u8; 3] = [200, 170, 120];
/// Cold tint target: HSL(1 560/7 ≈ 222.86°, 7/10, 41/51), a light blue; each channel rounded
/// to a byte gives (170, 190, 240).
const FRIGUS: [u8; 3] = [170, 190, 240];

/// Linear mix of one channel: `a + (b − a)·k/256`.
const fn mix(a: u8, b: u8, k: u16) -> u8 {
    let a16 = a as i32;
    let d = (b as i32).wrapping_sub(a16);
    a16.wrapping_add(d.wrapping_mul(k as i32) / 256) as u8
}

/// The sixteen colors of a chord (REQ-L5-02): a slate-to-chalk ramp, tinted by the tone family
/// in proportion to `chroma`, gold in tone 15 for affirmation, and tone 15 raised by 8 % on a
/// festival day.
#[must_use]
pub fn palette(c: &Concentus) -> [[u8; 3]; 16] {
    let mut out = [[0u8; 3]; 16];
    let target = match c.tonus {
        Tonus::Sepia => Some(SEPIA),
        Tonus::Frigus => Some(FRIGUS),
        Tonus::Ardesia | Tonus::Aurum => None,
    };
    for (k, color) in (0u16..).zip(out.iter_mut()) {
        for (ch, value) in color.iter_mut().enumerate() {
            let (lo, hi) = (
                ARDESIA.get(ch).copied().unwrap_or(0),
                CRETA.get(ch).copied().unwrap_or(0),
            );
            let base = mix(lo, hi, k.wrapping_mul(256) / 15);
            *value = match target {
                Some(t) => mix(base, t.get(ch).copied().unwrap_or(0), u16::from(c.chroma)),
                None => base,
            };
        }
    }
    if let Some(top) = out.last_mut() {
        if c.tonus == Tonus::Aurum {
            *top = AURUM;
        }
        if c.festum {
            for value in top.iter_mut() {
                *value = (u16::from(*value).wrapping_mul(27) / 25).min(255) as u8;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
    use super::*;
    use crate::hospes::{Facultates, Indicia, Regimen, Stilus};
    use crate::mundus::Mundus;
    use crate::verbum::Modus;

    /// Noon of the 2026 June solstice at −03:00.
    const NOON: i64 = 1_782_054_000_000;

    /// A world of genesis(7) with the given burden, serenity and mode.
    fn world(onus: u8, tranquillitas: u8, modus: Modus) -> Mundus {
        let mut m = Mundus::genesis(7);
        m.verbum = m
            .verbum
            .with_onus(onus)
            .with_tranquillitas(tranquillitas)
            .with_modus(modus);
        m
    }

    /// The same inputs give the same chord (REQ-L5-11).
    #[test]
    fn concentus_pure() {
        let m = world(90, 20, Modus::Vigilia);
        let f = Facultates::default();
        let i = Indicia {
            epoch_ms: NOON,
            ..Indicia::default()
        };
        let r = Regimen::default();
        assert_eq!(concentus(&m, &f, &i, &r), concentus(&m, &f, &i, &r));
    }

    /// Regimina that differ only in stilus give the same chord (REQ-L5-11).
    #[test]
    fn concentus_invariant_under_stilus() {
        let m = world(210, 40, Modus::Somnium);
        let f = Facultates::default();
        let i = Indicia {
            epoch_ms: NOON,
            energy: 100,
            ..Indicia::default()
        };
        let base = Regimen::default();
        for stilus in [
            Stilus::Creta,
            Stilus::Musivum,
            Stilus::Littera,
            Stilus::Bicolor,
        ] {
            let r = Regimen { stilus, ..base };
            assert_eq!(concentus(&m, &f, &i, &r), concentus(&m, &f, &i, &base));
        }
    }

    /// Tone precedence and the palette laws (REQ-L5-02).
    #[test]
    fn palette_laws() {
        let f = Facultates::default();
        let i = Indicia {
            epoch_ms: NOON + 10 * 86_400_000,
            ..Indicia::default()
        };
        let r = Regimen::default();
        let plain = concentus(&world(32, 24, Modus::Vigilia), &f, &i, &r);
        assert_eq!(plain.tonus, Tonus::Ardesia);
        let p = palette(&plain);
        assert_eq!(p[0], [18, 20, 24]);
        assert_eq!(p[15], [240, 240, 245]);
        for k in 1..16 {
            assert!(
                p[k][0] >= p[k - 1][0] && p[k][2] >= p[k - 1][2],
                "monotone at {k}"
            );
        }
        let heavy = concentus(&world(220, 0, Modus::Vigilia), &f, &i, &r);
        assert_eq!(heavy.tonus, Tonus::Sepia);
        let ps = palette(&heavy);
        assert!(ps[8][0] > ps[8][2], "sepia is warm: {:?}", ps[8]);
        let serene = concentus(&world(220, 63, Modus::Vigilia), &f, &i, &r);
        let pq = palette(&serene);
        let warmth = |p: &[[u8; 3]; 16]| i16::from(p[8][0]) - i16::from(p[8][2]);
        assert!(warmth(&pq) < warmth(&ps), "serenity reduces chroma");
        let dream = concentus(&world(220, 0, Modus::Somnium), &f, &i, &r);
        assert_eq!(dream.tonus, Tonus::Frigus);
        let pd = palette(&dream);
        assert!(pd[8][2] > pd[8][0], "cold is blue: {:?}", pd[8]);
        let fate = concentus(&world(220, 0, Modus::AmorFati), &f, &i, &r);
        assert_eq!(fate.tonus, Tonus::Aurum);
        assert_eq!(palette(&fate)[15], [255, 214, 102]);
        let festive = concentus(
            &world(32, 24, Modus::Vigilia),
            &f,
            &Indicia {
                epoch_ms: NOON,
                ..i
            },
            &r,
        );
        assert!(festive.festum);
        assert_eq!(palette(&festive)[15], [255, 255, 255]);
        assert_eq!(
            palette(&concentus(
                &world(220, 0, Modus::AmorFati),
                &f,
                &Indicia {
                    epoch_ms: NOON,
                    ..i
                },
                &r
            ))[15],
            [255, 231, 110]
        );
    }

    /// The tint targets are the colors of their HSL derivation: sepia at hue 37.5°, saturation
    /// 8/19 and lightness 32/51; cold at hue 1 560/7 ≈ 222.86°, saturation 7/10 and lightness
    /// 41/51; each channel rounded to a byte (REQ-L5-02).
    #[test]
    fn tint_targets_from_hsl() {
        let rgb = |hue: f64, saturation: f64, lightness: f64| {
            let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
            let second = chroma * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
            let base = lightness - chroma / 2.0;
            let channels = match (hue / 60.0) as u32 {
                0 => [chroma, second, 0.0],
                1 => [second, chroma, 0.0],
                2 => [0.0, chroma, second],
                3 => [0.0, second, chroma],
                4 => [second, 0.0, chroma],
                _ => [chroma, 0.0, second],
            };
            channels.map(|v| ((v + base) * 255.0).round() as u8)
        };
        assert_eq!(rgb(37.5, 8.0 / 19.0, 32.0 / 51.0), SEPIA);
        assert_eq!(rgb(1560.0 / 7.0, 0.7, 41.0 / 51.0), FRIGUS);
    }

    /// Contrast rises as energy falls (REQ-L5-02; `05-estetica.md` §10).
    #[test]
    fn contrast_by_energy() {
        let f = Facultates::default();
        let r = Regimen::default();
        let m = world(32, 24, Modus::Vigilia);
        let at = |energy: u8| {
            concentus(
                &m,
                &f,
                &Indicia {
                    epoch_ms: NOON,
                    energy,
                    ..Indicia::default()
                },
                &r,
            )
            .contrastus
        };
        assert_eq!(
            (
                at(255),
                at(179),
                at(178),
                at(77),
                at(76),
                at(39),
                at(38),
                at(0)
            ),
            (8, 8, 10, 10, 12, 12, 15, 15)
        );
    }

    /// Thick lines under agitation, thin lines in serenity (REQ-L5-05).
    #[test]
    fn linea_by_tranquillitas() {
        let f = Facultates::default();
        let r = Regimen::default();
        let i = Indicia {
            epoch_ms: NOON,
            ..Indicia::default()
        };
        assert_eq!(
            concentus(&world(32, 31, Modus::Vigilia), &f, &i, &r).linea,
            2
        );
        assert_eq!(
            concentus(&world(32, 32, Modus::Vigilia), &f, &i, &r).linea,
            1
        );
    }

    /// The base cadence of the gait: 64 steps per minute, 48 below 48 of effort, 0 when the world
    /// ticks once a second or slower; constant in time (REQ-L3-01).
    #[test]
    fn cadentia_base() {
        let f = Facultates::default();
        let r = Regimen::default();
        let at = |m: &Mundus, epoch_ms: i64, regimen: &Regimen| {
            let i = Indicia {
                epoch_ms,
                ..Indicia::default()
            };
            concentus(m, &f, &i, regimen).cadentia
        };
        let vigil = world(32, 40, Modus::Vigilia);
        let mut tired = vigil;
        tired.verbum = tired.verbum.with_conatus(47);
        let still = Regimen { tick_ms: 1000, ..r };
        for k in 0..512i64 {
            let t = NOON + k * 125;
            assert_eq!(at(&vigil, t, &r), 64);
            assert_eq!(at(&tired, t, &r), 48);
            assert_eq!(at(&vigil, t, &still), 0);
        }
        tired.verbum = tired.verbum.with_conatus(48);
        assert_eq!(at(&tired, NOON, &r), 64);
    }

    /// The register follows the effective last line, including the limits of the medium
    /// (REQ-L4-04, REQ-L5-11).
    #[test]
    fn registrum_by_annales() {
        let f = Facultates::default();
        let r = Regimen::default();
        let i = Indicia {
            epoch_ms: NOON,
            ..Indicia::default()
        };
        let at = |gradus: u16| {
            let mut m = world(32, 24, Modus::Vigilia);
            m.memoria = m.memoria.with_gradus(gradus).with_ramus(0);
            concentus(&m, &f, &i, &r).registrum
        };
        for (gradus, register) in [
            (0, Registrum::Intimum),
            (24, Registrum::Intimum),
            (72, Registrum::Vivum),
            (144, Registrum::Civile),
            (240, Registrum::Epochale),
            (360, Registrum::Abstractum),
        ] {
            assert_eq!(at(gradus), register);
        }
        for lingua in [Lingua::En, Lingua::Pt] {
            for (eventus, expected) in [
                (
                    3,
                    [Registrum::Intimum, Registrum::Vivum, Registrum::Abstractum],
                ),
                (11, [Registrum::Abstractum; 3]),
                (
                    12,
                    [
                        Registrum::Intimum,
                        Registrum::Abstractum,
                        Registrum::Abstractum,
                    ],
                ),
                (13, [Registrum::Vivum; 3]),
                (14, [Registrum::Abstractum; 3]),
                (15, [Registrum::Epochale; 3]),
            ] {
                let f = Facultates { lingua, ..f };
                let mut m = world(32, 24, Modus::Vigilia);
                m.memoria = m
                    .memoria
                    .with_eventus(eventus)
                    .with_ramus(1)
                    .with_gradus(400);
                for (lines, register) in [1, 2, 6].into_iter().zip(expected) {
                    let r = Regimen { lines, ..r };
                    assert_eq!(concentus(&m, &f, &i, &r).registrum, register);
                    let off = Regimen { layers: 0, ..r };
                    assert_eq!(concentus(&m, &f, &i, &off).registrum, Registrum::Intimum);
                }
                let f = Facultates {
                    cells: true,
                    height: 1,
                    ..f
                };
                assert_eq!(concentus(&m, &f, &i, &r).registrum, expected[0]);
            }
        }
        let mut m = world(32, 24, Modus::Vigilia);
        m.memoria = m.memoria.with_eventus(12).with_gradus(30);
        assert_eq!(concentus(&m, &f, &i, &r).registrum, Registrum::Intimum);
        m.memoria = m.memoria.with_gradus(80);
        assert_eq!(concentus(&m, &f, &i, &r).registrum, Registrum::Abstractum);
        m.memoria = m.memoria.with_eventus(3).with_ramus(3);
        let leave = Indicia { leaving: true, ..i };
        assert_eq!(
            concentus(&m, &f, &leave, &r).registrum,
            crate::arbitrium::annales(m.memoria, f.lingua, true).registrum
        );
    }

    /// The sound index falls with the integer square root of onus + 64
    /// (`docs/05-estetica.md` §10).
    #[test]
    fn sonus_by_mass() {
        let at = |onus| {
            concentus(
                &world(onus, 24, Modus::Vigilia),
                &Facultates::default(),
                &Indicia::default(),
                &Regimen::default(),
            )
            .sonus
        };
        assert_eq!(at(0), 255);
        assert_eq!(at(255), 120);
        for onus in 1..=255 {
            assert!(at(onus) <= at(onus - 1));
            assert_eq!(
                u32::from(at(onus)),
                2040 / crate::spatium::isqrt(64 + u32::from(onus))
            );
        }
        assert_eq!(
            concentus(
                &world(32, 24, Modus::Quies),
                &Facultates::default(),
                &Indicia::default(),
                &Regimen::default()
            )
            .contrastus,
            15
        );
    }
}
