//! The clock, and where the sun is on it.
//!
//! A world with no night has nowhere for a lamp to matter, and until this the
//! sun here was a `const` that six call sites each normalised their own copy
//! of. One answer now, derived from one clock, which is the same rule the
//! pointer and the binding table already keep: a fact written in six places is
//! a fact five of them will eventually have wrong.
//!
//! It lives in the core because what time it is and where the sun is are facts
//! a second client would have to agree about to the frame, and because they
//! depend on nothing but arithmetic.

use glam::{Quat, Vec3};
use std::f64::consts::{FRAC_PI_2, TAU as TAU64};

/// How long a whole day takes, seconds. Forty-eight minutes: a minute of play
/// is half an hour of world, which is the owner's number. This is the SOLAR
/// day, noon to noon, which is what the clock's hours count.
pub const DAY_S: f32 = 2880.0;

/// How many of those days the planet takes to go round its sun: the owner's
/// number. It turns one more time than this against the stars in a year,
/// because each day it has moved a little way round its orbit and has to turn
/// that much further to bring the sun back overhead.
pub const YEAR_DAYS: f64 = 100.0;

/// The hour the world opens on, in 0..24. Mid-morning: the sun is up, it is
/// plainly climbing, and nothing has to be waited out to see the world lit.
pub const START_HOUR: f32 = 9.0;

/// The planet's obliquity, radians: how far its equator leans off its orbit,
/// and so how far north and south of the equator the sun wanders over a year.
/// Earth's 23.5 degrees.
pub const TILT: f32 = 0.41;

/// Where the noon sun stands on day 0, the northern summer solstice the world
/// opens on: at [`TILT`]'s latitude, on the azimuth every capture has been lit
/// from. Day 0 IS the sky this module had before the planet had an orbit, so
/// nothing framed against it moved.
pub const SUN_FIXED: Vec3 = Vec3::new(0.807, 0.398, 0.435);

/// The orbital longitude of day 0: the northern summer solstice.
const START_LONGITUDE: f64 = FRAC_PI_2;

/// What time it is: world seconds since midnight of day 0.
///
/// `f64`, because it is orbital time and grows without bound: at `f32` a
/// world a few hundred days old could no longer tell one frame from the next.
/// The hour, the day and the season are all derived from it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clock {
    pub seconds: f64,
}

impl Default for Clock {
    fn default() -> Self {
        Self::at_hour(START_HOUR)
    }
}

impl Clock {
    /// This hour of day 0.
    pub fn at_hour(hour: f32) -> Self {
        Self::at(0, hour)
    }

    /// This hour of this day.
    pub fn at(day: u32, hour: f32) -> Self {
        let hour = if hour.is_finite() { hour } else { START_HOUR };
        Self {
            seconds: (day as f64 + (hour as f64 / 24.0).rem_euclid(1.0)) * DAY_S as f64,
        }
    }

    /// Advance by `seconds` of wall time. A non-finite step changes nothing.
    pub fn advance(&mut self, seconds: f32) {
        if seconds.is_finite() {
            self.seconds += seconds as f64;
        }
    }

    /// The solar time of day, 0..1, where 0 is midnight and 0.5 is noon.
    pub fn fraction(self) -> f32 {
        (self.seconds / DAY_S as f64).rem_euclid(1.0) as f32
    }

    pub fn hour(self) -> f32 {
        self.fraction() * 24.0
    }

    /// Which day it is, counting from 0.
    pub fn day(self) -> u64 {
        (self.seconds / DAY_S as f64).max(0.0).floor() as u64
    }

    /// How far round the year, 0..1, from the northern summer solstice.
    pub fn year_fraction(self) -> f64 {
        (self.seconds / DAY_S as f64 / YEAR_DAYS).rem_euclid(1.0)
    }

    /// Where the sun is in the SYSTEM frame the stars are fixed in: along the
    /// ecliptic at the planet's orbital longitude, the ecliptic leaning
    /// [`TILT`] off the equator about +X.
    pub fn sun_in_system(self) -> Vec3 {
        let longitude = TAU64 * self.year_fraction() + START_LONGITUDE;
        let (sin_l, cos_l) = longitude.sin_cos();
        let (sin_t, cos_t) = (TILT as f64).sin_cos();
        Vec3::new(cos_l as f32, (sin_t * sin_l) as f32, (cos_t * sin_l) as f32)
    }

    /// The sun's latitude, radians: `TILT` north at the northern solstice, 0
    /// at the equinoxes, `TILT` south at the southern solstice.
    pub fn declination(self) -> f32 {
        self.sun_in_system().y.clamp(-1.0, 1.0).asin()
    }

