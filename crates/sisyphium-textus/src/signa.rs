//! Narrow native signal boundary (REQ-D-02, ADR0008).
//! Only SIGINT/SIGTERM dispositions and `CTRL_C/CTRL_BREAK` are handled. Signal masks,
//! sigaction flags, blocked stdout and abrupt process termination are not controlled.
#![allow(unsafe_code)]

use std::{
    io,
    sync::atomic::{AtomicBool, Ordering},
};

/// Lock-free flag publishes no other data (REQ-D-02).
static FINIS: AtomicBool = AtomicBool::new(false);
/// Prevents two live guards from replacing one another's handlers (REQ-D-02).
static OCCUPATA: AtomicBool = AtomicBool::new(false);

/// Returns whether a cooperative shutdown was requested (REQ-D-02).
#[must_use]
pub fn requested() -> bool {
    OCCUPATA.load(Ordering::Relaxed) && FINIS.load(Ordering::Relaxed)
}

/// Exclusive ownership of this host's native signal registrations (REQ-D-02).
#[derive(Debug)]
pub struct Custodia {
    priora: Option<native::Prioritas>,
}
impl Custodia {
    /// Installs supported native handlers before entering the loop (REQ-D-02).
    ///
    /// # Errors
    /// Reports unsupported targets, overlapping guards and native registration failure.
    pub fn install() -> io::Result<Self> {
        if OCCUPATA
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return Err(io::Error::other("signal guard already active"));
        }
        FINIS.store(false, Ordering::Relaxed);
        match native::install() {
            Ok(previous) => Ok(Self {
                priora: Some(previous),
            }),
            Err(error) => {
                OCCUPATA.store(false, Ordering::Release);
                Err(error)
            }
        }
    }

    /// Restores previous dispositions or removes exactly this callback (REQ-D-02).
    ///
    /// # Errors
    /// Reports native restoration failures; no automatic retry is attempted.
    pub fn restore(mut self) -> io::Result<()> {
        self.release()
    }

    /// Performs cleanup once, including after an operational failure (REQ-D-02).
    fn release(&mut self) -> io::Result<()> {
        let Some(previous) = self.priora.take() else {
            return Ok(());
        };
        let result = native::restore(previous);
        OCCUPATA.store(false, Ordering::Release);
        result
    }
}
impl Drop for Custodia {
    /// Best-effort fallback; normal exit calls the fallible restore method (REQ-D-02).
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// Installs two dispositions and rolls back the first on second-call failure (REQ-D-02).
#[cfg(any(
    test,
    all(
        target_has_atomic = "8",
        target_pointer_width = "64",
        any(target_arch = "x86_64", target_arch = "aarch64"),
        any(target_os = "macos", all(target_os = "linux", target_env = "gnu"))
    )
))]
fn install_pair(
    mut replace: impl FnMut(i32, usize) -> io::Result<usize>,
    handler: usize,
) -> io::Result<[usize; 2]> {
    let first = replace(2, handler)?;
    match replace(15, handler) {
        Ok(second) => Ok([first, second]),
        Err(error) => match replace(2, first) {
            Ok(_) => Err(error),
            Err(rollback) => Err(io::Error::other(format!(
                "{error}; signal rollback failed: {rollback}"
            ))),
        },
    }
}

/// Attempts both restorations in reverse order and reports either failure (REQ-D-02).
#[cfg(any(
    test,
    all(
        target_has_atomic = "8",
        target_pointer_width = "64",
        any(target_arch = "x86_64", target_arch = "aarch64"),
        any(target_os = "macos", all(target_os = "linux", target_env = "gnu"))
    )
))]
fn restore_pair(
    mut replace: impl FnMut(i32, usize) -> io::Result<usize>,
    previous: [usize; 2],
) -> io::Result<()> {
    let second = replace(15, previous[1]);
    let first = replace(2, previous[0]);
    second.and(first).map(|_| ())
}

/// A Windows control event only publishes a stop request for C or BREAK (REQ-D-02).
#[cfg(any(test, windows))]
fn control(event: u32, stop: &AtomicBool) -> i32 {
    match event {
        0 | 1 => {
            stop.store(true, Ordering::Relaxed);
            1
        }
        _ => 0,
    }
}

#[cfg(all(
    target_has_atomic = "8",
    target_pointer_width = "64",
    any(target_arch = "x86_64", target_arch = "aarch64"),
    any(target_os = "macos", all(target_os = "linux", target_env = "gnu"))
))]
mod native {
    use super::{FINIS, Ordering, install_pair, restore_pair};
    use std::{ffi::c_int, io};
    /// Raw ABI bits retain `SIG_DFL=0`, `SIG_IGN=1` and `SIG_ERR=usize::MAX` (ADR0008).
    pub type Prioritas = [usize; 2];
    unsafe extern "C" {
        /// Preserves the native handler representation, including sentinels (ADR0008).
        fn signal(number: c_int, handler: usize) -> usize;
    }

    /// The asynchronous callback performs only a lock-free atomic store (REQ-D-02).
    extern "C" fn request_stop(_: c_int) {
        FINIS.store(true, Ordering::Relaxed);
    }

