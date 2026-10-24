//! Shared native host laws (REQ-C-05, REQ-H-06, REQ-D-02).
#![forbid(unsafe_code)]

pub mod archivum;
pub mod argumenta;
pub mod electio;
pub mod tempus;

use std::{fmt, io};

/// Maximum raster area: 32 Mi pixels, one byte per pixel (REQ-H-02).
pub const MAX_PIXELS: usize = 33_554_432;
/// Maximum native terminal area: 1 Mi cells (REQ-H-02).
pub const MAX_CELLS: usize = 1_048_576;
/// Maximum encoded terminal frame: 64 Mi bytes (REQ-H-02).
pub const MAX_TERMINAL_BYTES: usize = 67_108_864;

/// Failure classified by origin, independent of its I/O kind (REQ-H-01).
#[derive(Debug)]
pub enum Error {
    /// Invalid command-line usage.
    Usus(io::Error),
    /// Failure while performing an operation.
    Operatio(io::Error),
}
impl Error {
    /// Returns status 2 for usage and 1 for operations (REQ-H-01).
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::Usus(_) => 2,
            Self::Operatio(_) => 1,
        }
    }
}
impl fmt::Display for Error {
    /// Displays the original diagnostic (REQ-H-01).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usus(e) | Self::Operatio(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for Error {
    /// Retains the original I/O source (REQ-H-01).
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Usus(e) | Self::Operatio(e) => Some(e),
        }
    }
}

/// Checks positive dimensions and bounded multiplication before allocation (REQ-H-02).
///
/// # Errors
/// Returns `InvalidInput` for zero dimensions, overflow or an area above `limit`.
pub fn validate_area(width: u16, height: u16, limit: usize) -> io::Result<usize> {
    usize::from(width)
        .checked_mul(usize::from(height))
        .filter(|&area| area > 0 && area <= limit)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "dimensions exceed the positive area limit",
            )
        })
}
