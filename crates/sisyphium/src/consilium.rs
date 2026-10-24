//! `consilium`: the choice of the form (level L1). The regimen of a frame (stilus, strata, the
//! cadence of the world and of the presentation, layers and chronicle lines) is chosen for the
//! host and the narrative situation by a satisficing search (Simon, 1955) with simulated
//! annealing (Kirkpatrick et al., 1983), whose acceptance of a worse proposal follows Metropolis
//! et al. (1953), within a budget of work set by the energy and the compute of the host
//! (REQ-L1-07, REQ-C-02..05; `docs/06-consilium.md`).
//!
//! The score of a regimen is its semiotic value in the situation of the world ([`value`]) minus
//! λ = 1/8 of its cost on the host ([`cost`]). The value reads `modus`, `conatus`, `onus`,
//! `tranquillitas`, `ramus`, `eventus` and `clausurae`; the cost reads the dimensions of the host.
//! The forcings of REQ-C-04 bound the dimensions before any score. The search draws every
//! proposal from the Galois generator ([`galois`]) started at the `semen` of the world, so the
//! same host, world and energy give the same regimen (REQ-C-03). After each measurement and
//! evolution of the world, the host compares the reevaluation conditions of REQ-C-05, including
//! the budget and the semantic key, and reruns the search when a condition changes.

use crate::hospes::{
    ANNALES, CAELUM, Facultates, HEXAGRAMMA, IMPOSSIBILIA, LAYERS_ALL, MARGO, PERSONAE, PUNCTA,
    RUINAE, Regimen, Stilus,
};
use crate::memoria::RAMUS_EXSPECTANS;
use crate::mundus::Mundus;
use crate::natura::galois;
use crate::spatium::compositio;
use crate::verbum::Modus;

/// Cadences of presentation in milliseconds, from the fastest: the periods of 24 Hz
/// (⌈1000/24⌉ = 42), 8 Hz, 4 Hz and 1 Hz, and one frame per minute (`docs/05-estetica.md` §8;
/// REQ-L1-07, REQ-C-02). The first three and the last are also the cadences of the world, one
/// per band of energy ([`band_tick_ms`]).
pub const INTERVALS: [u32; 5] = [42, 125, 250, 1000, 60_000];

/// The acceptance table of the annealing (Metropolis et al., 1953; REQ-C-03): the byte at bit
/// 8·k is ⌊255·e^(−k/4)⌋ for k = 0..=15, that is 255, 198, 154, 120, 93, 73, 56, 44, 34, 26, 20,
/// 16, 12, 9, 7 and 5. A proposal worse by Δ at the temperature T is accepted when a byte of the
/// generator falls below the entry k = min(15, ⌊4·Δ/T⌋), with probability ⌊255·e^(−k/4)⌋/256
/// ≈ e^(−Δ/T).
pub const EXP: u128 = 0x0507_090C_1014_1A22_2C38_495D_789A_C6FF;

/// Lowest energy of the band of 42 ms: 179 = ⌈0,70·255⌉, seven tenths of the range of the
/// energy (REQ-L1-07).
const BAND_42_ENERGY: u8 = 179;

/// Lowest energy of the band of 125 ms: 77 = ⌈0,30·255⌉ (REQ-L1-07).
const BAND_125_ENERGY: u8 = 77;

/// Lowest energy of the band of 250 ms: 39 = ⌈0,15·255⌉. Below it the world rests: its band
/// ticks once a minute in Bicolor (REQ-L1-07) and the law of the mode gives Quies (REQ-L4-10).
const BAND_250_ENERGY: u8 = 39;

/// Divisor of the budget, `compute`·`energy`/64: a host of full compute on full energy has
/// 255·255/64 = 1016 units of work per second (`docs/06-consilium.md` §3).
const BUDGET_DIVISOR: u32 = 64;

/// Units of work of the base of every frame (the slate, one stratum of the slope, the figure and
/// the boulder) at 320×180: the reference of the table of `docs/06-consilium.md` §3.
const BASE_COST: u32 = 10;

/// Units of work of each stratum beyond the first (`docs/06-consilium.md` §3).
const STRATUM_COST: u32 = 18;

/// Units of work of each layer per frame at 320×180, the byte at bit 8·i for the layer of bit
/// 2ⁱ (`docs/06-consilium.md` §3): sky 2, frame 5, chronicles 1 per line, personae 1, ruins 2,
/// impossibilia 1, hexagram 1, dither 0. These are conservative isolated increments,
/// not a universal duration bound; see the calibration limitations in §3.
const LAYER_COSTS: u64 = 0x0001_0102_0101_0502;

/// Units of work each stilus adds per frame, the byte at bit 8·d for the discriminant d
/// (`docs/06-consilium.md` §3): Creta 0, Musivum 0, Littera 34 (the map to glyphs), Bicolor 3
/// (the threshold of the dither).
const STILUS_COSTS: u32 = 0x0322_0000;

/// Pixels of the frame on which the units of work are counted: 320·180 = 57 600, the logical
/// frame of scale 1 (REQ-L2-11).
const UNIT_AREA: u64 = 57_600;

/// Milliseconds per second: the cost counts per second, the interval in milliseconds.
const MS_PER_S: u64 = 1000;

/// Value of each layer, the byte at bit 8·i for the layer of bit 2ⁱ (`docs/06-consilium.md`
/// §4): sky 8, frame 5, chronicles 2 per line, personae 6, ruins 3, impossibilia 7, hexagram 2,
/// dither 2.
const LAYER_VALUES: u64 = 0x0202_0703_0602_0508;

/// Value of each stratum beyond the first (`docs/06-consilium.md` §4).
const STRATUM_VALUE: u32 = 2;

/// Value of each cadence of presentation, the byte at bit 8·k for the interval k of
/// [`INTERVALS`] (`docs/06-consilium.md` §4): 42 ms 6, 125 ms 4, 250 ms 2, 1000 ms 1,
/// 60 000 ms 0.
const CADENCE_VALUES: u64 = 0x0102_0406;

/// Value of each stilus, the byte at bit 8·d for the discriminant d (`docs/06-consilium.md`
/// §4): Creta 4, Musivum 3, Littera 2, Bicolor 1.
const STILUS_VALUES: u32 = 0x0102_0304;

/// Value of each stilus in the situation that prefers it, the byte at bit 8·d for the
/// discriminant d (`docs/06-consilium.md` §4, `docs/05-estetica.md` §9): Creta 8 in `Modus::AmorFati`,
/// its 4 plus a bonus of 4; Musivum, which no situation prefers, 3; Littera 6 in the eidetic
/// reduction; Bicolor 6 in Quies.
const PREFERRED_VALUES: u32 = 0x0606_0308;

/// Weight above which the value of the frame doubles: 128, half of the 256 values of `onus`,
/// where the wilting of the frame becomes visible (`docs/06-consilium.md` §4).
const WILT_ONUS: u8 = 128;

/// Effort and weight above which the impossible figures carry value: 200, the threshold of
/// excess of the palette and of the corrupted harmony (REQ-L5-02, REQ-L5-12).
const EXCESS: u8 = 200;

/// Serenity from which the value of the hexagram doubles and the eidetic reduction holds: 48,
/// three quarters of the 64 values of `tranquillitas` (`docs/06-consilium.md` §4,
/// `docs/05-estetica.md` §9).
const SERENITY: u8 = 48;

/// First `eventus` of a persona, Diogenes (REQ-L4-01).
const PERSONA_FIRST: u8 = 13;

/// Last `eventus` of a persona, the daemon (REQ-L4-01).
const PERSONA_LAST: u8 = 15;

/// Largest number of strata (REQ-C-02).
const STRATA_MAX: u8 = 4;

/// Strata of a dream: at least 2 (REQ-C-04).
const DREAM_STRATA: u8 = 2;

/// Largest number of chronicle lines (REQ-C-02).
const LINES_MAX: u8 = 6;

/// The stili in the order of their discriminants, the order of the proposals (REQ-C-02).
const STILI: [Stilus; 4] = [
    Stilus::Creta,
    Stilus::Musivum,
    Stilus::Littera,
    Stilus::Bicolor,
];

/// Dimensions of the space of regimina that a proposal changes: stilus, strata, interval,
/// layers and lines (`docs/06-consilium.md` §2).
const DIMENSIONS: u16 = 5;

/// Proposals of the search (REQ-C-03).
const PROPOSALS: u32 = 64;

/// The inverse of λ = 1/8: the score 8·value − cost orders the regimina as value − cost/8
/// (REQ-C-03).
const LAMBDA_INVERSE: i64 = 8;

/// Numerator and denominator of the level of aspiration, 80 % = 4/5 of the maximum of the value
/// (Simon, 1955; REQ-C-03).
const ASPIRATION: (u32, u32) = (4, 5);

/// Divisor of the temperature T = `energy`/16, 0..=15, one degree per entry of [`EXP`]
/// (REQ-C-03).
const TEMPERATURE_DIVISOR: u8 = 16;

/// Entries of [`EXP`] per unit of Δ/T: the entry k = 4·Δ/T holds 255·e^(−k/4) = 255·e^(−Δ/T).
const EXP_PER_UNIT: i64 = 4;

/// Last entry of [`EXP`].
const EXP_LAST: i64 = 15;

/// Mask of the byte of the generator that meets the entry of [`EXP`].
const DRAW_MASK: u16 = 0xFF;

