#![allow(clippy::unwrap_used)]
//! Static host public API parity and transaction boundaries.
use sisyphium::{
    consilium::consilium,
    hospes::{Facultates, Indicia, Tabula},
    mundus::Mundus,
    signum::pixel::lum_of,
};
use sisyphium_tabula::run;
use std::{
    ffi::OsString,
    fs,
    io::{self, Write},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static SERIAL: AtomicU64 = AtomicU64::new(0);
struct Locus(PathBuf);
impl Locus {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "sisyphium-tabula-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Locus {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}
fn scenario() -> Vec<OsString> {
    args(&["--epoch", "1782054000000", "--ticks", "4096", "--seed", "1"])
}

#[test]
fn cli_static_parity() {
    let dir = Locus::new();
    let state = dir.0.join("state");
    let mut options = scenario();
    options.extend([
        OsString::from("--state"),
        state.clone().into_os_string(),
        OsString::from("--format"),
        OsString::from("ppm"),
    ]);
    let mut output = Vec::new();
    run(&options, &mut output).unwrap();
    let host = Facultates {
        max_hz: 0,
        persistence: true,
        ..Facultates::default()
    };
    let measures = Indicia {
        epoch_ms: 1_782_054_000_000,
        ..Indicia::default()
    };
    let mut world = Mundus::genesis(1);
    let previous = consilium(&host, &world, 255);
    assert!(previous.interval_ms >= 60_000);
    world.advance(&host, &measures, 4096, previous.tick_ms);
    assert_eq!(fs::read(&state).unwrap(), world.format());
    let regimen = consilium(&host, &world, 255);
    let mut pixels = vec![0; 320 * 180];
    world.render(
        &host,
        &measures,
        &regimen,
        &mut Tabula::new(320, 180, &mut pixels).unwrap(),
    );
    let palette = world.palette(&host, &measures, &regimen);
    let rgb: Vec<u8> = pixels
        .iter()
        .flat_map(|&pixel| palette[usize::from(lum_of(pixel))])
        .collect();
    assert_eq!(&output[..15], b"P6\n320 180\n255\n");
    assert_eq!(&output[15..], rgb);
}

#[test]
fn default_png_and_synthetic_seed_are_repeatable() {
    let options = args(&["--epoch", "-1", "--ticks", "0"]);
    let mut first = Vec::new();
    let mut second = Vec::new();
    run(&options, &mut first).unwrap();
    run(&options, &mut second).unwrap();
    assert_eq!(first, second);
    assert_eq!(&first[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(&first[16..24], &[0, 0, 1, 64, 0, 0, 0, 180]);
    let mut explicit = options;
    explicit.extend(args(&["--seed", "18446744073709551615"]));
    second.clear();
    run(&explicit, &mut second).unwrap();
    assert_eq!(first, second);
}

struct Defectus(io::ErrorKind);
impl Write for Defectus {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::from(self.0))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[test]
fn image_failure_does_not_save_state_and_keeps_error_origin() {
    let dir = Locus::new();
    let path = dir.0.join("state");
    let initial = Mundus::genesis(1).format();
    fs::write(&path, initial).unwrap();
    let mut options = args(&["--epoch", "1782054000000", "--ticks", "4096", "--state"]);
    options.push(path.into_os_string());
    for kind in [io::ErrorKind::BrokenPipe, io::ErrorKind::InvalidInput] {
        assert_eq!(
            run(&options, &mut Defectus(kind)).unwrap_err().exit_code(),
            1
        );
        assert_eq!(fs::read(dir.0.join("state")).unwrap(), initial);
    }
}
#[test]
fn invalid_usage_has_no_output_or_state() {
    for values in [
        vec!["--width", "0"],
        vec!["--height", "65536"],
        vec!["--width", "65535", "--height", "65535"],
        vec!["--epoch", "1"],
        vec!["--ticks", "65537"],
        vec!["--metire"],
        vec!["--stilus", "creta"],
        vec!["--format", "jpeg"],
        vec!["--energy", "256"],
        vec!["--width", "1", "--width", "2"],
    ] {
        let mut output = Vec::new();
        assert_eq!(run(&args(&values), &mut output).unwrap_err().exit_code(), 2);
        assert!(output.is_empty());
    }
}
#[test]
fn invalid_state_and_aliases_remain_intact() {
    let dir = Locus::new();
    let state = dir.0.join("state");
    fs::write(&state, b"corrupt").unwrap();
    let mut options = args(&["--state"]);
    options.push(state.clone().into_os_string());
    assert_eq!(run(&options, &mut Vec::new()).unwrap_err().exit_code(), 1);
    assert_eq!(fs::read(&state).unwrap(), b"corrupt");
    fs::write(&state, Mundus::genesis(1).format()).unwrap();
    for output in [
        state.clone(),
        dir.0.join("./state"),
        dir.0.join("state.lock"),
        dir.0.join("missing/image.png"),
    ] {
        let mut options = options.clone();
        options.extend([OsString::from("--output"), output.into_os_string()]);
        assert!(run(&options, &mut Vec::new()).is_err());
        assert_eq!(fs::read(&state).unwrap(), Mundus::genesis(1).format());
    }
    assert!(!dir.0.join("state.lock").exists());
}

#[test]
fn help_version_and_file_output_do_not_pollute_stdout() {
    let dir = Locus::new();
    for flag in ["--help", "--version"] {
        let mut options = args(&[flag, "--state"]);
        options.push(dir.0.join("absent/state").into_os_string());
        let mut output = Vec::new();
        run(&options, &mut output).unwrap();
        assert!(!output.is_empty());
        assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 0);
    }
    let image = dir.0.join("explicit.ppm");
    fs::write(&image, b"previous image").unwrap();
    let mut options = scenario();
    options.extend([OsString::from("--output"), image.clone().into_os_string()]);
    let mut output = Vec::new();
    run(&options, &mut output).unwrap();
    assert!(output.is_empty());
    assert_eq!(&fs::read(&image).unwrap()[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
}

#[test]
fn explicit_ticks_override_real_mtime_and_future_time_is_conservative() {
    use std::time::{Duration, SystemTime};
    let dir = Locus::new();
    let path = dir.0.join("state");
    let initial = Mundus::genesis(1).format();
    fs::write(&path, initial).unwrap();
    let future = SystemTime::now() + Duration::from_secs(3600);
    fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(future))
        .unwrap();
    let original_mtime = fs::metadata(&path).unwrap().modified().unwrap();
    let mut options = args(&["--state"]);
    options.push(path.clone().into_os_string());
    run(&options, &mut Vec::new()).unwrap();
    assert_eq!(fs::read(&path).unwrap(), initial);
    assert_eq!(
        fs::metadata(&path).unwrap().modified().unwrap(),
        original_mtime
    );
    options.extend(args(&["--ticks", "4096"]));
    run(&options, &mut Vec::new()).unwrap();
    assert_ne!(fs::read(&path).unwrap(), initial);
    assert_eq!(fs::read(&path).unwrap().len(), 32);
}

#[test]
fn existing_state_and_seed_conflict_preserve_state() {
    let dir = Locus::new();
    let path = dir.0.join("state");
    let initial = Mundus::genesis(1).format();
    fs::write(&path, initial).unwrap();
    let mut options = scenario();
    options.extend([OsString::from("--state"), path.clone().into_os_string()]);
    let mut output = Vec::new();
    assert_eq!(run(&options, &mut output).unwrap_err().exit_code(), 1);
    assert!(output.is_empty());
    assert_eq!(fs::read(&path).unwrap(), initial);
}

#[cfg(unix)]
#[test]
fn symlinks_and_parent_aliases_are_rejected_before_state_access() {
    use std::os::unix::fs::symlink;
    let dir = Locus::new();
    let state = dir.0.join("state");
    let initial = Mundus::genesis(1).format();
    fs::write(&state, initial).unwrap();
    let alias = dir.0.join("parent");
    symlink(&dir.0, &alias).unwrap();
    let target = dir.0.join("link");
    symlink(&state, &target).unwrap();
    let dangling = dir.0.join("dangling");
    symlink(dir.0.join("absent"), &dangling).unwrap();
    for path in [
        alias.join("state"),
        alias.join("state.lock"),
        target,
        dangling,
    ] {
        let options = vec![
            OsString::from("--state"),
            state.clone().into_os_string(),
            OsString::from("--output"),
            path.into_os_string(),
        ];
        assert_eq!(run(&options, &mut Vec::new()).unwrap_err().exit_code(), 1);
        assert_eq!(fs::read(&state).unwrap(), initial);
        assert!(!dir.0.join("state.lock").exists());
    }
    let options = vec![
        OsString::from("--state"),
        dir.0.join("link").into_os_string(),
    ];
    assert_eq!(run(&options, &mut Vec::new()).unwrap_err().exit_code(), 1);
}

#[cfg(unix)]
#[test]
fn non_utf8_paths_survive_and_non_utf8_numbers_are_usage_errors() {
    use std::os::unix::ffi::OsStringExt;
    let dir = Locus::new();
    let state = dir.0.join(OsString::from_vec(b"state\xff".to_vec()));
    let output = dir.0.join(OsString::from_vec(b"image\xfe".to_vec()));
    let mut options = scenario();
    options.extend([
        OsString::from("--state"),
        state.clone().into_os_string(),
        OsString::from("--output"),
        output.clone().into_os_string(),
    ]);
    let mut parsed =
        sisyphium_hospes::argumenta::Argumenta::parse(&options, &[("--output", true)]).unwrap();
    assert_eq!(
        parsed.take("--output"),
        Some(output.clone().into_os_string())
    );
    assert_eq!(
        sisyphium_hospes::argumenta::Communia::parse(&mut parsed)
            .unwrap()
            .archivum,
        Some(state.clone())
    );
    // Some Unix filesystems reject non-UTF-8 names. Preserve their error, never lossy-convert.
    let probe = fs::File::create(&output);
    match probe {
        Ok(file) => {
            drop(file);
            run(&options, &mut Vec::new()).unwrap();
            assert_eq!(fs::read(state).unwrap().len(), 32);
            assert_eq!(&fs::read(output).unwrap()[..8], b"\x89PNG\r\n\x1a\n");
        }
        Err(native) => {
            let error = run(&options, &mut Vec::new()).unwrap_err();
            assert_eq!(error.exit_code(), 1);
            if let sisyphium_hospes::Error::Operatio(error) = error {
                assert_eq!(error.raw_os_error(), native.raw_os_error());
            }
        }
    }
    assert_eq!(
        run(
            &[OsString::from("--width"), OsString::from_vec(vec![255])],
            &mut Vec::new()
        )
        .unwrap_err()
        .exit_code(),
        2
    );
}

#[test]
fn binary_broken_pipe_is_operational_and_preserves_state() {
    use std::process::{Command, Stdio};
    let dir = Locus::new();
    let path = dir.0.join("state");
    let initial = Mundus::genesis(1).format();
    fs::write(&path, initial).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_sisyphium-tabula"))
        .args(["--ticks", "4096", "--state"])
        .arg(&path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let result = child.wait_with_output().unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(!result.stderr.is_empty());
    assert_eq!(fs::read(&path).unwrap(), initial);
}

struct Mutatio {
    via: PathBuf,
    litterae: Vec<u8>,
}
impl Write for Mutatio {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.litterae.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        fs::write(&self.via, Mundus::genesis(2).format())
    }
}
#[test]
fn state_conflict_after_image_publication_reports_without_rollback() {
    let dir = Locus::new();
    let path = dir.0.join("state");
    fs::write(&path, Mundus::genesis(1).format()).unwrap();
    let mut options = args(&["--ticks", "4096", "--state"]);
    options.push(path.clone().into_os_string());
    let mut writer = Mutatio {
        via: path.clone(),
        litterae: Vec::new(),
    };
    assert_eq!(run(&options, &mut writer).unwrap_err().exit_code(), 1);
    assert_eq!(&writer.litterae[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(
        &writer.litterae[writer.litterae.len() - 8..writer.litterae.len() - 4],
        b"IEND"
    );
    assert_eq!(fs::read(path).unwrap(), Mundus::genesis(2).format());
}

#[test]
fn scarce_energy_reselects_quies_after_public_advance() {
    let dir = Locus::new();
    let path = dir.0.join("state");
    let mut options = scenario();
    options.extend(args(&["--energy", "0", "--format", "ppm", "--state"]));
    options.push(path.clone().into_os_string());
    let mut output = Vec::new();
    run(&options, &mut output).unwrap();
    let host = Facultates {
        persistence: true,
        max_hz: 0,
        ..Facultates::default()
    };
    let measures = Indicia {
        energy: 0,
        epoch_ms: 1_782_054_000_000,
        ..Indicia::default()
    };
    let mut world = Mundus::genesis(1);
    let previous = consilium(&host, &world, 0);
    world.advance(&host, &measures, 4096, previous.tick_ms);
    assert_eq!(world.verbum.modus(), sisyphium::verbum::Modus::Quies);
    assert_eq!(fs::read(path).unwrap(), world.format());
    let regimen = consilium(&host, &world, 0);
    assert_eq!(regimen.tick_ms, 60_000);
    assert!(regimen.interval_ms >= 60_000);
    let mut pixels = vec![0; 320 * 180];
    world.render(
        &host,
        &measures,
        &regimen,
        &mut Tabula::new(320, 180, &mut pixels).unwrap(),
    );
    let palette = world.palette(&host, &measures, &regimen);
    let rgb: Vec<u8> = pixels
        .iter()
        .flat_map(|&pixel| palette[usize::from(lum_of(pixel))])
        .collect();
    assert_eq!(&output[15..], rgb);
}

#[test]
fn native_name_aliases_preserve_state_and_lock_namespace() {
    let dir = Locus::new();
    fs::write(dir.0.join("caseprobe"), b"probe").unwrap();
    let case_insensitive = dir.0.join("CASEPROBE").exists();
    fs::remove_file(dir.0.join("caseprobe")).unwrap();
    if !case_insensitive {
        return;
    }
    for (state_name, output_name, existing_state, existing_output) in [
        ("state", "STATE", true, false),
        ("state", "STATE.LOCK", true, false),
        ("state", "STATE", false, false),
        ("state", "STATE.LOCK", false, false),
        ("state", "STATE", false, true),
    ] {
        let case = Locus::new();
        let state = case.0.join(state_name);
        let output = case.0.join(output_name);
        let original = Mundus::genesis(1).format();
        if existing_state {
            fs::write(&state, original).unwrap();
        }
        if existing_output {
            fs::write(&output, original).unwrap();
        }
        let mut options = args(&[
            "--epoch",
            "1782054000000",
            "--ticks",
            "1",
            "--width",
            "1",
            "--height",
            "1",
            "--state",
        ]);
        options.extend([
            state.clone().into_os_string(),
            OsString::from("--output"),
            output.into_os_string(),
        ]);
        assert_eq!(run(&options, &mut Vec::new()).unwrap_err().exit_code(), 1);
        if existing_state || existing_output {
            assert_eq!(fs::read(&state).unwrap(), original);
        } else {
            assert!(!state.exists());
        }
        assert!(!case.0.join("state.lock").exists());
        assert_eq!(
            fs::read_dir(&case.0).unwrap().count(),
            usize::from(existing_state || existing_output)
        );
    }
}

#[cfg(unix)]
#[test]
fn existing_hardlink_aliases_preserve_both_names() {
    let dir = Locus::new();
    let state = dir.0.join("state");
    let output = dir.0.join("image");
    let original = Mundus::genesis(1).format();
    fs::write(&state, original).unwrap();
    fs::hard_link(&state, &output).unwrap();
    let options = vec![
        OsString::from("--state"),
        state.clone().into_os_string(),
        OsString::from("--output"),
        output.clone().into_os_string(),
    ];
    assert_eq!(run(&options, &mut Vec::new()).unwrap_err().exit_code(), 1);
    assert_eq!(fs::read(&state).unwrap(), original);
    assert_eq!(fs::read(&output).unwrap(), original);
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 2);
}

#[test]
fn native_unicode_equivalence_is_checked_for_absent_destinations() {
    let dir = Locus::new();
    let composed = "caf\u{e9}";
    let decomposed = "cafe\u{301}";
    fs::write(dir.0.join(composed), b"probe").unwrap();
    let equivalent = dir.0.join(decomposed).exists();
    fs::remove_file(dir.0.join(composed)).unwrap();
    if !equivalent {
        return;
    }
    for lock in [false, true] {
        let state = dir.0.join(composed);
        let output = dir.0.join(if lock {
            format!("{decomposed}.lock")
        } else {
            decomposed.to_owned()
        });
        let options = vec![
            OsString::from("--state"),
            state.into_os_string(),
            OsString::from("--output"),
            output.into_os_string(),
        ];
        assert_eq!(run(&options, &mut Vec::new()).unwrap_err().exit_code(), 1);
        assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 0);
    }
}

#[cfg(unix)]
#[test]
fn probe_error_cleans_only_owned_files_and_preserves_state() {
    let dir = Locus::new();
    // Valid final name, but its additional native probe prefix exceeds NAME_MAX.
    let state = dir.0.join("s".repeat(250));
    let original = Mundus::genesis(1).format();
    fs::write(&state, original).unwrap();
    let options = vec![
        OsString::from("--state"),
        state.clone().into_os_string(),
        OsString::from("--output"),
        dir.0.join("image").into_os_string(),
    ];
    assert_eq!(run(&options, &mut Vec::new()).unwrap_err().exit_code(), 1);
    assert_eq!(fs::read(&state).unwrap(), original);
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
}
