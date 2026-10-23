//! The sky: the constellation scattered from the automaton, the hexagram of serenity, the moon
//! by its phase and the ray of a rare event (REQ-L5-07, REQ-L5-08, REQ-L6-03).

use crate::natura::mix;
use crate::sensus::quadrant_of;
use crate::signum::margo::harmony;
use crate::signum::pixel::LABEL_NONE;
use crate::signum::raster::{Modo, Pictor, bayer};
use crate::spatium::{Compositio, Rect, isqrt, part, scatter64};

/// Negative-space masks, row first, bit 15 at column 0 (REQ-L5-10): hourglass, eye and balance.
/// Each row is the sixteen-cell silhouette specified in `docs/03-especificacao.md` REQ-L5-10.
pub const MASKS: [[u16; 16]; 3] = [
    [
        0x3FFC, 0x3FFC, 0x1FF8, 0x0FF0, 0x07E0, 0x03C0, 0x0180, 0x0180, 0x0180, 0x0180, 0x03C0,
        0x07E0, 0x0FF0, 0x1FF8, 0x3FFC, 0x3FFC,
    ],
    [
        0x0000, 0x0000, 0x0000, 0x03C0, 0x1FF8, 0x7FFE, 0xFC3F, 0xF81F, 0xF81F, 0xFC3F, 0x7FFE,
        0x1FF8, 0x03C0, 0x0000, 0x0000, 0x0000,
    ],
    [
        0x0180, 0x7FFE, 0x4182, 0x4182, 0xA185, 0xA185, 0xE187, 0x0180, 0x0180, 0x0180, 0x0180,
        0x0180, 0x0180, 0x0180, 0x07E0, 0x0FF0,
    ],
];

/// Whether a mask cell is full; cells outside the sixteen-cell grid are empty (REQ-L5-10).
fn filled(mask: &[u16; 16], x: i32, y: i32) -> bool {
    if !(0..16).contains(&x) || !(0..16).contains(&y) {
        return false;
    }
    mask.get(y as usize)
        .is_some_and(|row| row & (0x8000u16 >> x as u32) != 0)
}

/// A contour cell is full and has an empty four-neighbour, including outside the grid
/// (REQ-L5-10).
fn contour(mask: &[u16; 16], x: i32, y: i32) -> bool {
    filled(mask, x, y)
        && [
            (x.wrapping_sub(1), y),
            (x.wrapping_add(1), y),
            (x, y.wrapping_sub(1)),
            (x, y.wrapping_add(1)),
        ]
        .into_iter()
        .any(|(a, b)| !filled(mask, a, b))
}

/// Snaps a point to the centre of the nearest contour cell in `square` by Manhattan distance;
/// ties choose the smallest row-major index (REQ-L5-10). A zero side becomes one; an empty
/// mask leaves the point in place.
#[must_use]
pub fn snap(p: (i32, i32), square: Rect, mask: &[u16; 16]) -> (i32, i32) {
    let side = i64::from(square.w.min(square.h).max(1));
    let u = (i64::from(p.0)
        .wrapping_sub(i64::from(square.x))
        .wrapping_mul(16)
        .checked_div(side)
        .unwrap_or(0))
    .clamp(0, 15) as i32;
    let v = (i64::from(p.1)
        .wrapping_sub(i64::from(square.y))
        .wrapping_mul(16)
        .checked_div(side)
        .unwrap_or(0))
    .clamp(0, 15) as i32;
    let mut best = u32::MAX;
    let mut result = p;
    for y in 0i32..16 {
        for x in 0i32..16 {
            if contour(mask, x, y) {
                let distance = x
                    .wrapping_sub(u)
                    .unsigned_abs()
                    .saturating_add(y.wrapping_sub(v).unsigned_abs());
                if distance < best {
                    best = distance;
                    result = (
                        i64::from(square.x).wrapping_add(
                            i64::from(x.wrapping_mul(2).wrapping_add(1)).wrapping_mul(side) / 32,
                        ) as i32,
                        i64::from(square.y).wrapping_add(
                            i64::from(y.wrapping_mul(2).wrapping_add(1)).wrapping_mul(side) / 32,
                        ) as i32,
                    );
                }
            }
        }
    }
    result
}

