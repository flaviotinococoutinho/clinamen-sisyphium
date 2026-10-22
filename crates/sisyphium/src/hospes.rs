//! `hospes`: the threshold between the core and its host. Capabilities declared once per
//! session, measurements delivered every tick, the chosen regimen, the output surfaces and the
//! announcements returned by a step (REQ-C-01, REQ-C-02, REQ-H-01, REQ-H-02).

/// Language of the text of the experience.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Lingua {
    /// English.
    #[default]
    En = 0,
    /// Portuguese.
    Pt = 1,
}

/// Visual idiom of a frame (`docs/05-estetica.md` §3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Stilus {
    /// Chalk on slate: one-pixel lines in sixteen tones, dithered fills.
    #[default]
    Creta = 0,
    /// Mosaic: a low-resolution raster of block figures, scaled by nearest neighbour.
    Musivum = 1,
    /// Letters: a grid of glyph cells; the scene as text.
    Littera = 2,
    /// Two tones: one bit per pixel by the ordered dither.
    Bicolor = 3,
}

/// Capabilities a host declares once per session (REQ-C-01). Each flag is an independent
/// capability of the contract, hence the explicit booleans.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(clippy::struct_excessive_bools)]
pub struct Facultates {
    /// Width of the frame the host will request, in pixels.
    pub width: u16,
    /// Height of the frame, in pixels.
    pub height: u16,
    /// Color depth available: 1, 4, 8 or 24 bits.
    pub color_bits: u8,
    /// Maximum presentation rate in hertz; 0 means static only.
    pub max_hz: u8,
    /// The host delivers touch or pointer positions.
    pub touch: bool,
    /// The host delivers motion (accelerometer) agitation.
    pub motion: bool,
    /// The host signals occlusion by other windows or applications.
    pub occlusion: bool,
    /// The host can persist the 32-digit state.
    pub persistence: bool,
    /// The host shows glyph cells natively (a terminal).
    pub cells: bool,
    /// Southern hemisphere.
    pub australis: bool,
    /// Local clock offset from UTC in minutes.
    pub tz_offset_min: i16,
    /// Language of the text.
    pub lingua: Lingua,
    /// Relative compute budget, 0..=255.
    pub compute: u8,
}

impl Default for Facultates {
    fn default() -> Self {
        Self {
            width: 320,
            height: 180,
            color_bits: 8,
            max_hz: 24,
            touch: false,
            motion: false,
            occlusion: false,
            persistence: false,
            cells: false,
            australis: true,
            tz_offset_min: -180,
            lingua: Lingua::En,
            compute: 255,
        }
    }
}

/// Measurements of the environment delivered by the host at every tick (REQ-H-01).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Indicia {
    /// UTC time in milliseconds since the epoch.
    pub epoch_ms: i64,
    /// Available energy, 0..=255; 255 when plugged in or unknown.
    pub energy: u8,
    /// External agitation, 0..=255: accelerometer noise, interaction entropy, load.
    pub seism: u8,
    /// Mechanical regularity of the observer's interaction, 0..=255.
    pub automatism: u8,
    /// Seconds since the last interaction; `u32::MAX` when unknown.
    pub idle_s: u32,
    /// Milliseconds since the last return from occlusion; `u32::MAX` when never.
    pub returned_ms: u32,
    /// The work is covered by another window or application.
    pub occluded: bool,
    /// The work is about to be hidden because the observer returned.
    pub leaving: bool,
    /// A touch or pointer position normalized to 0..=255 on each axis.
    pub touch: Option<(u8, u8)>,
    /// Horizontal page offset of the launcher, 128 centered.
    pub offset: u8,
    /// Device roll, signed.
    pub tilt: i8,
}

impl Default for Indicia {
    fn default() -> Self {
        Self {
            epoch_ms: 0,
            energy: 255,
            seism: 0,
            automatism: 0,
            idle_s: u32::MAX,
            returned_ms: u32::MAX,
            occluded: false,
            leaving: false,
            touch: None,
            offset: 128,
            tilt: 0,
        }
    }
}

