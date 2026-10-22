//! The chalk font: 5×7 glyphs in 6×10 cells with two rows for diacritics above and one below
//! (REQ-L5-04). Uppercase Latin letters, digits, punctuation, the Portuguese accented capitals,
//! the modal and set symbols and three Greek letters. Characters outside the table draw `·`.

/// Glyph order of the table: the index of a character is its position here.
pub const ORDO: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .,:;!?-+=/()[]'\"*#%@·□◊→¬∧∨∅Δ∞φδκ";
/// 5×7 glyphs, seven rows of five bits, row 0 (top) in the low bits, leftmost column in bit 4 of each row.
pub const GLYPHI: [u64; 70] = [
    0x0004_631F_C62E, // A
    0x0007_A31F_463E, // B
    0x0003_A308_422E, // C
    0x0007_A318_C63E, // D
    0x0007_E10F_421F, // E
    0x0004_210F_421F, // F
    0x0003_E31B_C22E, // G
    0x0004_631F_C631, // H
    0x0003_8842_108E, // I
    0x0003_2421_0847, // J
    0x0004_654C_5251, // K
    0x0007_E108_4210, // L
    0x0004_631A_D771, // M
    0x0004_633A_E631, // N
    0x0003_A318_C62E, // O
    0x0004_210F_463E, // P
    0x0003_6558_C62E, // Q
    0x0004_654F_463E, // R
    0x0007_8217_420F, // S
    0x0001_0842_109F, // T
    0x0003_A318_C631, // U
    0x0001_1518_C631, // V
    0x0002_AB5A_C631, // W
    0x0004_62A2_2A31, // X
    0x0001_0842_2A31, // Y
    0x0007_E082_083F, // Z
    0x0003_A39A_CE2E, // 0
    0x0003_8842_1184, // 1
    0x0007_D041_062E, // 2
    0x0003_A211_105F, // 3
    0x0000_85F9_28C2, // 4
    0x0003_A210_FA1F, // 5
    0x0003_A31F_4106, // 6
    0x0002_1082_083F, // 7
    0x0003_A317_462E, // 8
    0x0003_0417_C62E, // 9
    0x0000_0000_0000, // space
    0x0003_1800_0000, // .
    0x0002_08C0_0000, // ,
    0x0000_18C0_3180, // :
    0x0002_08C0_3180, // ;
    0x0001_0042_1084, // !
    0x0001_0041_062E, // ?
    0x0000_000F_8000, // -
    0x0000_084F_9080, // +
    0x0000_01F0_7C00, // =
    0x0000_2082_0820, // /
    0x0000_8884_2082, // (
    0x0002_0821_0888, // )
    0x0003_9084_210E, // [
    0x0003_8421_084E, // ]
    0x0000_0000_2084, // '
    0x0000_0000_294A, // "
    0x0000_144F_9140, // *
    0x0002_95F5_7D4A, // #
    0x0000_E682_0B38, // %
    0x0003_E17A_DE2E, // @
    0x0000_0002_0000, // ·
    0x0000_3F18_C7E0, // □
    0x0001_1518_C544, // ◊
    0x0000_082F_8880, // →
    0x0000_0010_FC00, // ¬
    0x0004_62A5_1080, // ∧
    0x0001_08A5_4620, // ∨
    0x0004_1D9A_CDC1, // ∅
    0x0007_E315_2884, // Δ
    0x0000_155A_A800, // ∞
    0x0001_08EA_D5C4, // φ
    0x0003_A317_1106, // δ
    0x0000_254C_5240, // κ
];
/// Diacritic acute: two rows above the glyph.
pub const ACUTE: u16 = 0x082;
/// Diacritic grave: two rows above the glyph.
pub const GRAVE: u16 = 0x088;
/// Diacritic circ: two rows above the glyph.
pub const CIRC: u16 = 0x144;
/// Diacritic tilde: two rows above the glyph.
pub const TILDE: u16 = 0x2CD;
/// Cedilla: one row below the glyph.
pub const CEDILLA: u8 = 0b00100;

use crate::signum::pixel::LABEL_NONE;
use crate::signum::raster::{Modo, Pictor};

