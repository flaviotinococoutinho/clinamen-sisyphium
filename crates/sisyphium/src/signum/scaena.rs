//! The scene composer: one frame from the world, the host, the regimen and the visible lines, in
//! the order of `docs/04-arquitetura.md` §5, and the four stili (REQ-L5-03, REQ-L5-09, REQ-H-04).

use crate::arbitrium::{annales, line_limit, revealed};
use crate::concentus::{Concentus, concentus};
use crate::corpus::{
    Figura, Postura, boulder_texture, draw_boulder, draw_figure, gait_phase, postura,
};
use crate::hospes::{
    CAELUM, Cellae, Facultates, HEXAGRAMMA, Indicia, MARGO, PUNCTA, RUINAE, Regimen, Stilus, Tabula,
};
use crate::mundus::Mundus;
use crate::signum::caelum::{constellation, graph, hexagram, moon, sky_region};
use crate::signum::littera::{ORDO, cell_glyph, text};
use crate::signum::margo::draw_frame;
use crate::signum::mons::{draw_ruins, draw_slope};
use crate::signum::pixel::{VOID, label_of, lum_of, pixel, stratum_of};
use crate::signum::raster::{Modo, Pictor, bayer};
use crate::spatium::{Compositio, PHI_INV2, Rect, compositio, isqrt, part};

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

/// Whether the present word shows after the recognition: `eventus` 12 with `gradus` below
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

/// The sky in its region (REQ-L5-12): constellation, graph, hexagram and moon. Returns the
/// first node for the aphorism drawn after the world (REQ-L5-06..08; `docs/04-arquitetura.md` §5).
#[allow(clippy::too_many_arguments)]
fn compose_sky(
    pictor: &mut Pictor,
    comp: &Compositio,
    sky: Rect,
    chord: &Concentus,
    indicia: &Indicia,
    regimen: &Regimen,
    mundus: &Mundus,
) -> Option<(i32, i32)> {
    let mut first = None;
    let automatum = mundus.verbum.automatum();
    let tranquillitas = mundus.verbum.tranquillitas();
    let scale = comp.s;
    if regimen.has(CAELUM) && sky.w > 0 && sky.h > 0 {
        constellation(pictor, sky, automatum, 6, CLASSIS_CAELI);
        let memoria = mundus.memoria;
        let shown = if memoria.eventus() < 8 && memoria.ramus() < 3 {
            revealed(memoria.gradus())
        } else {
            0
        };
        let trunk = (
            i32::from(comp.m),
            i32::from(part(pictor.height(), PHI_INV2) / 2),
        );
        first = graph(
            pictor,
            sky,
            trunk,
            memoria.eventus(),
            memoria.ramus(),
            shown,
            mundus.verbum.onus(),
            scale,
            9,
            14,
        );
    }
    if let Some((x, y)) = hexagram_origin(comp, sky, indicia, regimen) {
        hexagram(pictor, x, y, scale, tranquillitas, 8, CLASSIS_CAELI);
    }
    if let Some((x, y, radius)) = moon_geometry(sky, chord) {
        moon(pictor, x, y, radius, chord.luna, 12, chord.australis);
    }
    first
}

/// The hexagram's origin when its layer, energy and sky room permit it (REQ-L5-07).
fn hexagram_origin(
    comp: &Compositio,
    sky: Rect,
    indicia: &Indicia,
    regimen: &Regimen,
) -> Option<(i32, i32)> {
    let side = i32::from(comp.s).wrapping_mul(12);
    (regimen.has(HEXAGRAMMA)
        && indicia.energy >= 77
        && i32::from(sky.w) >= side
        && i32::from(sky.h) >= side)
        .then(|| {
            (
                i32::from(sky.x)
                    .wrapping_add(i32::from(sky.w))
                    .wrapping_sub(side),
                i32::from(sky.y),
            )
        })
}

/// The moon's centre and radius while the night has an illuminated phase (REQ-L5-08).
fn moon_geometry(sky: Rect, chord: &Concentus) -> Option<(i32, i32, i32)> {
    (chord.nocturnal && chord.luna != 0).then(|| {
        (
            i32::from(sky.x).wrapping_add(i32::from(sky.w).wrapping_mul(3) / 4),
            i32::from(sky.y).wrapping_add(i32::from(sky.h) / 2),
            (i32::from(sky.h) / 3).max(2),
        )
    })
}

