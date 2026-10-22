//! The scene composer: one frame from the world, the host, the regimen and the visible lines, in
//! the order of `docs/04-arquitetura.md` §5, and the four stili (REQ-L5-03, REQ-L5-09, REQ-H-04).

use crate::concentus::{Concentus, concentus};
use crate::corpus::{
    Figura, Postura, boulder_texture, draw_boulder, draw_figure, gait_phase, postura,
};
use crate::hospes::{
    ANNALES, CAELUM, Cellae, Facultates, HEXAGRAMMA, Indicia, MARGO, PUNCTA, RUINAE, Regimen,
    Stilus, Tabula,
};
use crate::mundus::Mundus;
use crate::signum::caelum::{constellation, hexagram, moon, sky_region};
use crate::signum::littera::{ORDO, cell_glyph, text};
use crate::signum::margo::draw_frame;
use crate::signum::mons::{draw_ruins, draw_slope};
use crate::signum::pixel::{VOID, label_of, lum_of, pixel, stratum_of};
use crate::signum::raster::{Modo, Pictor, bayer};
use crate::spatium::{Compositio, Rect, compositio, isqrt};

/// FNV-1a over 64 bits: the hash of a frame for the golden tests.
#[must_use]
pub fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xCBF2_9CE4_8422_2325u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01B3)
    })
}

/// Luminance of the text of the chronicles.
const LUM_TEXT: u8 = 13;
/// Luminance of the word in hexadecimal after the recognition (REQ-L8-02).
const LUM_VERBUM: u8 = 10;
/// Stratum class of the sky: constellation, hexagram and moon (`docs/04-arquitetura.md` §5).
const CLASSIS_CAELI: u8 = 2;
/// Stratum class of the actors: the figure, its shadow and the boulder.
const CLASSIS_ACTORUM: u8 = 1;
/// Stratum class of the signs: the frame, the chronicles and the word after the recognition.
const CLASSIS_SIGNORUM: u8 = 0;
/// The glyph ramp of the Littera stilus, as indices in [`ORDO`]: ` .:-=+*#%@`.
const RAMP: [u8; 10] = [36, 37, 39, 43, 45, 44, 53, 54, 55, 56];

/// The first `width`·`height` items of a surface (REQ-H-02): the core reads and writes nothing
/// beyond them; a shorter slice is taken whole.
fn prefix<T>(items: &mut [T], width: u16, height: u16) -> &mut [T] {
    let area = usize::from(width).saturating_mul(usize::from(height));
    let end = area.min(items.len());
    items.get_mut(..end).unwrap_or_default()
}

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

/// The step and the room of line `i` of the plinth: line i steps in by
/// min(3·s·i, 6·s·(glyphs per line − 40)) pixels, or not at all when a line holds 40 glyphs or
/// fewer, so that every line keeps at least 40 glyphs (`docs/07-narrativa.md` §3); it then holds
/// ⌊(width of the chronicles − step)/(6·s)⌋ glyphs.
fn plinth_line(comp: &Compositio, i: u8) -> (i32, usize) {
    let s = i32::from(comp.s.max(1));
    let glyphs = i32::from(comp.glyphs_per_line);
    let indent = if glyphs > 40 {
        s.wrapping_mul(3)
            .wrapping_mul(i32::from(i))
            .min(s.wrapping_mul(6).wrapping_mul(glyphs.wrapping_sub(40)))
    } else {
        0
    };
    let room = i32::from(comp.annales.w)
        .wrapping_sub(indent)
        .checked_div(s.wrapping_mul(6))
        .unwrap_or(0)
        .max(0);
    (indent, room as usize)
}

/// Cuts `line` to at most `glyphs` characters.
fn cut(line: &str, glyphs: usize) -> &str {
    match line.char_indices().nth(glyphs) {
        Some((byte, _)) => line.get(..byte).unwrap_or(""),
        None => line,
    }
}

