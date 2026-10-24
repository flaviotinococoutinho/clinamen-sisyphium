//! Unique command-line options and shared host declarations (REQ-C-01, REQ-H-06).
use sisyphium::hospes::{Facultates, Indicia, Lingua};
use std::{collections::BTreeMap, ffi::OsString, io, path::PathBuf, str::FromStr};

/// Shared value-bearing options of the native hosts (REQ-C-01).
const COMMON: &[(&str, bool)] = &[
    ("--epoch", true),
    ("--ticks", true),
    ("--energy", true),
    ("--seism", true),
    ("--lingua", true),
    ("--seed", true),
    ("--state", true),
    ("--compute", true),
    ("--color-bits", true),
    ("--tz-offset", true),
    ("--australis", true),
];

/// Recognized options awaiting consumption (REQ-H-01).
#[derive(Debug)]
pub struct Argumenta {
    residua: BTreeMap<String, OsString>,
}
impl Argumenta {
    /// Parses unique common and host-specific flags, excluding the program name (REQ-H-01).
    /// Names in `allowed` include `--`; true requires a following value.
    ///
    /// # Errors
    /// Returns `InvalidInput` for unknown, repeated or incomplete arguments.
    pub fn parse(args: &[OsString], allowed: &[(&str, bool)]) -> io::Result<Self> {
        let mut options = BTreeMap::new();
        let mut iter = args.iter();
        while let Some(argument) = iter.next() {
            let name = argument
                .to_str()
                .ok_or_else(|| invalid("option name is not UTF-8"))?;
            let (_, has_value) = COMMON
                .iter()
                .chain(allowed)
                .find(|(key, _)| *key == name)
                .ok_or_else(|| invalid(format!("unknown option: {name}")))?;
            if options.contains_key(name) {
                return Err(invalid(format!("repeated option: {name}")));
            }
            let value = if *has_value {
                iter.next()
                    .ok_or_else(|| invalid(format!("missing value: {name}")))?
                    .clone()
            } else {
                OsString::new()
            };
            options.insert(name.to_owned(), value);
        }
        Ok(Self { residua: options })
    }

    /// Removes one option; a switch without a value yields an empty string (REQ-H-01).
    pub fn take(&mut self, name: &str) -> Option<OsString> {
        self.residua.remove(name)
    }

    /// Verifies that the selected host consumed every option (REQ-H-01).
    ///
    /// # Errors
    /// Returns `InvalidInput` for any unconsumed option.
    pub fn finish(self) -> io::Result<()> {
        if let Some(name) = self.residua.keys().next() {
            return Err(invalid(format!("unused option: {name}")));
        }
        Ok(())
    }
}

/// Common command-line values, without filesystem effects (REQ-C-01).
#[derive(Debug)]
pub struct Communia {
    /// Optional synthetic epoch in milliseconds.
    pub epocha: Option<i64>,
    /// Optional explicit catch-up count, 0..=65536.
    pub ictus: Option<u32>,
    /// Available energy, default 255.
    pub energia: u8,
    /// Declared agitation, default zero.
    pub sismus: u8,
    /// Language of the experience.
    pub lingua: Lingua,
    /// Optional explicit genesis seed.
    pub semen: Option<u64>,
    /// Optional state path; its bytes need not be UTF-8.
    pub archivum: Option<PathBuf>,
    /// Relative compute, default 255.
    pub computatio: u8,
    /// Color depth, 1, 4, 8 or 24.
    pub profunditas: u8,
    /// UTC offset in minutes, -1439..=1439.
    pub meridianus: i16,
    /// Southern hemisphere flag.
    pub australis: bool,
}
impl Communia {
    /// Consumes and validates common flags (REQ-C-01, REQ-H-06).
    ///
    /// # Errors
    /// Returns `InvalidInput` for malformed values or an epoch without explicit ticks.
    pub fn parse(args: &mut Argumenta) -> io::Result<Self> {
        let epoch = number(args, "--epoch")?;
        let ticks = number(args, "--ticks")?;
        if epoch.is_some() && ticks.is_none() {
            return Err(invalid("--epoch requires --ticks"));
        }
        if ticks.is_some_and(|ticks| ticks > 65_536) {
            return Err(invalid("--ticks must be 0..=65536"));
        }
        let lingua = match args.take("--lingua") {
            None => Lingua::En,
            Some(value) => match value.to_str() {
                Some("en") => Lingua::En,
                Some("pt") => Lingua::Pt,
                _ => return Err(invalid("--lingua must be en or pt")),
            },
        };
        let depth = number(args, "--color-bits")?.unwrap_or(8);
        if ![1, 4, 8, 24].contains(&depth) {
            return Err(invalid("--color-bits must be 1, 4, 8 or 24"));
        }
        let offset = number(args, "--tz-offset")?.unwrap_or(-180);
        if !(-1439..=1439).contains(&offset) {
            return Err(invalid("--tz-offset must be -1439..=1439"));
        }
        Ok(Self {
            epocha: epoch,
            ictus: ticks,
            energia: number(args, "--energy")?.unwrap_or(255),
            sismus: number(args, "--seism")?.unwrap_or(0),
            lingua,
            semen: number(args, "--seed")?,
            archivum: args.take("--state").map(PathBuf::from),
            computatio: number(args, "--compute")?.unwrap_or(255),
            profunditas: depth,
            meridianus: offset,
            australis: number(args, "--australis")?.unwrap_or(true),
        })
    }

    /// Declares supplied capabilities, leaving unmeasured capabilities false (REQ-C-01).
    #[must_use]
    pub fn facultates(&self, width: u16, height: u16, cells: bool, max_hz: u8) -> Facultates {
        Facultates {
            width,
            height,
            cells,
            max_hz,
            color_bits: self.profunditas,
            persistence: self.archivum.is_some(),
            australis: self.australis,
            tz_offset_min: self.meridianus,
            lingua: self.lingua,
            compute: self.computatio,
            ..Facultates::default()
        }
    }

    /// Supplies the caller's clock and declared measures, retaining unknowns (REQ-H-01).
    #[must_use]
    pub fn indicia(&self, epoch_ms: i64) -> Indicia {
        Indicia {
            epoch_ms,
            energy: self.energia,
            seism: self.sismus,
            ..Indicia::default()
        }
    }
}

/// Converts one optional UTF-8 scalar without accepting invalid numeric bytes (REQ-H-01).
fn number<T: FromStr>(args: &mut Argumenta, name: &str) -> io::Result<Option<T>> {
    args.take(name)
        .map(|value| {
            value
                .to_str()
                .and_then(|text| text.parse().ok())
                .ok_or_else(|| invalid(format!("invalid value: {name}")))
        })
        .transpose()
}

/// Constructs a usage diagnostic at the parsing boundary (REQ-H-01).
fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}
