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

use glam::Vec3;
use std::f32::consts::TAU;

/// How long a whole day takes, seconds. Six minutes: long enough that a night
/// is something a player plans around and short enough that they see one in a
/// sitting, which is the reference's own bargain at a different number.
pub const DAY_S: f32 = 360.0;

/// The hour the world opens on, in 0..24. Mid-morning: the sun is up, it is
/// plainly climbing, and nothing has to be waited out to see the world lit.
pub const START_HOUR: f32 = 9.0;

/// How far the sun's arc tilts off the pole, radians. A world whose sun ran
/// exactly over the equator would have a terminator that never moved off one
/// meridian; a tilt is what makes the light land differently at different
/// latitudes.
pub const TILT: f32 = 0.41;

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

    /// Where the sun is, as a unit vector in the planet's own frame.
    ///
    /// It goes round the tilted axis rather than round Y, so the terminator
    /// sweeps the surface instead of standing still on one meridian. At
    /// fraction 0.5 - noon - it is overhead at the sub-solar latitude, and at
    /// 0 it is on the far side.
    pub fn sun(self) -> Vec3 {
        let angle = (self.fraction - 0.5) * TAU;
        Vec3::new(
            angle.cos() * TILT.cos(),
            TILT.sin(),
            angle.sin() * TILT.cos(),
        )
        .normalize()
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

    /// The sun MOVES, and by an amount a player would notice over a minute.
    /// It was a constant before this, which is the whole of what this module
    /// is for.
    #[test]
    fn the_sun_moves_through_the_day() {
        let mut clock = Clock::at_hour(6.0);
        let dawn = clock.sun();
        clock.advance(60.0);
        let later = clock.sun();
        assert!(
            dawn.dot(later) < 0.999,
            "a minute moved the sun by nothing: {dawn:?} {later:?}"
        );
        assert!(dawn.dot(later) > 0.0, "and not by half the sky");
    }
}
