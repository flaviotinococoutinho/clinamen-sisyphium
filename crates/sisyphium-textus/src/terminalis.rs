//! Bounded native terminal encoding and size probing (REQ-H-02).
use sisyphium::signum::littera::{ACUTAE, ORDO};
use sisyphium_hospes::{MAX_CELLS, MAX_PIXELS, MAX_TERMINAL_BYTES, validate_area};
use std::{
    io::{self, Read, Write},
    process::{Command, Stdio},
};

/// Bounds a frame by two maximal RGB escapes (38 bytes), UTF-8 (4), row reset (4),
/// newline (1), final reset (4), and inter-frame home (3), before allocation (REQ-H-02).
///
/// # Errors
/// Rejects zero dimensions, excess cells, arithmetic overflow or more than 64 MiB.
pub fn frame_bound(cols: u16, rows: u16, plain: bool) -> io::Result<usize> {
    let area = validate_area(cols, rows, MAX_CELLS)?;
    area.checked_mul(if plain { 4 } else { 42 })
        .and_then(|bytes| bytes.checked_add(usize::from(rows) * if plain { 1 } else { 5 }))
        .and_then(|bytes| bytes.checked_add(if plain { 1 } else { 7 }))
        .filter(|&bytes| bytes <= MAX_TERMINAL_BYTES)
        .ok_or_else(|| invalid("terminal frame exceeds byte limit"))
}

/// Reserves the complete worst-case frame before output or persistence (REQ-H-02).
///
/// # Errors
/// Propagates invalid dimensions and allocation failure.
pub fn reserve_frame(cols: u16, rows: u16, plain: bool) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(frame_bound(cols, rows, plain)?)
        .map_err(io::Error::other)?;
    Ok(bytes)
}

/// Decodes glyph indices, accents and the tone nibble into a bounded frame (REQ-H-02).
///
/// # Errors
/// Rejects short input, excess dimensions or failed allocation.
pub fn format_cells(
    cells: &[u16],
    cols: u16,
    rows: u16,
    palette: &[[u8; 3]; 16],
    plain: bool,
) -> io::Result<Vec<u8>> {
    let mut bytes = reserve_frame(cols, rows, plain)?;
    encode_cells(&mut bytes, cells, cols, rows, palette, plain)?;
    Ok(bytes)
}

/// Writes native cells into an already reserved frame (REQ-H-02).
///
/// # Errors
/// Rejects short input or insufficient reserved capacity.
pub(crate) fn encode_cells(
    bytes: &mut Vec<u8>,
    cells: &[u16],
    cols: u16,
    rows: u16,
    palette: &[[u8; 3]; 16],
    plain: bool,
) -> io::Result<()> {
    check_buffer(bytes, cols, rows, plain)?;
    let area = validate_area(cols, rows, MAX_CELLS)?;
    let cells = cells
        .get(..area)
        .ok_or_else(|| invalid("short cell buffer"))?;
    for row in cells.chunks_exact(usize::from(cols)) {
        for &cell in row {
            if !plain {
                color(bytes, palette[usize::from((cell >> 8) & 15)], false)?;
            }
            let glyph = ORDO
                .chars()
                .chain(ACUTAE.chars())
                .nth(usize::from(cell & 255))
                .unwrap_or('·');
            let mut utf8 = [0; 4];
            bytes.extend_from_slice(glyph.encode_utf8(&mut utf8).as_bytes());
        }
        end_row(bytes, plain);
    }
    if !plain {
        bytes.extend_from_slice(b"\x1b[0m");
    }
    Ok(())
}

/// Pairs raster rows as upper-half blocks with tone one below an odd row (REQ-H-02).
///
/// # Errors
/// Rejects short input, excess dimensions or failed allocation.
pub fn format_blocks(
    pixels: &[u8],
    width: u16,
    height: u16,
    palette: &[[u8; 3]; 16],
    plain: bool,
) -> io::Result<Vec<u8>> {
    let rows = height / 2 + height % 2;
    let mut bytes = reserve_frame(width, rows, plain)?;
    encode_blocks(&mut bytes, pixels, width, height, palette, plain)?;
    Ok(bytes)
}

