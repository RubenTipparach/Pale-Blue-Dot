//! Two gravity fields with one human-scale surface constant. Gameplay actors
//! use the bounded anchor field; the inverse-square reference is not a ship solver.

use glam::DVec3;

/// Tenebris's 1 g: preserve a 12 m/s jump's 2.88 m apex on every body size.
pub const SURFACE_GRAVITY_MPS2_PER_G: f64 = 25.0;
pub const GRAVITY_FULL_RADIUS_MULTIPLIER: f64 = 1.4;
pub const GRAVITY_OUTER_RADIUS_MULTIPLIER: f64 = 1.8;

#[derive(Clone, Copy, Debug, Default)]
pub struct GravityBandOverrides {
    /// Full-pull radius / body radius; Some(0) starts tapering at the centre.
    pub full: Option<f64>,
    /// Zero-pull radius / body radius; must exceed the resolved full radius.
    pub outer: Option<f64>,
}

impl GravityBandOverrides {
    pub fn resolve(self) -> Result<(f64, f64), &'static str> {
        let full = self.full.unwrap_or(GRAVITY_FULL_RADIUS_MULTIPLIER);
        let outer = self.outer.unwrap_or(GRAVITY_OUTER_RADIUS_MULTIPLIER);
        if !full.is_finite() || !outer.is_finite() || full < 0.0 || outer <= full {
            return Err("gravity bands must be finite with 0 <= full < outer");
        }
        Ok((full, outer))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GravityWell {
    /// Stable body identity, also used to resolve equal-multiplier overlaps.
    pub body_id: usize,
    pub center: DVec3,
    /// Sea-level body radius in metres.
    pub radius: f64,
    /// Multiplier of the shared surface constant; zero is valid.
    pub gravity_g: f64,
    pub bands: GravityBandOverrides,
}

impl GravityWell {
    pub fn new(body_id: usize, center: DVec3, radius: f64, gravity_g: f64) -> Self {
        let well = Self {
            body_id,
            center,
            radius,
            gravity_g,
            bands: GravityBandOverrides::default(),
        };
        assert!(well.validate().is_ok());
        well
    }

    pub fn validate(self) -> Result<(), &'static str> {
        if !self.center.is_finite()
            || !self.radius.is_finite()
            || self.radius <= 0.0
            || !self.gravity_g.is_finite()
            || self.gravity_g < 0.0
            || !(self.gravity_g * SURFACE_GRAVITY_MPS2_PER_G).is_finite()
        {
            return Err("gravity well requires a finite centre, positive radius and nonnegative g");
        }
        self.bands.resolve()?;
        Ok(())
    }

    pub fn surface_acceleration(self) -> f64 {
        self.gravity_g * SURFACE_GRAVITY_MPS2_PER_G
    }

    fn anchor_at(self, position: DVec3) -> Option<GravityAnchor> {
        assert!(self.validate().is_ok() && position.is_finite());
        let (full, outer) = self.bands.resolve().unwrap();
        let offset = self.center - position;
        let distance = offset.length();
        let relative_distance = distance / self.radius;
        if relative_distance >= outer {
            return None;
        }
        let multiplier = if relative_distance <= full {
            1.0
        } else {
            (outer - relative_distance) / (outer - full)
        };
        Some(GravityAnchor {
            body_id: self.body_id,
            multiplier,
            altitude: distance - self.radius,
            acceleration: offset.normalize_or_zero() * self.surface_acceleration() * multiplier,
        })
    }