    /// The planet's rotation about its pole, +Y, in the system frame.
    ///
    /// This is the ONE rotation. The sun, the star field and the moon are
    /// fixed directions in the system frame, and every one of them reaches the
    /// planet's frame through [`sky_from_system`](Self::sky_from_system), so a
    /// frame in which the sun turned and the stars did not is not expressible.
    /// Tenebris's `orbit.rs` keeps the same model: pin the body, rotate the
    /// whole sky by its spin.
    pub fn spin(self) -> Quat {
        self.sky_from_system().inverse()
    }

    /// What carries a system-frame direction into the planet's frame.
    ///
    /// Two parts about the pole, composed as one angle: the part that carries
    /// the sun's right ascension for the season onto the noon azimuth, and the
    /// hour angle for the solar time. Over a year the first part goes round
    /// once, which is the extra turn against the stars.
    pub fn sky_from_system(self) -> Quat {
        let sun = self.sun_in_system();
        let right_ascension = (sun.z as f64).atan2(sun.x as f64);
        let noon_azimuth = (SUN_FIXED.z as f64).atan2(SUN_FIXED.x as f64);
        let hour_angle = (self.fraction() as f64 - 0.5) * TAU64;
        let angle = (hour_angle + right_ascension - noon_azimuth).rem_euclid(TAU64);
        Quat::from_axis_angle(Vec3::Y, angle as f32)
    }

    /// Where the sun is, as a unit vector in the planet's own frame.
    pub fn sun(self) -> Vec3 {
        (self.sky_from_system() * self.sun_in_system()).normalize()
    }

