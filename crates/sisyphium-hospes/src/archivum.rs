//! Cooperative state publication without deleting the prior destination (REQ-D-02).
//!
//! A private parent directory and cooperative writers are required. Bytes plus mtime admit
//! ABA; absent mtime leaves byte comparison only. Locks left by abrupt termination require
//! manual recovery. No guarantee is made for hostile path changes, remote filesystems or power
//! loss. An error after rename can coexist with the new state already published.
use sisyphium::mundus::Mundus;
use std::{
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::SystemTime,
};

/// Process-local suffix for exclusive temporary names; never persisted (REQ-D-02).
static SERIAL: AtomicU64 = AtomicU64::new(0);

/// Bytes and optional modification time from the same opened file (REQ-D-02).
#[derive(Debug, PartialEq, Eq)]
struct Imago {
    litterae: [u8; 32],
    tempus: Option<SystemTime>,
}

/// State path and volatile expected snapshot for cooperative writers (REQ-D-02).
#[derive(Debug)]
pub struct Archivum {
    via: PathBuf,
    imago: Option<Imago>,
    incertum: bool,
}
impl Archivum {
    /// Reads a valid state under a short exclusive lock, or creates genesis in memory (REQ-D-02).
    ///
    /// # Errors
    /// Rejects symlinks, corrupt state, explicit seed with existing state, lock contention and I/O.
    pub fn open(path: &Path, seed: Option<u64>, fallback_seed: u64) -> io::Result<(Self, Mundus)> {
        let path = canonical_path(path)?;
        let lock = acquire(&path)?;
        let result = (|| {
            let snapshot = read_snapshot(&path)?;
            let world = if let Some(ref snapshot) = snapshot {
                if seed.is_some() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "--seed conflicts with existing state",
                    ));
                }
                Mundus::parse(&snapshot.litterae)
                    .ok_or_else(|| invalid_state("state is not hexadecimal"))?
            } else {
                Mundus::genesis(seed.unwrap_or(fallback_seed))
            };
            Ok((
                Self {
                    via: path,
                    imago: snapshot,
                    incertum: false,
                },
                world,
            ))
        })();
        finish(result, lock.release())
    }

    /// Returns the snapshot's filesystem time, without adding a persisted clock (REQ-H-06).
    #[must_use]
    pub fn mtime(&self) -> Option<SystemTime> {
        self.imago.as_ref().and_then(|snapshot| snapshot.tempus)
    }

    /// Confirms the expected snapshot and publishes exactly 32 lowercase digits (REQ-D-02).
    /// Equal bytes preserve the existing mtime. No retry or rollback follows an error.
    ///
    /// # Errors
    /// Reports conflicts, locks or I/O. A post-rename error explicitly reports possible publication
    /// and prevents further saves through this instance; recovery requires reopening the state.
    pub fn save(&mut self, world: &Mundus) -> io::Result<()> {
        self.save_with(world, || Ok(()), || Ok(()))
    }

    /// Applies the state transaction with private checks around publication (REQ-D-02).
    fn save_with(
        &mut self,
        world: &Mundus,
        before_sync: impl FnOnce() -> io::Result<()>,
        after_rename: impl FnOnce() -> io::Result<()>,
    ) -> io::Result<()> {
        if self.incertum {
            return Err(io::Error::other(
                "state publication uncertain; stop and inspect before reopening",
            ));
        }
        let lock = acquire(&self.via)?;
        let mut published = false;
        let result = (|| {
            if read_snapshot(&self.via)? != self.imago {
                return Err(io::Error::other(
                    "state conflict; another writer changed bytes or mtime",
                ));
            }
            let bytes = world.format();
            if self
                .imago
                .as_ref()
                .is_some_and(|snapshot| snapshot.litterae == bytes)
            {
                return Ok(());
            }
            publish(
                &self.via,
                |file| {
                    file.write_all(&bytes)?;
                    before_sync()
                },
                || {
                    published = true;
                    self.incertum = true;
                    self.imago = read_snapshot(&self.via)?;
                    after_rename()
                },
            )
        })();
        let result = finish(result, lock.release());
        if published {
            if let Err(error) = result {
                return Err(io::Error::new(
                    error.kind(),
                    format!("state may already be published; do not retry or roll back: {error}"),
                ));
            }
            self.incertum = false;
        }
        result
    }
}

