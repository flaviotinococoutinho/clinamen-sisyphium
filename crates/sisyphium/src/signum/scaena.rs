//! The scene composer: one frame from the world, the host, the regimen and the visible lines, in
//! the order of `docs/04-arquitetura.md` §5, and the four stili (REQ-L5-03, REQ-L5-09, REQ-H-04).

use crate::concentus::{Concentus, concentus};
use crate::corpus::{Figura, boulder_texture, draw_boulder, draw_figure, gait_phase, postura};
use crate::hospes::{
    ANNALES, CAELUM, Cellae, Facultates, HEXAGRAMMA, Indicia, MARGO, PUNCTA, RUINAE, Regimen,
    Stilus, Tabula,
};
use crate::mundus::Mundus;
use crate::signum::caelum::{constellation, hexagram, moon};
use crate::signum::littera::{ORDO, text};
use crate::signum::margo::draw_frame;
use crate::signum::mons::{draw_ruins, draw_slope};
use crate::signum::pixel::{VOID, label_of, lum_of, pixel, stratum_of};
use crate::signum::raster::{Modo, Pictor, bayer};
use crate::spatium::{Compositio, compositio, isqrt};

/// FNV-1a over 64 bits: the hash of a frame for the golden tests.
#[must_use]
pub fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xCBF2_9CE4_8422_2325u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01B3)
    })
}

/// Luminance of the text of the chronicles.
const LUM_TEXT: u8 = 13;
/// The glyph ramp of the Littera stilus, as indices in [`ORDO`]: ` .:-=+*#%@`.
const RAMP: [u8; 10] = [36, 37, 39, 43, 45, 44, 53, 54, 55, 56];
/// Index of `·` in [`ORDO`].
const DOT: u8 = 57;

/// The point of the slope at `t` (0 at the foot, 255 at the summit).
fn slope_point(comp: &Compositio, t: u8) -> (i32, i32) {
    let (foot, summit) = comp.slope;
    let along = i64::from(t);
    let x = i64::from(foot.x)
        .wrapping_add(i64::from(summit.x.wrapping_sub(foot.x)).wrapping_mul(along) / 255);
    let y = i64::from(foot.y)
        .wrapping_add(i64::from(summit.y.wrapping_sub(foot.y)).wrapping_mul(along) / 255);
    (x as i32, y as i32)
}

/// A point `distance` pixels up the slope from `from`.
fn along_slope(comp: &Compositio, from: (i32, i32), distance: i32) -> (i32, i32) {
    let (foot, summit) = comp.slope;
    let (dx, dy) = (summit.x.wrapping_sub(foot.x), summit.y.wrapping_sub(foot.y));
    let len2 = (dx as i64)
        .wrapping_mul(dx as i64)
        .wrapping_add((dy as i64).wrapping_mul(dy as i64));
    let len = isqrt(len2.clamp(0, i64::from(u32::MAX)) as u32).max(1) as i32;
    (
        from.0
            .wrapping_add(dx.wrapping_mul(distance).checked_div(len).unwrap_or(0)),
        from.1
            .wrapping_add(dy.wrapping_mul(distance).checked_div(len).unwrap_or(0)),
    )
}

/// Cuts `line` to at most `glyphs` characters.
fn cut(line: &str, glyphs: usize) -> &str {
    match line.char_indices().nth(glyphs) {
        Some((byte, _)) => line.get(..byte).unwrap_or(""),
        None => line,
    }
}

/// Uppercase hexadecimal digits of a word, most significant first.
fn hex16(word: u64) -> [u8; 16] {
    let mut out = [b'0'; 16];
    for (i, slot) in (0u32..16).zip(out.iter_mut()) {
        let n = (word.wrapping_shr(60u32.wrapping_sub(i.wrapping_mul(4))) & 0xF) as u8;
        *slot = if n < 10 {
            b'0'.wrapping_add(n)
        } else {
            b'A'.wrapping_add(n.wrapping_sub(10))
        };
    }
    out
}