/// The cadence of the world for an energy (REQ-L1-07): from 179 (⌈0,70·255⌉) 42 ms, the period
/// of 24 Hz; from 77 (⌈0,30·255⌉) 125 ms, 8 Hz; from 39 (⌈0,15·255⌉) 250 ms, 4 Hz; below 39 one
/// tick per minute. The four cadences are entries of [`INTERVALS`].
#[must_use]
pub const fn band_tick_ms(energy: u8) -> u32 {
    let [fast, mean, slow, _, still] = INTERVALS;
    if energy >= BAND_42_ENERGY {
        fast
    } else if energy >= BAND_125_ENERGY {
        mean
    } else if energy >= BAND_250_ENERGY {
        slow
    } else {
        still
    }
}

/// The budget of work of a host, in units per second (`docs/06-consilium.md` §3):
/// ⌊`compute`·`energy`/64⌋. A host of full compute on full energy has 1016; a host of compute 96
/// at 30 % of its energy (77) has 115.
#[must_use]
pub fn budget(facultates: &Facultates, energy: u8) -> u32 {
    u32::from(facultates.compute).wrapping_mul(u32::from(energy)) / BUDGET_DIVISOR
}

/// Byte `index` of a packed table, the byte at bit 8·`index`; 0 beyond the table
/// (`docs/06-consilium.md` §3, §4).
const fn byte(table: u128, index: u32) -> u32 {
    match table.checked_shr(index.wrapping_mul(8)) {
        Some(word) => (word & 0xFF) as u32,
        None => 0,
    }
}

/// Sum of the entries of a packed table of layers over the enabled `layers`
/// (`docs/06-consilium.md` §3, §4): the byte at bit 8·i for the layer of bit 2ⁱ, the
/// chronicles once per line, and the layers of `doubled` twice.
fn layer_sum(table: u64, layers: u8, lines: u8, doubled: u8) -> u32 {
    (0u32..8).fold(0u32, |sum, index| {
        let layer = 1u8.wrapping_shl(index);
        if layers & layer == 0 {
            return sum;
        }
        let entry = byte(table as u128, index).wrapping_shl(u32::from(doubled & layer != 0));
        let count = if layer == ANNALES {
            u32::from(lines)
        } else {
            1
        };
        sum.saturating_add(entry.saturating_mul(count))
    })
}

/// Units of work of one frame of the regimen at 320×180 (`docs/06-consilium.md` §3): the base,
/// 18 per stratum beyond the first, the layers and the stilus.
fn units(regimen: &Regimen) -> u32 {
    let strata = u32::from(regimen.strata.saturating_sub(1)).saturating_mul(STRATUM_COST);
    BASE_COST
        .saturating_add(strata)
        .saturating_add(layer_sum(LAYER_COSTS, regimen.layers, regimen.lines, 0))
        .saturating_add(byte(STILUS_COSTS as u128, regimen.stilus as u32))
}

/// Pixels a frame of the regimen draws (`docs/06-consilium.md` §3; REQ-L5-03): W·H, and
/// ⌊W/s⌋·⌊H/s⌋ in Musivum, which rasterizes at the integer scale s of the composition
/// (REQ-L2-11) and enlarges by the nearest neighbour.
fn area(regimen: &Regimen, facultates: &Facultates) -> u64 {
    let (width, height) = (u64::from(facultates.width), u64::from(facultates.height));
    if matches!(regimen.stilus, Stilus::Musivum) {
        let s = u64::from(compositio(facultates.width, facultates.height, regimen.lines).s);
        width
            .checked_div(s)
            .unwrap_or(width)
            .saturating_mul(height.checked_div(s).unwrap_or(height))
    } else {
        width.saturating_mul(height)
    }
}

/// The cost of a regimen on a host, in units of work per second (`docs/06-consilium.md` §3):
/// ⌊units·area·1000/(57 600·`interval_ms`)⌋ in `u64`, the units of one frame at 320×180 scaled
/// by the pixels the stilus draws and by the frames per second. The product stays below 2⁶³ for
/// every regimen and host, at most 1 053 units over (2¹⁶ − 1)² pixels; the quotient saturates
/// at `u32::MAX`, and an interval of 0 ms, a presentation without pause, costs `u32::MAX`.
#[must_use]
pub fn cost(regimen: &Regimen, facultates: &Facultates) -> u32 {
    let work = u64::from(units(regimen))
        .saturating_mul(area(regimen, facultates))
        .saturating_mul(MS_PER_S);
    let span = UNIT_AREA.saturating_mul(u64::from(regimen.interval_ms));
    work.checked_div(span)
        .map_or(u32::MAX, |c| u32::try_from(c).unwrap_or(u32::MAX))
}

/// Whether the world rests (REQ-C-04, REQ-L1-07, REQ-L4-10): its mode is Quies, or the energy is
/// below 39, where the band ticks once a minute in Bicolor and the law of the mode gives Quies
/// at the next tick.
const fn quiescent(mundus: &Mundus, energy: u8) -> bool {
    matches!(mundus.verbum.modus(), Modus::Quies) || energy < BAND_250_ENERGY
}

/// Whether the world is in the eidetic reduction, the situation that prefers Littera
/// (`docs/05-estetica.md` §9): the energy of the band of 250 ms (39..=76) and `tranquillitas`
/// from 48.
const fn eidetic(mundus: &Mundus, energy: u8) -> bool {
    energy >= BAND_250_ENERGY
        && energy < BAND_125_ENERGY
        && mundus.verbum.tranquillitas() >= SERENITY
}

/// The layers that carry value in the situation (`docs/06-consilium.md` §4): sky, frame,
/// chronicles, hexagram and dither always; the personae while one is present (`eventus`
/// 13..=15); the ruins when a closed cycle was interrupted, popcount(`clausurae` ≫ 1) > 0
/// (REQ-L8-01); the impossible figures with `conatus` > 200 ∧ `onus` > 200, or in Somnium.
const fn salient_layers(mundus: &Mundus) -> u8 {
    let v = mundus.verbum;
    let m = mundus.memoria;
    let mut layers = CAELUM | MARGO | ANNALES | HEXAGRAMMA | PUNCTA;
    if matches!(m.eventus(), PERSONA_FIRST..=PERSONA_LAST) {
        layers |= PERSONAE;
    }
    if (m.clausurae() >> 1).count_ones() > 0 {
        layers |= RUINAE;
    }
    if (v.conatus() > EXCESS && v.onus() > EXCESS) || matches!(v.modus(), Modus::Somnium) {
        layers |= IMPOSSIBILIA;
    }
    layers
}

/// The layers whose value doubles in the situation (`docs/06-consilium.md` §4): sky and
/// chronicles during an active clinamen (`ramus` ≠ 3); the frame with `onus` > 128; the
/// hexagram with `tranquillitas` ≥ 48; the dither in Somnium.
const fn doubled_layers(mundus: &Mundus) -> u8 {
    let v = mundus.verbum;
    let mut layers = 0;
    if mundus.memoria.ramus() != RAMUS_EXSPECTANS {
        layers |= CAELUM | ANNALES;
    }
    if v.onus() > WILT_ONUS {
        layers |= MARGO;
    }
    if v.tranquillitas() >= SERENITY {
        layers |= HEXAGRAMMA;
    }
    if matches!(v.modus(), Modus::Somnium) {
        layers |= PUNCTA;
    }
    layers
}

/// Position in [`INTERVALS`] of the first interval at or above `interval_ms`; 5 above the last.
fn rank(interval_ms: u32) -> u32 {
    (0u32..)
        .zip(INTERVALS)
        .find(|&(_, entry)| entry >= interval_ms)
        .map_or(5, |(k, _)| k)
}

/// The value of a cadence of presentation (`docs/06-consilium.md` §4): 42 ms 6, 125 ms 4,
/// 250 ms 2, 1000 ms 1, 60 000 ms 0; an interval between two entries takes the value of the
/// slower one and an interval above a minute is worth 0. Below the band of 42 ms (energy < 179)
/// the world ticks at 125 ms or slower, and a presentation every 42 ms or faster shows no new
/// state: it is worth 0.
fn cadence_value(interval_ms: u32, energy: u8) -> u32 {
    let k = rank(interval_ms);
    if k == 0 && energy < BAND_42_ENERGY {
        0
    } else {
        byte(CADENCE_VALUES as u128, k)
    }
}

/// The value of a stilus in the situation (`docs/06-consilium.md` §4, `docs/05-estetica.md`
/// §9): Creta 4, Musivum 3, Littera 2, Bicolor 1, and the preferred value where a situation
/// prefers the stilus: Creta 8 in `Modus::AmorFati`, Littera 6 in the eidetic reduction, Bicolor 6 in
/// Quies.
const fn stilus_value(stilus: Stilus, mundus: &Mundus, energy: u8) -> u32 {
    let preferred = match stilus {
        Stilus::Creta => matches!(mundus.verbum.modus(), Modus::AmorFati),
        Stilus::Musivum => false,
        Stilus::Littera => eidetic(mundus, energy),
        Stilus::Bicolor => matches!(mundus.verbum.modus(), Modus::Quies),
    };
    let table = if preferred {
        PREFERRED_VALUES
    } else {
        STILUS_VALUES
    };
    byte(table as u128, stilus as u32)
}