/// Logical rectangles protected from the aphorism: the hexagram's twelve-scale-unit cell
/// (including the centred stroke) and the moon's full disc (REQ-L5-06..08), expanded by one
/// scale unit to separate projected pixels under quantization. Edges are exclusive.
fn sky_obstacles(
    comp: &Compositio,
    sky: Rect,
    chord: &Concentus,
    indicia: &Indicia,
    regimen: &Regimen,
) -> [Option<[i32; 4]>; 2] {
    let hex = hexagram_origin(comp, sky, indicia, regimen).map(|(x, y)| {
        let side = i32::from(comp.s).wrapping_mul(12);
        let overhang = i32::from((comp.s as u8).max(1).wrapping_sub(1) / 2);
        [
            x.wrapping_sub(overhang),
            y.wrapping_sub(overhang),
            x.wrapping_add(side).wrapping_add(overhang),
            y.wrapping_add(side).wrapping_add(overhang),
        ]
    });
    let lunar = moon_geometry(sky, chord).map(|(x, y, radius)| {
        [
            x.wrapping_sub(radius),
            y.wrapping_sub(radius),
            x.wrapping_add(radius).wrapping_add(1),
            y.wrapping_add(radius).wrapping_add(1),
        ]
    });
    let gap = i32::from(comp.s);
    [hex, lunar].map(|bounds| {
        bounds.map(|[left, top, right, bottom]| {
            [
                left.wrapping_sub(gap),
                top.wrapping_sub(gap),
                right.wrapping_add(gap),
                bottom.wrapping_add(gap),
            ]
        })
    })
}

