//! Terminal representation contracts (REQ-H-02).
#![allow(clippy::unwrap_used)]
use sisyphium_textus::terminalis::{format_blocks, format_cells, frame_bound, resolve_size};

/// Native indices include accents and ignore the stratum when selecting color (REQ-H-02).
#[test]
fn native_cells_decode() {
    let mut palette = [[0; 3]; 16];
    palette[3] = [11, 22, 33];
    let cells = [0x45 | 0x3300, 0x46 | 0x300, 0x51 | 0x300, 0xff | 0x300];
    assert_eq!(
        format_cells(&cells, 4, 1, &palette, true).unwrap(),
        "κÁÇ·\n".as_bytes()
    );
    let encoded = String::from_utf8(format_cells(&cells, 4, 1, &palette, false).unwrap()).unwrap();
    assert_eq!(encoded.matches("\x1b[38;2;11;22;33m").count(), 4);
}

/// Raster pairs select the high nibble and use tone one below an odd last row (REQ-H-02).
#[test]
fn blocks_pair_rows() {
    let mut palette = [[0; 3]; 16];
    palette[1] = [1, 2, 3];
    palette[2] = [4, 5, 6];
    let pixels = [0x2f, 0x10, 0x20];
    assert_eq!(
        format_blocks(&pixels, 1, 3, &palette, true).unwrap(),
        "▀\n▀\n".as_bytes()
    );
    let encoded =
        String::from_utf8(format_blocks(&pixels, 1, 3, &palette, false).unwrap()).unwrap();
    assert_eq!(
        encoded.matches("\x1b[38;2;4;5;6m\x1b[48;2;1;2;3m▀").count(),
        2
    );
}

/// Both input and worst-case output have independent finite limits (REQ-H-02).
#[test]
fn terminal_output_bound() {
    assert!(frame_bound(0, 1, false).is_err());
    assert!(frame_bound(u16::MAX, u16::MAX, false).is_err());
    assert!(format_cells(&[], 1, 1, &[[0; 3]; 16], true).is_err());
    assert!(format_blocks(&[], 1, 1, &[[0; 3]; 16], true).is_err());
    assert!(frame_bound(1024, 1024, false).unwrap() <= sisyphium_hospes::MAX_TERMINAL_BYTES);
}

/// Flags precede native measurement and environment, with one reserved physical row (REQ-H-02).
#[test]
fn terminal_size_precedence() {
    assert_eq!(
        resolve_size(None, None, None, None, None).unwrap(),
        (80, 23)
    );
    assert_eq!(
        resolve_size(Some(9), None, Some((90, 40)), Some(2), Some(3)).unwrap(),
        (9, 39)
    );
    assert_eq!(
        resolve_size(None, Some(8), None, Some(7), Some(9)).unwrap(),
        (7, 8)
    );
    assert_eq!(
        resolve_size(None, None, Some((1, 1)), None, None).unwrap(),
        (1, 1)
    );
    assert!(resolve_size(Some(0), None, None, None, None).is_err());
}