/// Writes paired raster rows into an already reserved frame (REQ-H-02).
///
/// # Errors
/// Rejects short input or insufficient reserved capacity.
pub(crate) fn encode_blocks(
    bytes: &mut Vec<u8>,
    pixels: &[u8],
    width: u16,
    height: u16,
    palette: &[[u8; 3]; 16],
    plain: bool,
) -> io::Result<()> {
    check_buffer(bytes, width, height / 2 + height % 2, plain)?;
    let area = validate_area(width, height, MAX_PIXELS)?;
    let pixels = pixels
        .get(..area)
        .ok_or_else(|| invalid("short raster buffer"))?;
    let (mut dark, mut light) = pixels.iter().fold((15_u8, 0_u8), |(dark, light), pixel| {
        (dark.min(pixel >> 4), light.max(pixel >> 4))
    });
    if height % 2 != 0 {
        dark = dark.min(1);
        light = light.max(1);
    }
    // Plain output retains the frame's observed contrast without terminal colors.
    let threshold = dark + (light - dark) / 2;
    for y in (0..usize::from(height)).step_by(2) {
        for x in 0..usize::from(width) {
            let top = pixels[y * usize::from(width) + x] >> 4;
            let bottom = pixels
                .get((y + 1) * usize::from(width) + x)
                .map_or(1, |pixel| pixel >> 4);
            if !plain {
                color(bytes, palette[usize::from(top)], false)?;
                color(bytes, palette[usize::from(bottom)], true)?;
            }
            let glyph = if plain {
                match (top > threshold, bottom > threshold) {
                    (false, false) => " ",
                    (true, false) => "▀",
                    (false, true) => "▄",
                    (true, true) => "█",
                }
            } else {
                "▀"
            };
            bytes.extend_from_slice(glyph.as_bytes());
        }
        end_row(bytes, plain);
    }
    if !plain {
        bytes.extend_from_slice(b"\x1b[0m");
    }
    Ok(())
}

/// Prevents implicit buffer growth beyond the preflight reservation (REQ-H-02).
fn check_buffer(bytes: &Vec<u8>, cols: u16, rows: u16, plain: bool) -> io::Result<()> {
    if bytes.capacity() < frame_bound(cols, rows, plain)? {
        return Err(invalid("frame buffer not reserved"));
    }
    Ok(())
}

/// Emits one bounded decimal RGB escape without a temporary string (REQ-H-02).
fn color(bytes: &mut Vec<u8>, rgb: [u8; 3], background: bool) -> io::Result<()> {
    write!(
        bytes,
        "\x1b[{};2;{};{};{}m",
        if background { 48 } else { 38 },
        rgb[0],
        rgb[1],
        rgb[2]
    )
}

/// Ends a row with neutral attributes and its geometry-preserving newline (REQ-H-02).
fn end_row(bytes: &mut Vec<u8>, plain: bool) {
    if !plain {
        bytes.extend_from_slice(b"\x1b[0m");
    }
    bytes.push(b'\n');
}

/// Resolves each axis independently; explicit rows are usable rows (REQ-H-02).
///
/// # Errors
/// Rejects explicitly zero dimensions.
pub fn resolve_size(
    cols: Option<u16>,
    rows: Option<u16>,
    native: Option<(u16, u16)>,
    env_cols: Option<u16>,
    env_rows: Option<u16>,
) -> io::Result<(u16, u16)> {
    let width = cols.unwrap_or_else(|| {
        native
            .map(|size| size.0)
            .filter(|&v| v > 0)
            .or(env_cols.filter(|&v| v > 0))
            .unwrap_or(80)
    });
    let height = rows.unwrap_or_else(|| {
        native
            .map(|size| size.1)
            .filter(|&v| v > 0)
            .or(env_rows.filter(|&v| v > 0))
            .unwrap_or(24)
            .saturating_sub(1)
            .max(1)
    });
    validate_area(width, height, MAX_CELLS)?;
    Ok((width, height))
}

/// Measures physical terminal dimensions through one bounded native probe (REQ-H-02).
/// The caller controls the one-second cache; no terminal mode is changed.
#[must_use]
pub fn measure() -> Option<(u16, u16)> {
    #[cfg(unix)]
    {
        probe(Command::new("stty").arg("size"), true)
    }
    #[cfg(windows)]
    {
        probe(
            Command::new("powershell").args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "[Console]::WindowWidth; [Console]::WindowHeight",
            ]),
            false,
        )
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

/// Reads at most 128 bytes, rejects larger output and reaps the probe (REQ-H-02).
#[cfg(any(unix, windows))]
fn probe(command: &mut Command, reverse: bool) -> Option<(u16, u16)> {
    let mut child = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut bytes = [0; 129];
    let result = child.stdout.take().and_then(|mut output| {
        let mut count = 0;
        while count < bytes.len() {
            let read = output.read(&mut bytes[count..]).ok()?;
            if read == 0 {
                break;
            }
            count += read;
        }
        (count <= 128).then_some(count)
    });
    if result.is_none() {
        let _ = child.kill();
    }
    let success = child.wait().ok()?.success();
    let text = std::str::from_utf8(bytes.get(..result?)?).ok()?;
    let mut values = text.split_whitespace();
    let first = values.next()?.parse::<u16>().ok()?;
    let second = values.next()?.parse::<u16>().ok()?;
    if !success || values.next().is_some() || first == 0 || second == 0 {
        return None;
    }
    Some(if reverse {
        (second, first)
    } else {
        (first, second)
    })
}

/// Constructs a dimension or representation diagnostic (REQ-H-02).
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
