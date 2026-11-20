//! The light a player carries (`glowing-mushrooms` part B): a torch, a
//! lantern or a glowing mushroom in the selected hotbar slot, and the body
//! glow a glowing mushroom gives when eaten, as one light per player at the
//! chest or the hand.
//!
//! What gives light is `pbd_core::light::carried_light`, one table derived
//! from the placed things' own levels. Each frame the player's sources are
//! merged (`pbd_core::light::merge`) and handed to the terrain, the water and
//! the field-lit shaders as a short uniform list, which they evaluate per
//! pixel with the falloff every baked lamp uses and no shadow (design 7, 8).
//! Nothing is baked, walked or read back for it, and nothing is saved: it is
//! worked out from the slot and the `meal` record every frame.
//!
//! `PBD_CARRIED=0` turns it off, for the owner's A/B (design 13).

use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use pbd_core::cooking::Effect;
use pbd_core::goods::Good;
use pbd_core::inventory::Item;
use pbd_core::light::{Carried, carried_light, merge};

/// The most carried lights drawn (`carried_max_lights`): the player's own,
/// then the nearest others. The shaders' arrays hold two vec4s a light, and
/// a test holds the three shaders' `CARRIED_LIGHTS` to this.
pub const MAX_CARRIED_LIGHTS: usize = 4;

/// The chest, under the eye, where the body glow is when nothing is held,
/// m (design 11).
pub const CHEST_BELOW_EYE_M: f32 = 0.55;

/// Where a held light burns in eye space when the held view has not said
/// (no hand drawn: a brazier, a candle): ahead and to the right, chest-high.
pub const HELD_LIGHT_EYE: Vec3 = Vec3::new(0.28, -0.05, -0.62);

/// The lights carried this frame, in the planet's frame (body-local), the
/// player's own first.
#[derive(Resource, Clone, Default, Debug, PartialEq, ExtractResource)]
pub struct CarriedLights(pub Vec<Carried>);

impl CarriedLights {
    /// As the shaders read them: two vec4s a light, where (w its reach, zero
    /// ending the list) and its colour times strength.
    pub fn packed(&self) -> [Vec4; 2 * MAX_CARRIED_LIGHTS] {
        let mut out = [Vec4::ZERO; 2 * MAX_CARRIED_LIGHTS];
        for (i, light) in self.0.iter().take(MAX_CARRIED_LIGHTS).enumerate() {
            out[2 * i] = light.at.extend(pbd_core::lamp_bake::REACH_M);
            out[2 * i + 1] = Vec3::from(light.colour).extend(0.0);
        }
        out
    }

    /// The player's own light, if they carry one.
    pub fn own(&self) -> Option<&Carried> {
        self.0.first()
    }
}

/// What the selected hotbar slot holds, written by the hotbar each frame.
#[derive(Resource, Clone, Copy, Default, Debug, PartialEq)]
pub struct HeldItem(pub Option<Item>);

/// Where the held light burns, in the eye's frame, and its flicker now (1
/// steady): written by the held view each frame, so the light on the ground
/// moves and flickers with the flame drawn (design 14).
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct HeldLightAt {
    pub eye: Vec3,
    pub flicker: f32,
}

impl Default for HeldLightAt {
    fn default() -> Self {
        Self {
            eye: HELD_LIGHT_EYE,
            flicker: 1.0,
        }
    }
}

/// Whether `PBD_CARRIED=0` turned the carried light off.
fn switched_off() -> bool {
    static OFF: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *OFF.get_or_init(|| std::env::var("PBD_CARRIED").is_ok_and(|v| v.trim() == "0"))
}

/// The body glow's colour times strength at `now_s`, from the `meal` record:
/// risen over `rise_s` after eating, faded over the last `fade_s`.
pub fn glow_colour(
    meals: &pbd_core::cooking::Meals,
    kitchen: &pbd_core::cooking::Kitchen,
    now_s: f64,
) -> Option<[f32; 3]> {
    let left = meals.left_s(Effect::Glow, now_s)?;
    let lasts = kitchen
        .dish(Good::GLOWING_MUSHROOM)
        .map_or(180.0, pbd_core::cooking::Dish::secs);
    let glow = &kitchen.glow;
    let k = glow.level_strength() * glow.strength((lasts - left).max(0.0), left);
    (k > 0.0).then(|| glow.tint.map(|c| c * k))
}

