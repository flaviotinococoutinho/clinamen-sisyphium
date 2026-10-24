#![allow(clippy::unwrap_used)]
//! Image format contracts, independent of world rendering.
use sisyphium_tabula::imago::{Forma, write_image};

#[test]
fn ppm_maps_luminance_only() {
    let mut palette = [[0; 3]; 16];
    palette[1] = [11, 22, 33];
    let mut result = Vec::new();
    write_image(Forma::Ppm, 2, 1, &[0x10, 0x1f, 0xff], &palette, &mut result).unwrap();
    assert_eq!(result, b"P6\n2 1\n255\n\x0b\x16\x21\x0b\x16\x21");
    let mut empty = Vec::new();
    assert!(write_image(Forma::Ppm, 2, 1, &[0x10], &palette, &mut empty).is_err());
    assert_eq!(empty, [] as [u8; 0]);
}

#[test]
fn png_structure_and_boundaries() {
    for (width, height) in [(1, 1), (320, 180), (254, 257), (257, 256)] {
        let count = usize::from(width) * usize::from(height);
        let mut pixels = vec![0x1f; count];
        pixels.push(0xff);
        let mut result = Vec::new();
        write_image(
            Forma::Png,
            width,
            height,
            &pixels,
            &[[17; 3]; 16],
            &mut result,
        )
        .unwrap();
        assert_eq!(&result[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(&result[12..16], b"IHDR");
        assert_eq!(&result[24..29], &[8, 3, 0, 0, 0]);
        assert_eq!(&result[result.len() - 8..result.len() - 4], b"IEND");
        let mut exact = Vec::new();
        write_image(
            Forma::Png,
            width,
            height,
            &pixels[..count],
            &[[17; 3]; 16],
            &mut exact,
        )
        .unwrap();
        assert_eq!(result, exact);
    }
}

#[test]
fn encoder_rejects_invalid_rasters_before_writing() {
    for format in [Forma::Png, Forma::Ppm] {
        for (width, height, pixels) in [
            (0, 1, &[0][..]),
            (1, 0, &[0][..]),
            (65535, 65535, &[][..]),
            (2, 1, &[0][..]),
        ] {
            let mut output = Vec::new();
            assert_eq!(
                write_image(format, width, height, pixels, &[[0; 3]; 16], &mut output)
                    .unwrap_err()
                    .kind(),
                std::io::ErrorKind::InvalidInput
            );
            assert_eq!(output, [] as [u8; 0]);
        }
    }
}