/// Streams an operational file into an exclusive sibling, syncs it and renames it (REQ-D-02).
/// The destination is never deleted first. This helper does not impose the 32-byte state format.
///
/// # Errors
/// Reports invalid paths, symlinks and I/O; errors after rename report possible publication.
pub fn write_atomic(
    path: &Path,
    write: impl FnOnce(&mut File) -> io::Result<()>,
) -> io::Result<()> {
    let path = canonical_path(path)?;
    let mut published = false;
    let result = publish(&path, write, || {
        published = true;
        Ok(())
    });
    result.map_err(|error| {
        if published {
            io::Error::new(
                error.kind(),
                format!("file may already be published; do not retry or roll back: {error}"),
            )
        } else {
            error
        }
    })
}

/// Removes only an exclusively created operational path (REQ-D-02).
struct Custodia {
    via: Option<PathBuf>,
}
impl Custodia {
    /// Reports cleanup failure rather than hiding it in Drop (REQ-D-02).
    fn release(mut self) -> io::Result<()> {
        if let Some(path) = self.via.take() {
            fs::remove_file(path)
        } else {
            Ok(())
        }
    }
}
impl Drop for Custodia {
    /// Best-effort cleanup during unwinding, never touching a preexisting path (REQ-D-02).
    fn drop(&mut self) {
        if let Some(path) = self.via.take() {
            let _ = fs::remove_file(path);
        }
    }
}

/// Preserves the main failure while also reporting an explicit cleanup failure (REQ-D-02).
fn finish<T>(result: io::Result<T>, cleanup: io::Result<()>) -> io::Result<T> {
    match (result, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => Err(io::Error::new(
            error.kind(),
            format!("{error}; cleanup failed: {cleanup}"),
        )),
    }
}

/// Resolves an existing parent while retaining and checking the final component (REQ-D-02).
fn canonical_path(path: &Path) -> io::Result<PathBuf> {
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path requires a file name"))?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let path = parent.canonicalize()?.join(name);
    reject_symlink(&path)?;
    Ok(path)
}

/// Refuses an existing symlink, including a dangling one (REQ-D-02).
fn reject_symlink(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "symbolic link destination is forbidden",
        )),
        Ok(metadata) if !metadata.is_file() => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "destination must be a regular file",
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

/// Appends an operational suffix without interpreting the path's encoding (REQ-D-02).
fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name: OsString = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

/// Acquires a zero-byte lock; a preexisting lock is never removed (REQ-D-02).
fn acquire(path: &Path) -> io::Result<Custodia> {
    let path = sibling(path, ".lock");
    match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(file) => {
            drop(file);
            Ok(Custodia { via: Some(path) })
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Err(io::Error::new(
            error.kind(),
            format!(
                "state lock exists: {}; recovery requires checking that no writer is active",
                path.display()
            ),
        )),
        Err(error) => Err(error),
    }
}

/// Reads at most 33 bytes, rejecting corruption without allocating by file length (REQ-D-02).
fn read_snapshot(path: &Path) -> io::Result<Option<Imago>> {
    reject_symlink(path)?;
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let metadata = file.metadata()?;
    if metadata.len() != 32 {
        return Err(invalid_state("state must contain exactly 32 bytes"));
    }
    let mut bytes = Vec::with_capacity(33);
    file.take(33).read_to_end(&mut bytes)?;
    let bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| invalid_state("state changed length during read"))?;
    if Mundus::parse(&bytes).is_none() {
        return Err(invalid_state("state is not hexadecimal"));
    }
    Ok(Some(Imago {
        litterae: bytes,
        tempus: metadata.modified().ok(),
    }))
}

