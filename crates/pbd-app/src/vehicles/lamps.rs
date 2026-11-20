//! The raft's own lantern as a light (`river-craft` design 9.8; the owner,
//! 2026-11-19: "Rafts should also have a light that comes with it at
//! night").
//!
//! A raft moves every frame, so its lamp cannot be baked into the cell
//! light field the town lamps are. It joins the short list of moving lights
//! the field-lit shaders already add per pixel - the terrain, the water and
//! every `LitByField` mesh - the list a carried torch is in
//! (`carried_light`, `glowing-mushrooms` part B), after the player's own.
//! Its light is the placed post lantern's, from the one table
//! (`pbd_core::light::carried_light`): the same level, tint and falloff,
//! lit from dusk to dawn on the town lamps' switch. Like a carried light it
//! casts no shadow. Nothing is saved but the lantern put out
//! (`Craft::lantern_out`, the boat's menu).

use super::Vehicle;
use bevy::prelude::*;
use pbd_core::inventory::Item;
use pbd_core::light::{Carried, carried_light};
use pbd_core::terrain::Material;
use pbd_core::vehicle::Craft;

/// The craft lanterns burning this frame, body-local, nearest first is not
/// promised: `carried_light::gather` takes the nearest to the eye.
#[derive(Resource, Clone, Default, Debug, PartialEq)]
pub struct CraftLamps(pub Vec<Carried>);

/// The lantern's light: the placed post lantern's.
fn lantern() -> pbd_core::light::CarriedLight {
    carried_light(Item::Block(Material::LanternPost)).expect("a post lantern gives light")
}

/// A craft's lantern as a light, if it has one, it is lit (`dusk`, the
/// town lamps' switch) and it has not been put out.
pub fn lamp_of(craft: &Craft, dusk: bool) -> Option<Carried> {
    let spec = craft.loon_spec().lamp?;
    let light = lantern();
    (light.lit(dusk) && !craft.lantern_out).then(|| Carried {
        at: craft.reference_point(spec.at).as_vec3(),
        colour: light.colour(),
    })
}

/// Every lit craft lantern, for `carried_light::gather`.
pub fn gather(
    vehicles: Query<&Vehicle>,
    dusk: Option<Res<crate::planet::lod::DuskLamps>>,
    mut lamps: ResMut<CraftLamps>,
) {
    let lit = dusk.is_some_and(|d| d.lit);
    let next: Vec<Carried> = vehicles
        .iter()
        .filter_map(|v| lamp_of(&v.craft, lit))
        .collect();
    if lamps.0 != next {
        lamps.0 = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::vehicle::{Hulls, Kind, spec::VehicleSpecs};
    use std::sync::Arc;

    fn craft(kind: Kind) -> Craft {
        let specs = Arc::new(VehicleSpecs::default());
        Craft::new(
            kind,
            1,
            specs.clone(),
            Hulls::new(&specs),
            pbd_core::DVec3::Y * 4800.0,
            pbd_core::DQuat::IDENTITY,
        )
    }

    /// `river-craft` design 9.9: the raft's lantern is a light only while
    /// the dusk lamps burn and only while it has not been put out; the Loon
    /// and the tube add none.
    #[test]
    fn the_rafts_lantern_lights_at_night_only_and_no_other_kind_has_one() {
        let mut raft = craft(Kind::Raft);
        assert_eq!(lamp_of(&raft, false), None, "out by day");
        let lamp = lamp_of(&raft, true).expect("lit at night");
        let at = raft.reference_point(raft.loon_spec().lamp.unwrap().at);
        assert!(lamp.at.distance(at.as_vec3()) < 1e-3);
        assert_eq!(lamp.colour, lantern().colour());
        raft.lantern_out = true;
        assert_eq!(lamp_of(&raft, true), None, "put out");
        assert_eq!(lamp_of(&craft(Kind::Loon), true), None);
        assert_eq!(lamp_of(&craft(Kind::Tube), true), None);
    }
}
