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
use std::f32::consts::TAU;

/// How long a whole day takes, seconds. Forty-eight minutes: a minute of play
/// is half an hour of world, which is the owner's number.
pub const DAY_S: f32 = 2880.0;

/// The hour the world opens on, in 0..24. Mid-morning: the sun is up, it is
/// plainly climbing, and nothing has to be waited out to see the world lit.
pub const START_HOUR: f32 = 9.0;

/// How far the sun stands off the planet's equator, radians: the sub-solar
/// latitude, Earth's own 23.5 degrees at a solstice. A sun over the equator
/// would light every latitude alike; a sun at 45 degrees would never set
/// north of it, which is what the old fixed light's latitude was.
pub const TILT: f32 = 0.41;

/// Where the sun IS, in the system frame the sky is fixed in. It does not move;
/// the planet turns under it. The azimuth is the old fixed light's, so noon
/// faces where every capture has been lit from, at [`TILT`]'s latitude.
pub const SUN_FIXED: Vec3 = Vec3::new(0.807, 0.398, 0.435);

/// What time it is, as a fraction of a day in 0..1 where 0 is midnight.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clock {
    pub fraction: f32,
}

impl Default for Clock {
    fn default() -> Self {
        Self::at_hour(START_HOUR)
    }
}

impl Clock {
    pub fn at_hour(hour: f32) -> Self {
        Self {
            fraction: (hour / 24.0).rem_euclid(1.0),
        }
    }

    /// Advance by `seconds` of wall time.
    pub fn advance(&mut self, seconds: f32) {
        if !seconds.is_finite() || DAY_S <= 0.0 {
            return;
        }
        self.fraction = (self.fraction + seconds / DAY_S).rem_euclid(1.0);
    }

    pub fn hour(self) -> f32 {
        self.fraction * 24.0
    }

    /// The planet's rotation about its pole, +Y, in the system frame: one
    /// full turn a day, noon meridian toward the sun at fraction 0.5.
    ///
    /// This is the ONE rotation. The sun, the star field and the moon are
    /// fixed directions in the system frame, and every one of them reaches the
    /// planet's frame through [`sky_from_system`](Self::sky_from_system), so a
    /// frame in which the sun turned and the stars did not is not expressible.
    /// Tenebris's `orbit.rs` keeps the same model: pin the body, rotate the
    /// whole sky by its spin.
    pub fn spin(self) -> Quat {
        let angle = (self.fraction - 0.5) * TAU;
        Quat::from_axis_angle(Vec3::Y, -angle)
    }

    /// What carries a system-frame direction into the planet's frame: the
    /// inverse of the spin.
    pub fn sky_from_system(self) -> Quat {
        self.spin().inverse()
    }

    /// Where the sun is, as a unit vector in the planet's own frame: the fixed
    /// sun carried in by the spin. At fraction 0.5 - noon - it is
    /// [`SUN_FIXED`] itself, and at 0 it is on the far side.
    pub fn sun(self) -> Vec3 {
        (self.sky_from_system() * SUN_FIXED.normalize()).normalize()
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
        assert!(
            (clock.fraction - 0.0).abs() < 1e-4,
            "a whole day is a whole turn: {}",
            clock.fraction
        );
        clock.advance(DAY_S * 3.5);
        assert!((0.0..1.0).contains(&clock.fraction), "{}", clock.fraction);
        clock.advance(f32::NAN);
        assert!(clock.fraction.is_finite(), "a bad step changes nothing");
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
        assert!((quarter * noon.sun()).dot(later.sun()) > 0.9999);
        assert!((quarter * (noon.sky_from_system() * star)).dot(star_then) > 0.9999);
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
}