/// Creates a same-parent temporary without clobbering colliding names (REQ-D-02).
fn temporary(path: &Path) -> io::Result<(File, Custodia)> {
    for _ in 0..128 {
        let suffix = format!(
            ".{}.{}.tmp",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        );
        let path = sibling(path, &suffix);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((file, Custodia { via: Some(path) })),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "temporary names exhausted",
    ))
}

/// Writes, synchronizes and replaces the name; the callback runs strictly after rename (REQ-D-02).
fn publish(
    path: &Path,
    write: impl FnOnce(&mut File) -> io::Result<()>,
    after_rename: impl FnOnce() -> io::Result<()>,
) -> io::Result<()> {
    reject_symlink(path)?;
    let (mut file, mut guard) = temporary(path)?;
    let result = write(&mut file).and_then(|()| file.sync_all());
    drop(file);
    if let Err(error) = result {
        return finish(Err(error), guard.release());
    }
    let Some(temporary) = guard.via.as_ref() else {
        return Err(io::Error::other("temporary ownership missing"));
    };
    if let Err(error) = fs::rename(temporary, path) {
        return finish(Err(error), guard.release());
    }
    guard.via = None;
    after_rename()?;
    sync_parent(path)
}

/// Requests Unix directory synchronization without claiming absolute power-loss durability (REQ-D-02).
fn sync_parent(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/// Distinguishes corrupt persisted bytes from an absent state (REQ-D-02).
fn invalid_state(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A failure before rename retains both the file and expected snapshot (REQ-D-02).
    #[test]
    fn pre_rename_failure_preserves_snapshot() -> io::Result<()> {
        let directory = std::env::temp_dir().join(format!(
            "sisyphium-pre-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory)?;
        let path = directory.join("state");
        let original = Mundus::genesis(5);
        fs::write(&path, original.format())?;
        let (mut archive, _) = Archivum::open(&path, None, 1)?;
        let before = archive.mtime();
        let foreign = directory.join("state.foreign.tmp");
        fs::write(&foreign, b"foreign")?;
        let result = archive.save_with(
            &Mundus::genesis(73),
            || Err(io::Error::other("injected pre-sync failure")),
            || Ok(()),
        );
        assert!(result.is_err());
        assert_eq!(fs::read(&path)?, original.format());
        assert_eq!(
            archive.imago.as_ref().map(|s| s.litterae),
            Some(original.format())
        );
        assert_eq!(archive.mtime(), before);
        assert_eq!(fs::read(&foreign)?, b"foreign");
        assert_eq!(fs::read_dir(&directory)?.count(), 2);
        archive.save(&Mundus::genesis(73))?;
        assert_eq!(fs::read(&path)?, Mundus::genesis(73).format());
        fs::remove_dir_all(directory)?;
        Ok(())
    }

    /// A failure after publication preserves the new snapshot and refuses a retry (REQ-D-02).
    #[test]
    fn post_rename_failure_does_not_roll_back() -> io::Result<()> {
        let directory = std::env::temp_dir().join(format!(
            "sisyphium-post-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory)?;
        let path = directory.join("state");
        let (mut archive, _) = Archivum::open(&path, None, 1)?;
        let world = Mundus::genesis(73);
        let result = archive.save_with(
            &world,
            || Ok(()),
            || Err(io::Error::other("injected directory sync failure")),
        );
        assert!(result.is_err());
        assert!(
            result
                .err()
                .is_some_and(|e| e.to_string().contains("may already be published"))
        );
        assert_eq!(fs::read(&path)?, world.format());
        assert_eq!(
            archive.imago.as_ref().map(|s| s.litterae),
            Some(world.format())
        );
        assert!(archive.save(&Mundus::genesis(99)).is_err());
        assert!(!sibling(&path, ".lock").exists());
        fs::remove_dir_all(directory)?;
        Ok(())
    }
}
