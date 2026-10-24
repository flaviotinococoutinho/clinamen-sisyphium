//! C boundary laws (REQ-H-03).
#![allow(unsafe_code, clippy::unwrap_used, clippy::borrow_as_ptr)]
use sisyphium::{hospes::*, mundus::Mundus, verbum::Modus};
use sisyphium_limen::*;

/// Serialization writes exactly 32 bytes and preserves both words (REQ-L0-03).
#[test]
fn ffi_roundtrip() {
    for seed in [0, 1, u64::MAX] {
        let (mut word, mut memory) = (0, 0);
        let mut text = [0xA5; 34];
        let (mut parsed_word, mut parsed_memory) = (99, 88);
        // SAFETY: live disjoint aligned objects; the text middle has exactly 32 bytes.
        unsafe {
            sisyphium_genesis(seed, &mut word, &mut memory);
            sisyphium_format(word, memory, text.as_mut_ptr().add(1));
            assert_eq!(
                sisyphium_parse(text.as_ptr().add(1), &mut parsed_word, &mut parsed_memory),
                1
            );
        }
        assert_eq!((word, memory), (parsed_word, parsed_memory));
        assert_eq!((text[0], text[33]), (0xA5, 0xA5));
        assert_eq!(&text[1..33], &Mundus::genesis(seed).format());
    }
    let parts: Vec<u32> = env!("CARGO_PKG_VERSION")
        .split('.')
        .map(|v| v.parse().unwrap())
        .collect();
    assert!(parts.iter().all(|&part| part <= 255));
    assert_eq!(
        sisyphium_version(),
        parts[0] << 16 | parts[1] << 8 | parts[2]
    );
}

/// All public operations retain the core's deterministic laws (REQ-H-03).
#[test]
fn ffi_matches_core() {
    for mode in [Modus::Vigilia, Modus::Quies, Modus::Somnium] {
        for cells in [false, true] {
            let f = Facultates {
                width: 24,
                height: 16,
                cells,
                ..Facultates::default()
            };
            let i = Indicia {
                epoch_ms: 1_700_000_000_000,
                touch: Some((45, 200)),
                ..Indicia::default()
            };
            let sf = SyFacultates::from(f);
            let si = SyIndicia::from(i);
            let mut current = Mundus::genesis(123);
            current.verbum = current.verbum.with_modus(mode);
            let (mut word, mut memory) = (current.verbum.bits(), current.memoria.bits());
            let r = sisyphium::consilium::consilium(&f, &current, i.energy);
            let mut sr = SyRegimen::default();
            let mut actual = [0xA5; 386];
            let mut expected = [0; 384];
            let mut palette = [0xA5; 50];
            current.render(&f, &i, &r, &mut Tabula::new(24, 16, &mut expected).unwrap());
            // SAFETY: all objects are live, aligned and disjoint; buffers meet exact extents.
            unsafe {
                sisyphium_consilium(&sf, word, memory, i.energy, &mut sr);
                assert_eq!(sr, SyRegimen::from(r));
                sisyphium_render(
                    word,
                    memory,
                    &sf,
                    &sr,
                    &si,
                    actual.as_mut_ptr().add(1),
                    24,
                    16,
                );
                sisyphium_palette(word, memory, &sf, &sr, &si, palette.as_mut_ptr().add(1));
                assert_eq!(
                    sisyphium_step(&mut word, &mut memory, &sf, &si),
                    u32::from(current.step(&f, &i).0)
                );
                assert_eq!(
                    (word, memory),
                    (current.verbum.bits(), current.memoria.bits())
                );
                assert_eq!(
                    sisyphium_advance(&mut word, &mut memory, &sf, &si, 33, r.tick_ms),
                    u32::from(current.advance(&f, &i, 33, r.tick_ms).0)
                );
            }
            assert_eq!(
                (word, memory),
                (current.verbum.bits(), current.memoria.bits())
            );
            assert_eq!(&actual[1..385], expected);
            let initial = Mundus {
                verbum: sisyphium::verbum::Verbum::new(
                    Mundus::genesis(123).verbum.with_modus(mode).bits(),
                ),
                memoria: Mundus::genesis(123).memoria,
            };
            assert_eq!(&palette[1..49], initial.palette(&f, &i, &r).as_flattened());
            assert_eq!(
                (actual[0], actual[385], palette[0], palette[49]),
                (0xA5, 0xA5, 0xA5, 0xA5)
            );
            assert_eq!(sf, SyFacultates::from(f));
            assert_eq!(si, SyIndicia::from(i));
        }
    }
}

