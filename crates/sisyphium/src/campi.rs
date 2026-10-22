//! Bit-field laws shared by the two state words: read, write and the accessor macro.

/// Reads `width` bits of `bits` starting at `shift`.
pub(crate) const fn campus(bits: u64, shift: u32, width: u32) -> u64 {
    bits.wrapping_shr(shift) & u64::MAX.wrapping_shr(64u32.wrapping_sub(width))
}

/// Writes `value` (truncated to `width` bits) into `bits` at `shift`.
pub(crate) const fn cum_campo(bits: u64, shift: u32, width: u32, value: u64) -> u64 {
    let mask = u64::MAX
        .wrapping_shr(64u32.wrapping_sub(width))
        .wrapping_shl(shift);
    (bits & !mask) | (value.wrapping_shl(shift) & mask)
}

/// Defines, for a `#[repr(transparent)]` word type, one accessor and one `with_` counterpart
/// per field from the bit layout `[shift, width]`.
macro_rules! campi {
    ($word:ident { $($(#[$doc:meta])* $get:ident / $with:ident : $t:ty [$shift:literal, $width:literal]),* $(,)? }) => {
        impl $word {
            $(
                $(#[$doc])*
                #[must_use]
                pub const fn $get(self) -> $t {
                    $crate::campi::campus(self.0, $shift, $width) as $t
                }

                #[doc = concat!("Returns the word with the field read by `", stringify!($get), "` replaced by `value` masked to the field width.")]
                #[must_use]
                pub const fn $with(self, value: $t) -> Self {
                    Self::new($crate::campi::cum_campo(self.0, $shift, $width, value as u64))
                }
            )*
        }
    };
}
pub(crate) use campi;
