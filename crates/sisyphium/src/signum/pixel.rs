//! The pixel of a `Tabula`: bits 0..=1 stratum class, 2..=3 edge label, 4..=7 luminance
//! (REQ-L5-01).

/// Edge label: convex edge.
pub const LABEL_CONVEX: u8 = 0;
/// Edge label: concave edge.
pub const LABEL_CONCAVE: u8 = 1;
/// Edge label: occluding contour.
pub const LABEL_OCCLUDING: u8 = 2;
/// Edge label: none.
pub const LABEL_NONE: u8 = 3;

/// Packs luminance (0..=15), stratum class (0..=3) and edge label (0..=3) into one byte.
#[must_use]
pub const fn pixel(lum: u8, stratum: u8, label: u8) -> u8 {
    ((lum & 0xF) << 4) | ((label & 3) << 2) | (stratum & 3)
}

/// Luminance of a pixel, 0..=15.
#[must_use]
pub const fn lum_of(p: u8) -> u8 {
    p >> 4
}

/// Stratum class of a pixel, 0..=3 (3 = void).
#[must_use]
pub const fn stratum_of(p: u8) -> u8 {
    p & 3
}

/// Edge label of a pixel, 0..=3.
#[must_use]
pub const fn label_of(p: u8) -> u8 {
    (p >> 2) & 3
}

/// The slate: luminance 1, void class, no label. Every frame starts as this.
pub const VOID: u8 = pixel(1, 3, 3);

#[cfg(test)]
mod tests {
    use super::*;

    /// Packing and the three accessors (REQ-L5-01).
    #[test]
    fn pixel_fields() {
        for lum in 0..16u8 {
            for stratum in 0..4u8 {
                for label in 0..4u8 {
                    let p = pixel(lum, stratum, label);
                    assert_eq!(
                        (lum_of(p), stratum_of(p), label_of(p)),
                        (lum, stratum, label)
                    );
                }
            }
        }
        assert_eq!(pixel(15, 3, 3), 0xFF);
        assert_eq!(pixel(0, 0, 0), 0);
        assert_eq!(pixel(0x1F, 7, 7), 0xFF);
        assert_eq!(VOID, pixel(1, 3, 3));
        assert_eq!(
            (LABEL_CONVEX, LABEL_CONCAVE, LABEL_OCCLUDING, LABEL_NONE),
            (0, 1, 2, 3)
        );
    }
}
