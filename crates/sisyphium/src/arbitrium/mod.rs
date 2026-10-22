//! `arbitrium`: choice and consequence (level L4). The table of the eight clinamina, with the
//! band of `ascensus` where each may fire and the deltas of its three branches (REQ-L4-01,
//! REQ-L4-02); the chronicles in two languages, one module each; and the selection of the
//! visible text (REQ-L4-04, REQ-L4-08).

pub mod annales_en;
pub mod annales_pt;

use crate::hospes::Lingua;
use crate::memoria::{Memoria, RAMUS_EXSPECTANS};
use crate::natura::mix;

/// Scale of the chronicle text (`docs/07-narrativa.md` §3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Registrum {
    /// Seconds; the body and the object.
    Intimum,
    /// Minutes; an animal, a plant, a craftsman.
    Vivum,
    /// Days to years; a city, a trade, a gate.
    Civile,
    /// Centuries; a civilization, a lineage.
    Epochale,
    /// Ideas; what is lost or gained to knowledge.
    Abstractum,
}

/// Ticks of age at which each line of a chronicle becomes visible (REQ-L4-04): line `i` appears
/// when `gradus` reaches `OFFSETS[i]`. The corpus has five lines per branch, so [`revealed`]
/// stops at the fifth offset.
pub const OFFSETS: [u16; 6] = [24, 72, 144, 240, 360, 480];

/// `eventus` of the metalepsis, the inversion of agency by the observer's touch (REQ-L4-01).
pub const METALEPSIS: u8 = 11;
/// `eventus` of the recognition, the waking from the dream (REQ-L4-01, REQ-L4-11).
pub const AGNITIO: u8 = 12;
/// `eventus` of Diogenes (REQ-L4-01, REQ-L4-09).
pub const DIOGENES: u8 = 13;
/// `eventus` of Heraclitus (REQ-L4-01, REQ-L4-09).
pub const HERACLITUS: u8 = 14;
/// `eventus` of the daemon (REQ-L4-01, REQ-L4-09).
pub const DAEMON: u8 = 15;

/// Bands of `ascensus` of the eight clinamina, sixteen bits per `eventus`: event `e` holds
/// `lo | hi << 8` in bits `16·e..16·e + 16` (`docs/07-narrativa.md` §4).
const BANDS: u128 = 0x643C_E6C8_3214_BEA0_9678_4830_735A_2808;

/// Band of `ascensus` where the clinamen `eventus` may fire, as `(lo, hi)` (REQ-L4-01,
/// `docs/07-narrativa.md` §4); `(0, 0)` outside 0..=7, where the shift leaves the 128 bits of
/// the table.
#[must_use]
pub const fn band(eventus: u8) -> (u8, u8) {
    match BANDS.checked_shr((eventus as u32) << 4) {
        Some(word) => (word as u8, (word >> 8) as u8),
        None => (0, 0),
    }
}

/// Deltas `(Δonus, Δconatus, Δtranquillitas)` that the branch `ramus` of the clinamen `eventus`
/// applies when it fires (REQ-L4-02, `docs/07-narrativa.md` §4); `(0, 0, 0)` outside the table.
/// The Δonus of −30 of `eventus` 1, `ramus` 1 applies once: the state keeps no duration.
#[must_use]
pub const fn delta(eventus: u8, ramus: u8) -> (i8, i8, i8) {
    match (eventus, ramus) {
        (0, 0) => (12, 0, -2),
        (0, 1) => (6, -4, 0),
        (0, 2) => (0, -2, 2),
        (1, 0) => (5, -8, 4),
        (1, 1) => (-30, 0, 0),
        (1, 2) => (45, -10, -6),
        (2, 0) => (0, 2, 2),
        (2, 1) => (4, 0, 0),
        (2, 2) => (8, -6, -4),
        (3, 0) => (0, -1, 0),
        (3, 1) => (2, 0, 1),
        (3, 2) => (-4, 4, -2),
        (4, 0) => (0, -3, 1),
        (4, 1) => (3, -1, 0),
        (4, 2) => (20, -8, -3),
        (5, 0) => (-6, -4, 6),
        (5, 1) => (0, -2, 1),
        (5, 2) => (16, 0, -2),
        (6, 0) => (2, 6, -1),
        (6, 1) => (0, 0, 3),
        (6, 2) => (-8, 2, 4),
        (7, 0) => (1, -6, 0),
        (7, 1) => (0, 3, 2),
        (7, 2) => (6, -2, -4),
        _ => (0, 0, 0),
    }
}