/// The event's quadrant, with the odd right and bottom remainders left outside its equal
/// halves (REQ-L6-04).
fn quadrant(sky: Rect, eventus: u8) -> Rect {
    let q = quadrant_of(eventus);
    Rect {
        x: sky
            .x
            .saturating_add(u16::from(q & 1).saturating_mul(sky.w / 2)),
        y: sky
            .y
            .saturating_add(u16::from(q >> 1).saturating_mul(sky.h / 2)),
        w: sky.w / 2,
        h: sky.h / 2,
    }
}

/// Node `i` scattered from the event, branch and line in the event quadrant (REQ-L5-06,
/// REQ-L6-04). Above burden 200 it snaps to the event mask in the centred square whose side
/// is max(1, min(quadrant width, quadrant height)) (REQ-L5-10).
#[must_use]
pub fn node(sky: Rect, eventus: u8, ramus: u8, i: u8, onus: u8) -> (i32, i32) {
    let q = quadrant(sky, eventus);
    let key = mix(u64::from(eventus) | (u64::from(ramus) << 8) | (u64::from(i) << 16));
    let p = (
        i32::from(q.x).wrapping_add((scatter64(key, 16).wrapping_mul(u64::from(q.w)) >> 16) as i32),
        i32::from(q.y)
            .wrapping_add((scatter64(key >> 32, 16).wrapping_mul(u64::from(q.h)) >> 16) as i32),
    );
    if onus <= 200 {
        return p;
    }
    let side = q.w.min(q.h).max(1);
    let square = Rect {
        x: q.x.saturating_add(q.w.saturating_sub(side) / 2),
        y: q.y.saturating_add(q.h.saturating_sub(side) / 2),
        w: side,
        h: side,
    };
    MASKS
        .get(usize::from(eventus % 3))
        .map_or(p, |mask| snap(p, square, mask))
}

/// Draws the graph in stratum 2 (REQ-L5-06): first node to trunk, then a chain for branch 0,
/// a bifurcation to node i−2 for branch 1, or a chain with two rotated half-length twigs for
/// branch 2. Edges precede the 2s-square nodes. Returns the first node, or none with no lines;
/// no more than the five chronicle lines are drawn.
#[allow(clippy::too_many_arguments)]
pub fn graph(
    pictor: &mut Pictor,
    sky: Rect,
    trunk: (i32, i32),
    eventus: u8,
    ramus: u8,
    shown: u8,
    onus: u8,
    s: u16,
    lum_edge: u8,
    lum_node: u8,
) -> Option<(i32, i32)> {
    let shown = shown.min(5);
    if shown == 0 {
        return None;
    }
    let scale = s.max(1);
    let thickness = scale.min(u16::from(u8::MAX)) as u8;
    let at = |i| node(sky, eventus, ramus, i, onus);
    for i in 0..shown {
        let p = at(i);
        let parent = if i == 0 {
            trunk
        } else if ramus == 1 {
            at(i.saturating_sub(2))
        } else {
            at(i.saturating_sub(1))
        };
        pictor.line(
            p.0,
            p.1,
            parent.0,
            parent.1,
            lum_edge,
            2,
            thickness,
            Modo::Super,
        );
        if ramus == 2 && i > 0 {
            let (dx, dy) = (p.0.wrapping_sub(parent.0), p.1.wrapping_sub(parent.1));
            for (x, y) in [
                (dx.wrapping_sub(dy) / 2, dx.wrapping_add(dy) / 2),
                (dx.wrapping_add(dy) / 2, dy.wrapping_sub(dx) / 2),
            ] {
                pictor.line(
                    p.0,
                    p.1,
                    p.0.wrapping_add(x),
                    p.1.wrapping_add(y),
                    lum_edge,
                    2,
                    thickness,
                    Modo::Super,
                );
            }
        }
    }
    let side = i32::from(scale).wrapping_mul(2);
    for i in 0..shown {
        let p = at(i);
        pictor.fill(p.0, p.1, side, side, lum_node, 2, false);
    }
    Some(at(0))
}