    /// Inverse-square reference, with a continuous uniform-density interior.
    /// Gameplay walkers and assisted ships use `anchor_gravity_at` instead.
    pub fn orbital_acceleration_at(self, position: DVec3) -> DVec3 {
        assert!(self.validate().is_ok() && position.is_finite());
        let offset = self.center - position;
        let distance = offset.length();
        let falloff = if distance >= self.radius {
            (self.radius / distance).powi(2)
        } else {
            distance / self.radius
        };
        offset.normalize_or_zero() * self.surface_acceleration() * falloff
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GravityAnchor {
    pub body_id: usize,
    pub multiplier: f64,
    /// Metres above the body's sea-level radius, negative inside the body.
    pub altitude: f64,
    pub acceleration: DVec3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GravityQuery {
    pub anchor: Option<GravityAnchor>,
}

impl GravityQuery {
    pub fn is_in_space(self) -> bool {
        self.anchor.is_none()
    }

    pub fn acceleration(self) -> DVec3 {
        self.anchor
            .map_or(DVec3::ZERO, |anchor| anchor.acceleration)
    }
}

/// Select one bounded anchor by multiplier, breaking ties by stable body ID.
/// Input order cannot change the outcome; fields are never summed for actors.
/// Each authored body must have a unique ID.
pub fn anchor_gravity_at(
    wells: impl IntoIterator<Item = GravityWell>,
    position: DVec3,
) -> GravityQuery {
    assert!(position.is_finite());
    let anchor = wells
        .into_iter()
        .filter_map(|well| well.anchor_at(position))
        .max_by(|a, b| {
            a.multiplier
                .total_cmp(&b.multiplier)
                .then_with(|| b.body_id.cmp(&a.body_id))
        });
    GravityQuery { anchor }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_bands_have_exact_edges_and_space_comes_from_absence() {
        let well = GravityWell::new(3, DVec3::ZERO, 100.0, 1.0);
        for (radius, multiplier) in [(100.0, 1.0), (140.0, 1.0), (160.0, 0.5)] {
            let query = anchor_gravity_at([well], DVec3::Y * radius);
            let anchor = query.anchor.unwrap();
            assert!(!query.is_in_space());
            assert_eq!(anchor.body_id, 3);
            assert!((anchor.multiplier - multiplier).abs() < 1e-12);
            assert_eq!(anchor.altitude, radius - 100.0);
            assert!((anchor.acceleration - DVec3::NEG_Y * 25.0 * multiplier).length() < 1e-12);
        }
        for radius in [180.0, 200.0, 1e9] {
            let query = anchor_gravity_at([well], DVec3::Y * radius);
            assert!(query.is_in_space());
            assert_eq!(query.acceleration(), DVec3::ZERO);
        }
        assert!(anchor_gravity_at([], DVec3::ZERO).is_in_space());
    }

    #[test]
    fn strongest_multiplier_beats_nearest_centre_and_surface_acceleration() {
        let near = GravityWell::new(0, DVec3::X * 160.0, 100.0, 10.0);
        let far = GravityWell::new(1, DVec3::Y * 260.0, 200.0, 0.1);
        for wells in [[near, far], [far, near]] {
            let anchor = anchor_gravity_at(wells, DVec3::ZERO).anchor.unwrap();
            assert_eq!(anchor.body_id, 1);
            assert_eq!(anchor.acceleration, DVec3::Y * 2.5);
        }
    }

    #[test]
    fn equal_multiplier_ties_use_stable_id_independent_of_order() {
        let a = GravityWell::new(9, DVec3::X * 100.0, 100.0, 1.0);
        let b = GravityWell::new(2, DVec3::Y * 100.0, 100.0, 1.0);
        for wells in [[a, b], [b, a]] {
            assert_eq!(
                anchor_gravity_at(wells, DVec3::ZERO)
                    .anchor
                    .unwrap()
                    .body_id,
                2
            );
        }
    }

    #[test]
    fn explicit_zero_overrides_and_zero_gravity_do_not_inherit_defaults() {
        let mut well = GravityWell::new(0, DVec3::ZERO, 100.0, 1.0);
        well.bands.full = Some(0.0);
        well.bands.outer = Some(2.0);
        assert_eq!(
            anchor_gravity_at([well], DVec3::Y * 100.0).acceleration(),
            DVec3::NEG_Y * 12.5
        );
        well.gravity_g = 0.0;
        let query = anchor_gravity_at([well], DVec3::Y * 100.0);
        assert!(!query.is_in_space());
        assert_eq!(query.acceleration(), DVec3::ZERO);
        well.bands.outer = Some(0.0);
        assert!(well.validate().is_err());
    }

    #[test]
    fn both_fields_share_surface_constant_and_have_finite_centres() {
        let center = DVec3::new(1e9, -2e9, 3e9);
        for radius in [300.0, 4_800.0] {
            let well = GravityWell::new(0, center, radius, 0.7);
            let surface = center + DVec3::Y * radius;
            assert_eq!(
                anchor_gravity_at([well], surface).acceleration(),
                well.orbital_acceleration_at(surface)
            );
            assert_eq!(well.orbital_acceleration_at(surface), DVec3::NEG_Y * 17.5);
            assert_eq!(
                well.orbital_acceleration_at(center + DVec3::Y * radius * 0.5),
                DVec3::NEG_Y * 8.75
            );
            for scale in [1.0 - 1e-6, 1.0 + 1e-6] {
                let acceleration = well.orbital_acceleration_at(center + DVec3::Y * radius * scale);
                assert!((acceleration.length() - 17.5).abs() < 0.0001);
            }
            assert_eq!(
                well.orbital_acceleration_at(center + DVec3::Y * radius * 2.0),
                DVec3::NEG_Y * 4.375
            );
            assert_eq!(well.orbital_acceleration_at(center), DVec3::ZERO);
            assert_eq!(
                anchor_gravity_at([well], center).acceleration(),
                DVec3::ZERO
            );
        }
    }

    #[test]
    fn invalid_gravity_data_is_rejected() {
        let well = GravityWell::new(0, DVec3::ZERO, 100.0, 1.0);
        for invalid in [
            GravityWell {
                radius: 0.0,
                ..well
            },
            GravityWell {
                radius: f64::INFINITY,
                ..well
            },
            GravityWell {
                gravity_g: -1.0,
                ..well
            },
            GravityWell {
                gravity_g: f64::NAN,
                ..well
            },
            GravityWell {
                center: DVec3::splat(f64::NAN),
                ..well
            },
            GravityWell {
                bands: GravityBandOverrides {
                    full: Some(2.0),
                    outer: None,
                },
                ..well
            },
        ] {
            assert!(invalid.validate().is_err());
        }
    }
}
