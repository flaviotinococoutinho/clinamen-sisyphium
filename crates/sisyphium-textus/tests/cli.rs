//! Finite terminal CLI and persistence contracts (REQ-H-01, REQ-D-02).
use sisyphium::mundus::Mundus;
use sisyphium_hospes::Error;
use std::{
    ffi::OsString,
    fs,
    io::{self, Write},
    path::PathBuf,
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

/// Allocates distinct fixture directories without changing process-global environment (REQ-D-02).
static SERIAL: AtomicU64 = AtomicU64::new(0);
struct Locus(PathBuf);
impl Locus {
    /// Owns a fresh temporary test directory (REQ-D-02).
    fn new() -> io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "sisyphium-textus-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }
}
impl Drop for Locus {
    /// Cleans only the exclusively created fixture directory (REQ-D-02).
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Invokes the actual executable with captured output and no interactive input (REQ-H-01).
fn invoke(args: &[&str]) -> io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_sisyphium-textus"))
        .args(args)
        .stdin(Stdio::null())
        .output()
}

/// Pipe defaults and explicit finite frames terminate with unchanged row geometry (REQ-H-02).
#[test]
fn finite_frames_and_pipe() -> io::Result<()> {
    let output = invoke(&["--plain", "--frames", "2", "--cols", "40", "--rows", "12"])?;
    assert!(output.status.success(), "{:?}", output.stderr);
    assert!(!output.stdout.contains(&27));
    let text = String::from_utf8(output.stdout).map_err(io::Error::other)?;
    assert_eq!(text.lines().count(), 25);
    assert_eq!(
        text.lines()
            .filter(|line| line.chars().count() == 40)
            .count(),
        24
    );
    let output = invoke(&["--cols", "9", "--rows", "3"])?;
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout.clone())
            .map_err(io::Error::other)?
            .lines()
            .count(),
        3
    );
    assert!(!output.stdout.contains(&27));
    let blocks = invoke(&[
        "--cells",
        "false",
        "--cols",
        "9",
        "--rows",
        "3",
        "--seed",
        "17",
        "--epoch",
        "1782054000000",
        "--ticks",
        "0",
        "--frames",
        "1",
    ])?;
    assert!(blocks.status.success());
    assert!(!blocks.stdout.contains(&27));
    let text = String::from_utf8(blocks.stdout).map_err(io::Error::other)?;
    assert_eq!(text.lines().count(), 3);
    assert!(text.lines().all(|line| line.chars().count() == 9));
    assert!(text.chars().all(|glyph| " ▀▄█\n".contains(glyph)));
    assert!(text.chars().any(|glyph| glyph != '▀' && glyph != '\n'));
    Ok(())
}

/// Synthetic time is repeatable and never starts a mixed civil-clock animation (REQ-H-06).
#[test]
fn synthetic_epoch_and_usage() -> io::Result<()> {
    let args = [
        "--epoch",
        "1700000000000",
        "--ticks",
        "3",
        "--frames",
        "1",
        "--cols",
        "40",
        "--rows",
        "12",
    ];
    let first = invoke(&args)?;
    let second = invoke(&args)?;
    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    for args in [
        vec!["--frames", "0"],
        vec!["--cols", "0"],
        vec!["--max-hz", "25"],
        vec!["--stilus", "littera"],
        vec!["--epoch", "0", "--ticks", "0"],
        vec!["--epoch", "0", "--ticks", "0", "--frames", "2"],
        vec!["--plain", "--plain"],
    ] {
        let output = invoke(&args)?;
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert_eq!(output.stdout, [] as [u8; 0]);
    }
    Ok(())
}

/// Invalid dimensions and help/version cannot create state artifacts (REQ-H-01).
#[test]
fn bounds_and_information_have_no_state_effects() -> io::Result<()> {
    let directory = Locus::new()?;
    let path = directory.0.join("world");
    for args in [
        vec!["--cols", "65535", "--rows", "65535"],
        vec!["--cols", "1", "--rows", "65535", "--cells", "false"],
        vec!["--help"],
        vec!["--version"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_sisyphium-textus"))
            .args(&args)
            .arg("--state")
            .arg(&path)
            .stdin(Stdio::null())
            .output()?;
        assert_eq!(
            output.status.code(),
            Some(if args[0] == "--help" || args[0] == "--version" {
                0
            } else {
                2
            })
        );
        assert!(!path.exists());
        assert_eq!(fs::read_dir(&directory.0)?.count(), 0);
    }
    Ok(())
}

/// A writer witnesses that presentation itself does not publish state (REQ-D-02).
struct Observator {
    via: PathBuf,
    defectus: bool,
    influxus: bool,
    scriptiones: usize,
}
impl Write for Observator {
    /// Counts actual output writes and optionally interrupts stdout (REQ-D-02).
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        assert!(!self.via.exists(), "a frame must not trigger persistence");
        self.scriptiones += 1;
        if self.defectus {
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "test output closed",
            ))
        } else {
            Ok(bytes.len())
        }
    }
    /// Injects an independent flush failure after successful frame output (REQ-D-02).
    fn flush(&mut self) -> io::Result<()> {
        if self.influxus {
            Err(io::Error::other("test flush failed"))
        } else {
            Ok(())
        }
    }
}