/// The region of the sky under the harmony of the burden (REQ-L5-12): the sky of the
/// composition while `onus` ≤ 200; above, its right edge is ⌊`width`·harmony(`onus`)⌋ in place of
/// ⌊`width`·φ⁻¹⌋, by the formula of the composition (REQ-L2-11).
#[must_use]
pub fn sky_region(comp: &Compositio, width: u16, onus: u8) -> Rect {
    if onus <= 200 {
        return comp.sky;
    }
    Rect {
        w: part(width, harmony(onus)).saturating_sub(comp.sky.x),
        ..comp.sky
    }
}

/// The six lines of the hexagram from the six bits of serenity, bit 0 first (the bottom line);
/// `true` is an unbroken line (REQ-L5-07).
#[must_use]
pub const fn hexagram_lines(tranquillitas: u8) -> [bool; 6] {
    let t = tranquillitas;
    [
        t & 1 != 0,
        t & 2 != 0,
        t & 4 != 0,
        t & 8 != 0,
        t & 16 != 0,
        t & 32 != 0,
    ]
}

/// Draws the hexagram with its top-left corner at `(x, y)`: six lines of 12·`s` pixels, two
/// scale units apart, the bottom line being bit 0; a broken line is two halves of 5·`s` with a
/// gap of 2·`s`.
pub fn hexagram(
    pictor: &mut Pictor,
    x: i32,
    y: i32,
    s: u16,
    tranquillitas: u8,
    lum: u8,
    stratum: u8,
) {
    let unit = i32::from(s.max(1));
    let lines = hexagram_lines(tranquillitas);
    for (k, unbroken) in (0i32..).zip(lines) {
        let row = y.wrapping_add(unit.wrapping_mul(10i32.wrapping_sub(k.wrapping_mul(2))));
        let right = x.wrapping_add(unit.wrapping_mul(12)).wrapping_sub(1);
        if unbroken {
            pictor.line(x, row, right, row, lum, stratum, s as u8, Modo::Super);
        } else {
            let gap_start = x.wrapping_add(unit.wrapping_mul(5)).wrapping_sub(1);
            let gap_end = x.wrapping_add(unit.wrapping_mul(7));
            pictor.line(x, row, gap_start, row, lum, stratum, s as u8, Modo::Super);
            pictor.line(gap_end, row, right, row, lum, stratum, s as u8, Modo::Super);
        }
    }
}

/// Scatters twelve stars over `sky` from the automaton: star j sits at the dispersion of
/// `mix(automatum | j << 8)`.
pub fn constellation(pictor: &mut Pictor, sky: Rect, automatum: u8, lum: u8, stratum: u8) {
    let (w, h) = (i64::from(sky.w), i64::from(sky.h));
    for j in 0u64..12 {
        let key = mix(u64::from(automatum) | (j << 8));
        let fx = i64::from(scatter64(key, 16) as u16);
        let fy = i64::from(scatter64(key >> 32, 16) as u16);
        let px = i64::from(sky.x).wrapping_add(fx.wrapping_mul(w).wrapping_shr(16));
        let py = i64::from(sky.y).wrapping_add(fy.wrapping_mul(h).wrapping_shr(16));
        if w > 0 && h > 0 {
            pictor.plot(px as i32, py as i32, lum, stratum, LABEL_NONE, Modo::Super);
        }
    }
}

/// Tone of the lit part of the moon where the ordered dither does not reach the moon's tone
/// (REQ-L5-08).
pub const PUNCTA_LUNAE: u8 = 9;

