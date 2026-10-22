//! `arbitrium`: choice and consequence (level L4). The table of the eight clinamina, with the
//! band of `ascensus` where each may fire and the deltas of its three branches (REQ-L4-01,
//! REQ-L4-02); the chronicles in two languages, one module each; the selection of the visible
//! text (REQ-L4-04, REQ-L4-08); and the laws that change the two state words in each tick: the
//! pace of the ascent (REQ-L4-05), the continuous laws of effort, weight and serenity
//! (REQ-L4-06), the clinamina and their closure (REQ-L4-02, REQ-L4-03), the personae
//! (REQ-L4-09), the closure and the interruption of a cycle (REQ-L4-07, REQ-L4-08), the mode of
//! the world and the recognition (REQ-L4-10, REQ-L4-11), the curvature (REQ-L4-11) and the
//! contemplation (REQ-L4-05, REQ-L4-06).

pub mod annales_en;
pub mod annales_pt;

use crate::hospes::{Lingua, Nuntius};
use crate::memoria::{Memoria, RAMUS_EXSPECTANS};
use crate::natura::{R_INF, logistic_r, mix, wind};
use crate::verbum::{Modus, Verbum};

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

/// Number of clinamina: `eventus` 0..=7 (REQ-L4-01). The events from 8 on are special: the
/// unused 8..=10, the metalepsis, the recognition and the personae.
const CLINAMINA: u8 = 8;

/// Age in ticks at which a clinamen that has chosen its branch closes (REQ-L4-03).
const CLINAMEN_SPAN: u16 = 600;

/// Age in ticks at which a special event ends (REQ-L4-03, `docs/07-narrativa.md` §2).
const SPECIAL_SPAN: u16 = 240;

/// Ticks of sleep after the recognition: the figure lies down while `gradus` is below this age
/// (REQ-L3-02).
const SLEEP_SPAN: u16 = 64;

/// Largest age: `gradus` has ten bits and saturates at 2¹⁰ − 1 (REQ-L4-03).
const GRADUS_MAX: u16 = 1023;

/// Largest serenity: `tranquillitas` has six bits, 2⁶ − 1 (REQ-L4-06).
const TRANQUILLITAS_MAX: u8 = 63;

/// Applies the changes `(onus, conatus, tranquillitas)` to the word by saturating sum in the
/// domain of each field: `onus` and `conatus` in 0..=255, `tranquillitas` in 0..=63 (REQ-L4-02,
/// REQ-L4-06, REQ-L4-07, REQ-L4-09, REQ-L4-11).
const fn offset(v: Verbum, onus: i8, conatus: i8, tranquillitas: i8) -> Verbum {
    let calm = v.tranquillitas().saturating_add_signed(tranquillitas);
    v.with_onus(v.onus().saturating_add_signed(onus))
        .with_conatus(v.conatus().saturating_add_signed(conatus))
        .with_tranquillitas(if calm > TRANQUILLITAS_MAX {
            TRANQUILLITAS_MAX
        } else {
            calm
        })
}

/// Whether the generator passes the gate of the bits `mask`: (`semen` ∧ `mask`) = 0, a chance of
/// 1 in `mask` + 1 when `mask` is one less than a power of two (REQ-L4-06, REQ-L4-09,
/// REQ-L4-10).
const fn gate(semen: u16, mask: u16) -> bool {
    semen & mask == 0
}

/// The threshold T of the pace (REQ-L4-05, REQ-L1-08): clamp(`conatus` − ⌊`onus`/2⌋, 16, 255),
/// halved in wind and shifted right by ⌊(`ascensus` − 240)/4⌋ from `ascensus` 240 on, so that
/// the likelihood of a step halves every four units near the summit (Zeigarnik, 1927). The upper
/// bound of the clamp is the largest `conatus`; the threshold is never below 1.
#[must_use]
pub const fn pace_threshold(v: Verbum) -> u16 {
    let effort = (v.conatus() as u16).saturating_sub((v.onus() >> 1) as u16);
    let base = if effort < 16 { 16 } else { effort };
    let halvings =
        (wind(v.automatum()) as u32).wrapping_add((v.ascensus().saturating_sub(240) >> 2) as u32);
    base.wrapping_shr(halvings)
}

/// Whether the figure takes a step in this tick (REQ-L4-05, REQ-L1-08): the fifteen low bits of
/// `semen` fall below [`pace_threshold`]. There is no step at `ascensus` 255, where the cycle
/// closes, and the last step, from 254 to 255, needs `conatus` of at least 160 and calm air.
/// The step is withheld in contemplation (REQ-L4-05); the caller applies [`contemplating`].
#[must_use]
pub const fn paces(v: Verbum) -> bool {
    let last = v.ascensus() == 254 && (v.conatus() < 160 || wind(v.automatum()));
    v.ascensus() < 255 && !last && (v.semen() & 0x7FFF) < pace_threshold(v)
}

/// The continuous laws of one tick (REQ-L4-06), each gated by the low bits of `semen`:
///
/// - `conatus` −1 on a tick with a step, when (`semen` ∧ 3) = 0;
/// - `conatus` +1 on a tick without a step, when (`semen` ∧ 0x1FF) = 0 in ascent or
///   (`semen` ∧ 0x1F) = 0 in contemplation (Quies, persona or sleep);
/// - `conatus` −1 when `seism` ≥ 128 and (`semen` ∧ 3) = 0;
/// - `onus` one unit toward 32, rising from below and falling from above, when
///   (`semen` ∧ 0x3F) = 0;
/// - `tranquillitas` +1 when `idle_s` ≥ 600 and (`semen` ∧ 0xFF) = 0, and −1 when
///   `seism` ≥ 64.
///
/// The changes of one field in the same tick add up before one saturating application in the
/// domain of the field, so that opposite laws cancel at the edge of a domain.
#[must_use]
pub const fn continuous(
    v: Verbum,
    stepped: bool,
    contemplating: bool,
    seism: u8,
    idle_s: u32,
) -> Verbum {
    let semen = v.semen();
    let quarter = gate(semen, 3);
    let rest = if contemplating { 0x1F } else { 0x1FF };
    let spent = stepped && quarter;
    let rested = !stepped && gate(semen, rest);
    let strained = seism >= 128 && quarter;
    let effort = (rested as i8)
        .wrapping_sub(spent as i8)
        .wrapping_sub(strained as i8);
    let toward = ((v.onus() < 32) as i8).wrapping_sub((v.onus() > 32) as i8);
    let weight = if gate(semen, 0x3F) { toward } else { 0 };
    let serene = idle_s >= 600 && gate(semen, 0xFF);
    let agitated = seism >= 64;
    let calm = (serene as i8).wrapping_sub(agitated as i8);
    offset(v, weight, effort, calm)
}

/// Whether the clinamen of the present event fires in this tick (REQ-L4-02): the event is a
/// clinamen (`eventus` 0..=7) that awaits its place (`ramus` 3), `ascensus` lies in its band
/// ([`band`]) and the generator passes the gate (`semen` ∧ 0xFF) < 24, a chance of 24 in 256.
#[must_use]
pub const fn clinamen_fires(v: Verbum, m: Memoria) -> bool {
    let (lo, hi) = band(m.eventus());
    m.eventus() < CLINAMINA
        && m.ramus() == RAMUS_EXSPECTANS
        && v.ascensus() >= lo
        && v.ascensus() <= hi
        && v.semen() & 0xFF < 24
}

/// Fires the clinamen (REQ-L4-02; Lucretius, II, 216–293): the branch is (`semen` ≫ 12) mod 3,
/// the age restarts and the deltas of the branch ([`delta`]) apply by saturating sum in the
/// domains of the fields. The event stays the same.
#[must_use]
pub const fn fire(v: Verbum, m: Memoria) -> (Verbum, Memoria) {
    let ramus = ((v.semen() >> 12) % 3) as u8;
    let (onus, conatus, tranquillitas) = delta(m.eventus(), ramus);
    (
        offset(v, onus, conatus, tranquillitas),
        m.with_ramus(ramus).with_gradus(0),
    )
}

/// Ages the present event by one tick, saturating at the largest `gradus` (REQ-L4-03).
#[must_use]
pub const fn age(m: Memoria) -> Memoria {
    let next = m.gradus().saturating_add(1);
    m.with_gradus(if next > GRADUS_MAX { GRADUS_MAX } else { next })
}

