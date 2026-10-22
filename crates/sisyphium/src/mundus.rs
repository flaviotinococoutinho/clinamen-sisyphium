//! `mundus`: the world as the pair of state words, its genesis and its persistence as 32
//! hexadecimal digits (REQ-L0-03, REQ-L0-04).

use crate::memoria::{Memoria, RAMUS_EXSPECTANS};
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
}

#[cfg(test)]
mod tests {
    #![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
    use super::*;
    use crate::memoria::RAMUS_EXSPECTANS;
    use crate::natura::{galois, mix};
    use crate::verbum::{Modus, SEMEN_DEFAULT};

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
}
