//! Static image host consuming the public world laws (REQ-H-02, REQ-H-06).
#![forbid(unsafe_code)]
pub mod imago;
pub mod metire;
mod via;

use imago::{Forma, write_image};
use sisyphium::{
    hospes::{Stilus, Tabula},
    mundus::Mundus,
};
use sisyphium_hospes::{
    Error, MAX_PIXELS,
    archivum::{Archivum, write_atomic},
    argumenta::{Argumenta, Communia},
    electio::Electio,
    tempus::{elapsed_ticks, epoch_ms},
    validate_area,
};
use std::{
    ffi::OsString,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

/// Static host options; common options are recognized by Argumenta (REQ-H-01).
const OPTIONS: &[(&str, bool)] = &[
    ("--width", true),
    ("--height", true),
    ("--output", true),
    ("--format", true),
    ("--help", false),
    ("--version", false),
    ("--metire", false),
    ("--stilus", true),
];
/// Documents implemented flags without advertising future measurement modes (REQ-H-01).
const HELP: &str = "sisyphium-tabula [options]\n\
--width 1..65535       default 320\n\
--height 1..65535      default 180; area <= 33554432 pixels\n\
--output PATH|-       default - (stdout); existing parent required\n\
--format png|ppm      default png, independent of extension\n\
--epoch I64           epoch milliseconds; requires --ticks\n\
--ticks 0..65536      explicit catch-up; otherwise state mtime\n\
--seed U64            genesis seed; default effective epoch bits\n\
--state PATH          optional 32-byte state; conflicts with seed if existing\n\
--energy 0..255       default 255\n\
--seism 0..255        default 0\n\
--lingua en|pt        default en\n\
--compute 0..255      default 255\n\
--color-bits 1|4|8|24 default 8\n\
--tz-offset -1439..1439 default -180 minutes\n\
--australis true|false default true\n\
--metire              render-only diagnostic TSV; no experience flags\n\
--stilus NAME         with --metire only; otherwise measures all four\n\
--help                display usage\n\
--version             display version\n";

/// Parses, evolves, renders and publishes one static frame (REQ-H-06, REQ-D-02).
/// The argument slice excludes the executable name. Stdout contains image bytes only,
/// except explicit help/version. Image failure never commits state; a subsequent state
/// failure may coexist with an already published image and does not roll it back.
///
/// # Errors
/// Returns `Error::Usus` for parsing errors and `Error::Operatio` for operational failures.
pub fn run(args: &[OsString], output: &mut dyn Write) -> Result<(), Error> {
    let mut options = Argumenta::parse(args, OPTIONS).map_err(Error::Usus)?;
    let help = options.take("--help").is_some();
    let version = options.take("--version").is_some();
    if help || version {
        let text = if help {
            HELP
        } else {
            concat!("sisyphium-tabula ", env!("CARGO_PKG_VERSION"), "\n")
        };
        output
            .write_all(text.as_bytes())
            .and_then(|()| output.flush())
            .map_err(Error::Operatio)?;
        return Ok(());
    }
    let measure = options.take("--metire").is_some();
    let style = options.take("--stilus");
    if measure {
        let style = style
            .map(|value| match value.to_str() {
                Some("creta") => Ok(Stilus::Creta),
                Some("musivum") => Ok(Stilus::Musivum),
                Some("littera") => Ok(Stilus::Littera),
                Some("bicolor") => Ok(Stilus::Bicolor),
                _ => Err(Error::Usus(invalid("unknown measurement stilus"))),
            })
            .transpose()?;
        options.finish().map_err(Error::Usus)?;
        return metire::run(style, output).map_err(Error::Operatio);
    }
    if style.is_some() {
        return Err(Error::Usus(invalid("--stilus requires --metire")));
    }
    let common = Communia::parse(&mut options).map_err(Error::Usus)?;
    let width = dimension(options.take("--width"), 320).map_err(Error::Usus)?;
    let height = dimension(options.take("--height"), 180).map_err(Error::Usus)?;
    let area = validate_area(width, height, MAX_PIXELS).map_err(Error::Usus)?;
    let format = match options.take("--format") {
        None => Forma::Png,
        Some(value) if value == "png" => Forma::Png,
        Some(value) if value == "ppm" => Forma::Ppm,
        _ => return Err(Error::Usus(invalid("--format must be png or ppm"))),
    };
    let destination = options
        .take("--output")
        .filter(|value| value != "-")
        .map(PathBuf::from);
    options.finish().map_err(Error::Usus)?;
    validate_destinations(destination.as_deref(), common.archivum.as_deref())
        .map_err(Error::Operatio)?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(area)
        .map_err(|error| Error::Operatio(io::Error::other(error)))?;
    pixels.resize(area, 0);
    let now = common
        .epocha
        .or_else(|| epoch_ms(SystemTime::now()))
        .ok_or_else(|| Error::Operatio(io::Error::other("clock outside i64 milliseconds")))?;
    let fallback = u64::from_ne_bytes(now.to_ne_bytes());
    let (mut archive, mut world) = if let Some(path) = &common.archivum {
        let (archive, world) =
            Archivum::open(path, common.semen, fallback).map_err(Error::Operatio)?;
        (Some(archive), world)
    } else {
        (None, Mundus::genesis(common.semen.unwrap_or(fallback)))
    };
    let host = common.facultates(width, height, false, 0);
    let measures = common.indicia(now);
    let mut choice = Electio::new();
    let previous = choice.select(&host, &world, common.energia);
    let modified = archive
        .as_ref()
        .and_then(Archivum::mtime)
        .and_then(epoch_ms);
    let ticks = common
        .ictus
        .unwrap_or_else(|| elapsed_ticks(now, modified, previous.tick_ms));
    world.advance(&host, &measures, ticks, previous.tick_ms);
    let regimen = choice.select(&host, &world, common.energia);
    let mut raster = Tabula::new(width, height, &mut pixels)
        .ok_or_else(|| Error::Operatio(invalid("invalid raster")))?;
    world.render(&host, &measures, &regimen, &mut raster);
    let palette = world.palette(&host, &measures, &regimen);
    let write = |output: &mut dyn Write| {
        write_image(format, width, height, &pixels, &palette, output)?;
        output.flush()
    };
    if let Some(path) = destination {
        write_atomic(&path, |file| write(file)).map_err(Error::Operatio)?;
    } else {
        write(output).map_err(Error::Operatio)?;
    }
    if let Some(archive) = &mut archive {
        archive.save(&world).map_err(Error::Operatio)?;
    }
    Ok(())
}

/// Parses positive 16-bit dimensions before allocation (REQ-H-02).
fn dimension(value: Option<OsString>, fallback: u16) -> io::Result<u16> {
    value.map_or(Ok(fallback), |value| {
        value
            .to_str()
            .and_then(|value| value.parse::<u16>().ok())
            .filter(|&value| value > 0)
            .ok_or_else(|| invalid("dimension must be 1..65535"))
    })
}
/// Creates a diagnostic without conflating its kind with its origin (REQ-H-01).
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

/// Resolves only the parent, rejecting nonregular final components (REQ-D-02).
fn destination(path: &Path) -> io::Result<PathBuf> {
    let name = path
        .file_name()
        .ok_or_else(|| invalid("destination requires a file name"))?;
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let resolved = parent.canonicalize()?.join(name);
    match fs::symlink_metadata(&resolved) {
        Ok(metadata) if !metadata.is_file() || metadata.file_type().is_symlink() => {
            return Err(invalid(
                "destination must be a regular file, never a symlink",
            ));
        }
        Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
        _ => (),
    }
    Ok(resolved)
}
/// Refuses image aliases of state or its cooperative lock before opening state (REQ-D-02).
fn validate_destinations(output: Option<&Path>, state: Option<&Path>) -> io::Result<()> {
    let output = output.map(destination).transpose()?;
    let state = state.map(destination).transpose()?;
    if let (Some(output), Some(state)) = (output, state) {
        let mut lock = state.as_os_str().to_os_string();
        lock.push(".lock");
        if via::aliases(&output, &state)? || via::aliases(&output, Path::new(&lock))? {
            return Err(invalid("image aliases state or state lock"));
        }
    }
    Ok(())
}
