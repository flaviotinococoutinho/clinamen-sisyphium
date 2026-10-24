//! Visual export through the static host's encoder, with fixed public golden regimina.
#![allow(clippy::unwrap_used)]
use sisyphium::signum::scaena::fnv1a;
use sisyphium_tabula::imago::{Forma, write_image};
use std::{fs, io, path::Path};

/// Consumes the core's explicit golden export; normal CLI selection remains unchanged.
#[test]
#[ignore = "requires dump_golden_ppm; writes visual inspection artifacts"]
fn export_golden_png() -> io::Result<()> {
    let source = Path::new(env!("CARGO_TARGET_TMPDIR")).join("aurea-cp3");
    let destination = Path::new(env!("CARGO_TARGET_TMPDIR")).join("aurea-cp4");
    fs::create_dir_all(&destination)?;
    let manifest = fs::read_to_string(source.join("manifest.tsv"))?;
    assert_eq!(manifest.lines().count(), 12);
    for line in manifest.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 4);
        let name = fields[0];
        let width = fields[1].parse::<u16>().unwrap();
        let height = fields[2].parse::<u16>().unwrap();
        let expected = u64::from_str_radix(fields[3], 16).unwrap();
        let pixels = fs::read(source.join(format!("{name}.raster")))?;
        assert_eq!(pixels.len(), usize::from(width) * usize::from(height));
        assert_eq!(fnv1a(&pixels), expected, "{name}");
        let colors = fs::read(source.join(format!("{name}.palette")))?;
        assert_eq!(colors.len(), 48);
        let mut palette = [[0; 3]; 16];
        for (color, bytes) in palette.iter_mut().zip(colors.chunks_exact(3)) {
            color.copy_from_slice(bytes);
        }
        for (format, extension) in [(Forma::Png, "png"), (Forma::Ppm, "ppm")] {
            let path = destination.join(format!("{name}.{extension}"));
            let mut file = fs::File::create(&path)?;
            write_image(format, width, height, &pixels, &palette, &mut file)?;
        }
        assert_eq!(
            fs::read(source.join(format!("{name}.ppm")))?,
            fs::read(destination.join(format!("{name}.ppm")))?
        );
        println!("{name}: {expected:016x} {}", destination.display());
    }
    Ok(())
}