/// The semiotic value of a regimen in the situation of the world (`docs/06-consilium.md` §4;
/// REQ-C-03), the sum of one term per dimension:
///
/// - layers and lines: sky 8, frame 5, chronicles 2 per line, personae 6, ruins 3, impossibilia
///   7, hexagram 2 and dither 2, for the enabled layers that carry value in the situation, each
///   doubled where the situation doubles it;
/// - strata: 2 per stratum beyond the first, 4 in Somnium;
/// - cadence: 6, 4, 2, 1 or 0 by the interval of presentation, 0 for 42 ms below the band of
///   42 ms;
/// - stilus: Creta 4, Musivum 3, Littera 2, Bicolor 1; Creta 8 in `Modus::AmorFati`, Littera 6 in the
///   eidetic reduction (energy 39..=76 ∧ `tranquillitas` ≥ 48), Bicolor 6 in Quies. Low energy
///   alone forces Bicolor without changing its value before the world enters Quies.
///
/// The value reads the regimen, the world and the energy; the host enters the score through
/// [`cost`] and the forcings, so `_facultates` is not read.
#[must_use]
pub fn value(regimen: &Regimen, mundus: &Mundus, _facultates: &Facultates, energy: u8) -> u32 {
    let layers = layer_sum(
        LAYER_VALUES,
        regimen.layers & salient_layers(mundus),
        regimen.lines,
        doubled_layers(mundus),
    );
    let dream = matches!(mundus.verbum.modus(), Modus::Somnium);
    let strata = u32::from(regimen.strata.saturating_sub(1))
        .saturating_mul(STRATUM_VALUE.wrapping_shl(u32::from(dream)));
    layers
        .saturating_add(strata)
        .saturating_add(cadence_value(regimen.interval_ms, energy))
        .saturating_add(stilus_value(regimen.stilus, mundus, energy))
}

/// The floor of the interval of presentation (REQ-L1-07, REQ-C-04): the first entry of
/// [`INTERVALS`] at or above both the cadence of the world and the period of the fastest rate of
/// the host, max(`tick_ms`, ⌈1000/`max_hz`⌉); one frame per minute for a host of static frames
/// (`max_hz` = 0) and above the last entry.
fn interval_floor(tick_ms: u32, max_hz: u8) -> u32 {
    let [.., still] = INTERVALS;
    if max_hz == 0 {
        return still;
    }
    let bound = tick_ms.max(1000u32.div_ceil(u32::from(max_hz)));
    INTERVALS
        .into_iter()
        .find(|&entry| entry >= bound)
        .unwrap_or(still)
}

/// The forcings of the host and of the situation (REQ-C-04, REQ-L1-07), in this precedence: a
/// host of glyph cells gives Littera; else a host of one bit gives Bicolor; else a world at rest
/// (Quies, or energy below 39) gives Bicolor; else Somnium turns Littera into Creta. A world at
/// rest ticks once a minute; any other ticks at the cadence of its band. Somnium needs at least
/// two strata, on any host. The interval rises to its floor ([`interval_floor`]), which a host
/// of static frames (`max_hz` = 0) sets at a minute. The other dimensions pass unchanged, and a
/// forced regimen is its own image.
fn force(regimen: Regimen, facultates: &Facultates, mundus: &Mundus, energy: u8) -> Regimen {
    let quiet = quiescent(mundus, energy);
    let dream = matches!(mundus.verbum.modus(), Modus::Somnium);
    let stilus = if facultates.cells {
        Stilus::Littera
    } else if facultates.color_bits == 1 || quiet {
        Stilus::Bicolor
    } else if dream && matches!(regimen.stilus, Stilus::Littera) {
        Stilus::Creta
    } else {
        regimen.stilus
    };
    let [.., still] = INTERVALS;
    let tick_ms = if quiet { still } else { band_tick_ms(energy) };
    let strata = if dream {
        regimen.strata.max(DREAM_STRATA)
    } else {
        regimen.strata
    };
    let interval_ms = regimen
        .interval_ms
        .max(interval_floor(tick_ms, facultates.max_hz));
    Regimen {
        stilus,
        strata,
        tick_ms,
        interval_ms,
        ..regimen
    }
}

/// The theoretical maximum of the value in the situation and under the forcings (REQ-C-03): the
/// sum of the maxima of the five dimensions. The value is one term per dimension and each
/// forcing bounds one dimension, so the sum is the value of the regimen that takes every
/// dimension at its maximum: four strata, the floor of the interval (the value of a cadence
/// falls as its interval grows), every layer with six lines, and the stilus of the highest value
/// among those the forcings leave.
#[must_use]
pub fn value_max(mundus: &Mundus, facultates: &Facultates, energy: u8) -> u32 {
    STILI
        .into_iter()
        .map(|stilus| {
            let top = Regimen {
                stilus,
                strata: STRATA_MAX,
                tick_ms: band_tick_ms(energy),
                interval_ms: 0,
                layers: LAYERS_ALL,
                lines: LINES_MAX,
            };
            value(
                &force(top, facultates, mundus, energy),
                mundus,
                facultates,
                energy,
            )
        })
        .max()
        .unwrap_or(0)
}

/// The floor regimen (REQ-C-03, REQ-C-04): the stilus the forcings fix, otherwise Musivum, whose
/// area is the logical frame at every resolution; one stratum, two in Somnium; no layer and no
/// line; one frame per minute; the cadence of the world of its band, a minute when the world
/// rests. The search returns it when the budget is zero or even the floor exceeds it.
#[must_use]
pub fn floor(facultates: &Facultates, mundus: &Mundus, energy: u8) -> Regimen {
    let [.., still] = INTERVALS;
    let least = Regimen {
        stilus: Stilus::Musivum,
        strata: 1,
        tick_ms: band_tick_ms(energy),
        interval_ms: still,
        layers: 0,
        lines: 0,
    };
    force(least, facultates, mundus, energy)
}

/// The score of a regimen (REQ-C-03): 8·value − cost in `i64`, the order of value − λ·cost with
/// λ = 1/8.
fn score(regimen: &Regimen, mundus: &Mundus, facultates: &Facultates, energy: u8) -> i64 {
    i64::from(value(regimen, mundus, facultates, energy))
        .saturating_mul(LAMBDA_INVERSE)
        .saturating_sub(i64::from(cost(regimen, facultates)))
}

/// The neighbour of a regimen in one dimension (Kirkpatrick et al., 1983; REQ-C-03), drawn by
/// `pick`: 0 the stilus of discriminant `pick` mod 4; 1 the strata 1 + `pick` mod 4; 2 the
/// interval of [`INTERVALS`] at `pick` mod 5; 3 the layers with the bit `pick` mod 8 flipped;
/// otherwise the lines `pick` mod 7. The forcings come after.
fn neighbour(regimen: Regimen, dimension: u16, pick: u16) -> Regimen {
    match dimension {
        0 => Regimen {
            stilus: STILI
                .into_iter()
                .nth(usize::from(pick % 4))
                .unwrap_or(regimen.stilus),
            ..regimen
        },
        1 => Regimen {
            strata: ((pick % 4) as u8).wrapping_add(1),
            ..regimen
        },
        2 => Regimen {
            interval_ms: INTERVALS
                .into_iter()
                .nth(usize::from(pick % 5))
                .unwrap_or(regimen.interval_ms),
            ..regimen
        },
        3 => Regimen {
            layers: regimen.layers ^ 1u8.wrapping_shl(u32::from(pick % 8)),
            ..regimen
        },
        _ => Regimen {
            lines: (pick % 7) as u8,
            ..regimen
        },
    }
}

/// The entry of [`EXP`] that a proposal worse by `delta` meets at the temperature
/// `temperature` (Metropolis et al., 1953; REQ-C-03): k = min(15, ⌊4·Δ/T⌋), the byte
/// ⌊255·e^(−k/4)⌋ ≈ 255·e^(−Δ/T). A temperature of 0 meets the last entry.
fn acceptance(delta: i64, temperature: i64) -> u32 {
    let k = delta
        .saturating_mul(EXP_PER_UNIT)
        .checked_div(temperature)
        .map_or(EXP_LAST, |q| q.clamp(0, EXP_LAST));
    byte(EXP, k as u32)
}

/// Whether a value reaches the level of aspiration, 80 % of the maximum (Simon, 1955;
/// REQ-C-03), compared exactly: 5·value ≥ 4·maximum.
const fn satisfices(value: u32, maximum: u32) -> bool {
    let (num, den) = ASPIRATION;
    (value as u64).wrapping_mul(den as u64) >= (maximum as u64).wrapping_mul(num as u64)
}

