//! Geometric signs of prophecy, recurrence and affirmation (REQ-L2-10, REQ-L6-03, REQ-L7-02).

use crate::natura::{henon, mix};
use crate::signum::raster::{Modo, Pictor};
use crate::spatium::{Fx, Rect, cos, penrose, sin};

/// Thirty-two equal angular intervals divide the u16 turn (REQ-L7-02).
const ARC_STEP: u16 = 2048;
/// The open circle omits four of the thirty-two intervals (REQ-L7-02).
const ARC_SEGMENTS: u16 = 28;

/// A point on the circle in integer pixels, using Q16.16 trigonometry (REQ-L7-02).
fn circle_point(center: (i32, i32), radius: i32, angle: u16) -> (i32, i32) {
    let radius = i64::from(radius.max(0));
    (
        center
            .0
            .saturating_add((radius.wrapping_mul(i64::from(cos(angle).0)) >> 16) as i32),
        center
            .1
            .saturating_add((radius.wrapping_mul(i64::from(sin(angle).0)) >> 16) as i32),
    )
}

/// Draws twenty-eight segments of an open circle, thinning its last four to `s` pixels
/// (REQ-L7-02). The sign belongs to stratum one.
#[allow(clippy::too_many_arguments)]
pub fn enso(
    pictor: &mut Pictor,
    center: (i32, i32),
    r: i32,
    start: u16,
    thick: u8,
    s: u16,
    lum: u8,
) {
    for segment in 0..ARC_SEGMENTS {
        let angle = start.wrapping_add(segment.wrapping_mul(ARC_STEP));
        let from = circle_point(center, r, angle);
        let to = circle_point(center, r, angle.wrapping_add(ARC_STEP));
        let width = if segment >= ARC_SEGMENTS.wrapping_sub(4) {
            s.min(255) as u8
        } else {
            thick
        };
        pictor.line(from.0, from.1, to.0, to.1, lum, 1, width, Modo::Super);
    }
}

/// The center of quadrant `q & 3` over the complete frame (REQ-L7-02).
#[must_use]
pub const fn quadrant_center(width: u16, height: u16, q: u8) -> (i32, i32) {
    (
        (width as i32).wrapping_mul(1i32.wrapping_add(2i32.wrapping_mul((q & 1) as i32))) / 4,
        (height as i32).wrapping_mul(1i32.wrapping_add(2i32.wrapping_mul(((q >> 1) & 1) as i32)))
            / 4,
    )
}

/// Joins the four flights of the closed impossible staircase with cyclic class dominance
/// and flight labels modulo three (REQ-L2-08, REQ-L2-10). Span follows `penrose`'s domain.
pub fn staircase(pictor: &mut Pictor, center: (i32, i32), span: u32, k: i32, lum: u8) {
    let span = span.clamp(1, 16_384);
    for step in 0..span.wrapping_mul(4) {
        let (x, y, class) = penrose(step, 0, 0, span);
        let (nx, ny, _) = penrose(step.wrapping_add(1), 0, 0, span);
        pictor.edge(
            center.0.saturating_add(x.saturating_mul(k)),
            center.1.saturating_add(y.saturating_mul(k)),
            center.0.saturating_add(nx.saturating_mul(k)),
            center.1.saturating_add(ny.saturating_mul(k)),
            lum,
            class,
            (step.checked_div(span).unwrap_or(0) % 3) as u8,
            Modo::Dominans,
        );
    }
}

/// The deterministic Hénon attractor point after 32..=63 iterations, mapped from
/// x in −1.5..=1.5 and y in −0.5..=0.5 into the sky and clipped (REQ-L6-03).
/// An empty dimension maps to its origin.
#[must_use]
pub fn henon_point(window: i64, sky: Rect) -> (i32, i32) {
    let (mut x, mut y) = (Fx::ZERO, Fx::ZERO);
    for _ in 0..32u64.wrapping_add(mix(window as u64) & 31) {
        (x, y) = henon(x, y);
    }
    let width = i64::from(sky.w.saturating_sub(1));
    let height = i64::from(sky.h.saturating_sub(1));
    (
        i32::from(sky.x).saturating_add(
            (i64::from(x.0).saturating_add(98_304).saturating_mul(width) / 196_608).clamp(0, width)
                as i32,
        ),
        i32::from(sky.y).saturating_add(
            (i64::from(y.0).saturating_add(32_768).saturating_mul(height) / 65_536).clamp(0, height)
                as i32,
        ),
    )
}