/// Diacritic drawn with a glyph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Marca {
    /// None.
    Nulla,
    /// Acute accent above.
    Acute,
    /// Grave accent above.
    Grave,
    /// Circumflex above.
    Circ,
    /// Tilde above.
    Tilde,
    /// Cedilla below.
    Cedilla,
}

/// Index of a character in [`ORDO`].
fn index(c: char) -> Option<usize> {
    ORDO.chars().position(|o| o == c)
}

/// The accented capitals of REQ-L5-04, in the order of their cell glyph indices 70..=81.
pub const ACUTAE: &str = "ÁÀÂÃÉÊÍÓÔÕÚÇ";

/// Glyph index of a character in a cell (REQ-H-02): its position in [`ORDO`] (0..=69), else 70
/// plus its position in [`ACUTAE`] (70..=81), else the index of `·` (57).
#[must_use]
pub fn cell_glyph(c: char) -> u8 {
    let accented = || {
        ACUTAE
            .chars()
            .position(|a| a == c)
            .map(|i| i.wrapping_add(GLYPHI.len()))
    };
    index(c)
        .or_else(accented)
        .or_else(|| index('·'))
        .map_or(0, |i| i as u8)
}

/// The glyph and the mark of a character: accented capitals decompose into their base letter
/// and a mark; characters outside the table become `·`.
#[must_use]
pub fn glyph(c: char) -> (u64, Marca) {
    let (base, mark) = match c {
        'Á' => ('A', Marca::Acute),
        'À' => ('A', Marca::Grave),
        'Â' => ('A', Marca::Circ),
        'Ã' => ('A', Marca::Tilde),
        'É' => ('E', Marca::Acute),
        'Ê' => ('E', Marca::Circ),
        'Í' => ('I', Marca::Acute),
        'Ó' => ('O', Marca::Acute),
        'Ô' => ('O', Marca::Circ),
        'Õ' => ('O', Marca::Tilde),
        'Ú' => ('U', Marca::Acute),
        'Ç' => ('C', Marca::Cedilla),
        other => (other, Marca::Nulla),
    };
    let bits = index(base)
        .or_else(|| index('·'))
        .and_then(|i| GLYPHI.get(i).copied())
        .unwrap_or(0);
    (bits, mark)
}

/// Width in pixels of `glyphs` cells at scale `s`: 6·s per cell.
#[must_use]
pub const fn width(glyphs: u32, s: u16) -> u32 {
    glyphs.saturating_mul(6).saturating_mul(s as u32)
}

/// Draws one block of `scale`×`scale` pixels at a cell-relative column and row.
fn block(
    pictor: &mut Pictor,
    origin: (i32, i32),
    scale: u16,
    col: i32,
    row: i32,
    lum: u8,
    stratum: u8,
) {
    let scale = i32::from(scale.max(1));
    let px = origin.0.wrapping_add(col.wrapping_mul(scale));
    let py = origin.1.wrapping_add(row.wrapping_mul(scale));
    let mut dy = 0;
    while dy < scale {
        let mut dx = 0;
        while dx < scale {
            pictor.plot(
                px.wrapping_add(dx),
                py.wrapping_add(dy),
                lum,
                stratum,
                LABEL_NONE,
                Modo::Super,
            );
            dx = dx.wrapping_add(1);
        }
        dy = dy.wrapping_add(1);
    }
}

/// Draws `count` rows of five bits (leftmost column in bit 4 of each row) from `first_row`.
#[allow(clippy::too_many_arguments)]
fn draw_rows(
    pictor: &mut Pictor,
    origin: (i32, i32),
    scale: u16,
    bits: u64,
    count: u32,
    first_row: i32,
    lum: u8,
    stratum: u8,
) {
    let mut row = 0u32;
    while row < count {
        let line = (bits >> row.wrapping_mul(5)) & 0x1F;
        let mut col = 0u32;
        while col < 5 {
            if (line >> (4u32.wrapping_sub(col))) & 1 == 1 {
                block(
                    pictor,
                    origin,
                    scale,
                    col as i32,
                    first_row.wrapping_add(row as i32),
                    lum,
                    stratum,
                );
            }
            col = col.wrapping_add(1);
        }
        row = row.wrapping_add(1);
    }
}