/// Gather this frame's carried light from the slot and the body glow.
#[allow(clippy::too_many_arguments)]
pub fn gather(
    held: Res<HeldItem>,
    at: Res<HeldLightAt>,
    save: Option<Res<crate::saves::WorldSave>>,
    kitchen: Option<Res<crate::config::KitchenConfig>>,
    sun: Option<Res<crate::sky::Sun>>,
    dusk: Option<Res<crate::planet::lod::DuskLamps>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    crafts: Option<Res<crate::vehicles::lamps::CraftLamps>>,
    mut lights: ResMut<CarriedLights>,
) {
    let mut next = Vec::new();
    if !switched_off()
        && let Some((eye, _)) = cameras.iter().find(|(_, c)| c.is_active)
    {
        let rotation = eye.compute_transform().rotation;
        let centre = frame.as_deref().map(|f| f.center);
        // The eye in the planet's frame, subtracted in f64 before the cast.
        let eye_at = crate::render_origin::local_of(frame.as_deref(), eye.translation());
        let up = centre.map_or(Vec3::Y, |c| {
            (eye.translation().as_dvec3() - c)
                .try_normalize()
                .unwrap_or(bevy::math::DVec3::Y)
                .as_vec3()
        });
        let mut sources = Vec::new();
        let lit = dusk.is_some_and(|d| d.lit);
        if let Some(light) = held.0.and_then(carried_light).filter(|l| l.lit(lit)) {
            let colour = light.colour().map(|c| c * at.flicker.max(0.0));
            sources.push((eye_at + rotation * at.eye, colour));
        }
        if let (Some(save), Some(kitchen), Some(sun)) = (&save, &kitchen, &sun)
            && let Some(colour) =
                glow_colour(&crate::kitchen::meals(save), &kitchen.0, sun.clock.seconds)
        {
            sources.push((eye_at - up * CHEST_BELOW_EYE_M, colour));
        }
        next.extend(merge(&sources));
        // Then the craft lanterns burning, nearest the eye first
        // (`river-craft` design 9.8), as many as the list holds.
        if let Some(crafts) = crafts.as_deref() {
            let mut near: Vec<Carried> = crafts.0.clone();
            near.sort_by(|a, b| {
                a.at.distance_squared(eye_at)
                    .total_cmp(&b.at.distance_squared(eye_at))
            });
            next.extend(near);
            next.truncate(MAX_CARRIED_LIGHTS);
        }
    }
    if *lights.0 != next {
        lights.0 = next;
    }
}

pub struct CarriedLightPlugin;

