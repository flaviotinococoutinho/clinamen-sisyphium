//! Validated C boundary for the 128-bit world (REQ-H-03).
//!
//! Non-null pointers require live, aligned memory of the stated extent for the entire call.
//! Outputs must be disjoint from all inputs and other outputs, with no conflicting concurrent
//! accesses. Null checks do not establish validity of arbitrary addresses. Invalid values cause
//! no writes. Zero from step/advance also means an empty valid announcement.

mod forma;
pub use forma::{SyFacultates, SyIndicia, SyRegimen};
use sisyphium::{
    hospes::{Facultates, Indicia, Regimen, Tabula},
    memoria::Memoria,
    mundus::Mundus,
    verbum::Verbum,
};

/// Parses one package version component, refusing values wider than eight bits (REQ-H-03).
#[allow(clippy::panic)]
const fn component(text: &str) -> u32 {
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut value = 0;
    assert!(!bytes.is_empty());
    while index < bytes.len() {
        assert!(bytes[index] >= b'0' && bytes[index] <= b'9');
        value = value * 10 + (bytes[index] - b'0') as u32;
        assert!(value <= 255);
        index += 1;
    }
    value
}
/// Package version, packed into three checked eight-bit fields (REQ-H-03).
const VERSIO: u32 = component(env!("CARGO_PKG_VERSION_MAJOR")) << 16
    | component(env!("CARGO_PKG_VERSION_MINOR")) << 8
    | component(env!("CARGO_PKG_VERSION_PATCH"));

/// Normalizes the raw state through the core constructors (REQ-L0-03).
fn world(word: u64, memory: u64) -> Mundus {
    Mundus {
        verbum: Verbum::new(word),
        memoria: Memoria::new(memory),
    }
}

/// Copies a C value without manufacturing a Rust reference (REQ-H-03).
///
/// # Safety
/// A non-null pointer must be aligned and readable for one initialized `T` for this call.
#[allow(unsafe_code)]
unsafe fn read<T: Copy>(pointer: *const T) -> Option<T> {
    if pointer.is_null() {
        None
    } else {
        // SAFETY: the caller supplies a live aligned initialized object; null was rejected.
        Some(unsafe { pointer.read() })
    }
}

/// Writes the two state words only after all arguments were validated (REQ-H-03).
///
/// # Safety
/// Both pointers must address distinct, aligned writable u64 objects and not overlap inputs.
#[allow(unsafe_code)]
unsafe fn write_world(current: Mundus, word: *mut u64, memory: *mut u64) {
    // SAFETY: the caller established two live, aligned and disjoint output objects.
    unsafe {
        word.write(current.verbum.bits());
        memory.write(current.memoria.bits());
    }
}

/// Returns the package version as major<<16 | minor<<8 | patch (REQ-H-03).
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn sisyphium_version() -> u32 {
    VERSIO
}

/// Creates the seed's world; null output causes no writes (REQ-L0-04).
///
/// # Safety
/// Non-null outputs must be aligned, live, writable u64 objects for the call, mutually disjoint.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sisyphium_genesis(seed: u64, verbum: *mut u64, memoria: *mut u64) {
    if verbum.is_null() || memoria.is_null() {
        return;
    }
    // SAFETY: the caller guarantees writable disjoint words; both were checked for null.
    unsafe {
        write_world(Mundus::genesis(seed), verbum, memoria);
    }
}

/// Parses exactly 32 bytes; invalid text or null returns zero without writes (REQ-L0-03).
///
/// # Safety
/// Non-null text must be readable for 32 initialized bytes. Outputs must be live, aligned,
/// writable u64 objects, disjoint from text and one another, throughout the call.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sisyphium_parse(
    text: *const u8,
    verbum: *mut u64,
    memoria: *mut u64,
) -> i32 {
    if text.is_null() || verbum.is_null() || memoria.is_null() {
        return 0;
    }
    // SAFETY: the caller guarantees 32 readable bytes; copy ends access before output writes.
    let bytes = unsafe { text.cast::<[u8; 32]>().read() };
    let Some(parsed) = Mundus::parse(&bytes) else {
        return 0;
    };
    // SAFETY: checked non-null outputs are aligned, writable and disjoint by caller contract.
    unsafe {
        write_world(parsed, verbum, memoria);
    }
    1
}

/// Writes exactly 32 hexadecimal bytes, with no terminator (REQ-L0-03).
///
/// # Safety
/// Non-null out must be live and writable for 32 bytes, exclusively accessible for the call.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sisyphium_format(verbum: u64, memoria: u64, out: *mut u8) {
    if out.is_null() {
        return;
    }
    // SAFETY: the caller supplies 32 writable bytes; the array has alignment one.
    unsafe {
        out.cast::<[u8; 32]>()
            .write(world(verbum, memoria).format());
    }
}

/// Selects the core regimen after validating capabilities (REQ-C-01, REQ-H-03).
///
/// # Safety
/// Non-null f must be a live aligned readable `SyFacultates`. Out must be a live aligned writable
/// `SyRegimen`, disjoint from f and exclusively accessible throughout the call.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sisyphium_consilium(
    f: *const SyFacultates,
    verbum: u64,
    memoria: u64,
    energy: u8,
    out: *mut SyRegimen,
) {
    if out.is_null() {
        return;
    }
    // SAFETY: caller promises readable aligned input when non-null; copied before writing out.
    let Some(f) = (unsafe { read(f) }).and_then(|v| Facultates::try_from(v).ok()) else {
        return;
    };
    let regimen = sisyphium::consilium::consilium(&f, &world(verbum, memoria), energy);
    // SAFETY: out is non-null, aligned, writable and disjoint from inputs by contract.
    unsafe {
        out.write(regimen.into());
    }
}