/// Invalid descriptors never mutate any output (REQ-H-03).
fn rejected(f: SyFacultates, i: SyIndicia, r: SyRegimen, bad_f: bool, bad_i: bool, bad_r: bool) {
    let (mut word, mut memory) = (17, 19);
    let mut pixels = [0xA5; 384];
    let mut palette = [0xA5; 48];
    let mut regimen = SyRegimen::default();
    // SAFETY: all objects are initialized, aligned, live and disjoint; arrays have full extents.
    unsafe {
        if bad_f || bad_i {
            assert_eq!(sisyphium_step(&mut word, &mut memory, &f, &i), 0);
            assert_eq!(sisyphium_advance(&mut word, &mut memory, &f, &i, 10, 42), 0);
        }
        if bad_f {
            sisyphium_consilium(&f, word, memory, 255, &mut regimen);
        }
        assert!(bad_f || bad_i || bad_r);
        sisyphium_render(word, memory, &f, &r, &i, pixels.as_mut_ptr(), 24, 16);
        sisyphium_palette(word, memory, &f, &r, &i, palette.as_mut_ptr());
    }
    assert_eq!((word, memory), (17, 19));
    assert_eq!(regimen, SyRegimen::default());
    assert_eq!(pixels, [0xA5; 384]);
    assert_eq!(palette, [0xA5; 48]);
}

/// Every flag, enum, cadence, range and reserved byte is validated (REQ-H-03).
#[test]
#[allow(clippy::too_many_lines)] // One matrix enumerates every independent wire validation.
fn ffi_invalid_parameters_no_write() {
    let f = SyFacultates::from(Facultates {
        width: 24,
        height: 16,
        ..Facultates::default()
    });
    let i = SyIndicia::from(Indicia::default());
    let r = SyRegimen::from(Regimen::default());
    for value in 2..=255 {
        for field in 0..7 {
            let mut invalid = f;
            match field {
                0 => invalid.touch = value,
                1 => invalid.motion = value,
                2 => invalid.occlusion = value,
                3 => invalid.persistence = value,
                4 => invalid.cells = value,
                5 => invalid.australis = value,
                _ => invalid.lingua = value,
            }
            rejected(invalid, i, r, true, false, false);
        }
        for field in 0..3 {
            let mut invalid = i;
            match field {
                0 => invalid.occluded = value,
                1 => invalid.leaving = value,
                _ => invalid.has_touch = value,
            }
            rejected(f, invalid, r, false, true, false);
        }
    }
    for value in 0..=255 {
        if !matches!(value, 1 | 4 | 8 | 24) {
            rejected(
                SyFacultates {
                    color_bits: value,
                    ..f
                },
                i,
                r,
                true,
                false,
                false,
            );
        }
        if value > 3 {
            rejected(f, i, SyRegimen { stilus: value, ..r }, false, false, true);
        }
        if !(1..=4).contains(&value) {
            rejected(f, i, SyRegimen { strata: value, ..r }, false, false, true);
        }
        if value > 6 {
            rejected(f, i, SyRegimen { lines: value, ..r }, false, false, true);
        }
    }
    rejected(SyFacultates { width: 0, ..f }, i, r, true, false, false);
    rejected(SyFacultates { height: 0, ..f }, i, r, true, false, false);
    rejected(
        f,
        SyIndicia {
            reservatum0: 1,
            ..i
        },
        r,
        false,
        true,
        false,
    );
    for index in 0..5 {
        let mut bad = i;
        bad.reservata[index] = 1;
        rejected(f, bad, r, false, true, false);
    }
    for index in 0..2 {
        let mut bad = r;
        bad.reservata0[index] = 1;
        rejected(f, i, bad, false, false, true);
        let mut bad = r;
        bad.reservata1[index] = 1;
        rejected(f, i, bad, false, false, true);
    }
    for tick_ms in [0, 1, 43, 1000, u32::MAX] {
        rejected(f, i, SyRegimen { tick_ms, ..r }, false, false, true);
    }
    for interval_ms in [0, 1, 43, u32::MAX] {
        rejected(f, i, SyRegimen { interval_ms, ..r }, false, false, true);
    }
    rejected(
        f,
        i,
        SyRegimen {
            tick_ms: 125,
            interval_ms: 42,
            ..r
        },
        false,
        false,
        true,
    );
    let (mut word, mut memory) = (17, 19);
    let mut pixels = [0xA5; 384];
    // SAFETY: initialized disjoint objects and full extents; null pointers are explicitly allowed.
    unsafe {
        for (ticks, tick_ms) in [(65537, 42), (1, 0), (1, 1000), (u32::MAX, 42)] {
            assert_eq!(
                sisyphium_advance(&mut word, &mut memory, &f, &i, ticks, tick_ms),
                0
            );
        }
        sisyphium_render(word, memory, &f, &r, &i, pixels.as_mut_ptr(), 23, 16);
        assert_eq!(
            sisyphium_parse([b'Z'; 32].as_ptr(), &mut word, &mut memory),
            0
        );
    }
    assert_eq!((word, memory), (17, 19));
    assert_eq!(pixels, [0xA5; 384]);
}