/// Draws the moon of radius `r` centered at `(cx, cy)` in the sky's class 2: nothing at new moon,
/// a full disc at 128, and in between the disc minus a shadow disc displaced by the illuminated
/// fraction, to the left while waxing and to the right while waning in the northern hemisphere,
/// mirrored in the southern one (`australis`). The lit part is dotted: a pixel takes `lum` where
/// the Bayer matrix lights it at `lum`, and [`PUNCTA_LUNAE`] elsewhere (REQ-L5-08). Only the rows
/// and columns that can reach the frame are visited.
pub fn moon(pictor: &mut Pictor, cx: i32, cy: i32, r: i32, luna: u8, lum: u8, australis: bool) {
    if luna == 0 {
        return;
    }
    let r = r.saturating_abs().min(4096);
    let waxing = luna <= 128;
    let fraction = if waxing {
        i32::from(luna)
    } else {
        i32::from(255u8.wrapping_sub(luna).wrapping_add(1))
    };
    let offset = r.wrapping_mul(2).wrapping_mul(fraction).wrapping_shr(7);
    let shadow_x = if waxing == australis {
        offset
    } else {
        offset.wrapping_neg()
    };
    let r2 = (r as u32).saturating_mul(r as u32);
    let (cx, cy, radius) = (i64::from(cx), i64::from(cy), i64::from(r));
    let (top, bottom) = pictor.clip_rows(cy.wrapping_sub(radius), cy.wrapping_add(radius));
    for y in top..=bottom {
        let dy = y.wrapping_sub(cy);
        let dy2 = dy.wrapping_mul(dy) as u32;
        let span = i64::from(isqrt(r2.saturating_sub(dy2)));
        let (left, right) = pictor.clip_columns(cx.wrapping_sub(span), cx.wrapping_add(span));
        for x in left..=right {
            let sx = x.wrapping_sub(cx).wrapping_sub(i64::from(shadow_x));
            let shadow2 = sx.wrapping_mul(sx).wrapping_add(dy.wrapping_mul(dy));
            if shadow2 > i64::from(r2) {
                let tone = if bayer(x as u32, y as u32, lum) {
                    lum
                } else {
                    PUNCTA_LUNAE
                };
                pictor.plot(x as i32, y as i32, tone, 2, LABEL_NONE, Modo::Super);
            }
        }
    }
}