/// The sky: constellation, hexagram and moon (REQ-L5-06..08).
fn compose_sky(
    pictor: &mut Pictor,
    comp: &Compositio,
    chord: &Concentus,
    indicia: &Indicia,
    regimen: &Regimen,
    automatum: u8,
    tranquillitas: u8,
) {
    let scale = comp.s;
    if regimen.has(CAELUM) {
        constellation(pictor, comp.sky, automatum, 6, 1);
    }
    if regimen.has(HEXAGRAMMA) && indicia.energy >= 77 {
        let hx = i32::from(comp.sky.x)
            .wrapping_add(i32::from(comp.sky.w))
            .wrapping_sub(i32::from(scale).wrapping_mul(12));
        hexagram(
            pictor,
            hx,
            i32::from(comp.sky.y),
            scale,
            tranquillitas,
            8,
            1,
        );
    }
    if chord.nocturnal {
        let r = i32::from(comp.sky.h).wrapping_div(3).max(2);
        let mx = i32::from(comp.sky.x)
            .wrapping_add(i32::from(comp.sky.w).wrapping_mul(3).wrapping_div(4));
        let my = i32::from(comp.sky.y).wrapping_add(i32::from(comp.sky.h).wrapping_shr(1));
        moon(pictor, mx, my, r, chord.luna, 12);
    }
}

/// The figure, its shadow and the boulder on the slope (REQ-L3-01..05).
fn compose_actors(
    pictor: &mut Pictor,
    comp: &Compositio,
    chord: &Concentus,
    indicia: &Indicia,
    regimen: &Regimen,
    mundus: &Mundus,
) {
    let verbum = mundus.verbum;
    let memoria = mundus.memoria;
    let scale = comp.s;
    let foot = slope_point(comp, verbum.ascensus());
    let asleep = memoria.eventus() == 12 && memoria.gradus() < 64;
    let post = postura(verbum, regimen.tick_ms >= 1000, asleep, indicia.leaving);
    let phase = gait_phase(indicia.epoch_ms, chord.cadentia);
    let shade = i32::from(comp.figure_h).wrapping_div(3);
    pictor.line(
        foot.0,
        foot.1,
        foot.0.wrapping_sub(shade),
        foot.1,
        2,
        0,
        scale as u8,
        Modo::Super,
    );
    let figura = Figura {
        foot,
        h: comp.figure_h,
        s: scale,
        postura: post,
        phase,
        linea: chord.linea,
    };
    draw_figure(pictor, &figura, 15, 0);
    let reach = i32::from(comp.figure_h)
        .wrapping_shr(1)
        .wrapping_add(i32::from(comp.boulder_d).wrapping_shr(1));
    let ahead = along_slope(comp, foot, reach);
    let center = (
        ahead.0,
        ahead
            .1
            .wrapping_sub(i32::from(comp.boulder_d).wrapping_shr(1)),
    );
    draw_boulder(
        pictor,
        center,
        comp.boulder_d,
        comp.boulder_scale,
        boulder_texture(verbum),
        12,
        4,
        15,
    );
}

/// The chronicle lines on the plinth and, after the recognition, the word in hexadecimal.
fn compose_text(
    pictor: &mut Pictor,
    comp: &Compositio,
    regimen: &Regimen,
    mundus: &Mundus,
    lines: &[&str],
) {
    let scale = comp.s;
    let memoria = mundus.memoria;
    if regimen.has(ANNALES) {
        for (i, line) in (0i32..).zip(lines.iter().take(usize::from(regimen.lines))) {
            let indent = i.wrapping_mul(3).wrapping_mul(i32::from(scale));
            let x = i32::from(comp.annales.x).wrapping_add(indent);
            let y = i32::from(comp.annales.y)
                .wrapping_add(i.wrapping_mul(10).wrapping_mul(i32::from(scale)));
            let room =
                usize::from(comp.glyphs_per_line).saturating_sub((i as usize).wrapping_add(1) / 2);
            text(pictor, x, y, scale, cut(line, room), LUM_TEXT, 0);
        }
    }
    if memoria.eventus() == 12 && memoria.gradus() < 240 {
        let digits = hex16(mundus.verbum.bits());
        if let Ok(word) = core::str::from_utf8(&digits) {
            let x = i32::from(pictor.width())
                .wrapping_sub(i32::from(comp.m))
                .wrapping_sub(i32::from(scale).wrapping_mul(96));
            text(pictor, x, i32::from(comp.m), scale, word, 10, 0);
        }
    }
}

