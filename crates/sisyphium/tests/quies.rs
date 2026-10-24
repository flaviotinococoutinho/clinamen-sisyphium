//! The mountain remains visible in the quiet two-tone world (REQ-L5-11).

use sisyphium::hospes::{Facultates, Indicia, Regimen, Stilus, Tabula};
use sisyphium::mundus::Mundus;
use sisyphium::signum::pixel::{lum_of, stratum_of};
use sisyphium::spatium::compositio;
use sisyphium::verbum::Modus;

/// The canonical Quies frame retains dithered ground at both verification sizes. The crop
/// spans 3W/4..9W/10, below the sky and above the text, from S minus one figure height to
/// S minus eight scale units. The actors remain at the slope's foot, left of the crop;
/// the frame stays near the side margins. Classes 0 and 1 cannot enter the crop, so a
/// frame or actor cannot satisfy the visibility assertion. Genesis has no ruins or persona.
#[test]
fn quies_preserves_dithered_slope() {
    let mut world = Mundus::genesis(4);
    world.verbum = world.verbum.with_modus(Modus::Quies);
    let indicia = Indicia {
        epoch_ms: 1_774_008_000_000,
        energy: 20,
        ..Indicia::default()
    };
    let regimen = Regimen {
        stilus: Stilus::Bicolor,
        strata: 1,
        tick_ms: 60_000,
        interval_ms: 60_000,
        ..Regimen::default()
    };
    let mut measurements = Vec::new();
    for (width, height) in [(320u16, 180u16), (1920, 1080)] {
        let facultates = Facultates {
            width,
            height,
            ..Facultates::default()
        };
        let comp = compositio(width, height, regimen.lines);
        let left = width * 3 / 4;
        let right = width * 9 / 10;
        let top = comp.scene_h - comp.figure_h;
        let bottom = comp.scene_h - 8 * comp.s;
        assert!(i32::from(left) > comp.slope.0.x);
        assert!(left > comp.sky.x + comp.sky.w);
        assert!(right < width - comp.m);
        assert!(top > comp.sky.y + comp.sky.h);
        assert!(top < bottom && bottom < comp.annales.y);
        let mut pixels = vec![0; usize::from(width) * usize::from(height)];
        let mut tab = Tabula {
            width,
            height,
            pixels: &mut pixels,
        };
        world.render(&facultates, &indicia, &regimen, &mut tab);
        assert!(pixels.iter().all(|&p| matches!(lum_of(p), 0 | 15)));
        let mut white = 0;
        let mut black = 0;
        for y in top..bottom {
            for x in left..right {
                let p = pixels[usize::from(y) * usize::from(width) + usize::from(x)];
                assert!(matches!(stratum_of(p), 2 | 3), "actor or frame in crop");
                if lum_of(p) == 15 {
                    assert_eq!(stratum_of(p), 2, "white pixels belong to the slope");
                    white += 1;
                } else {
                    black += 1;
                }
            }
        }
        measurements.push((width, height, left, top, right, bottom, white, black));
    }
    eprintln!(
        "slope crop (width, height, left, top, right, bottom, white, black): {measurements:?}"
    );
    assert!(
        measurements
            .iter()
            .all(|&(_, _, _, _, _, _, white, black)| white > 0 && black > 0),
        "dithered slope (width, height, left, top, right, bottom, white, black): {measurements:?}"
    );
}
