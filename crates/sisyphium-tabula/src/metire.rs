//! Isolated rendering measurements and paired component experiments (REQ-H-05).

use sisyphium::{
    hospes::{
        ANNALES, CAELUM, Facultates, HEXAGRAMMA, IMPOSSIBILIA, Indicia, MARGO, PERSONAE, RUINAE,
        Regimen, Stilus, Tabula,
    },
    mundus::Mundus,
    verbum::Modus,
};
use std::{
    hint::black_box,
    io::{self, Write},
    time::Instant,
};

/// Names and discriminants used by the diagnostic TSV.
const STILI: [(Stilus, &str); 4] = [
    (Stilus::Creta, "creta"),
    (Stilus::Musivum, "musivum"),
    (Stilus::Littera, "littera"),
    (Stilus::Bicolor, "bicolor"),
];

/// A prepared world; no construction or evolution occurs within a timed interval.
#[derive(Clone, Copy)]
struct Scaena {
    nomen: &'static str,
    mundus: Mundus,
    indicia: Indicia,
    regimen: Regimen,
}

/// The five reference families of docs/08-verificacao.md §6–7.
fn scenes() -> [Scaena; 5] {
    let day = Indicia {
        epoch_ms: 1_782_054_000_000,
        ..Indicia::default()
    };
    let evening = Indicia {
        epoch_ms: 1_790_112_600_000,
        energy: 120,
        ..day
    };
    let night = Indicia {
        epoch_ms: 1_797_831_000_000,
        energy: 200,
        idle_s: 3600,
        ..day
    };
    let mut vigil = Mundus::genesis(1);
    vigil.advance(&Facultates::default(), &day, 4096, 42);
    let mut acedia = Mundus::genesis(2);
    acedia.advance(&Facultates::default(), &evening, 4096, 125);
    acedia.verbum = acedia.verbum.with_onus(160);
    let mut dream = Mundus::genesis(3);
    dream.verbum = dream.verbum.with_modus(Modus::Somnium).with_curvatura(-48);
    let mut quies = Mundus::genesis(4);
    quies.verbum = quies.verbum.with_modus(Modus::Quies);
    let base = Regimen::default();
    [
        Scaena {
            nomen: "vigilia",
            mundus: vigil,
            indicia: day,
            regimen: base,
        },
        Scaena {
            nomen: "acedia",
            mundus: acedia,
            indicia: evening,
            regimen: Regimen {
                strata: 2,
                tick_ms: 125,
                interval_ms: 125,
                ..base
            },
        },
        Scaena {
            nomen: "somnium",
            mundus: dream,
            indicia: night,
            regimen: base,
        },
        Scaena {
            nomen: "quies",
            mundus: quies,
            indicia: Indicia {
                epoch_ms: 1_774_008_000_000,
                energy: 20,
                ..day
            },
            regimen: Regimen {
                strata: 1,
                tick_ms: 60_000,
                interval_ms: 60_000,
                ..base
            },
        },
        Scaena {
            nomen: "littera",
            mundus: vigil,
            indicia: day,
            regimen: base,
        },
    ]
}

/// Clock boundary shared by rendering and the ordering test; conversion is not accepted here.
fn interval<T>(render: impl FnOnce(), mut now: impl FnMut() -> T) -> (T, T) {
    let start = now();
    render();
    (start, now())
}

/// Times only render, with prepared references crossing the optimization barrier first.
fn render_ns(scene: &Scaena, host: &Facultates, regimen: &Regimen, tab: &mut Tabula<'_>) -> u128 {
    let (world, host, indicia, regimen, tab) =
        black_box((&scene.mundus, host, &scene.indicia, regimen, tab));
    let (start, end) = interval(|| world.render(host, indicia, regimen, tab), Instant::now);
    black_box(&*tab.pixels);
    end.duration_since(start).as_nanos()
}