/// The nearest usable aphorism anchor (REQ-L5-06): at nominal x, choose among nominal y and
/// positions immediately above/below the protected symbols, by least vertical displacement,
/// ties above. If none fits, try integer horizontal distances in increasing order, left before
/// right, with the same vertical candidates. Every box stays inside the logical frame/scene
/// and the projected writing region. No position means no annotation on that small surface.
fn aphorism_anchor(
    pictor: &Pictor,
    nominal: (i32, i32),
    width: i32,
    scale: u16,
    scene_h: u16,
    obstacles: [Option<[i32; 4]>; 2],
) -> Option<(i32, i32)> {
    let unit = i32::from(scale);
    let height = unit.wrapping_mul(12);
    let frame_width = i32::from(pictor.width());
    if height > i32::from(scene_h) || width > frame_width {
        return None;
    }
    let below = unit.wrapping_mul(11);
    for distance in 0..=frame_width {
        for x in [
            nominal.0.wrapping_sub(distance),
            nominal.0.wrapping_add(distance),
        ] {
            let left = x.wrapping_sub(unit);
            let right = left.wrapping_add(width);
            if left < 0 || right > frame_width {
                continue;
            }
            let sides = obstacles
                .into_iter()
                .flatten()
                .flat_map(|[_, top, _, bottom]| {
                    [top.wrapping_sub(below), bottom.wrapping_add(unit)]
                });
            let chosen = core::iter::once(nominal.1)
                .chain(sides)
                .filter(|&y| {
                    let top = y.wrapping_sub(unit);
                    let bottom = y.wrapping_add(below);
                    top >= 0
                        && bottom <= i32::from(scene_h)
                        && pictor.contains_projection(left, top, width, height)
                        && obstacles.iter().flatten().all(|&[sx, sy, ex, ey]| {
                            right <= sx || left >= ex || bottom <= sy || top >= ey
                        })
                })
                .min_by_key(|&y| (y.wrapping_sub(nominal.1).unsigned_abs(), y));
            if let Some(y) = chosen {
                return Some((x, y));
            }
        }
    }
    None
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
    if comp.figure_h == 0 {
        return;
    }
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
    {
        for (i, line) in (0..=u8::MAX).zip(
            lines
                .iter()
                .take(usize::from(line_limit(regimen, u16::MAX))),
        ) {
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
    if hexadecimal {
        compose_word(pictor, comp, mundus);
    }
}

/// Draws the present word after recognition (REQ-L8-02), through the scene projection.
fn compose_word(pictor: &mut Pictor, comp: &Compositio, mundus: &Mundus) {
    let scale = comp.s;
    if recognized(mundus) {
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
    let first = compose_sky(&mut pictor, &comp, sky, chord, indicia, regimen, mundus);
    compose_ground(tab, &comp, chord, indicia, regimen, mundus);
    let mut pictor = Pictor::new(tab, chord.curvatura);
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
    compose_aphorism(
        &mut Pictor::ground(tab, chord.curvatura, comp.scene_h, 15),
        &comp,
        sky_obstacles(&comp, sky, chord, indicia, regimen),
        first,
        annales(mundus.memoria, chord.lingua, indicia.leaving).aphorismus,
    );
    let mut pictor = Pictor::flat(tab);
    compose_text(&mut pictor, &comp, regimen, mundus, lines, false);
    if hexadecimal {
        compose_word(&mut Pictor::new(tab, chord.curvatura), &comp, mundus);
    }
}

/// Reserves a slate rectangle for the aphorism, near the first node's anchor and under the
/// scene projection (REQ-L5-06). The 6s by 10s font cell gets one scale unit of space on each
/// side: (6·characters + 2)s by 12s. The caller clips projected writes above the plinth.
fn compose_aphorism(
    pictor: &mut Pictor,
    comp: &Compositio,
    obstacles: [Option<[i32; 4]>; 2],
    first: Option<(i32, i32)>,
    aphorism: Option<&str>,
) {
    let scale = comp.s;
    if let (Some((x, y)), Some(aphorism)) = (first, aphorism) {
        let unit = i32::from(scale);
        let x = x.wrapping_add(unit.wrapping_mul(2));
        let y = y.wrapping_sub(unit.wrapping_mul(5));
        let width = (aphorism.chars().count() as i32)
            .wrapping_mul(6)
            .wrapping_add(2)
            .wrapping_mul(unit);
        let Some((x, y)) = aphorism_anchor(pictor, (x, y), width, scale, comp.scene_h, obstacles)
        else {
            return;
        };
        pictor.fill(
            x.wrapping_sub(unit),
            y.wrapping_sub(unit),
            width,
            unit.wrapping_mul(12),
            1,
            CLASSIS_CAELI,
            false,
        );
        text(pictor, x, y, scale, aphorism, 12, CLASSIS_CAELI);
    }
}

/// The slope and ruins keep the frame's projection but stop at the plinth and the chord's
/// tone ceiling (REQ-L2-06, REQ-L5-11; `docs/05-estetica.md` §10).
fn compose_ground(
    tab: &mut Tabula,
    comp: &Compositio,
    chord: &Concentus,
    indicia: &Indicia,
    regimen: &Regimen,
    mundus: &Mundus,
) {
    let mut pictor = Pictor::ground(
        tab,
        chord.curvatura,
        comp.scene_h,
        15u8.saturating_sub(chord.contrastus),
    );
    draw_slope(
        &mut pictor,
        comp,
        mundus.verbum.automatum(),
        regimen.strata,
        indicia.offset,
        3,
        6,
        regimen.has(PUNCTA),
    );
    if regimen.has(RUINAE) {
        draw_ruins(&mut pictor, comp, mundus.memoria.clausurae(), 7, comp.s);
    }
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

/// Normalizes only the medium declared by the actual rendering call before selecting the
/// chord (REQ-L5-11): raster has no cell-row limit, while cells use the actual grid dimensions.
fn surface_chord(
    mundus: &Mundus,
    facultates: &Facultates,
    indicia: &Indicia,
    regimen: &Regimen,
    grid: Option<(u16, u16)>,
) -> Concentus {
    let effective = match grid {
        Some((cols, rows)) => Facultates {
            cells: true,
            width: cols,
            height: rows,
            ..*facultates
        },
        None => Facultates {
            cells: false,
            ..*facultates
        },
    };
    concentus(mundus, &effective, indicia, regimen)
}

/// Renders one raster frame in the regimen's stilus (REQ-L5-03, REQ-H-04). The chord uses
/// `cells = false` because this surface has no cell-row limit, preserving the caller's other
/// capabilities and the chosen regimen (REQ-L5-11).
pub fn render(
    mundus: &Mundus,
    facultates: &Facultates,
    indicia: &Indicia,
    regimen: &Regimen,
    lines: &[&str],
    tab: &mut Tabula,
) {
    let chord = surface_chord(mundus, facultates, indicia, regimen, None);
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
/// cols·rows cells and pixels are read and written (REQ-H-02). The chord uses cell capabilities
/// normalized to the actual grid, so its register and the letters share the same row limit
/// (REQ-L5-11); recognition digits remain independent of ANNALES.
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
    let chord = surface_chord(
        mundus,
        facultates,
        indicia,
        regimen,
        Some((cells.cols, cells.rows)),
    );
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
    let shown = lines
        .len()
        .min(usize::from(line_limit(regimen, cells.rows)));
    for (i, line) in lines.iter().take(shown).enumerate() {
        let row = rows.wrapping_sub(shown).wrapping_add(i);
        for (k, ch) in line.chars().take(cols).enumerate() {
            if let Some(cell) = cells.cells.get_mut(row.wrapping_mul(cols).wrapping_add(k)) {
                *cell = letter(ch, LUM_TEXT);
            }
        }
    }
    if recognized(mundus) && rows > 0 {
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
    ("vigilia", 0x5A6A_21F0_DB26_18D1),
    ("acedia", 0xF476_2F2C_93C8_105B),
    ("somnium", 0xAC2D_7475_7129_BCA8),
    ("quies", 0xD668_7590_F327_FB9F),
    ("littera", 0x32C8_0A84_CCF4_85E3),
    ("somnium-1920x1080", 0xB1AC_03C5_B13F_6E38),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hospes::{ANNALES, Cellae, Facultates, Indicia, Regimen, Stilus, Tabula};
    use crate::mundus::Mundus;
    use crate::signum::pixel::{lum_of, stratum_of};
    use crate::verbum::Modus;
    use std::vec::Vec;

    /// The actual rendering surface governs the register even when the caller declares
    /// different cell capabilities; other channels retain the same inputs (REQ-L5-11).
    #[test]
    fn render_chord_follows_surface() {
        use crate::arbitrium::Registrum;
        let mut world = Mundus::genesis(1);
        world.memoria = world.memoria.with_eventus(3).with_ramus(1).with_gradus(400);
        let declared_cells = Facultates {
            cells: true,
            height: 1,
            ..Facultates::default()
        };
        let declared_raster = Facultates::default();
        let measures = Indicia::default();
        let regimen = Regimen::default();
        assert_eq!(
            concentus(&world, &declared_cells, &measures, &regimen).registrum,
            Registrum::Intimum
        );
        assert_eq!(
            surface_chord(&world, &declared_cells, &measures, &regimen, None).registrum,
            Registrum::Abstractum
        );
        assert_eq!(
            surface_chord(&world, &declared_raster, &measures, &regimen, Some((80, 1))).registrum,
            Registrum::Intimum
        );
        assert_eq!(
            surface_chord(&world, &declared_raster, &measures, &regimen, Some((80, 2))).registrum,
            Registrum::Vivum
        );
        let mut cells_chord =
            surface_chord(&world, &declared_raster, &measures, &regimen, Some((80, 2)));
        let raster_chord = surface_chord(&world, &declared_cells, &measures, &regimen, None);
        cells_chord.registrum = Registrum::Abstractum;
        assert_eq!(cells_chord, raster_chord);
        let mut actual = std::vec![VOID;320*180];
        let mut expected = actual.clone();
        world.render(
            &declared_cells,
            &measures,
            &regimen,
            &mut Tabula::new(320, 180, &mut actual).unwrap(),
        );
        world.render(
            &declared_raster,
            &measures,
            &regimen,
            &mut Tabula::new(320, 180, &mut expected).unwrap(),
        );
        assert_eq!(actual, expected);
    }

    /// The plinth stays flat and the ground stays above it under curvature (REQ-L2-06).
    #[test]
    fn plinth_outside_dream() {
        let render_at = |kappa| {
            let mut m = Mundus::genesis(1);
            m.verbum = m.verbum.with_ascensus(90).with_curvatura(kappa);
            m.memoria = m
                .memoria
                .with_ramus(1)
                .with_gradus(400)
                .with_clausurae(0x3FFE);
            let mut buf = std::vec![0; 320 * 180];
            render(
                &m,
                &Facultates::default(),
                &Indicia {
                    epoch_ms: NOON,
                    ..Indicia::default()
                },
                &Regimen::default(),
                &LINES,
                &mut Tabula::new(320, 180, &mut buf).unwrap(),
            );
            buf
        };
        let plain = render_at(0);
        let start = usize::from(compositio(320, 180, 6).scene_h) * 320;
        for kappa in [-64, 64] {
            let curved = render_at(kappa);
            assert_eq!(
                plain[start..]
                    .iter()
                    .map(|&p| lum_of(p) == 13)
                    .collect::<Vec<_>>(),
                curved[start..]
                    .iter()
                    .map(|&p| lum_of(p) == 13)
                    .collect::<Vec<_>>()
            );
            assert_ne!(&plain[..start], &curved[..start]);
        }
    }

    /// No actor exists when the scene has no height; degenerate skies and scaled mosaics
    /// keep the caller's suffix intact (REQ-H-02, REQ-L5-06).
    #[test]
    fn degenerate_sky() {
        let mut m = Mundus::genesis(7);
        m.memoria = m.memoria.with_ramus(2).with_gradus(400);
        m.verbum = m.verbum.with_onus(230);
        for (w, h, stilus) in [
            (272u16, 60u16, Stilus::Creta),
            (1, 1, Stilus::Creta),
            (640, 360, Stilus::Musivum),
            (1920, 1080, Stilus::Musivum),
        ] {
            let mut buf = std::vec![0xA5; usize::from(w)*usize::from(h)+17];
            let f = Facultates {
                width: w,
                height: h,
                ..Facultates::default()
            };
            let r = Regimen {
                stilus,
                ..Regimen::default()
            };
            m.render(
                &f,
                &Indicia::default(),
                &r,
                &mut Tabula::new(w, h, &mut buf).unwrap(),
            );
            assert!(
                buf[usize::from(w) * usize::from(h)..]
                    .iter()
                    .all(|&p| p == 0xA5)
            );
            let comp = compositio(w, h, 6);
            if comp.scene_h == 0 {
                let mut actors = std::vec![VOID; usize::from(w)*usize::from(h)];
                let mut tab = Tabula::new(w, h, &mut actors).unwrap();
                let chord = concentus(&m, &f, &Indicia::default(), &r);
                compose_actors(
                    &mut Pictor::flat(&mut tab),
                    &comp,
                    &chord,
                    &Indicia::default(),
                    &m,
                );
                assert!(actors.iter().all(|&p| p == VOID));
            }
        }
    }

    /// Accent indices survive the cell renderer and ANNALES controls the letters (REQ-H-04).
    #[test]
    fn render_cellae_keeps_accents() {
        for enabled in [false, true] {
            let mut scratch = [0; 20 * 4];
            let mut cells = [0; 20 * 4];
            let mut r = Regimen::default();
            if !enabled {
                r.layers &= !ANNALES;
            }
            assert!(render_cellae(
                &Mundus::genesis(1),
                &Facultates::default(),
                &Indicia::default(),
                &r,
                &["AÇÃO"],
                &mut Tabula::new(20, 4, &mut scratch).unwrap(),
                &mut Cellae::new(20, 4, &mut cells).unwrap()
            ));
            if enabled {
                assert_eq!(
                    cells[60..64].iter().map(|&c| c & 255).collect::<Vec<_>>(),
                    [0, 81, 73, 14]
                );
            } else {
                assert!(cells.iter().all(|&c| (c >> 8) & 15 != 13));
            }
        }
    }

    /// The burden's corrupted harmony reaches the composed sky (REQ-L5-12).
    #[test]
    fn harmony_reaches_scene() {
        let at = |onus| {
            let mut m = Mundus::genesis(1);
            m.verbum = m.verbum.with_onus(onus);
            let mut buf = std::vec![VOID; 320*180];
            m.render(
                &Facultates::default(),
                &Indicia {
                    epoch_ms: NOON,
                    ..Indicia::default()
                },
                &Regimen::default(),
                &mut Tabula::new(320, 180, &mut buf).unwrap(),
            );
            buf
        };
        assert_ne!(&at(200)[..320 * 32], &at(201)[..320 * 32]);
    }

    /// Every ground mark is bounded by 15 minus the chord contrast and remains above the
    /// plinth after contraction or expansion (REQ-L2-06, REQ-L5-11).
    #[test]
    fn contrast_governs_ground() {
        for (energy, modus, contrast) in [
            (255, Modus::Vigilia, 8),
            (120, Modus::Vigilia, 10),
            (60, Modus::Vigilia, 12),
            (255, Modus::Quies, 15),
        ] {
            for kappa in [-64, 0, 64] {
                for dither in [false, true] {
                    let mut m = Mundus::genesis(1);
                    m.verbum = m.verbum.with_modus(modus).with_curvatura(kappa);
                    m.memoria = m.memoria.with_clausurae(0x3FFE);
                    let i = Indicia {
                        epoch_ms: NOON,
                        energy,
                        ..Indicia::default()
                    };
                    let r = Regimen {
                        layers: RUINAE | if dither { PUNCTA } else { 0 },
                        ..Regimen::default()
                    };
                    let chord = concentus(&m, &Facultates::default(), &i, &r);
                    assert_eq!(chord.contrastus, contrast);
                    let comp = compositio(320, 180, 6);
                    let mut buf = std::vec![VOID;320*180+8];
                    compose_ground(
                        &mut Tabula::new(320, 180, &mut buf).unwrap(),
                        &comp,
                        &chord,
                        &i,
                        &r,
                        &m,
                    );
                    let marks: Vec<_> = buf.iter().copied().filter(|&p| p != VOID).collect();
                    assert_ne!(marks, Vec::<u8>::new());
                    assert!(marks.iter().all(|&p| lum_of(p) <= 15 - contrast));
                    assert!(
                        buf[usize::from(comp.scene_h) * 320..]
                            .iter()
                            .all(|&p| p == VOID)
                    );
                }
            }
        }
    }

    /// The aphorism's reserved slate keeps its font intact after the graph, hexagram and
    /// ground, including a curved scene (REQ-L5-06).
    #[test]
    fn aphorism_survives_scene() {
        for eventus in [1, 3] {
            for kappa in [-64, 0, 64] {
                let mut world = Mundus::genesis(7);
                world.verbum = world.verbum.with_curvatura(kappa).with_ascensus(90);
                world.memoria = world
                    .memoria
                    .with_eventus(eventus)
                    .with_ramus(2)
                    .with_gradus(400);
                let f = Facultates {
                    lingua: crate::hospes::Lingua::Pt,
                    ..Facultates::default()
                };
                let i = Indicia {
                    epoch_ms: NOON,
                    ..Indicia::default()
                };
                let regimen = Regimen::default();
                let comp = compositio(320, 180, 6);
                let first = crate::signum::caelum::node(comp.sky, eventus, 2, 0, 32);
                let aphorism = annales(world.memoria, f.lingua, false).aphorismus.unwrap();
                let mut expected = std::vec![VOID;320*180];
                let mut tab = Tabula::new(320, 180, &mut expected).unwrap();
                let mut painter = Pictor::ground(&mut tab, kappa, comp.scene_h, 15);
                let (x, y) = (first.0 + 2, if eventus == 1 || kappa > 0 { 30 } else { 4 });
                painter.fill(
                    x - 1,
                    y - 1,
                    6 * aphorism.chars().count() as i32 + 2,
                    12,
                    1,
                    2,
                    false,
                );
                text(&mut painter, x, y, 1, aphorism, 12, 2);
                let mut actual = std::vec![VOID;320*180];
                world.render(
                    &f,
                    &i,
                    &regimen,
                    &mut Tabula::new(320, 180, &mut actual).unwrap(),
                );
                for (at, &pixel) in expected.iter().enumerate().filter(|(_, p)| **p != VOID) {
                    assert_eq!(
                        actual[at], pixel,
                        "event {eventus}, kappa {kappa}, pixel {at}"
                    );
                }
            }
        }
    }

    /// Both astronomical signs survive every graph branch by day and night, under
    /// contraction and expansion; larger frames exercise scales two and six (REQ-L5-06..08).
    #[test]
    fn aphorism_preserves_symbols() {
        for (w, h) in [(320u16, 180u16), (640, 360), (1920, 1080)] {
            for eventus in 0..8 {
                if w > 320 && ![1, 3].contains(&eventus) {
                    continue;
                }
                for ramus in 0..3 {
                    if w > 320 && ramus != 2 {
                        continue;
                    }
                    for kappa in [-64, 0, 64] {
                        for onus in [32, 230] {
                            for epoch_ms in [NOON, NOON + 12 * 3_600_000] {
                                let mut m = Mundus::genesis(7);
                                m.verbum = m
                                    .verbum
                                    .with_onus(onus)
                                    .with_curvatura(kappa)
                                    .with_ascensus(90);
                                m.memoria = m
                                    .memoria
                                    .with_eventus(eventus)
                                    .with_ramus(ramus)
                                    .with_gradus(400);
                                let f = Facultates {
                                    width: w,
                                    height: h,
                                    lingua: crate::hospes::Lingua::Pt,
                                    ..Facultates::default()
                                };
                                let i = Indicia {
                                    epoch_ms,
                                    ..Indicia::default()
                                };
                                let r = Regimen::default();
                                let chord = concentus(&m, &f, &i, &r);
                                let comp = compositio(w, h, 6);
                                let sky = sky_region(&comp, w, onus);
                                let mut expected = std::vec![VOID;usize::from(w)*usize::from(h)];
                                let mut tab = Tabula::new(w, h, &mut expected).unwrap();
                                let _ = compose_sky(
                                    &mut Pictor::new(&mut tab, kappa),
                                    &comp,
                                    sky,
                                    &chord,
                                    &i,
                                    &Regimen {
                                        layers: HEXAGRAMMA,
                                        ..r
                                    },
                                    &m,
                                );
                                let mut actual = std::vec![VOID;usize::from(w)*usize::from(h)];
                                m.render(&f, &i, &r, &mut Tabula::new(w, h, &mut actual).unwrap());
                                for (at, &pixel) in
                                    expected.iter().enumerate().filter(|(_, p)| **p != VOID)
                                {
                                    assert_eq!(
                                        actual[at], pixel,
                                        "{w}x{h}, event {eventus}, branch {ramus}, onus {onus}, kappa {kappa}, epoch {epoch_ms}, pixel {at}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// The annotation fits logically and after projection for all branches and languages at
    /// the checked scales, while preserving both symbols (REQ-L5-06). A degenerate surface omits it.
    #[test]
    fn aphorism_anchor_bounds() {
        for (w, h) in [(272, 180), (320, 180), (640, 360), (1920, 1080)] {
            let comp = compositio(w, h, 6);
            let mut buffer = std::vec![VOID;usize::from(w)*usize::from(h)];
            let mut tab = Tabula::new(w, h, &mut buffer).unwrap();
            for kappa in [-64, 0, 64] {
                let painter = Pictor::ground(&mut tab, kappa, comp.scene_h, 15);
                for lingua in [crate::hospes::Lingua::En, crate::hospes::Lingua::Pt] {
                    let f = Facultates {
                        width: w,
                        height: h,
                        lingua,
                        ..Facultates::default()
                    };
                    for onus in [32, 230] {
                        for epoch_ms in [NOON, NOON + 12 * 3_600_000] {
                            for eventus in 0..8 {
                                for ramus in 0..3 {
                                    let mut world = Mundus::genesis(7);
                                    world.verbum =
                                        world.verbum.with_onus(onus).with_curvatura(kappa);
                                    world.memoria = world
                                        .memoria
                                        .with_eventus(eventus)
                                        .with_ramus(ramus)
                                        .with_gradus(400);
                                    let measures = Indicia {
                                        epoch_ms,
                                        ..Indicia::default()
                                    };
                                    let regimen = Regimen::default();
                                    let chord = concentus(&world, &f, &measures, &regimen);
                                    let sky = sky_region(&comp, w, onus);
                                    let first =
                                        crate::signum::caelum::node(sky, eventus, ramus, 0, onus);
                                    let phrase =
                                        annales(world.memoria, lingua, false).aphorismus.unwrap();
                                    let unit = i32::from(comp.s);
                                    let nominal = (first.0 + 2 * unit, first.1 - 5 * unit);
                                    let width = (6 * phrase.chars().count() as i32 + 2) * unit;
                                    let obstacles =
                                        sky_obstacles(&comp, sky, &chord, &measures, &regimen);
                                    let anchor = aphorism_anchor(
                                        &painter,
                                        nominal,
                                        width,
                                        comp.s,
                                        comp.scene_h,
                                        obstacles,
                                    )
                                    .unwrap();
                                    let (left, right, top, bottom) = (
                                        anchor.0 - unit,
                                        anchor.0 - unit + width,
                                        anchor.1 - unit,
                                        anchor.1 + 11 * unit,
                                    );
                                    assert!(left >= 0 && right <= i32::from(w));
                                    assert!(top >= 0 && bottom <= i32::from(comp.scene_h));
                                    assert!(painter.contains_projection(
                                        left,
                                        top,
                                        width,
                                        12 * unit
                                    ));
                                    assert!(obstacles.iter().flatten().all(
                                        |&[sx, sy, ex, ey]| right <= sx
                                            || left >= ex
                                            || bottom <= sy
                                            || top >= ey
                                    ));
                                    if w == 320
                                        && eventus == 5
                                        && ramus == 0
                                        && onus == 32
                                        && kappa == 64
                                        && epoch_ms == NOON
                                        && lingua == crate::hospes::Lingua::Pt
                                    {
                                        assert!(
                                            anchor.0 < nominal.0,
                                            "the expanding right edge needs a horizontal correction"
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Equal vertical distances prefer the upper anchor, and surfaces too small for the
    /// padded annotation omit it instead of covering another sign (REQ-L5-06).
    #[test]
    fn aphorism_ties_and_degenerate_surface() {
        let mut buffer = [VOID; 80 * 80];
        let mut tab = Tabula::new(80, 80, &mut buffer).unwrap();
        let painter = Pictor::flat(&mut tab);
        assert_eq!(
            aphorism_anchor(&painter, (10, 10), 20, 1, 80, [Some([0, 15, 40, 17]), None]),
            Some((10, 4))
        );
        assert_eq!(
            aphorism_anchor(&painter, (21, 5), 20, 1, 20, [Some([20, 0, 40, 20]), None]),
            Some((1, 5))
        );
        assert_eq!(
            aphorism_anchor(&painter, (10, 10), 20, 1, 0, [None, None]),
            None
        );
        assert_eq!(
            aphorism_anchor(&painter, (10, 10), 81, 1, 80, [None, None]),
            None
        );
    }

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
                "sky" => {
                    let _ = compose_sky(
                        &mut pictor,
                        &comp,
                        comp.sky,
                        &chord,
                        &indicia,
                        &regimen,
                        &world,
                    );
                }
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
            compose_sky(&mut pictor, &comp, sky, &chord, &indicia, &regimen, &world);
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
        world.memoria = world.memoria.with_gradus(100);
        let mut empty = [0u8; 0];
        let mut tab = Tabula::new(cols, 0, &mut empty).unwrap();
        let mut grid = Cellae::new(cols, 0, &mut cells).unwrap();
        assert!(render_cellae(
            &world,
            &facultates,
            &indicia,
            &regimen,
            &LINES,
            &mut tab,
            &mut grid,
        ));
        assert!(
            cells.iter().all(|&c| c == 0xBEEF),
            "a grid without rows has no first row"
        );
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

    /// Writes public render samples of the graph's quadrants, masks and dream plinth for
    /// visual verification (REQ-L2-06, REQ-L5-06, REQ-L5-10).
    #[test]
    #[ignore = "writes files; run on request"]
    fn dump_graph_ppm() {
        use std::io::Write;
        let dir = std::env::var("SISYPHIUM_DUMP").unwrap_or_else(|_| "/tmp".into());
        for (eventus, onus, kappa, night) in [
            (0, 32, 0, false),
            (1, 32, 0, false),
            (2, 32, 0, false),
            (3, 32, 0, false),
            (0, 230, 0, false),
            (1, 230, 0, false),
            (2, 230, 0, false),
            (3, 32, -64, false),
            (1, 32, 0, true),
            (3, 32, 0, true),
            (3, 32, -64, true),
            (3, 32, 64, true),
            (5, 32, 64, false),
        ] {
            let suffix = if night { "-night" } else { "" };
            let name = std::format!("graph-{eventus}-onus-{onus}-kappa-{kappa}{suffix}");
            let branch = if eventus == 5 { 0 } else { 2 };
            let mut m = Mundus::genesis(7);
            m.memoria = m
                .memoria
                .with_eventus(eventus)
                .with_ramus(branch)
                .with_gradus(400);
            m.verbum = m
                .verbum
                .with_onus(onus)
                .with_curvatura(kappa)
                .with_ascensus(90);
            let f = Facultates {
                lingua: crate::hospes::Lingua::Pt,
                ..Facultates::default()
            };
            let i = Indicia {
                epoch_ms: if night { NOON + 12 * 3_600_000 } else { NOON },
                ..Indicia::default()
            };
            let r = Regimen::default();
            let mut buf = std::vec![0;320*180];
            m.render(&f, &i, &r, &mut Tabula::new(320, 180, &mut buf).unwrap());
            let palette = m.palette(&f, &i, &r);
            let mut out = b"P6\n320 180\n255\n".to_vec();
            for &p in &buf {
                out.extend_from_slice(&palette[usize::from(lum_of(p))]);
            }
            std::fs::File::create(std::format!("{dir}/{name}.ppm"))
                .unwrap()
                .write_all(&out)
                .unwrap();
            let sky = sky_region(&compositio(320, 180, 6), 320, onus);
            let first = crate::signum::caelum::node(sky, eventus, branch, 0, onus);
            let aphorism = annales(m.memoria, f.lingua, false).aphorismus.unwrap();
            std::println!(
                "{name}: node {first:?}, text right {}, aphorism {aphorism}",
                first.0 + 2 + 6 * aphorism.chars().count() as i32
            );
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