/// Number of lines of a chronicle visible at the age `gradus` (REQ-L4-04): the offsets of
/// [`OFFSETS`] already reached, at most five, one per rung of the ladder of registers.
#[must_use]
pub const fn revealed(gradus: u16) -> u8 {
    let [first, second, third, fourth, fifth, _] = OFFSETS;
    if gradus < first {
        0
    } else if gradus < second {
        1
    } else if gradus < third {
        2
    } else if gradus < fourth {
        3
    } else if gradus < fifth {
        4
    } else {
        5
    }
}

/// Register of line `line` of a chronicle (`docs/07-narrativa.md` §3): line `i` is rung `i + 1`
/// of the ladder, intimate, living, civil, epochal, abstract; the lines from the fifth on are
/// abstract.
#[must_use]
pub const fn registrum_of(line: u8) -> Registrum {
    match line {
        0 => Registrum::Intimum,
        1 => Registrum::Vivum,
        2 => Registrum::Civile,
        3 => Registrum::Epochale,
        _ => Registrum::Abstractum,
    }
}

/// Index of the sixth and last line of the visible text.
const SIXTH: u8 = 5;

/// The visible text of a world: the lines of the present chronicle that its age has revealed,
/// the register of the last of them and the aphorism of the branching node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Annales {
    /// The visible lines in reading order; the entries from `count` on are empty.
    pub lines: [&'static str; 6],
    /// Number of visible lines, 0..=6.
    pub count: u8,
    /// Register of the last visible line; [`Registrum::Intimum`] when there is none.
    pub registrum: Registrum,
    /// The aphorism of the branching node, visible from the first line of a clinamen.
    pub aphorismus: Option<&'static str>,
}

impl Annales {
    /// Appends a visible line and makes its register the register of the text; a text that
    /// already has six lines gives the line the place of the sixth.
    fn push(&mut self, line: &'static str, registrum: Registrum) {
        let at = self.count.min(SIXTH);
        if let Some(slot) = self.lines.get_mut(usize::from(at)) {
            *slot = line;
        }
        self.count = at.saturating_add(1);
        self.registrum = registrum;
    }
}

/// The tables of one language, read from its module.
struct Codex {
    /// Chronicle lines by `eventus`, `ramus` and line index.
    annales: &'static [[[&'static str; 5]; 3]; 8],
    /// The aphorism of each clinamen.
    aphorismi: &'static [&'static str; 8],
    /// The lines of Diogenes, Heraclitus and the daemon.
    personae: &'static [&'static str; 3],
    /// The two lines of the recognition.
    agnitio: &'static [&'static str; 2],
    /// The line of the metalepsis.
    metalepsis: &'static str,
    /// The four lines of the interruption.
    interruptio: &'static [&'static str; 4],
}

/// The English tables.
static EN: Codex = Codex {
    annales: &annales_en::ANNALES,
    aphorismi: &annales_en::APHORISMI,
    personae: &annales_en::PERSONAE,
    agnitio: &annales_en::AGNITIO,
    metalepsis: annales_en::METALEPSIS,
    interruptio: &annales_en::INTERRUPTIO,
};

/// The Portuguese tables.
static PT: Codex = Codex {
    annales: &annales_pt::ANNALES,
    aphorismi: &annales_pt::APHORISMI,
    personae: &annales_pt::PERSONAE,
    agnitio: &annales_pt::AGNITIO,
    metalepsis: annales_pt::METALEPSIS,
    interruptio: &annales_pt::INTERRUPTIO,
};

/// The tables of a language.
const fn codex(lingua: Lingua) -> &'static Codex {
    match lingua {
        Lingua::En => &EN,
        Lingua::Pt => &PT,
    }
}

