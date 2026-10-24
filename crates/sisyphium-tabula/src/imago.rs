//! PPM P6 and indexed PNG, following W3C PNG, RFC 1950 and RFC 1951 (REQ-H-02).
use sisyphium::signum::pixel::lum_of;
use sisyphium_hospes::{MAX_PIXELS, validate_area};
use std::io::{self, Write};

/// Portable static image encodings (REQ-H-02).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Forma {
    /// Indexed PNG, depth 8, sixteen palette entries, stored DEFLATE blocks.
    Png,
    /// Binary PPM P6, RGB samples with maximum 255.
    Ppm,
}

/// Writes exactly the raster prefix, decoding only the luminance nibble (REQ-H-02).
/// PPM holds one RGB row; PNG holds at most 65535 raw filtered bytes.
///
/// # Errors
/// Rejects invalid dimensions, short rasters, failed allocations and output failures.
pub fn write_image(
    format: Forma,
    width: u16,
    height: u16,
    pixels: &[u8],
    palette: &[[u8; 3]; 16],
    output: &mut dyn Write,
) -> io::Result<()> {
    let area = validate_area(width, height, MAX_PIXELS)?;
    let pixels = pixels
        .get(..area)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "short raster"))?;
    match format {
        Forma::Ppm => write_ppm(width, height, pixels, palette, output),
        Forma::Png => write_png(width, height, pixels, palette, output),
    }
}

/// Reserves bounded storage before any image bytes are written (REQ-H-02).
fn buffer(length: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length).map_err(io::Error::other)?;
    Ok(bytes)
}

/// Expands a row to RGB according to PPM P6 (REQ-H-02).
fn write_ppm(
    width: u16,
    height: u16,
    pixels: &[u8],
    palette: &[[u8; 3]; 16],
    output: &mut dyn Write,
) -> io::Result<()> {
    let mut row = buffer(usize::from(width) * 3)?;
    write!(output, "P6\n{width} {height}\n255\n")?;
    for source in pixels.chunks_exact(usize::from(width)) {
        row.clear();
        for &pixel in source {
            row.extend_from_slice(&palette[usize::from(lum_of(pixel))]);
        }
        output.write_all(&row)?;
    }
    Ok(())
}

/// Accumulates reflected CRC-32, polynomial 0xedb88320 (W3C PNG §5.5).
fn crc_update(mut crc: u32, bytes: &[u8]) -> u32 {
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    crc
}

/// Frames one chunk and includes its type and all payload parts in CRC (W3C PNG §5.3).
fn chunk(output: &mut dyn Write, kind: [u8; 4], parts: &[&[u8]]) -> io::Result<()> {
    let length = parts
        .iter()
        .try_fold(0u32, |sum, part| {
            u32::try_from(part.len())
                .ok()
                .and_then(|length| sum.checked_add(length))
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "PNG chunk too large"))?;
    output.write_all(&length.to_be_bytes())?;
    output.write_all(&kind)?;
    let mut crc = crc_update(u32::MAX, &kind);
    for part in parts {
        output.write_all(part)?;
        crc = crc_update(crc, part);
    }
    output.write_all(&(!crc).to_be_bytes())
}

/// Streams a single zlib stream through contiguous IDAT chunks (W3C PNG §11.2).
fn write_png(
    width: u16,
    height: u16,
    pixels: &[u8],
    palette: &[[u8; 3]; 16],
    output: &mut dyn Write,
) -> io::Result<()> {
    let stride = usize::from(width) + 1;
    let total = stride * usize::from(height);
    let mut block = buffer(total.min(65_535))?;
    let mut header = [0; 13];
    header[..4].copy_from_slice(&u32::from(width).to_be_bytes());
    header[4..8].copy_from_slice(&u32::from(height).to_be_bytes());
    header[8] = 8;
    header[9] = 3;
    let mut colors = [0; 48];
    for (rgb, color) in colors.chunks_exact_mut(3).zip(palette) {
        rgb.copy_from_slice(color);
    }
    output.write_all(b"\x89PNG\r\n\x1a\n")?;
    chunk(output, *b"IHDR", &[&header])?;
    chunk(output, *b"PLTE", &[&colors])?;
    chunk(output, *b"IDAT", &[&[0x78, 0x01]])?;
    let (mut first, mut second) = (1u32, 0u32);
    let mut offset = 0;
    while offset < total {
        block.clear();
        let end = (offset + 65_535).min(total);
        for index in offset..end {
            let column = index % stride;
            let byte = if column == 0 {
                0
            } else {
                lum_of(pixels[index / stride * usize::from(width) + column - 1])
            };
            block.push(byte);
            first = (first + u32::from(byte)) % 65_521;
            second = (second + first) % 65_521;
        }
        // Each stored block starts at a byte boundary; unused header bits are zero (RFC 1951 §3.2.4).
        let length = block.len() as u16;
        let mut stored = [0; 5];
        stored[0] = u8::from(end == total);
        stored[1..3].copy_from_slice(&length.to_le_bytes());
        stored[3..5].copy_from_slice(&(!length).to_le_bytes());
        chunk(output, *b"IDAT", &[&stored, &block])?;
        offset = end;
    }
    chunk(output, *b"IDAT", &[&((second << 16) | first).to_be_bytes()])?;
    chunk(output, *b"IEND", &[])
}
