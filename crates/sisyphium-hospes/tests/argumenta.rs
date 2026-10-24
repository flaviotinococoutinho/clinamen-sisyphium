//! Common argument domains (REQ-C-01, REQ-H-06).
use sisyphium_hospes::argumenta::{Argumenta, Communia};
use sisyphium_hospes::{Error, MAX_CELLS, MAX_PIXELS, MAX_TERMINAL_BYTES, validate_area};
use std::{ffi::OsString, io};

/// Parses a complete common invocation without filesystem effects (REQ-H-01).
fn common(values: &[&str]) -> io::Result<Communia> {
    let values: Vec<_> = values.iter().map(OsString::from).collect();
    let mut args = Argumenta::parse(&values, &[])?;
    let common = Communia::parse(&mut args)?;
    args.finish()?;
    Ok(common)
}

/// Exercises the named host law against independent inputs (REQ-C-05, REQ-H-06, REQ-D-02).
#[test]
fn argument_domains() -> io::Result<()> {
    for values in [
        vec!["--epoch", "1"],
        vec!["--ticks", "65537"],
        vec!["--lingua", "xx"],
        vec!["--ticks", "1", "--ticks", "2"],
        vec!["--energy", "256"],
        vec!["--color-bits", "2"],
        vec!["--tz-offset", "1440"],
        vec!["--australis", "yes"],
        vec!["--unknown"],
        vec!["--energy"],
        vec!["--seed", "-1"],
    ] {
        assert_eq!(
            common(&values).err().map(|e| e.kind()),
            Some(io::ErrorKind::InvalidInput)
        );
    }
    let value = common(&["--epoch", "-1", "--ticks", "0"])?;
    assert_eq!(value.epocha, Some(-1));
    assert_eq!(value.ictus, Some(0));
    assert_eq!(value.indicia(23).epoch_ms, 23);
    let value = common(&[])?;
    assert_eq!(value.energia, 255);
    assert!(!value.facultates(10, 20, false, 0).persistence);
    assert!(validate_area(0, 1, MAX_PIXELS).is_err());
    assert!(validate_area(65535, 65535, MAX_PIXELS).is_err());
    assert_eq!(validate_area(1024, 1024, MAX_CELLS)?, 1_048_576);
    assert_eq!(
        (MAX_PIXELS, MAX_CELLS, MAX_TERMINAL_BYTES),
        (33_554_432, 1_048_576, 67_108_864)
    );
    assert_eq!(
        Error::Usus(io::Error::from(io::ErrorKind::InvalidInput)).exit_code(),
        2
    );
    assert_eq!(
        Error::Operatio(io::Error::from(io::ErrorKind::InvalidInput)).exit_code(),
        1
    );
    Ok(())
}

/// Exercises the named host law against independent inputs (REQ-C-05, REQ-H-06, REQ-D-02).
#[test]
fn host_flags_consumed_exactly_once() -> io::Result<()> {
    let mut args = Argumenta::parse(
        &["--plain".into(), "--width".into(), "25".into()],
        &[("--plain", false), ("--width", true)],
    )?;
    assert_eq!(args.take("--plain"), Some(OsString::new()));
    assert!(args.finish().is_err());
    Ok(())
}

#[cfg(unix)]
/// Exercises the named host law against independent inputs (REQ-C-05, REQ-H-06, REQ-D-02).
#[test]
fn non_utf8_paths_are_not_numbers() -> io::Result<()> {
    use std::os::unix::ffi::OsStringExt;
    let bytes = OsString::from_vec(vec![0xff]);
    let mut args = Argumenta::parse(&["--state".into(), bytes.clone()], &[])?;
    assert_eq!(
        Communia::parse(&mut args)?
            .archivum
            .map(std::path::PathBuf::into_os_string),
        Some(bytes.clone())
    );
    let mut args = Argumenta::parse(&["--ticks".into(), bytes], &[])?;
    assert!(Communia::parse(&mut args).is_err());
    Ok(())
}
