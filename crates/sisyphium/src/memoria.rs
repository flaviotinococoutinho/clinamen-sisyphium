//! `Memoria`: the 64-bit word that holds the past and the intent of the world (levels L0, L8).
//!
//! Layout (bit 0 = least significant), from `docs/04-arquitetura.md` §3.2:
//!
//! | bits    | field       | law |
//! |---------|-------------|-----|
//! | 0..=13  | `clausurae` | ring of cycle closures; bit 0 is the present cycle |
//! | 14..=15 | `quadrans`  | last screen quadrant touched |
//! | 16..=19 | `eventus`   | present event: 0..=7 clinamina, 11..=15 reserved |
//! | 20..=21 | `ramus`     | chosen branch 0..=2, or 3 while awaiting |
//! | 22..=31 | `gradus`    | age of the present event in ticks, saturating |
//! | 32..=63 | `oraculum`  | four rows of four 2-bit weights: the figure's model of the observer |
//!
//! Every 64-bit pattern is a valid memory (REQ-L0-02).

use crate::campi::{campi, campus, cum_campo};

/// Marker in `ramus` while the present event awaits its place on the slope.
pub const RAMUS_EXSPECTANS: u8 = 3;

/// The past and the intent of the world: one 64-bit word.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Memoria(u64);

impl Memoria {
    /// Builds a memory from raw bits; every pattern is valid.
    #[must_use]
    pub const fn new(bits: u64) -> Self {
        Self(bits)
    }

    /// The raw 64 bits.
    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Four rows of four 2-bit weights, bits 32..=63: the figure's model of the observer.
    #[must_use]
    pub const fn oraculum(self) -> u32 {
        campus(self.0, 32, 32) as u32
    }

    /// Returns the memory with the oracle replaced.
    #[must_use]
    pub const fn with_oraculum(self, value: u32) -> Self {
        Self::new(cum_campo(self.0, 32, 32, value as u64))
    }
}

campi! { Memoria {
    /// Ring of cycle closures, bits 0..=13: bit 0 is the present cycle; 1 = interrupted by the observer.
    clausurae / with_clausurae: u16 [0, 14],
    /// Last screen quadrant touched, bits 14..=15.
    quadrans / with_quadrans: u8 [14, 2],
    /// Present event, bits 16..=19: 0..=7 clinamina, 11..=15 reserved.
    eventus / with_eventus: u8 [16, 4],
    /// Chosen branch 0..=2, bits 20..=21; [`RAMUS_EXSPECTANS`] while awaiting.
    ramus / with_ramus: u8 [20, 2],
    /// Age of the present event in ticks, bits 22..=31, saturating at 1023.
    gradus / with_gradus: u16 [22, 10],
} }

#[cfg(test)]
mod tests {
    #![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
    use super::*;

    /// Deterministic sweep of 65 535 words: every field written is read back unchanged,
    /// and `with_*` followed by the accessor is the identity (REQ-L0-02).
    #[test]
    fn memoria_pack_unpack_identity() {
        let mut x: u64 = 0x2545_F491_4F6C_DD1D;
        for _ in 0..65_535u32 {
            x = x.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
            let m = Memoria::new(x);
            assert_eq!(m.bits(), x);
            assert_eq!(m.clausurae(), (x & 0x3FFF) as u16);
            assert_eq!(m.quadrans(), ((x >> 14) & 3) as u8);
            assert_eq!(m.eventus(), ((x >> 16) & 0xF) as u8);
            assert_eq!(m.ramus(), ((x >> 20) & 3) as u8);
            assert_eq!(m.gradus(), ((x >> 22) & 0x3FF) as u16);
            assert_eq!(m.oraculum(), (x >> 32) as u32);
            let w = m
                .with_clausurae(m.clausurae())
                .with_quadrans(m.quadrans())
                .with_eventus(m.eventus())
                .with_ramus(m.ramus())
                .with_gradus(m.gradus())
                .with_oraculum(m.oraculum());
            assert_eq!(w.bits(), x);
            let c = (x >> 7) as u16;
            assert_eq!(m.with_clausurae(c).clausurae(), c & 0x3FFF);
            assert_eq!(m.with_quadrans(c as u8).quadrans(), (c & 3) as u8);
            assert_eq!(m.with_eventus(c as u8).eventus(), (c & 0xF) as u8);
            assert_eq!(m.with_ramus(c as u8).ramus(), (c & 3) as u8);
            assert_eq!(m.with_gradus(c).gradus(), c & 0x3FF);
            assert_eq!(m.with_oraculum(x as u32).oraculum(), x as u32);
        }
    }

    /// The awaiting branch marker is 3.
    #[test]
    fn ramus_awaiting_marker() {
        assert_eq!(RAMUS_EXSPECTANS, 3);
        assert_eq!(Memoria::new(0).with_ramus(RAMUS_EXSPECTANS).ramus(), 3);
    }
}
