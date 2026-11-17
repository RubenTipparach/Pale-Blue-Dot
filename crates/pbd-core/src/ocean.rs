//! Light under water (`ocean-environments` group 4, design 2, 3 and B2-B4).
//!
//! Two coefficients, two jobs:
//!
//! - **downwelling** (`downwell_per_m`): daylight ARRIVING at a depth. It is
//!   what lights the seabed and the water round the eye, and what the depth
//!   zones are read from;
//! - **absorption** (`absorption_per_m`, in the app's water settings): the
//!   view's path through water, which this module does not need.
//!
//! The zones are defined by the light left, not by real-ocean metres, so they
//! fall inside the column (`column::BASE_M` never moves): sunlit while blue
//! keeps over 15%, twilight down to 1.7%, midnight below.
//!
//! Turbidity thickens the view toward the water's own colour. Until the
//! environment function exists (6.1) it is read off the water class under the
//! eye and the depth (design B3).
//!
//! Everything here is derived and never saved.

pub mod places;
pub mod plates;

use crate::fauna::WaterClass;
use serde::{Deserialize, Serialize};

/// The zones' edges, as the share of BLUE daylight left (blue reaches
/// deepest, so it decides when the water is dark).
pub const SUNLIT_ABOVE: f32 = 0.15;
pub const TWILIGHT_ABOVE: f32 = 0.017;

/// The depth zones (design 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    Sunlit,
    Twilight,
    Midnight,
}

/// The share of daylight left per colour at `depth_m` under the surface.
pub fn light_left(downwell_per_m: [f32; 3], depth_m: f32) -> [f32; 3] {
    let d = if depth_m.is_finite() {
        depth_m.max(0.0)
    } else {
        0.0
    };
    downwell_per_m.map(|k| (-k.max(0.0) * d).exp())
}

/// The darkest share of daylight a creature or a sea piece is drawn with,
/// so a shape still reads against the murk.
pub const UNDER_WATER_FLOOR: f32 = 0.04;

/// What the water makes of a colour `depth_m` under its surface
/// (`ocean-environments` B23): the terrain's own rule for the seabed
/// (`planet_surface.wgsl`, `mix(c*att, deep*light*att, (1-mean(att))*0.5)`)
/// with the light taken out, so a creature or a sea piece beside the bed is
/// tinted as the bed is. `att` is the daylight left ([`light_left`]),
/// floored at [`UNDER_WATER_FLOOR`] where it dims; the deep's share is the
/// terrain's, from the unfloored light. Linear RGB. At or above the surface
/// the colour is returned as it is.
pub fn under_water_albedo(
    colour: [f32; 3],
    downwell_per_m: [f32; 3],
    deep: [f32; 3],
    depth_m: f32,
) -> [f32; 3] {
    let att = light_left(downwell_per_m, depth_m);
    let share = (1.0 - (att[0] + att[1] + att[2]) / 3.0) * 0.5;
    std::array::from_fn(|i| {
        let tinted = colour[i] + (deep[i] - colour[i]) * share;
        tinted * att[i].max(UNDER_WATER_FLOOR)
    })
}

/// The zone a point `depth_m` under the surface is in.
pub fn zone(downwell_per_m: [f32; 3], depth_m: f32) -> Zone {
    let blue = light_left(downwell_per_m, depth_m)[2];
    if blue > SUNLIT_ABOVE {
        Zone::Sunlit
    } else if blue > TWILIGHT_ABOVE {
        Zone::Twilight
    } else {
        Zone::Midnight
    }
}

/// Where each zone starts going down, m: the twilight's top and the
/// midnight's top, from the blue coefficient.
pub fn zone_depths(downwell_per_m: [f32; 3]) -> (f32, f32) {
    let k = downwell_per_m[2].max(1e-6);
    (-SUNLIT_ABOVE.ln() / k, -TWILIGHT_ABOVE.ln() / k)
}

/// Turbidity per metre, toward the water's own colour, by environment
/// (design 2's table). `coast`, `ocean` and `river` are read today; the rest
/// wait for the environment function (6.1).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Turbidity {
    pub ocean: f32,
    pub coast: f32,
    pub river: f32,
    pub reef: f32,
    pub kelp: f32,
    pub polar: f32,
    pub marsh: f32,
    /// How long the eye's turbidity takes to ease to a new value, s.
    pub ease_s: f32,
}

impl Default for Turbidity {
    fn default() -> Self {
        Self {
            ocean: 0.022,
            coast: 0.07,
            river: 0.12,
            reef: 0.016,
            kelp: 0.05,
            polar: 0.03,
            marsh: 0.35,
            ease_s: 2.0,
        }
    }
}