/// Every required pointer accepts null as a rejected call without writing (REQ-H-03).
#[test]
fn ffi_all_null_positions() {
    use std::ptr::{null, null_mut};
    let f = SyFacultates::from(Facultates {
        width: 24,
        height: 16,
        ..Facultates::default()
    });
    let i = SyIndicia::from(Indicia::default());
    let r = SyRegimen::from(Regimen::default());
    let (mut word, mut memory) = (17, 19);
    let mut out = SyRegimen::default();
    let mut pixels = [0xA5; 384];
    let mut palette = [0xA5; 48];
    let text = [b'0'; 32];
    // SAFETY: each pointer is null or addresses a live aligned disjoint object of the full extent.
    unsafe {
        sisyphium_genesis(1, null_mut(), &mut memory);
        sisyphium_genesis(1, &mut word, null_mut());
        assert_eq!(sisyphium_parse(null(), &mut word, &mut memory), 0);
        assert_eq!(sisyphium_parse(text.as_ptr(), null_mut(), &mut memory), 0);
        assert_eq!(sisyphium_parse(text.as_ptr(), &mut word, null_mut()), 0);
        sisyphium_format(word, memory, null_mut());
        sisyphium_consilium(null(), word, memory, 255, &mut out);
        sisyphium_consilium(&f, word, memory, 255, null_mut());
        for index in 0..4 {
            let wp = if index == 0 {
                null_mut()
            } else {
                &raw mut word
            };
            let mp = if index == 1 {
                null_mut()
            } else {
                &raw mut memory
            };
            let fp = if index == 2 { null() } else { &raw const f };
            let ip = if index == 3 { null() } else { &raw const i };
            assert_eq!(sisyphium_step(wp, mp, fp, ip), 0);
            assert_eq!(sisyphium_advance(wp, mp, fp, ip, 1, 42), 0);
        }
        for index in 0..4 {
            let fp = if index == 0 { null() } else { &raw const f };
            let rp = if index == 1 { null() } else { &raw const r };
            let ip = if index == 2 { null() } else { &raw const i };
            sisyphium_render(
                word,
                memory,
                fp,
                rp,
                ip,
                if index == 3 {
                    null_mut()
                } else {
                    pixels.as_mut_ptr()
                },
                24,
                16,
            );
            sisyphium_palette(
                word,
                memory,
                fp,
                rp,
                ip,
                if index == 3 {
                    null_mut()
                } else {
                    palette.as_mut_ptr()
                },
            );
        }
    }
    assert_eq!((word, memory), (17, 19));
    assert_eq!(out, SyRegimen::default());
    assert_eq!(pixels, [0xA5; 384]);
    assert_eq!(palette, [0xA5; 48]);
}

/// Accepted extreme offsets and all cadences preserve the core law (REQ-H-03, REQ-H-06).
#[test]
fn ffi_extremes_and_normalization() {
    for offset in [i16::MIN, i16::MAX] {
        let f = Facultates {
            tz_offset_min: offset,
            ..Facultates::default()
        };
        let i = Indicia::default();
        for tick in [42, 125, 250, 60_000] {
            for ticks in [0, 65_536] {
                let mut expected = Mundus {
                    verbum: sisyphium::verbum::Verbum::new(0),
                    memoria: sisyphium::memoria::Memoria::new(0),
                };
                let result = expected.advance(&f, &i, ticks, tick);
                let (mut word, mut memory) = (0, 0);
                // SAFETY: two disjoint aligned words and live input temporaries meet the contract.
                let actual = unsafe {
                    sisyphium_advance(&mut word, &mut memory, &f.into(), &i.into(), ticks, tick)
                };
                assert_eq!(actual, u32::from(result.0));
                assert_eq!(
                    (word, memory),
                    (expected.verbum.bits(), expected.memoria.bits())
                );
            }
        }
    }
}

