//! Boundary witnesses of the tick order (REQ-L0-05; `docs/04-arquitetura.md` §7.1).
//! The public step preserves the precedence of closure, occlusion, chip, mode and pace.

use sisyphium::hospes::{Facultates, Indicia, Nuntius};
use sisyphium::memoria::Memoria;
use sisyphium::mundus::Mundus;
use sisyphium::natura::galois;
use sisyphium::verbum::{Modus, Verbum};

/// Solstice noon at the default UTC offset: circadian phase 128, by day.
const NOON: i64 = 1_782_054_000_000;

/// An inert reserved event isolates the scalar laws (REQ-L0-05). The generator starts at the
/// predecessor of `next`: the top bit of the next state selects the odd inverse of Galois.
fn order_world(
    next: u16,
    modus: Modus,
    ascensus: u8,
    conatus: u8,
    onus: u8,
    automatum: u8,
) -> Mundus {
    let old = if next & 0x8000 != 0 {
        ((next ^ 0xB400) << 1) | 1
    } else {
        next << 1
    };
    assert_eq!(galois(old), next);
    Mundus {
        verbum: Verbum::new(u64::from(old))
            .with_modus(modus)
            .with_ascensus(ascensus)
            .with_conatus(conatus)
            .with_onus(onus)
            .with_automatum(automatum)
            .with_tranquillitas(24),
        memoria: Memoria::new(0)
            .with_eventus(8)
            .with_ramus(0)
            .with_gradus(0)
            .with_clausurae(0b101)
            .with_oraculum(0x5555_5555),
    }
}

/// The observer is present at noon, without idleness (REQ-L4-10).
fn order_indicia() -> Indicia {
    Indicia {
        epoch_ms: NOON,
        idle_s: 0,
        ..Indicia::default()
    }
}

/// Scarce energy changes Vigilia to Quies before an otherwise open pace (REQ-L4-05/10).
#[test]
fn modus_precedes_the_pace() {
    let mut w = order_world(2, Modus::Vigilia, 100, 192, 32, 0x10);
    let n = w.step(
        &Facultates::default(),
        &Indicia {
            energy: 20,
            ..order_indicia()
        },
    );
    assert_eq!((w.verbum.modus(), w.verbum.ascensus()), (Modus::Quies, 100));
    assert_eq!(
        (w.verbum.bits(), w.memoria.bits()),
        (0x5810_0064_20C0_0002, 0x5555_5555_0048_0005)
    );
    assert_eq!(n, Nuntius::NIHIL);
}

/// Occlusion creates wind before the last pace can pass (REQ-L6-01, REQ-L1-08, REQ-L4-05).
#[test]
fn occlusion_precedes_the_pace() {
    let mut w = order_world(2, Modus::Vigilia, 254, 200, 129, 0x55);
    let n = w.step(
        &Facultates::default(),
        &Indicia {
            occluded: true,
            ..order_indicia()
        },
    );
    assert_eq!((w.verbum.automatum(), w.verbum.ascensus()), (0xFF, 254));
    assert_eq!(
        (w.verbum.bits(), w.memoria.bits()),
        (0x18FF_00FE_81C8_0002, 0x5555_5555_0048_0005)
    );
    assert_eq!(n, Nuntius::NIHIL);
}

/// The chip creates wind that halves the threshold before the pace (REQ-L3-04, REQ-L4-05).
#[test]
fn chip_precedes_the_pace() {
    let mut w = order_world(0x40, Modus::Vigilia, 100, 100, 0, 0xF8);
    let n = w.step(
        &Facultates::default(),
        &Indicia {
            seism: 240,
            ..order_indicia()
        },
    );
    assert_eq!((w.verbum.automatum(), w.verbum.ascensus()), (0xFA, 100));
    assert_eq!(
        (
            w.verbum.conatus(),
            w.verbum.onus(),
            w.verbum.tranquillitas()
        ),
        (99, 1, 23)
    );
    assert_eq!(
        (w.verbum.bits(), w.memoria.bits()),
        (0x17FA_0064_0163_0040, 0x5555_5555_0048_0005)
    );
    assert_eq!(n, Nuntius::LAPSUS);
}

/// The closure lowers weight below the branch of occlusion (REQ-L4-07, REQ-L6-01).
#[test]
fn closure_precedes_the_occlusion() {
    let mut w = order_world(0x201, Modus::Vigilia, 255, 100, 129, 0x10);
    let n = w.step(
        &Facultates::default(),
        &Indicia {
            occluded: true,
            ..order_indicia()
        },
    );
    assert_eq!(
        (w.verbum.onus(), w.verbum.automatum(), w.verbum.ascensus()),
        (121, 0x20, 0)
    );
    assert_eq!((w.verbum.conatus(), w.memoria.clausurae()), (124, 0b1010));
    assert_eq!(
        (w.verbum.bits(), w.memoria.bits()),
        (0x1820_0000_797C_0201, 0x5555_5555_0048_000A)
    );
    assert_eq!(n, Nuntius::CLAUSURA);
}

/// The closure sees affirmation before scarce energy chooses Quies (REQ-L4-07/10).
#[test]
fn closure_precedes_the_modus() {
    let mut w = order_world(2, Modus::AmorFati, 255, 192, 32, 0x10);
    let n = w.step(
        &Facultates::default(),
        &Indicia {
            energy: 20,
            ..order_indicia()
        },
    );
    assert_eq!(
        (
            w.verbum.modus(),
            w.verbum.tranquillitas(),
            w.verbum.ascensus()
        ),
        (Modus::Quies, 28, 0)
    );
    assert_eq!(
        (w.verbum.conatus(), w.verbum.onus(), w.memoria.clausurae()),
        (216, 24, 0b1010)
    );
    assert_eq!(
        (w.verbum.bits(), w.memoria.bits()),
        (0x5C10_0000_18D8_0002, 0x5555_5555_0048_000A)
    );
    assert_eq!(n, Nuntius::CLAUSURA);
}