impl Plugin for CarriedLightPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CarriedLights>()
            .init_resource::<HeldItem>()
            .init_resource::<HeldLightAt>()
            .add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
                CarriedLights,
            >::default())
            // From the eye where it is drawn this frame: after the
            // transforms are propagated, before the frame is extracted.
            .add_systems(
                PostUpdate,
                gather.after(bevy::transform::TransformSystems::Propagate),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The list packs two vec4s a light, the reach in w, the rest zero; a
    /// list longer than the shaders hold is cut.
    #[test]
    fn the_list_packs_two_vec4s_a_light() {
        let one = Carried {
            at: Vec3::new(1.0, 2.0, 3.0),
            colour: [0.5, 0.25, 0.125],
        };
        let packed = CarriedLights(vec![one; 6]).packed();
        assert_eq!(packed[0], Vec4::new(1.0, 2.0, 3.0, 16.0));
        assert_eq!(packed[1], Vec4::new(0.5, 0.25, 0.125, 0.0));
        assert_eq!(packed.len(), 2 * MAX_CARRIED_LIGHTS);
        let none = CarriedLights::default().packed();
        assert!(none.iter().all(|v| *v == Vec4::ZERO));
    }

    /// `glowing-mushrooms` 1.2-1.4: one carried light, three shaders. The
    /// terrain, the field-lit things and the water carry the same
    /// `carried_light`, its list the length the Rust packs, its gain the
    /// bake's times a torch's and its falloff the bake's (`lamp_cell`'s
    /// line, which `lamp_bake::falloff` is held to), with no shadow walk;
    /// and each adds it where a lamp would show.
    #[test]
    fn three_shaders_take_one_carried_light() {
        let read = |name: &str| {
            std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../assets/shaders")
                    .join(name),
            )
            .unwrap()
            .replace("\r\n", "\n")
        };
        let body = |s: &str| {
            let i = s
                .find("// `glowing-mushrooms` design 8: the lights players carry")
                .expect("the carried light's block");
            let j = s[i..].find("\n}\n").expect("its end");
            s[i..i + j].to_string()
        };
        let terrain = read("planet_surface.wgsl");
        let field = read("field_lit.wgsl");
        let water = read("water.wgsl");
        let one = body(&terrain);
        assert_eq!(one, body(&field), "field_lit.wgsl's carried light");
        assert_eq!(one, body(&water), "water.wgsl's carried light");
        for line in [
            format!("const CARRIED_LIGHTS: u32 = {MAX_CARRIED_LIGHTS}u;"),
            format!(
                "const CARRIED_GAIN: f32 = {:?};",
                pbd_core::lamp_bake::GAIN * pbd_core::light::TORCH_GAIN
            ),
            format!(
                "fn carried_light(lights: array<vec4<f32>, {}>, p: vec3<f32>, n: vec3<f32>) -> vec3<f32> {{",
                2 * MAX_CARRIED_LIGHTS
            ),
            "nl*w*w/(1.0 + (d/2.2)*(d/2.2))".to_string(),
        ] {
            assert!(one.contains(&line), "the carried light should say `{line}`");
        }
        assert!(terrain.contains("direct += nl*w*w/(1.0 + (d/2.2)*(d/2.2))*LAMP_BAKE_GAIN"));
        assert!(
            !one.contains("solid_at") && !one.contains("columns["),
            "no shadow walk"
        );
        for (shader, line) in [
            (&terrain, "carried: array<vec4<f32>,8>,"),
            (
                &terrain,
                "color += albedo*carried_light(params.carried, input.position, n)*share;",
            ),
            (&field, "carried: array<vec4<f32>, 8>,"),
            (
                &field,
                "lamp += base * carried_light(scene.carried, body, n) * lamp_share(sky * field.look.y, daylight);",
            ),
            (&water, "carried: array<vec4<f32>, 8>,"),
            (
                &water,
                "*carried_light(view.carried, in.body_position, normal)*share;",
            ),
        ] {
            assert!(shader.contains(line), "a shader should say `{line}`");
        }
    }

    /// `player/glow`: a mushroom eaten at 0 s glows from nothing, is whole
    /// a minute in, at half with 10 s left, and out at the end, in the cap's
    /// colour at its level.
    #[test]
    fn the_body_glow_follows_the_meal() {
        let kitchen = pbd_core::cooking::Kitchen::default();
        let dish = *kitchen.dish(Good::GLOWING_MUSHROOM).unwrap();
        let mut meals = pbd_core::cooking::Meals::default();
        meals.eat(&dish, 0.0, 3);
        let at = |s| glow_colour(&meals, &kitchen, s);
        assert_eq!(at(0.0), None, "lit from nothing");
        let whole = at(60.0).unwrap();
        let k = kitchen.glow.level_strength();
        for (c, t) in whole.iter().zip(kitchen.glow.tint) {
            assert!((c - t * k).abs() < 1e-6);
        }
        let half = at(170.0).unwrap();
        assert!((half[1] / whole[1] - 0.5).abs() < 1e-4);
        assert_eq!(at(180.0), None, "out at the end");
    }
}
