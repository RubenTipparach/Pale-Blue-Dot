//! The night side's far lights in the game (`night-lights-from-space`).
//!
//! The rules are `pbd_core::night_lights`, read from
//! `assets/config/night_lights.ron`. The list is the world's saved sites
//! (`WorldSites`) and the groups of the player's lights in the save's edits,
//! rebuilt when either changes, and never saved. The render world takes it
//! each frame and the terrain pass draws it (`planet_surface.wgsl`,
//! `far_night_lights`), each light at its strength for this frame's sun and
//! eye.

use crate::saves::WorldSave;
use crate::sites::WorldSites;
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use pbd_core::edits::Edits;
use pbd_core::night_lights::{self, FarLight, NightLightsConfig};
use pbd_core::records::YieldSet;
use std::path::PathBuf;

/// How many far lights the terrain pass takes: two `vec4` each in
/// `PlanetParams::night_lights`. About 55 sites, and room for the player's.
pub const MAX_FAR_LIGHTS: usize = 64;

/// `assets/config/night_lights.ron`.
pub fn rules_path() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/config/night_lights.ron"
    ))
}

/// The rules, read and checked: a file that does not parse or validate is a
/// startup error naming the file.
pub fn load_rules() -> NightLightsConfig {
    let path = rules_path();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let rules: NightLightsConfig =
        ron::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    rules
        .validate()
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    rules
}

/// Every far light of this world, and the rules that say how bright each is
/// now. Derived, never saved.
#[derive(Resource, Clone, ExtractResource)]
pub struct FarLights {
    pub rules: NightLightsConfig,
    /// The settlements', in site order, then the player's groups.
    pub lights: Vec<FarLight>,
    /// What the list was built from: the sites (count and the sum of their
    /// ids) and the edits' revision. A change to either rebuilds it.
    built_from: Option<((usize, u64), u64)>,
    /// How many of `lights` are the player's.
    pub player: usize,
}

impl FarLights {
    pub fn new(rules: NightLightsConfig) -> Self {
        Self {
            rules,
            lights: Vec::new(),
            built_from: None,
            player: 0,
        }
    }

    /// The list for this frame, packed as the terrain pass reads it: x the
    /// number of lights; then two `vec4` each, the direction and the cosine
    /// of its reach, then its radius, its strength now, its seed and its
    /// points' spacing. Lights of no strength are left out, and the strongest
    /// [`MAX_FAR_LIGHTS`] kept, ties in list order.
    pub fn pack(
        &self,
        sun: Vec3,
        camera: Vec3,
        radius_m: f32,
    ) -> (Vec4, [Vec4; 2 * MAX_FAR_LIGHTS]) {
        let mut lit: Vec<(f32, &FarLight)> = self
            .lights
            .iter()
            .map(|l| {
                (
                    night_lights::strength(l, sun, camera, radius_m, &self.rules),
                    l,
                )
            })
            .filter(|(s, _)| *s > 1e-4)
            .collect();
        // Stable: equal strengths keep the list's order.
        lit.sort_by(|a, b| b.0.total_cmp(&a.0));
        lit.truncate(MAX_FAR_LIGHTS);
        let mut packed = [Vec4::ZERO; 2 * MAX_FAR_LIGHTS];
        for (i, (strength, light)) in lit.iter().enumerate() {
            // The reach a pixel tests: the points' radius and a blob's width
            // beyond it, as an angle on the sphere.
            let reach = (light.radius_m + 2.0 * light.spacing_m) / radius_m;
            packed[2 * i] = light
                .direction
                .extend(reach.min(std::f32::consts::PI).cos());
            packed[2 * i + 1] = Vec4::new(
                light.radius_m,
                *strength,
                (light.seed & 0xffff) as f32,
                light.spacing_m,
            );
        }
        (Vec4::new(lit.len() as f32, 0.0, 0.0, 0.0), packed)
    }
}

/// The player's light blocks: every edit that emits on a cell the player has
/// touched, at its cell's direction.
pub fn player_lamps(edits: &Edits, yields: &YieldSet) -> Vec<Vec3> {
    edits
        .all()
        .into_iter()
        .filter(|e| e.material.emission() > 0 && yields.has_cell(e.cell))
        .filter_map(|e| pbd_core::cell_key::key_direction(e.cell))
        .collect()
}