/// Layer bit: the sky graph.
pub const CAELUM: u8 = 1;
/// Layer bit: the organic frame.
pub const MARGO: u8 = 2;
/// Layer bit: the chronicles.
pub const ANNALES: u8 = 4;
/// Layer bit: the secondary figures.
pub const PERSONAE: u8 = 8;
/// Layer bit: the ruins.
pub const RUINAE: u8 = 16;
/// Layer bit: the impossible figures.
pub const IMPOSSIBILIA: u8 = 32;
/// Layer bit: the hexagram.
pub const HEXAGRAMMA: u8 = 64;
/// Layer bit: dithered fills.
pub const PUNCTA: u8 = 128;
/// Every layer.
pub const LAYERS_ALL: u8 = 255;

/// The configuration chosen by `consilium` (REQ-C-02).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Regimen {
    /// Visual idiom.
    pub stilus: Stilus,
    /// Number of parallax strata, 1..=4.
    pub strata: u8,
    /// Cadence of the world in milliseconds per tick: 42, 125, 250 or 60 000.
    pub tick_ms: u32,
    /// Cadence of presentation in milliseconds, at least `tick_ms`.
    pub interval_ms: u32,
    /// Enabled layers, one bit each.
    pub layers: u8,
    /// Visible chronicle lines, 0..=6.
    pub lines: u8,
}

impl Regimen {
    /// Whether a layer is enabled.
    #[must_use]
    pub const fn has(self, layer: u8) -> bool {
        self.layers & layer != 0
    }
}

impl Default for Regimen {
    fn default() -> Self {
        Self {
            stilus: Stilus::Creta,
            strata: 4,
            tick_ms: 42,
            interval_ms: 42,
            layers: LAYERS_ALL,
            lines: 6,
        }
    }
}

/// Announcements returned by a step, one bit each.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Nuntius(pub u8);

impl Nuntius {
    /// Nothing happened.
    pub const NIHIL: Self = Self(0);
    /// A cycle closed.
    pub const CLAUSURA: Self = Self(1);
    /// A clinamen fired.
    pub const CLINAMEN: Self = Self(2);
    /// A secondary figure appeared.
    pub const PERSONA: Self = Self(4);
    /// The dream ended in recognition.
    pub const AGNITIO: Self = Self(8);
    /// The observer's touch fell inside the predicted circle.
    pub const METALEPSIS: Self = Self(16);
    /// A rare event window opened.
    pub const RARUM: Self = Self(32);
    /// A chip of the boulder flipped a bit of the world.
    pub const LAPSUS: Self = Self(64);

    /// Union of two announcements.
    #[must_use]
    pub const fn cum(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Whether every bit of `other` is set.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

/// A raster output surface owned by the host: one byte per pixel, row-major (REQ-H-02). The
/// fields are public by contract, so the length guard of `new` is advisory: every write of the
/// core goes through checked access and never trusts `width · height ≤ pixels.len()`.
#[derive(Debug)]
pub struct Tabula<'a> {
    /// Width in pixels.
    pub width: u16,
    /// Height in pixels.
    pub height: u16,
    /// At least `width · height` bytes.
    pub pixels: &'a mut [u8],
}

impl<'a> Tabula<'a> {
    /// Wraps a buffer, refusing one shorter than `width · height`.
    #[must_use]
    pub fn new(width: u16, height: u16, pixels: &'a mut [u8]) -> Option<Self> {
        let needed = usize::from(width).checked_mul(usize::from(height))?;
        (pixels.len() >= needed).then_some(Self {
            width,
            height,
            pixels,
        })
    }
}

/// A glyph-cell output surface owned by the host: bits 0..=7 glyph, 8..=11 tone, 12..=13
/// stratum per cell, row-major (REQ-H-02). As with [`Tabula`], every write uses checked access.
#[derive(Debug)]
pub struct Cellae<'a> {
    /// Columns.
    pub cols: u16,
    /// Rows.
    pub rows: u16,
    /// At least `cols · rows` cells.
    pub cells: &'a mut [u16],
}