/// Each field retains the published byte offset and native C alignment (REQ-H-03).
#[test]
fn ffi_layouts() {
    use std::mem::{align_of, offset_of, size_of};
    assert_eq!(size_of::<SyFacultates>(), 16);
    assert_eq!(offset_of!(SyFacultates, width), 0);
    assert_eq!(offset_of!(SyFacultates, height), 2);
    assert_eq!(offset_of!(SyFacultates, color_bits), 4);
    assert_eq!(offset_of!(SyFacultates, max_hz), 5);
    assert_eq!(offset_of!(SyFacultates, touch), 6);
    assert_eq!(offset_of!(SyFacultates, motion), 7);
    assert_eq!(offset_of!(SyFacultates, occlusion), 8);
    assert_eq!(offset_of!(SyFacultates, persistence), 9);
    assert_eq!(offset_of!(SyFacultates, cells), 10);
    assert_eq!(offset_of!(SyFacultates, australis), 11);
    assert_eq!(offset_of!(SyFacultates, tz_offset_min), 12);
    assert_eq!(offset_of!(SyFacultates, lingua), 14);
    assert_eq!(offset_of!(SyFacultates, compute), 15);
    assert_eq!(size_of::<SyIndicia>(), 32);
    assert_eq!(offset_of!(SyIndicia, epoch_ms), 0);
    assert_eq!(offset_of!(SyIndicia, energy), 8);
    assert_eq!(offset_of!(SyIndicia, seism), 9);
    assert_eq!(offset_of!(SyIndicia, automatism), 10);
    assert_eq!(offset_of!(SyIndicia, reservatum0), 11);
    assert_eq!(offset_of!(SyIndicia, idle_s), 12);
    assert_eq!(offset_of!(SyIndicia, returned_ms), 16);
    assert_eq!(offset_of!(SyIndicia, occluded), 20);
    assert_eq!(offset_of!(SyIndicia, leaving), 21);
    assert_eq!(offset_of!(SyIndicia, touch_x), 22);
    assert_eq!(offset_of!(SyIndicia, touch_y), 23);
    assert_eq!(offset_of!(SyIndicia, has_touch), 24);
    assert_eq!(offset_of!(SyIndicia, offset), 25);
    assert_eq!(offset_of!(SyIndicia, tilt), 26);
    assert_eq!(offset_of!(SyIndicia, reservata), 27);
    assert_eq!(size_of::<SyRegimen>(), 16);
    assert_eq!(offset_of!(SyRegimen, stilus), 0);
    assert_eq!(offset_of!(SyRegimen, strata), 1);
    assert_eq!(offset_of!(SyRegimen, reservata0), 2);
    assert_eq!(offset_of!(SyRegimen, tick_ms), 4);
    assert_eq!(offset_of!(SyRegimen, interval_ms), 8);
    assert_eq!(offset_of!(SyRegimen, layers), 12);
    assert_eq!(offset_of!(SyRegimen, lines), 13);
    assert_eq!(offset_of!(SyRegimen, reservata1), 14);
    assert_eq!(align_of::<SyFacultates>(), 2);
    assert_eq!(align_of::<SyRegimen>(), 4);
    assert_eq!(align_of::<SyIndicia>(), align_of::<i64>());
    #[cfg(target_pointer_width = "64")]
    assert_eq!(align_of::<SyIndicia>(), 8);
}

/// All four stili accept uninitialized output storage and match the core raster (REQ-H-03).
#[test]
fn ffi_render_all_stili_uninitialized_output() {
    use std::mem::MaybeUninit;
    let current = Mundus::genesis(321);
    let f = Facultates {
        width: 24,
        height: 16,
        lingua: Lingua::Pt,
        ..Facultates::default()
    };
    let i = Indicia::default();
    for stilus in [
        Stilus::Creta,
        Stilus::Musivum,
        Stilus::Littera,
        Stilus::Bicolor,
    ] {
        let r = Regimen {
            stilus,
            ..Regimen::default()
        };
        let mut expected = [0; 384];
        current.render(&f, &i, &r, &mut Tabula::new(24, 16, &mut expected).unwrap());
        let mut output = MaybeUninit::<[u8; 384]>::uninit();
        let mut palette = [0; 48];
        // SAFETY: the output allocation is writable for 384 bytes; render initializes it before
        // forming a slice. All descriptors are valid live temporaries, disjoint from outputs.
        unsafe {
            sisyphium_render(
                current.verbum.bits(),
                current.memoria.bits(),
                &f.into(),
                &r.into(),
                &i.into(),
                output.as_mut_ptr().cast(),
                24,
                16,
            );
            assert_eq!(output.assume_init(), expected);
            sisyphium_palette(
                current.verbum.bits(),
                current.memoria.bits(),
                &f.into(),
                &r.into(),
                &i.into(),
                palette.as_mut_ptr(),
            );
        }
        assert_eq!(palette, current.palette(&f, &i, &r).as_flattened());
    }
}
