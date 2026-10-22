//! Clinamen Sisyphium: the core of a dynamic wallpaper that stages the repeated ascent of
//! Sisyphus under laws derived from the device that runs it.
//!
//! The whole state of the world is two 64-bit words, [`verbum::Verbum`] (the present) and
//! [`memoria::Memoria`] (the past and the intent); every other quantity is recomputed from
//! them, from the host's measurements ([`hospes::Indicia`]) and from its capabilities
//! ([`hospes::Facultates`]). The crate performs no input, no output, no allocation, no
//! floating-point arithmetic and no unchecked indexing: every function is total and
//! deterministic, so the same inputs yield the same frame on every platform.
#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::missing_docs_in_private_items
)]

#[cfg(test)]
extern crate std;

mod campi;
pub mod ephemeris;
pub mod hospes;
pub mod memoria;
pub mod natura;
pub mod spatium;
pub mod verbum;
