//! Taming by feeding (decision 3; S4): trust that feeds fill, a spook
//! costs and time wears away. A wild animal is derived, so its trust is not
//! saved: walk away half-way and it is a stranger again next time.

use super::TamingSettings;

/// Trust is full at this.
pub const FULL: f32 = 100.0;

/// How far one wild animal trusts the player.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Trust {
    pub value: f32,
}

impl Trust {
    /// One feed: the species' gain times the animal's trust factor. True
    /// when it fills trust.
    pub fn feed(&mut self, gain: f32, factor: f32) -> bool {
        self.value = (self.value + gain * factor).min(FULL);
        self.full()
    }

    /// The player moved inside its fear radius.
    pub fn spook(&mut self, settings: &TamingSettings) {
        self.value = (self.value - settings.spook_cost).max(0.0);
    }

    /// `hours` of world time passed without it being tamed.
    pub fn decay(&mut self, hours: f32, settings: &TamingSettings) {
        if self.full() {
            return;
        }
        self.value = (self.value - settings.decay_per_hour * hours.max(0.0)).max(0.0);
    }

    pub fn full(&self) -> bool {
        self.value >= FULL
    }
}

/// Hearts shown over an animal fed at this trust (`taming-hearts`): one a
/// fifth of full trust, at least one; 5 at full trust.
pub fn hearts(trust: f32) -> u8 {
    ((trust / (FULL / 5.0)).ceil() as i32).clamp(1, 5) as u8
}

/// How near the player may be before a wild animal bolts, standing still
/// being the crouch the game does not have yet (S4). With its food in hand
/// and still, nothing frightens it: it comes to the hand. With food and
/// moving, the crouched radius; empty-handed, the full one while moving and
/// the crouched one while still.
pub fn fear_radius(settings: &TamingSettings, moving: bool, food_in_hand: bool) -> f32 {
    match (food_in_hand, moving) {
        (true, false) => 0.0,
        (true, true) | (false, false) => settings.fear_still_m,
        (false, true) => settings.fear_moving_m,
    }
}

#[cfg(test)]
mod heart_tests {
    use super::*;

    /// `taming-hearts`: more trust, more hearts; one at the least, five full.
    #[test]
    fn more_trust_shows_more_hearts() {
        assert_eq!(hearts(0.0), 1);
        assert_eq!(hearts(25.0), 2);
        assert_eq!(hearts(50.0), 3);
        assert_eq!(hearts(75.0), 4);
        assert_eq!(hearts(FULL), 5);
        assert_eq!(hearts(250.0), 5);
    }
}