/// Rebuild the list when the sites or the edits have changed.
pub fn gather(
    sites: Option<Res<WorldSites>>,
    save: Option<Res<WorldSave>>,
    mut far: ResMut<FarLights>,
) {
    let ready = sites.as_ref().and_then(|s| s.ready()).unwrap_or(&[]);
    let site_key = (
        ready.len(),
        ready.iter().map(|s| u64::from(s.id)).sum::<u64>(),
    );
    let revision = save.as_ref().map_or(0, |s| s.edits.revision());
    // A world opened anew (the save inserted again) is rebuilt even where its
    // sites and revision happen to match the last world's.
    let reopened = save.as_ref().is_some_and(|s| s.is_added());
    if far.built_from == Some((site_key, revision)) && !reopened {
        return;
    }
    let rules = far.rules.clone();
    let mut lights: Vec<FarLight> = ready
        .iter()
        .filter_map(|site| night_lights::site_light(site, &rules))
        .collect();
    let towns = lights.len();
    let player = save
        .as_ref()
        .map(|save| {
            night_lights::player_clusters(
                &player_lamps(&save.edits, &save.yields),
                crate::planet::PLANET_RADIUS,
                &rules,
            )
        })
        .unwrap_or_default();
    // Only worth a line when the player's lights change what is drawn.
    if player.len() != far.player || far.built_from.is_none() {
        info!(
            "night lights: {towns} settlements and {} groups of the player's lights show from afar",
            player.len()
        );
    }
    far.player = player.len();
    lights.extend(player);
    far.lights = lights;
    far.built_from = Some((site_key, revision));
}

/// The far lights: the rules at startup, the list kept, and handed to the
/// render world each frame.
pub struct NightLightsPlugin;

impl Plugin for NightLightsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FarLights::new(load_rules()))
            .add_plugins(ExtractResourcePlugin::<FarLights>::default())
            .add_systems(Update, gather);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::cell_key::{Address, key};
    use pbd_core::edits::Edit;
    use pbd_core::terrain::Material;

    /// A test world's edits: `n` torches in a row of finest cells, each
    /// touched by the player, and one more lamp on a cell the player never
    /// touched (a world process's), which does not count.
    fn torches(n: u32) -> (Edits, YieldSet) {
        let mut edits = Edits::new();
        let mut yields = YieldSet::new();
        for i in 0..n {
            let cell = key(Address {
                face: 4,
                level: 11,
                i: 600 + 3 * i,
                j: 500,
            })
            .unwrap();
            edits.set(Edit {
                cell,
                layer: 90,
                material: Material::Torch,
            });
            yields.touch_cell(cell);
        }
        let stranger = key(Address {
            face: 4,
            level: 11,
            i: 600,
            j: 510,
        })
        .unwrap();
        edits.set(Edit {
            cell: stranger,
            layer: 90,
            material: Material::LanternPost,
        });
        (edits, yields)
    }

    /// Place lamps in a test world's edits one at a time: no far light until
    /// the group reaches the threshold, then one, over the lamps, which the
    /// pack draws at night and not by day (task 1.4).
    #[test]
    fn the_players_far_light_appears_past_the_threshold() {
        let rules = load_rules();
        let threshold = rules.player_threshold;
        let radius = crate::planet::PLANET_RADIUS;
        let (below, below_yields) = torches(threshold - 1);
        assert_eq!(
            player_lamps(&below, &below_yields).len(),
            (threshold - 1) as usize
        );
        assert!(
            night_lights::player_clusters(&player_lamps(&below, &below_yields), radius, &rules)
                .is_empty()
        );
        let (at, yields) = torches(threshold);
        let lamps = player_lamps(&at, &yields);
        let clusters = night_lights::player_clusters(&lamps, radius, &rules);
        assert_eq!(clusters.len(), 1);
        let centre = clusters[0].direction;
        assert!(
            lamps
                .iter()
                .all(|l| l.distance(centre) * radius < clusters[0].radius_m + 1.0)
        );

        let mut far = FarLights::new(rules);
        far.lights = clusters;
        let eye = centre * (radius + 9000.0);
        let (count, packed) = far.pack(-centre, eye, radius);
        assert_eq!(count.x, 1.0, "lit at night from 9 km");
        assert!(packed[0].truncate().distance(centre) < 1e-6);
        assert!(packed[1].y > 0.0);
        let (count, _) = far.pack(centre, eye, radius);
        assert_eq!(count.x, 0.0, "dark by day");
    }

    /// The shipped file parses and validates.
    #[test]
    fn the_rules_load() {
        let rules = load_rules();
        assert!(rules.player_threshold >= 1);
    }
}