/// Draws `text` in cells of 6×10·`scale` pixels from `(x, y)`: diacritics in rows 0..=1, the
/// glyph in rows 2..=8, the cedilla in row 9. Unknown characters draw `·` and keep the alignment.
pub fn text(pictor: &mut Pictor, x: i32, y: i32, scale: u16, text: &str, lum: u8, stratum: u8) {
    let pitch = i32::from(scale.max(1)).wrapping_mul(6);
    for (k, c) in (0i32..).zip(text.chars()) {
        let origin = (x.wrapping_add(k.wrapping_mul(pitch)), y);
        let (bits, mark) = glyph(c);
        draw_rows(pictor, origin, scale, bits, 7, 2, lum, stratum);
        match mark {
            Marca::Nulla => {}
            Marca::Acute => draw_rows(pictor, origin, scale, u64::from(ACUTE), 2, 0, lum, stratum),
            Marca::Grave => draw_rows(pictor, origin, scale, u64::from(GRAVE), 2, 0, lum, stratum),
            Marca::Circ => draw_rows(pictor, origin, scale, u64::from(CIRC), 2, 0, lum, stratum),
            Marca::Tilde => draw_rows(pictor, origin, scale, u64::from(TILDE), 2, 0, lum, stratum),
            Marca::Cedilla => draw_rows(
                pictor,
                origin,
                scale,
                u64::from(CEDILLA),
                1,
                9,
                lum,
                stratum,
            ),
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

    /// Every character of REQ-L5-04 has a glyph; accented capitals carry a mark over the base
    /// letter; anything else maps to `·`.
    #[test]
    fn font_coverage() {
        for c in ORDO.chars() {
            let (g, mark) = glyph(c);
            assert_eq!(mark, Marca::Nulla, "{c}");
            assert_eq!(g, GLYPHI[ORDO.chars().position(|o| o == c).unwrap()]);
        }
        let (dot, _) = glyph('·');
        for (c, base, mark) in [
            ('Á', 'A', Marca::Acute),
            ('À', 'A', Marca::Grave),
            ('Â', 'A', Marca::Circ),
            ('Ã', 'A', Marca::Tilde),
            ('É', 'E', Marca::Acute),
            ('Ê', 'E', Marca::Circ),
            ('Í', 'I', Marca::Acute),
            ('Ó', 'O', Marca::Acute),
            ('Ô', 'O', Marca::Circ),
            ('Õ', 'O', Marca::Tilde),
            ('Ú', 'U', Marca::Acute),
            ('Ç', 'C', Marca::Cedilla),
        ] {
            assert_eq!(glyph(c), (glyph(base).0, mark), "{c}");
        }
        for c in ['a', 'z', 'ß', '😀', '\n', 'é'] {
            assert_eq!(glyph(c), (dot, Marca::Nulla), "{c:?}");
        }
        assert_eq!(ORDO.chars().count(), GLYPHI.len());
    }

    /// Glyph indices of cells (REQ-H-02): the position in `ORDO` for its seventy characters, 70
    /// plus the position in `ACUTAE` for the twelve accented capitals, and 57 (`·`) otherwise.
    #[test]
    fn cell_glyph_accents() {
        let word: std::vec::Vec<u8> = "AÇÃO".chars().map(cell_glyph).collect();
        assert_eq!(word, [0, 81, 73, 14]);
        assert_eq!(cell_glyph('a'), 57);
        assert_eq!(cell_glyph('·'), 57);
        assert_eq!(ORDO.chars().position(|c| c == '·'), Some(57));
        for (i, c) in ORDO.chars().enumerate() {
            assert_eq!(usize::from(cell_glyph(c)), i, "{c}");
        }
        assert_eq!(ACUTAE, "ÁÀÂÃÉÊÍÓÔÕÚÇ");
        for (i, c) in ACUTAE.chars().enumerate() {
            assert_eq!(usize::from(cell_glyph(c)), 70 + i, "{c}");
            assert_ne!(glyph(c).1, Marca::Nulla, "{c} carries a mark");
        }
        for c in ['é', 'ß', '😀', '\n', 'ç'] {
            assert_eq!(cell_glyph(c), 57, "{c:?}");
        }
    }

    /// No glyph but the space is empty, and every glyph fits seven rows of five bits.
    #[test]
    fn font_no_empty_glyph() {
        for (c, g) in ORDO.chars().zip(GLYPHI) {
            assert_eq!(g >> 35, 0, "{c} exceeds 35 bits");
            if c == ' ' {
                assert_eq!(g, 0);
            } else {
                assert_ne!(g, 0, "{c} is empty");
            }
        }
    }

    /// Marks occupy the two rows above the glyph; the cedilla occupies the one row below.
    #[test]
    fn font_accents_two_rows() {
        for m in [ACUTE, GRAVE, CIRC, TILDE] {
            assert_eq!(m >> 10, 0);
            assert_ne!(m, 0);
        }
        assert_eq!(CEDILLA >> 5, 0);
        assert_ne!(CEDILLA, 0);
        let mut buf = [VOID; 6 * 10];
        let mut tab = Tabula::new(6, 10, &mut buf).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        text(&mut pictor, 0, 0, 1, "A", 15, 0);
        assert!(buf[..12].iter().all(|&p| p == VOID), "no mark above A");
        assert!(buf[54..60].iter().all(|&p| p == VOID), "nothing below A");
        let mut buf2 = [VOID; 6 * 10];
        let mut tab = Tabula::new(6, 10, &mut buf2).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        text(&mut pictor, 0, 0, 1, "Á", 15, 0);
        assert!(buf2[..12].iter().any(|&p| lum_of(p) == 15), "mark above Á");
        let mut buf3 = [VOID; 6 * 10];
        let mut tab = Tabula::new(6, 10, &mut buf3).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        text(&mut pictor, 0, 0, 1, "Ç", 15, 0);
        assert!(
            buf3[54..60].iter().any(|&p| lum_of(p) == 15),
            "cedilla below Ç"
        );
    }

    /// Cells are 6×10 times the scale; glyph rows are 2..=8; the width law.
    #[test]
    fn text_cell_pitch() {
        let mut buf = [VOID; 24 * 12];
        let mut tab = Tabula::new(24, 12, &mut buf).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        text(&mut pictor, 0, 0, 1, "LL", 15, 0);
        let lit = |x: usize, y: usize| lum_of(buf[x + y * 24]) == 15;
        assert!(lit(0, 2) && lit(0, 8) && lit(4, 8), "first L");
        assert!(lit(6, 2) && lit(6, 8) && lit(10, 8), "second L at pitch 6");
        assert!(!lit(5, 8) && !lit(11, 8), "gap column");
        assert!((0..24).all(|x| !lit(x, 9)), "row 9 empty");
        assert_eq!(width(2, 1), 12);
        assert_eq!(width(3, 4), 72);
        let mut buf2 = [VOID; 24 * 24];
        let mut tab = Tabula::new(24, 24, &mut buf2).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        text(&mut pictor, 0, 0, 2, "L", 15, 0);
        let lit2 = |x: usize, y: usize| lum_of(buf2[x + y * 24]) == 15;
        assert!(
            lit2(0, 4) && lit2(1, 5) && lit2(9, 17) && !lit2(10, 17),
            "scale 2 blocks"
        );
    }

    /// Unknown characters light exactly the dot, keeping the alignment of the following glyphs.
    #[test]
    fn font_unknown_maps_to_dot() {
        let mut buf = [VOID; 12 * 10];
        let mut tab = Tabula::new(12, 10, &mut buf).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        text(&mut pictor, 0, 0, 1, "aL", 15, 0);
        let lit: std::vec::Vec<(usize, usize)> = (0..120)
            .filter(|&i| lum_of(buf[i]) == 15)
            .map(|i| (i % 12, i / 12))
            .collect();
        assert!(lit.contains(&(2, 5)), "dot at column 2 row 3 of the cell");
        assert_eq!(
            lit.iter().filter(|&&(x, _)| x < 6).count(),
            1,
            "only the dot in the first cell"
        );
        assert!(lit.contains(&(6, 8)), "L still at the second cell");
    }
}