/// The regimen of a host for a world and an energy (REQ-C-03, REQ-C-04): a satisficing search
/// (Simon, 1955) by simulated annealing (Kirkpatrick et al., 1983) over the regimina that the
/// forcings leave.
///
/// 1. The floor ([`floor`]) is returned when the budget ([`budget`]) is zero or even the floor
///    exceeds it.
/// 2. The base is Creta with four strata, six lines, the floor of the interval and the layers
///    that carry value in the situation, under the forcings. The search starts at the base when
///    it fits the budget, and at the floor otherwise.
/// 3. The score is 8·value − cost (λ = 1/8); the temperature is T = `energy`/16.
/// 4. Up to 64 proposals, each drawn from the Galois generator g started at `semen`: the search
///    stops when the best regimen reaches 80 % of the maximum ([`value_max`]); g advances and
///    g mod 5 names the dimension; g advances and g ≫ 4 picks the new value;
///    the forcings apply. A proposal equal to the present regimen or above the budget is
///    skipped. A proposal no worse is accepted; a proposal worse by Δ is accepted, at T > 0, when
///    the low byte of the advanced generator falls below the entry of [`EXP`] for Δ and T
///    (Metropolis et al., 1953), and never at T = 0.
/// 5. The result is the regimen of the best score accepted.
#[must_use]
pub fn consilium(facultates: &Facultates, mundus: &Mundus, energy: u8) -> Regimen {
    let least = floor(facultates, mundus, energy);
    let allowance = budget(facultates, energy);
    if allowance == 0 || cost(&least, facultates) > allowance {
        return least;
    }
    let base = force(
        Regimen {
            stilus: Stilus::Creta,
            strata: STRATA_MAX,
            tick_ms: band_tick_ms(energy),
            interval_ms: interval_floor(band_tick_ms(energy), facultates.max_hz),
            layers: salient_layers(mundus),
            lines: LINES_MAX,
        },
        facultates,
        mundus,
        energy,
    );
    let mut current = if cost(&base, facultates) <= allowance {
        base
    } else {
        least
    };
    let mut best = current;
    let maximum = value_max(mundus, facultates, energy);
    let temperature = i64::from(energy / TEMPERATURE_DIVISOR);
    let mut g = mundus.verbum.semen();
    for _ in 0..PROPOSALS {
        if satisfices(value(&best, mundus, facultates, energy), maximum) {
            break;
        }
        g = galois(g);
        let dimension = g % DIMENSIONS;
        g = galois(g);
        let proposal = force(
            neighbour(current, dimension, g >> 4),
            facultates,
            mundus,
            energy,
        );
        if proposal == current || cost(&proposal, facultates) > allowance {
            continue;
        }
        let gained = score(&proposal, mundus, facultates, energy);
        let delta = score(&current, mundus, facultates, energy).saturating_sub(gained);
        let accepted = if delta <= 0 {
            true
        } else if temperature > 0 {
            g = galois(g);
            u32::from(g & DRAW_MASK) < acceptance(delta, temperature)
        } else {
            false
        };
        if accepted {
            current = proposal;
            if gained > score(&best, mundus, facultates, energy) {
                best = proposal;
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arbitrium::{DAEMON, DIOGENES};
    use crate::concentus::{Tonus, concentus};
    use crate::hospes::Indicia;
    use crate::memoria::Memoria;
    use crate::natura::mix;
    use crate::signum::scaena::fnv1a;
    use crate::spatium::KNUTH64;
    use crate::verbum::Verbum;
    use std::vec::Vec;

    /// Energy bands fix the world's cadence (REQ-L1-07).
    #[test]
    fn regimen_energy_bands() {
        let cases = [
            (255, 42),
            (179, 42),
            (178, 125),
            (77, 125),
            (76, 250),
            (39, 250),
            (38, 60_000),
            (0, 60_000),
        ];
        for (e, t) in cases {
            assert_eq!(band_tick_ms(e), t, "energy {e}");
        }
        let r = consilium(&Facultates::default(), &Mundus::genesis(1), 20);
        assert_eq!((r.stilus, r.tick_ms), (Stilus::Bicolor, 60_000));
    }

    /// Forcings in their precedence (REQ-C-04).
    #[test]
    fn consilium_forcings() {
        let w = Mundus::genesis(1);
        let cells = Facultates {
            cells: true,
            color_bits: 1,
            ..Facultates::default()
        };
        assert_eq!(consilium(&cells, &w, 255).stilus, Stilus::Littera);
        let mono = Facultates {
            color_bits: 1,
            ..Facultates::default()
        };
        assert_eq!(consilium(&mono, &w, 255).stilus, Stilus::Bicolor);
        let mut quies = w;
        quies.verbum = quies.verbum.with_modus(Modus::Quies);
        let q = consilium(&Facultates::default(), &quies, 255);
        assert_eq!(
            (q.stilus, q.tick_ms, q.interval_ms),
            (Stilus::Bicolor, 60_000, 60_000)
        );
        let mut dream = w;
        dream.verbum = dream.verbum.with_modus(Modus::Somnium);
        let d = consilium(&Facultates::default(), &dream, 200);
        assert!(d.strata >= 2 && d.stilus != Stilus::Littera);
        assert_eq!(
            consilium(
                &Facultates {
                    cells: true,
                    ..Facultates::default()
                },
                &dream,
                200
            )
            .stilus,
            Stilus::Littera
        );
        let still = consilium(
            &Facultates {
                max_hz: 0,
                ..Facultates::default()
            },
            &w,
            255,
        );
        assert_eq!((still.interval_ms, still.tick_ms), (60_000, 42));
        let mut amor = w;
        amor.verbum = amor.verbum.with_modus(Modus::AmorFati);
        assert_eq!(
            consilium(&Facultates::default(), &amor, 255).stilus,
            Stilus::Creta
        );
    }

    /// Same inputs, same regimen (REQ-C-03).
    #[test]
    fn consilium_deterministic() {
        let f = Facultates {
            width: 1920,
            height: 1080,
            ..Facultates::default()
        };
        for seed in [1u64, 2, 0xBEEF] {
            let w = Mundus::genesis(seed);
            assert_eq!(consilium(&f, &w, 150), consilium(&f, &w, 150));
        }
    }

    /// The chosen regimen fits the budget, or is the floor when nothing fits; the presentation
    /// interval never undercuts the host's rate (REQ-C-03, REQ-L1-07).
    #[test]
    fn consilium_respects_budget() {
        for (w, h) in [(272u16, 60u16), (320, 180), (640, 360), (1920, 1080)] {
            for energy in [0u8, 38, 39, 76, 77, 178, 179, 255] {
                for compute in [0u8, 96, 255] {
                    for max_hz in [0u8, 1, 24, 60] {
                        let f = Facultates {
                            width: w,
                            height: h,
                            compute,
                            max_hz,
                            ..Facultates::default()
                        };
                        let world = Mundus::genesis(u64::from(w) ^ u64::from(energy) << 16);
                        let r = consilium(&f, &world, energy);
                        assert!(
                            cost(&r, &f) <= budget(&f, energy) || r == floor(&f, &world, energy)
                        );
                        assert!(r.interval_ms >= r.tick_ms);
                        if max_hz == 0 {
                            assert!(r.interval_ms >= 60_000);
                        } else {
                            assert!(r.interval_ms >= 1000u32.div_ceil(u32::from(max_hz)));
                        }
                    }
                }
            }
        }
    }

    /// A zero allowance returns the floor even when truncation gives richer regimina a zero
    /// cost, whether the host has no compute or no energy (REQ-C-03).
    #[test]
    fn consilium_zero_budget_is_floor() {
        for (width, height) in [(320, 180), (1920, 1080)] {
            for modus in [
                Modus::Vigilia,
                Modus::Quies,
                Modus::Somnium,
                Modus::AmorFati,
            ] {
                let w = world(modus);
                for (compute, energy) in [(0, 255), (255, 0), (0, 0), (1, 1)] {
                    for cells in [false, true] {
                        let f = Facultates {
                            cells,
                            ..host(width, height, compute, 24)
                        };
                        assert_eq!(budget(&f, energy), 0);
                        assert_eq!(consilium(&f, &w, energy), floor(&f, &w, energy));
                    }
                }
            }
        }
    }

    /// A host whose complete base fits the budget reaches the aspiration immediately
    /// (REQ-C-03); 160×90 admits that base with the calibrated costs.
    #[test]
    fn consilium_reaches_aspiration() {
        let f = host(160, 90, 255, 24);
        let w = Mundus::genesis(1);
        let r = consilium(&f, &w, 255);
        assert_eq!(value(&r, &w, &f, 255), 45);
        assert!(value(&r, &w, &f, 255) * 5 >= value_max(&w, &f, 255) * 4);
        assert!(cost(&r, &f) <= budget(&f, 255));
    }

    /// The independent model retains the 320×180 case: 64 proposals may finish below
    /// aspiration even though an admissible regimen attains it (REQ-C-03).
    #[test]
    fn consilium_calibrated_search_can_miss_aspiration() {
        let f = Facultates::default();
        let w = Mundus::genesis(1);
        let r = consilium(&f, &w, 255);
        assert_eq!(
            r,
            Regimen {
                stilus: Stilus::Littera,
                strata: 4,
                tick_ms: 42,
                interval_ms: 60_000,
                layers: 181,
                lines: 6,
            }
        );
        assert_eq!((value(&r, &w, &f, 255), value_max(&w, &f, 255)), (30, 45));
        assert!(cost(&r, &f) <= budget(&f, 255));
        let attainable = Regimen {
            stilus: Stilus::Creta,
            strata: 4,
            tick_ms: 42,
            interval_ms: 125,
            layers: CAELUM | MARGO | ANNALES | HEXAGRAMMA | PUNCTA,
            lines: 6,
        };
        assert!(cost(&attainable, &f) <= budget(&f, 255));
        assert_eq!(value(&attainable, &w, &f, 255), 43);
        assert!(satisfices(
            value(&attainable, &w, &f, 255),
            value_max(&w, &f, 255)
        ));
    }

    /// The bytes of a packed table, from bit 0.
    fn bytes(table: u128, count: u32) -> Vec<u32> {
        (0..count).map(|k| byte(table, k)).collect()
    }

    /// The world of the first seed in the mode `modus`.
    fn world(modus: Modus) -> Mundus {
        let mut w = Mundus::genesis(1);
        w.verbum = w.verbum.with_modus(modus);
        w
    }

    /// A host of `width`×`height` pixels, compute `compute` and rate `max_hz`; the other
    /// capabilities by default.
    fn host(width: u16, height: u16, compute: u8, max_hz: u8) -> Facultates {
        Facultates {
            width,
            height,
            max_hz,
            compute,
            ..Facultates::default()
        }
    }

    /// Every regimen of the space of `docs/06-consilium.md` §2, with the cadence of the world
    /// `tick_ms`: 4 stili, 4 strata, 5 intervals, 256 sets of layers and 7 counts of lines,
    /// 143 360 in all.
    fn space(tick_ms: u32) -> impl Iterator<Item = Regimen> {
        STILI.into_iter().flat_map(move |stilus| {
            (1..=STRATA_MAX).flat_map(move |strata| {
                INTERVALS.into_iter().flat_map(move |interval_ms| {
                    (0..=u8::MAX).flat_map(move |layers| {
                        (0..=LINES_MAX).map(move |lines| Regimen {
                            stilus,
                            strata,
                            tick_ms,
                            interval_ms,
                            layers,
                            lines,
                        })
                    })
                })
            })
        })
    }

    /// The cost of `docs/06-consilium.md` §3 read from its table by name, with the scale
    /// s = max(1, min(W/320, H/180)) of REQ-L2-11: an independent statement of the law.
    fn cost_reference(regimen: &Regimen, host: &Facultates) -> u32 {
        let mut units = 10 + 18 * u64::from(regimen.strata.saturating_sub(1));
        for (layer, work) in [
            (CAELUM, 2),
            (MARGO, 5),
            (PERSONAE, 1),
            (RUINAE, 2),
            (IMPOSSIBILIA, 1),
            (HEXAGRAMMA, 1),
            (PUNCTA, 0),
        ] {
            if regimen.has(layer) {
                units += work;
            }
        }
        if regimen.has(ANNALES) {
            units += u64::from(regimen.lines);
        }
        units += match regimen.stilus {
            Stilus::Littera => 34,
            Stilus::Bicolor => 3,
            Stilus::Creta | Stilus::Musivum => 0,
        };
        let (width, height) = (u64::from(host.width), u64::from(host.height));
        let scale = (width / 320).min(height / 180).max(1);
        let area = if regimen.stilus == Stilus::Musivum {
            (width / scale) * (height / scale)
        } else {
            width * height
        };
        if regimen.interval_ms == 0 {
            return u32::MAX;
        }
        u32::try_from(units * area * 1000 / (57_600 * u64::from(regimen.interval_ms)))
            .unwrap_or(u32::MAX)
    }

    /// The value of `docs/06-consilium.md` §4 read from its table by name: an independent
    /// statement of the law.
    fn value_reference(r: &Regimen, w: &Mundus, energy: u8) -> u32 {
        let (v, m) = (w.verbum, w.memoria);
        let active = m.ramus() != 3;
        let dream = v.modus() == Modus::Somnium;
        let quiet = v.modus() == Modus::Quies;
        let pick = |doubled: bool, single: u32| if doubled { 2 * single } else { single };
        let mut total = 0u32;
        if r.has(CAELUM) {
            total += pick(active, 8);
        }
        if r.has(MARGO) {
            total += pick(v.onus() > 128, 5);
        }
        if r.has(ANNALES) {
            total += u32::from(r.lines) * pick(active, 2);
        }
        if r.has(PERSONAE) && (13..=15).contains(&m.eventus()) {
            total += 6;
        }
        if r.has(RUINAE) && (m.clausurae() >> 1) != 0 {
            total += 3;
        }
        if r.has(IMPOSSIBILIA) && ((v.conatus() > 200 && v.onus() > 200) || dream) {
            total += 7;
        }
        if r.has(HEXAGRAMMA) {
            total += pick(v.tranquillitas() >= 48, 2);
        }
        if r.has(PUNCTA) {
            total += pick(dream, 2);
        }
        total += u32::from(r.strata.saturating_sub(1)) * pick(dream, 2);
        total += match r.interval_ms {
            0..=42 => {
                if energy >= 179 {
                    6
                } else {
                    0
                }
            }
            43..=125 => 4,
            126..=250 => 2,
            251..=1000 => 1,
            _ => 0,
        };
        total += match r.stilus {
            Stilus::Creta => pick(v.modus() == Modus::AmorFati, 4),
            Stilus::Musivum => 3,
            Stilus::Littera if (39..=76).contains(&energy) && v.tranquillitas() >= 48 => 6,
            Stilus::Littera => 2,
            Stilus::Bicolor if quiet => 6,
            Stilus::Bicolor => 1,
        };
        total
    }

    /// The numbers of the laws are the derivations recorded beside them (REQ-L1-07,
    /// REQ-C-02..04; `docs/06-consilium.md` §2..§5).
    #[test]
    fn law_constants() {
        assert_eq!(
            INTERVALS,
            [1000u32.div_ceil(24), 1000 / 8, 1000 / 4, 1000, 60 * 1000]
        );
        assert_eq!(
            [BAND_42_ENERGY, BAND_125_ENERGY, BAND_250_ENERGY].map(u32::from),
            [70u32, 30, 15].map(|percent| (255 * percent).div_ceil(100))
        );
        assert_eq!(255 * 255 / BUDGET_DIVISOR, 1016);
        assert_eq!(96 * 77 / BUDGET_DIVISOR, 115);
        assert_eq!(UNIT_AREA, 320 * 180);
        assert_eq!(compositio(320, 180, LINES_MAX).s, 1);
        assert_eq!(MS_PER_S, 1000);
        assert_eq!((BASE_COST, STRATUM_COST, STRATUM_VALUE), (10, 18, 2));
        assert_eq!(bytes(LAYER_COSTS.into(), 9), [2, 5, 1, 1, 2, 1, 1, 0, 0]);
        assert_eq!(bytes(STILUS_COSTS.into(), 5), [0, 0, 34, 3, 0]);
        assert_eq!(bytes(LAYER_VALUES.into(), 9), [8, 5, 2, 6, 3, 7, 2, 2, 0]);
        assert_eq!(bytes(CADENCE_VALUES.into(), 6), [6, 4, 2, 1, 0, 0]);
        assert_eq!(bytes(STILUS_VALUES.into(), 5), [4, 3, 2, 1, 0]);
        assert_eq!(bytes(PREFERRED_VALUES.into(), 5), [4 + 4, 3, 6, 6, 0]);
        assert_eq!(WILT_ONUS, 1 << 7);
        let tone = |onus: u8| {
            let mut w = world(Modus::Vigilia);
            w.verbum = w.verbum.with_onus(onus);
            concentus(
                &w,
                &Facultates::default(),
                &Indicia::default(),
                &Regimen::default(),
            )
            .tonus
        };
        assert_eq!(
            (tone(EXCESS), tone(EXCESS + 1)),
            (Tonus::Ardesia, Tonus::Sepia)
        );
        let serenities = u32::from(Verbum::new(0).with_tranquillitas(u8::MAX).tranquillitas()) + 1;
        assert_eq!(u32::from(SERENITY), serenities * 3 / 4);
        assert_eq!((PERSONA_FIRST, PERSONA_LAST), (DIOGENES, DAEMON));
        assert_eq!((STRATA_MAX, DREAM_STRATA, LINES_MAX), (4, 2, 6));
        assert_eq!(STILI.map(|s| s as u8), [0, 1, 2, 3]);
        assert_eq!(DIMENSIONS, 5);
        assert_eq!(
            STILI.len() * usize::from(STRATA_MAX) * INTERVALS.len() * 256 * 7,
            143_360
        );
        assert_eq!((PROPOSALS, LAMBDA_INVERSE, ASPIRATION), (64, 8, (4, 5)));
        assert_eq!(i64::from(u8::MAX / TEMPERATURE_DIVISOR), EXP_LAST);
        assert_eq!((EXP_PER_UNIT, EXP_LAST), (4, 15));
        assert_eq!(u32::from(DRAW_MASK), byte(EXP, 0));
    }

    /// The acceptance table holds ⌊255·e^(−k/4)⌋ for k = 0..=15, and a proposal worse by Δ at
    /// the temperature T meets the entry k = min(15, ⌊4·Δ/T⌋), within the rounding of e^(−Δ/T)
    /// (Metropolis et al., 1953; REQ-C-03).
    #[test]
    fn annealing_acceptance() {
        let table = [
            255, 198, 154, 120, 93, 73, 56, 44, 34, 26, 20, 16, 12, 9, 7, 5,
        ];
        assert_eq!(bytes(EXP, 17), [&table[..], &[0]].concat());
        for k in 0..16u32 {
            let exact = (255.0 * (-f64::from(k) / 4.0).exp()).floor();
            assert_eq!(f64::from(byte(EXP, k)), exact, "k {k}");
            assert_eq!(byte(EXP, k), ((EXP >> (8 * k)) & 0xFF) as u32);
        }
        for t in 1..=15i64 {
            for d in 1..=400i64 {
                let k = (4 * d / t).min(15);
                assert_eq!(acceptance(d, t), byte(EXP, k as u32), "Δ {d} T {t}");
                assert!(acceptance(d + 1, t) <= acceptance(d, t));
                assert!(t == 15 || acceptance(d, t + 1) >= acceptance(d, t));
                let ideal = 255.0 * (-(d as f64) / (t as f64)).exp();
                let entry = f64::from(acceptance(d, t));
                assert!(entry >= ideal - 1.0, "Δ {d} T {t}");
                assert!(k == 15 || entry <= ideal * 0.25f64.exp(), "Δ {d} T {t}");
            }
        }
        assert_eq!(acceptance(i64::MAX, 1), 5);
        assert_eq!(acceptance(1, 0), 5);
    }

    /// The budget is ⌊`compute`·`energy`/64⌋ over the whole domain (`docs/06-consilium.md` §3).
    #[test]
    fn budget_law() {
        for compute in 0..=u8::MAX {
            for energy in 0..=u8::MAX {
                let f = Facultates {
                    compute,
                    ..Facultates::default()
                };
                assert_eq!(
                    budget(&f, energy),
                    u32::from(compute) * u32::from(energy) / 64
                );
            }
        }
        assert_eq!(budget(&Facultates::default(), 255), 1016);
        assert_eq!(budget(&host(320, 180, 96, 24), 77), 115);
    }

    /// The cost is the units of one frame at 320×180 scaled by the pixels of the stilus and by
    /// the frames per second, ⌊units·area·1000/(57 600·`interval_ms`)⌋, saturating at
    /// `u32::MAX` (`docs/06-consilium.md` §3).
    #[test]
    fn cost_law() {
        let small = Facultates::default();
        let large = host(1920, 1080, 255, 24);
        let base = Regimen {
            stilus: Stilus::Creta,
            strata: 4,
            tick_ms: 42,
            interval_ms: 42,
            layers: CAELUM | MARGO | ANNALES | HEXAGRAMMA | PUNCTA,
            lines: 6,
        };
        assert_eq!(cost(&base, &small), 78_000 / 42);
        let full = Regimen {
            layers: LAYERS_ALL,
            ..base
        };
        assert_eq!(cost(&full, &small), 82_000 / 42);
        let as_stilus = |stilus| Regimen { stilus, ..base };
        assert_eq!(cost(&as_stilus(Stilus::Littera), &small), 112_000 / 42);
        assert_eq!(cost(&as_stilus(Stilus::Bicolor), &small), 81_000 / 42);
        assert_eq!(cost(&base, &large), 78 * 36 * 1000 / 42);
        let mosaic = as_stilus(Stilus::Musivum);
        assert_eq!(cost(&mosaic, &large), cost(&mosaic, &small));
        assert_eq!(cost(&mosaic, &small), cost(&base, &small));
        assert_eq!(
            u64::from(cost(&mosaic, &host(1000, 500, 255, 24))),
            78 * 500 * 250 * 1000 / (57_600 * 42)
        );
        let silent = Regimen {
            layers: base.layers & !ANNALES,
            ..base
        };
        assert_eq!(
            cost(&silent, &small),
            cost(&Regimen { lines: 0, ..silent }, &small)
        );
        assert_eq!(
            cost(&Regimen { lines: 2, ..base }, &small),
            (78 - 4) * 1000 / 42
        );
        assert_eq!(
            cost(
                &Regimen {
                    interval_ms: 60_000,
                    ..base
                },
                &small
            ),
            1
        );
        assert_eq!(
            cost(
                &Regimen {
                    interval_ms: 0,
                    ..base
                },
                &small
            ),
            u32::MAX
        );
        let vast = host(u16::MAX, u16::MAX, 255, 255);
        let heaviest = Regimen {
            stilus: Stilus::Bicolor,
            strata: u8::MAX,
            tick_ms: 42,
            interval_ms: 1,
            layers: LAYERS_ALL,
            lines: u8::MAX,
        };
        assert_eq!(units(&heaviest), 4852);
        assert_eq!(cost(&heaviest, &vast), u32::MAX);
        assert_eq!(cost(&base, &host(0, 0, 255, 24)), 0);
        let hosts = [
            small,
            large,
            vast,
            host(272, 60, 96, 1),
            host(1000, 500, 40, 60),
            host(3840, 2160, 255, 60),
            host(0, 0, 255, 24),
        ];
        let mut g = 1u16;
        for _ in 0..8192 {
            g = galois(g);
            let a = u64::from(g) | u64::from(galois(g)) << 16;
            g = galois(galois(g));
            let r = Regimen {
                stilus: STILI[(a & 3) as usize],
                strata: (a >> 2) as u8 % 9,
                tick_ms: 42,
                interval_ms: [0, 1, 42, 125, 250, 999, 1000, 60_000, 70_000][(a >> 6) as usize % 9],
                layers: (a >> 10) as u8,
                lines: (a >> 18) as u8 % 9,
            };
            for f in &hosts {
                assert_eq!(cost(&r, f), cost_reference(&r, f), "{r:?} on {f:?}");
            }
        }
    }

    /// A regimen of Musivum with one stratum, a minute between frames, the layers `layers` and
    /// `lines` lines: the value of each term apart from the others.
    fn only(layers: u8, lines: u8) -> Regimen {
        Regimen {
            stilus: Stilus::Musivum,
            strata: 1,
            tick_ms: 42,
            interval_ms: 60_000,
            layers,
            lines,
        }
    }

    /// The world of the first seed with its words rewritten by `edit`.
    fn situation(edit: impl Fn(Verbum, Memoria) -> (Verbum, Memoria)) -> Mundus {
        let w = world(Modus::Vigilia);
        let (verbum, memoria) = edit(w.verbum, w.memoria);
        Mundus { verbum, memoria }
    }

    /// The value that the layers `layers` with `lines` lines add in the world `w`.
    fn layer_value(w: &Mundus, layers: u8, lines: u8) -> u32 {
        let f = Facultates::default();
        value(&only(layers, lines), w, &f, 255) - value(&only(0, lines), w, &f, 255)
    }

    /// Each layer carries its value of `docs/06-consilium.md` §4 in the situations that give it
    /// value, doubled in the situations that double it.
    #[test]
    fn value_of_layers() {
        let calm = world(Modus::Vigilia);
        let active = situation(|v, m| (v, m.with_ramus(0)));
        assert_eq!(
            (
                layer_value(&calm, CAELUM, 0),
                layer_value(&active, CAELUM, 0)
            ),
            (8, 16)
        );
        assert_eq!(
            (
                layer_value(&calm, ANNALES, 6),
                layer_value(&active, ANNALES, 6)
            ),
            (12, 24)
        );
        assert_eq!(layer_value(&calm, ANNALES, 0), 0);
        let weight = |onus: u8| situation(|v, m| (v.with_onus(onus), m));
        assert_eq!(
            (
                layer_value(&weight(128), MARGO, 0),
                layer_value(&weight(129), MARGO, 0)
            ),
            (5, 10)
        );
        let event = |eventus: u8| situation(|v, m| (v, m.with_eventus(eventus)));
        assert_eq!(
            [12, 13, 14, 15].map(|e| layer_value(&event(e), PERSONAE, 0)),
            [0, 6, 6, 6]
        );
        let closures = |clausurae: u16| situation(|v, m| (v, m.with_clausurae(clausurae)));
        assert_eq!(
            [0b1, 0b10, 0b10_0000_0000_0000].map(|c| layer_value(&closures(c), RUINAE, 0)),
            [0, 3, 3]
        );
        let strained =
            |conatus: u8, onus: u8| situation(|v, m| (v.with_conatus(conatus).with_onus(onus), m));
        assert_eq!(
            [(200, 201), (201, 200), (201, 201)].map(|(c, o)| layer_value(
                &strained(c, o),
                IMPOSSIBILIA,
                0
            )),
            [0, 0, 7]
        );
        let dream = world(Modus::Somnium);
        assert_eq!(layer_value(&dream, IMPOSSIBILIA, 0), 7);
        let serene = |t: u8| situation(|v, m| (v.with_tranquillitas(t), m));
        assert_eq!(
            (
                layer_value(&serene(47), HEXAGRAMMA, 0),
                layer_value(&serene(48), HEXAGRAMMA, 0)
            ),
            (2, 4)
        );
        assert_eq!(
            (
                layer_value(&calm, PUNCTA, 0),
                layer_value(&dream, PUNCTA, 0)
            ),
            (2, 4)
        );
    }

    /// Strata are worth 2 each beyond the first, 4 in Somnium; the cadence 6, 4, 2, 1 and 0 by
    /// the interval, 0 for 42 ms below the band of 42 ms, and an interval between two entries is
    /// worth the slower one (`docs/06-consilium.md` §4).
    #[test]
    fn value_of_strata_and_cadence() {
        let f = Facultates::default();
        let calm = world(Modus::Vigilia);
        let dream = world(Modus::Somnium);
        let strata = |w: &Mundus, strata: u8| {
            value(
                &Regimen {
                    strata,
                    ..only(0, 0)
                },
                w,
                &f,
                255,
            ) - value(&only(0, 0), w, &f, 255)
        };
        assert_eq!(
            (strata(&calm, 4), strata(&dream, 4), strata(&dream, 2)),
            (6, 12, 4)
        );
        let cadence = |interval_ms: u32, energy: u8| {
            value(
                &Regimen {
                    interval_ms,
                    ..only(0, 0)
                },
                &calm,
                &f,
                energy,
            ) - value(&only(0, 0), &calm, &f, energy)
        };
        assert_eq!(INTERVALS.map(|i| cadence(i, 179)), [6, 4, 2, 1, 0]);
        assert_eq!(INTERVALS.map(|i| cadence(i, 178)), [0, 4, 2, 1, 0]);
        assert_eq!(
            [0, 43, 126, 251, 1001, 60_001].map(|i| cadence(i, 255)),
            [6, 4, 2, 1, 0, 0]
        );
    }

    /// Each stilus carries its value, and the preferred value in the situation that prefers it:
    /// Creta 8 in `Modus::AmorFati`, Littera 6 in the eidetic reduction, Bicolor 6 in Quies
    /// (`docs/06-consilium.md` §4, `docs/05-estetica.md` §9).
    #[test]
    fn value_of_stilus() {
        let f = Facultates::default();
        let stilus = |stilus: Stilus, w: &Mundus, energy: u8| {
            let with = |stilus| Regimen {
                stilus,
                ..only(0, 0)
            };
            value(&with(stilus), w, &f, energy) + 3 - value(&with(Stilus::Musivum), w, &f, energy)
        };
        let calm = world(Modus::Vigilia);
        assert_eq!(STILI.map(|s| stilus(s, &calm, 255)), [4, 3, 2, 1]);
        assert_eq!(stilus(Stilus::Creta, &world(Modus::AmorFati), 255), 8);
        assert_eq!(stilus(Stilus::Bicolor, &world(Modus::Quies), 255), 6);
        assert_eq!(
            (
                stilus(Stilus::Bicolor, &calm, 38),
                stilus(Stilus::Bicolor, &calm, 39)
            ),
            (1, 1)
        );
        let serene = |t: u8| situation(|v, m| (v.with_tranquillitas(t), m));
        assert_eq!(
            [(38, 48), (39, 48), (76, 48), (77, 48), (60, 47)].map(|(e, t)| stilus(
                Stilus::Littera,
                &serene(t),
                e
            )),
            [2, 6, 6, 2, 2]
        );
    }

    /// The value equals the table of `docs/06-consilium.md` §4 read by name, for arbitrary
    /// worlds, regimina inside and outside the space and every energy, and does not read the
    /// host (REQ-C-03).
    #[test]
    fn value_law() {
        let f = Facultates::default();
        let r = Regimen {
            layers: LAYERS_ALL,
            ..Regimen::default()
        };
        let calm = world(Modus::Vigilia);
        assert_eq!(
            value(&r, &calm, &f, 255),
            value(&r, &calm, &host(1920, 1080, 0, 0), 255)
        );
        let mut x = 0x2545_F491_4F6C_DD1Du64;
        for _ in 0..4096 {
            x = mix(x);
            let w = Mundus {
                verbum: Verbum::new(x),
                memoria: Memoria::new(mix(!x)),
            };
            let r = Regimen {
                stilus: STILI[(x >> 7) as usize & 3],
                strata: (x >> 9) as u8 % 9,
                tick_ms: 42,
                interval_ms: [0, 42, 100, 125, 250, 999, 1000, 60_000, 70_000]
                    [(x >> 13) as usize % 9],
                layers: (x >> 17) as u8,
                lines: (x >> 25) as u8 % 9,
            };
            let energy = (x >> 33) as u8;
            assert_eq!(
                value(&r, &w, &f, energy),
                value_reference(&r, &w, energy),
                "{r:?}"
            );
        }
    }

    /// The layers of the base are those that carry value in the situation, and the doubled
    /// layers follow their conditions (`docs/06-consilium.md` §4, §5).
    #[test]
    fn situation_layers() {
        let calm = world(Modus::Vigilia);
        let always = CAELUM | MARGO | ANNALES | HEXAGRAMMA | PUNCTA;
        assert_eq!(salient_layers(&calm), always);
        assert_eq!(doubled_layers(&calm), 0);
        let mut full = calm;
        full.verbum = full
            .verbum
            .with_onus(201)
            .with_conatus(201)
            .with_tranquillitas(48);
        full.memoria = full
            .memoria
            .with_ramus(2)
            .with_eventus(15)
            .with_clausurae(0b100);
        assert_eq!(salient_layers(&full), LAYERS_ALL);
        assert_eq!(doubled_layers(&full), CAELUM | ANNALES | MARGO | HEXAGRAMMA);
        let dream = world(Modus::Somnium);
        assert_eq!(salient_layers(&dream), always | IMPOSSIBILIA);
        assert_eq!(doubled_layers(&dream), PUNCTA);
        let f = Facultates::default();
        for w in [calm, full, dream] {
            let lines = |layers| Regimen {
                layers,
                ..Regimen::default()
            };
            for bit in 0..8 {
                let layer = 1u8 << bit;
                let worth = value(&lines(layer), &w, &f, 255) > value(&lines(0), &w, &f, 255);
                assert_eq!(salient_layers(&w) & layer != 0, worth, "layer {layer}");
            }
        }
    }

    /// The floor of the interval is the first entry of [`INTERVALS`] at or above the cadence of
    /// the world and the period of the host's fastest rate, and a minute for a host of static
    /// frames (REQ-L1-07, REQ-C-04).
    #[test]
    fn interval_floor_law() {
        assert_eq!(
            [24, 23, 4, 3, 1, 0].map(|hz| interval_floor(42, hz)),
            [42, 125, 250, 1000, 1000, 60_000]
        );
        assert_eq!(interval_floor(250, 255), 250);
        assert_eq!(interval_floor(60_000, 255), 60_000);
        assert_eq!(interval_floor(60_001, 255), 60_000);
        for tick in [42u32, 125, 250, 60_000] {
            for max_hz in 0..=u8::MAX {
                let floor = interval_floor(tick, max_hz);
                assert!(INTERVALS.contains(&floor) && floor >= tick);
                if max_hz == 0 {
                    assert_eq!(floor, 60_000);
                    continue;
                }
                let period = 1000u32.div_ceil(u32::from(max_hz));
                assert!(floor >= period, "tick {tick} max_hz {max_hz}");
                if let Some(&faster) = INTERVALS.iter().rev().find(|&&i| i < floor) {
                    assert!(faster < tick.max(period), "tick {tick} max_hz {max_hz}");
                }
            }
        }
    }

    /// The forcings in their precedence over hosts of cells and of one bit, the four modes, the
    /// edges of the bands of energy, the rates of the host and every stilus, strata and
    /// interval; a forced regimen is its own image (REQ-C-04, REQ-L1-07).
    #[test]
    fn forcings_precedence() {
        let modes = [
            Modus::Vigilia,
            Modus::Quies,
            Modus::Somnium,
            Modus::AmorFati,
        ];
        for cells in [false, true] {
            for color_bits in [1u8, 4, 8, 24] {
                for modus in modes {
                    for energy in [0u8, 38, 39, 76, 77, 178, 179, 255] {
                        for max_hz in [0u8, 1, 2, 23, 24, 60, 255] {
                            let f = Facultates {
                                color_bits,
                                max_hz,
                                cells,
                                ..Facultates::default()
                            };
                            let w = world(modus);
                            let quiet = modus == Modus::Quies || energy < 39;
                            let dream = modus == Modus::Somnium;
                            let tick = if quiet { 60_000 } else { band_tick_ms(energy) };
                            let rate = match max_hz {
                                0 => 60_000,
                                hz => 1000u32.div_ceil(u32::from(hz)),
                            };
                            let least = INTERVALS
                                .into_iter()
                                .find(|&i| i >= tick.max(rate))
                                .unwrap();
                            for stilus in STILI {
                                let expected = if cells {
                                    Stilus::Littera
                                } else if color_bits == 1 || quiet {
                                    Stilus::Bicolor
                                } else if dream && stilus == Stilus::Littera {
                                    Stilus::Creta
                                } else {
                                    stilus
                                };
                                for strata in 1..=4u8 {
                                    for interval_ms in [0u32, 42, 125, 250, 1000, 60_000, 70_000] {
                                        let r = Regimen {
                                            stilus,
                                            strata,
                                            tick_ms: 125,
                                            interval_ms,
                                            layers: 0b1010_0101,
                                            lines: 3,
                                        };
                                        let forced = force(r, &f, &w, energy);
                                        let want = Regimen {
                                            stilus: expected,
                                            strata: if dream { strata.max(2) } else { strata },
                                            tick_ms: tick,
                                            interval_ms: interval_ms.max(least),
                                            ..r
                                        };
                                        assert_eq!(forced, want, "{f:?} {modus:?} {energy}");
                                        assert_eq!(force(forced, &f, &w, energy), forced);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// The floor takes the stilus the forcings fix, otherwise Musivum, one stratum (two in
    /// Somnium), no layer, no line, a minute between frames and the cadence of its band (a
    /// minute at rest) (REQ-C-03, REQ-C-04).
    #[test]
    fn floor_regimen() {
        let least = |stilus, strata, tick_ms| Regimen {
            stilus,
            strata,
            tick_ms,
            interval_ms: 60_000,
            layers: 0,
            lines: 0,
        };
        let cells = Facultates {
            cells: true,
            ..Facultates::default()
        };
        let mono = Facultates {
            color_bits: 1,
            ..Facultates::default()
        };
        let small = Facultates::default();
        let cases = [
            (
                small,
                world(Modus::Vigilia),
                255,
                least(Stilus::Musivum, 1, 42),
            ),
            (
                small,
                world(Modus::Vigilia),
                60,
                least(Stilus::Musivum, 1, 250),
            ),
            (
                small,
                world(Modus::Vigilia),
                20,
                least(Stilus::Bicolor, 1, 60_000),
            ),
            (
                small,
                world(Modus::Quies),
                255,
                least(Stilus::Bicolor, 1, 60_000),
            ),
            (
                small,
                world(Modus::Somnium),
                150,
                least(Stilus::Musivum, 2, 125),
            ),
            (
                small,
                world(Modus::AmorFati),
                255,
                least(Stilus::Musivum, 1, 42),
            ),
            (
                cells,
                world(Modus::Somnium),
                200,
                least(Stilus::Littera, 2, 42),
            ),
            (
                cells,
                world(Modus::Quies),
                200,
                least(Stilus::Littera, 1, 60_000),
            ),
            (
                mono,
                world(Modus::AmorFati),
                255,
                least(Stilus::Bicolor, 1, 42),
            ),
            (
                host(320, 180, 255, 0),
                world(Modus::Vigilia),
                100,
                least(Stilus::Musivum, 1, 125),
            ),
        ];
        for (f, w, energy, expected) in cases {
            let r = floor(&f, &w, energy);
            assert_eq!(r, expected, "{f:?} {:?} {energy}", w.verbum.modus());
            assert_eq!(force(r, &f, &w, energy), r);
        }
        assert_eq!(
            cost(
                &floor(&small, &world(Modus::Vigilia), 255),
                &host(1920, 1080, 0, 24)
            ),
            0
        );
        let wide = Facultates {
            cells: true,
            ..host(3840, 2160, 255, 24)
        };
        assert_eq!(
            cost(&floor(&wide, &world(Modus::Vigilia), 255), &wide),
            44 * 144 / 60
        );
    }

    /// The maximum of the value is the largest value over the regimina the forcings leave, and
    /// the floor is the cheapest of them, for hosts and situations that move every forcing
    /// (REQ-C-03, REQ-C-04).
    #[test]
    fn value_max_law() {
        let mut serene = world(Modus::Vigilia);
        serene.verbum = serene.verbum.with_tranquillitas(48);
        let mut active = world(Modus::Vigilia);
        active.verbum = active
            .verbum
            .with_onus(201)
            .with_conatus(201)
            .with_tranquillitas(50);
        active.memoria = active
            .memoria
            .with_ramus(0)
            .with_eventus(13)
            .with_clausurae(0b10);
        let cells = Facultates {
            cells: true,
            ..host(3840, 2160, 255, 24)
        };
        let mono = Facultates {
            color_bits: 1,
            ..host(1920, 1080, 255, 24)
        };
        let small = Facultates::default();
        let cases = [
            (small, world(Modus::Vigilia), 255, 45, 0),
            (cells, world(Modus::Somnium), 200, 58, 62 * 144 / 60),
            (mono, world(Modus::AmorFati), 255, 42, 13 * 36 / 60),
            (small, world(Modus::Quies), 255, 41, 0),
            (host(1920, 1080, 255, 24), serene, 60, 45, 0),
            (host(320, 180, 255, 0), active, 100, 82, 0),
            (host(640, 360, 96, 1), world(Modus::Somnium), 150, 55, 0),
            (small, world(Modus::Vigilia), 20, 36, 0),
        ];
        for (f, w, energy, most, least) in cases {
            let (mut top, mut bottom) = (0, u32::MAX);
            for r in space(band_tick_ms(energy)) {
                let forced = force(r, &f, &w, energy);
                top = top.max(value(&forced, &w, &f, energy));
                bottom = bottom.min(cost(&forced, &f));
            }
            assert_eq!(
                (value_max(&w, &f, energy), top),
                (most, most),
                "{f:?} {energy}"
            );
            assert_eq!((cost(&floor(&f, &w, energy), &f), bottom), (least, least));
        }
    }

    /// A neighbour changes the one dimension that the draw names, to the value that `pick`
    /// selects (Kirkpatrick et al., 1983; REQ-C-03).
    #[test]
    fn neighbour_changes_one_dimension() {
        let r = Regimen {
            stilus: Stilus::Musivum,
            strata: 3,
            tick_ms: 125,
            interval_ms: 1000,
            layers: 0b0101_0011,
            lines: 4,
        };
        for pick in 0..4096u16 {
            let p = usize::from(pick);
            assert_eq!(
                neighbour(r, 0, pick),
                Regimen {
                    stilus: STILI[p % 4],
                    ..r
                }
            );
            assert_eq!(
                neighbour(r, 1, pick),
                Regimen {
                    strata: (p % 4) as u8 + 1,
                    ..r
                }
            );
            assert_eq!(
                neighbour(r, 2, pick),
                Regimen {
                    interval_ms: INTERVALS[p % 5],
                    ..r
                }
            );
            assert_eq!(
                neighbour(r, 3, pick),
                Regimen {
                    layers: r.layers ^ 1 << (p % 8),
                    ..r
                }
            );
            assert_eq!(
                neighbour(r, 4, pick),
                Regimen {
                    lines: (p % 7) as u8,
                    ..r
                }
            );
        }
    }

    /// The level of aspiration is compared exactly, 5·value ≥ 4·maximum, without rounding 80 %
    /// of the maximum down (Simon, 1955; REQ-C-03).
    #[test]
    fn aspiration_is_exact() {
        assert!(satisfices(36, 45) && !satisfices(35, 45));
        assert!(!satisfices(31, 39) && satisfices(32, 39));
        assert!(satisfices(0, 0) && satisfices(u32::MAX, u32::MAX));
        assert!(!satisfices(u32::MAX / 5 * 4 - 1, u32::MAX));
        let f = Facultates::default();
        let w = world(Modus::Vigilia);
        let r = Regimen::default();
        assert_eq!(
            score(&r, &w, &f, 255),
            8 * i64::from(value(&r, &w, &f, 255)) - i64::from(cost(&r, &f))
        );
    }

    /// Known regimina of the search, from an independent statement of the pseudo-code of
    /// `docs/06-consilium.md` §5. The cases cross every branch: worse proposals accepted and
    /// refused by the table, refusals at T = 0, proposals above the budget, a stop at the level of
    /// aspiration, the base kept at once and the floor above the budget (REQ-C-03).
    #[test]
    fn consilium_known_regimens() {
        let regimen = |stilus, strata, tick_ms, interval_ms, layers, lines| Regimen {
            stilus,
            strata,
            tick_ms,
            interval_ms,
            layers,
            lines,
        };
        let mut dream = Mundus::genesis(3);
        dream.verbum = dream.verbum.with_modus(Modus::Somnium);
        let mut serene = Mundus::genesis(9);
        serene.verbum = serene.verbum.with_tranquillitas(50);
        let mut amor = Mundus::genesis(11);
        amor.verbum = amor.verbum.with_modus(Modus::AmorFati);
        let first = Mundus::genesis(1);
        let terminal = Facultates {
            cells: true,
            ..host(1280, 720, 96, 24)
        };
        let panel = Facultates {
            color_bits: 1,
            ..host(3840, 2160, 10, 60)
        };
        let cases = [
            (
                host(1920, 1080, 255, 24),
                first,
                150,
                (Stilus::Musivum, 4, 125, 141, 6),
            ),
            (
                host(1280, 720, 40, 60),
                first,
                12,
                (Stilus::Bicolor, 1, 60_000, 235, 0),
            ),
            (
                host(640, 360, 96, 24),
                first,
                200,
                (Stilus::Creta, 4, 42, 165, 6),
            ),
            (
                host(1920, 1080, 255, 24),
                first,
                12,
                (Stilus::Bicolor, 4, 60_000, 203, 0),
            ),
            (
                host(320, 180, 40, 24),
                dream,
                200,
                (Stilus::Creta, 4, 42, 199, 6),
            ),
            (terminal, serene, 60, (Stilus::Littera, 4, 250, 134, 6)),
            (
                host(1920, 1080, 255, 24),
                amor,
                255,
                (Stilus::Musivum, 3, 42, 221, 6),
            ),
            (panel, first, 150, (Stilus::Bicolor, 1, 125, 0, 0)),
        ];
        for (f, w, energy, (stilus, strata, tick_ms, layers, lines)) in cases {
            let expected = regimen(stilus, strata, tick_ms, 60_000, layers, lines);
            assert_eq!(consilium(&f, &w, energy), expected, "{f:?} {energy}");
        }
    }

    /// The regimina of 1 568 searches, arbitrary worlds from the mixing function on seven hosts
    /// at seven energies, hash to the value of an independent statement of the pseudo-code of
    /// `docs/06-consilium.md` §5 (FNV-1a over stilus, strata, the two cadences in little-endian
    /// order, layers and lines) (REQ-C-03).
    #[test]
    fn consilium_sweep_hash() {
        let hosts = [
            (1920u16, 1080u16, 255u8, 24u8, false, 8u8),
            (640, 360, 96, 24, false, 8),
            (3840, 2160, 255, 60, false, 8),
            (1280, 720, 40, 60, true, 8),
            (320, 180, 255, 24, false, 1),
            (272, 60, 96, 1, false, 4),
            (3840, 2160, 10, 60, false, 1),
        ];
        let mut stream = Vec::new();
        for seed in 1u64..=32 {
            let w = Mundus {
                verbum: Verbum::new(mix(seed)),
                memoria: Memoria::new(mix(seed ^ KNUTH64)),
            };
            for (width, height, compute, max_hz, cells, color_bits) in hosts {
                let f = Facultates {
                    width,
                    height,
                    color_bits,
                    max_hz,
                    cells,
                    compute,
                    ..Facultates::default()
                };
                for energy in [12u8, 40, 60, 100, 150, 200, 255] {
                    let r = consilium(&f, &w, energy);
                    stream.extend([r.stilus as u8, r.strata]);
                    stream.extend(r.tick_ms.to_le_bytes());
                    stream.extend(r.interval_ms.to_le_bytes());
                    stream.extend([r.layers, r.lines]);
                }
            }
        }
        assert_eq!(stream.len(), 1568 * 12);
        assert_eq!(fnv1a(&stream), 0xE3C9_7CB2_8671_2F55);
    }

    /// The search satisfices: a start that reaches the level of aspiration is kept, the result
    /// never scores below its start, never exceeds the maximum of the value and is its own
    /// image under the forcings (Simon, 1955; REQ-C-03, REQ-C-04).
    #[test]
    fn consilium_satisfices() {
        let hosts = [
            host(320, 180, 255, 24),
            host(1920, 1080, 96, 60),
            host(640, 360, 40, 1),
            host(3840, 2160, 255, 0),
            Facultates {
                cells: true,
                ..host(1280, 720, 96, 24)
            },
        ];
        let mut x = 0x9E37_79B9_7F4A_7C15u64;
        for _ in 0..96 {
            x = mix(x);
            let w = Mundus {
                verbum: Verbum::new(x),
                memoria: Memoria::new(mix(!x)),
            };
            for f in &hosts {
                for energy in [0u8, 15, 16, 40, 90, 180, 255] {
                    let r = consilium(f, &w, energy);
                    let maximum = value_max(&w, f, energy);
                    assert_eq!(force(r, f, &w, energy), r);
                    assert!(value(&r, &w, f, energy) <= maximum);
                    let least = floor(f, &w, energy);
                    let allowance = budget(f, energy);
                    if allowance == 0 || cost(&least, f) > allowance {
                        assert_eq!(r, least);
                        continue;
                    }
                    assert!(cost(&r, f) <= allowance);
                    let base = force(
                        Regimen {
                            stilus: Stilus::Creta,
                            strata: 4,
                            tick_ms: band_tick_ms(energy),
                            interval_ms: 0,
                            layers: salient_layers(&w),
                            lines: 6,
                        },
                        f,
                        &w,
                        energy,
                    );
                    let start = if cost(&base, f) <= allowance {
                        base
                    } else {
                        least
                    };
                    assert!(score(&r, &w, f, energy) >= score(&start, &w, f, energy));
                    if satisfices(value(&start, &w, f, energy), maximum) {
                        assert_eq!(r, start);
                    }
                }
            }
        }
    }
}
