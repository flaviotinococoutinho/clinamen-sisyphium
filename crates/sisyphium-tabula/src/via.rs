//! Native destination equivalence before publication, in a private parent (REQ-D-02).
use std::{
    ffi::OsString,
    fs::{self, Metadata, OpenOptions},
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

/// Process-local names for exclusive, nonpersistent equivalence probes (REQ-D-02).
static SERIAL: AtomicU64 = AtomicU64::new(0);

/// Detects existing identity and native name equivalence, including absent destinations.
/// No lowercase or Unicode approximation substitutes for the filesystem's rules.
///
/// # Errors
/// Rejects failed metadata reads, unsupported probes and cleanup failures without fallback.
pub(crate) fn aliases(first: &Path, second: &Path) -> io::Result<bool> {
    if first == second || same_existing(first, second)? {
        return Ok(true);
    }
    let first_parent = first.parent().ok_or_else(invalid_path)?;
    let second_parent = second.parent().ok_or_else(invalid_path)?;
    if !same_existing(first_parent, second_parent)? {
        return Ok(false);
    }
    let first_name = first.file_name().ok_or_else(invalid_path)?;
    let second_name = second.file_name().ok_or_else(invalid_path)?;
    let prefix = format!(
        ".sisyphium-probe-{}-{}-",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    );
    let mut first_probe = OsString::from(&prefix);
    first_probe.push(first_name);
    let mut second_probe = OsString::from(prefix);
    second_probe.push(second_name);
    let first_guard = Custodia::create(first_parent.join(first_probe))?;
    let comparison = match Custodia::create(first_parent.join(second_probe)) {
        Ok(second_guard) => second_guard.release().map(|()| false),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(true),
        Err(error) => Err(error),
    };
    let cleanup = first_guard.release();
    match (comparison, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => Err(io::Error::new(
            error.kind(),
            format!("{error}; probe cleanup failed: {cleanup}"),
        )),
    }
}

/// Requires actual path components; public callers already canonicalize existing parents.
fn invalid_path() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "destination requires parent and name",
    )
}

/// Preserves all metadata errors except a genuinely absent destination (REQ-D-02).
fn existing(path: &Path) -> io::Result<Option<Metadata>> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// Compares Unix inode identity, including hardlinks; other hosts use native canonical paths.
fn same_existing(first: &Path, second: &Path) -> io::Result<bool> {
    let Some(first_metadata) = existing(first)? else {
        return Ok(false);
    };
    let Some(second_metadata) = existing(second)? else {
        return Ok(false);
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(first_metadata.dev() == second_metadata.dev()
            && first_metadata.ino() == second_metadata.ino())
    }
    #[cfg(not(unix))]
    {
        let _ = (first_metadata, second_metadata);
        Ok(first.canonicalize()? == second.canonicalize()?)
    }
}

/// Owns only a probe created exclusively by this invocation (REQ-D-02).
struct Custodia {
    via: Option<PathBuf>,
}
impl Custodia {
    /// Closes the empty file immediately so Windows can remove the owned name.
    fn create(path: PathBuf) -> io::Result<Self> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        drop(file);
        Ok(Self { via: Some(path) })
    }

    /// Reports failed cleanup and leaves Drop one final best-effort attempt.
    fn release(mut self) -> io::Result<()> {
        if let Some(path) = &self.via {
            fs::remove_file(path)?;
        }
        self.via = None;
        Ok(())
    }
}
impl Drop for Custodia {
    /// Removes no preexisting destination, even after a failed second probe.
    fn drop(&mut self) {
        if let Some(path) = &self.via {
            let _ = fs::remove_file(path);
        }
    }
}
