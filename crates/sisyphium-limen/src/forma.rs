//! Checked representations of the documented C layouts (REQ-H-03).

use sisyphium::{
    consilium::INTERVALS,
    hospes::{Facultates, Indicia, Lingua, Regimen, Stilus},
};

/// C representation of `Facultates`; explicit reserved bytes (REQ-H-03).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SyFacultates {
    /// Frame width, positive.
    pub width: u16,
    /// Frame height, positive.
    pub height: u16,
    /// Color depth: 1, 4, 8 or 24.
    pub color_bits: u8,
    /// Maximum presentation rate; zero means static.
    pub max_hz: u8,
    /// Boolean capability, exactly zero or one.
    pub touch: u8,
    /// Boolean capability, exactly zero or one.
    pub motion: u8,
    /// Boolean capability, exactly zero or one.
    pub occlusion: u8,
    /// Boolean capability, exactly zero or one.
    pub persistence: u8,
    /// Boolean capability, exactly zero or one.
    pub cells: u8,
    /// Boolean capability, exactly zero or one.
    pub australis: u8,
    /// UTC offset in minutes.
    pub tz_offset_min: i16,
    /// Language: English zero, Portuguese one.
    pub lingua: u8,
    /// Relative compute budget.
    pub compute: u8,
}

/// C representation of `Indicia`; explicit reserved bytes (REQ-H-03).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SyIndicia {
    /// UTC epoch in milliseconds.
    pub epoch_ms: i64,
    /// Available energy.
    pub energy: u8,
    /// External agitation.
    pub seism: u8,
    /// Interaction regularity.
    pub automatism: u8,
    /// Reserved; must be zero.
    pub reservatum0: u8,
    /// Idle seconds; maximum means unknown.
    pub idle_s: u32,
    /// Time since return; maximum means never.
    pub returned_ms: u32,
    /// Occluded flag, zero or one.
    pub occluded: u8,
    /// Leaving flag, zero or one.
    pub leaving: u8,
    /// Normalized horizontal touch.
    pub touch_x: u8,
    /// Normalized vertical touch.
    pub touch_y: u8,
    /// Touch present flag, zero or one.
    pub has_touch: u8,
    /// Horizontal launcher offset.
    pub offset: u8,
    /// Device roll.
    pub tilt: i8,
    /// Reserved; all bytes must be zero.
    pub reservata: [u8; 5],
}

/// C representation of `Regimen`; explicit reserved bytes (REQ-H-03).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SyRegimen {
    /// Visual idiom: Creta zero through Bicolor three.
    pub stilus: u8,
    /// Parallax strata, 1 through 4.
    pub strata: u8,
    /// Reserved; all bytes must be zero.
    pub reservata0: [u8; 2],
    /// Tick cadence: 42, 125, 250 or 60000 ms.
    pub tick_ms: u32,
    /// Presentation interval; member of `INTERVALS` at least `tick_ms`.
    pub interval_ms: u32,
    /// Enabled layer bits.
    pub layers: u8,
    /// Chronicle lines, zero through six.
    pub lines: u8,
    /// Reserved; all bytes must be zero.
    pub reservata1: [u8; 2],
}

/// Decodes flags without creating invalid Rust booleans (REQ-H-03).
fn flag(value: u8) -> Result<bool, ()> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(()),
    }
}