impl Turbidity {
    /// The turbidity of the water over a point of this class and floor depth:
    /// a river its own; the sea from the coast's in the shallows to the open
    /// ocean's past the shelf, smoothly by depth so a swim out never steps.
    /// Dry land (`None`) reads as the coast's, the water nearest it.
    pub fn at(&self, class: Option<WaterClass>, depth_m: f32, limits: (f32, f32)) -> f32 {
        if class == Some(WaterClass::River) {
            return self.river;
        }
        let (shallows, shelf) = limits;
        let t = if depth_m.is_finite() && shelf > shallows {
            ((depth_m - shallows) / (shelf - shallows)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let t = t * t * (3.0 - 2.0 * t);
        self.coast + (self.ocean - self.coast) * t
    }

    /// One step of the eye's eased turbidity toward `target`.
    pub fn ease(&self, current: f32, target: f32, dt: f32) -> f32 {
        if !current.is_finite() || self.ease_s <= 0.0 {
            return target;
        }
        let k = 1.0 - (-dt.max(0.0) / self.ease_s).exp();
        current + (target - current) * k
    }

    pub fn validate(&self) -> Result<(), String> {
        let all = [
            self.ocean,
            self.coast,
            self.river,
            self.reef,
            self.kelp,
            self.polar,
            self.marsh,
            self.ease_s,
        ];
        if all.iter().all(|v| v.is_finite() && *v >= 0.0) {
            Ok(())
        } else {
            Err("turbidity: every value must be finite and not negative".into())
        }
    }
}

/// Ocean water's downwelling, per colour per metre (design 2).
pub const OCEAN_DOWNWELL_PER_M: [f32; 3] = [0.40, 0.065, 0.046];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::BASE_M;

    #[test]
    fn under_water_a_colour_dims_and_greys_toward_the_deep() {
        let down = OCEAN_DOWNWELL_PER_M;
        let deep = [0.004, 0.018, 0.040];
        let yellow = [0.85, 0.63, 0.23];
        assert_eq!(under_water_albedo(yellow, down, deep, 0.0), yellow);
        assert_eq!(under_water_albedo(yellow, down, deep, -3.0), yellow);
        let at_2 = under_water_albedo(yellow, down, deep, 2.0);
        let at_8 = under_water_albedo(yellow, down, deep, 8.0);
        for i in 0..3 {
            assert!(
                at_2[i] < yellow[i] && at_8[i] < at_2[i],
                "{at_2:?} {at_8:?}"
            );
        }
        // Red goes first: the yellow turns toward the water's green-blue.
        assert!(at_8[0] / at_8[1] < yellow[0] / yellow[1]);
        // The terrain's rule, worked by hand at 2 m.
        let att = light_left(down, 2.0);
        let share = (1.0 - (att[0] + att[1] + att[2]) / 3.0) * 0.5;
        for i in 0..3 {
            let want = (yellow[i] * (1.0 - share) + deep[i] * share) * att[i];
            assert!((at_2[i] - want).abs() < 1e-6);
        }
        // Never quite black, however deep.
        let abyss = under_water_albedo([1.0; 3], down, [1.0; 3], 500.0);
        assert!(abyss.iter().all(|&c| c >= UNDER_WATER_FLOOR * 0.999));
    }

    #[test]
    fn the_zones_fall_at_forty_and_ninety_metres_inside_the_column() {
        let k = OCEAN_DOWNWELL_PER_M;
        let at40 = light_left(k, 40.0)[2];
        let at90 = light_left(k, 90.0)[2];
        assert!(at40 >= SUNLIT_ABOVE, "15% or more at 40 m: {at40}");
        assert!(at90 <= TWILIGHT_ABOVE, "1.7% or less at 90 m: {at90}");
        let (twilight, midnight) = zone_depths(k);
        assert!((twilight - 41.2).abs() < 1.0, "twilight from {twilight} m");
        assert!((midnight - 88.6).abs() < 1.0, "midnight from {midnight} m");
        assert!(
            midnight < -(BASE_M as f32),
            "every zone is above the column's floor"
        );
        assert_eq!(zone(k, 10.0), Zone::Sunlit);
        assert_eq!(zone(k, 60.0), Zone::Twilight);
        assert_eq!(zone(k, 120.0), Zone::Midnight);
    }

    #[test]
    fn red_goes_first() {
        let [r, g, b] = light_left(OCEAN_DOWNWELL_PER_M, 10.0);
        assert!(r < 0.02, "red at 10 m: {r}");
        assert!(g > 0.5 && b > 0.5, "green {g} and blue {b} at 10 m");
    }

    #[test]
    fn the_light_left_is_one_at_the_surface_and_never_grows() {
        let k = OCEAN_DOWNWELL_PER_M;
        assert_eq!(light_left(k, 0.0), [1.0; 3]);
        assert_eq!(
            light_left(k, -3.0),
            [1.0; 3],
            "above the water is the surface's"
        );
        assert_eq!(light_left(k, f32::NAN), [1.0; 3]);
        let mut last = [1.0; 3];
        for d in 1..=145 {
            let now = light_left(k, d as f32);
            for c in 0..3 {
                assert!(now[c] <= last[c]);
            }
            last = now;
        }
    }

    #[test]
    fn turbidity_eases_from_the_coast_to_the_open_ocean_and_a_river_is_its_own() {
        let t = Turbidity::default();
        let limits = (6.0, 40.0);
        assert_eq!(t.at(Some(WaterClass::Shallows), 3.0, limits), t.coast);
        assert_eq!(t.at(Some(WaterClass::Deep), 90.0, limits), t.ocean);
        assert_eq!(t.at(Some(WaterClass::River), 2.0, limits), t.river);
        let mid = t.at(Some(WaterClass::Shelf), 23.0, limits);
        assert!(mid < t.coast && mid > t.ocean);
        // No step anywhere along a swim out.
        let mut last = t.at(Some(WaterClass::Shallows), 0.0, limits);
        for d in 1..400 {
            let now = t.at(Some(WaterClass::Shelf), d as f32 * 0.25, limits);
            assert!(
                (now - last).abs() < 0.002,
                "a step at {} m",
                d as f32 * 0.25
            );
            last = now;
        }
        // The ease never overshoots and gets there.
        let mut v = t.coast;
        for _ in 0..600 {
            v = t.ease(v, t.ocean, 1.0 / 60.0);
            assert!(v >= t.ocean - 1e-6 && v <= t.coast + 1e-6);
        }
        assert!((v - t.ocean).abs() < 0.01 * (t.coast - t.ocean));
        assert!(t.validate().is_ok());
    }
}