/// Closes a clinamen that has chosen its branch (REQ-L4-03): the next event is
/// (`eventus` + 1 + (`semen` ≫ 8) mod 7) mod 8, which is never the event itself, it awaits its
/// place (`ramus` 3) and its age restarts.
#[must_use]
pub const fn close(v: Verbum, m: Memoria) -> Memoria {
    let skipped = (v.semen() >> 8) % 7;
    let next = (m.eventus() as u16).wrapping_add(1).wrapping_add(skipped) & 7;
    m.with_eventus(next as u8)
        .with_ramus(RAMUS_EXSPECTANS)
        .with_gradus(0)
}

/// Ends a special event (REQ-L4-03): the next event is the clinamen (`semen` ≫ 8) ∧ 7, which
/// awaits its place (`ramus` 3), and its age restarts.
#[must_use]
pub const fn release(v: Verbum, m: Memoria) -> Memoria {
    m.with_eventus(((v.semen() >> 8) & 7) as u8)
        .with_ramus(RAMUS_EXSPECTANS)
        .with_gradus(0)
}

/// The persona that enters in this tick, if any (REQ-L4-09; Diogenes Laertius, VI; Heraclitus,
/// DK B12 and B91). Nothing enters under the observer's eye, so Diogenes and Heraclitus need
/// `idle_s` ≥ 60 or occlusion. All three need a clinamen awaiting its place (`ramus` 3) and the
/// gate (`semen` ∧ 0x1FF) = 0, a chance of 1 in 512:
///
/// - Diogenes ([`DIOGENES`]) at circadian `phase` 112..=144 with `ascensus` 96..=160;
/// - Heraclitus ([`HERACLITUS`]) at `phase` 184..=200 with `ascensus` 64..=128;
/// - the daemon ([`DAEMON`]) at `phase` < 32 with `ascensus` ≥ 250 and `idle_s` ≥ 1800, which
///   occlusion alone does not replace.
///
/// The three hours do not overlap, so at most one persona qualifies.
#[must_use]
pub const fn persona(v: Verbum, m: Memoria, phase: u8, idle_s: u32, occluded: bool) -> Option<u8> {
    let open = gate(v.semen(), 0x1FF) && m.ramus() == RAMUS_EXSPECTANS;
    let unwatched = idle_s >= 60 || occluded;
    match (phase, v.ascensus()) {
        (112..=144, 96..=160) if open && unwatched => Some(DIOGENES),
        (184..=200, 64..=128) if open && unwatched => Some(HERACLITUS),
        (0..=31, 250..=255) if open && idle_s >= 1800 => Some(DAEMON),
        _ => None,
    }
}

/// A persona enters (REQ-L4-09): it takes the event with `ramus` 0 and the age 0, and acts on
/// the word by saturating sum in the domains of the fields:
///
/// - Diogenes: `ascensus` −32, `onus` +8;
/// - Heraclitus: `onus` −16, `curvatura` +8, saturating in `i8`;
/// - the daemon: `modus` becomes `Modus::AmorFati` when `conatus` ≥ 200, otherwise `onus` +16.
///
/// A number outside [`DIOGENES`]..=[`DAEMON`] is no persona and changes nothing.
#[must_use]
pub const fn visit(v: Verbum, m: Memoria, persona: u8) -> (Verbum, Memoria) {
    let changed = match persona {
        DIOGENES => offset(v, 8, 0, 0).with_ascensus(v.ascensus().saturating_sub(32)),
        HERACLITUS => offset(v, -16, 0, 0).with_curvatura(v.curvatura().saturating_add(8)),
        DAEMON if v.conatus() >= 200 => v.with_modus(Modus::AmorFati),
        DAEMON => offset(v, 16, 0, 0),
        _ => return (v, m),
    };
    (
        changed,
        m.with_eventus(persona).with_ramus(0).with_gradus(0),
    )
}

/// The event of one tick (REQ-L4-02, REQ-L4-03, REQ-L4-09), in this order:
///
/// 1. the present event ages ([`age`]);
/// 2. a clinamen that has chosen its branch (`eventus` < 8, `ramus` ≠ 3) closes at `gradus` ≥
///    600 ([`close`]); a special event (`eventus` ≥ 8) ends at `gradus` ≥ 240 ([`release`]);
/// 3. if the clinamen fires ([`clinamen_fires`]) it fires ([`fire`]), announced by
///    [`Nuntius::CLINAMEN`]; otherwise, if a persona enters ([`persona`]) it enters ([`visit`]),
///    announced by [`Nuntius::PERSONA`]. A clinamen and a persona that qualify in the same tick
///    leave the tick to the clinamen.
///
/// Only `eventus`, `ramus` and `gradus` of the memory change, and the word changes only with an
/// announcement.
#[must_use]
pub const fn event(
    v: Verbum,
    m: Memoria,
    phase: u8,
    idle_s: u32,
    occluded: bool,
) -> (Verbum, Memoria, Nuntius) {
    let aged = age(m);
    let settled = if aged.eventus() < CLINAMINA
        && aged.ramus() != RAMUS_EXSPECTANS
        && aged.gradus() >= CLINAMEN_SPAN
    {
        close(v, aged)
    } else if aged.eventus() >= CLINAMINA && aged.gradus() >= SPECIAL_SPAN {
        release(v, aged)
    } else {
        aged
    };
    if clinamen_fires(v, settled) {
        let (fired, memory) = fire(v, settled);
        (fired, memory, Nuntius::CLINAMEN)
    } else if let Some(entering) = persona(v, settled, phase, idle_s, occluded) {
        let (visited, memory) = visit(v, settled, entering);
        (visited, memory, Nuntius::PERSONA)
    } else {
        (v, settled, Nuntius::NIHIL)
    }
}

/// Closes the cycle of the climb (REQ-L4-07), when `ascensus` is 255: the ring of closures
/// shifts one bit within its 14 bits and the new bit 0 is 0, `ascensus` returns to 0, `onus`
/// falls by 8 and `conatus` rises by 24 by saturating sum; under `Modus::AmorFati`, `tranquillitas`
/// rises by 4 and the mode returns to `Modus::Vigilia`. The caller decides when the cycle
/// closes.
#[must_use]
pub const fn closure(v: Verbum, m: Memoria) -> (Verbum, Memoria) {
    let affirmed = matches!(v.modus(), Modus::AmorFati);
    let closed = offset(v, -8, 24, if affirmed { 4 } else { 0 })
        .with_ascensus(0)
        .with_modus(if affirmed { Modus::Vigilia } else { v.modus() });
    (closed, m.with_clausurae((m.clausurae() << 1) & 0x3FFF))
}

/// Marks the present cycle as interrupted by the observer's return (REQ-L4-08): bit 0 of the
/// ring of closures becomes 1.
#[must_use]
pub const fn interrupt(m: Memoria) -> Memoria {
    m.with_clausurae(m.clausurae() | 1)
}

/// Whether the dream may begin (REQ-L1-04): the parameter of the logistic map ([`logistic_r`])
/// reaches r∞ ([`R_INF`], Feigenbaum, 1978), which happens at night after 2 820 s of idleness.
#[must_use]
pub const fn somnium_eligible(nocturnal: bool, idle_s: u32) -> bool {
    logistic_r(nocturnal, idle_s).0 >= R_INF.0
}

/// The recognition, the waking from the dream (REQ-L4-11; Aristotle, Poetics 11): `ascensus` and
/// `curvatura` return to 0, `conatus` rises by 32 by saturating sum, the mode becomes
/// `Modus::Vigilia`, and the event is [`AGNITIO`] with `ramus` 0 and the age 0.
#[must_use]
pub const fn agnitio(v: Verbum, m: Memoria) -> (Verbum, Memoria) {
    let awake = offset(v, 0, 32, 0)
        .with_ascensus(0)
        .with_curvatura(0)
        .with_modus(Modus::Vigilia);
    (awake, m.with_eventus(AGNITIO).with_ramus(0).with_gradus(0))
}