/// Draws the chalk scene into `tab` with `lines` of chronicle, in the order of
/// `docs/04-arquitetura.md` §5: slate, sky, slope and ruins, frame, actors, text.
fn compose(
    mundus: &Mundus,
    chord: &Concentus,
    indicia: &Indicia,
    regimen: &Regimen,
    lines: &[&str],
    tab: &mut Tabula,
) {
    let comp = compositio(tab.width, tab.height, regimen.lines);
    let verbum = mundus.verbum;
    tab.pixels.fill(VOID);
    let mut pictor = Pictor::new(tab, chord.curvatura);
    compose_sky(
        &mut pictor,
        &comp,
        chord,
        indicia,
        regimen,
        verbum.automatum(),
        verbum.tranquillitas(),
    );
    draw_slope(
        &mut pictor,
        &comp,
        verbum.automatum(),
        regimen.strata,
        indicia.offset,
        3,
        6,
        regimen.has(PUNCTA),
    );
    if regimen.has(RUINAE) {
        draw_ruins(&mut pictor, &comp, mundus.memoria.clausurae(), 7, comp.s);
    }
    if regimen.has(MARGO) {
        draw_frame(
            &mut pictor,
            &comp,
            verbum.onus(),
            indicia.energy,
            verbum.semen(),
            chord.linea,
            9,
            0,
        );
    }
    compose_actors(&mut pictor, &comp, chord, indicia, regimen, mundus);
    compose_text(&mut pictor, &comp, regimen, mundus, lines);
}

/// Two tones: the slate (luminance ≤ 1) is 0, the marks of the figure and of the text
/// (luminance ≥ 13) are 15, and everything between is thresholded by the ordered dither;
/// classes and labels are kept.
fn bicolor(tab: &mut Tabula) {
    let width = usize::from(tab.width).max(1);
    for (y, row) in tab.pixels.chunks_mut(width).enumerate() {
        for (x, p) in row.iter_mut().enumerate() {
            let lum = lum_of(*p);
            let two = if lum <= 1 {
                0
            } else if lum >= 13 || bayer(x as u32, y as u32, lum) {
                15
            } else {
                0
            };
            *p = pixel(two, stratum_of(*p), label_of(*p));
        }
    }
}

/// Letters: every cell of 6×10·s above the plinth becomes the ramp glyph of its mean luminance.
fn littera_cells(tab: &mut Tabula, comp: &Compositio) {
    let s = usize::from(comp.s.max(1));
    let (cw, ch) = (6usize.wrapping_mul(s), 10usize.wrapping_mul(s));
    let w = usize::from(tab.width);
    let cols = w.checked_div(cw).unwrap_or(0);
    let rows = usize::from(comp.scene_h).checked_div(ch).unwrap_or(0);
    let area = cw.wrapping_mul(ch).max(1) as u32;
    for cy in 0..rows {
        for cx in 0..cols {
            let mut sum = 0u32;
            for dy in 0..ch {
                for dx in 0..cw {
                    let idx = cy
                        .wrapping_mul(ch)
                        .wrapping_add(dy)
                        .wrapping_mul(w)
                        .wrapping_add(cx.wrapping_mul(cw))
                        .wrapping_add(dx);
                    sum =
                        sum.wrapping_add(u32::from(tab.pixels.get(idx).map_or(0, |&p| lum_of(p))));
                }
            }
            let mean = sum.checked_div(area).unwrap_or(0).min(15) as usize;
            let glyph = ORDO
                .chars()
                .nth(usize::from(
                    *RAMP.get(mean.wrapping_mul(10) / 16).unwrap_or(&36),
                ))
                .unwrap_or(' ');
            let mut cell = [0u8; 4];
            let txt = glyph.encode_utf8(&mut cell);
            let (x, y) = ((cx.wrapping_mul(cw)) as i32, (cy.wrapping_mul(ch)) as i32);
            let mut pictor = Pictor::flat(tab);
            pictor.fill(x, y, cw as i32, ch as i32, 1, 3, false);
            text(&mut pictor, x, y, comp.s, txt, 12, 1);
        }
    }
}

