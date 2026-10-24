//! Public golden frames and measurements for checkpoint 3 (REQ-L5-09, REQ-H-04..06).

#![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]

use sisyphium::hospes::{Facultates, Indicia, Regimen, Stilus, Tabula};
use sisyphium::mundus::Mundus;
use sisyphium::signum::pixel::lum_of;
use sisyphium::signum::scaena::fnv1a;
use sisyphium::verbum::Modus;

/// Noon on 2026-06-21 at UTC−03:00.
const NOON: i64 = 1_782_054_000_000;

/// Encoded-raster hashes recorded after visual inspection (REQ-L5-09).
/// Each name and position matches `golden_scenarios`.
const GOLDEN: [(&str, u64); 12] = [
    ("vigilia", 0x8C5A_591A_9B47_86D3),
    ("acedia", 0xD036_E23A_C852_49B7),
    ("somnium", 0x2CDF_2A67_C888_8772),
    ("quies", 0x4FE9_C3D1_D1A4_F542),
    ("littera", 0x431A_6AE3_4C5F_D2AF),
    ("somnium-1920x1080", 0x0BFA_3267_A8D9_75D5),
    ("graph-linear", 0xA28C_7853_445C_10C8),
    ("graph-bifurcation", 0x701F_6601_6DF9_21F6),
    ("graph-fractal", 0x82E0_640C_F2A2_791E),
    ("graph-mask", 0x82FA_FEAD_DC4E_2D53),
    ("night", 0x8722_6D14_ACB3_C0D9),
    ("agnitio", 0x6BF1_5185_B62D_1F74),
];

/// One complete public rendering fixture.
#[derive(Clone, Copy)]
struct Scaena {
    /// Stable artifact name.
    name: &'static str,
    /// Pixel width.
    width: u16,
    /// Pixel height.
    height: u16,
    /// Exact world, evolved only where the scenario requires it.
    world: Mundus,
    /// Measurements at the final epoch.
    indicia: Indicia,
    /// Explicit presentation policy, without a consilium search.
    regimen: Regimen,
}

/// Builds a 320×180 fixture; larger variants replace dimensions explicitly.
fn scene(name: &'static str, world: Mundus, indicia: Indicia, regimen: Regimen) -> Scaena {
    Scaena {
        name,
        width: 320,
        height: 180,
        world,
        indicia,
        regimen,
    }
}

/// The twelve public rendering fixtures of `docs/08-verificacao.md` §7.
#[allow(clippy::too_many_lines)]
fn golden_scenarios() -> [Scaena; 12] {
    let f = Facultates::default();
    let base = Regimen::default();
    let day = Indicia {
        epoch_ms: NOON,
        energy: 255,
        seism: 0,
        ..Indicia::default()
    };
    let evening = Indicia {
        epoch_ms: 1_790_112_600_000,
        energy: 120,
        ..Indicia::default()
    };
    let night = Indicia {
        epoch_ms: 1_797_831_000_000,
        energy: 200,
        idle_s: 3600,
        ..Indicia::default()
    };
    let mut vigil = Mundus::genesis(1);
    let _ = vigil.advance(&f, &day, 4096, 42);
    let mut acedia = Mundus::genesis(2);
    let _ = acedia.advance(&f, &evening, 4096, 125);
    acedia.verbum = acedia.verbum.with_onus(160);
    let mut dream = Mundus::genesis(3);
    dream.verbum = dream.verbum.with_modus(Modus::Somnium).with_curvatura(-48);
    let mut quies = Mundus::genesis(4);
    quies.verbum = quies.verbum.with_modus(Modus::Quies);
    let mut graph = Mundus::genesis(1);
    graph.memoria = graph.memoria.with_eventus(3).with_ramus(0).with_gradus(400);
    let mut graph_one = graph;
    graph_one.memoria = graph_one.memoria.with_ramus(1);
    let mut graph_two = graph;
    graph_two.memoria = graph_two.memoria.with_ramus(2);
    let mut masked = graph_two;
    masked.verbum = masked.verbum.with_onus(230);
    let mut moon = Mundus::genesis(5);
    moon.verbum = moon.verbum.with_ascensus(60);
    let mut recognized = Mundus::genesis(6);
    recognized.memoria = recognized
        .memoria
        .with_eventus(12)
        .with_ramus(0)
        .with_gradus(100);
    [
        scene("vigilia", vigil, day, base),
        scene(
            "acedia",
            acedia,
            evening,
            Regimen {
                stilus: Stilus::Musivum,
                strata: 2,
                tick_ms: 125,
                interval_ms: 125,
                ..base
            },
        ),
        scene("somnium", dream, night, base),
        scene(
            "quies",
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
                ..base
            },
        ),
        scene(
            "littera",
            vigil,
            day,
            Regimen {
                stilus: Stilus::Littera,
                ..base
            },
        ),
        Scaena {
            width: 1920,
            height: 1080,
            ..scene("somnium-1920x1080", dream, night, base)
        },
        scene("graph-linear", graph, day, base),
        scene("graph-bifurcation", graph_one, day, base),
        scene("graph-fractal", graph_two, day, base),
        scene("graph-mask", masked, day, base),
        scene(
            "night",
            moon,
            Indicia {
                epoch_ms: 1_798_090_200_000,
                energy: 200,
                ..Indicia::default()
            },
            base,
        ),
        scene("agnitio", recognized, day, base),
    ]
}

