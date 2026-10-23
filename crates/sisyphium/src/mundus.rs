//! `mundus`: the world as the pair of state words, its genesis, its persistence as 32
//! hexadecimal digits (REQ-L0-03, REQ-L0-04) and its tick (REQ-L0-05, REQ-H-06). The tick
//! applies the laws of `arbitrium`, `sensus` and `transumptio` in one fixed order,
//! `docs/04-arquitetura.md` §7.1; the module contains no law of its own, it orders them.

use crate::arbitrium::{
    closure, contemplating, continuous, curvature, event, interrupt, modus_law, paces,
};
use crate::corpus::chip;
use crate::ephemeris::ephemeris;
use crate::hospes::{Cellae, Facultates, Indicia, Nuntius, Regimen, Tabula};
use crate::memoria::{Memoria, RAMUS_EXSPECTANS};
use crate::natura::galois;
use crate::sensus::{occlusion, rare};
use crate::transumptio::touch;
use crate::verbum::{Modus, Verbum};

/// The world: the present word and the past-and-intent word.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Mundus {
    /// The present state.
    pub verbum: Verbum,
    /// The past and the intent.
    pub memoria: Memoria,
}

/// Hexadecimal digit of a nibble.
const fn digit(nibble: u64) -> u8 {
    match nibble & 0xF {
        n @ 0..=9 => b'0'.wrapping_add(n as u8),
        n => b'a'.wrapping_add((n as u8).wrapping_sub(10)),
    }
}

/// Value of a hexadecimal digit, upper or lower case.
const fn nibble(c: u8) -> Option<u64> {
    match c {
        b'0'..=b'9' => Some(c.wrapping_sub(b'0') as u64),
        b'a'..=b'f' => Some(c.wrapping_sub(b'a').wrapping_add(10) as u64),
        b'A'..=b'F' => Some(c.wrapping_sub(b'A').wrapping_add(10) as u64),
        _ => None,
    }
}

/// Writes the 16 digits of `word`, most significant first, into `out`.
fn write_word(word: u64, out: &mut [u8]) {
    for (i, slot) in (0u32..16).zip(out.iter_mut()) {
        *slot = digit(word.wrapping_shr(60u32.wrapping_sub(i.wrapping_mul(4))));
    }
}

/// Reads 16 hexadecimal digits as a word; `None` on any other byte.
fn read_word(text: &[u8]) -> Option<u64> {
    text.iter()
        .try_fold(0u64, |acc, &c| nibble(c).map(|n| (acc << 4) | n))
}

/// Cuts a wide integer at the ends of the `i64` range (REQ-H-06).
const fn cut(wide: i128) -> i64 {
    if wide < i64::MIN as i128 {
        i64::MIN
    } else if wide > i64::MAX as i128 {
        i64::MAX
    } else {
        wide as i64
    }
}

/// The epoch of the call `k` of the `ticks` calls that [`Mundus::advance`] makes (REQ-H-06): the
/// clock starts at `epoch_ms` − `ticks`·`tick_ms` and moves `tick_ms` per call, so that the call
/// `ticks` falls at `epoch_ms` whenever the start fits. Both stages, the start and the call
/// start + `k`·`tick_ms`, are computed in 128 bits, where no product of two 32-bit counts
/// overflows, and cut at the ends of the `i64` range: at the ends of time the clock stops
/// instead of wrapping around them.
const fn tick_epoch(epoch_ms: i64, ticks: u32, tick_ms: u32, k: u32) -> i64 {
    let period = tick_ms as i128;
    let start = cut((epoch_ms as i128).saturating_sub((ticks as i128).saturating_mul(period)));
    cut((start as i128).saturating_add((k as i128).saturating_mul(period)))
}

impl Mundus {
    /// The first world of a seed (REQ-L0-04): `semen` from the low 16 bits (zero becomes the
    /// default seed), `conatus` 192, `onus` 32, `ascensus` 0, `curvatura` 0, `automatum` 0x10,
    /// `tranquillitas` 24, `modus` Vigilia; `eventus` from bits 16..=18, `ramus` awaiting,
    /// `gradus` 0, `oraculum` uniform.
    #[must_use]
    pub const fn genesis(seed: u64) -> Self {
        let verbum = Verbum::new(seed & 0xFFFF)
            .with_conatus(192)
            .with_onus(32)
            .with_ascensus(0)
            .with_curvatura(0)
            .with_automatum(0x10)
            .with_tranquillitas(24)
            .with_modus(Modus::Vigilia);
        let memoria = Memoria::new(0)
            .with_eventus(((seed >> 16) & 7) as u8)
            .with_ramus(RAMUS_EXSPECTANS)
            .with_oraculum(0x5555_5555);
        Self { verbum, memoria }
    }

    /// The persisted form: 32 lowercase hexadecimal digits, `verbum` then `memoria`, most
    /// significant nibble first (REQ-L0-03).
    #[must_use]
    pub fn format(&self) -> [u8; 32] {
        let mut out = [b'0'; 32];
        let (head, tail) = out.split_at_mut(16);
        write_word(self.verbum.bits(), head);
        write_word(self.memoria.bits(), tail);
        out
    }

    /// Reads the persisted form: exactly 32 hexadecimal digits of either case; anything else is
    /// `None`; a zero `semen` is normalized (REQ-L0-03).
    #[must_use]
    pub fn parse(text: &[u8]) -> Option<Self> {
        if text.len() != 32 {
            return None;
        }
        let (head, tail) = text.split_at(16);
        Some(Self {
            verbum: Verbum::new(read_word(head)?),
            memoria: Memoria::new(read_word(tail)?),
        })
    }

    /// Renders the world's effective chronicle in the chosen stilus (REQ-L4-04, REQ-H-04).
    pub fn render(
        &self,
        facultates: &Facultates,
        indicia: &Indicia,
        regimen: &Regimen,
        tab: &mut Tabula,
    ) {
        let text = crate::arbitrium::visible_annales(
            self.memoria,
            facultates.lingua,
            indicia.leaving,
            regimen,
            u16::MAX,
        );
        let lines = text
            .lines
            .get(..usize::from(text.count))
            .unwrap_or_default();
        crate::signum::scaena::render(self, facultates, indicia, regimen, lines, tab);
    }

