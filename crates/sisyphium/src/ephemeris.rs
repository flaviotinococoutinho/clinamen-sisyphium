//! `ephemeris`: time (level L1). The circadian, seasonal and lunar phases of the world, the
//! night, and the astronomical festival days, all derived from the epoch with integer
//! arithmetic (REQ-L1-06).

/// The synodic month in seconds: 29.530 589 days (Meeus, 1991).
pub const SYNODIC_S: i64 = 2_551_443;
/// A new moon: 2000-01-06 18:14 UTC, in seconds since the epoch.
pub const LUNA_EPOCH_S: i64 = 947_182_440;

/// The astronomical state of the world at one instant (REQ-L1-06).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Ephemeris {
    /// Circadian phase, 0 at local midnight, 128 at local noon.
    pub phase: u8,
    /// Seasonal phase, 0 at the March equinox (day 79) in the boreal hemisphere; +128 austral.
    pub season: u8,
    /// Lunar phase, 0 at new moon, 128 at full moon.
    pub luna: u8,
    /// Night: the phase lies outside 64..=191 (06:00 to 18:00).
    pub nocturnal: bool,
    /// Festival day: within one day of a solstice or equinox, or within 4/256 of a new or full moon.
    pub festum: bool,
    /// Day of the local year, 1..=366.
    pub day_of_year: u16,
}

/// Day of the year (1..=366) of the day counted from 1970-01-01 in the proleptic Gregorian
/// calendar, by the integer civil-date conversion on 400-year eras (Fliegel and Van Flandern,
/// 1968, with the era shifted to March).
#[must_use]
pub const fn day_of_year(days: i64) -> u16 {
    let z = days.wrapping_add(719_468);
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = doe
        .wrapping_sub(doe.wrapping_div(1_460))
        .wrapping_add(doe.wrapping_div(36_524))
        .wrapping_sub(doe.wrapping_div(146_096))
        .wrapping_div(365);
    let doy_march = doe.wrapping_sub(
        yoe.wrapping_mul(365)
            .wrapping_add(yoe.wrapping_div(4))
            .wrapping_sub(yoe.wrapping_div(100)),
    );
    let mp = doy_march.wrapping_mul(5).wrapping_add(2).wrapping_div(153);
    let day = doy_march
        .wrapping_sub(mp.wrapping_mul(153).wrapping_add(2).wrapping_div(5))
        .wrapping_add(1);
    let month = if mp < 10 {
        mp.wrapping_add(3)
    } else {
        mp.wrapping_sub(9)
    };
    let year = yoe
        .wrapping_add(era.wrapping_mul(400))
        .wrapping_add(if month <= 2 { 1 } else { 0 });
    let leap = (year.rem_euclid(4) == 0 && year.rem_euclid(100) != 0) || year.rem_euclid(400) == 0;
    let before = month.wrapping_mul(367).wrapping_sub(362).wrapping_div(12);
    let february = if month > 2 {
        if leap { 1 } else { 2 }
    } else {
        0
    };
    before.wrapping_sub(february).wrapping_add(day) as u16
}

/// Whether `day` lies within one day of `anchor`.
const fn near(day: u16, anchor: u16) -> bool {
    day.abs_diff(anchor) <= 1
}

/// The astronomical state at `epoch_ms` (UTC milliseconds), for a clock offset in minutes and
/// a hemisphere (REQ-L1-06). Negative epochs use Euclidean division.
#[must_use]
pub const fn ephemeris(epoch_ms: i64, tz_offset_min: i16, australis: bool) -> Ephemeris {
    let seconds = epoch_ms.div_euclid(1_000);
    let local_s = seconds.saturating_add((tz_offset_min as i64).wrapping_mul(60));
    let days = local_s.div_euclid(86_400);
    let second = local_s.rem_euclid(86_400);
    let phase = second.wrapping_mul(256).wrapping_div(86_400) as u8;
    let day_of_year = day_of_year(days);
    let since_equinox = (day_of_year as i64).wrapping_sub(79).rem_euclid(365);
    let boreal = since_equinox.wrapping_mul(256).wrapping_div(365) as u8;
    let season = if australis {
        boreal.wrapping_add(128)
    } else {
        boreal
    };
    let luna = seconds
        .wrapping_sub(LUNA_EPOCH_S)
        .rem_euclid(SYNODIC_S)
        .wrapping_mul(256)
        .wrapping_div(SYNODIC_S) as u8;
    let nocturnal = phase < 64 || phase > 191;
    let solar = near(day_of_year, 79)
        || near(day_of_year, 172)
        || near(day_of_year, 265)
        || near(day_of_year, 355);
    let lunar = luna <= 4 || luna >= 252 || (luna >= 124 && luna <= 132);
    Ephemeris {
        phase,
        season,
        luna,
        nocturnal,
        festum: solar || lunar,
        day_of_year,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::arithmetic_side_effects)]
    use super::*;

    /// Known dates: a solstice at local noon, the lunar epoch, a solstice night (REQ-L1-06).
    #[test]
    fn ephemeris_known_dates() {
        let noon = ephemeris(1_782_054_000_000, -180, false);
        assert_eq!(noon.day_of_year, 172);
        assert!(noon.festum);
        assert_eq!(noon.phase, 128);
        assert!(!noon.nocturnal);
        assert_eq!(noon.season, 65);
        assert_eq!(noon.luna, 55);
        let austral = ephemeris(1_782_054_000_000, -180, true);
        assert_eq!(austral.season, 193);
        let lunar_epoch = ephemeris(947_182_440_000, 0, false);
        assert_eq!(lunar_epoch.luna, 0);
        assert!(lunar_epoch.festum);
        let night = ephemeris(1_797_831_000_000, -180, false);
        assert!(night.nocturnal);
        assert!(night.festum);
        assert_eq!(night.day_of_year, 355);
        assert_eq!(night.phase, 26);
        let equinox = ephemeris(1_774_008_000_000, 0, false);
        assert_eq!((equinox.day_of_year, equinox.season), (79, 0));
        assert_eq!(ephemeris(1_774_008_000_000, 0, true).season, 128);
        assert!(equinox.festum);
        let plain = ephemeris(1_774_008_000_000 + 10 * 86_400_000, 0, false);
        assert!(!plain.festum);
    }

    /// Negative epochs and positive offsets use Euclidean division (REQ-L1-06).
    #[test]
    fn negative_epoch_and_offset() {
        let e = ephemeris(-3_600_000, 60, false);
        assert_eq!((e.day_of_year, e.phase), (1, 0));
        assert!(e.nocturnal);
        let west = ephemeris(-1, -180, false);
        assert_eq!((west.phase, west.day_of_year), (223, 365));
        assert!(west.nocturnal);
        let before = ephemeris(-86_400_000 * 400, 0, false);
        assert!(before.day_of_year >= 1 && before.day_of_year <= 366);
    }

    /// The civil calendar conversion is exact on leap-year boundaries.
    #[test]
    fn day_of_year_leap_years() {
        assert_eq!(day_of_year(0), 1);
        assert_eq!(day_of_year(365), 1);
        assert_eq!(day_of_year(365 + 365), 1);
        assert_eq!(day_of_year(365 + 365 + 59), 60);
        assert_eq!(day_of_year(365 + 365 + 365), 366);
        assert_eq!(day_of_year(365 + 365 + 366), 1);
        assert_eq!(day_of_year(10_957), 1);
    }
}