/// The ray of a rare event: a line of the brightest tone from a point of the sky to the figure.
pub fn ray(pictor: &mut Pictor, from: (i32, i32), to: (i32, i32), lum: u8) {
    pictor.line(from.0, from.1, to.0, to.1, lum, 2, 1, Modo::Super);
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
    use crate::spatium::Rect;

    /// Each revealed line has one deterministic node in the event quadrant; bifurcation
    /// connects the third node to the first (REQ-L5-06, REQ-L6-04).
    #[test]
    fn sky_graph_nodes() {
        let sky = crate::spatium::compositio(320, 180, 6).sky;
        for eventus in 0..8 {
            let q = crate::sensus::quadrant_of(eventus);
            let (x, y) = (
                sky.x + u16::from(q & 1) * (sky.w / 2),
                sky.y + u16::from(q >> 1) * (sky.h / 2),
            );
            for ramus in 0..3 {
                for i in 0..5 {
                    let p = node(sky, eventus, ramus, i, 0);
                    assert_eq!(p, node(sky, eventus, ramus, i, 200));
                    assert!((i32::from(x)..i32::from(x + sky.w / 2)).contains(&p.0));
                    assert!((i32::from(y)..i32::from(y + sky.h / 2)).contains(&p.1));
                }
                for shown in 0..=5 {
                    let mut buf = [VOID; 320 * 180];
                    let mut tab = Tabula::new(320, 180, &mut buf).unwrap();
                    let result = graph(
                        &mut Pictor::flat(&mut tab),
                        sky,
                        (16, 34),
                        eventus,
                        ramus,
                        shown,
                        0,
                        1,
                        9,
                        14,
                    );
                    assert_eq!(result, (shown > 0).then(|| node(sky, eventus, ramus, 0, 0)));
                    if shown == 0 {
                        assert!(buf.iter().all(|&p| p == VOID));
                    }
                    for (at, _) in buf.iter().enumerate().filter(|(_, p)| lum_of(**p) == 14) {
                        let p = ((at % 320) as i32, (at / 320) as i32);
                        assert!((0..shown).any(|i| {
                            let position = node(sky, eventus, ramus, i, 0);
                            (p.0 - position.0).unsigned_abs() <= 1
                                && (p.1 - position.1).unsigned_abs() <= 1
                        }));
                    }
                }
            }
        }
        assert_eq!(node(sky, 0, 2, 0, 0), (59, 16));
        for ramus in 0..3 {
            for shown in 1..=5 {
                let mut actual = [VOID; 320 * 180];
                let mut expected = actual;
                let mut tab = Tabula::new(320, 180, &mut actual).unwrap();
                let _ = graph(
                    &mut Pictor::flat(&mut tab),
                    sky,
                    (16, 34),
                    3,
                    ramus,
                    shown,
                    0,
                    1,
                    9,
                    14,
                );
                let mut tab = Tabula::new(320, 180, &mut expected).unwrap();
                let mut painter = Pictor::flat(&mut tab);
                let nodes: std::vec::Vec<_> =
                    (0..shown).map(|i| node(sky, 3, ramus, i, 0)).collect();
                for (i, &point) in nodes.iter().enumerate() {
                    let parent = if i == 0 {
                        (16, 34)
                    } else {
                        nodes[if ramus == 1 {
                            i.saturating_sub(2)
                        } else {
                            i - 1
                        }]
                    };
                    painter.line(point.0, point.1, parent.0, parent.1, 9, 2, 1, Modo::Super);
                    if ramus == 2 && i > 0 {
                        let dx = point.0 - parent.0;
                        let dy = point.1 - parent.1;
                        for end in [
                            (point.0 + (dx - dy) / 2, point.1 + (dx + dy) / 2),
                            (point.0 + (dx + dy) / 2, point.1 + (dy - dx) / 2),
                        ] {
                            painter.line(point.0, point.1, end.0, end.1, 9, 2, 1, Modo::Super);
                        }
                    }
                }
                for (x, y) in nodes {
                    painter.fill(x, y, 2, 2, 14, 2, false);
                }
                assert_eq!(actual, expected, "branch {ramus}, shown {shown}");
            }
        }
    }

    /// The sixteen rows of each silhouette match the canonical mask specification
    /// (REQ-L5-10), with the most significant bit at the left edge.
    #[test]
    fn mask_rows() {
        let rows = [
            "3FFC 3FFC 1FF8 0FF0 07E0 03C0 0180 0180 0180 0180 03C0 07E0 0FF0 1FF8 3FFC 3FFC",
            "0000 0000 0000 03C0 1FF8 7FFE FC3F F81F F81F FC3F 7FFE 1FF8 03C0 0000 0000 0000",
            "0180 7FFE 4182 4182 A185 A185 E187 0180 0180 0180 0180 0180 0180 0180 07E0 0FF0",
        ];
        for (mask, text) in MASKS.into_iter().zip(rows) {
            let expected: std::vec::Vec<_> = text
                .split_whitespace()
                .map(|row| u16::from_str_radix(row, 16).unwrap())
                .collect();
            assert_eq!(mask.as_slice(), expected);
        }
    }

    /// A heavy node snaps to the nearest four-neighbour contour cell, ties in row order;
    /// a contour centre stays fixed (REQ-L5-10).
    #[test]
    fn negative_space_mask_snap() {
        let square = Rect {
            x: 16,
            y: 8,
            w: 160,
            h: 160,
        };
        for mask in MASKS {
            let filled = |x: i32, y: i32| {
                (0..16).contains(&x)
                    && (0..16).contains(&y)
                    && mask[y as usize] & (1 << (15 - x)) != 0
            };
            let boundary = |x: i32, y: i32| {
                filled(x, y)
                    && [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
                        .into_iter()
                        .any(|(a, b)| !filled(a, b))
            };
            for i in 0..64 {
                let key = mix(i);
                let p = ((key & 255) as i32 - 20, ((key >> 32) & 255) as i32 - 20);
                let u = ((p.0 - 16) * 16 / 160).clamp(0, 15);
                let v = ((p.1 - 8) * 16 / 160).clamp(0, 15);
                let best = (0..256)
                    .filter(|k| boundary(k % 16, k / 16))
                    .min_by_key(|k| {
                        (
                            (k % 16 - u).unsigned_abs() + (k / 16 - v).unsigned_abs(),
                            *k,
                        )
                    })
                    .unwrap();
                let expected = (
                    16 + (2 * (best % 16) + 1) * 5,
                    8 + (2 * (best / 16) + 1) * 5,
                );
                assert_eq!(snap(p, square, &mask), expected);
                assert_eq!(snap(expected, square, &mask), expected);
            }
        }
        assert_eq!(MASKS[0][0], 0x3FFC);
        assert_eq!(MASKS[1][7], 0xF81F);
        assert_eq!(MASKS[2][15], 0x0FF0);
        let sky = Rect {
            x: 16,
            y: 8,
            w: 640,
            h: 640,
        };
        for eventus in 0..8 {
            let q = crate::sensus::quadrant_of(eventus);
            let square = Rect {
                x: 16 + u16::from(q & 1) * 320,
                y: 8 + u16::from(q >> 1) * 320,
                w: 320,
                h: 320,
            };
            for i in 0..5 {
                assert_eq!(
                    node(sky, eventus, 2, i, 201),
                    snap(
                        node(sky, eventus, 2, i, 200),
                        square,
                        &MASKS[usize::from(eventus % 3)]
                    )
                );
            }
        }
    }

    /// Six lines from the six bits, bit 0 at the bottom; 1 = unbroken (REQ-L5-07).
    #[test]
    fn hexagram_lines() {
        assert_eq!(
            super::hexagram_lines(0b10_1010),
            [false, true, false, true, false, true]
        );
        assert_eq!(super::hexagram_lines(0b11_1111), [true; 6]);
        assert_eq!(super::hexagram_lines(0), [false; 6]);
        assert_eq!(
            super::hexagram_lines(0b100_0001),
            [true, false, false, false, false, false],
            "bit 6 ignored"
        );
        let mut buf = [VOID; 16 * 16];
        let mut tab = Tabula::new(16, 16, &mut buf).unwrap();
        let mut pictor = Pictor::flat(&mut tab);
        hexagram(&mut pictor, 2, 2, 1, 0b00_0001, 9, 2);
        let row = |y: usize| (0..16).filter(|&x| lum_of(buf[x + y * 16]) == 9).count();
        assert_eq!(row(12), 12, "bottom line unbroken over 12 px");
        assert_eq!(row(10), 10, "second line broken: a gap of 2 px");
    }

    /// The harmony of the burden shapes the sky (REQ-L5-12): up to 200 the region of the
    /// composition; above, its right edge is ⌊W·(φ⁻¹ ⊕ 0x1337)⌋ by the composition's formula.
    #[test]
    fn harmony_shapes_sky() {
        use crate::signum::margo::harmony;
        use crate::spatium::compositio;
        for (w, h) in [(320u16, 180u16), (1920, 1080), (272, 60)] {
            let comp = compositio(w, h, 6);
            assert_eq!(sky_region(&comp, w, 0), comp.sky);
            assert_eq!(sky_region(&comp, w, 200), comp.sky);
            let corrupted = sky_region(&comp, w, 201);
            let right = ((u64::from(w) * harmony(201).0 as u64) >> 16) as u16;
            assert_eq!(
                corrupted,
                Rect {
                    w: right.saturating_sub(comp.sky.x),
                    ..comp.sky
                }
            );
            assert_eq!(sky_region(&comp, w, 255), corrupted);
            if w >= 320 {
                assert_ne!(corrupted.w, comp.sky.w, "{w}×{h}");
            }
        }
    }

    /// The same automaton scatters the same constellation; a different one differs.
    #[test]
    fn constellation_deterministic() {
        let sky = Rect {
            x: 4,
            y: 4,
            w: 56,
            h: 24,
        };
        let render = |automatum: u8| {
            let mut buf = [VOID; 64 * 32];
            let mut tab = Tabula::new(64, 32, &mut buf).unwrap();
            let mut pictor = Pictor::flat(&mut tab);
            constellation(&mut pictor, sky, automatum, 7, 1);
            buf
        };
        assert_eq!(render(0x10), render(0x10));
        assert_ne!(render(0x10), render(0x11));
        let buf = render(0x10);
        let lit = (0..64 * 32).filter(|&i| lum_of(buf[i]) == 7).count();
        assert!(lit >= 12 && lit <= 12 * 5, "lit {lit}");
        for i in 0..64 * 32 {
            if lum_of(buf[i]) == 7 {
                let (x, y) = (i % 64, i / 64);
                assert!(
                    (4..60).contains(&x) && (4..28).contains(&y),
                    "outside the sky at {x},{y}"
                );
            }
        }
    }

    /// New moon draws nothing; full moon fills the disc; half moon lights about half; the lit
    /// part is dotted, the cells of the dither in the moon's tone and the others in tone 9
    /// (REQ-L5-08).
    #[test]
    fn moon_phase_fill() {
        let render = |luna: u8| {
            let mut buf = [VOID; 32 * 32];
            let mut tab = Tabula::new(32, 32, &mut buf).unwrap();
            let mut pictor = Pictor::flat(&mut tab);
            moon(&mut pictor, 16, 16, 8, luna, 12, false);
            let tone = |t: u8| (0..32 * 32).filter(|&i| lum_of(buf[i]) == t).count();
            (tone(12), tone(PUNCTA_LUNAE))
        };
        assert_eq!(render(0), (0, 0));
        let (bright, dim) = render(128);
        let full = bright + dim;
        assert!((160..=230).contains(&full), "full {full}");
        assert!(dim > 0, "the lit part is dotted");
        assert!(
            bright * 16 >= full * 10 && bright * 16 <= full * 14,
            "twelve in sixteen cells bright: {bright} of {full}"
        );
        let (b, d) = render(64);
        assert!(
            b + d > full / 4 && b + d < full * 3 / 4,
            "half {} of {full}",
            b + d
        );
        let (b, d) = render(192);
        assert!(
            b + d > full / 4 && b + d < full * 3 / 4,
            "waning {} of {full}",
            b + d
        );
    }

    /// The moon of the southern hemisphere is the mirror image of the northern one about its
    /// vertical axis: the waxing moon is lit on the right in the north and on the left in the
    /// south (REQ-L5-08).
    #[test]
    fn moon_hemisphere_mirror() {
        let lit = |luna: u8, australis: bool| {
            let mut buf = [VOID; 33 * 33];
            let mut tab = Tabula::new(33, 33, &mut buf).unwrap();
            let mut pictor = Pictor::flat(&mut tab);
            moon(&mut pictor, 16, 16, 10, luna, 12, australis);
            buf.map(|p| p != VOID)
        };
        for luna in [20u8, 64, 100, 160, 200, 240] {
            let (north, south) = (lit(luna, false), lit(luna, true));
            for y in 0..33 {
                for x in 0..33 {
                    assert_eq!(
                        north[x + y * 33],
                        south[32 - x + y * 33],
                        "luna {luna} at {x},{y}"
                    );
                }
            }
            assert_ne!(north, south, "luna {luna}");
        }
        let side = |m: &[bool; 33 * 33], right: bool| {
            (0..33 * 33)
                .filter(|&i| m[i] && if right { i % 33 > 16 } else { i % 33 < 16 })
                .count()
        };
        let (north, south) = (lit(64, false), lit(64, true));
        assert!(
            side(&north, true) > side(&north, false),
            "waxing north lit on the right"
        );
        assert!(
            side(&south, false) > side(&south, true),
            "waxing south lit on the left"
        );
    }
}
