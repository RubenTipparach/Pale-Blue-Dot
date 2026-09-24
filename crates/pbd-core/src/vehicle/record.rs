//! What is saved of a craft: enough that it is in the world after a reload
//! exactly where it was left, with the same ID, still boardable.
//!
//! Poses are body-local (the planet's centre is the origin) in double
//! precision, and the frame they are in is named, because a craft's pose is
//! meaningless without it.

use super::{Craft, CraftState, Hulls, Kind, Mooring};
use crate::vehicle::spec::VehicleSpecs;
use glam::{DQuat, DVec3};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// The layout's version; a reader refuses one it does not know.
pub const RECORD_VERSION: u32 = 1;

/// The frame every craft's pose is in today: the planet's own.
pub const PLANET_FRAME: &str = "planet:0";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VehicleRecord {
    pub id: u64,
    pub kind: String,
    pub frame: String,
    /// Where the craft's reference point is, and how it is turned.
    pub position: [f64; 3],
    pub orientation: [f64; 4],
    pub velocity: [f64; 3],
    pub angular_velocity: [f64; 3],
    /// Water aboard, kg.
    pub bilge_kg: f64,
    /// Made fast: where (body-local), the line's length, and whether it is an
    /// anchor.
    pub mooring: Option<([f64; 3], f64, bool)>,
    /// Who it belongs to; nobody yet, in a world with one player.
    pub owner: Option<String>,
    /// The Kestrel's nacelles and brake; the Tern's sheet.
    pub nacelle: Option<f64>,
    pub brake: Option<bool>,
    pub sheet: Option<f64>,
}

/// Every craft in a world, as the save file holds them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VehicleFile {
    pub version: u32,
    pub next_id: u64,
    pub vehicles: Vec<VehicleRecord>,
}

impl Craft {
    /// This craft as the save holds it. Occupancy is not saved: a player who
    /// quits aboard comes back on foot beside the craft, which is where the
    /// load puts them, and the craft is unattended until they board again.
    pub fn record(&self) -> VehicleRecord {
        let (nacelle, brake, sheet) = match &self.state {
            CraftState::Kestrel(s) => (Some(s.nacelle), Some(s.brake), None),
            CraftState::Tern(s) => (None, None, Some(s.sheet)),
            CraftState::Loon(_) => (None, None, None),
        };
        VehicleRecord {
            id: self.id,
            kind: self.kind.key().into(),
            frame: PLANET_FRAME.into(),
            position: self.reference_position().to_array(),
            orientation: self.body.orientation.to_array(),
            velocity: self.body.velocity.to_array(),
            angular_velocity: self.body.angular_velocity.to_array(),
            bilge_kg: self.bilge_kg,
            mooring: self
                .mooring
                .map(|m| (m.at.to_array(), m.length, m.anchored)),
            owner: None,
            nacelle,
            brake,
            sheet,
        }
    }

    /// A craft back from its record, or why it cannot be.
    pub fn from_record(
        record: &VehicleRecord,
        specs: Arc<VehicleSpecs>,
        hulls: Hulls,
    ) -> Result<Craft, String> {
        let kind = Kind::from_key(&record.kind)
            .ok_or_else(|| format!("unknown craft kind '{}'", record.kind))?;
        if record.frame != PLANET_FRAME {
            return Err(format!(
                "craft {} is in unknown frame '{}'",
                record.id, record.frame
            ));
        }
        let finite = record
            .position
            .iter()
            .chain(&record.orientation)
            .chain(&record.velocity)
            .chain(&record.angular_velocity)
            .all(|v| v.is_finite())
            && record.bilge_kg.is_finite();
        if !finite {
            return Err(format!("craft {} holds a non-finite value", record.id));
        }
        let orientation = DQuat::from_array(record.orientation).normalize();
        let mut craft = Craft::new(
            kind,
            record.id,
            specs,
            hulls,
            DVec3::from(record.position),
            orientation,
        );
        craft.bilge_kg = record.bilge_kg.max(0.0);
        craft.weigh();
        craft.set_reference_pose(DVec3::from(record.position), orientation);
        craft.body.velocity = DVec3::from(record.velocity);
        craft.body.angular_velocity = DVec3::from(record.angular_velocity);
        craft.mooring = record.mooring.map(|(at, length, anchored)| Mooring {
            at: DVec3::from(at),
            length,
            anchored,
        });
        match &mut craft.state {
            CraftState::Kestrel(s) => {
                if let Some(n) = record.nacelle.filter(|n| n.is_finite()) {
                    s.nacelle = n.clamp(0.0, std::f64::consts::FRAC_PI_2);
                }
                if let Some(b) = record.brake {
                    s.brake = b;
                }
            }
            CraftState::Tern(s) => {
                if let Some(sheet) = record.sheet.filter(|s| s.is_finite()) {
                    s.sheet = sheet.clamp(0.0, 1.0);
                }
            }
            CraftState::Loon(_) => {}
        }
        Ok(craft)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_craft_comes_back_from_its_record_where_it_was() {
        let specs = Arc::new(VehicleSpecs::default());
        let hulls = Hulls::new(&specs);
        let q = DQuat::from_rotation_y(0.7);
        let mut craft = Craft::new(
            Kind::Tern,
            42,
            specs.clone(),
            hulls.clone(),
            DVec3::new(10.0, 4799.3, -3.0),
            q,
        );
        craft.body.velocity = DVec3::new(0.3, 0.0, -1.2);
        craft.bilge_kg = 12.5;
        craft.mooring = Some(Mooring {
            at: DVec3::new(1.0, 4800.0, 2.0),
            length: 6.0,
            anchored: false,
        });
        let record = craft.record();
        let text = ron::to_string(&VehicleFile {
            version: RECORD_VERSION,
            next_id: 43,
            vehicles: vec![record.clone()],
        })
        .unwrap();
        let file: VehicleFile = ron::from_str(&text).unwrap();
        let back = Craft::from_record(&file.vehicles[0], specs, hulls).unwrap();
        assert_eq!(back.id, 42);
        assert_eq!(back.kind, Kind::Tern);
        assert!((back.reference_position() - craft.reference_position()).length() < 1e-9);
        assert!(back.body.orientation.angle_between(q) < 1e-9);
        assert_eq!(back.bilge_kg, 12.5);
        assert_eq!(back.mooring, craft.mooring);
        assert!(!back.occupied);
    }

    #[test]
    fn a_record_that_cannot_be_is_refused() {
        let specs = Arc::new(VehicleSpecs::default());
        let hulls = Hulls::new(&specs);
        let craft = Craft::new(
            Kind::Loon,
            1,
            specs.clone(),
            hulls.clone(),
            DVec3::Y,
            DQuat::IDENTITY,
        );
        let mut bad = craft.record();
        bad.kind = "submarine".into();
        assert!(Craft::from_record(&bad, specs.clone(), hulls.clone()).is_err());
        let mut bad = craft.record();
        bad.position[0] = f64::NAN;
        assert!(Craft::from_record(&bad, specs, hulls).is_err());
    }
}