impl<'a> Cellae<'a> {
    /// Wraps a buffer, refusing one shorter than `cols · rows`.
    #[must_use]
    pub fn new(cols: u16, rows: u16, cells: &'a mut [u16]) -> Option<Self> {
        let needed = usize::from(cols).checked_mul(usize::from(rows))?;
        (cells.len() >= needed).then_some(Self { cols, rows, cells })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Default capabilities: the logical frame, 8-bit color, 24 Hz, no inputs, austral, −03:00,
    /// English, full compute (REQ-C-01).
    #[test]
    fn facultates_defaults() {
        let f = Facultates::default();
        assert_eq!(
            (f.width, f.height, f.color_bits, f.max_hz),
            (320, 180, 8, 24)
        );
        assert!(!f.touch && !f.motion && !f.occlusion && !f.persistence && !f.cells);
        assert!(f.australis);
        assert_eq!(f.tz_offset_min, -180);
        assert_eq!(f.lingua, Lingua::En);
        assert_eq!(f.compute, 255);
    }

    /// Default measurements: full energy, no agitation, unknown idleness, no touch (REQ-H-01).
    #[test]
    fn indicia_defaults() {
        let i = Indicia::default();
        assert_eq!(
            (i.epoch_ms, i.energy, i.seism, i.automatism),
            (0, 255, 0, 0)
        );
        assert_eq!((i.idle_s, i.returned_ms), (u32::MAX, u32::MAX));
        assert!(!i.occluded && !i.leaving);
        assert_eq!(i.touch, None);
        assert_eq!((i.offset, i.tilt), (128, 0));
    }

    /// The default regimen is kinetic and complete; layer bits are distinct (REQ-C-02).
    #[test]
    fn regimen_fields() {
        let r = Regimen::default();
        assert_eq!(r.stilus, Stilus::Creta);
        assert_eq!(
            (r.strata, r.tick_ms, r.interval_ms, r.lines),
            (4, 42, 42, 6)
        );
        assert_eq!(r.layers, LAYERS_ALL);
        let bits = [
            CAELUM,
            MARGO,
            ANNALES,
            PERSONAE,
            RUINAE,
            IMPOSSIBILIA,
            HEXAGRAMMA,
            PUNCTA,
        ];
        assert_eq!(bits.iter().fold(0u8, |a, b| a | b), 255);
        for (i, a) in bits.iter().enumerate() {
            assert_eq!(a.count_ones(), 1);
            for b in bits.iter().skip(i.wrapping_add(1)) {
                assert_eq!(a & b, 0);
            }
        }
        assert!(r.has(ANNALES) && !Regimen { layers: 0, ..r }.has(ANNALES));
    }

    /// Announcements compose by union and are queried by bit.
    #[test]
    fn nuntius_bits() {
        let n = Nuntius::NIHIL.cum(Nuntius::CLINAMEN).cum(Nuntius::RARUM);
        assert!(n.contains(Nuntius::CLINAMEN) && n.contains(Nuntius::RARUM));
        assert!(!n.contains(Nuntius::AGNITIO));
        assert_eq!(n.0, 2 | 32);
        assert_eq!(Nuntius::LAPSUS.0, 64);
    }

    /// Surfaces refuse buffers shorter than their dimensions (REQ-H-02).
    #[test]
    fn surfaces_guard_their_slices() {
        let mut small = [0u8; 10];
        assert!(Tabula::new(4, 3, &mut small).is_none());
        let mut exact = [0u8; 12];
        let t = Tabula::new(4, 3, &mut exact);
        assert!(t.is_some());
        let mut cells = [0u16; 5];
        assert!(Cellae::new(3, 2, &mut cells).is_none());
        let mut cells6 = [0u16; 6];
        assert!(Cellae::new(3, 2, &mut cells6).is_some());
        let mut zero = [0u8; 0];
        assert!(Tabula::new(0, 0, &mut zero).is_some());
    }
}