/// Advances one tick and returns announcement bits; invalid parameters preserve outputs (REQ-H-03).
///
/// # Safety
/// Every non-null pointer must be aligned and live for its complete initialized object.
/// Words must be readable and writable, disjoint from each other and f/i. Inputs must remain
/// readable; no conflicting accesses may occur during the call.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sisyphium_step(
    verbum: *mut u64,
    memoria: *mut u64,
    f: *const SyFacultates,
    i: *const SyIndicia,
) -> u32 {
    // SAFETY: this function's caller contract supplies the same objects and alias restrictions.
    unsafe { sisyphium_advance(verbum, memoria, f, i, 1, 42) }
}

/// Advances at most 65536 ticks at a legal cadence; invalid values preserve outputs (REQ-H-06).
/// Zero can also mean a valid empty announcement; it is not an unambiguous error code.
///
/// # Safety
/// Non-null pointers must be live, aligned and readable for their full initialized objects.
/// Words must additionally be writable and disjoint from each other and all inputs throughout
/// the call. Conflicting concurrent accesses are forbidden.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sisyphium_advance(
    verbum: *mut u64,
    memoria: *mut u64,
    f: *const SyFacultates,
    i: *const SyIndicia,
    ticks: u32,
    tick_ms: u32,
) -> u32 {
    if verbum.is_null()
        || memoria.is_null()
        || f.is_null()
        || i.is_null()
        || ticks > 65_536
        || !matches!(tick_ms, 42 | 125 | 250 | 60_000)
    {
        return 0;
    }
    // SAFETY: caller supplies aligned readable objects; copies precede any mutable output access.
    let (Some(f), Some(i), Some(word), Some(memory)) =
        (unsafe { (read(f), read(i), read(verbum), read(memoria)) })
    else {
        return 0;
    };
    let (Ok(f), Ok(i)) = (Facultates::try_from(f), Indicia::try_from(i)) else {
        return 0;
    };
    let mut current = world(word, memory);
    let said = current.advance(&f, &i, ticks, tick_ms);
    // SAFETY: caller supplies disjoint writable words; all validation has completed.
    unsafe {
        write_world(current, verbum, memoria);
    }
    u32::from(said.0)
}

/// Copies and validates the three presentation descriptors (REQ-H-03).
///
/// # Safety
/// Non-null pointers must refer to live aligned initialized readable objects of their types.
#[allow(unsafe_code)]
unsafe fn presentation(
    f: *const SyFacultates,
    r: *const SyRegimen,
    i: *const SyIndicia,
) -> Option<(Facultates, Regimen, Indicia)> {
    if f.is_null() || r.is_null() || i.is_null() {
        return None;
    }
    // SAFETY: caller guarantees readable aligned objects; all values are copied immediately.
    let (f, r, i) = unsafe { (read(f)?, read(r)?, read(i)?) };
    Some((f.try_into().ok()?, r.try_into().ok()?, i.try_into().ok()?))
}

/// Renders width*height bytes; dimensions must equal f (REQ-H-02, REQ-H-03).
///
/// # Safety
/// Descriptors must be live, aligned and readable for their initialized types. Non-null pixels
/// must address at least width*height writable bytes within one allocation, disjoint from every
/// input and exclusively accessible for the call. The extent must not exceed `isize::MAX`.
#[allow(unsafe_code, clippy::too_many_arguments)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sisyphium_render(
    verbum: u64,
    memoria: u64,
    f: *const SyFacultates,
    r: *const SyRegimen,
    i: *const SyIndicia,
    pixels: *mut u8,
    width: u16,
    height: u16,
) {
    if pixels.is_null() {
        return;
    }
    // SAFETY: descriptor validity is a caller precondition; copies precede the mutable slice.
    let Some((f, r, i)) = (unsafe { presentation(f, r, i) }) else {
        return;
    };
    if width != f.width || height != f.height {
        return;
    }
    let Some(length) = usize::from(width)
        .checked_mul(usize::from(height))
        .filter(|&n| isize::try_from(n).is_ok())
    else {
        return;
    };
    // SAFETY: the caller supplies a writable allocation of the checked length. Initialize it
    // before forming a Rust slice, so C callers may supply an uninitialized output buffer.
    unsafe {
        pixels.write_bytes(0, length);
    }
    // SAFETY: the disjoint live allocation now contains initialized bytes; checked isize bound.
    let buffer = unsafe { std::slice::from_raw_parts_mut(pixels, length) };
    if let Some(mut surface) = Tabula::new(width, height, buffer) {
        world(verbum, memoria).render(&f, &i, &r, &mut surface);
    }
}

/// Writes exactly 48 RGB bytes (sixteen triples), without a terminator (REQ-H-03).
///
/// # Safety
/// Descriptors must be live, aligned and readable for their initialized types. Non-null out
/// must address 48 writable bytes, disjoint from inputs and exclusively accessible for the call.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sisyphium_palette(
    verbum: u64,
    memoria: u64,
    f: *const SyFacultates,
    r: *const SyRegimen,
    i: *const SyIndicia,
    out: *mut u8,
) {
    if out.is_null() {
        return;
    }
    // SAFETY: descriptor validity is promised by the caller; inputs are copied before output.
    let Some((f, r, i)) = (unsafe { presentation(f, r, i) }) else {
        return;
    };
    let palette = world(verbum, memoria).palette(&f, &i, &r);
    // SAFETY: the caller supplies 48 writable bytes; [[u8;3];16] is contiguous, alignment one.
    unsafe {
        out.cast::<[[u8; 3]; 16]>().write(palette);
    }
}