/// Selection law of the visible text (REQ-L4-04, REQ-L4-08; `docs/07-narrativa.md` §3, §5 to
/// §7), applied in this order:
///
/// 1. A clinamen (`eventus` 0..=7) with a chosen branch (`ramus` 0..=2) shows the first
///    `revealed(gradus)` lines of its branch, the register of the last of them and, from the
///    first line on, its aphorism.
/// 2. The metalepsis shows its line from the first offset, abstract; the recognition its two
///    lines at the first two offsets, intimate then abstract; Diogenes, Heraclitus and the
///    daemon their line from the first offset, living, abstract and epochal.
/// 3. A clinamen awaiting its place (`ramus` 3) and the unused events 8..=10 show no line, with
///    the register [`Registrum::Intimum`].
/// 4. `leaving` appends the line `INTERRUPTIO[k]`, k = hash(`clausurae` ∨ `eventus` ≪ 16) ∧ 3, as
///    the last line, in the place of the sixth when there are six: epochal for k in {0, 1},
///    civil for k in {2, 3}.
#[must_use]
pub fn annales(memoria: Memoria, lingua: Lingua, leaving: bool) -> Annales {
    let codex = codex(lingua);
    let eventus = memoria.eventus();
    let ramus = memoria.ramus();
    let shown = usize::from(revealed(memoria.gradus()));
    let mut text = Annales {
        lines: [""; 6],
        count: 0,
        registrum: Registrum::Intimum,
        aphorismus: None,
    };
    match eventus {
        0..=7 if ramus != RAMUS_EXSPECTANS => {
            let branch = codex
                .annales
                .get(usize::from(eventus))
                .and_then(|branches| branches.get(usize::from(ramus)));
            for (line, rung) in branch.into_iter().flatten().copied().take(shown).zip(0u8..) {
                text.push(line, registrum_of(rung));
            }
            if shown > 0 {
                text.aphorismus = codex.aphorismi.get(usize::from(eventus)).copied();
            }
        }
        METALEPSIS => {
            for line in [codex.metalepsis].into_iter().take(shown) {
                text.push(line, Registrum::Abstractum);
            }
        }
        AGNITIO => {
            let registers = [Registrum::Intimum, Registrum::Abstractum];
            for (line, registrum) in codex.agnitio.iter().copied().zip(registers).take(shown) {
                text.push(line, registrum);
            }
        }
        DIOGENES..=DAEMON => {
            let persona = eventus.saturating_sub(DIOGENES);
            let registrum = match persona {
                0 => Registrum::Vivum,
                1 => Registrum::Abstractum,
                _ => Registrum::Epochale,
            };
            let lines = codex.personae.get(usize::from(persona)).copied();
            for line in lines.into_iter().take(shown) {
                text.push(line, registrum);
            }
        }
        _ => {}
    }
    if leaving {
        let key = u64::from(memoria.clausurae()) | (u64::from(eventus) << 16);
        let k = (mix(key) & 3) as usize;
        let registrum = if k < 2 {
            Registrum::Epochale
        } else {
            Registrum::Civile
        };
        if let Some(line) = codex.interruptio.get(k).copied() {
            text.push(line, registrum);
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signum::littera::{ACUTAE, ORDO};
    use std::vec::Vec;

    /// Every string of one language module: the chronicles, the aphorisms, the lines of the
    /// personae, of the recognition and of the metalepsis, and the interruptions.
    macro_rules! texts {
        ($module:ident) => {{
            let mut all: Vec<&'static str> = $module::ANNALES
                .iter()
                .flatten()
                .flatten()
                .copied()
                .collect();
            all.extend($module::APHORISMI);
            all.extend($module::PERSONAE);
            all.extend($module::AGNITIO);
            all.push($module::METALEPSIS);
            all.extend($module::INTERRUPTIO);
            all
        }};
    }

    /// Personal and possessive pronouns, clitics included, that no English line may contain.
    const PRONOUNS_EN: [&str; 31] = [
        "I", "ME", "MY", "MINE", "MYSELF", "YOU", "YOUR", "YOURS", "HE", "HIM", "HIS", "SHE",
        "HER", "HERS", "IT", "ITS", "WE", "US", "OUR", "OURS", "THEY", "THEM", "THEIR", "THEIRS",
        "WHO", "WHOM", "WHOSE", "SOMEONE", "ANYONE", "EVERYONE", "NOBODY",
    ];

    /// Personal and possessive pronouns, clitics included, that no Portuguese line may contain.
    const PRONOUNS_PT: [&str; 37] = [
        "EU", "MIM", "COMIGO", "TU", "TI", "CONTIGO", "VOCÊ", "VOCÊS", "ELE", "ELA", "ELES",
        "ELAS", "NÓS", "VÓS", "ME", "TE", "SE", "LHE", "LHES", "VOS", "MEU", "MINHA", "MEUS",
        "MINHAS", "TEU", "TUA", "SEU", "SUA", "SEUS", "SUAS", "NOSSO", "NOSSA", "DELE", "DELA",
        "QUEM", "NINGUÉM", "ALGUÉM",
    ];

    /// A memory that holds only the event, the branch and the age of the event.
    fn memory(eventus: u8, ramus: u8, gradus: u16) -> Memoria {
        Memoria::new(0)
            .with_eventus(eventus)
            .with_ramus(ramus)
            .with_gradus(gradus)
    }

    /// Eight clinamina with their bands and branch deltas; reserved events carry their lines
    /// (REQ-L4-01).
    #[test]
    fn event_table_complete() {
        let bands = [
            (8, 40),
            (90, 115),
            (48, 72),
            (120, 150),
            (160, 190),
            (20, 50),
            (200, 230),
            (60, 100),
        ];
        for (e, b) in (0u8..).zip(bands) {
            assert_eq!(band(e), b);
        }
        assert_eq!(band(8), (0, 0));
        assert_eq!(delta(1, 1), (-30, 0, 0));
        assert_eq!(delta(1, 2), (45, -10, -6));
        assert_eq!(delta(7, 2), (6, -2, -4));
        assert_eq!(delta(9, 0), (0, 0, 0));
        for table in [annales_en::ANNALES, annales_pt::ANNALES] {
            assert!(table.iter().flatten().flatten().all(|l| !l.is_empty()));
        }
    }

    /// The bands of `ascensus` are empty outside the clinamina, and the reserved events carry the
    /// numbers of REQ-L4-01.
    #[test]
    fn event_bands_and_reserved_numbers() {
        for eventus in 8..=u8::MAX {
            assert_eq!(band(eventus), (0, 0), "eventus {eventus}");
        }
        for eventus in 0..8 {
            let (lo, hi) = band(eventus);
            assert!(lo < hi, "eventus {eventus}");
        }
        assert_eq!(
            [METALEPSIS, AGNITIO, DIOGENES, HERACLITUS, DAEMON],
            [11, 12, 13, 14, 15]
        );
    }

    /// The 24 branch deltas of `docs/07-narrativa.md` §4 and the neutral delta outside the table
    /// (REQ-L4-02).
    #[test]
    fn branch_deltas() {
        let table: [[(i8, i8, i8); 3]; 8] = [
            [(12, 0, -2), (6, -4, 0), (0, -2, 2)],
            [(5, -8, 4), (-30, 0, 0), (45, -10, -6)],
            [(0, 2, 2), (4, 0, 0), (8, -6, -4)],
            [(0, -1, 0), (2, 0, 1), (-4, 4, -2)],
            [(0, -3, 1), (3, -1, 0), (20, -8, -3)],
            [(-6, -4, 6), (0, -2, 1), (16, 0, -2)],
            [(2, 6, -1), (0, 0, 3), (-8, 2, 4)],
            [(1, -6, 0), (0, 3, 2), (6, -2, -4)],
        ];
        for (eventus, row) in (0u8..).zip(table) {
            for (ramus, expected) in (0u8..).zip(row) {
                assert_eq!(delta(eventus, ramus), expected, "{eventus}.{ramus}");
            }
        }
        for eventus in 0..=u8::MAX {
            for ramus in 0..=u8::MAX {
                if eventus >= 8 || ramus >= 3 {
                    assert_eq!(delta(eventus, ramus), (0, 0, 0), "{eventus}.{ramus}");
                }
            }
        }
    }

    /// Line i of a chronicle becomes visible at offset i; five lines per branch (REQ-L4-04).
    #[test]
    fn annales_reveal_offsets() {
        assert_eq!(OFFSETS, [24, 72, 144, 240, 360, 480]);
        let cases = [
            (0, 0),
            (23, 0),
            (24, 1),
            (71, 1),
            (72, 2),
            (143, 2),
            (144, 3),
            (239, 3),
            (240, 4),
            (359, 4),
            (360, 5),
            (599, 5),
            (1023, 5),
        ];
        for (g, n) in cases {
            assert_eq!(revealed(g), n, "gradus {g}");
        }
        let m = Memoria::new(0)
            .with_eventus(2)
            .with_ramus(1)
            .with_gradus(150);
        let a = annales(m, Lingua::En, false);
        assert_eq!(a.count, 3);
        assert_eq!(a.lines[..3], annales_en::ANNALES[2][1][..3]);
        assert_eq!(a.registrum, Registrum::Civile);
        assert_eq!(a.aphorismus, Some(annales_en::APHORISMI[2]));
        assert_eq!(
            annales(m.with_gradus(23), Lingua::Pt, false).aphorismus,
            None
        );
    }

    /// Line i of a chronicle is rung i + 1 of the ladder of registers; every rung from the fifth on
    /// is abstract (`docs/07-narrativa.md` §3).
    #[test]
    fn registrum_ladder() {
        let ladder = [
            Registrum::Intimum,
            Registrum::Vivum,
            Registrum::Civile,
            Registrum::Epochale,
            Registrum::Abstractum,
        ];
        for (line, expected) in (0u8..).zip(ladder) {
            assert_eq!(registrum_of(line), expected, "line {line}");
        }
        assert_eq!(registrum_of(5), Registrum::Abstractum);
        assert_eq!(registrum_of(u8::MAX), Registrum::Abstractum);
    }

    /// The visible lines of a clinamen are the first `revealed(gradus)` lines of its branch in the
    /// language asked for; the register is that of the last of them and the aphorism appears with
    /// the first line (REQ-L4-04).
    #[test]
    fn annales_follow_language() {
        let ages = [
            0, 1, 23, 24, 71, 72, 143, 144, 239, 240, 359, 360, 479, 480, 600, 1023,
        ];
        for (lingua, chronicles, aphorisms) in [
            (Lingua::En, annales_en::ANNALES, annales_en::APHORISMI),
            (Lingua::Pt, annales_pt::ANNALES, annales_pt::APHORISMI),
        ] {
            for eventus in 0u8..8 {
                for ramus in 0u8..3 {
                    for gradus in ages {
                        let a = annales(memory(eventus, ramus, gradus), lingua, false);
                        let shown = usize::from(revealed(gradus));
                        let branch = chronicles[usize::from(eventus)][usize::from(ramus)];
                        let at = std::format!("{lingua:?} {eventus}.{ramus} at {gradus}");
                        assert_eq!(usize::from(a.count), shown, "{at}");
                        assert_eq!(a.lines[..shown], branch[..shown], "{at}");
                        assert!(a.lines[shown..].iter().all(|l| l.is_empty()), "{at}");
                        assert_eq!(a.registrum, registrum_of(a.count.saturating_sub(1)), "{at}");
                        let aphorism = (gradus >= 24).then_some(aphorisms[usize::from(eventus)]);
                        assert_eq!(a.aphorismus, aphorism, "{at}");
                    }
                }
            }
        }
    }

    /// The reserved events carry their own lines on the same schedule: the metalepsis and each
    /// persona one line from age 24, the recognition a second line from age 72; none carries an
    /// aphorism (REQ-L4-01, `docs/07-narrativa.md` §5 to §7).
    #[test]
    fn reserved_events_lines() {
        for (lingua, personae, agnitio, metalepsis) in [
            (
                Lingua::En,
                annales_en::PERSONAE,
                annales_en::AGNITIO,
                annales_en::METALEPSIS,
            ),
            (
                Lingua::Pt,
                annales_pt::PERSONAE,
                annales_pt::AGNITIO,
                annales_pt::METALEPSIS,
            ),
        ] {
            let at = |eventus: u8, gradus: u16| annales(memory(eventus, 0, gradus), lingua, false);
            for eventus in [METALEPSIS, AGNITIO, DIOGENES, HERACLITUS, DAEMON] {
                let early = at(eventus, 23);
                assert_eq!(
                    (early.count, early.registrum, early.aphorismus),
                    (0, Registrum::Intimum, None),
                    "eventus {eventus}"
                );
                assert_eq!(at(eventus, 1023).aphorismus, None, "eventus {eventus}");
            }
            let meta = at(METALEPSIS, 24);
            assert_eq!(
                (meta.count, meta.lines[0], meta.registrum),
                (1, metalepsis, Registrum::Abstractum)
            );
            assert_eq!(at(METALEPSIS, 1023), meta);
            let first = at(AGNITIO, 71);
            assert_eq!(
                (first.count, first.lines[0], first.registrum),
                (1, agnitio[0], Registrum::Intimum)
            );
            let both = at(AGNITIO, 72);
            assert_eq!(both.count, 2);
            assert_eq!(both.lines[..2], agnitio);
            assert_eq!(both.registrum, Registrum::Abstractum);
            assert_eq!(at(AGNITIO, 1023), both);
            for (line, eventus, registrum) in [
                (0, DIOGENES, Registrum::Vivum),
                (1, HERACLITUS, Registrum::Abstractum),
                (2, DAEMON, Registrum::Epochale),
            ] {
                let persona = at(eventus, 24);
                assert_eq!(
                    (persona.count, persona.lines[0], persona.registrum),
                    (1, personae[line], registrum),
                    "eventus {eventus}"
                );
                assert_eq!(at(eventus, 1023), persona);
            }
        }
    }

    /// A clinamen awaiting its place and the unused events 8..=10 show no line and no aphorism at
    /// any age (REQ-L4-04).
    #[test]
    fn annales_silent_events() {
        let silent = Annales {
            lines: [""; 6],
            count: 0,
            registrum: Registrum::Intimum,
            aphorismus: None,
        };
        for lingua in [Lingua::En, Lingua::Pt] {
            for gradus in [0, 24, 72, 360, 600, 1023] {
                for eventus in 0..8 {
                    let m = memory(eventus, crate::memoria::RAMUS_EXSPECTANS, gradus);
                    assert_eq!(annales(m, lingua, false), silent, "eventus {eventus}");
                }
                for eventus in 8..=10 {
                    for ramus in 0..4 {
                        let m = memory(eventus, ramus, gradus);
                        assert_eq!(annales(m, lingua, false), silent, "eventus {eventus}");
                    }
                }
            }
        }
    }

    /// The interruption line is `INTERRUPTIO[k]` with k = hash(`clausurae` ∨ `eventus` ≪ 16) ∧ 3,
    /// appended as the last line; epochal for k in {0, 1}, civil for k in {2, 3} (REQ-L4-08).
    #[test]
    fn interruption_line() {
        for (lingua, lines) in [
            (Lingua::En, annales_en::INTERRUPTIO),
            (Lingua::Pt, annales_pt::INTERRUPTIO),
        ] {
            let mut seen = [false; 4];
            for clausurae in (0..0x4000u16).step_by(37) {
                for eventus in 0..16u8 {
                    let m = memory(eventus, 1, 600).with_clausurae(clausurae);
                    let calm = annales(m, lingua, false);
                    let left = annales(m, lingua, true);
                    let key = u64::from(clausurae) | (u64::from(eventus) << 16);
                    let k = (mix(key) & 3) as usize;
                    seen[k] = true;
                    let n = usize::from(calm.count);
                    assert_eq!(left.count, calm.count + 1, "{clausurae} {eventus}");
                    assert_eq!(left.lines[..n], calm.lines[..n], "{clausurae} {eventus}");
                    assert_eq!(left.lines[n], lines[k], "{clausurae} {eventus}");
                    let registrum = if k < 2 {
                        Registrum::Epochale
                    } else {
                        Registrum::Civile
                    };
                    assert_eq!(left.registrum, registrum, "{clausurae} {eventus}");
                    assert_eq!(left.aphorismus, calm.aphorismus, "{clausurae} {eventus}");
                }
            }
            assert_eq!(seen, [true; 4], "every interruption line is reachable");
        }
    }

    /// A line appended to a text of six lines replaces the sixth (REQ-L4-08).
    #[test]
    fn full_text_replaces_sixth_line() {
        let mut text = Annales {
            lines: ["A", "B", "C", "D", "E", "F"],
            count: 6,
            registrum: Registrum::Abstractum,
            aphorismus: None,
        };
        text.push("G", Registrum::Civile);
        assert_eq!(text.lines, ["A", "B", "C", "D", "E", "G"]);
        assert_eq!((text.count, text.registrum), (6, Registrum::Civile));
    }

    /// Every line fits 40 glyphs of the font, in uppercase (`docs/07-narrativa.md` §8).
    #[test]
    fn annales_lines_within_width() {
        for text in texts!(annales_en).into_iter().chain(texts!(annales_pt)) {
            assert!(text.chars().count() <= 40, "{text} is wider than 40 glyphs");
            for c in text.chars() {
                assert!(
                    ORDO.contains(c) || ACUTAE.contains(c),
                    "{c} of {text} has no glyph"
                );
                assert!(!c.is_lowercase(), "{c} of {text} is lowercase");
            }
        }
    }

    /// No personal or possessive pronoun, clitics included (`docs/07-narrativa.md` §3).
    #[test]
    fn annales_impersonal() {
        const MARKS: [char; 9] = ['.', ',', ';', ':', '!', '?', '"', '(', ')'];
        for (texts, banned) in [
            (texts!(annales_en), PRONOUNS_EN.as_slice()),
            (texts!(annales_pt), PRONOUNS_PT.as_slice()),
        ] {
            for text in texts {
                for word in text.split([' ', '-']).map(|w| w.trim_matches(MARKS)) {
                    assert!(!banned.contains(&word), "{word} in {text}");
                }
            }
        }
    }

    /// Aphorisms read `□X → ◊Y`, one word each side, at most 16 glyphs (REQ-L4-12).
    #[test]
    fn aphorism_format() {
        for aphorisms in [annales_en::APHORISMI, annales_pt::APHORISMI] {
            for aphorism in aphorisms {
                let parts: Vec<&str> = aphorism.split(" → ").collect();
                assert_eq!(parts.len(), 2, "{aphorism}");
                assert!(parts[0].starts_with('□'), "{aphorism}");
                assert!(parts[1].starts_with('◊'), "{aphorism}");
                for part in parts {
                    assert!(part.chars().count() > 1, "{aphorism}");
                    assert!(!part.contains(' '), "{aphorism}");
                }
                assert!(aphorism.chars().count() <= 16, "{aphorism}");
            }
        }
    }

    /// Every string of the six tables of both languages carries text and no stray space: the
    /// reserved events and the interruption have their lines as the clinamina do (REQ-L4-01).
    #[test]
    fn annales_tables_filled() {
        for text in texts!(annales_en).into_iter().chain(texts!(annales_pt)) {
            assert!(!text.is_empty(), "an empty string in the tables");
            assert_eq!(text, text.trim(), "{text:?} has a stray space");
        }
    }

    /// Reserved events, the interruption and arbitrary memories select lines by the law above.
    #[test]
    fn annales_arbitrary_memoria() {
        let agn = Memoria::new(0).with_eventus(AGNITIO).with_ramus(0);
        assert_eq!(
            annales(agn.with_gradus(30), Lingua::En, false).registrum,
            Registrum::Intimum
        );
        let late = annales(agn.with_gradus(80), Lingua::En, false);
        assert_eq!((late.count, late.registrum), (2, Registrum::Abstractum));
        let her = annales(
            Memoria::new(0).with_eventus(HERACLITUS).with_gradus(30),
            Lingua::Pt,
            false,
        );
        assert_eq!(
            (her.lines[0], her.registrum),
            (annales_pt::PERSONAE[1], Registrum::Abstractum)
        );
        let leave = annales(
            Memoria::new(0).with_eventus(4).with_ramus(3),
            Lingua::En,
            true,
        );
        assert_eq!(leave.count, 1);
        assert!(annales_en::INTERRUPTIO.contains(&leave.lines[0]));
        for bits in [0u64, u64::MAX, 0x0000_0000_FFFF_FFFF, 0x8000_0000_0029_0000] {
            for leaving in [false, true] {
                let a = annales(Memoria::new(bits), Lingua::Pt, leaving);
                assert!(a.count <= 6);
            }
        }
    }

    /// Over arbitrary memories the text never exceeds six lines, the lines before `count` are
    /// not empty and the ones after it are, and a silent event shows at most the interruption
    /// (REQ-L4-04).
    #[test]
    fn annales_invariants() {
        let mut x = 0x2545_F491_4F6C_DD1Du64;
        for _ in 0..20_000u32 {
            x = mix(x);
            let m = Memoria::new(x);
            let silent = matches!(m.eventus(), 8..=10)
                || (m.eventus() < 8 && m.ramus() == crate::memoria::RAMUS_EXSPECTANS);
            for lingua in [Lingua::En, Lingua::Pt] {
                for leaving in [false, true] {
                    let a = annales(m, lingua, leaving);
                    let n = usize::from(a.count);
                    assert!(n <= 6, "{x:#x}");
                    assert!(a.lines[..n].iter().all(|l| !l.is_empty()), "{x:#x}");
                    assert!(a.lines[n..].iter().all(|l| l.is_empty()), "{x:#x}");
                    assert!(n >= usize::from(leaving), "{x:#x}");
                    if silent {
                        assert_eq!(n, usize::from(leaving), "{x:#x}");
                        assert_eq!(a.aphorismus, None, "{x:#x}");
                    }
                }
            }
        }
    }
}