/// Captures the encoded raster and the public palette for the exact same fixture.
fn capture(scene: Scaena) -> (Vec<u8>, [[u8; 3]; 16]) {
    let f = Facultates {
        width: scene.width,
        height: scene.height,
        ..Facultates::default()
    };
    let mut pixels = vec![0; usize::from(scene.width) * usize::from(scene.height)];
    let mut tab = Tabula {
        width: scene.width,
        height: scene.height,
        pixels: &mut pixels,
    };
    scene
        .world
        .render(&f, &scene.indicia, &scene.regimen, &mut tab);
    let palette = scene.world.palette(&f, &scene.indicia, &scene.regimen);
    (pixels, palette)
}

/// Checks a named subset of the fixed twelve-frame table through the public world API.
fn assert_goldens(indices: &[usize]) {
    let scenarios = golden_scenarios();
    assert_eq!(scenarios.len(), GOLDEN.len());
    for &index in indices {
        let scenario = scenarios[index];
        let (expected_name, expected_hash) = GOLDEN[index];
        assert_eq!(scenario.name, expected_name, "fixture order at {index}");
        assert_ne!(
            expected_hash, 0,
            "missing inspected hash for {expected_name}"
        );
        let (pixels, _) = capture(scenario);
        let actual_hash = fnv1a(&pixels);
        assert_eq!(
            actual_hash, expected_hash,
            "{expected_name}: actual {actual_hash:#018x}"
        );
    }
}

/// The six canonical verification frames, including Littera and the larger dream (REQ-L5-09).
#[test]
fn golden_frames() {
    assert_goldens(&[0, 1, 2, 3, 4, 5]);
}

/// The three graph branches and the burden mask use the world's own text (REQ-L5-06/10).
#[test]
fn sky_graph_golden() {
    assert_goldens(&[6, 7, 8, 9]);
}

/// The specified December night renders its lunar phase through the public API (REQ-L5-08).
#[test]
fn night_golden() {
    assert_goldens(&[10]);
}

/// Recognition renders both revealed lines and the present hexadecimal word (REQ-L8-02).
#[test]
fn agnitio_golden() {
    assert_goldens(&[11]);
}

/// Writes all twelve frames and their encoded-raster hashes to the integration-test directory.
/// This export does not approve or overwrite the expected hashes.
#[test]
#[ignore = "writes files; run for visual inspection"]
fn dump_golden_ppm() -> std::io::Result<()> {
    use std::io::Write;
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("aurea-cp3");
    std::fs::create_dir_all(&dir)?;
    for scenario in golden_scenarios() {
        let (pixels, palette) = capture(scenario);
        let mut bytes = format!("P6\n{} {}\n255\n", scenario.width, scenario.height).into_bytes();
        for &pixel in &pixels {
            bytes.extend_from_slice(&palette[usize::from(lum_of(pixel))]);
        }
        let path = dir.join(format!("{}.ppm", scenario.name));
        std::fs::File::create(&path)?.write_all(&bytes)?;
        println!(
            "{}: {:#018x} {}",
            scenario.name,
            fnv1a(&pixels),
            path.display()
        );
    }
    Ok(())
}

/// Measures the public renderer at both required dimensions, the public advance over 65 536
/// ticks, and the state size (REQ-H-05/06). Setup, allocation, palette conversion and file I/O
/// stay outside each frame interval. Sorted sample indices n/2 and n*95/100 preserve the
/// existing median and p95 estimator; the eleven advance samples use median index five.
#[test]
#[ignore = "measurement; run in release without concurrent builds"]
#[allow(clippy::assertions_on_constants)] // Refuse debug execution without rejecting its build.
fn metire_frames() {
    use std::hint::black_box;
    use std::time::Instant;

    assert!(
        !cfg!(debug_assertions),
        "run this measurement with cargo test --release"
    );
    let state_bytes = std::mem::size_of::<Mundus>();
    assert_eq!(state_bytes, 16, "the world occupies exactly two words");
    println!("Mundus: {state_bytes} bytes");

    let scenarios = golden_scenarios();
    for scenario in &scenarios[..5] {
        for (width, height, count) in [(320u16, 180u16, 1000usize), (1920, 1080, 100)] {
            let f = Facultates {
                width,
                height,
                ..Facultates::default()
            };
            let mut pixels = vec![0; usize::from(width) * usize::from(height)];
            let mut tab = Tabula {
                width,
                height,
                pixels: &mut pixels,
            };
            let mut times = Vec::with_capacity(count);

            // Warm the same composition once, outside the measured sample set.
            black_box(&scenario.world).render(
                black_box(&f),
                black_box(&scenario.indicia),
                black_box(&scenario.regimen),
                black_box(&mut tab),
            );
            black_box(&*tab.pixels);

            for _ in 0..count {
                let start = Instant::now();
                black_box(&scenario.world).render(
                    black_box(&f),
                    black_box(&scenario.indicia),
                    black_box(&scenario.regimen),
                    black_box(&mut tab),
                );
                let elapsed = start.elapsed().as_micros();
                black_box(&*tab.pixels);
                times.push(elapsed);
            }
            times.sort_unstable();
            let median = times[count / 2];
            let p95 = times[count * 95 / 100];
            println!(
                "{} {width}x{height} {:?}: n={count}, median {median} us, p95 {p95} us",
                scenario.name, scenario.regimen.stilus
            );
        }
    }

    let f = Facultates::default();
    let indicia = Indicia {
        epoch_ms: NOON,
        ..Indicia::default()
    };
    let mut times = [0u128; 11];
    for elapsed in &mut times {
        let mut world = Mundus::genesis(1);
        let start = Instant::now();
        let announcement = black_box(&mut world).advance(
            black_box(&f),
            black_box(&indicia),
            black_box(65_536),
            black_box(42),
        );
        *elapsed = start.elapsed().as_micros();
        black_box((world, announcement));
    }
    times.sort_unstable();
    println!(
        "advance 65536 ticks of 42 ms: n=11, median {} us, min {} us, max {} us",
        times[5], times[0], times[10]
    );
}