/// Mosaic: the scene at W/s × H/s in the prefix of the buffer, then scaled up in place by
/// nearest neighbour from the last pixel to the first.
fn musivum(
    mundus: &Mundus,
    chord: &Concentus,
    indicia: &Indicia,
    regimen: &Regimen,
    lines: &[&str],
    tab: &mut Tabula,
) {
    let comp = compositio(tab.width, tab.height, regimen.lines);
    let scale = usize::from(comp.s.max(1));
    if scale == 1 {
        compose(mundus, chord, indicia, regimen, lines, tab);
        return;
    }
    let (width, height) = (usize::from(tab.width), usize::from(tab.height));
    let (sw, sh) = (
        width.checked_div(scale).unwrap_or(0),
        height.checked_div(scale).unwrap_or(0),
    );
    let Some(prefix) = tab.pixels.get_mut(..sw.wrapping_mul(sh)) else {
        return;
    };
    let mut small = Tabula {
        width: sw as u16,
        height: sh as u16,
        pixels: prefix,
    };
    compose(mundus, chord, indicia, regimen, lines, &mut small);
    let mut y = height;
    while y > 0 {
        y = y.wrapping_sub(1);
        let sy = y.checked_div(scale).unwrap_or(0).min(sh.saturating_sub(1));
        let mut x = width;
        while x > 0 {
            x = x.wrapping_sub(1);
            let sx = x.checked_div(scale).unwrap_or(0).min(sw.saturating_sub(1));
            let value = tab
                .pixels
                .get(sy.wrapping_mul(sw).wrapping_add(sx))
                .copied()
                .unwrap_or(VOID);
            if let Some(slot) = tab.pixels.get_mut(y.wrapping_mul(width).wrapping_add(x)) {
                *slot = value;
            }
        }
    }
}

/// Renders one frame in the regimen's stilus (REQ-L5-03, REQ-H-04).
pub fn render(
    mundus: &Mundus,
    facultates: &Facultates,
    indicia: &Indicia,
    regimen: &Regimen,
    lines: &[&str],
    tab: &mut Tabula,
) {
    let chord = concentus(mundus, facultates, indicia, regimen);
    match regimen.stilus {
        Stilus::Creta => compose(mundus, &chord, indicia, regimen, lines, tab),
        Stilus::Musivum => musivum(mundus, &chord, indicia, regimen, lines, tab),
        Stilus::Littera => {
            compose(mundus, &chord, indicia, regimen, lines, tab);
            let comp = compositio(tab.width, tab.height, regimen.lines);
            littera_cells(tab, &comp);
        }
        Stilus::Bicolor => {
            compose(mundus, &chord, indicia, regimen, lines, tab);
            bicolor(tab);
        }
    }
}