    /// Renders the world's effective chronicle as cells, bounded by the grid's rows as well
    /// as its regimen; refuses a scratch with different dimensions (REQ-L4-04, REQ-H-04).
    #[must_use]
    pub fn render_cellae(
        &self,
        facultates: &Facultates,
        indicia: &Indicia,
        regimen: &Regimen,
        scratch: &mut Tabula,
        cells: &mut Cellae,
    ) -> bool {
        let text = crate::arbitrium::visible_annales(
            self.memoria,
            facultates.lingua,
            indicia.leaving,
            regimen,
            cells.rows,
        );
        let lines = text
            .lines
            .get(..usize::from(text.count))
            .unwrap_or_default();
        crate::signum::scaena::render_cellae(
            self, facultates, indicia, regimen, lines, scratch, cells,
        )
    }

    /// The sixteen RGB colors of the world's chord (REQ-L5-02, REQ-L5-11).
    #[must_use]
    pub fn palette(
        &self,
        facultates: &Facultates,
        indicia: &Indicia,
        regimen: &Regimen,
    ) -> [[u8; 3]; 16] {
        crate::concentus::palette(&crate::concentus::concentus(
            self, facultates, indicia, regimen,
        ))
    }

    /// Takes the words that a law returns and adds its announcement to `said` (REQ-L0-05).
    fn absorb(&mut self, said: Nuntius, law: (Verbum, Memoria, Nuntius)) -> Nuntius {
        (self.verbum, self.memoria) = (law.0, law.1);
        said.cum(law.2)
    }

    /// One tick of the world (REQ-L0-05; `docs/04-arquitetura.md` §7.1), the laws of the levels
    /// L3, L4, L6 and L7 applied in this order, with the ephemeris of the epoch, the zone and
    /// the hemisphere:
    ///
    /// 1. the generator advances `semen` ([`galois`]), so every later law reads the new value;
    /// 2. at `ascensus` 255 the cycle closes ([`closure`], [`Nuntius::CLAUSURA`]);
    /// 3. `leaving` marks the present cycle as interrupted ([`interrupt`]);
    /// 4. `occluded` mutates the ring of cells ([`occlusion`]);
    /// 5. the boulder chips ([`chip`], [`Nuntius::LAPSUS`]);
    /// 6. a touch meets the oracle ([`touch`], [`Nuntius::METALEPSIS`]);
    /// 7. the law of the mode ([`modus_law`], [`Nuntius::AGNITIO`]);
    /// 8. the pace: unless the figure contemplates ([`contemplating`]), a step ([`paces`]) takes
    ///    `ascensus` up by one;
    /// 9. the continuous laws of effort, weight and serenity ([`continuous`]);
    /// 10. the event: age, end, clinamen, persona ([`event`], [`Nuntius::CLINAMEN`],
    ///     [`Nuntius::PERSONA`]);
    /// 11. the dynamics of the curvature ([`curvature`]);
    /// 12. the rare gate reads the memory that the tick leaves ([`rare`], [`Nuntius::RARUM`]).
    ///
    /// The capabilities of the host serve only the ephemeris, through `tz_offset_min` and
    /// `australis`; the touch is a measurement and counts without the `touch` capability. The
    /// result is the union of the announcements of the tick. The function is total: no world and
    /// no measurement makes it panic.
    pub fn step(&mut self, facultates: &Facultates, indicia: &Indicia) -> Nuntius {
        let eph = ephemeris(
            indicia.epoch_ms,
            facultates.tz_offset_min,
            facultates.australis,
        );
        let mut said = Nuntius::NIHIL;
        self.verbum = self.verbum.with_semen(galois(self.verbum.semen()));
        if self.verbum.ascensus() == u8::MAX {
            (self.verbum, self.memoria) = closure(self.verbum, self.memoria);
            said = said.cum(Nuntius::CLAUSURA);
        }
        if indicia.leaving {
            self.memoria = interrupt(self.memoria);
        }
        if indicia.occluded {
            self.verbum = occlusion(self.verbum);
        }
        if let Some(chipped) = chip(self.verbum, indicia.seism) {
            self.verbum = chipped;
            said = said.cum(Nuntius::LAPSUS);
        }
        if let Some((x, y)) = indicia.touch {
            said = self.absorb(said, touch(self.verbum, self.memoria, x, y));
        }
        let modus = modus_law(
            self.verbum,
            self.memoria,
            indicia.energy,
            indicia.seism,
            eph.nocturnal,
            indicia.touch.is_some(),
            indicia.idle_s,
        );
        said = self.absorb(said, modus);
        let at_rest = contemplating(self.verbum, self.memoria);
        let stepped = !at_rest && paces(self.verbum);
        if stepped {
            let climbed = self.verbum.ascensus().saturating_add(1);
            self.verbum = self.verbum.with_ascensus(climbed);
        }
        self.verbum = continuous(self.verbum, stepped, at_rest, indicia.seism, indicia.idle_s);
        let happening = event(
            self.verbum,
            self.memoria,
            eph.phase,
            indicia.idle_s,
            indicia.occluded,
        );
        said = self.absorb(said, happening);
        self.verbum = curvature(self.verbum);
        if rare(
            indicia.epoch_ms,
            self.memoria.clausurae(),
            self.memoria.eventus(),
        ) {
            said = said.cum(Nuntius::RARUM);
        }
        said
    }