/// Median at n/2 and p95 at floor(95n/100), with zero-based indices.
fn statistics(samples: &mut [u128]) -> (u128, u128) {
    samples.sort_unstable();
    (
        samples[samples.len() / 2],
        samples[samples.len() * 95 / 100],
    )
}

/// An isolated change, whose baseline has the same narrative activation.
struct Experimentum {
    elementum: &'static str,
    quantitas: u8,
    scaena: Scaena,
    basis: Regimen,
    varians: Regimen,
}

/// Builds component pairs, activating narrative preconditions in both halves.
fn pairs(scene: Scaena, stilus: Stilus) -> Vec<Experimentum> {
    let base = Regimen {
        stilus,
        strata: 1,
        layers: 0,
        lines: 0,
        ..scene.regimen
    };
    let mut pairs = Vec::new();
    for strata in 2..=4 {
        pairs.push(Experimentum {
            elementum: "stratum",
            quantitas: strata - 1,
            scaena: scene,
            basis: base,
            varians: Regimen { strata, ..base },
        });
    }
    for (index, feature) in [
        "caelum",
        "margo",
        "annales",
        "personae",
        "ruinae",
        "impossibilia",
        "hexagramma",
        "puncta",
    ]
    .into_iter()
    .enumerate()
    {
        let layer = 1 << index;
        let mut active = scene;
        match layer {
            CAELUM | ANNALES => {
                active.mundus.memoria = active
                    .mundus
                    .memoria
                    .with_eventus(3)
                    .with_ramus(0)
                    .with_gradus(599);
                if layer == ANNALES {
                    active.indicia.leaving = true;
                }
            }
            MARGO => active.mundus.verbum = active.mundus.verbum.with_onus(201),
            PERSONAE => {
                active.mundus.memoria = active.mundus.memoria.with_eventus(13).with_gradus(100);
            }
            RUINAE => active.mundus.memoria = active.mundus.memoria.with_clausurae(0x3ffe),
            IMPOSSIBILIA => {
                active.mundus.verbum = active.mundus.verbum.with_conatus(201).with_onus(201);
            }
            HEXAGRAMMA => active.indicia.energy = active.indicia.energy.max(77),
            _ => (),
        }
        for lines in 1..=if layer == ANNALES { 6 } else { 1 } {
            pairs.push(Experimentum {
                elementum: feature,
                quantitas: lines,
                scaena: active,
                basis: base,
                varians: Regimen {
                    layers: layer,
                    lines: if layer == ANNALES { lines } else { 0 },
                    ..base
                },
            });
        }
    }
    if matches!(stilus, Stilus::Littera | Stilus::Bicolor) {
        pairs.push(Experimentum {
            elementum: if stilus == Stilus::Littera {
                "littera"
            } else {
                "bicolor"
            },
            quantitas: 1,
            scaena: scene,
            basis: Regimen {
                stilus: Stilus::Creta,
                ..base
            },
            varians: base,
        });
    }
    pairs
}