    /// How high the sun stands over a point on the surface, as the cosine
    /// between straight up there and the sun. Positive is day.
    pub fn elevation(self, up: Vec3) -> f32 {
        self.sun().dot(up.normalize_or(Vec3::Y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_day_comes_round_and_the_clock_never_leaves_its_range() {
        let mut clock = Clock::at_hour(0.0);
        clock.advance(DAY_S);
        let f = clock.fraction();
        assert!(f.min(1.0 - f) < 1e-4, "a whole day is a whole turn: {f}");
        clock.advance(DAY_S * 3.5);
        assert!(
            (0.0..1.0).contains(&clock.fraction()),
            "{}",
            clock.fraction()
        );
        let before = clock;
        clock.advance(f32::NAN);
        assert_eq!(clock, before, "a bad step changes nothing");
    }

    /// The sun is a unit vector at every hour, which every consumer assumes
    /// and none of them checks.
    #[test]
    fn the_sun_is_a_direction_at_every_hour() {
        for step in 0..48 {
            let clock = Clock::at_hour(step as f32 * 0.5);
            let sun = clock.sun();
            assert!(sun.is_finite(), "{sun:?}");
            assert!((sun.length() - 1.0).abs() < 1e-5, "{sun:?}");
        }
    }

    /// Noon and midnight are opposite, which is what makes a night a night:
    /// the point the sun is over at noon is the point it is under at
    /// midnight.
    #[test]
    fn midnight_is_the_other_side_of_noon() {
        let noon = Clock::at_hour(12.0).sun();
        let midnight = Clock::at_hour(0.0).sun();
        assert!(
            noon.dot(midnight) < -0.5,
            "noon {noon:?} against midnight {midnight:?}"
        );
        let up = noon;
        assert!(Clock::at_hour(12.0).elevation(up) > 0.9, "noon is overhead");
        assert!(
            Clock::at_hour(0.0).elevation(up) < -0.5,
            "midnight is under"
        );
    }

    /// Noon is the fixed sun exactly, so every capture framed against the old
    /// constant still reads, and a quarter turn later the sun and a star have
    /// both moved a quarter turn about the pole the same way: one spin.
    #[test]
    fn the_sky_turns_as_one_about_the_pole() {
        let noon = Clock::at_hour(12.0);
        assert!(
            noon.sun().dot(SUN_FIXED.normalize()) > 0.9999,
            "{:?}",
            noon.sun()
        );
        // At the tilt's latitude, on the old light's azimuth.
        let fixed = SUN_FIXED.normalize();
        // Noon of day 0 is half a day into the year: a hair off the solstice.
        assert!((noon.declination() - TILT).abs() < 1e-3);
        assert!(
            (fixed.y.asin() - TILT).abs() < 0.01,
            "latitude {}",
            fixed.y.asin()
        );
        let old = Vec3::new(0.65, 0.0, 0.35).normalize();
        assert!(Vec3::new(fixed.x, 0.0, fixed.z).normalize().dot(old) > 0.9999);
        let later = Clock::at_hour(18.0);
        let star = Vec3::new(0.3, -0.2, 0.9).normalize();
        let star_then = later.sky_from_system() * star;
        // Both are a quarter turn about Y from where they were at noon.
        let quarter = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2);
        // A quarter of a day also carries the planet a four-hundredth of the
        // way round its orbit, which turns the sky a further degree.
        assert!((quarter * noon.sun()).dot(later.sun()) > 0.999);
        assert!((quarter * (noon.sky_from_system() * star)).dot(star_then) > 0.999);
        // Latitude is preserved: the pole is the axis.
        assert!((star_then.y - star.y).abs() < 1e-5);
    }

    /// The sun MOVES, and by an amount a player would notice over a minute.
    /// It was a constant before this, which is the whole of what this module
    /// is for.
    #[test]
    fn the_sun_moves_through_the_day() {
        let mut clock = Clock::at_hour(6.0);
        let dawn = clock.sun();
        clock.advance(60.0);
        let later = clock.sun();
        // A minute is half an hour of world: 7.5 degrees, cos 0.9914.
        assert!(
            dawn.dot(later) < 0.995,
            "a minute moved the sun by nothing: {dawn:?} {later:?}"
        );
        assert!(dawn.dot(later) > 0.0, "and not by half the sky");
    }

    fn degrees(radians: f32) -> f32 {
        radians.to_degrees()
    }

    /// The solstices and the equinoxes fall where a 100-day year puts them.
    #[test]
    fn the_sun_swings_north_and_south_over_a_year() {
        let at = |day: u32| Clock::at(day, 0.0).declination();
        let tilt = degrees(TILT);
        assert!((degrees(at(0)) - tilt).abs() < 0.2, "{}", degrees(at(0)));
        assert!((degrees(at(50)) + tilt).abs() < 0.2, "{}", degrees(at(50)));
        assert!(degrees(at(25)).abs() < 0.2, "{}", degrees(at(25)));
        assert!(degrees(at(75)).abs() < 0.2, "{}", degrees(at(75)));
    }

    /// Noon is the middle of the day on every day of the year: the sun is at
    /// its highest on the noon meridian, whatever the season.
    #[test]
    fn noon_is_noon_all_year() {
        let meridian = Vec3::new(SUN_FIXED.x, 0.0, SUN_FIXED.z).normalize();
        for day in [0, 13, 37, 50, 88] {
            let noon = Clock::at(day, 12.0).sun();
            let flat = Vec3::new(noon.x, 0.0, noon.z).normalize();
            assert!(flat.dot(meridian) > 0.99999, "day {day}: {noon:?}");
            let mut best = (f32::MIN, 0.0);
            for step in 0..96 {
                let hour = step as f32 * 0.25;
                let e = Clock::at(day, hour).elevation(meridian);
                if e > best.0 {
                    best = (e, hour);
                }
            }
            assert!(
                (best.1 - 12.0).abs() < 0.26,
                "day {day} peaks at {}",
                best.1
            );
        }
    }

    /// A solar day brings the sun back; a year of them turns the stars one
    /// more time than there are days, so the whole sky is back after a year.
    #[test]
    fn a_year_turns_the_stars_once_more_than_the_days() {
        let start = Clock::at(0, 12.0);
        let next = Clock::at(1, 12.0);
        assert!(start.sun().dot(next.sun()) > 0.9995);
        let year = Clock::at(YEAR_DAYS as u32, 12.0);
        let star = Vec3::new(0.3, -0.2, 0.9).normalize();
        assert!((start.sky_from_system() * star).dot(year.sky_from_system() * star) > 0.99999);
        assert!(start.sun().dot(year.sun()) > 0.99999);
        // Each solar day turns the stars a whole turn plus a little more (a
        // little more near the solstices, less near the equinoxes: the
        // equation of time). Summed over the year the extras are one turn.
        let mut extra = 0.0f64;
        let azimuth = |q: Quat| {
            let x = q * Vec3::X;
            (x.z as f64).atan2(x.x as f64)
        };
        for day in 0..YEAR_DAYS as u32 {
            let a = azimuth(Clock::at(day, 12.0).sky_from_system());
            let b = azimuth(Clock::at(day + 1, 12.0).sky_from_system());
            let mut step = b - a;
            while step > std::f64::consts::PI {
                step -= std::f64::consts::TAU;
            }
            while step <= -std::f64::consts::PI {
                step += std::f64::consts::TAU;
            }
            extra += step;
        }
        assert!(
            (extra.abs() - std::f64::consts::TAU).abs() < 1e-3,
            "a year's extra turning is {extra} radians"
        );
    }
}