/// Renders one frame as glyph cells: the scene drawn in `scratch` (which must have the grid's
/// dimensions) becomes ramp glyphs, and the chronicle lines occupy the last rows as letters.
/// Bits 0..=7 glyph, 8..=11 tone, 12..=13 stratum (REQ-H-02).
pub fn render_cellae(
    mundus: &Mundus,
    facultates: &Facultates,
    indicia: &Indicia,
    regimen: &Regimen,
    lines: &[&str],
    scratch: &mut Tabula,
    cells: &mut Cellae,
) {
    if scratch.width != cells.cols || scratch.height != cells.rows {
        return;
    }
    let chord = concentus(mundus, facultates, indicia, regimen);
    let quiet = Regimen {
        lines: 0,
        ..*regimen
    };
    compose(mundus, &chord, indicia, &quiet, &[], scratch);
    for (cell, &p) in cells.cells.iter_mut().zip(scratch.pixels.iter()) {
        let lum = lum_of(p);
        let glyph = *RAMP
            .get(usize::from(lum).wrapping_mul(10) / 16)
            .unwrap_or(&36);
        *cell = u16::from(glyph) | (u16::from(lum) << 8) | (u16::from(stratum_of(p) & 3) << 12);
    }
    let cols = usize::from(cells.cols);
    let rows = usize::from(cells.rows);
    let shown = lines.len().min(usize::from(regimen.lines)).min(rows);
    for (i, line) in lines.iter().take(shown).enumerate() {
        let row = rows.wrapping_sub(shown).wrapping_add(i);
        for (k, ch) in line.chars().take(cols).enumerate() {
            let glyph = ORDO.chars().position(|o| o == ch).map_or(DOT, |g| g as u8);
            if let Some(cell) = cells.cells.get_mut(row.wrapping_mul(cols).wrapping_add(k)) {
                *cell = u16::from(glyph) | (u16::from(LUM_TEXT) << 8);
            }
        }
    }
}