    /// Applies `ticks` calls of [`Mundus::step`] (REQ-H-06) with the measurements `indicia`, whose
    /// `epoch_ms` is the epoch of the last call: the call `k` in 1..=`ticks` runs at
    /// `epoch_ms` − `ticks`·`tick_ms` + `k`·`tick_ms`, cut at the ends of the `i64` range, and
    /// every other measurement stays as it is. The result is the union of the announcements of
    /// the calls. The host bounds `ticks` by the time it recovers; the function takes any `u32`.
    pub fn advance(
        &mut self,
        facultates: &Facultates,
        indicia: &Indicia,
        ticks: u32,
        tick_ms: u32,
    ) -> Nuntius {
        (1..=ticks).fold(Nuntius::NIHIL, |said, k| {
            let at = Indicia {
                epoch_ms: tick_epoch(indicia.epoch_ms, ticks, tick_ms, k),
                ..*indicia
            };
            said.cum(self.step(facultates, &at))
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
    use super::*;
    use crate::arbitrium::HERACLITUS;
    use crate::ephemeris::ephemeris;
    use crate::hospes::{Facultates, Indicia, Lingua, Nuntius};
    use crate::memoria::RAMUS_EXSPECTANS;
    use crate::natura::{galois, mix};
    use crate::sensus::rare;
    use crate::transumptio::weight;
    use crate::verbum::{Modus, SEMEN_DEFAULT};
    use std::vec::Vec;

    /// The public render uses the world's chronicle and honours its layer in both media
    /// (REQ-L4-04, REQ-H-04).
    #[test]
    fn render_uses_annales() {
        use crate::hospes::{ANNALES, Cellae, Regimen, Tabula};
        use crate::signum::pixel::lum_of;
        let mut m = Mundus::genesis(1);
        let f = Facultates::default();
        let i = Indicia {
            epoch_ms: NOON,
            ..Indicia::default()
        };
        let r = Regimen::default();
        let mut buf = std::vec![0; 320 * 180 + 8];
        for ramus in [3, 1] {
            m.memoria = m.memoria.with_eventus(3).with_ramus(ramus).with_gradus(300);
            for enabled in [false, true] {
                let r = Regimen {
                    layers: if enabled {
                        r.layers
                    } else {
                        r.layers & !ANNALES
                    },
                    ..r
                };
                m.render(&f, &i, &r, &mut Tabula::new(320, 180, &mut buf).unwrap());
                let start = usize::from(crate::spatium::compositio(320, 180, 6).annales.y) * 320;
                assert_eq!(
                    buf[start..320 * 180].iter().any(|&p| lum_of(p) == 13),
                    enabled && ramus != 3
                );
                assert!(buf[320 * 180..].iter().all(|&p| p == 0));
                let mut scratch = [0; 80 * 2 + 8];
                let mut cells = [0xBEEF; 80 * 2 + 8];
                assert!(m.render_cellae(
                    &f,
                    &i,
                    &r,
                    &mut Tabula::new(80, 2, &mut scratch).unwrap(),
                    &mut Cellae::new(80, 2, &mut cells).unwrap()
                ));
                assert_eq!(
                    cells[..160].iter().any(|&c| (c >> 8) & 15 == 13),
                    enabled && ramus != 3
                );
                assert!(cells[160..].iter().all(|&c| c == 0xBEEF));
                if enabled && ramus != 3 {
                    let full = crate::arbitrium::annales(m.memoria, f.lingua, false);
                    for (row, line) in full.lines[..2].iter().enumerate() {
                        for (x, ch) in line.chars().enumerate() {
                            assert_eq!(
                                cells[row * 80 + x] & 255,
                                u16::from(crate::signum::littera::cell_glyph(ch))
                            );
                        }
                    }
                }
            }
        }
        assert_eq!(
            m.palette(&f, &i, &r),
            crate::concentus::palette(&crate::concentus::concentus(&m, &f, &i, &r))
        );
        m.memoria = m.memoria.with_eventus(12).with_gradus(80);
        let off = Regimen { layers: 0, ..r };
        m.render(&f, &i, &off, &mut Tabula::new(320, 180, &mut buf).unwrap());
        assert!(buf[..320 * 180].iter().any(|&p| lum_of(p) == 10));
        assert!(buf[..320 * 180].iter().all(|&p| lum_of(p) != 13));
        let mut scratch = [0; 80 * 2];
        let mut cells = [0; 80 * 2];
        assert!(m.render_cellae(
            &f,
            &i,
            &off,
            &mut Tabula::new(80, 2, &mut scratch).unwrap(),
            &mut Cellae::new(80, 2, &mut cells).unwrap()
        ));
        assert_eq!(
            cells[..80].iter().filter(|&&c| (c >> 8) & 15 == 10).count(),
            16
        );
        assert!(cells.iter().all(|&c| (c >> 8) & 15 != 13));
    }

    /// The solstice noon of `docs/08-verificacao.md` §7, 2026-06-21 12:00 at −03:00 (circadian
    /// phase 128), in milliseconds since the epoch.
    const NOON: i64 = 1_782_054_000_000;

    /// 17:37:30 of the same day (circadian phase 188): the hour of Heraclitus, by day.
    const TWILIGHT: i64 = NOON + (5 * 3600 + 37 * 60 + 30) * 1000;

    /// The eight vectors of measurements of the totality property (REQ-L0-05): the defaults; the
    /// solstice noon with 120 of energy and an hour of idleness; the solstice night with 200 of
    /// energy and 4 000 s of idleness; 20 of energy; the greatest agitation and regularity right
    /// after a return; a touch in a corner without idleness; occlusion with departure; the
    /// earliest epoch with no energy and no idleness.
    fn environments() -> [Indicia; 8] {
        let base = Indicia::default();
        [
            base,
            Indicia {
                epoch_ms: 1_782_054_000_000,
                energy: 120,
                idle_s: 3600,
                ..base
            },
            Indicia {
                epoch_ms: 1_797_831_000_000,
                energy: 200,
                idle_s: 4000,
                ..base
            },
            Indicia { energy: 20, ..base },
            Indicia {
                seism: 255,
                automatism: 255,
                returned_ms: 0,
                ..base
            },
            Indicia {
                touch: Some((0, 255)),
                idle_s: 0,
                ..base
            },
            Indicia {
                occluded: true,
                leaving: true,
                ..base
            },
            Indicia {
                epoch_ms: i64::MIN,
                energy: 0,
                idle_s: 0,
                ..base
            },
        ]
    }

    /// The default capabilities and the measurements of the hour of Heraclitus: the defaults at
    /// `TWILIGHT`, which falls at circadian phase 188 under the default zone.
    fn twilight() -> (Facultates, Indicia) {
        let f = Facultates::default();
        assert_eq!(ephemeris(TWILIGHT, f.tz_offset_min, f.australis).phase, 188);
        (
            f,
            Indicia {
                epoch_ms: TWILIGHT,
                ..Indicia::default()
            },
        )
    }

    /// The smallest state of the generator that does not open the gate of the personae,
    /// (`semen` ∧ 0x1FF) = 0, and whose successor opens it.
    fn opens_next() -> u16 {
        (1..=u16::MAX)
            .find(|&s| s & 0x1FF != 0 && galois(s) & 0x1FF == 0)
            .unwrap()
    }

    /// The smallest state of the generator that opens the gate of the personae and whose
    /// successor does not.
    fn closes_next() -> u16 {
        (1..=u16::MAX)
            .find(|&s| s & 0x1FF == 0 && galois(s) & 0x1FF != 0)
            .unwrap()
    }

    /// A world one tick before the hour of Heraclitus: the clinamen of event 3 awaits its place
    /// outside its band, `ascensus` lies in the window of the persona, the pace stays shut for
    /// the successor of `semen` and the gate of the personae is the only one that may open.
    fn awaiting(semen: u16) -> Mundus {
        Mundus {
            verbum: Verbum::new(0)
                .with_semen(semen)
                .with_ascensus(100)
                .with_conatus(100)
                .with_onus(50),
            memoria: Memoria::new(0)
                .with_eventus(3)
                .with_ramus(3)
                .with_gradus(10),
        }
    }

    /// The state of the generator whose successor is `next`: the inverse of the step of Galois,
    /// whose successor has the top bit set exactly when the state was odd (REQ-L1-01).
    fn preceding(next: u16) -> u16 {
        let old = if next & 0x8000 != 0 {
            ((next ^ 0xB400) << 1) | 1
        } else {
            next << 1
        };
        assert_eq!(galois(old), next);
        old
    }

    /// The world of `awaiting` whose tick reads `next` as `semen`: the generator stands at the
    /// state that precedes it.
    fn reading(next: u16) -> Mundus {
        awaiting(preceding(next))
    }

    /// The world of `Mundus::genesis(6)` whose oracle predicts quadrant 0 after a touch in
    /// quadrant 1: the row 1 holds the weights 3, 1, 0, 0, a ready trap (REQ-L7-02).
    fn predicting() -> Mundus {
        let mut w = Mundus::genesis(6);
        let row_one = (0x5555_5555u32 & !0xFF00) | (0x07 << 8);
        w.memoria = w.memoria.with_oraculum(row_one).with_quadrans(1);
        w
    }

    /// Genesis sets every field to its documented value (REQ-L0-04).
    #[test]
    fn genesis_fields() {
        let m = Mundus::genesis(0x0007_1234);
        let v = m.verbum;
        assert_eq!(v.semen(), 0x1234);
        assert_eq!((v.conatus(), v.onus(), v.ascensus()), (192, 32, 0));
        assert_eq!(
            (v.curvatura(), v.automatum(), v.tranquillitas()),
            (0, 0x10, 24)
        );
        assert_eq!(v.modus(), Modus::Vigilia);
        let r = m.memoria;
        assert_eq!((r.clausurae(), r.quadrans(), r.eventus()), (0, 0, 7));
        assert_eq!(
            (r.ramus(), r.gradus(), r.oraculum()),
            (RAMUS_EXSPECTANS, 0, 0x5555_5555)
        );
        assert_eq!(Mundus::genesis(0).verbum.semen(), SEMEN_DEFAULT);
        assert_eq!(Mundus::genesis(0x0005_0000).memoria.eventus(), 5);
        assert_eq!(Mundus::genesis(0x000F_0000).memoria.eventus(), 7);
    }

    /// Writing then reading the 32 digits is the identity over 65 535 worlds (REQ-L0-03).
    #[test]
    fn format_parse_roundtrip() {
        let mut semen = 1u16;
        let mut key = 0u64;
        for _ in 0..65_535u32 {
            semen = galois(semen);
            key = mix(key ^ u64::from(semen));
            let m = Mundus {
                verbum: crate::verbum::Verbum::new(key),
                memoria: crate::memoria::Memoria::new(mix(key)),
            };
            let text = m.format();
            assert!(
                text.iter()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
            );
            assert_eq!(Mundus::parse(&text), Some(m));
        }
        let text = Mundus::genesis(0xACE1).format();
        assert_eq!(&text, b"1810000020c0ace15555555500300000");
    }

    /// Other lengths and non-hexadecimal characters are rejected; uppercase is accepted;
    /// a zero `semen` is normalized (REQ-L0-03).
    #[test]
    fn parse_rejects_invalid() {
        let valid = b"0123456789abcdef0123456789abcdef";
        assert!(Mundus::parse(valid).is_some());
        assert_eq!(
            Mundus::parse(b"0123456789ABCDEF0123456789ABCDEF"),
            Mundus::parse(valid)
        );
        assert!(Mundus::parse(&valid[..31]).is_none());
        let mut long = [b'0'; 33];
        long[..32].copy_from_slice(valid);
        assert!(Mundus::parse(&long).is_none());
        let mut bad = *valid;
        bad[5] = b'g';
        assert!(Mundus::parse(&bad).is_none());
        let zero = Mundus::parse(b"00000000000000000000000000000000");
        assert_eq!(zero.map(|m| m.verbum.semen()), Some(SEMEN_DEFAULT));
        assert!(Mundus::parse(b"").is_none());
    }

    /// Every world, under eight environments, steps without panic; the generator advances by one;
    /// operational curvature stays operational; only the seven announcement bits appear (REQ-L0-05).
    #[test]
    fn step_is_total() {
        let f = Facultates::default();
        let envs: [Indicia; 8] = environments();
        let mut semen = 1u16;
        for _ in 0..65_535u32 {
            semen = galois(semen);
            let k = mix(u64::from(semen));
            let world = Mundus {
                verbum: Verbum::new(k),
                memoria: Memoria::new(mix(k)),
            };
            for i in &envs {
                let mut w = world;
                let n = w.step(&f, i);
                assert_eq!(w.verbum.semen(), galois(world.verbum.semen()));
                if world.verbum.curvatura().unsigned_abs() <= 64 {
                    assert!(w.verbum.curvatura().unsigned_abs() <= 64);
                }
                assert_eq!(n.0 & 0x80, 0);
            }
        }
    }

    /// `advance` is `ticks` steps at interpolated epochs and returns the union of announcements
    /// (REQ-H-06).
    #[test]
    fn advance_equals_repeated_step() {
        let f = Facultates::default();
        let i = Indicia {
            epoch_ms: 1_782_054_000_000,
            energy: 255,
            idle_s: 3600,
            ..Indicia::default()
        };
        for ticks in [0u32, 1, 7, 4096] {
            let mut a = Mundus::genesis(1);
            let na = a.advance(&f, &i, ticks, 42);
            let mut b = Mundus::genesis(1);
            let mut nb = Nuntius::NIHIL;
            for k in 1..=ticks {
                let at = Indicia {
                    epoch_ms: i.epoch_ms - i64::from(ticks) * 42 + i64::from(k) * 42,
                    ..i
                };
                nb = nb.cum(b.step(&f, &at));
            }
            assert_eq!((a, na), (b, nb), "ticks {ticks}");
        }
    }

    /// Epochs saturate at the ends of time (review focus 1).
    #[test]
    fn advance_saturates_epoch() {
        let f = Facultates::default();
        for epoch in [i64::MIN + 5, i64::MAX - 5] {
            let mut w = Mundus::genesis(9);
            let _ = w.advance(
                &f,
                &Indicia {
                    epoch_ms: epoch,
                    ..Indicia::default()
                },
                65_536,
                60_000,
            );
            assert_ne!(w, Mundus::genesis(9));
        }
    }

    /// The summit rests one tick, then the cycle closes; a touch wakes the dreamer; Quies holds
    /// the figure still (D2, D7, D8).
    #[test]
    fn step_order() {
        let f = Facultates::default();
        let i = Indicia {
            epoch_ms: 1_782_054_000_000,
            ..Indicia::default()
        };
        let mut top = Mundus::genesis(5);
        top.verbum = top.verbum.with_ascensus(255);
        let n = top.step(&f, &i);
        assert!(n.contains(Nuntius::CLAUSURA));
        assert!(top.verbum.ascensus() <= 1);
        let mut dream = Mundus::genesis(6);
        dream.verbum = dream.verbum.with_modus(Modus::Somnium).with_ascensus(100);
        let night = Indicia {
            epoch_ms: 1_797_831_000_000,
            touch: Some((10, 10)),
            idle_s: 4000,
            ..Indicia::default()
        };
        assert!(dream.step(&f, &night).contains(Nuntius::AGNITIO));
        assert_eq!((dream.memoria.eventus(), dream.verbum.ascensus()), (12, 0));
        let mut still = Mundus::genesis(7);
        still.verbum = still.verbum.with_modus(Modus::Quies).with_ascensus(77);
        let dark = Indicia { energy: 20, ..i };
        let _ = still.advance(&f, &dark, 10_000, 60_000);
        assert_eq!(still.verbum.ascensus(), 77);
    }

    /// The dynamics of the curvature follow the event: the visit of Heraclitus adds 8 to a
    /// curvature near the operational bound, and the tick leaves it at the bound in affirmation
    /// and at three quarters of the sum in waking, never beyond +64 (REQ-L4-09, REQ-L4-11).
    #[test]
    fn curvature_follows_the_event() {
        let (f, i) = twilight();
        for kappa in 57..=64i8 {
            for (modus, expected) in [
                (Modus::AmorFati, 64),
                (Modus::Vigilia, (i16::from(kappa) + 8) * 3 / 4),
            ] {
                let mut w = awaiting(opens_next());
                w.verbum = w.verbum.with_modus(modus).with_curvatura(kappa);
                let n = w.step(&f, &i);
                let at = std::format!("{modus:?} {kappa}");
                assert!(n.contains(Nuntius::PERSONA), "{at}");
                assert_eq!(w.memoria.eventus(), HERACLITUS, "{at}");
                assert_eq!(i16::from(w.verbum.curvatura()), expected, "{at}");
            }
        }
    }

    /// Whatever curvature a world carries, a tick leaves it in −64..=64, and both the worlds
    /// inside the range and the worlds outside it occur (REQ-L4-11).
    #[test]
    fn curvature_is_operational_after_every_step() {
        let f = Facultates::default();
        let envs = environments();
        let (mut s, mut seen) = (1u16, [false; 2]);
        for _ in 0..8192u32 {
            s = galois(s);
            let k = mix(u64::from(s));
            let world = Mundus {
                verbum: Verbum::new(k),
                memoria: Memoria::new(mix(k)),
            };
            for i in &envs {
                let mut w = world;
                let _ = w.step(&f, i);
                assert!(w.verbum.curvatura().unsigned_abs() <= 64, "{k:#x}");
                seen[usize::from(world.verbum.curvatura().unsigned_abs() > 64)] = true;
            }
        }
        assert_eq!(seen, [true; 2]);
    }

    /// Every law of the tick reads the generator after its step: the personae enter when the
    /// successor of `semen` opens their gate and not when only `semen` itself opens it
    /// (REQ-L1-01, REQ-L4-09).
    #[test]
    fn step_reads_the_advanced_generator() {
        let (f, i) = twilight();
        let mut entering = awaiting(opens_next());
        assert!(entering.step(&f, &i).contains(Nuntius::PERSONA));
        assert_eq!(entering.memoria.eventus(), HERACLITUS);
        let mut waiting = awaiting(closes_next());
        assert!(!waiting.step(&f, &i).contains(Nuntius::PERSONA));
        assert_eq!(waiting.memoria.eventus(), 3);
    }

    /// The interruption marks the cycle that the closure opens: a world at the summit that the
    /// observer leaves shifts the ring and then sets its bit 0 (REQ-L4-07, REQ-L4-08).
    #[test]
    fn interruption_follows_closure() {
        let f = Facultates::default();
        let mut w = Mundus::genesis(5);
        w.verbum = w.verbum.with_ascensus(255);
        w.memoria = w.memoria.with_clausurae(0b101);
        let leaving = Indicia {
            epoch_ms: NOON,
            leaving: true,
            ..Indicia::default()
        };
        let mut left = w;
        assert!(left.step(&f, &leaving).contains(Nuntius::CLAUSURA));
        assert_eq!(left.memoria.clausurae(), 0b1011);
        let mut stayed = w;
        let _ = stayed.step(
            &f,
            &Indicia {
                leaving: false,
                ..leaving
            },
        );
        assert_eq!(stayed.memoria.clausurae(), 0b1010);
    }

    /// The rare gate closes the tick: it reads the ring of closures that the interruption has
    /// marked and the event that the end of the previous one has chosen, so that the windows it
    /// announces are those that the memory after the tick opens, and not those that the memory
    /// before it opened (REQ-L6-03).
    #[test]
    fn rare_gate_reads_the_memory_of_the_tick() {
        let f = Facultates::default();
        let mut interrupted = Mundus::genesis(7);
        interrupted.memoria = interrupted.memoria.with_eventus(3).with_ramus(3);
        let mut ended = Mundus::genesis(7);
        ended.memoria = ended.memoria.with_eventus(9).with_ramus(0).with_gradus(239);
        for (world, leaving) in [(interrupted, true), (ended, false)] {
            let mut after = world;
            let _ = after.step(
                &f,
                &Indicia {
                    leaving,
                    ..Indicia::default()
                },
            );
            let (before, later) = (world.memoria, after.memoria);
            assert_ne!(
                (before.clausurae(), before.eventus()),
                (later.clausurae(), later.eventus())
            );
            let opens = |w: i64, m: Memoria| rare(w * 4000, m.clausurae(), m.eventus());
            let after_only = (0..1i64 << 18)
                .find(|&w| opens(w, later) && !opens(w, before))
                .unwrap();
            let before_only = (0..1i64 << 18)
                .find(|&w| opens(w, before) && !opens(w, later))
                .unwrap();
            for (w, announced) in [(after_only, true), (before_only, false)] {
                let mut probe = world;
                let at = Indicia {
                    epoch_ms: w * 4000,
                    leaving,
                    ..Indicia::default()
                };
                let n = probe.step(&f, &at);
                assert_eq!(n.contains(Nuntius::RARUM), announced, "{leaving} {w}");
            }
        }
    }

    /// `step` takes from the capabilities only the zone and the hemisphere, for the ephemeris:
    /// every other capability changes nothing, the hemisphere only turns the season, a touch
    /// counts without the `touch` capability and the zone moves the hour of the personae;
    /// every zone and hemisphere at the extremes of the epoch still leave a world whose
    /// generator has advanced (REQ-H-06, REQ-L0-05).
    #[test]
    fn step_reads_facultates_only_for_the_ephemeris() {
        let base = Facultates::default();
        assert!(!base.touch);
        let other = Facultates {
            width: 1920,
            height: 1080,
            color_bits: 1,
            max_hz: 0,
            touch: true,
            motion: true,
            occlusion: true,
            persistence: true,
            cells: true,
            australis: !base.australis,
            lingua: Lingua::Pt,
            compute: 0,
            ..base
        };
        let envs = environments();
        let mut s = 1u16;
        for _ in 0..512u32 {
            s = galois(s);
            let k = mix(u64::from(s));
            let world = Mundus {
                verbum: Verbum::new(k),
                memoria: Memoria::new(mix(k)),
            };
            for i in &envs {
                let (mut a, mut b) = (world, world);
                assert_eq!(a.step(&base, i), b.step(&other, i));
                assert_eq!(a, b);
            }
        }
        let mut touched = Mundus::genesis(1);
        let at = Indicia {
            touch: Some((200, 10)),
            ..Indicia::default()
        };
        let _ = touched.step(&base, &at);
        assert_eq!(touched.memoria.quadrans(), 1);
        assert_eq!(weight(touched.memoria.oraculum(), 0, 1), 2);
        let (f, i) = twilight();
        assert!(
            awaiting(opens_next())
                .step(&f, &i)
                .contains(Nuntius::PERSONA)
        );
        let utc = Facultates {
            tz_offset_min: 0,
            ..f
        };
        assert!(
            !awaiting(opens_next())
                .step(&utc, &i)
                .contains(Nuntius::PERSONA)
        );
        for tz_offset_min in [i16::MIN, -720, 0, 840, i16::MAX] {
            for epoch_ms in [i64::MIN, -1, 0, NOON, i64::MAX] {
                for australis in [false, true] {
                    let zone = Facultates {
                        australis,
                        tz_offset_min,
                        ..base
                    };
                    let mut w = Mundus::genesis(3);
                    let at = Indicia {
                        epoch_ms,
                        ..Indicia::default()
                    };
                    let _ = w.step(&zone, &at);
                    assert_eq!(w.verbum.semen(), galois(Mundus::genesis(3).verbum.semen()));
                }
            }
        }
    }

    /// A special event, whatever its branch, holds for 239 steps and gives way to a clinamen in
    /// the 240th, for a persisted world as for one the laws reach; an event older than 240 ticks
    /// ends in the first step (REQ-L4-03, REQ-L4-09).
    #[test]
    fn special_events_end_in_a_step() {
        let f = Facultates::default();
        let i = Indicia {
            epoch_ms: NOON,
            idle_s: 0,
            ..Indicia::default()
        };
        for eventus in 8..=15u8 {
            for ramus in 0..=3u8 {
                let mut w = Mundus::genesis(11);
                w.memoria = w.memoria.with_eventus(eventus).with_ramus(ramus);
                let mut old = w;
                old.memoria = old.memoria.with_gradus(1023);
                for tick in 1..240u16 {
                    let _ = w.step(&f, &i);
                    let held = (w.memoria.eventus(), w.memoria.ramus(), w.memoria.gradus());
                    assert_eq!(held, (eventus, ramus, tick), "{eventus}.{ramus}");
                }
                for (world, age) in [(w, 240), (old, 1023)] {
                    let mut ended = world;
                    let _ = ended.step(&f, &i);
                    let (next, gradus) = (ended.memoria.eventus(), ended.memoria.gradus());
                    assert!(
                        next < 8 && gradus == 0,
                        "{eventus}.{ramus} at {age}: {next}"
                    );
                }
            }
        }
    }

    /// The clock of `advance` is the clock of wide integers cut at the ends of the `i64` range,
    /// at its two stages, the start `epoch_ms` − `ticks`·`tick_ms` and the call
    /// start + `k`·`tick_ms`; it ends at `epoch_ms` when the start fits, and it stops at the end of
    /// time when it does not (REQ-H-06).
    #[test]
    fn tick_epoch_saturates_like_wide_integers() {
        let cut = |wide: i128| wide.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64;
        for epoch in [i64::MIN, i64::MIN + 5, -1, 0, NOON, i64::MAX - 5, i64::MAX] {
            for tick_ms in [0u32, 1, 42, 60_000, u32::MAX] {
                for ticks in [0u32, 1, 2, 65_536, u32::MAX] {
                    let start = cut(i128::from(epoch) - i128::from(ticks) * i128::from(tick_ms));
                    for k in [0u32, 1, ticks / 2, ticks.saturating_sub(1), ticks, u32::MAX] {
                        let expected = cut(i128::from(start) + i128::from(k) * i128::from(tick_ms));
                        let at = tick_epoch(epoch, ticks, tick_ms, k);
                        assert_eq!(at, expected, "{epoch} {ticks} {tick_ms} {k}");
                    }
                    let fits = i128::from(epoch) - i128::from(ticks) * i128::from(tick_ms)
                        >= i128::from(i64::MIN);
                    if fits {
                        assert_eq!(tick_epoch(epoch, ticks, tick_ms, ticks), epoch);
                    }
                }
            }
        }
        assert_eq!(tick_epoch(i64::MIN + 5, 65_536, 60_000, 0), i64::MIN);
        assert_eq!(
            tick_epoch(i64::MIN + 5, 65_536, 60_000, 1),
            i64::MIN + 60_000
        );
        assert_eq!(
            tick_epoch(i64::MAX - 5, 65_536, 60_000, 65_536),
            i64::MAX - 5
        );
    }

    /// A touch that the trap has predicted wakes a dreamer without a metalepsis, and arms the
    /// affirmation in a waking world: the touch law meets the mode that the tick found, and the
    /// law of the mode meets the touch (D2, D8; REQ-L7-03, REQ-L4-10).
    #[test]
    fn touch_wakes_the_dreamer_without_metalepsis() {
        let f = Facultates::default();
        let night = Indicia {
            epoch_ms: 1_797_831_000_000,
            touch: Some((10, 10)),
            idle_s: 4000,
            ..Indicia::default()
        };
        let world = predicting();
        let mut dreamer = world;
        dreamer.verbum = dreamer.verbum.with_modus(Modus::Somnium).with_ascensus(100);
        let n = dreamer.step(&f, &night);
        assert!(n.contains(Nuntius::AGNITIO) && !n.contains(Nuntius::METALEPSIS));
        assert_eq!(
            (dreamer.verbum.modus(), dreamer.memoria.eventus()),
            (Modus::Vigilia, 12)
        );
        let mut waking = world;
        let n = waking.step(&f, &night);
        assert!(n.contains(Nuntius::METALEPSIS) && !n.contains(Nuntius::AGNITIO));
        assert_eq!(
            (waking.verbum.modus(), waking.memoria.eventus()),
            (Modus::AmorFati, 11)
        );
    }

    /// Scarcity has the last word on the mode: with 20 of energy the tick of a predicted touch
    /// still announces the metalepsis and sets its event, and the law of the mode puts Quies
    /// over the affirmation (REQ-L4-10, REQ-L7-03).
    #[test]
    fn scarcity_overrides_the_affirmation() {
        let f = Facultates::default();
        let scarce = Indicia {
            epoch_ms: NOON,
            energy: 20,
            touch: Some((10, 10)),
            ..Indicia::default()
        };
        let mut w = predicting();
        let n = w.step(&f, &scarce);
        assert!(n.contains(Nuntius::METALEPSIS));
        assert_eq!((w.verbum.modus(), w.memoria.eventus()), (Modus::Quies, 11));
    }

    /// The continuous laws read the measurements of the tick: a step costs one unit of `conatus`;
    /// an agitation of 128 strains one more and, from 64, takes one of serenity; contemplation
    /// regains `conatus` through the gate 0x1F where the climb needs 0x1FF; an idleness of 600 s
    /// lets serenity grow (REQ-L4-05, REQ-L4-06).
    #[test]
    fn continuous_laws_read_the_measurements() {
        let f = Facultates::default();
        let by_day = Indicia {
            epoch_ms: NOON,
            ..Indicia::default()
        };
        let run = |next: u16, at: &Indicia| {
            let mut w = reading(next);
            w.verbum = w
                .verbum
                .with_ascensus(50)
                .with_conatus(200)
                .with_onus(0)
                .with_tranquillitas(24);
            let _ = w.step(&f, at);
            w.verbum
        };
        let read = |v: Verbum| (v.ascensus(), v.conatus(), v.tranquillitas());
        assert_eq!(read(run(100, &by_day)), (51, 199, 24));
        let strained = Indicia {
            seism: 128,
            ..by_day
        };
        assert_eq!(read(run(100, &strained)), (51, 198, 23));
        assert_eq!(
            read(run(
                100,
                &Indicia {
                    seism: 64,
                    ..by_day
                }
            )),
            (51, 199, 23)
        );
        assert_eq!(
            read(run(
                100,
                &Indicia {
                    seism: 63,
                    ..by_day
                }
            )),
            (51, 199, 24)
        );
        let quies = Indicia {
            energy: 20,
            ..by_day
        };
        assert_eq!(run(0x0100, &quies).modus(), Modus::Quies);
        assert_eq!(run(0x0100, &quies).ascensus(), 50);
        assert_eq!(run(0x0100, &quies).conatus(), 201);
        assert_eq!(run(0x0100, &by_day).conatus(), 200);
        let patient = Indicia {
            idle_s: 600,
            ..by_day
        };
        assert_eq!(run(0x0100, &patient).tranquillitas(), 25);
        let hurried = Indicia {
            idle_s: 599,
            ..by_day
        };
        assert_eq!(run(0x0100, &hurried).tranquillitas(), 24);
    }

    /// The continuous laws come before the event: a clinamen that adds 45 to a weight of 250 in
    /// the tick where the weight also drifts toward 32 leaves 255, because the drift takes the
    /// weight to 249 and the event then saturates it, where the reverse order would saturate
    /// first and leave 254; the clinamen also wins over a persona whose gate is open in the same
    /// tick (REQ-L4-06, REQ-L4-02, REQ-L4-09).
    #[test]
    fn continuous_laws_precede_the_event() {
        let f = Facultates::default();
        let i = Indicia {
            epoch_ms: NOON,
            ..Indicia::default()
        };
        let mut w = reading(0x2000);
        w.verbum = w.verbum.with_ascensus(100).with_onus(250);
        w.memoria = w.memoria.with_eventus(1).with_ramus(3);
        let n = w.step(&f, &i);
        assert!(n.contains(Nuntius::CLINAMEN) && !n.contains(Nuntius::PERSONA));
        assert_eq!((w.memoria.ramus(), w.memoria.gradus()), (2, 0));
        assert_eq!(w.verbum.onus(), 255);
    }

    /// Nothing enters under the observer's eye: the persona of the hour enters when the idleness
    /// reaches a minute or the work is occluded, and not before (REQ-L4-09).
    #[test]
    fn personae_enter_in_absence_only() {
        let (f, i) = twilight();
        for (idle_s, occluded, enters) in [
            (59u32, false, false),
            (60, false, true),
            (0, true, true),
            (0, false, false),
        ] {
            let mut w = awaiting(opens_next());
            let at = Indicia {
                idle_s,
                occluded,
                ..i
            };
            let n = w.step(&f, &at);
            assert_eq!(n.contains(Nuntius::PERSONA), enters, "{idle_s} {occluded}");
        }
    }

    /// The chip comes after the occlusion: with the successor of `semen` at 0x0380 the chip flips
    /// bit 0 of the ring, and an occluded world rotates the ring first, so that the flip lands on
    /// the rotated ring; the chip is announced, and only from an agitation of 240 (REQ-L3-04,
    /// REQ-L6-01).
    #[test]
    fn chip_follows_the_occlusion() {
        let f = Facultates::default();
        let shaken = Indicia {
            epoch_ms: NOON,
            seism: 240,
            ..Indicia::default()
        };
        let mut world = reading(0x0380);
        world.verbum = world.verbum.with_automatum(0b10).with_onus(100);
        let ring = |at: &Indicia| {
            let mut w = world;
            let n = w.step(&f, at);
            (w.verbum.automatum(), n.contains(Nuntius::LAPSUS))
        };
        assert_eq!(ring(&shaken), (0b11, true));
        let hidden = Indicia {
            occluded: true,
            ..shaken
        };
        assert_eq!(ring(&hidden), (0b101, true));
        assert_eq!(
            ring(&Indicia {
                seism: 239,
                ..shaken
            }),
            (0b10, false)
        );
    }

    /// The cycle closes at the summit and not before: a world at 254 that takes no step stays
    /// there, one at 255 closes, and one that takes the last step rests a tick at 255, in sight,
    /// and closes in the next tick (D2; REQ-L4-07).
    #[test]
    fn cycle_closes_a_tick_after_the_summit() {
        let f = Facultates::default();
        let i = Indicia {
            epoch_ms: NOON,
            ..Indicia::default()
        };
        for (ascensus, closes) in [(253u8, false), (254, false), (255, true)] {
            let mut w = reading(0x0201);
            w.verbum = w.verbum.with_ascensus(ascensus);
            let n = w.step(&f, &i);
            assert_eq!(n.contains(Nuntius::CLAUSURA), closes, "{ascensus}");
            assert_eq!(w.verbum.ascensus(), if closes { 0 } else { ascensus });
        }
        let mut climbing = reading(4);
        climbing.verbum = climbing
            .verbum
            .with_ascensus(254)
            .with_conatus(200)
            .with_onus(0);
        assert!(!climbing.step(&f, &i).contains(Nuntius::CLAUSURA));
        assert_eq!(climbing.verbum.ascensus(), 255);
        assert!(climbing.step(&f, &i).contains(Nuntius::CLAUSURA));
        assert!(climbing.verbum.ascensus() <= 1);
    }

    /// The dream follows the night and the absence: a waking world opens it at night, from
    /// 2 820 s of idleness and below an agitation of 2, through the gate of the generator, and
    /// not by day, nor with less idleness, nor with more agitation; once open it persists through
    /// the night and ends in the recognition at dawn or under an agitation of 16 (REQ-L4-10).
    #[test]
    fn dream_follows_the_night_and_the_absence() {
        let f = Facultates::default();
        let night = Indicia {
            epoch_ms: 1_797_831_000_000,
            idle_s: 2820,
            ..Indicia::default()
        };
        let by_day = Indicia {
            epoch_ms: NOON,
            ..night
        };
        let after = |world: Mundus, at: &Indicia| {
            let mut w = world;
            let n = w.step(&f, at);
            (w.verbum.modus(), n.contains(Nuntius::AGNITIO))
        };
        let waking = reading(0x1000);
        assert_eq!(after(waking, &night), (Modus::Somnium, false));
        assert_eq!(after(waking, &by_day).0, Modus::Vigilia);
        assert_eq!(
            after(
                waking,
                &Indicia {
                    idle_s: 2819,
                    ..night
                }
            )
            .0,
            Modus::Vigilia
        );
        assert_eq!(
            after(waking, &Indicia { seism: 2, ..night }).0,
            Modus::Vigilia
        );
        let mut dreamer = waking;
        dreamer.verbum = dreamer.verbum.with_modus(Modus::Somnium);
        assert_eq!(after(dreamer, &night), (Modus::Somnium, false));
        assert_eq!(after(dreamer, &by_day), (Modus::Vigilia, true));
        assert_eq!(
            after(dreamer, &Indicia { seism: 16, ..night }),
            (Modus::Vigilia, true)
        );
    }

    /// `advance` runs each call at its own epoch: a persona whose hour ends before the last call
    /// enters in the first one, as a repeated `step` at the interpolated epochs does (REQ-H-06).
    #[test]
    fn advance_runs_each_call_at_its_epoch() {
        let (f, _) = twilight();
        let hours = 3 * 3_600_000u32;
        let last = Indicia {
            epoch_ms: TWILIGHT + i64::from(hours),
            ..Indicia::default()
        };
        let mut a = awaiting(opens_next());
        let n = a.advance(&f, &last, 2, hours);
        assert!(n.contains(Nuntius::PERSONA));
        let mut b = awaiting(opens_next());
        let first = b.step(
            &f,
            &Indicia {
                epoch_ms: TWILIGHT,
                ..last
            },
        );
        let second = b.step(&f, &last);
        assert_eq!((a, n), (b, first.cum(second)));
    }

    /// Measures the median of 11 runs of `advance` over 65 536 ticks when requested (REQ-H-06);
    /// the figure goes to the report of the checkpoint.
    #[test]
    #[ignore = "measurement; run in release on request"]
    fn metire_advance() {
        let f = Facultates::default();
        let i = Indicia {
            epoch_ms: NOON,
            ..Indicia::default()
        };
        let mut times: Vec<u128> = Vec::new();
        for _ in 0..11 {
            let mut w = Mundus::genesis(1);
            let start = std::time::Instant::now();
            let n = w.advance(
                std::hint::black_box(&f),
                std::hint::black_box(&i),
                std::hint::black_box(65_536),
                std::hint::black_box(42),
            );
            times.push(start.elapsed().as_micros());
            std::hint::black_box((w, n));
        }
        times.sort_unstable();
        std::println!(
            "advance de 65 536 ticks: mediana {} µs, mínimo {} µs, máximo {} µs",
            times[5],
            times[0],
            times[10]
        );
    }
}