/// Exit and both output failure boundaries persist in normal execution context (REQ-D-02).
#[test]
fn exit_and_output_failure_persist() -> io::Result<()> {
    for (fail_write, fail_flush) in [(false, false), (true, false), (false, true)] {
        let directory = Locus::new()?;
        let path = directory.0.join("world");
        let mut args = [
            "--plain", "--frames", "1", "--cols", "4", "--rows", "2", "--epoch", "0", "--ticks",
            "0", "--seed", "17", "--state",
        ]
        .map(OsString::from)
        .to_vec();
        args.push(path.as_os_str().to_owned());
        let mut output = Observator {
            via: path.clone(),
            defectus: fail_write,
            influxus: fail_flush,
            scriptiones: 0,
        };
        let result = sisyphium_textus::run(&args, &mut output);
        assert_eq!(result.is_ok(), !fail_write && !fail_flush);
        if let Err(error) = result {
            assert!(matches!(error, Error::Operatio(_)));
        }
        assert_eq!(output.scriptiones, 1);
        assert_eq!(fs::read(path)?, Mundus::genesis(17).format());
        assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
    }
    Ok(())
}

/// Two finite frames with initial ticks zero leave a parseable controlled-exit snapshot (REQ-D-02).
#[test]
fn finite_run_persists_valid_state() -> io::Result<()> {
    let directory = Locus::new()?;
    let path = directory.0.join("world");
    let output = Command::new(env!("CARGO_BIN_EXE_sisyphium-textus"))
        .args([
            "--plain", "--frames", "2", "--cols", "4", "--rows", "2", "--ticks", "0", "--seed",
            "17", "--state",
        ])
        .arg(&path)
        .stdin(Stdio::null())
        .output()?;
    assert!(output.status.success(), "{:?}", output.stderr);
    let bytes = fs::read(&path)?;
    assert_eq!(bytes.len(), 32);
    assert!(Mundus::parse(&bytes).is_some());
    assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
    Ok(())
}

/// Native probes are bounded in count, tolerate absence, and explicit dimensions bypass them (REQ-H-02).
#[cfg(unix)]
#[test]
fn native_probe_is_cached_and_optional() -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let directory = Locus::new()?;
    let script = directory.0.join("stty");
    let log = directory.0.join("calls");
    fs::write(
        &script,
        "#!/bin/sh\nprintf 'probe\\n' >> \"$PROBE_LOG\"\nprintf '4 7\\n'\n",
    )?;
    fs::set_permissions(&script, fs::Permissions::from_mode(0o700))?;
    let output = Command::new(env!("CARGO_BIN_EXE_sisyphium-textus"))
        .args(["--plain", "--frames", "2"])
        .env("PATH", &directory.0)
        .env("PROBE_LOG", &log)
        .stdin(Stdio::null())
        .output()?;
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).map_err(io::Error::other)?;
    assert_eq!(text.lines().count(), 7);
    assert_eq!(fs::read_to_string(&log)?.lines().count(), 1);
    fs::remove_file(&log)?;
    let output = Command::new(env!("CARGO_BIN_EXE_sisyphium-textus"))
        .args(["--cols", "3", "--rows", "2"])
        .env("PATH", &directory.0)
        .env("PROBE_LOG", &log)
        .stdin(Stdio::null())
        .output()?;
    assert!(output.status.success());
    assert!(!log.exists());
    fs::remove_file(script)?;
    let output = Command::new(env!("CARGO_BIN_EXE_sisyphium-textus"))
        .env("PATH", &directory.0)
        .env("COLUMNS", "5")
        .env("LINES", "3")
        .stdin(Stdio::null())
        .output()?;
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout)
            .map_err(io::Error::other)?
            .lines()
            .count(),
        2
    );
    Ok(())
}

/// Publishes a competing snapshot after this invocation has read its own snapshot (REQ-D-02).
struct Mutator {
    via: PathBuf,
    novum: [u8; 32],
}
impl Write for Mutator {
    /// Simulates an independent writer before controlled exit persistence (REQ-D-02).
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        fs::write(&self.via, self.novum)?;
        Ok(bytes.len())
    }
    /// Completes output without modifying the competing snapshot (REQ-D-02).
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Exit conflict is an operational failure and never overwrites a competing snapshot (REQ-D-02).
#[test]
fn exit_conflict_preserves_competing_state() -> io::Result<()> {
    let directory = Locus::new()?;
    let path = directory.0.join("world");
    let mut args = [
        "--plain", "--frames", "1", "--cols", "4", "--rows", "2", "--epoch", "0", "--ticks", "0",
        "--seed", "17", "--state",
    ]
    .map(OsString::from)
    .to_vec();
    args.push(path.as_os_str().to_owned());
    let competing = Mundus::genesis(29).format();
    let mut output = Mutator {
        via: path.clone(),
        novum: competing,
    };
    assert!(matches!(
        sisyphium_textus::run(&args, &mut output),
        Err(Error::Operatio(_))
    ));
    assert_eq!(fs::read(path)?, competing);
    assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
    Ok(())
}
