//! `Verbum`: the 64-bit word that holds the present state of the world (level L0).
//!
//! Layout (bit 0 = least significant), from `docs/04-arquitetura.md` §3.1:
//!
//! | bits   | field          | law |
//! |--------|----------------|-----|
//! | 0..=15 | `semen`        | Galois generator state; never zero |
//! | 16..=23| `conatus`      | vital effort |
//! | 24..=31| `onus`         | moral weight |
//! | 32..=39| `ascensus`     | progress on the slope |
//! | 40..=47| `curvatura`    | Gaussian curvature of space, signed |
//! | 48..=55| `automatum`    | ring of eight cells under rule 30 |
//! | 56..=61| `tranquillitas`| serenity, six bits |
//! | 62..=63| `modus`        | epistemic mode |
//!
//! Every 64-bit pattern is a valid world except `semen == 0`, which is normalized to
//! [`SEMEN_DEFAULT`] on construction (REQ-L0-01).

/// The seed that replaces `semen == 0`: a Galois generator state of zero would stay zero forever.
pub const SEMEN_DEFAULT: u16 = 0xACE1;

/// Epistemic mode of the world, bits 62..=63 (REQ-L4-10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Modus {
    /// Kinetic waking state: the default.
    Vigilia = 0,
    /// Stillness under scarce energy: a static, two-tone frame.
    Quies = 1,
    /// Dream: non-Euclidean geometry without notice, revealed afterwards.
    Somnium = 2,
    /// Affirmation: the figure kicks the boulder down by choice.
    AmorFati = 3,
}

impl Modus {
    /// Decodes the two low bits; higher bits are ignored.
    #[must_use]
    pub const fn from_bits(bits: u8) -> Self {
        match bits & 3 {
            0 => Self::Vigilia,
            1 => Self::Quies,
            2 => Self::Somnium,
            _ => Self::AmorFati,
        }
    }
}

use crate::campi::{campi, campus, cum_campo};

/// The present state of the world: one 64-bit word.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Verbum(u64);

impl Verbum {
    /// Builds a word from raw bits, normalizing `semen == 0` to [`SEMEN_DEFAULT`].
    #[must_use]
    pub const fn new(bits: u64) -> Self {
        if campus(bits, 0, 16) == 0 {
            Self(bits | SEMEN_DEFAULT as u64)
        } else {
            Self(bits)
        }
    }

    /// The raw 64 bits.
    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Epistemic mode, bits 62..=63.
    #[must_use]
    pub const fn modus(self) -> Modus {
        Modus::from_bits(campus(self.0, 62, 2) as u8)
    }

    /// Returns the word with the mode replaced.
    #[must_use]
    pub const fn with_modus(self, modus: Modus) -> Self {
        Self::new(cum_campo(self.0, 62, 2, modus as u64))
    }
}

campi! { Verbum {
    /// Galois generator state, bits 0..=15; never zero.
    semen / with_semen: u16 [0, 16],
    /// Vital effort, bits 16..=23 (Spinoza, Ethics III, prop. 6).
    conatus / with_conatus: u8 [16, 8],
    /// Moral weight, bits 24..=31.
    onus / with_onus: u8 [24, 8],
    /// Progress on the slope, bits 32..=39.
    ascensus / with_ascensus: u8 [32, 8],
    /// Gaussian curvature of space, bits 40..=47, signed: 0 flat, negative hyperbolic, positive elliptic.
    curvatura / with_curvatura: i8 [40, 8],
    /// Ring of eight cells under rule 30, bits 48..=55.
    automatum / with_automatum: u8 [48, 8],
    /// Serenity, bits 56..=61 (six bits; shown as a hexagram).
    tranquillitas / with_tranquillitas: u8 [56, 6],
} }

#[cfg(test)]
mod tests {
    #![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
    use super::*;

    /// Deterministic sweep of 65 535 words: every field written is read back unchanged,
    /// and `with_*` followed by the accessor is the identity (REQ-L0-01).
    #[test]
    fn verbum_pack_unpack_identity() {
        let mut semen = 1u16;
        let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
        for _ in 0..65_535u32 {
            semen = crate::natura::galois(semen);
            x = crate::natura::mix(x ^ u64::from(semen));
            let semen = (x & 0xFFFF) as u16;
            let semen = if semen == 0 { SEMEN_DEFAULT } else { semen };
            let v = Verbum::new(x);
            assert_eq!(v.semen(), semen);
            assert_eq!(v.conatus(), ((x >> 16) & 0xFF) as u8);
            assert_eq!(v.onus(), ((x >> 24) & 0xFF) as u8);
            assert_eq!(v.ascensus(), ((x >> 32) & 0xFF) as u8);
            assert_eq!(v.curvatura(), ((x >> 40) & 0xFF) as u8 as i8);
            assert_eq!(v.automatum(), ((x >> 48) & 0xFF) as u8);
            assert_eq!(v.tranquillitas(), ((x >> 56) & 0x3F) as u8);
            assert_eq!(v.modus() as u8, ((x >> 62) & 0x3) as u8);
            let w = v
                .with_semen(v.semen())
                .with_conatus(v.conatus())
                .with_onus(v.onus())
                .with_ascensus(v.ascensus())
                .with_curvatura(v.curvatura())
                .with_automatum(v.automatum())
                .with_tranquillitas(v.tranquillitas())
                .with_modus(v.modus());
            assert_eq!(w.bits(), v.bits());
            let c = ((x >> 8) & 0xFF) as u8;
            assert_eq!(v.with_conatus(c).conatus(), c);
            assert_eq!(v.with_onus(c).onus(), c);
            assert_eq!(v.with_ascensus(c).ascensus(), c);
            assert_eq!(v.with_curvatura(c as i8).curvatura(), c as i8);
            assert_eq!(v.with_automatum(c).automatum(), c);
            assert_eq!(v.with_tranquillitas(c).tranquillitas(), c & 0x3F);
            assert_eq!(
                v.with_modus(Modus::from_bits(c)).modus(),
                Modus::from_bits(c & 3)
            );
        }
    }

    /// `semen == 0` is never representable: `new` and `with_semen` normalize it (REQ-L0-01).
    #[test]
    fn zero_semen_normalized() {
        assert_eq!(Verbum::new(0).semen(), SEMEN_DEFAULT);
        assert_eq!(Verbum::new(0xFFFF_0000).semen(), SEMEN_DEFAULT);
        assert_eq!(Verbum::new(1).with_semen(0).semen(), SEMEN_DEFAULT);
        assert_eq!(Verbum::new(0).bits() & 0xFFFF, u64::from(SEMEN_DEFAULT));
    }

    /// `Modus` round-trips through its two bits.
    #[test]
    fn modus_bits_roundtrip() {
        for m in [
            Modus::Vigilia,
            Modus::Quies,
            Modus::Somnium,
            Modus::AmorFati,
        ] {
            assert_eq!(Modus::from_bits(m as u8), m);
        }
        assert_eq!(Modus::from_bits(7), Modus::AmorFati);
    }
}