/// Whether the word of the night shows after the recognition: `eventus` 12 with `gradus` below
/// 240 (REQ-L8-02).
fn recognized(mundus: &Mundus) -> bool {
    mundus.memoria.eventus() == 12 && mundus.memoria.gradus() < 240
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

/// The sky in its region (REQ-L5-12): constellation, hexagram and moon (REQ-L5-06..08).
#[allow(clippy::too_many_arguments)]
fn compose_sky(
    pictor: &mut Pictor,
    comp: &Compositio,
    sky: Rect,
    chord: &Concentus,
    indicia: &Indicia,
    regimen: &Regimen,
    automatum: u8,
    tranquillitas: u8,
) {
    let scale = comp.s;
    if regimen.has(CAELUM) {
        constellation(pictor, sky, automatum, 6, CLASSIS_CAELI);
    }
    let room = i32::from(scale).wrapping_mul(12);
    if regimen.has(HEXAGRAMMA)
        && indicia.energy >= 77
        && i32::from(sky.w) >= room
        && i32::from(sky.h) >= room
    {
        let hx = i32::from(sky.x)
            .wrapping_add(i32::from(sky.w))
            .wrapping_sub(i32::from(scale).wrapping_mul(12));
        hexagram(
            pictor,
            hx,
            i32::from(sky.y),
            scale,
            tranquillitas,
            8,
            CLASSIS_CAELI,
        );
    }
    if chord.nocturnal {
        let r = i32::from(sky.h).wrapping_div(3).max(2);
        let mx = i32::from(sky.x).wrapping_add(i32::from(sky.w).wrapping_mul(3).wrapping_div(4));
        let my = i32::from(sky.y).wrapping_add(i32::from(sky.h).wrapping_shr(1));
        moon(pictor, mx, my, r, chord.luna, 12, chord.australis);
    }
}

/// The posture of the figure from the state (REQ-L3-02): contemplative during a persona
/// (`eventus` 13..=15 with `ramus` 0 and `gradus` < 240) or in Quies, asleep for 64 ticks after
/// the recognition (`eventus` 12), facing the observer when `leaving`.
fn figure_posture(mundus: &Mundus, leaving: bool) -> Postura {
    let memoria = mundus.memoria;
    let persona =
        matches!(memoria.eventus(), 13..=15) && memoria.ramus() == 0 && memoria.gradus() < 240;
    let asleep = memoria.eventus() == 12 && memoria.gradus() < 64;
    postura(mundus.verbum, persona, asleep, leaving)
}

/// The figure, its shadow and the boulder on the slope (REQ-L3-01..05).
fn compose_actors(
    pictor: &mut Pictor,
    comp: &Compositio,
    chord: &Concentus,
    indicia: &Indicia,
    mundus: &Mundus,
) {
    let verbum = mundus.verbum;
    let scale = comp.s;
    let foot = slope_point(comp, verbum.ascensus());
    let post = figure_posture(mundus, indicia.leaving);
    let phase = gait_phase(indicia.epoch_ms, chord.cadentia);
    let shade = i32::from(comp.figure_h).wrapping_div(3);
    pictor.line(
        foot.0,
        foot.1,
        foot.0.wrapping_sub(shade),
        foot.1,
        2,
        CLASSIS_ACTORUM,
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
    draw_figure(pictor, &figura, 15, CLASSIS_ACTORUM);
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
        [12, 4, 15],
        CLASSIS_ACTORUM,
    );
}

/// The chronicle lines on the plinth and, after the recognition, the word in hexadecimal when
/// `hexadecimal` (the cell render writes the word as glyph cells instead).
fn compose_text(
    pictor: &mut Pictor,
    comp: &Compositio,
    regimen: &Regimen,
    mundus: &Mundus,
    lines: &[&str],
    hexadecimal: bool,
) {
    let scale = comp.s;
    if regimen.has(ANNALES) {
        for (i, line) in (0..=u8::MAX).zip(lines.iter().take(usize::from(regimen.lines))) {
            let (indent, room) = plinth_line(comp, i);
            let x = i32::from(comp.annales.x).wrapping_add(indent);
            let y = i32::from(comp.annales.y)
                .wrapping_add(i32::from(i).wrapping_mul(10).wrapping_mul(i32::from(scale)));
            text(
                pictor,
                x,
                y,
                scale,
                cut(line, room),
                LUM_TEXT,
                CLASSIS_SIGNORUM,
            );
        }
    }
    if hexadecimal && recognized(mundus) {
        let digits = hex16(mundus.verbum.bits());
        if let Ok(word) = core::str::from_utf8(&digits) {
            let x = i32::from(pictor.width())
                .wrapping_sub(i32::from(comp.m))
                .wrapping_sub(i32::from(scale).wrapping_mul(96));
            text(
                pictor,
                x,
                i32::from(comp.m),
                scale,
                word,
                LUM_VERBUM,
                CLASSIS_SIGNORUM,
            );
        }
    }
}

/// Draws the chalk scene into `tab` with `lines` of chronicle, in the order of
/// `docs/04-arquitetura.md` §5: slate, sky, slope and ruins, frame, actors, text, and the word
/// in hexadecimal when `hexadecimal`.
fn compose(
    mundus: &Mundus,
    chord: &Concentus,
    indicia: &Indicia,
    regimen: &Regimen,
    lines: &[&str],
    tab: &mut Tabula,
    hexadecimal: bool,
) {
    let comp = compositio(tab.width, tab.height, regimen.lines);
    let verbum = mundus.verbum;
    prefix(tab.pixels, tab.width, tab.height).fill(VOID);
    let sky = sky_region(&comp, tab.width, verbum.onus());
    let mut pictor = Pictor::new(tab, chord.curvatura);
    compose_sky(
        &mut pictor,
        &comp,
        sky,
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
            CLASSIS_SIGNORUM,
        );
    }
    compose_actors(&mut pictor, &comp, chord, indicia, mundus);
    compose_text(&mut pictor, &comp, regimen, mundus, lines, hexadecimal);
}