/// The law of the mode in one tick (REQ-L4-10), in this precedence:
///
/// 1. `energy` < 39 gives `Modus::Quies`;
/// 2. a dream (`Modus::Somnium`) ends in the recognition ([`agnitio`], announced by
///    [`Nuntius::AGNITIO`]) when `seism` ≥ 16, when it is not night or when the observer
///    `touched`; without a stimulus it persists, with no limit of duration;
/// 3. `Modus::Vigilia` becomes a dream when it is night, `seism` < 2, `idle_s` ≥ 1200, the
///    parameter of the logistic map reaches r∞ ([`somnium_eligible`]) and the generator passes
///    the gate (`semen` ∧ 0xFFF) = 0, a chance of 1 in 4096; the night and the idleness of
///    1200 s are implied by r ≥ r∞, which needs the night and 2 820 s, and stay as the
///    requirement states them;
/// 4. `Modus::AmorFati` holds until the cycle closes;
/// 5. otherwise the mode is `Modus::Vigilia`, which `Modus::Quies` returns to when the energy
///    allows.
#[must_use]
pub const fn modus_law(
    v: Verbum,
    m: Memoria,
    energy: u8,
    seism: u8,
    nocturnal: bool,
    touched: bool,
    idle_s: u32,
) -> (Verbum, Memoria, Nuntius) {
    if energy < 39 {
        return (v.with_modus(Modus::Quies), m, Nuntius::NIHIL);
    }
    match v.modus() {
        Modus::Somnium if seism >= 16 || !nocturnal || touched => {
            let (awake, memory) = agnitio(v, m);
            (awake, memory, Nuntius::AGNITIO)
        }
        Modus::Somnium | Modus::AmorFati => (v, m, Nuntius::NIHIL),
        Modus::Quies => (v.with_modus(Modus::Vigilia), m, Nuntius::NIHIL),
        Modus::Vigilia => {
            let dreams = nocturnal
                && seism < 2
                && idle_s >= 1200
                && somnium_eligible(nocturnal, idle_s)
                && gate(v.semen(), 0xFFF);
            let next = if dreams {
                Modus::Somnium
            } else {
                Modus::Vigilia
            };
            (v.with_modus(next), m, Nuntius::NIHIL)
        }
    }
}

/// The curvature of space in one tick (REQ-L4-11): in `Modus::Somnium` it falls by 3 down to
/// −64, in `Modus::AmorFati` it rises by 2 up to +64, and in `Modus::Vigilia` and `Modus::Quies`
/// it shrinks to three quarters, truncated toward 0. The sums saturate in `i8`; a value beyond
/// ±64 enters the operational range at the bound the mode moves toward.
#[must_use]
pub const fn curvature(v: Verbum) -> Verbum {
    let kappa = v.curvatura();
    let next = match v.modus() {
        Modus::Somnium => {
            let fallen = kappa.saturating_sub(3);
            if fallen < -64 { -64 } else { fallen }
        }
        Modus::AmorFati => {
            let risen = kappa.saturating_add(2);
            if risen > 64 { 64 } else { risen }
        }
        Modus::Vigilia | Modus::Quies => ((kappa as i16).wrapping_mul(3) / 4) as i8,
    };
    v.with_curvatura(next)
}

/// Whether the figure sleeps (REQ-L3-02, REQ-L4-06): the event is the recognition and its age is
/// below 64 ticks.
#[must_use]
pub const fn asleep(m: Memoria) -> bool {
    m.eventus() == AGNITIO && m.gradus() < SLEEP_SPAN
}