/// Golden hashes of the four scenarios (REQ-L5-09), recorded after visual inspection.
#[cfg(test)]
const GOLDEN: [(&str, u64); 4] = [
    ("vigilia", 0x6939_00D0_E0E1_CC05),
    ("acedia", 0x9F0B_5319_454B_90FC),
    ("somnium", 0x156B_FE74_674C_1B17),
    ("quies", 0xD5C6_687C_FD16_F2B7),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hospes::{Cellae, Facultates, Indicia, Regimen, Stilus, Tabula};
    use crate::mundus::Mundus;
    use crate::signum::pixel::{lum_of, stratum_of};
    use crate::verbum::Modus;
    use std::vec::Vec;

    /// Noon of the 2026 June solstice at −03:00.
    const NOON: i64 = 1_782_054_000_000;
    const LINES: [&str; 4] = [
        "THE LEAF CRACKS UNDER THE SANDAL.",
        "AN OWL LEAVES THE OLIVE TREE.",
        "A LAMP FALLS IN ATHENS.",
        "THE LIBRARY BURNS. ROME WAITS.",
    ];

    fn frame(width: u16, height: u16, stilus: Stilus, modus: Modus, energy: u8) -> Vec<u8> {
        let mut world = Mundus::genesis(1);
        let kappa = if modus == Modus::Somnium { -48 } else { 0 };
        world.verbum = world
            .verbum
            .with_ascensus(90)
            .with_modus(modus)
            .with_curvatura(kappa);
        world.memoria = world
            .memoria
            .with_clausurae(0b1010)
            .with_ramus(0)
            .with_gradus(300);
        let facultates = Facultates {
            width,
            height,
            ..Facultates::default()
        };
        let indicia = Indicia {
            epoch_ms: NOON,
            energy,
            ..Indicia::default()
        };
        let regimen = Regimen {
            stilus,
            ..Regimen::default()
        };
        let mut buf = std::vec![0u8; usize::from(width) * usize::from(height)];
        let mut tab = Tabula::new(width, height, &mut buf).unwrap();
        render(&world, &facultates, &indicia, &regimen, &LINES, &mut tab);
        buf
    }

    /// The same inputs give the same frame; the frame is not empty (REQ-H-04).
    #[test]
    fn render_deterministic() {
        let a = frame(320, 180, Stilus::Creta, Modus::Vigilia, 255);
        let b = frame(320, 180, Stilus::Creta, Modus::Vigilia, 255);
        assert_eq!(a, b);
        let tones: std::collections::BTreeSet<u8> = a.iter().map(|&p| lum_of(p)).collect();
        assert!(tones.len() >= 6, "tones used: {tones:?}");
        assert!(a.iter().any(|&p| stratum_of(p) == 0) && a.iter().any(|&p| stratum_of(p) == 2));
        assert_eq!(fnv1a(b""), 0xCBF2_9CE4_8422_2325);
        assert_eq!(fnv1a(b"a"), 0xAF63_DC4C_8601_EC8C);
    }

    /// Two-tone frames use only luminance 0 and 15 (REQ-L5-03).
    #[test]
    fn bicolor_only_two_tones() {
        let a = frame(320, 180, Stilus::Bicolor, Modus::Quies, 20);
        assert!(a.iter().all(|&p| lum_of(p) == 0 || lum_of(p) == 15));
        assert!(a.iter().any(|&p| lum_of(p) == 15) && a.iter().any(|&p| lum_of(p) == 0));
    }

    /// Mosaic at scale 1 is the chalk frame; at scale 2 it is made of 2×2 blocks (REQ-L5-03).
    #[test]
    fn musivum_scale_one_identity() {
        assert_eq!(
            frame(320, 180, Stilus::Musivum, Modus::Vigilia, 255),
            frame(320, 180, Stilus::Creta, Modus::Vigilia, 255)
        );
        let big = frame(640, 360, Stilus::Musivum, Modus::Vigilia, 255);
        for y in (0..360).step_by(2) {
            for x in (0..640).step_by(2) {
                let p = big[x + y * 640];
                assert_eq!(p, big[x + 1 + y * 640], "block at {x},{y}");
                assert_eq!(p, big[x + (y + 1) * 640], "block at {x},{y}");
            }
        }
    }

    /// Letters: the scene above the plinth becomes glyph cells; the text lines stay intact.
    #[test]
    fn littera_ramp_and_text() {
        let letters = frame(320, 180, Stilus::Littera, Modus::Vigilia, 255);
        let chalk = frame(320, 180, Stilus::Creta, Modus::Vigilia, 255);
        let plinth_start = 104 * 320;
        assert_eq!(
            &letters[plinth_start..],
            &chalk[plinth_start..],
            "plinth text unchanged"
        );
        assert_ne!(
            &letters[..plinth_start],
            &chalk[..plinth_start],
            "scene turned into glyphs"
        );
        let mut scratch = [0u8; 80 * 24];
        let mut cells = [0u16; 80 * 24];
        let mut world = Mundus::genesis(1);
        world.memoria = world.memoria.with_ramus(0).with_gradus(300);
        let facultates = Facultates {
            width: 80,
            height: 24,
            cells: true,
            ..Facultates::default()
        };
        let indicia = Indicia {
            epoch_ms: NOON,
            ..Indicia::default()
        };
        let regimen = Regimen {
            stilus: Stilus::Littera,
            ..Regimen::default()
        };
        let mut tab = Tabula::new(80, 24, &mut scratch).unwrap();
        let mut grid = Cellae::new(80, 24, &mut cells).unwrap();
        render_cellae(
            &world,
            &facultates,
            &indicia,
            &regimen,
            &LINES,
            &mut tab,
            &mut grid,
        );
        let glyphs: Vec<u8> = cells.iter().map(|&c| (c & 0xFF) as u8).collect();
        assert!(glyphs.iter().any(|&g| g != 36), "some non-space glyphs");
        let last_rows = &cells[80 * 20..];
        let text_cells = last_rows
            .iter()
            .filter(|&&c| (c & 0xFF) < 36 && (c >> 12) & 3 == 0)
            .count();
        assert!(
            text_cells >= 20,
            "chronicle letters in the plinth rows: {text_cells}"
        );
    }

    /// Lines longer than the plinth are cut at the cell boundary, never wrapped or shifted.
    #[test]
    fn littera_truncates_lines() {
        let long = ["ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJ"];
        let m = Mundus::genesis(1);
        let f = Facultates::default();
        let i = Indicia {
            epoch_ms: NOON,
            ..Indicia::default()
        };
        let r = Regimen::default();
        let mut buf = [0u8; 320 * 180];
        let mut tab = Tabula::new(320, 180, &mut buf).unwrap();
        render(&m, &f, &i, &r, &long, &mut tab);
        let comp = crate::spatium::compositio(320, 180, 6);
        let right = usize::from(comp.annales.x + comp.annales.w);
        let row = usize::from(comp.annales.y) + 5;
        assert!(
            (right..320).all(|x| lum_of(buf[x + row * 320]) != 13),
            "nothing beyond the right margin"
        );
        assert!(
            (0..usize::from(comp.annales.x)).all(|x| lum_of(buf[x + row * 320]) != 13),
            "nothing before the left margin"
        );
    }

    /// Extreme curvature keeps every drawn pixel inside the frame and the frame non-empty.
    #[test]
    fn warp_keeps_inside_frame() {
        for kappa in [-64i8, 64, -128, 127] {
            let mut m = Mundus::genesis(3);
            m.verbum = m
                .verbum
                .with_modus(Modus::Somnium)
                .with_curvatura(kappa)
                .with_ascensus(120);
            let f = Facultates {
                width: 480,
                height: 270,
                ..Facultates::default()
            };
            let i = Indicia {
                epoch_ms: NOON + 14 * 3_600_000,
                energy: 200,
                idle_s: 3600,
                ..Indicia::default()
            };
            let r = Regimen::default();
            let mut buf = std::vec![0xA5u8; 480 * 270 + 16];
            let mut tab = Tabula::new(480, 270, &mut buf[..480 * 270]).unwrap();
            render(&m, &f, &i, &r, &LINES, &mut tab);
            assert!(
                buf[480 * 270..].iter().all(|&b| b == 0xA5),
                "κ {kappa} wrote past the frame"
            );
            assert!(
                buf[..480 * 270].iter().any(|&p| lum_of(p) >= 12),
                "κ {kappa} empty frame"
            );
        }
    }

    /// Writes the four golden scenarios as PPM files for inspection when requested.
    #[test]
    #[ignore = "writes files; run on request"]
    fn dump_golden_ppm() {
        use std::io::Write;
        let dir = std::env::var("SISYPHIUM_DUMP").unwrap_or_else(|_| "/tmp".into());
        for (name, m, i, r) in golden_scenarios() {
            let mut buf = std::vec![0u8; 320 * 180];
            let mut tab = Tabula::new(320, 180, &mut buf).unwrap();
            let f = Facultates::default();
            render(&m, &f, &i, &r, &LINES, &mut tab);
            let c = crate::concentus::concentus(&m, &f, &i, &r);
            let pal = crate::concentus::palette(&c);
            let mut out = std::vec::Vec::new();
            out.extend_from_slice(b"P6\n320 180\n255\n");
            for &p in &buf {
                out.extend_from_slice(&pal[usize::from(lum_of(p))]);
            }
            std::fs::File::create(std::format!("{dir}/{name}.ppm"))
                .unwrap()
                .write_all(&out)
                .unwrap();
            std::println!("{name}: {:#018x}", fnv1a(&buf));
        }
    }

    /// Measures the median and the 95th percentile of the frame time per stilus (REQ-H-05) when
    /// requested; results go to the pull request of the checkpoint.
    #[test]
    #[ignore = "measurement; run in release on request"]
    fn metire_frames() {
        let mut scenarios: Vec<(&str, Mundus, Indicia, Regimen)> =
            golden_scenarios().into_iter().collect();
        let (name, m, i, r) = scenarios[0];
        scenarios.push((
            name,
            m,
            i,
            Regimen {
                stilus: Stilus::Littera,
                ..r
            },
        ));
        for (name, m, i, r) in &scenarios {
            for (w, h, n) in [(320u16, 180u16, 1000u32), (1920, 1080, 100)] {
                let f = Facultates {
                    width: w,
                    height: h,
                    ..Facultates::default()
                };
                let mut buf = std::vec![0u8; usize::from(w) * usize::from(h)];
                let mut times: Vec<u128> = Vec::new();
                for _ in 0..n {
                    let mut tab = Tabula::new(w, h, &mut buf).unwrap();
                    let start = std::time::Instant::now();
                    render(m, &f, i, r, &LINES, &mut tab);
                    times.push(start.elapsed().as_micros());
                }
                times.sort_unstable();
                let median = times[times.len() / 2];
                let p95 = times[times.len() * 95 / 100];
                std::println!(
                    "{name} {w}x{h} {:?}: mediana {median} µs, p95 {p95} µs",
                    r.stilus
                );
            }
        }
    }

    /// The four scenarios of `docs/08-verificacao.md` §7.
    fn golden_scenarios() -> [(&'static str, Mundus, Indicia, Regimen); 4] {
        let mut vigil = Mundus::genesis(1);
        vigil.verbum = vigil.verbum.with_ascensus(90);
        vigil.memoria = vigil
            .memoria
            .with_ramus(0)
            .with_gradus(300)
            .with_clausurae(0b110);
        let mut acedia = Mundus::genesis(2);
        acedia.verbum = acedia
            .verbum
            .with_ascensus(140)
            .with_onus(160)
            .with_tranquillitas(10);
        acedia.memoria = acedia.memoria.with_ramus(1).with_gradus(100);
        let mut dream = Mundus::genesis(3);
        dream.verbum = dream
            .verbum
            .with_ascensus(200)
            .with_modus(Modus::Somnium)
            .with_curvatura(-48);
        let quies = Mundus::genesis(4);
        [
            (
                "vigilia",
                vigil,
                Indicia {
                    epoch_ms: NOON,
                    energy: 255,
                    ..Indicia::default()
                },
                Regimen::default(),
            ),
            (
                "acedia",
                acedia,
                Indicia {
                    epoch_ms: 1_790_108_200_000,
                    energy: 120,
                    ..Indicia::default()
                },
                Regimen {
                    stilus: Stilus::Musivum,
                    strata: 2,
                    tick_ms: 125,
                    interval_ms: 125,
                    ..Regimen::default()
                },
            ),
            (
                "somnium",
                dream,
                Indicia {
                    epoch_ms: 1_797_831_000_000,
                    energy: 200,
                    idle_s: 3600,
                    ..Indicia::default()
                },
                Regimen::default(),
            ),
            (
                "quies",
                quies,
                Indicia {
                    epoch_ms: 1_774_008_000_000 - 3 * 3_600_000,
                    energy: 20,
                    ..Indicia::default()
                },
                Regimen {
                    stilus: Stilus::Bicolor,
                    strata: 1,
                    tick_ms: 60_000,
                    interval_ms: 60_000,
                    ..Regimen::default()
                },
            ),
        ]
    }

    /// The golden hashes (REQ-L5-09); updated only with an intentional change of form.
    #[test]
    fn golden_frames() {
        let expected: [(&str, u64); 4] = GOLDEN;
        for ((name, m, i, r), (ename, hash)) in golden_scenarios().into_iter().zip(expected) {
            assert_eq!(name, ename);
            let mut buf = std::vec![0u8; 320 * 180];
            let mut tab = Tabula::new(320, 180, &mut buf).unwrap();
            render(&m, &Facultates::default(), &i, &r, &LINES, &mut tab);
            assert_eq!(fnv1a(&buf), hash, "{name}: {:#018x}", fnv1a(&buf));
        }
    }
}