/// Two tones: the slate (luminance ≤ 1) is 0, the marks of the figure and of the text
/// (luminance ≥ 13) are 15, and everything between is thresholded by the ordered dither;
/// classes and labels are kept. Only the frame's prefix is read and written.
fn bicolor(tab: &mut Tabula) {
    let width = usize::from(tab.width).max(1);
    let frame = prefix(tab.pixels, tab.width, tab.height);
    for (y, row) in frame.chunks_mut(width).enumerate() {
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

/// Letters: every cell of 6×10·s above the plinth, including the partial cells of the right
/// border and of the band between the scene and the plinth, becomes the ramp glyph of the mean
/// luminance of its pixels inside the scene; the glyphs are clipped to the scene, and the plinth
/// is left as drawn (REQ-L5-03).
fn littera_cells(tab: &mut Tabula, comp: &Compositio) {
    let s = usize::from(comp.s.max(1));
    let (cw, ch) = (6usize.wrapping_mul(s), 10usize.wrapping_mul(s));
    let w = usize::from(tab.width);
    let scene_h = comp.scene_h.min(tab.height);
    let Some(pixels) = tab.pixels.get_mut(..w.wrapping_mul(usize::from(scene_h))) else {
        return;
    };
    let mut scene = Tabula {
        width: tab.width,
        height: scene_h,
        pixels,
    };
    let cols = w.div_ceil(cw);
    let rows = usize::from(scene_h).div_ceil(ch);
    for cy in 0..rows {
        for cx in 0..cols {
            let (x0, y0) = (cx.wrapping_mul(cw), cy.wrapping_mul(ch));
            let x1 = x0.wrapping_add(cw).min(w);
            let y1 = y0.wrapping_add(ch).min(usize::from(scene_h));
            let (mut sum, mut count) = (0u32, 0u32);
            for y in y0..y1 {
                for x in x0..x1 {
                    let idx = y.wrapping_mul(w).wrapping_add(x);
                    let lum = scene.pixels.get(idx).map_or(0, |&p| lum_of(p));
                    sum = sum.wrapping_add(u32::from(lum));
                    count = count.wrapping_add(1);
                }
            }
            let mean = sum.checked_div(count).unwrap_or(0).min(15) as usize;
            let glyph = ORDO
                .chars()
                .nth(usize::from(
                    *RAMP.get(mean.wrapping_mul(10) / 16).unwrap_or(&36),
                ))
                .unwrap_or(' ');
            let mut cell = [0u8; 4];
            let txt = glyph.encode_utf8(&mut cell);
            let (x, y) = (x0 as i32, y0 as i32);
            let mut pictor = Pictor::flat(&mut scene);
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
        compose(mundus, chord, indicia, regimen, lines, tab, true);
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
    compose(mundus, chord, indicia, regimen, lines, &mut small, true);
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
        Stilus::Creta => compose(mundus, &chord, indicia, regimen, lines, tab, true),
        Stilus::Musivum => musivum(mundus, &chord, indicia, regimen, lines, tab),
        Stilus::Littera => {
            compose(mundus, &chord, indicia, regimen, lines, tab, true);
            let comp = compositio(tab.width, tab.height, regimen.lines);
            littera_cells(tab, &comp);
        }
        Stilus::Bicolor => {
            compose(mundus, &chord, indicia, regimen, lines, tab, true);
            bicolor(tab);
        }
    }
}

/// Renders one frame as glyph cells and returns whether it did: the scene drawn in `scratch`
/// becomes ramp glyphs, the chronicle lines occupy the last rows as letters, and after the
/// recognition the sixteen hexadecimal digits of the word stand at the right end of the first
/// row (REQ-L8-02). A `scratch` without the grid's dimensions is refused and nothing is written.
/// Bits 0..=7 glyph ([`cell_glyph`]), 8..=11 tone, 12..=13 stratum; only the prefixes of
/// cols·rows cells and pixels are read and written (REQ-H-02).
#[must_use]
pub fn render_cellae(
    mundus: &Mundus,
    facultates: &Facultates,
    indicia: &Indicia,
    regimen: &Regimen,
    lines: &[&str],
    scratch: &mut Tabula,
    cells: &mut Cellae,
) -> bool {
    if scratch.width != cells.cols || scratch.height != cells.rows {
        return false;
    }
    let chord = concentus(mundus, facultates, indicia, regimen);
    let quiet = Regimen {
        lines: 0,
        ..*regimen
    };
    compose(mundus, &chord, indicia, &quiet, &[], scratch, false);
    let raster = prefix(scratch.pixels, scratch.width, scratch.height);
    for (cell, &p) in prefix(cells.cells, cells.cols, cells.rows)
        .iter_mut()
        .zip(raster.iter())
    {
        let lum = lum_of(p);
        let glyph = *RAMP
            .get(usize::from(lum).wrapping_mul(10) / 16)
            .unwrap_or(&36);
        *cell = u16::from(glyph) | (u16::from(lum) << 8) | (u16::from(stratum_of(p) & 3) << 12);
    }
    let cols = usize::from(cells.cols);
    let rows = usize::from(cells.rows);
    let letter = |c: char, lum: u8| {
        u16::from(cell_glyph(c)) | (u16::from(lum) << 8) | (u16::from(CLASSIS_SIGNORUM) << 12)
    };
    let shown = lines.len().min(usize::from(regimen.lines)).min(rows);
    for (i, line) in lines.iter().take(shown).enumerate() {
        let row = rows.wrapping_sub(shown).wrapping_add(i);
        for (k, ch) in line.chars().take(cols).enumerate() {
            if let Some(cell) = cells.cells.get_mut(row.wrapping_mul(cols).wrapping_add(k)) {
                *cell = letter(ch, LUM_TEXT);
            }
        }
    }
    if recognized(mundus) {
        let digits = hex16(mundus.verbum.bits());
        for (k, digit) in digits.iter().enumerate() {
            let column = (cols as i64).wrapping_sub(16).wrapping_add(k as i64);
            if column >= 0 {
                if let Some(cell) = cells.cells.get_mut(column as usize) {
                    *cell = letter(char::from(*digit), LUM_VERBUM);
                }
            }
        }
    }
    true
}

/// Golden hashes of the six scenarios (REQ-L5-09), recorded after visual inspection.
#[cfg(test)]
const GOLDEN: [(&str, u64); 6] = [
    ("vigilia", 0x7456_D704_92AF_0BB7),
    ("acedia", 0x3A8A_F286_4867_3454),
    ("somnium", 0xE4A8_6523_FA7C_0407),
    ("quies", 0xBCDB_E4CF_E9EE_0CCF),
    ("littera", 0x4988_A946_E3FF_681D),
    ("somnium-1920x1080", 0xF947_ADD8_C765_ADAF),
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
    /// Four chronicle lines for the frames of the tests.
    const LINES: [&str; 4] = [
        "THE LEAF CRACKS UNDER THE SANDAL.",
        "AN OWL LEAVES THE OLIVE TREE.",
        "A LAMP FALLS IN ATHENS.",
        "THE LIBRARY BURNS. ROME WAITS.",
    ];

    /// A frame of genesis(1) at noon in `stilus` and `modus` (curvature −48 in Somnium).
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

    /// The posture comes from the state alone: contemplative during a persona (`eventus`
    /// 13..=15, `ramus` 0, `gradus` < 240) and in Quies, asleep for 64 ticks after the
    /// recognition, facing the observer when leaving, upright otherwise; no regimen enters
    /// (REQ-L3-02).
    #[test]
    fn posture_reads_the_persona() {
        use crate::corpus::Postura;
        let base = Mundus::genesis(1);
        let with = |eventus: u8, ramus: u8, gradus: u16| {
            let mut m = base;
            m.memoria = m
                .memoria
                .with_eventus(eventus)
                .with_ramus(ramus)
                .with_gradus(gradus);
            m
        };
        for eventus in 13..=15u8 {
            for gradus in [0u16, 239] {
                assert_eq!(
                    figure_posture(&with(eventus, 0, gradus), false),
                    Postura::Contemplans
                );
            }
            assert_eq!(
                figure_posture(&with(eventus, 0, 240), false),
                Postura::Erecta
            );
            assert_eq!(
                figure_posture(&with(eventus, 1, 10), false),
                Postura::Erecta
            );
        }
        assert_eq!(figure_posture(&with(12, 0, 63), false), Postura::Dormiens);
        assert_eq!(figure_posture(&with(12, 0, 64), false), Postura::Erecta);
        assert_eq!(figure_posture(&with(3, 3, 0), false), Postura::Erecta);
        let mut quies = base;
        quies.verbum = quies.verbum.with_modus(Modus::Quies);
        assert_eq!(figure_posture(&quies, false), Postura::Contemplans);
        assert_eq!(figure_posture(&with(13, 0, 5), true), Postura::Spectans);
    }

    /// Each layer writes its stratum class (`docs/04-arquitetura.md` §5): the sky
    /// (constellation, hexagram, moon) class 2; the figure, its shadow and the boulder class 1;
    /// the frame and the text, with the word after the recognition, class 0 (REQ-L2-08).
    #[test]
    fn composition_classes() {
        use std::collections::BTreeSet;
        let mut world = Mundus::genesis(1);
        world.verbum = world.verbum.with_ascensus(90);
        world.memoria = world
            .memoria
            .with_eventus(12)
            .with_ramus(0)
            .with_gradus(100);
        let facultates = Facultates::default();
        let indicia = Indicia {
            epoch_ms: NOON + 12 * 3_600_000,
            energy: 255,
            ..Indicia::default()
        };
        let regimen = Regimen::default();
        let chord = concentus(&world, &facultates, &indicia, &regimen);
        assert!(chord.nocturnal && chord.luna != 0, "the moon is up");
        let comp = compositio(320, 180, 6);
        for (layer, expected) in [("sky", 2u8), ("actors", 1), ("text", 0), ("frame", 0)] {
            let mut buf = std::vec![VOID; 320 * 180];
            let mut tab = Tabula::new(320, 180, &mut buf).unwrap();
            let mut pictor = Pictor::flat(&mut tab);
            match layer {
                "sky" => compose_sky(
                    &mut pictor,
                    &comp,
                    comp.sky,
                    &chord,
                    &indicia,
                    &regimen,
                    0x10,
                    0b10_1101,
                ),
                "actors" => compose_actors(&mut pictor, &comp, &chord, &indicia, &world),
                "text" => compose_text(&mut pictor, &comp, &regimen, &world, &LINES, true),
                _ => draw_frame(&mut pictor, &comp, 32, 255, 0xACE1, 1, 9, 0),
            }
            let classes: BTreeSet<u8> = buf
                .iter()
                .filter(|&&p| p != VOID)
                .map(|&p| stratum_of(p))
                .collect();
            assert_eq!(classes, BTreeSet::from([expected]), "{layer}");
        }
    }

    /// The hexagram is drawn only where the sky holds it: at least 12·s wide and 12·s high
    /// (REQ-L5-07).
    #[test]
    fn hexagram_needs_room() {
        let world = Mundus::genesis(1);
        let facultates = Facultates::default();
        let indicia = Indicia {
            epoch_ms: NOON,
            energy: 255,
            ..Indicia::default()
        };
        let regimen = Regimen {
            layers: HEXAGRAMMA,
            ..Regimen::default()
        };
        let chord = concentus(&world, &facultates, &indicia, &regimen);
        let comp = compositio(320, 180, 6);
        for (w, h, drawn) in [
            (181u16, 11u16, false),
            (11, 23, false),
            (12, 12, true),
            (181, 23, true),
        ] {
            let mut buf = std::vec![VOID; 320 * 180];
            let mut tab = Tabula::new(320, 180, &mut buf).unwrap();
            let mut pictor = Pictor::flat(&mut tab);
            let sky = Rect { w, h, ..comp.sky };
            compose_sky(
                &mut pictor,
                &comp,
                sky,
                &chord,
                &indicia,
                &regimen,
                0x10,
                0b11_1111,
            );
            let marks = buf.iter().filter(|&&p| p != VOID).count();
            assert_eq!(marks > 0, drawn, "sky {w}×{h}: {marks} marks");
        }
    }

    /// Letters quantize every cell above the plinth, including the partial cells of the right
    /// border and of the band between the scene and the plinth: there only the slate of the
    /// cells (tone 1) and the ramp glyphs (tone 12) remain (REQ-L5-03).
    #[test]
    fn littera_quantizes_partial_cells() {
        let (w, h) = (322u16, 180u16);
        let letters = frame(w, h, Stilus::Littera, Modus::Vigilia, 255);
        let comp = compositio(w, h, 6);
        let (wu, scene) = (usize::from(w), usize::from(comp.scene_h));
        assert!(
            wu % 6 != 0 && scene % 10 != 0,
            "partial cells on both sides"
        );
        let (right, band) = (wu - wu % 6, scene - scene % 10);
        for y in 0..scene {
            for x in 0..wu {
                if x >= right || y >= band {
                    let lum = lum_of(letters[x + y * wu]);
                    assert!(lum == 1 || lum == 12, "({x}, {y}) keeps tone {lum}");
                }
            }
        }
        let plinth = scene * wu;
        let chalk = frame(w, h, Stilus::Creta, Modus::Vigilia, 255);
        assert_eq!(
            &letters[plinth..],
            &chalk[plinth..],
            "the plinth is left as drawn"
        );
    }

    /// After the recognition the sixteen hexadecimal digits of the word stand as glyph cells at
    /// the right end of the first row, in the tone of the raster digits, and the scene behind
    /// them carries no raster digits; the cell render refuses a scratch of other dimensions
    /// without writing (REQ-L8-02, REQ-H-02).
    #[test]
    fn cellae_agnitio_digits() {
        let (cols, rows) = (80u16, 24u16);
        let facultates = Facultates {
            width: cols,
            height: rows,
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
        let cellae_of = |world: &Mundus| {
            let mut scratch = [0u8; 80 * 24];
            let mut cells = [0u16; 80 * 24];
            let mut tab = Tabula::new(cols, rows, &mut scratch).unwrap();
            let mut grid = Cellae::new(cols, rows, &mut cells).unwrap();
            let done = render_cellae(
                world,
                &facultates,
                &indicia,
                &regimen,
                &LINES,
                &mut tab,
                &mut grid,
            );
            (done, cells)
        };
        let mut world = Mundus::genesis(9);
        world.memoria = world
            .memoria
            .with_eventus(12)
            .with_ramus(0)
            .with_gradus(100);
        let (done, cells) = cellae_of(&world);
        assert!(done);
        for (k, digit) in hex16(world.verbum.bits()).iter().enumerate() {
            let cell = cells[usize::from(cols) - 16 + k];
            assert_eq!(
                cell & 0xFF,
                u16::from(cell_glyph(char::from(*digit))),
                "digit {k}"
            );
            assert_eq!((cell >> 8) & 0xF, u16::from(LUM_VERBUM), "digit {k}");
        }
        let mut quiet = world;
        quiet.memoria = quiet.memoria.with_eventus(3).with_ramus(3);
        let (_, plain) = cellae_of(&quiet);
        assert_eq!(
            &cells[usize::from(cols)..],
            &plain[usize::from(cols)..],
            "below the first row the cells are the scene of the same world"
        );
        world.memoria = world.memoria.with_gradus(240);
        let (_, late) = cellae_of(&world);
        assert!(
            late[usize::from(cols) - 16..usize::from(cols)]
                .iter()
                .all(|&c| RAMP.contains(&((c & 0xFF) as u8))),
            "no digits once the recognition is over"
        );
        let mut scratch = [0u8; 10];
        let mut cells = [0xBEEFu16; 80 * 24];
        let mut tab = Tabula::new(5, 2, &mut scratch).unwrap();
        let mut grid = Cellae::new(cols, rows, &mut cells).unwrap();
        let done = render_cellae(
            &world,
            &facultates,
            &indicia,
            &regimen,
            &LINES,
            &mut tab,
            &mut grid,
        );
        assert!(!done);
        assert!(cells.iter().all(|&c| c == 0xBEEF), "nothing written");
    }

    /// Every chronicle line keeps at least 40 glyphs (`docs/07-narrativa.md` §3): line i of the
    /// plinth steps in by min(3·s·i, 6·s·(glyphs per line − 40)) pixels, nothing when a line
    /// holds 40 glyphs or fewer, and holds ⌊(width of the chronicles − step)/(6·s)⌋ glyphs.
    #[test]
    fn plinth_lines_keep_forty_glyphs() {
        for width in 240u16..=700 {
            for height in [60u16, 180, 400] {
                let comp = compositio(width, height, 6);
                let (s, glyphs) = (i32::from(comp.s), i32::from(comp.glyphs_per_line));
                for i in 0..6u8 {
                    let (indent, room) = plinth_line(&comp, i);
                    let law = if glyphs > 40 {
                        (3 * s * i32::from(i)).min(6 * s * (glyphs - 40))
                    } else {
                        0
                    };
                    assert_eq!(indent, law, "{width}×{height} line {i}");
                    assert_eq!(
                        room as i32,
                        (i32::from(comp.annales.w) - indent) / (6 * s),
                        "{width}×{height} line {i}"
                    );
                    assert!(
                        room as i32 >= glyphs.min(40),
                        "{width}×{height} line {i}: {room} glyphs"
                    );
                }
            }
        }
    }

    /// The ramp of the Littera stilus points at ` .:-=+*#%@` in `ORDO`, darkest first
    /// (REQ-L5-03).
    #[test]
    fn ramp_matches_ordo() {
        let ramp: Vec<char> = RAMP
            .iter()
            .map(|&i| ORDO.chars().nth(usize::from(i)).unwrap())
            .collect();
        assert_eq!(ramp, " .:-=+*#%@".chars().collect::<Vec<char>>());
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
        assert!(render_cellae(
            &world,
            &facultates,
            &indicia,
            &regimen,
            &LINES,
            &mut tab,
            &mut grid,
        ));
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

    /// Lines longer than the grid are cut at the last column and the lines beyond the last row are
    /// left out: with fewer columns than glyphs and fewer rows than lines, each row of the grid is
    /// the exact prefix of its line, in order, and nothing else appears (REQ-L5-03).
    #[test]
    fn littera_truncates_lines() {
        let lines = [
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
            "THE LEAF CRACKS UNDER THE SANDAL.",
            "AN OWL LEAVES THE OLIVE TREE.",
            "A LAMP FALLS IN ATHENS.",
        ];
        let (cols, rows) = (20u16, 3u16);
        let world = Mundus::genesis(1);
        let facultates = Facultates {
            width: cols,
            height: rows,
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
        let mut scratch = [0u8; 20 * 3];
        let mut cells = [0u16; 20 * 3];
        let mut tab = Tabula::new(cols, rows, &mut scratch).unwrap();
        let mut grid = Cellae::new(cols, rows, &mut cells).unwrap();
        assert!(render_cellae(
            &world,
            &facultates,
            &indicia,
            &regimen,
            &lines,
            &mut tab,
            &mut grid,
        ));
        for (row, line) in lines.iter().take(usize::from(rows)).enumerate() {
            let expected: Vec<u16> = line
                .chars()
                .take(usize::from(cols))
                .map(|c| {
                    let glyph = ORDO.chars().position(|o| o == c).unwrap() as u16;
                    glyph | (u16::from(LUM_TEXT) << 8)
                })
                .collect();
            assert_eq!(
                expected.len(),
                usize::from(cols),
                "every line is longer than the grid"
            );
            let start = row * usize::from(cols);
            assert_eq!(
                &cells[start..start + usize::from(cols)],
                &expected[..],
                "row {row}"
            );
        }
    }

    /// Extreme curvature at wide desktop frames keeps every write in the prefix of
    /// width·height pixels and the frame non-empty (REQ-L2-06, REQ-H-02).
    #[test]
    fn warp_keeps_inside_frame() {
        for (width, height) in [(1920u16, 1080u16), (3840, 1080)] {
            for kappa in [-64i8, 64] {
                let mut world = Mundus::genesis(3);
                world.verbum = world
                    .verbum
                    .with_modus(Modus::Somnium)
                    .with_curvatura(kappa)
                    .with_ascensus(120);
                let facultates = Facultates {
                    width,
                    height,
                    ..Facultates::default()
                };
                let indicia = Indicia {
                    epoch_ms: NOON + 14 * 3_600_000,
                    energy: 200,
                    idle_s: 3600,
                    ..Indicia::default()
                };
                let area = usize::from(width) * usize::from(height);
                let mut buf = std::vec![0xA5u8; area + 64];
                let mut tab = Tabula::new(width, height, &mut buf).unwrap();
                render(
                    &world,
                    &facultates,
                    &indicia,
                    &Regimen::default(),
                    &LINES,
                    &mut tab,
                );
                assert!(
                    buf[area..].iter().all(|&b| b == 0xA5),
                    "{width}×{height} κ {kappa} wrote past the frame"
                );
                assert!(
                    buf[..area].iter().any(|&p| lum_of(p) >= 12),
                    "{width}×{height} κ {kappa} empty frame"
                );
            }
        }
    }

    /// The core writes only the prefix of width·height pixels and of cols·rows cells: renders in
    /// the four stili and cell renders, over 64 frame sizes from the Galois generator (area at
    /// most 16 384) and 1×1, 272×60, 65 535×1 and 1×65 535, under curvature −64, 0 and 64, with
    /// worlds, measurements and regimina dispersed by `mix`, leave 64 sentinels after the frame
    /// intact and never panic (REQ-H-02).
    #[test]
    fn render_never_exceeds_slice() {
        use crate::memoria::Memoria;
        use crate::natura::{galois, mix};
        use crate::verbum::Verbum;
        let mut sizes: Vec<(u16, u16)> = Vec::new();
        let mut semen = 0xACE1u16;
        while sizes.len() < 64 {
            semen = galois(semen);
            let w = 1 + (semen & 0xFF);
            semen = galois(semen);
            let h = 1 + semen % (16_384 / w);
            sizes.push((w, h));
        }
        sizes.extend([(1, 1), (272, 60), (65_535, 1), (1, 65_535)]);
        let lines = [
            "AÇÃO E CORAÇÃO NÃO SÃO PÊSAMES.",
            "THE LEAF CRACKS UNDER THE SANDAL.",
            "lower case, emoji 😀 and marks",
            "A LAMP FALLS IN ATHENS.",
            "THE LIBRARY BURNS.",
            "ROME WAITS.",
            "A SEVENTH LINE",
        ];
        let mut key = 0x2545_F491_4F6C_DD1Du64;
        for (w, h) in sizes {
            let area = usize::from(w) * usize::from(h);
            for kappa in [-64i8, 0, 64] {
                for stilus in [
                    Stilus::Creta,
                    Stilus::Musivum,
                    Stilus::Littera,
                    Stilus::Bicolor,
                ] {
                    key = mix(key);
                    let world = Mundus {
                        verbum: Verbum::new(key).with_curvatura(kappa),
                        memoria: Memoria::new(mix(key ^ 0x5A5A)),
                    };
                    let facultates = Facultates {
                        width: w,
                        height: h,
                        ..Facultates::default()
                    };
                    let indicia = Indicia {
                        epoch_ms: (key as i64) >> 20,
                        energy: (key >> 8) as u8,
                        leaving: key & 1 == 1,
                        ..Indicia::default()
                    };
                    let regimen = Regimen {
                        stilus,
                        strata: 1 + ((key >> 16) & 3) as u8,
                        layers: (key >> 24) as u8,
                        lines: ((key >> 32) % 7) as u8,
                        ..Regimen::default()
                    };
                    let mut buf = std::vec![0xA5u8; area + 64];
                    let mut tab = Tabula::new(w, h, &mut buf).unwrap();
                    render(&world, &facultates, &indicia, &regimen, &lines, &mut tab);
                    assert!(
                        buf[area..].iter().all(|&b| b == 0xA5),
                        "{w}×{h} κ {kappa} {stilus:?}: pixels past the frame"
                    );
                    let mut scratch = std::vec![0xA5u8; area + 64];
                    let mut cells = std::vec![0xA5A5u16; area + 64];
                    let mut tab = Tabula::new(w, h, &mut scratch).unwrap();
                    let mut grid = Cellae::new(w, h, &mut cells).unwrap();
                    assert!(render_cellae(
                        &world,
                        &facultates,
                        &indicia,
                        &regimen,
                        &lines,
                        &mut tab,
                        &mut grid,
                    ));
                    assert!(
                        scratch[area..].iter().all(|&b| b == 0xA5),
                        "{w}×{h} κ {kappa} {stilus:?}: scratch past the grid"
                    );
                    assert!(
                        cells[area..].iter().all(|&c| c == 0xA5A5),
                        "{w}×{h} κ {kappa} {stilus:?}: cells past the grid"
                    );
                }
            }
        }
    }

    /// Writes the six golden scenarios as PPM files, each at its own dimensions, for inspection
    /// when requested.
    #[test]
    #[ignore = "writes files; run on request"]
    fn dump_golden_ppm() {
        use std::io::Write;
        let dir = std::env::var("SISYPHIUM_DUMP").unwrap_or_else(|_| "/tmp".into());
        for (name, w, h, m, i, r) in golden_scenarios() {
            let mut buf = std::vec![0u8; usize::from(w) * usize::from(h)];
            let mut tab = Tabula::new(w, h, &mut buf).unwrap();
            let f = Facultates {
                width: w,
                height: h,
                ..Facultates::default()
            };
            render(&m, &f, &i, &r, &LINES, &mut tab);
            let c = crate::concentus::concentus(&m, &f, &i, &r);
            let pal = crate::concentus::palette(&c);
            let mut out = std::format!("P6\n{w} {h}\n255\n").into_bytes();
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
        let scenarios: Vec<(&str, Mundus, Indicia, Regimen)> = golden_scenarios()
            .into_iter()
            .filter(|&(_, w, _, _, _, _)| w == 320)
            .map(|(name, _, _, m, i, r)| (name, m, i, r))
            .collect();
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

    /// A golden scenario: its name, the frame's width and height, the world, the measurements
    /// and the regimen.
    type Scaena = (&'static str, u16, u16, Mundus, Indicia, Regimen);

    /// The scenarios of `docs/08-verificacao.md` §7 and their two variants: vigilia in the
    /// Littera stilus (REQ-L5-03, one golden frame per stilus) and somnium at 1920×1080, which
    /// locks the curvature projection at desktop resolution (REQ-L2-06). The 4 096 ticks of §7
    /// need `step`, which arrives with checkpoint 3; until then each world fixes directly the
    /// fields those ticks would set (`ascensus`, `onus`, `tranquillitas`, `gradus`,
    /// `clausurae`, `modus`, `curvatura`).
    fn golden_scenarios() -> [Scaena; 6] {
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
        let mut quies = Mundus::genesis(4);
        quies.verbum = quies.verbum.with_modus(Modus::Quies);
        let day = Indicia {
            epoch_ms: NOON,
            energy: 255,
            ..Indicia::default()
        };
        let night = Indicia {
            epoch_ms: 1_797_831_000_000,
            energy: 200,
            idle_s: 3600,
            ..Indicia::default()
        };
        [
            ("vigilia", 320, 180, vigil, day, Regimen::default()),
            (
                "acedia",
                320,
                180,
                acedia,
                Indicia {
                    epoch_ms: 1_790_112_600_000,
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
            ("somnium", 320, 180, dream, night, Regimen::default()),
            (
                "quies",
                320,
                180,
                quies,
                Indicia {
                    epoch_ms: 1_774_008_000_000,
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
            (
                "littera",
                320,
                180,
                vigil,
                day,
                Regimen {
                    stilus: Stilus::Littera,
                    ..Regimen::default()
                },
            ),
            (
                "somnium-1920x1080",
                1920,
                1080,
                dream,
                night,
                Regimen::default(),
            ),
        ]
    }

    /// The golden hashes (REQ-L5-09); updated only with an intentional change of form.
    #[test]
    fn golden_frames() {
        for ((name, w, h, m, i, r), (expected_name, hash)) in
            golden_scenarios().into_iter().zip(GOLDEN)
        {
            assert_eq!(name, expected_name);
            let mut buf = std::vec![0u8; usize::from(w) * usize::from(h)];
            let mut tab = Tabula::new(w, h, &mut buf).unwrap();
            let f = Facultates {
                width: w,
                height: h,
                ..Facultates::default()
            };
            render(&m, &f, &i, &r, &LINES, &mut tab);
            assert_eq!(fnv1a(&buf), hash, "{name}: {:#018x}", fnv1a(&buf));
        }
    }
}