/// Measures the reference matrix and paired increments; writes TSV only (REQ-H-05).
/// All allocations, world evolution, palette, encoding, hashing and I/O are outside timing.
///
/// # Errors
/// Returns an I/O error if the TSV cannot be written or the raster cannot be constructed.
pub fn run(selected: Option<Stilus>, output: &mut dyn Write) -> io::Result<()> {
    writeln!(
        output,
        "scenario\tstilus\tlargura\taltura\tstrata\tlayers\tlines\tn\tmediana_ns\tp95_ns\ttipo\tfeature\tquantity\tbase_ns\tdelta_ns\tpair"
    )?;
    for scene in scenes() {
        for (stilus, name) in STILI
            .into_iter()
            .filter(|(s, _)| selected.is_none_or(|wanted| wanted == *s))
        {
            for (width, height, count) in [(320, 180, 1000), (1920, 1080, 100)] {
                let host = Facultates {
                    width,
                    height,
                    ..Facultates::default()
                };
                let mut pixels = vec![0; usize::from(width) * usize::from(height)];
                let mut tab = Tabula::new(width, height, &mut pixels)
                    .ok_or_else(|| io::Error::other("invalid measurement raster"))?;
                let regimen = Regimen {
                    stilus,
                    ..scene.regimen
                };
                let mut full = Vec::with_capacity(count);
                render_ns(&scene, &host, &regimen, &mut tab);
                for _ in 0..count {
                    full.push(render_ns(&scene, &host, &regimen, &mut tab));
                }
                let (median, p95) = statistics(&mut full);
                writeln!(
                    output,
                    "{}\t{name}\t{width}\t{height}\t{}\t{}\t{}\t{count}\t{median}\t{p95}\tfull\tfull\t0\t0\t0\tfull",
                    scene.nomen, regimen.strata, regimen.layers, regimen.lines
                )?;
                for (key, pair) in pairs(scene, stilus).into_iter().enumerate() {
                    let mut baseline = Vec::with_capacity(count);
                    let mut changed = Vec::with_capacity(count);
                    let mut deltas = Vec::with_capacity(count);
                    render_ns(&pair.scaena, &host, &pair.basis, &mut tab);
                    render_ns(&pair.scaena, &host, &pair.varians, &mut tab);
                    for index in 0..count {
                        // Alternate order to distribute short-term drift between both halves.
                        let (a, b) = if index % 2 == 0 {
                            (
                                render_ns(&pair.scaena, &host, &pair.basis, &mut tab),
                                render_ns(&pair.scaena, &host, &pair.varians, &mut tab),
                            )
                        } else {
                            let b = render_ns(&pair.scaena, &host, &pair.varians, &mut tab);
                            (render_ns(&pair.scaena, &host, &pair.basis, &mut tab), b)
                        };
                        baseline.push(a);
                        changed.push(b);
                        deltas.push(b as i128 - a as i128);
                    }
                    let (base_ns, _) = statistics(&mut baseline);
                    let (median, p95) = statistics(&mut changed);
                    deltas.sort_unstable();
                    let delta = deltas[count / 2];
                    let r = pair.varians;
                    writeln!(
                        output,
                        "{}\t{name}\t{width}\t{height}\t{}\t{}\t{}\t{count}\t{median}\t{p95}\tpair\t{}\t{}\t{base_ns}\t{delta}\t{key}",
                        scene.nomen, r.strata, r.layers, r.lines, pair.elementum, pair.quantitas
                    )?;
                }
                output.flush()?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// Conversion and allocation are outside the shared timing boundary (REQ-H-05).
    #[test]
    fn measurement_excludes_conversion() {
        let events = RefCell::new(vec!["allocation", "world", "regimen"]);
        interval(
            || events.borrow_mut().push("render"),
            || events.borrow_mut().push("clock"),
        );
        events
            .borrow_mut()
            .extend(["palette", "conversion", "hash", "disk"]);
        assert_eq!(
            *events.borrow(),
            [
                "allocation",
                "world",
                "regimen",
                "clock",
                "render",
                "clock",
                "palette",
                "conversion",
                "hash",
                "disk"
            ]
        );
    }

    /// Six requested annal lines really exist and all pairs keep baseline parameters fixed.
    #[test]
    fn component_pairs_activate_narrative() {
        for scene in scenes() {
            for pair in pairs(scene, Stilus::Creta) {
                if pair.elementum == "annales" {
                    let text = sisyphium::arbitrium::visible_annales(
                        pair.scaena.mundus.memoria,
                        Facultates::default().lingua,
                        pair.scaena.indicia.leaving,
                        &pair.varians,
                        u16::MAX,
                    );
                    assert_eq!(text.count, pair.quantitas);
                }
                assert_eq!(pair.basis.strata, 1);
                assert_eq!(pair.basis.layers, 0);
            }
        }
    }
}