/// Whether the figure contemplates (REQ-L4-05, REQ-L4-06): the mode is `Modus::Quies`, a persona
/// is present (`eventus` [`DIOGENES`]..=[`DAEMON`]) or the figure sleeps ([`asleep`]). A figure
/// that contemplates takes no step.
#[must_use]
pub const fn contemplating(v: Verbum, m: Memoria) -> bool {
    matches!(v.modus(), Modus::Quies) || matches!(m.eventus(), DIOGENES..=DAEMON) || asleep(m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signum::littera::{ACUTAE, ORDO};
    use crate::spatium::Fx;
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
    /// aphorism; the lines show at any `ramus` (REQ-L4-01, `docs/07-narrativa.md` §5 to §7).
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
            for ramus in 1..=3u8 {
                for eventus in METALEPSIS..=DAEMON {
                    for gradus in [0, 23, 24, 71, 72, 1023] {
                        assert_eq!(
                            annales(memory(eventus, ramus, gradus), lingua, false),
                            at(eventus, gradus),
                            "eventus {eventus} ramus {ramus} gradus {gradus}"
                        );
                    }
                }
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

    /// T = clamp(conatus − onus/2, 16, 255) (REQ-L4-05).
    #[test]
    fn pace_threshold() {
        let v = Verbum::new(0)
            .with_conatus(192)
            .with_onus(32)
            .with_automatum(0x10);
        assert_eq!(super::pace_threshold(v), 176);
        assert_eq!(super::pace_threshold(v.with_conatus(10)), 16);
        assert_eq!(super::pace_threshold(v.with_conatus(255).with_onus(0)), 255);
        assert_eq!(super::pace_threshold(v.with_conatus(0).with_onus(255)), 16);
    }

    /// Six living cells make wind and halve the threshold (REQ-L1-08).
    #[test]
    fn wind_halves_pace() {
        let v = Verbum::new(0).with_conatus(192).with_onus(32);
        assert_eq!(super::pace_threshold(v.with_automatum(0b0001_1111)), 176);
        assert_eq!(super::pace_threshold(v.with_automatum(0b0011_1111)), 88);
    }

    /// Near the summit the threshold halves every four steps; the last step needs effort and calm
    /// air (REQ-L4-05).
    #[test]
    fn zeigarnik_asymptote() {
        let top = Verbum::new(0)
            .with_conatus(200)
            .with_onus(0)
            .with_automatum(0x10);
        for (a, t) in [
            (239, 200),
            (240, 200),
            (244, 100),
            (248, 50),
            (252, 25),
            (255, 25),
        ] {
            assert_eq!(
                super::pace_threshold(top.with_ascensus(a)),
                t,
                "ascensus {a}"
            );
        }
        let ready = top.with_ascensus(254).with_semen(1);
        assert!(paces(ready));
        assert!(!paces(ready.with_conatus(159)));
        assert!(!paces(ready.with_automatum(0xFF)));
        assert!(!paces(top.with_ascensus(255).with_semen(1)));
        assert!(!paces(top.with_ascensus(10).with_semen(200)));
    }

    /// The continuous laws of one tick, gated by the generator's low bits (REQ-L4-06).
    #[test]
    fn continuous_laws() {
        let b = Verbum::new(0)
            .with_conatus(100)
            .with_onus(100)
            .with_tranquillitas(30);
        assert_eq!(
            continuous(b.with_semen(0x0004), true, false, 0, 0).conatus(),
            99
        );
        assert_eq!(
            continuous(b.with_semen(0x0005), true, false, 0, 0).conatus(),
            100
        );
        assert_eq!(
            continuous(b.with_semen(0x0200), false, false, 0, 0).conatus(),
            101
        );
        assert_eq!(
            continuous(b.with_semen(0x0020), false, false, 0, 0).conatus(),
            100
        );
        assert_eq!(
            continuous(b.with_semen(0x0020), false, true, 0, 0).conatus(),
            101
        );
        assert_eq!(
            continuous(b.with_semen(0x0004), true, false, 128, 0).conatus(),
            98
        );
        assert_eq!(
            continuous(b.with_semen(0x0040), false, false, 0, 0).onus(),
            99
        );
        assert_eq!(
            continuous(b.with_onus(10).with_semen(0x0040), false, false, 0, 0).onus(),
            11
        );
        assert_eq!(
            continuous(b.with_onus(32).with_semen(0x0040), false, false, 0, 0).onus(),
            32
        );
        assert_eq!(
            continuous(b.with_semen(0x0100), false, false, 0, 600).tranquillitas(),
            31
        );
        assert_eq!(
            continuous(b.with_semen(0x0100), false, false, 0, 599).tranquillitas(),
            30
        );
        assert_eq!(
            continuous(b.with_semen(0x0001), false, false, 64, 0).tranquillitas(),
            29
        );
        assert_eq!(
            continuous(b.with_conatus(0).with_semen(0x0004), true, false, 255, 0).conatus(),
            0
        );
        assert_eq!(
            continuous(b.with_conatus(255).with_semen(0x0200), false, false, 0, 0).conatus(),
            255
        );
        assert_eq!(
            continuous(
                b.with_tranquillitas(63).with_semen(0x0100),
                false,
                false,
                0,
                600
            )
            .tranquillitas(),
            63
        );
        assert_eq!(
            continuous(
                b.with_tranquillitas(0).with_semen(0x0001),
                false,
                false,
                255,
                0
            )
            .tranquillitas(),
            0
        );
    }

    /// A clinamen fires only while awaiting, inside its band, through the 24/256 gate (REQ-L4-02).
    #[test]
    fn clinamen_fires_in_band() {
        let m = Memoria::new(0).with_eventus(0).with_ramus(3);
        let v = Verbum::new(0).with_ascensus(8).with_semen(0x0017);
        assert!(clinamen_fires(v, m));
        assert!(!clinamen_fires(v.with_semen(0x0018), m));
        assert!(!clinamen_fires(v.with_ascensus(7), m));
        assert!(clinamen_fires(v.with_ascensus(40), m));
        assert!(!clinamen_fires(v.with_ascensus(41), m));
        assert!(!clinamen_fires(v, m.with_ramus(0)));
        assert!(!clinamen_fires(v, m.with_eventus(AGNITIO)));
        for e in 0..8u8 {
            let (lo, hi) = band(e);
            let me = m.with_eventus(e);
            assert!(
                clinamen_fires(v.with_ascensus(lo), me) && clinamen_fires(v.with_ascensus(hi), me)
            );
            assert!(
                !clinamen_fires(v.with_ascensus(lo - 1), me)
                    && !clinamen_fires(v.with_ascensus(hi + 1), me)
            );
        }
    }

    /// Firing picks the branch by semen ≫ 12 mod 3, resets the age and applies the deltas
    /// (REQ-L4-02).
    #[test]
    fn clinamen_applies_deltas() {
        let v = Verbum::new(0)
            .with_conatus(100)
            .with_onus(100)
            .with_tranquillitas(30)
            .with_ascensus(100);
        let m = Memoria::new(0)
            .with_eventus(1)
            .with_ramus(3)
            .with_gradus(77);
        let (v1, m1) = fire(v.with_semen(0x1010), m);
        assert_eq!((m1.eventus(), m1.ramus(), m1.gradus()), (1, 1, 0));
        assert_eq!((v1.onus(), v1.conatus(), v1.tranquillitas()), (70, 100, 30));
        let (v2, m2) = fire(v.with_semen(0x2010), m);
        assert_eq!(m2.ramus(), 2);
        assert_eq!((v2.onus(), v2.conatus(), v2.tranquillitas()), (145, 90, 24));
        let (v0, _) = fire(v.with_semen(0x3010), m);
        assert_eq!((v0.onus(), v0.conatus(), v0.tranquillitas()), (105, 92, 34));
        let (vs, _) = fire(v.with_onus(240).with_tranquillitas(2).with_semen(0x2010), m);
        assert_eq!((vs.onus(), vs.tranquillitas()), (255, 0));
    }

    /// At 600 ticks a clinamen yields to another, never itself; special events end at 240
    /// (REQ-L4-03).
    #[test]
    fn clinamen_closes_and_rotates() {
        let m = Memoria::new(0)
            .with_eventus(6)
            .with_ramus(1)
            .with_gradus(600);
        let m1 = close(Verbum::new(0).with_semen(0x0300), m);
        assert_eq!((m1.eventus(), m1.ramus(), m1.gradus()), (2, 3, 0));
        for s in 1..=0xFFFFu16 {
            assert_ne!(close(Verbum::new(0).with_semen(s), m).eventus(), 6);
        }
        let v = Verbum::new(0).with_semen(0x0500);
        for e in 8..=15u8 {
            let r = release(
                v,
                Memoria::new(0)
                    .with_eventus(e)
                    .with_ramus(0)
                    .with_gradus(240),
            );
            assert_eq!((r.eventus(), r.ramus(), r.gradus()), (5, 3, 0));
        }
        let (_, m2, n) = event(
            Verbum::new(0).with_semen(0x0100),
            m.with_eventus(2).with_gradus(599),
            0,
            0,
            false,
        );
        assert_eq!(
            (m2.eventus(), m2.ramus(), m2.gradus(), n),
            (4, 3, 0, Nuntius::NIHIL)
        );
        assert_eq!(age(Memoria::new(0).with_gradus(1023)).gradus(), 1023);
    }

    /// The ring shifts within 14 bits; ascent resets; weight falls, effort rises; affirmation
    /// ends in serenity (REQ-L4-07).
    #[test]
    fn cycle_closure() {
        let v = Verbum::new(0)
            .with_ascensus(255)
            .with_onus(100)
            .with_conatus(100)
            .with_tranquillitas(10);
        let m = Memoria::new(0).with_clausurae(0x2003);
        let (v1, m1) = closure(v, m);
        assert_eq!(m1.clausurae(), 0x0006);
        assert_eq!(
            (v1.ascensus(), v1.onus(), v1.conatus(), v1.tranquillitas()),
            (0, 92, 124, 10)
        );
        assert_eq!(v1.modus(), Modus::Vigilia);
        let (va, _) = closure(v.with_modus(Modus::AmorFati), m);
        assert_eq!((va.tranquillitas(), va.modus()), (14, Modus::Vigilia));
        let (vs, _) = closure(
            v.with_onus(3)
                .with_conatus(250)
                .with_tranquillitas(62)
                .with_modus(Modus::AmorFati),
            m,
        );
        assert_eq!((vs.onus(), vs.conatus(), vs.tranquillitas()), (0, 255, 63));
    }

    /// The observer's return marks the present cycle (REQ-L4-08).
    #[test]
    fn interruption_marks_cycle() {
        let m = Memoria::new(0).with_clausurae(0b110);
        assert_eq!(interrupt(m).clausurae(), 0b111);
        assert_eq!(interrupt(interrupt(m)).clausurae(), 0b111);
    }

    /// Personae enter only in absence, through the shared gate, at their hour and place
    /// (REQ-L4-09).
    #[test]
    fn personae_conditions() {
        let v = Verbum::new(0)
            .with_semen(0x0200)
            .with_ascensus(128)
            .with_onus(50)
            .with_conatus(100);
        let m = Memoria::new(0).with_eventus(3).with_ramus(3);
        assert_eq!(persona(v, m, 128, 60, false), Some(DIOGENES));
        assert_eq!(persona(v, m, 128, 59, false), None);
        assert_eq!(persona(v, m, 128, 0, true), Some(DIOGENES));
        assert_eq!(persona(v.with_semen(0x0201), m, 128, 60, false), None);
        assert_eq!(persona(v, m.with_ramus(0), 128, 60, false), None);
        assert_eq!(persona(v, m, 111, 60, false), None);
        assert_eq!(
            persona(v.with_ascensus(100), m, 190, 60, false),
            Some(HERACLITUS)
        );
        assert_eq!(persona(v.with_ascensus(129), m, 190, 60, false), None);
        assert_eq!(
            persona(v.with_ascensus(250), m, 31, 1800, false),
            Some(DAEMON)
        );
        assert_eq!(persona(v.with_ascensus(250), m, 31, 1799, false), None);
        assert_eq!(persona(v.with_ascensus(250), m, 32, 1800, false), None);
        let (v13, m13) = visit(v.with_ascensus(20), m, DIOGENES);
        assert_eq!((v13.ascensus(), v13.onus()), (0, 58));
        assert_eq!((m13.eventus(), m13.ramus(), m13.gradus()), (DIOGENES, 0, 0));
        let (v14, _) = visit(v.with_curvatura(-10), m, HERACLITUS);
        assert_eq!((v14.onus(), v14.curvatura()), (34, -2));
        assert_eq!(
            visit(v.with_conatus(200), m, DAEMON).0.modus(),
            Modus::AmorFati
        );
        let (v15, _) = visit(v.with_conatus(199), m, DAEMON);
        assert_eq!((v15.modus(), v15.onus()), (Modus::Vigilia, 66));
    }

    /// Quies first; a stimulus wakes the dreamer; the dream needs night, calm, absence and
    /// r ≥ r∞; affirmation holds; the dream persists without a stimulus (REQ-L4-10).
    #[test]
    fn modus_precedence() {
        use Modus::{AmorFati, Quies, Somnium, Vigilia};
        let v = Verbum::new(0).with_semen(0x1000);
        let m = Memoria::new(0).with_eventus(2).with_ramus(3);
        for mo in [Vigilia, Quies, Somnium, AmorFati] {
            assert_eq!(
                modus_law(v.with_modus(mo), m, 38, 0, true, false, 0)
                    .0
                    .modus(),
                Quies
            );
        }
        let dream = v.with_modus(Somnium);
        let (a, _, n) = modus_law(dream, m, 200, 16, true, false, 4000);
        assert_eq!((a.modus(), n), (Vigilia, Nuntius::AGNITIO));
        assert_eq!(
            modus_law(dream, m, 200, 0, false, false, 4000).2,
            Nuntius::AGNITIO
        );
        assert_eq!(
            modus_law(dream, m, 200, 0, true, true, 4000).2,
            Nuntius::AGNITIO
        );
        assert_eq!(
            modus_law(dream, m, 200, 15, true, false, 4000).0.modus(),
            Somnium
        );
        assert_eq!(
            modus_law(v, m, 200, 1, true, false, 2820).0.modus(),
            Somnium
        );
        assert_eq!(
            modus_law(v, m, 200, 1, true, false, 2819).0.modus(),
            Vigilia
        );
        assert_eq!(
            modus_law(v, m, 200, 2, true, false, 2820).0.modus(),
            Vigilia
        );
        assert_eq!(
            modus_law(v.with_semen(0x1001), m, 200, 1, true, false, 2820)
                .0
                .modus(),
            Vigilia
        );
        assert_eq!(
            modus_law(v, m, 200, 1, false, false, u32::MAX).0.modus(),
            Vigilia
        );
        assert_eq!(
            modus_law(v.with_modus(AmorFati), m, 200, 255, false, true, 0)
                .0
                .modus(),
            AmorFati
        );
        assert_eq!(
            modus_law(v.with_modus(Quies), m, 39, 0, true, false, 0)
                .0
                .modus(),
            Vigilia
        );
    }

    /// The recognition: ascent and curvature reset, effort +32, event 12 (REQ-L4-11).
    #[test]
    fn agnitio_resets() {
        let dream = Verbum::new(0)
            .with_modus(Modus::Somnium)
            .with_ascensus(180)
            .with_curvatura(-64)
            .with_conatus(100)
            .with_semen(7);
        let m = Memoria::new(0)
            .with_eventus(4)
            .with_ramus(1)
            .with_gradus(300);
        let (v, m1) = agnitio(dream, m);
        assert_eq!(
            (v.ascensus(), v.curvatura(), v.conatus(), v.modus()),
            (0, 0, 132, Modus::Vigilia)
        );
        assert_eq!((m1.eventus(), m1.ramus(), m1.gradus()), (AGNITIO, 0, 0));
        assert_eq!(agnitio(dream.with_conatus(240), m).0.conatus(), 255);
    }

    /// Curvature per tick by mode (REQ-L4-11).
    #[test]
    fn curvature_dynamics() {
        use Modus::{AmorFati, Quies, Somnium, Vigilia};
        let c = |mo: Modus, k: i8| {
            curvature(Verbum::new(0).with_modus(mo).with_curvatura(k)).curvatura()
        };
        assert_eq!(
            [
                c(Somnium, 0),
                c(Somnium, -62),
                c(Somnium, -64),
                c(Somnium, -128)
            ],
            [-3, -64, -64, -64]
        );
        assert_eq!(
            [c(AmorFati, 0), c(AmorFati, 63), c(AmorFati, 127)],
            [2, 64, 64]
        );
        assert_eq!(
            [
                c(Vigilia, -64),
                c(Vigilia, 64),
                c(Vigilia, 1),
                c(Vigilia, -1),
                c(Quies, 40)
            ],
            [-48, 48, 0, 0, 30]
        );
        let mut v = Verbum::new(0).with_curvatura(-64);
        for _ in 0..16 {
            v = curvature(v);
        }
        assert_eq!(v.curvatura(), 0);
    }

    /// r = 2.5 + 0.6·night + 0.6·min(idle/3600, 1); the dream needs r ≥ r∞ (REQ-L1-04).
    #[test]
    fn somnium_eligibility_threshold() {
        assert_eq!(crate::natura::logistic_r(false, 0), Fx(0x28000));
        assert_eq!(crate::natura::logistic_r(true, 3600), Fx(0x3B334));
        assert_eq!(crate::natura::logistic_r(true, u32::MAX), Fx(0x3B334));
        assert!(!somnium_eligible(true, 2819));
        assert!(somnium_eligible(true, 2820));
        assert!(!somnium_eligible(false, u32::MAX));
    }

    /// Contemplation: Quies, a persona, or sleep for 64 ticks after the recognition (REQ-L4-05,
    /// REQ-L4-06).
    #[test]
    fn contemplation_states() {
        let v = Verbum::new(0);
        assert!(contemplating(
            v.with_modus(Modus::Quies),
            Memoria::new(0).with_ramus(3)
        ));
        assert!(contemplating(v, Memoria::new(0).with_eventus(HERACLITUS)));
        assert!(contemplating(
            v,
            Memoria::new(0).with_eventus(AGNITIO).with_gradus(63)
        ));
        assert!(!contemplating(
            v,
            Memoria::new(0).with_eventus(AGNITIO).with_gradus(64)
        ));
        assert!(!contemplating(
            v,
            Memoria::new(0).with_eventus(3).with_ramus(1)
        ));
    }

    /// The numbers the laws share with the fields and the corpus: `gradus` and `tranquillitas`
    /// saturate at the largest value of their bits, the eight clinamina are the events that
    /// have a band, the five lines of a chronicle show before its clinamen closes, and a special
    /// event outlasts the two lines of the recognition (REQ-L4-03, REQ-L4-04, REQ-L4-09).
    #[test]
    fn law_constants() {
        assert_eq!(GRADUS_MAX, Memoria::new(0).with_gradus(u16::MAX).gradus());
        assert_eq!(
            TRANQUILLITAS_MAX,
            Verbum::new(0).with_tranquillitas(u8::MAX).tranquillitas()
        );
        for eventus in 0..=u8::MAX {
            assert_eq!(
                band(eventus) != (0, 0),
                eventus < CLINAMINA,
                "eventus {eventus}"
            );
        }
        assert_eq!(
            (CLINAMINA, CLINAMEN_SPAN, SPECIAL_SPAN, SLEEP_SPAN),
            (8, 600, 240, 64)
        );
        assert_eq!(revealed(CLINAMEN_SPAN - 1), 5);
        assert!(SPECIAL_SPAN > OFFSETS[1] && SLEEP_SPAN < SPECIAL_SPAN);
    }

    /// The age counts one tick at a time and saturates at the largest `gradus` (REQ-L4-03).
    #[test]
    fn age_saturates() {
        for gradus in 0..=GRADUS_MAX {
            let aged = age(memory(3, 1, gradus));
            assert_eq!(aged.gradus(), gradus.saturating_add(1).min(GRADUS_MAX));
            assert_eq!(aged.with_gradus(gradus), memory(3, 1, gradus));
        }
    }

    /// The threshold of the pace over the domain: clamp(conatus − ⌊onus/2⌋, 16, 255), halved in
    /// wind and shifted right by ⌊(ascensus − 240)/4⌋ from 240 on, never below 1
    /// (REQ-L4-05, REQ-L1-08).
    #[test]
    fn pace_threshold_domain() {
        for conatus in [0u8, 1, 15, 16, 17, 32, 100, 159, 160, 161, 200, 254, 255] {
            for onus in [0u8, 1, 2, 31, 32, 33, 64, 128, 200, 254, 255] {
                for ascensus in 0..=u8::MAX {
                    for cells in [0x10u8, 0xFF] {
                        let v = Verbum::new(0)
                            .with_conatus(conatus)
                            .with_onus(onus)
                            .with_ascensus(ascensus)
                            .with_automatum(cells);
                        let base = (i32::from(conatus) - i32::from(onus) / 2).clamp(16, 255);
                        let halvings =
                            u32::from(cells == 0xFF) + u32::from(ascensus.saturating_sub(240)) / 4;
                        let expected = u16::try_from(base >> halvings).unwrap();
                        assert!(expected >= 1);
                        assert_eq!(
                            super::pace_threshold(v),
                            expected,
                            "conatus {conatus} onus {onus} ascensus {ascensus} cells {cells:#x}"
                        );
                    }
                }
            }
        }
    }

    /// The gate of the pace reads the fifteen low bits of `semen` against the threshold; the
    /// summit is never stepped onto from inside, and the step from 254 to 255 needs effort and
    /// calm air (REQ-L4-05).
    #[test]
    fn pace_gate_and_summit() {
        let v = Verbum::new(0)
            .with_conatus(192)
            .with_onus(32)
            .with_automatum(0x10);
        assert!(paces(v.with_semen(175)));
        assert!(!paces(v.with_semen(176)));
        assert!(paces(v.with_semen(0x8000 | 0x00AF)));
        assert!(!paces(v.with_semen(0x7FFF)));
        assert!(!paces(v.with_semen(0x4000)));
        assert!(!paces(v.with_semen(0x4005)));
        for conatus in [0u8, 100, 159, 160, 200, 255] {
            for cells in [0x10u8, 0xFF] {
                for ascensus in 0..=u8::MAX {
                    let open = v
                        .with_conatus(conatus)
                        .with_onus(0)
                        .with_ascensus(ascensus)
                        .with_automatum(cells)
                        .with_semen(0x8000);
                    let expected = match ascensus {
                        255 => false,
                        254 => conatus >= 160 && cells != 0xFF,
                        _ => true,
                    };
                    assert_eq!(
                        paces(open),
                        expected,
                        "conatus {conatus} ascensus {ascensus} cells {cells:#x}"
                    );
                    assert!(!paces(open.with_semen(0x7FFF)));
                }
            }
        }
    }

    /// The changes of one field in the same tick add up before the single saturating
    /// application: opposite laws cancel at the edge of a domain, and the same laws add
    /// (REQ-L4-06).
    #[test]
    fn continuous_changes_add_up() {
        let b = Verbum::new(0)
            .with_conatus(255)
            .with_onus(100)
            .with_tranquillitas(0);
        assert_eq!(
            continuous(b.with_semen(0x0200), false, false, 128, 0).conatus(),
            255
        );
        assert_eq!(
            continuous(b.with_semen(0x0004), true, false, 128, 0).conatus(),
            253
        );
        assert_eq!(
            continuous(b.with_semen(0x0004), true, true, 0, 0).conatus(),
            254
        );
        assert_eq!(
            continuous(b.with_semen(0x0100), false, false, 64, 600).tranquillitas(),
            0
        );
        let calm = b.with_tranquillitas(10).with_semen(0x0100);
        assert_eq!(continuous(calm, false, false, 64, 600).tranquillitas(), 10);
        assert_eq!(continuous(calm, false, false, 63, 600).tranquillitas(), 11);
        assert_eq!(continuous(calm, false, false, 0, 0), calm.with_onus(99));
    }

    /// Weight drifts one step toward 32 from either side, only when the six low bits of
    /// `semen` are zero, and rests at 32 (REQ-L4-06).
    #[test]
    fn weight_drifts_toward_thirty_two() {
        let b = Verbum::new(0).with_semen(0x0040);
        for onus in 0..=u8::MAX {
            let drifted = continuous(b.with_onus(onus), false, false, 0, 0).onus();
            let expected = match onus {
                0..=31 => onus + 1,
                32 => 32,
                _ => onus - 1,
            };
            assert_eq!(drifted, expected, "onus {onus}");
            let held = continuous(b.with_onus(onus).with_semen(0x0041), false, false, 0, 0);
            assert_eq!(held.onus(), onus, "onus {onus}");
        }
    }

    /// Firing applies the deltas of each of the 24 branches by saturating sum, chooses the branch
    /// by the generator and touches nothing else (REQ-L4-02).
    #[test]
    fn fire_applies_every_delta() {
        let v = Verbum::new(0)
            .with_conatus(100)
            .with_onus(100)
            .with_tranquillitas(30)
            .with_ascensus(77)
            .with_curvatura(-5);
        for eventus in 0..8u8 {
            for ramus in 0..3u8 {
                let semen = ((u16::from(ramus) + 3) << 12) | 0x0123;
                let (w, m) = fire(v.with_semen(semen), memory(eventus, 3, 500));
                let (onus, conatus, calm) = delta(eventus, ramus);
                assert_eq!(
                    i16::from(w.onus()),
                    100 + i16::from(onus),
                    "{eventus}.{ramus}"
                );
                assert_eq!(
                    i16::from(w.conatus()),
                    100 + i16::from(conatus),
                    "{eventus}.{ramus}"
                );
                assert_eq!(
                    i16::from(w.tranquillitas()),
                    30 + i16::from(calm),
                    "{eventus}.{ramus}"
                );
                let rest = w.with_onus(100).with_conatus(100).with_tranquillitas(30);
                assert_eq!(rest, v.with_semen(semen), "{eventus}.{ramus}");
                assert_eq!(m, memory(eventus, ramus, 0), "{eventus}.{ramus}");
            }
        }
    }

    /// Closing a cycle shifts the ring of closures within 14 bits for every value and moves
    /// nothing else of the memory (REQ-L4-07).
    #[test]
    fn closure_ring_shift() {
        let v = Verbum::new(0).with_ascensus(255);
        for ring in 0..0x4000u16 {
            let m = Memoria::new(0xDEAD_BEEF_0000_0000)
                .with_clausurae(ring)
                .with_eventus(9)
                .with_ramus(2)
                .with_gradus(300);
            let next = closure(v, m).1;
            assert_eq!(next.clausurae(), (ring << 1) & 0x3FFF, "ring {ring:#x}");
            assert_eq!(next.with_clausurae(ring), m, "ring {ring:#x}");
            let marked = interrupt(m);
            assert_eq!(marked.clausurae(), ring | 1, "ring {ring:#x}");
            assert_eq!(marked.with_clausurae(ring), m, "ring {ring:#x}");
        }
    }

    /// Each persona has its hour and its place: the closed windows of phase and `ascensus` of
    /// REQ-L4-09 and nothing outside them; the general entry needs a minute of idleness or
    /// occlusion, and the daemon needs 1 800 s of idleness even when occluded (REQ-L4-09).
    #[test]
    fn persona_windows() {
        let v = Verbum::new(0).with_semen(0x0200);
        let m = memory(3, 3, 0);
        for phase in 0..=u8::MAX {
            for ascensus in 0..=u8::MAX {
                let w = v.with_ascensus(ascensus);
                let expected = match (phase, ascensus) {
                    (112..=144, 96..=160) => Some(DIOGENES),
                    (184..=200, 64..=128) => Some(HERACLITUS),
                    (0..=31, 250..=255) => Some(DAEMON),
                    _ => None,
                };
                assert_eq!(
                    persona(w, m, phase, 3600, false),
                    expected,
                    "{phase} {ascensus}"
                );
                let watched = persona(w, m, phase, 59, false);
                assert_eq!(watched, None, "watched at {phase} {ascensus}");
                let hidden = persona(w, m, phase, 0, true);
                assert_eq!(
                    hidden,
                    expected.filter(|&p| p != DAEMON),
                    "occluded at {phase} {ascensus}"
                );
            }
        }
        for bit in 0..16u32 {
            let probe = Verbum::new(0).with_semen(1 << bit).with_ascensus(128);
            let entered = persona(probe, m, 128, 3600, false).is_some();
            assert_eq!(entered, bit >= 9, "gate bit {bit}");
        }
    }

    /// Each persona applies its effect by saturating sum, takes the event with `ramus` 0 and
    /// `gradus` 0, and a number that is no persona changes nothing (REQ-L4-09).
    #[test]
    fn persona_effects_saturate() {
        let v = Verbum::new(0)
            .with_ascensus(20)
            .with_onus(250)
            .with_conatus(100)
            .with_curvatura(120)
            .with_tranquillitas(40);
        let m = memory(3, 3, 77).with_quadrans(2);
        let (walker, left) = visit(v, m, DIOGENES);
        assert_eq!(walker, v.with_ascensus(0).with_onus(255));
        assert_eq!(left, m.with_eventus(DIOGENES).with_ramus(0).with_gradus(0));
        let (sage, left) = visit(v.with_onus(5), m, HERACLITUS);
        assert_eq!(sage, v.with_onus(0).with_curvatura(127));
        assert_eq!(
            left,
            m.with_eventus(HERACLITUS).with_ramus(0).with_gradus(0)
        );
        let (bound, left) = visit(v.with_conatus(200), m, DAEMON);
        assert_eq!(bound, v.with_conatus(200).with_modus(Modus::AmorFati));
        let daemon = m.with_eventus(DAEMON).with_ramus(0).with_gradus(0);
        assert_eq!(left, daemon);
        let (burdened, left) = visit(v.with_conatus(199).with_onus(245), m, DAEMON);
        assert_eq!(burdened, v.with_conatus(199).with_onus(255));
        assert_eq!(left, daemon);
        for other in (0..=u8::MAX).filter(|p| !(DIOGENES..=DAEMON).contains(p)) {
            assert_eq!(visit(v, m, other), (v, m), "persona {other}");
        }
    }

    /// The recognition lets the figure sleep for 64 ticks and no other event does (REQ-L3-02,
    /// REQ-L4-06).
    #[test]
    fn sleep_after_recognition() {
        for gradus in 0..=GRADUS_MAX {
            assert_eq!(
                asleep(memory(AGNITIO, 0, gradus)),
                gradus < 64,
                "gradus {gradus}"
            );
        }
        for eventus in (0..16u8).filter(|&e| e != AGNITIO) {
            assert!(!asleep(memory(eventus, 0, 0)), "eventus {eventus}");
        }
    }

    /// A clinamen that awaits its place never closes, however old; its age saturates (REQ-L4-03).
    #[test]
    fn awaiting_clinamen_keeps_its_place() {
        let v = Verbum::new(0).with_semen(0x0500);
        for eventus in 0..8u8 {
            let mut m = memory(eventus, 3, 0);
            for _ in 0..1100u32 {
                m = event(v, m, 0, 0, false).1;
            }
            assert_eq!(m, memory(eventus, 3, GRADUS_MAX), "eventus {eventus}");
        }
    }

    /// A special event lasts 240 ticks, whatever its branch, and then yields to an awaiting
    /// clinamen chosen by the generator (REQ-L4-03).
    #[test]
    fn special_events_end() {
        let v = Verbum::new(0).with_semen(0x0500);
        for eventus in 8..=15u8 {
            for ramus in 0..=3u8 {
                let mut m = memory(eventus, ramus, 0);
                for tick in 1..SPECIAL_SPAN {
                    let (_, next, n) = event(v, m, 0, 0, false);
                    assert_eq!(
                        (next.eventus(), next.ramus(), next.gradus(), n),
                        (eventus, ramus, tick, Nuntius::NIHIL),
                        "eventus {eventus} ramus {ramus} tick {tick}"
                    );
                    m = next;
                }
                let (_, ended, n) = event(v, m, 0, 0, false);
                assert_eq!(
                    (ended.eventus(), ended.ramus(), ended.gradus(), n),
                    (5, 3, 0, Nuntius::NIHIL),
                    "eventus {eventus} ramus {ramus}"
                );
            }
        }
    }

    /// In one tick the age comes first, then the end of the event, then the clinamen, and the
    /// persona only when no clinamen fires (REQ-L4-02, REQ-L4-03, REQ-L4-09).
    #[test]
    fn event_orders_the_laws() {
        let v = Verbum::new(0)
            .with_semen(0x0200)
            .with_ascensus(130)
            .with_onus(50)
            .with_conatus(100);
        let awaiting = memory(3, 3, 10);
        let (w, m, n) = event(v, awaiting, 128, 3600, false);
        assert_eq!(n, Nuntius::CLINAMEN);
        assert_eq!((m.eventus(), m.ramus(), m.gradus()), (3, 0, 0));
        assert_eq!((w.ascensus(), w.onus(), w.conatus()), (130, 50, 99));
        let (w, m, n) = event(v.with_ascensus(100), awaiting, 128, 3600, false);
        assert_eq!(n, Nuntius::PERSONA);
        assert_eq!((m.eventus(), m.ramus(), m.gradus()), (DIOGENES, 0, 0));
        assert_eq!((w.ascensus(), w.onus()), (68, 58));
        let (w, m, n) = event(v.with_ascensus(100), awaiting, 128, 0, false);
        assert_eq!((w, n), (v.with_ascensus(100), Nuntius::NIHIL));
        assert_eq!((m.eventus(), m.ramus(), m.gradus()), (3, 3, 11));
        let closing = v.with_semen(0x0100).with_ascensus(170);
        let (w, m, n) = event(closing, memory(2, 1, 599), 0, 0, false);
        assert_eq!(n, Nuntius::CLINAMEN);
        assert_eq!((m.eventus(), m.ramus(), m.gradus()), (4, 0, 0));
        assert_eq!((w.conatus(), w.tranquillitas()), (97, 1));
        let releasing = v.with_semen(0x0300);
        let (_, m, n) = event(releasing, memory(DIOGENES, 0, 239), 0, 0, false);
        assert_eq!(n, Nuntius::CLINAMEN);
        assert_eq!((m.eventus(), m.ramus(), m.gradus()), (3, 0, 0));
    }

    /// Over a grid of memories, words and measurements `event` moves only `eventus`, `ramus` and
    /// `gradus`, and its announcement composes the laws in their order: the age, then the end of
    /// the event, then the clinamen, then the persona; the word changes only with an
    /// announcement, and every outcome occurs (REQ-L4-02, REQ-L4-03, REQ-L4-09).
    #[test]
    fn event_is_total() {
        let mut seen = [0u32; 3];
        let mut personae = [false; 3];
        let (mut closings, mut endings) = (0u32, 0u32);
        for eventus in 0..16u8 {
            for ramus in 0..4u8 {
                for gradus in [0u16, 238, 239, 240, 598, 599, 600, 1023] {
                    let m = memory(eventus, ramus, gradus)
                        .with_quadrans(1)
                        .with_clausurae(0x155)
                        .with_oraculum(0xABCD_1234);
                    let aged = age(m);
                    let due_close = aged.eventus() < 8 && aged.ramus() != 3 && aged.gradus() >= 600;
                    let due_release = aged.eventus() >= 8 && aged.gradus() >= 240;
                    closings += u32::from(due_close);
                    endings += u32::from(due_release);
                    for ascensus in [0u8, 8, 40, 64, 100, 128, 130, 160, 170, 250, 255] {
                        for semen in [0x0200u16, 0x0201, 0x0017, 0x0018, 0x1100] {
                            let v = Verbum::new(0)
                                .with_ascensus(ascensus)
                                .with_semen(semen)
                                .with_conatus(100)
                                .with_onus(50);
                            let settled = if due_close {
                                close(v, aged)
                            } else if due_release {
                                release(v, aged)
                            } else {
                                aged
                            };
                            for (phase, idle, occluded) in [
                                (0u8, 0u32, false),
                                (10, 1800, false),
                                (128, 3600, false),
                                (128, 0, true),
                                (190, 60, false),
                            ] {
                                let (w, next, n) = event(v, m, phase, idle, occluded);
                                let at = std::format!(
                                    "{:?}",
                                    (
                                        eventus, ramus, gradus, ascensus, semen, phase, idle,
                                        occluded
                                    )
                                );
                                let kept = next
                                    .with_eventus(m.eventus())
                                    .with_ramus(m.ramus())
                                    .with_gradus(m.gradus());
                                assert_eq!(kept, m, "{at}");
                                let visitor = persona(v, settled, phase, idle, occluded);
                                if clinamen_fires(v, settled) {
                                    assert_eq!(n, Nuntius::CLINAMEN, "{at}");
                                    assert_eq!((w, next), fire(v, settled), "{at}");
                                    seen[0] += 1;
                                } else if let Some(p) = visitor {
                                    assert_eq!(n, Nuntius::PERSONA, "{at}");
                                    assert_eq!((w, next), visit(v, settled, p), "{at}");
                                    seen[1] += 1;
                                    personae[usize::from(p - DIOGENES)] = true;
                                } else {
                                    assert_eq!(n, Nuntius::NIHIL, "{at}");
                                    assert_eq!((w, next), (v, settled), "{at}");
                                    seen[2] += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        assert!(seen.iter().all(|&k| k > 0), "{seen:?}");
        assert_eq!(personae, [true; 3]);
        assert!(closings > 0 && endings > 0);
    }

    /// Curvature over the whole `i8` domain: Somnium falls by 3 down to −64, affirmation rises
    /// by 2 up to +64, Vigilia and Quies shrink by three quarters toward 0, and nothing else of
    /// the word moves (REQ-L4-11).
    #[test]
    fn curvature_over_the_domain() {
        for k in i8::MIN..=i8::MAX {
            let wide = i32::from(k);
            let expected = |mo: Modus| match mo {
                Modus::Somnium => (wide - 3).max(-64),
                Modus::AmorFati => (wide + 2).min(64),
                Modus::Vigilia | Modus::Quies => wide * 3 / 4,
            };
            for mo in [
                Modus::Vigilia,
                Modus::Quies,
                Modus::Somnium,
                Modus::AmorFati,
            ] {
                let v = Verbum::new(0x1234_5678_9ABC_DEF1)
                    .with_modus(mo)
                    .with_curvatura(k);
                let next = curvature(v);
                assert_eq!(i32::from(next.curvatura()), expected(mo), "{mo:?} {k}");
                assert_eq!(next.with_curvatura(k), v, "{mo:?} {k}");
            }
        }
    }

    /// The dream opens only at night and from 2 820 s of idleness, and stays open for longer
    /// idleness (REQ-L1-04).
    #[test]
    fn somnium_eligibility_window() {
        for idle in 0..=4000u32 {
            assert!(!somnium_eligible(false, idle), "day {idle}");
            assert_eq!(somnium_eligible(true, idle), idle >= 2820, "night {idle}");
        }
        assert!(somnium_eligible(true, u32::MAX));
    }

    /// Over a grid of words and measurements the law changes only the mode, except in the
    /// recognition; the announcement is AGNITIO exactly when a dream ends; Quies holds exactly
    /// below the energy of 39; a dream opens only from Vigilia (REQ-L4-10).
    #[test]
    fn modus_law_is_total() {
        use Modus::{AmorFati, Quies, Somnium, Vigilia};
        let mut seed = 0x2545_F491_4F6C_DD1Du64;
        let (mut dreamt, mut woke) = (0u32, 0u32);
        for mode in [Vigilia, Quies, Somnium, AmorFati] {
            for energy in [0u8, 38, 39, 255] {
                for seism in [0u8, 1, 2, 15, 16, 255] {
                    for (nocturnal, touched) in
                        [(false, false), (false, true), (true, false), (true, true)]
                    {
                        for idle in [0u32, 1199, 1200, 2819, 2820, u32::MAX] {
                            for semen in [0x1000u16, 0x1001, 0x0FFF, 0x0800, 0x8000] {
                                seed = mix(seed);
                                let word = Verbum::new(seed).with_modus(mode).with_semen(semen);
                                let past = Memoria::new(mix(seed));
                                let (after, later, said) =
                                    modus_law(word, past, energy, seism, nocturnal, touched, idle);
                                let stimulus = seism >= 16 || !nocturnal || touched;
                                let opens =
                                    nocturnal && seism < 2 && idle >= 2820 && semen & 0xFFF == 0;
                                let expected = if energy < 39 {
                                    Quies
                                } else {
                                    match mode {
                                        Somnium if stimulus => Vigilia,
                                        Somnium => Somnium,
                                        Vigilia if opens => Somnium,
                                        Vigilia | Quies => Vigilia,
                                        AmorFati => AmorFati,
                                    }
                                };
                                let wakes = mode == Somnium && energy >= 39 && stimulus;
                                let at = std::format!(
                                    "{:?}",
                                    (mode, energy, seism, nocturnal, touched, idle, semen)
                                );
                                assert_eq!(after.modus(), expected, "{at}");
                                if wakes {
                                    woke += 1;
                                    assert_eq!(said, Nuntius::AGNITIO, "{at}");
                                    assert_eq!((after, later), agnitio(word, past), "{at}");
                                } else {
                                    assert_eq!(said, Nuntius::NIHIL, "{at}");
                                    assert_eq!(later, past, "{at}");
                                    assert_eq!(after, word.with_modus(expected), "{at}");
                                }
                                dreamt += u32::from(mode == Vigilia && after.modus() == Somnium);
                            }
                        }
                    }
                }
            }
        }
        assert!(dreamt > 0 && woke > 0, "{dreamt} {woke}");
    }

    /// Each gate of the continuous laws reads exactly the bits its mask names: a single bit of
    /// `semen` below the width of the gate blocks the law and the next bit lets it through; the
    /// thresholds of `seism` are exact; a step excludes the rest of the same tick (REQ-L4-06).
    #[test]
    fn continuous_gates_read_their_bits() {
        let base = Verbum::new(0)
            .with_conatus(100)
            .with_onus(100)
            .with_tranquillitas(30);
        let moved = |a: u8, b: u8| i16::from(a) - i16::from(b);
        for bit in 0..16u32 {
            let b = base.with_semen(1 << bit);
            let spent = continuous(b, true, false, 0, 0);
            assert_eq!(
                moved(100, spent.conatus()),
                i16::from(bit >= 2),
                "step, bit {bit}"
            );
            let strained = continuous(b, true, false, 128, 0);
            assert_eq!(
                moved(spent.conatus(), strained.conatus()),
                i16::from(bit >= 2),
                "strain, bit {bit}"
            );
            let ascent = continuous(b, false, false, 0, 0);
            assert_eq!(
                moved(ascent.conatus(), 100),
                i16::from(bit >= 9),
                "ascent, bit {bit}"
            );
            let stillness = continuous(b, false, true, 0, 0);
            assert_eq!(
                moved(stillness.conatus(), 100),
                i16::from(bit >= 5),
                "contemplation, bit {bit}"
            );
            assert_eq!(
                moved(100, ascent.onus()),
                i16::from(bit >= 6),
                "weight, bit {bit}"
            );
            let serene = continuous(b, false, false, 0, 600);
            assert_eq!(
                moved(serene.tranquillitas(), 30),
                i16::from(bit >= 8),
                "serenity, bit {bit}"
            );
        }
        let b = base.with_semen(0x0004);
        for (seism, conatus, tranquillitas) in [
            (63u8, 99u8, 30u8),
            (64, 99, 29),
            (127, 99, 29),
            (128, 98, 29),
        ] {
            let tired = continuous(b, true, false, seism, 0);
            assert_eq!(
                (tired.conatus(), tired.tranquillitas()),
                (conatus, tranquillitas),
                "seism {seism}"
            );
        }
        let rested = base.with_semen(0x0200);
        assert_eq!(continuous(rested, true, false, 0, 0).conatus(), 99);
        assert_eq!(continuous(rested, false, false, 0, 0).conatus(), 101);
    }

    /// The successor of a closing clinamen is (`eventus` + 1 + (`semen` ≫ 8) mod 7) mod 8 for
    /// every byte of the generator and reaches the seven other clinamina; the successor of a
    /// special event is the clinamen (`semen` ≫ 8) ∧ 7 (REQ-L4-03).
    #[test]
    fn clinamen_successors() {
        for eventus in 0..8u8 {
            let mut reached = [false; 8];
            for byte in 0..=255u16 {
                let v = Verbum::new(0).with_semen((byte << 8) | 0x0001);
                let next = close(v, memory(eventus, 1, 600));
                let expected = (u16::from(eventus) + 1 + byte % 7) % 8;
                assert_eq!(
                    u16::from(next.eventus()),
                    expected,
                    "eventus {eventus} byte {byte}"
                );
                assert_eq!((next.ramus(), next.gradus()), (3, 0));
                reached[usize::from(next.eventus())] = true;
                let ended = release(v, memory(8 + eventus, 0, 240));
                assert_eq!(
                    u16::from(ended.eventus()),
                    byte & 7,
                    "special {eventus} byte {byte}"
                );
                assert_eq!((ended.ramus(), ended.gradus()), (3, 0));
            }
            assert_eq!(
                reached.iter().filter(|&&seen| seen).count(),
                7,
                "eventus {eventus}"
            );
            assert!(!reached[usize::from(eventus)], "eventus {eventus}");
        }
    }

    /// The recognition changes only `ascensus`, `curvatura`, `conatus` and `modus` of the word
    /// and `eventus`, `ramus` and `gradus` of the memory, for any word (REQ-L4-11).
    #[test]
    fn agnitio_touches_only_its_fields() {
        let mut seed = 0x9E37_79B9_7F4A_7C15u64;
        for _ in 0..2000u32 {
            seed = mix(seed);
            let word = Verbum::new(seed);
            let past = Memoria::new(mix(seed));
            let (awake, later) = agnitio(word, past);
            let expected = word
                .with_ascensus(0)
                .with_curvatura(0)
                .with_conatus(word.conatus().saturating_add(32))
                .with_modus(Modus::Vigilia);
            assert_eq!(awake, expected, "{seed:#x}");
            let marked = past.with_eventus(AGNITIO).with_ramus(0).with_gradus(0);
            assert_eq!(later, marked, "{seed:#x}");
        }
    }

    /// Closing a cycle returns affirmation to Vigilia with its serenity and leaves every other
    /// mode as it is (REQ-L4-07).
    #[test]
    fn closure_keeps_other_modes() {
        let v = Verbum::new(0).with_ascensus(255).with_tranquillitas(10);
        for mode in [Modus::Vigilia, Modus::Quies, Modus::Somnium] {
            let closed = closure(v.with_modus(mode), memory(3, 3, 0)).0;
            assert_eq!(
                (closed.modus(), closed.tranquillitas()),
                (mode, 10),
                "{mode:?}"
            );
        }
        let affirmed = closure(v.with_modus(Modus::AmorFati), memory(3, 3, 0)).0;
        assert_eq!(
            (affirmed.modus(), affirmed.tranquillitas()),
            (Modus::Vigilia, 14)
        );
    }

    /// A persona is contemplation at any branch and age, and the recognition is contemplation
    /// for its 64 ticks of sleep; no other event is, outside Quies (REQ-L4-05, REQ-L4-06).
    #[test]
    fn contemplation_by_event() {
        let v = Verbum::new(0);
        for eventus in 0..16u8 {
            for ramus in 0..4u8 {
                for gradus in [0u16, 63, 64, 239, 240, 1023] {
                    let m = memory(eventus, ramus, gradus);
                    let expected = (DIOGENES..=DAEMON).contains(&eventus)
                        || (eventus == AGNITIO && gradus < 64);
                    let at = std::format!("{eventus}.{ramus}@{gradus}");
                    assert!(contemplating(v.with_modus(Modus::Quies), m), "Quies {at}");
                    for mode in [Modus::Vigilia, Modus::Somnium, Modus::AmorFati] {
                        assert_eq!(
                            contemplating(v.with_modus(mode), m),
                            expected,
                            "{mode:?} {at}"
                        );
                    }
                }
            }
        }
    }
}
