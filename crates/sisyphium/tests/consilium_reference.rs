//! Independent integer-model corpus for all calibrated search vectors (REQ-C-03).

use sisyphium::{
    consilium::consilium,
    hospes::{Facultates, Regimen, Stilus},
    memoria::Memoria,
    mundus::Mundus,
    verbum::Verbum,
};

/// Reproduced by `python3 scripts/consilium-reference.py`; no Rust output feeds the oracle.
#[test]
fn consilium_independent_corpus() -> Result<(), Box<dyn std::error::Error>> {
    let mut count = 0;
    for line in include_str!("fixtures/consilium.tsv").lines().skip(1) {
        let fields = line
            .split('\t')
            .map(str::parse::<u64>)
            .collect::<Result<Vec<_>, _>>()?;
        let [
            verbum,
            memoria,
            width,
            height,
            compute,
            hz,
            cells,
            bits,
            energy,
            stilus,
            strata,
            tick,
            interval,
            layers,
            lines,
        ] = fields.as_slice()
        else {
            return Err("invalid reference row".into());
        };
        let world = Mundus {
            verbum: Verbum::new(*verbum),
            memoria: Memoria::new(*memoria),
        };
        let host = Facultates {
            width: u16::try_from(*width)?,
            height: u16::try_from(*height)?,
            compute: u8::try_from(*compute)?,
            max_hz: u8::try_from(*hz)?,
            cells: *cells != 0,
            color_bits: u8::try_from(*bits)?,
            ..Facultates::default()
        };
        let expected = Regimen {
            stilus: match stilus {
                0 => Stilus::Creta,
                1 => Stilus::Musivum,
                2 => Stilus::Littera,
                3 => Stilus::Bicolor,
                _ => return Err("invalid stilus".into()),
            },
            strata: u8::try_from(*strata)?,
            tick_ms: u32::try_from(*tick)?,
            interval_ms: u32::try_from(*interval)?,
            layers: u8::try_from(*layers)?,
            lines: u8::try_from(*lines)?,
        };
        assert_eq!(
            consilium(&host, &world, u8::try_from(*energy)?),
            expected,
            "vector {count}"
        );
        count += 1;
    }
    assert_eq!(count, 1632);
    Ok(())
}
