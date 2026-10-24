//! Emits C fixtures by calling only the core, independently of the ABI (REQ-H-03).
use sisyphium::{
    consilium::consilium,
    hospes::{Facultates, Indicia, Tabula},
    mundus::Mundus,
};

/// Writes an exact byte array for independent C comparisons (REQ-H-03).
fn emit_bytes(name: &str, bytes: &[u8]) {
    print!("static const uint8_t {name}[] = {{");
    for byte in bytes {
        print!("{byte},");
    }
    println!("}};");
}

/// Emits genesis, selection, step, advance, raster and palette from core laws (REQ-H-03).
fn main() {
    let f = Facultates {
        width: 24,
        height: 16,
        ..Facultates::default()
    };
    let i = Indicia {
        epoch_ms: 1_700_000_000_000,
        ..Indicia::default()
    };
    let mut current = Mundus::genesis(123);
    emit_bytes("GENESIS", &current.format());
    let r = consilium(&f, &current, i.energy);
    println!(
        "static const SyRegimen REGIMEN = {{{},{},{{0,0}},{},{},{},{},{{0,0}}}};",
        r.stilus as u8, r.strata, r.tick_ms, r.interval_ms, r.layers, r.lines
    );
    let mut pixels = [0; 384];
    current.render(
        &f,
        &i,
        &r,
        &mut Tabula {
            width: 24,
            height: 16,
            pixels: &mut pixels,
        },
    );
    emit_bytes("PIXELS", &pixels);
    emit_bytes("PALETTE", current.palette(&f, &i, &r).as_flattened());
    println!("static const uint32_t STEP = {};", current.step(&f, &i).0);
    emit_bytes("STEPPED", &current.format());
    println!(
        "static const uint32_t ADVANCE = {};",
        current.advance(&f, &i, 33, r.tick_ms).0
    );
    emit_bytes("ADVANCED", &current.format());
}