/// Four dispersed points joined by a seam of light on the affirmed boulder
/// (`docs/05-estetica.md` §6). Each key byte sets a vertical displacement within the radius.
pub fn seam(pictor: &mut Pictor, center: (i32, i32), r: i32, key: u64, s: u16, lum: u8) {
    let r = r.max(0);
    let offsets = [
        r.saturating_neg(),
        r.wrapping_div(3).saturating_neg(),
        r.wrapping_div(3),
        r,
    ];
    let mut previous = None;
    for (j, dx) in (0u32..4).zip(offsets) {
        let byte = (key.wrapping_shr(j.wrapping_mul(8)) & 255) as i32;
        let dy = byte
            .checked_rem(r.wrapping_div(2).saturating_add(1))
            .unwrap_or(0)
            .saturating_sub(r.wrapping_div(4));
        let point = (center.0.saturating_add(dx), center.1.saturating_add(dy));
        if let Some((x, y)) = previous {
            pictor.line(
                x,
                y,
                point.0,
                point.1,
                lum,
                1,
                s.min(255) as u8,
                Modo::Super,
            );
        }
        previous = Some(point);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hospes::Tabula;
    use crate::signum::pixel::{VOID, label_of, lum_of, stratum_of};

    /// The arc constants divide a full u16 turn into thirty-two pieces with four omitted.
    #[test]
    fn law_constants() {
        assert_eq!(u32::from(ARC_STEP) * 32, 65_536);
        assert_eq!(ARC_SEGMENTS, 32 - 4);
    }

    /// Each successive byte places one seam vertex, including negative vertical offsets.
    #[test]
    fn seam_key_vertices() {
        let mut pixels = [VOID; 80 * 80];
        let key = 0x00FF_8040u64;
        seam(
            &mut Pictor::flat(&mut Tabula::new(80, 80, &mut pixels).unwrap()),
            (40, 40),
            12,
            key,
            1,
            15,
        );
        for (j, x) in [28, 36, 44, 52].into_iter().enumerate() {
            let y = 40 + ((key >> (8 * j)) & 255) as usize % 7 - 3;
            assert_eq!(lum_of(pixels[y * 80 + x]), 15);
        }
        assert!(pixels.iter().filter(|&&p| lum_of(p) == 15).count() >= 25);
    }

    /// Twenty-eight segments cover seven eighths of the circle; its final eighth stays open.
    #[test]
    fn enso_open_circle() {
        let mut pixels = [VOID; 200 * 200];
        let mut tab = Tabula::new(200, 200, &mut pixels).unwrap();
        enso(&mut Pictor::flat(&mut tab), (100, 100), 40, 0, 1, 1, 14);
        let mut reached = 0;
        for sample in 0..64 {
            let angle = f64::from(sample) * core::f64::consts::TAU / 64.0;
            let (x, y) = (100.0 + 40.0 * angle.cos(), 100.0 + 40.0 * angle.sin());
            let found = (-1..=1).any(|dy| {
                (-1..=1).any(|dx| {
                    lum_of(
                        pixels[(y.round() as i32 + dy) as usize * 200
                            + (x.round() as i32 + dx) as usize],
                    ) == 14
                })
            });
            reached += usize::from(found);
            if (58..=61).contains(&sample) {
                assert!(!found, "gap sample {sample}");
            }
        }
        assert!(reached * 100 >= 64 * 85, "{reached} samples");
    }

    /// The last flight cannot replace the first and all three labels remain represented.
    #[test]
    fn impossibilia_dominance() {
        for kappa in [0, 64] {
            let mut pixels = [VOID; 100 * 100];
            let mut first = pixels;
            let mut last = pixels;
            staircase(
                &mut Pictor::new(&mut Tabula::new(100, 100, &mut pixels).unwrap(), kappa),
                (50, 50),
                6,
                2,
                11,
            );
            for (buffer, range) in [(&mut first, 0..6), (&mut last, 18..24)] {
                let mut tab = Tabula::new(100, 100, buffer).unwrap();
                let mut painter = Pictor::new(&mut tab, kappa);
                for t in range {
                    let (x, y, _) = penrose(t, 0, 0, 6);
                    let (nx, ny, _) = penrose(t + 1, 0, 0, 6);
                    painter.edge(
                        50 + x * 2,
                        50 + y * 2,
                        50 + nx * 2,
                        50 + ny * 2,
                        11,
                        0,
                        0,
                        Modo::Super,
                    );
                }
            }
            let mut intersections = 0;
            for (i, &p) in pixels.iter().enumerate() {
                if first[i] != VOID && last[i] != VOID {
                    intersections += 1;
                    assert_eq!(stratum_of(p), 0, "intersection {i}");
                }
            }
            assert!(intersections > 0);
            for label in 0..3 {
                assert!(
                    pixels
                        .iter()
                        .any(|&p| lum_of(p) > 0 && label_of(p) == label)
                );
            }
        }
    }

    /// Hénon points remain within nonempty skies, including degenerate dimensions.
    #[test]
    fn henon_point_within_sky() {
        for sky in [
            Rect {
                x: 10,
                y: 20,
                w: 100,
                h: 40,
            },
            Rect {
                x: 5,
                y: 9,
                w: 1,
                h: 1,
            },
            Rect::default(),
        ] {
            for window in [i64::MIN, -1, 0, 1, 42, i64::MAX] {
                let (x, y) = henon_point(window, sky);
                assert!(
                    (i32::from(sky.x)..i32::from(sky.x) + i32::from(sky.w.max(1))).contains(&x)
                );
                assert!(
                    (i32::from(sky.y)..i32::from(sky.y) + i32::from(sky.h.max(1))).contains(&y)
                );
                assert_eq!((x, y), henon_point(window, sky));
            }
        }
    }

    /// Quadrants use the full surface; their two-bit index wraps.
    #[test]
    fn quadrant_centers() {
        assert_eq!(
            (0..4)
                .map(|q| quadrant_center(320, 180, q))
                .collect::<std::vec::Vec<_>>(),
            [(80, 45), (240, 45), (80, 135), (240, 135)]
        );
        assert_eq!(quadrant_center(320, 180, 255), (240, 135));
    }
}
