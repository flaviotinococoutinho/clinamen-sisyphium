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
// Tests may panic, index, count and compare freely: they are the verification, not the law.
#![cfg_attr(
    test,
    allow(
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::float_arithmetic,
        clippy::float_cmp,
        clippy::cast_precision_loss,
        clippy::needless_range_loop,
        clippy::manual_range_contains,
        clippy::naive_bytecount
    )
)]

#[cfg(test)]
extern crate std;

mod campi;
pub mod concentus;
pub mod corpus;
pub mod ephemeris;
pub mod hospes;
pub mod memoria;
pub mod mundus;
pub mod natura;
pub mod signum;
pub mod spatium;
pub mod verbum;
