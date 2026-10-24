//! State publication and conflict laws (REQ-D-02).
use sisyphium::mundus::Mundus;
use sisyphium_hospes::archivum::{Archivum, write_atomic};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static SERIAL: AtomicU64 = AtomicU64::new(0);
struct Locus(PathBuf);
impl Locus {
    /// Creates an exclusively owned fixture directory (REQ-D-02).
    fn new() -> io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "sisyphium-hospes-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }
}
impl Drop for Locus {
    /// Removes only the directory owned by this fixture (REQ-D-02).
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
/// Exercises the named host law against independent inputs (REQ-C-05, REQ-H-06, REQ-D-02).
#[test]
fn state_transaction_preserves_previous() -> io::Result<()> {
    let dir = Locus::new()?;
    let path = dir.0.join("mundus.hex");
    for bytes in [vec![], vec![b'0'; 31], vec![b'0'; 33], vec![b'g'; 32]] {
        fs::write(&path, &bytes)?;
        assert!(Archivum::open(&path, None, 1).is_err());
        assert_eq!(fs::read(&path)?, bytes);
    }
    fs::write(&path, b"ABCDEF0123456789ABCDEF0123456789")?;
    assert!(Archivum::open(&path, Some(1), 1).is_err());
    let (mut first, world) = Archivum::open(&path, None, 1)?;
    let (mut second, _) = Archivum::open(&path, None, 1)?;
    first.save(&world)?;
    assert_eq!(fs::read(&path)?, world.format());
    assert!(second.save(&Mundus::genesis(23)).is_err());
    let lock = dir.0.join("mundus.hex.lock");
    fs::write(&lock, [])?;
    assert!(Archivum::open(&path, None, 1).is_err());
    assert_eq!(fs::metadata(&lock)?.len(), 0);
    assert_eq!(fs::read(&path)?.len(), 32);
    Ok(())
}
/// Exercises the named host law against independent inputs (REQ-C-05, REQ-H-06, REQ-D-02).
#[test]
fn absent_snapshot_conflicts_with_creation() -> io::Result<()> {
    let dir = Locus::new()?;
    let path = dir.0.join("mundus.hex");
    let (mut first, world) = Archivum::open(&path, Some(42), 1)?;
    let (mut second, _) = Archivum::open(&path, None, 1)?;
    assert_eq!(world, Mundus::genesis(42));
    first.save(&world)?;
    assert!(second.save(&Mundus::genesis(3)).is_err());
    Ok(())
}
/// Exercises the named host law against independent inputs (REQ-C-05, REQ-H-06, REQ-D-02).
#[test]
fn atomic_failure_keeps_committed_state() -> io::Result<()> {
    let dir = Locus::new()?;
    let path = dir.0.join("image");
    fs::write(&path, b"previous")?;
    let unrelated = dir.0.join("image.foreign.tmp");
    fs::write(&unrelated, b"foreign")?;
    assert!(
        write_atomic(&path, |file| {
            file.write_all(b"partial")?;
            Err(io::Error::other("injected write failure"))
        })
        .is_err()
    );
    assert_eq!(fs::read(&path)?, b"previous");
    assert_eq!(fs::read(&unrelated)?, b"foreign");
    assert_eq!(fs::read_dir(&dir.0)?.count(), 2);
    write_atomic(&path, |file| file.write_all(b"new"))?;
    assert_eq!(fs::read(&path)?, b"new");
    Ok(())
}
#[cfg(unix)]
/// Exercises the named host law against independent inputs (REQ-C-05, REQ-H-06, REQ-D-02).
#[test]
fn symlink_state_is_rejected() -> io::Result<()> {
    let dir = Locus::new()?;
    let real = dir.0.join("real");
    let link = dir.0.join("link");
    fs::write(&real, Mundus::genesis(1).format())?;
    std::os::unix::fs::symlink(&real, &link)?;
    assert!(Archivum::open(&link, None, 1).is_err());
    assert!(write_atomic(&link, |file| file.write_all(b"new")).is_err());
    assert_eq!(fs::read(&real)?, Mundus::genesis(1).format());
    Ok(())
}

/// Exercises the named host law against independent inputs (REQ-C-05, REQ-H-06, REQ-D-02).
#[test]
fn unchanged_state_preserves_time_and_mtime_change_conflicts() -> io::Result<()> {
    use std::time::{Duration, UNIX_EPOCH};
    let dir = Locus::new()?;
    let path = dir.0.join("state");
    let world = Mundus::genesis(1);
    fs::write(&path, world.format())?;
    let file = fs::OpenOptions::new().write(true).open(&path)?;
    file.set_times(fs::FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(10_000)))?;
    let (mut archive, _) = Archivum::open(&path, None, 1)?;
    let original = archive.mtime();
    archive.save(&world)?;
    assert_eq!(fs::metadata(&path)?.modified().ok(), original);
    file.set_times(fs::FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(20_000)))?;
    assert!(archive.save(&world).is_err());
    assert_eq!(fs::read(&path)?, world.format());
    Ok(())
}

/// Exercises the named host law against independent inputs (REQ-C-05, REQ-H-06, REQ-D-02).
#[test]
fn simultaneous_writers_publish_only_one_snapshot() -> io::Result<()> {
    use std::sync::{Arc, Barrier};
    let dir = Locus::new()?;
    let path = dir.0.join("state");
    let (mut first, _) = Archivum::open(&path, None, 1)?;
    let (mut second, _) = Archivum::open(&path, None, 1)?;
    let barrier = Arc::new(Barrier::new(2));
    let other = Arc::clone(&barrier);
    let writer = std::thread::spawn(move || {
        other.wait();
        first.save(&Mundus::genesis(31))
    });
    barrier.wait();
    let second = second.save(&Mundus::genesis(37));
    let first = writer
        .join()
        .map_err(|_| io::Error::other("writer panicked"))?;
    assert_ne!(first.is_ok(), second.is_ok());
    let expected = if first.is_ok() { 31 } else { 37 };
    assert_eq!(fs::read(&path)?, Mundus::genesis(expected).format());
    Ok(())
}

#[cfg(unix)]
/// Exercises the named host law against independent inputs (REQ-C-05, REQ-H-06, REQ-D-02).
#[test]
fn dangling_symlinks_and_foreign_locks_survive() -> io::Result<()> {
    let dir = Locus::new()?;
    let path = dir.0.join("state");
    let missing = dir.0.join("absent");
    std::os::unix::fs::symlink(&missing, &path)?;
    assert!(Archivum::open(&path, None, 1).is_err());
    assert!(fs::symlink_metadata(&path)?.file_type().is_symlink());
    let other = dir.0.join("other");
    let lock = dir.0.join("other.lock");
    std::os::unix::fs::symlink(&missing, &lock)?;
    assert!(Archivum::open(&other, None, 1).is_err());
    assert!(fs::symlink_metadata(&lock)?.file_type().is_symlink());
    assert!(!other.exists());
    Ok(())
}