impl TryFrom<SyFacultates> for Facultates {
    type Error = ();
    /// Rejects invalid dimensions, flags, depth and language before core entry (REQ-H-03).
    fn try_from(f: SyFacultates) -> Result<Self, Self::Error> {
        if f.width == 0 || f.height == 0 || !matches!(f.color_bits, 1 | 4 | 8 | 24) {
            return Err(());
        }
        Ok(Self {
            width: f.width,
            height: f.height,
            color_bits: f.color_bits,
            max_hz: f.max_hz,
            touch: flag(f.touch)?,
            motion: flag(f.motion)?,
            occlusion: flag(f.occlusion)?,
            persistence: flag(f.persistence)?,
            cells: flag(f.cells)?,
            australis: flag(f.australis)?,
            tz_offset_min: f.tz_offset_min,
            lingua: match f.lingua {
                0 => Lingua::En,
                1 => Lingua::Pt,
                _ => return Err(()),
            },
            compute: f.compute,
        })
    }
}
impl From<Facultates> for SyFacultates {
    /// Encodes each boolean and language explicitly (REQ-H-03).
    fn from(f: Facultates) -> Self {
        Self {
            width: f.width,
            height: f.height,
            color_bits: f.color_bits,
            max_hz: f.max_hz,
            touch: u8::from(f.touch),
            motion: u8::from(f.motion),
            occlusion: u8::from(f.occlusion),
            persistence: u8::from(f.persistence),
            cells: u8::from(f.cells),
            australis: u8::from(f.australis),
            tz_offset_min: f.tz_offset_min,
            lingua: f.lingua as u8,
            compute: f.compute,
        }
    }
}
impl TryFrom<SyIndicia> for Indicia {
    type Error = ();
    /// Rejects invalid flags and nonzero reserved bytes (REQ-H-03).
    fn try_from(i: SyIndicia) -> Result<Self, Self::Error> {
        if i.reservatum0 != 0 || i.reservata != [0; 5] {
            return Err(());
        }
        Ok(Self {
            epoch_ms: i.epoch_ms,
            energy: i.energy,
            seism: i.seism,
            automatism: i.automatism,
            idle_s: i.idle_s,
            returned_ms: i.returned_ms,
            occluded: flag(i.occluded)?,
            leaving: flag(i.leaving)?,
            touch: if flag(i.has_touch)? {
                Some((i.touch_x, i.touch_y))
            } else {
                None
            },
            offset: i.offset,
            tilt: i.tilt,
        })
    }
}
impl From<Indicia> for SyIndicia {
    /// Encodes measurements with zeroed reserved bytes (REQ-H-03).
    fn from(i: Indicia) -> Self {
        let (x, y) = i.touch.unwrap_or_default();
        Self {
            epoch_ms: i.epoch_ms,
            energy: i.energy,
            seism: i.seism,
            automatism: i.automatism,
            reservatum0: 0,
            idle_s: i.idle_s,
            returned_ms: i.returned_ms,
            occluded: u8::from(i.occluded),
            leaving: u8::from(i.leaving),
            touch_x: x,
            touch_y: y,
            has_touch: u8::from(i.touch.is_some()),
            offset: i.offset,
            tilt: i.tilt,
            reservata: [0; 5],
        }
    }
}
impl TryFrom<SyRegimen> for Regimen {
    type Error = ();
    /// Validates cadence, ranges, reserved bytes and idiom (REQ-H-03).
    fn try_from(r: SyRegimen) -> Result<Self, Self::Error> {
        if r.reservata0 != [0; 2]
            || r.reservata1 != [0; 2]
            || !(1..=4).contains(&r.strata)
            || r.lines > 6
            || !matches!(r.tick_ms, 42 | 125 | 250 | 60_000)
            || r.interval_ms < r.tick_ms
            || !INTERVALS.contains(&r.interval_ms)
        {
            return Err(());
        }
        Ok(Self {
            stilus: match r.stilus {
                0 => Stilus::Creta,
                1 => Stilus::Musivum,
                2 => Stilus::Littera,
                3 => Stilus::Bicolor,
                _ => return Err(()),
            },
            strata: r.strata,
            tick_ms: r.tick_ms,
            interval_ms: r.interval_ms,
            layers: r.layers,
            lines: r.lines,
        })
    }
}
impl From<Regimen> for SyRegimen {
    /// Emits the ABI regimen with every reserved byte zeroed (REQ-H-03).
    fn from(r: Regimen) -> Self {
        Self {
            stilus: r.stilus as u8,
            strata: r.strata,
            reservata0: [0; 2],
            tick_ms: r.tick_ms,
            interval_ms: r.interval_ms,
            layers: r.layers,
            lines: r.lines,
            reservata1: [0; 2],
        }
    }
}