    /// Crosses only the admitted Darwin/glibc 64-bit integer handler ABI (ADR0008).
    fn replace(number: i32, handler: usize) -> io::Result<usize> {
        // SAFETY: the admitted ABIs represent handler pointers and sentinels in usize.
        // Values passed here are the static callback or untouched prior dispositions.
        // No returned value is converted to a Rust function or invoked.
        let previous = unsafe { signal(number, handler) };
        if previous == usize::MAX {
            Err(io::Error::last_os_error())
        } else {
            Ok(previous)
        }
    }
    /// Installs both supported signals with rollback (REQ-D-02).
    pub fn install() -> io::Result<Prioritas> {
        install_pair(replace, request_stop as *const () as usize)
    }
    /// Restores both previous dispositions, not masks or sigaction flags (REQ-D-02).
    pub fn restore(previous: Prioritas) -> io::Result<()> {
        restore_pair(replace, previous)
    }
}

#[cfg(all(windows, target_has_atomic = "8"))]
mod native {
    use super::{FINIS, control};
    use std::io;
    /// Windows removes exactly the callback registered by this guard (REQ-D-02).
    pub type Prioritas = ();
    type Tractator = extern "system" fn(u32) -> i32;
    #[link(name = "Kernel32")]
    unsafe extern "system" {
        /// Adds or removes the exact static callback (REQ-D-02).
        fn SetConsoleCtrlHandler(handler: Option<Tractator>, add: i32) -> i32;
        /// Reads the existing output handle without changing it (REQ-H-02).
        fn GetStdHandle(kind: u32) -> *mut std::ffi::c_void;
        /// Reads existing mode bits without enabling terminal processing (REQ-H-02).
        fn GetConsoleMode(handle: *mut std::ffi::c_void, mode: *mut u32) -> i32;
    }
    /// Only C and BREAK publish a lock-free stop request (REQ-D-02).
    extern "system" fn handler(event: u32) -> i32 {
        control(event, &FINIS)
    }
    /// Registers the static callback without changing the ignore-Ctrl-C attribute (REQ-D-02).
    pub fn install() -> io::Result<Prioritas> {
        // SAFETY: the static callback uses the documented DWORD/BOOL WINAPI signature.
        if unsafe { SetConsoleCtrlHandler(Some(handler), 1) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
    /// Removes only this process's registered callback (REQ-D-02).
    pub fn restore(_: Prioritas) -> io::Result<()> {
        // SAFETY: this is the same static callback that install registered.
        if unsafe { SetConsoleCtrlHandler(Some(handler), 0) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
    /// Observes pre-existing VT output mode without changing console settings (REQ-H-02).
    pub fn ansi_enabled() -> bool {
        let mut mode = 0;
        // SAFETY: STD_OUTPUT_HANDLE is -11 as DWORD; the returned opaque handle is
        // passed unchanged, and mode points to a live writable u32 for the call.
        unsafe {
            GetConsoleMode(GetStdHandle((-11_i32) as u32), &raw mut mode) != 0 && mode & 4 != 0
        }
    }
}

#[cfg(not(any(
    all(windows, target_has_atomic = "8"),
    all(
        target_has_atomic = "8",
        target_pointer_width = "64",
        any(target_arch = "x86_64", target_arch = "aarch64"),
        any(target_os = "macos", all(target_os = "linux", target_env = "gnu"))
    )
)))]
mod native {
    use std::io;
    /// No native handler ABI is asserted for this target (ADR0008).
    pub type Prioritas = ();
    /// Refuses unsupported signal registration before loop effects (REQ-D-02).
    pub fn install() -> io::Result<Prioritas> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "native signal ABI unsupported",
        ))
    }
    /// No registration was possible (REQ-D-02).
    pub fn restore(_: Prioritas) -> io::Result<()> {
        Ok(())
    }
}

/// Checks Windows VT output without enabling it (REQ-H-02).
#[cfg(all(windows, target_has_atomic = "8"))]
#[must_use]
pub fn ansi_enabled() -> bool {
    native::ansi_enabled()
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Registration failure preserves raw sentinel dispositions through rollback (REQ-D-02).
    #[test]
    fn signal_registration_rollback() {
        let mut calls = Vec::new();
        let result = install_pair(
            |number, handler| {
                calls.push((number, handler));
                if number == 15 {
                    Err(io::Error::other("second failed"))
                } else {
                    Ok(1)
                }
            },
            42,
        );
        assert!(result.is_err());
        assert_eq!(calls, [(2, 42), (15, 42), (2, 1)]);
        let mut calls = Vec::new();
        assert!(
            restore_pair(
                |number, handler| {
                    calls.push((number, handler));
                    Err(io::Error::other("restore failed"))
                },
                [0, 1]
            )
            .is_err()
        );
        assert_eq!(calls, [(15, 1), (2, 0)]);
    }
    /// Windows close/logoff/shutdown remain unhandled; tests use a local flag (REQ-D-02).
    #[test]
    fn windows_callback_mapping() {
        for event in [0, 1, 2, 5, 6, u32::MAX] {
            let stop = AtomicBool::new(false);
            assert_eq!(control(event, &stop), i32::from(event <= 1));
            assert_eq!(stop.load(Ordering::Relaxed), event <= 1);
        }
    }
}
