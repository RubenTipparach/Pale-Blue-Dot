//! What the game does, turned into asks for sound (`audio-and-ambience`
//! sections 4 to 6 and build decision 5). Every system here reads game state
//! or the messages the game already raises and writes only [`Sfx`] and
//! [`Environment`]: sound reads the world and writes nothing to it.

use super::{AudioData, Environment, Listener, Sfx, Where};
use crate::planet::{EyeWater, EyeWaterState, PLANET_RADIUS, PlanetContact, PlanetFine};
use crate::walking::{GroundState, HALF_HEIGHT, Walker, WalkingConfig, WalkingState};
use avian3d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;
use pbd_core::audio::{
    BIOMES, Bed, FallBand, FallHeard, FallReach, FallVoices, Family, Stride, Surroundings,
    air_density_scale, bed_weights, loudest_falls, stride_m, surface_family, thunder,
};
use pbd_core::fishing::Event;
use pbd_core::planet_gen::Biome;
use pbd_core::terrain::Material;

/// Every cue name the code below (and `desktop/sounds.rs`) can ask for, so a
/// test can hold them to `cues.ron`. The family-made names are listed by
/// [`family_cues`].
pub const NAMED: &[&str] = &[
    "step_wet",
    "water_enter_small",
    "water_enter_big",
    "water_exit",
    "swim_stroke",
    "swim_stroke_under",
    "bubbles",
    "breath",
    "eat_crisp",
    "eat_soft",
    "pickup",
    "place_dirt",
    "dig_dirt",
    "place_wood",
    "place_lamp",
    "ui_toggle",
    "ui_click",
    "ui_hover",
    "ui_back",
    "ui_slot",
    "cast_whoosh",
    "float_plop",
    "float_dry",
    "nibble_tick",
    "bite_splash",
    "reel_loop",
    "line_snap",
    // A worn tool breaking (`survival-mode` design 11), played by the
    // desktop's wear.
    "tool_break",
    // A predator's warning (`survival-mode` design 6), played by the combat.
    "predator_growl",
    "catch_splash",
    "catch_chime",
    "mission_done",
    "thunder_crack",
    "thunder_close",
    "thunder_roll",
    "fire_crackle",
    "fire_ignite_small",
    "fall_light",
    "fall_medium",
    "fall_roar",
    "gull",
    "owl",
    "dog_far",
];

/// The cue names made from a family: steps, jumps, landings, digs, breaks
/// and places.
pub fn family_cues() -> Vec<String> {
    let mut out = Vec::new();
    for family in Family::ALL {
        for verb in ["step", "jump"] {
            out.push(format!("{verb}_{}", family.name()));
        }
        if family.diggable() {
            for verb in ["dig", "break", "place"] {
                out.push(format!("{verb}_{}", family.name()));
            }
        }
    }
    out
}

/// A cue name made at run time, interned: `Sfx` takes `&'static str`, and
/// there are a fixed few dozen of these.
pub fn named(verb: &str, family: Family) -> &'static str {
    use std::sync::{Mutex, OnceLock};
    static NAMES: OnceLock<Mutex<std::collections::HashMap<String, &'static str>>> =
        OnceLock::new();
    let key = format!("{verb}_{}", family.name());
    let mut names = NAMES
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(name) = names.get(&key) {
        return name;
    }
    let leaked: &'static str = Box::leak(key.clone().into_boxed_str());
    names.insert(key, leaked);
    leaked
}

/// A fishing event, where it happened (raised by `fish.rs`).
#[derive(Message, Clone, Copy, Debug)]
pub struct FishHeard {
    pub event: Event,
    /// The float, body-local.
    pub at: Vec3,
}

pub(super) fn register(app: &mut App) {
    app.add_message::<FishHeard>()
        .add_message::<crate::hotbar::Gained>()
        .add_message::<crate::hotbar::Planted>()
        .add_message::<crate::husbandry::Deed>()
        .add_message::<crate::kitchen::Cooked>()
        .add_message::<crate::kitchen::Ate>()
        .add_message::<crate::missions::Did>()
        .add_message::<crate::missions::MissionDone>()
        .add_systems(
            Update,
            (
                read_surroundings,
                walker_sounds.run_if(crate::loading::ready),
                deeds,
                fishing,
                menus,
                fires,
                falls,
                thunder_from_strikes,
                scatters,
            ),
        );
}

/// The point a hand-held action sounds at: a step in front of the eye.
fn in_hand(listener: &Listener) -> Where {
    Where::At(listener.eye + listener.forward * 1.2)
}

/// Biome to the beds' index (`pbd_core::audio::BIOMES` order).
fn biome_index(biome: Biome) -> usize {
    match biome {
        Biome::Ocean => 0,
        Biome::Beach => 1,
        Biome::Fields => 2,
        Biome::Desert => 3,
        Biome::Jungle => 4,
        Biome::Swamp => 5,
        Biome::Mountains => 6,
        Biome::Tundra => 7,
    }
}

/// A point `metres` along the ground from `up` toward `toward`.
fn along(up: Vec3, toward: Vec3, metres: f32) -> Vec3 {
    let tangent = (toward - up * toward.dot(up)).normalize_or_zero();
    let angle = metres / PLANET_RADIUS;
    (up * angle.cos() + tangent * angle.sin()).normalize_or(up)
}

/// Whether the eye is under the water's surface.
pub fn eye_under_water(eye: Vec3, water: Option<&EyeWaterState>, sheet_radius: f32) -> bool {
    match water.map(|w| w.0) {
        Some(EyeWater::Water { surface_radius }) => eye.length() < surface_radius,
        Some(EyeWater::Air) => false,
        _ => {
            crate::planet::surface_height(eye.normalize_or(Vec3::Y)) < 0.0
                && eye.length() < sheet_radius
        }
    }
}

/// The listener's surroundings, four times a second (design section 5), and
/// the world bus's state.
#[allow(clippy::too_many_arguments)]
fn read_surroundings(
    time: Res<Time>,
    listener: Res<Listener>,
    data: Res<AudioData>,
    contact: Option<Res<PlanetContact>>,
    weather: Option<Res<crate::weather::Weather>>,
    sun: Option<Res<crate::sky::Sun>>,
    towns: Option<Res<crate::towns::Towns>>,
    water: Option<Res<EyeWaterState>>,
    settings: Option<Res<crate::config::WaterSettings>>,
    menu: Option<Res<crate::controls::MenuOpen>>,
    walking: Option<Res<WalkingState>>,
    mut env: ResMut<Environment>,
    mut last: Local<(f32, Vec3)>,
) {
    let _spent = super::Spent::new("read_surroundings");
    let _ = menu;
    let (since, was) = &mut *last;
    *since += time.delta_secs();
    if *since < 0.25 || !listener.valid {
        return;
    }
    let dt = *since;
    *since = 0.0;
    let Some(contact) = contact else {
        return;
    };
    let mix = &data.mix;
    let eye = listener.eye;
    let up = eye.normalize_or(Vec3::Y);
    let here = contact.sample(up);
    let height = eye.length() - here.radius;
    let sheet = PLANET_RADIUS - settings.as_ref().map_or(0.5, |s| s.depth_offset_m);
    let underwater = eye_under_water(eye, water.as_deref(), sheet);
    let sheltered = weather.as_ref().is_some_and(|w| w.sheltered);
    let underground = if sheltered && !underwater {
        (here.radius - eye.length()).max(0.0)
    } else {
        0.0
    };
    let airspeed = if *was == Vec3::ZERO {
        0.0
    } else {
        eye.distance(*was) / dt.max(1e-3)
    };
    *was = eye;
    // Biomes at the eye and a ring 150 m out, so a border crossfades.
    let config = crate::planet::terrain_config();
    let mut biomes = [0.0f32; BIOMES];
    biomes[biome_index(pbd_core::planet_gen::biome(config, up))] += 2.0;
    let side = up.any_orthonormal_vector();
    for k in 0..6 {
        let toward = Quat::from_axis_angle(up, k as f32 * std::f32::consts::TAU / 6.0) * side;
        let point = along(up, up + toward, 150.0);
        biomes[biome_index(pbd_core::planet_gen::biome(config, point))] += 1.0;
    }
    for b in &mut biomes {
        *b /= 8.0;
    }
    // The shore: the nearest change between land and sea within 200 m.
    let wet_here = here.water_depth > 0.0;
    let mut shore = None;
    'rings: for metres in [25.0, 50.0, 100.0, 150.0, 200.0] {
        for k in 0..8 {
            let toward = Quat::from_axis_angle(up, k as f32 * std::f32::consts::TAU / 8.0) * side;
            let wet = contact.sample(along(up, up + toward, metres)).water_depth > 0.0;
            if wet != wet_here {
                shore = Some(metres);
                break 'rings;
            }
        }
    }
    let on_foot = walking.as_ref().is_some_and(|w| w.active);
    let open_water = wet_here && shore.is_none() && !on_foot;
    let town = towns.as_ref().and_then(|towns| {
        towns
            .held
            .iter()
            .map(|held| {
                // A town's edge is taken as 60 m from its anchor.
                (held.point(PLANET_RADIUS).distance(eye) - 60.0).max(0.0)
            })
            .min_by(f32::total_cmp)
            .filter(|d| *d < 150.0)
    });
    let sun_elevation = sun.as_ref().map_or(45.0, |s| {
        s.elevation(up).clamp(-1.0, 1.0).asin().to_degrees()
    });
    let air = air_density_scale(mix, height);
    let surroundings = Surroundings {
        height_m: height,
        airspeed_m_s: airspeed,
        rain: weather.as_ref().map_or(0.0, |w| w.liquid()),
        snowing: weather.as_ref().is_some_and(|w| w.snowing),
        sheltered,
        underground_m: underground,
        underwater,
        biomes,
        sun_elevation_deg: sun_elevation,
        shore_m: shore,
        open_water,
        town_m: town,
        air,
    };
    let cave = {
        let t =
            ((underground - mix.cave_from_m) / (mix.cave_full_m - mix.cave_from_m)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    env.underwater = underwater;
    env.sheltered = sheltered && underground < mix.cave_from_m;
    env.cave = cave;
    env.air = air;
    env.beds = Some(bed_weights(mix, &surroundings));
}

/// What the walker's sounds hear of the water at a body-local walker
/// position (its body's centre): whether the feet are in water, so a step is
/// a splash, and whether the body is, so entering and leaving it are heard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Underfoot {
    pub feet_wet: bool,
    pub in_water: bool,
}

/// [`Underfoot`] at `position`: the walker's own answer
/// ([`crate::walking::Sea`]), so a walker the physics wades is the walker
/// the audio splashes. The column's sea tops at the sea level; the drawn
/// sheet is half a metre under it, and the walker swims to the sheet, so
/// feet over the sheet are dry to both (owner, 2026-11-16: "water sfx also
/// plays when I'm standing on land").
pub(crate) fn water_underfoot(
    sea: &crate::walking::Sea,
    config: &WalkingConfig,
    position: Vec3,
) -> Underfoot {
    let wet = sea.state(config, position);
    Underfoot {
        feet_wet: wet.depth > 0.0,
        in_water: wet.depth > HALF_HEIGHT,
    }
}

/// The walker's own sounds: steps, jumps, landings, and the water (design
/// section 4, Steps and Water).
#[derive(Default)]
struct Body {
    stride: Stride,
    was_grounded: bool,
    last_up_speed: f32,
    in_water: bool,
    eyes_under_s: f32,
    eyes_under: bool,
    swim: Stride,
    bubble_in_s: f32,
    seen: bool,
}

#[allow(clippy::too_many_arguments)]
fn walker_sounds(
    walking: Option<Res<WalkingState>>,
    config: Option<Res<WalkingConfig>>,
    walkers: Query<(&Position, &LinearVelocity, &GroundState), With<Walker>>,
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    contact: Option<Res<PlanetContact>>,
    sea: Option<crate::walking::Sea>,
    fine: Option<Res<PlanetFine>>,
    weather: Option<Res<crate::weather::Weather>>,
    listener: Res<Listener>,
    env: Res<Environment>,
    data: Res<AudioData>,
    time: Res<Time>,
    mut sfx: ResMut<Sfx>,
    mut body: Local<Body>,
) {
    let _spent = super::Spent::new("walker_sounds");
    let (Some(walking), Some(config), Some(contact), Some(sea)) = (walking, config, contact, sea)
    else {
        return;
    };
    let Ok((position, velocity, ground)) = walkers.single() else {
        return;
    };
    if !walking.active {
        body.seen = false;
        return;
    }
    let mix = &data.mix;
    let dt = time.delta_secs();
    let Some(frame) = frame else {
        return;
    };
    let position = (position.0.as_dvec3() - frame.physics_center()).as_vec3();
    let up = position.normalize_or(Vec3::Y);
    let feet = position - up * HALF_HEIGHT;
    let v = velocity.0;
    let up_speed = v.dot(up);
    let flat = (v - up * up_speed).length();
    let Underfoot { feet_wet, in_water } = water_underfoot(&sea, &config, position);
    let eyes_under = env.underwater;
    if !body.seen {
        // The first frame on foot sets the state without a sound.
        body.seen = true;
        body.was_grounded = ground.grounded;
        body.in_water = in_water;
        body.eyes_under = eyes_under;
        return;
    }
    // What is underfoot.
    let family = if ground.on.is_some() {
        Some(Family::Wood)
    } else if feet_wet {
        Some(Family::Shallow)
    } else {
        material_under(&contact, fine.as_deref(), feet)
            .and_then(surface_family)
            .or(Some(Family::Dirt))
    };
    let wet = weather
        .as_ref()
        .is_some_and(|w| w.wetness > mix.wet_step_from && !w.sheltered);
    let at = Where::At(feet);
    if ground.grounded && body.was_grounded {
        let stride = stride_m(mix, flat, config.walk_speed, config.sprint_speed);
        let steps = body.stride.advance(flat * dt, stride);
        if flat < 0.3 {
            body.stride.reset();
        }
        if steps > 0
            && let Some(family) = family
        {
            let quiet = if flat < config.walk_speed * 0.5 {
                -4.0
            } else {
                0.0
            };
            if wet && family == Family::Dirt {
                // Wet dirt is mud: the squelch is the step, brought up to a
                // step's level (`step_wet` sits 6 dB under `step_dirt` in
                // the book, as a layer). The owner, 2026-11-16: "walking
                // outside sounds like walking on wet wood, but the ground is
                // wet dirt".
                sfx.play_with("step_wet", at, quiet + 6.0, 1.0);
            } else {
                sfx.play_with(named("step", family), at, quiet, 1.0);
                if wet && family.takes_wet() {
                    sfx.play_with("step_wet", at, quiet, 1.0);
                }
            }
        }
    }
    if body.was_grounded && !ground.grounded && body.last_up_speed.max(up_speed) > 2.0 {
        if let Some(family) = family {
            sfx.play(named("jump", family), at);
        }
        body.stride.reset();
    }
    if !body.was_grounded && ground.grounded {
        let fall = -body.last_up_speed;
        if fall > mix.land_from_m_s
            && let Some(family) = family
        {
            // A landing is heard as an ordinary step, at a step's level,
            // however far the fall (owner, 2026-11-15: "I dont like the fall
            // thump, please get rid of that, regular step when falling and
            // hitting ground is fine"). The `land_*` and `land_hard` takes
            // stay in the book, unplayed.
            if wet && family == Family::Dirt {
                sfx.play_with("step_wet", at, 6.0, 1.0);
            } else {
                sfx.play(named("step", family), at);
            }
        }
        body.stride.reset();
    }
    // The water.
    if in_water && !body.in_water {
        let speed = (-body.last_up_speed).max(0.0).max(flat * 0.5);
        let cue = if speed > mix.enter_big_m_s {
            "water_enter_big"
        } else {
            "water_enter_small"
        };
        sfx.play(cue, at);
    }
    if !in_water && body.in_water && !eyes_under {
        sfx.play("water_exit", at);
    }
    if in_water && !ground.grounded {
        let strokes = body.swim.advance(v.length() * dt, 1.8);
        if strokes > 0 {
            let cue = if eyes_under {
                "swim_stroke_under"
            } else {
                "swim_stroke"
            };
            sfx.play(cue, Where::At(position));
        }
    } else {
        body.swim.reset();
    }
    if eyes_under {
        body.eyes_under_s += dt;
        body.bubble_in_s -= dt;
        if body.bubble_in_s <= 0.0 {
            sfx.play("bubbles", Where::At(listener.eye));
            body.bubble_in_s = 2.5 + (body.eyes_under_s * 7.3).sin().abs() * 2.5;
        }
    } else {
        if body.eyes_under && body.eyes_under_s > mix.breath_after_s {
            sfx.play("breath", Where::At(listener.eye));
        }
        body.eyes_under_s = 0.0;
        body.bubble_in_s = 1.0;
    }
    body.eyes_under = eyes_under;
    body.in_water = in_water;
    body.was_grounded = ground.grounded;
    body.last_up_speed = up_speed;
}

/// The block under a pair of feet, from the column tier where it is
/// resident.
pub fn material_under(
    contact: &PlanetContact,
    fine: Option<&PlanetFine>,
    feet: Vec3,
) -> Option<Material> {
    let fine = fine?;
    let record = contact.finest_cell(feet.normalize_or(Vec3::Y))?;
    let column = fine.set.columns.column(record)?;
    let layer = pbd_core::column::layer_at(feet.length() - PLANET_RADIUS - 0.3)?;
    let material = column.material(layer);
    if material == Material::Air {
        // Standing on a run's top: the layer below.
        return pbd_core::column::layer_at(feet.length() - PLANET_RADIUS - 1.0)
            .map(|l| column.material(l));
    }
    Some(material)
}

/// Phase 1's actions (build decision 6), from the messages they raise.
#[allow(clippy::too_many_arguments)]
fn deeds(
    listener: Res<Listener>,
    data: Res<AudioData>,
    mut gained: MessageReader<crate::hotbar::Gained>,
    mut planted: MessageReader<crate::hotbar::Planted>,
    mut deeds: MessageReader<crate::husbandry::Deed>,
    mut cooked: MessageReader<crate::kitchen::Cooked>,
    mut ate: MessageReader<crate::kitchen::Ate>,
    mut did: MessageReader<crate::missions::Did>,
    mut finished: MessageReader<crate::missions::MissionDone>,
    hotbar: Option<Res<crate::hotbar::Hotbar>>,
    mut sfx: ResMut<Sfx>,
    mut slot: Local<Option<usize>>,
) {
    let _spent = super::Spent::new("deeds");
    let hand = in_hand(&listener);
    if gained.read().count() > 0 {
        sfx.play("pickup", Where::Menu);
    }
    for _ in planted.read() {
        sfx.play("place_dirt", hand);
    }
    for deed in deeds.read() {
        match deed {
            crate::husbandry::Deed::PenBuilt { .. } => sfx.play("place_wood", hand),
            _ => sfx.play("ui_toggle", Where::Menu),
        }
    }
    if cooked.read().count() > 0 {
        sfx.play("pickup", Where::Menu);
    }
    // Eating munches (design "Eating munches"; owner, 2026-11-26): at the
    // eye, the player's own mouth, on the effects bus; the cue's cap of one
    // makes two meals in a frame one munch.
    for crate::kitchen::Ate { dish, .. } in ate.read() {
        let cue = pbd_core::audio::eat_cue(*dish, &data.mix);
        sfx.play(cue, Where::At(listener.eye));
    }
    for crate::missions::Did(happened) in did.read() {
        if happened.kind == pbd_core::missions::Kind::Till {
            sfx.play("dig_dirt", hand);
        }
    }
    // A mission done (owner, 2026-11-17: "a nice jingle when mission
    // success"): once a frame, however many closed in it.
    if finished.read().count() > 0 {
        sfx.play("mission_done", Where::Menu);
    }
    if let Some(hotbar) = hotbar {
        let now = hotbar.selected();
        if slot.is_some_and(|s| s != now) {
            sfx.play("ui_slot", Where::Menu);
        }
        *slot = Some(now);
    }
}

/// The rod (design section 4, Fishing).
fn fishing(
    mut heard: MessageReader<FishHeard>,
    fishery: Option<Res<crate::fish::Fishery>>,
    mut sfx: ResMut<Sfx>,
    listener: Res<Listener>,
) {
    let _spent = super::Spent::new("fishing");
    for FishHeard { event, at } in heard.read() {
        let at = Where::At(*at);
        match event {
            Event::Cast { speed_mps } => sfx.play_with(
                "cast_whoosh",
                in_hand(&listener),
                0.0,
                (0.85 + speed_mps / 40.0).clamp(0.85, 1.2),
            ),
            Event::Splash => sfx.play("float_plop", at),
            Event::Dry => sfx.play("float_dry", at),
            Event::Nibble { .. } => sfx.play("nibble_tick", at),
            Event::Bite { .. } => sfx.play("bite_splash", at),
            // A fish on the hook (owner, 2026-11-17: "some sort of jingle
            // when a fish is on the hook"); the catch has its chime.
            Event::Hooked { .. } => sfx.play("hook_jingle", Where::Menu),
            Event::Snapped { .. } => sfx.play("line_snap", in_hand(&listener)),
            Event::Caught { .. } => {
                sfx.play("catch_splash", at);
                sfx.play("catch_chime", Where::Menu);
            }
            _ => {}
        }
    }
    if let Some(fishery) = fishery
        && fishery.line.phase == pbd_core::fishing::Phase::Hooked
    {
        sfx.keep(("reel", 0), "reel_loop", in_hand(&listener), 0.0, 1.0);
    }
}

/// A button whose state changed this frame.
type ChangedButtons<'w, 's> =
    Query<'w, 's, (Entity, &'static Interaction), (Changed<Interaction>, With<Button>)>;

/// Every UI button: a hover and a press (design section 4, Menus).
fn menus(
    buttons: ChangedButtons,
    mut was: Local<std::collections::HashMap<Entity, Interaction>>,
    mut sfx: ResMut<Sfx>,
) {
    let _spent = super::Spent::new("menus");
    for (entity, interaction) in &buttons {
        let before = was.insert(entity, *interaction);
        match interaction {
            Interaction::Pressed => sfx.play("ui_click", Where::Menu),
            Interaction::Hovered if before != Some(Interaction::Pressed) => {
                sfx.play("ui_hover", Where::Menu)
            }
            _ => {}
        }
    }
    if was.len() > 4096 {
        was.clear();
    }
}

/// The fires the tier holds, by its fine version and light version: each
/// one's point and whether it is a campfire.
type FireCache = (u64, u64, Vec<(Vec3, bool)>);

/// The campfires and braziers near the ears crackle (build decision 7).
fn fires(
    fine: Option<Res<PlanetFine>>,
    listener: Res<Listener>,
    data: Res<AudioData>,
    mut sfx: ResMut<Sfx>,
    mut cache: Local<FireCache>,
) {
    let _spent = super::Spent::new("fires");
    let Some(fine) = fine else {
        return;
    };
    if !listener.valid {
        return;
    }
    let tier = &fine.set.columns;
    let (version, relit, fires) = &mut *cache;
    if *version != fine.version || *relit != tier.light_version() {
        *version = fine.version;
        *relit = tier.light_version();
        fires.clear();
        for spot in tier.lamps() {
            let Some(column) = tier.columns.get(spot.slot) else {
                continue;
            };
            let Some(layer) = pbd_core::column::layer_at(spot.altitude) else {
                continue;
            };
            let material = column.material(layer);
            if matches!(material, Material::Campfire | Material::Brazier) {
                let point = spot.direction * (PLANET_RADIUS + spot.altitude - 0.3);
                fires.push((point, material == Material::Campfire));
            }
        }
    }
    let reach = data.mix.fire_reach_m;
    let mut near: Vec<(f32, usize)> = fires
        .iter()
        .enumerate()
        .map(|(i, (p, _))| (p.distance(listener.eye), i))
        .filter(|(d, _)| *d < reach)
        .collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (_, i) in near.into_iter().take(4) {
        let (point, campfire) = fires[i];
        let key = (point.x.to_bits() as u64) << 32 | point.z.to_bits() as u64;
        let db = if campfire { 0.0 } else { -4.0 };
        sfx.keep(("fire", key), "fire_crackle", Where::At(point), db, 1.0);
    }
}

/// What `falls` keeps between frames: the heard list and the voices, both
/// capped at `fall_voices`, so nothing is allocated after the first frame.
#[derive(Default)]
struct FallSounds {
    heard: Vec<FallHeard>,
    voices: FallVoices,
}

/// Every fall sounds at its foot, light, medium or roaring by its drop
/// (`water-flow` design "The sound of a fall"): the loudest few at the ears,
/// each a placed loop fading in and out. The feet are the splash's own
/// (`FallFeet`, found by the water mesher).
fn falls(
    feet: Option<Res<crate::water_flow::FallFeet>>,
    listener: Res<Listener>,
    data: Res<AudioData>,
    time: Res<Time>,
    mut sfx: ResMut<Sfx>,
    mut state: Local<FallSounds>,
) {
    let _spent = super::Spent::new("falls");
    let state = &mut *state;
    let mix = &data.mix;
    let reach = FallBand::ALL.map(|band| {
        let cue = data.cues.cues.get(band.cue());
        FallReach {
            volume_db: cue.map_or(0.0, |c| c.volume_db),
            full_within_m: cue.map_or(1.0, |c| c.full_within_m),
            audible_m: cue.map_or(0.0, |c| c.audible_m),
        }
    });
    match feet.as_deref() {
        Some(feet) if listener.valid => loudest_falls(
            feet.0.iter().map(|f| (f.at, f.height)),
            listener.eye,
            mix,
            &reach,
            &state.voices.voices,
            &mut state.heard,
        ),
        _ => state.heard.clear(),
    }
    state.voices.update(&state.heard, mix, time.delta_secs());
    for voice in &state.voices.voices {
        sfx.keep(
            ("fall", voice.id),
            voice.band.cue(),
            Where::At(voice.at),
            pbd_core::audio::decibels(voice.gain()),
            1.0,
        );
    }
}

/// Thunder from the atmosphere's strikes (design section 6).
fn thunder_from_strikes(
    air: Option<Res<crate::atmosphere::Air>>,
    sun: Option<Res<crate::sky::Sun>>,
    listener: Res<Listener>,
    data: Res<AudioData>,
    time: Res<Time>,
    mut sfx: ResMut<Sfx>,
    mut heard: Local<(Option<u64>, f32)>,
) {
    let _spent = super::Spent::new("thunder_from_strikes");
    let (Some(air), Some(sun)) = (air, sun) else {
        return;
    };
    let (seen, quiet_s) = &mut *heard;
    *quiet_s += time.delta_secs();
    let newest = air.now.strikes.iter().map(|s| s.step).max();
    let Some(last) = *seen else {
        // Strikes already in the state at the start were before the player
        // arrived.
        *seen = Some(newest.unwrap_or(air.now.step));
        return;
    };
    if !listener.valid || newest.is_none_or(|n| n <= last) {
        return;
    }
    *seen = newest;
    let mix = &data.mix;
    let dt = air.now.settings.dt_s;
    let since_state = (sun.clock.seconds - air.at_seconds).max(0.0) as f32;
    // The nearest new strike speaks for its step (build decision 11); a far
    // one waits out `thunder_gap_s` after the last thunder, a near one
    // never does.
    let Some((strike, heard)) = air
        .now
        .strikes
        .iter()
        .filter(|s| s.step > last)
        .filter_map(|strike| {
            thunder(
                mix,
                listener.eye,
                strike.direction,
                PLANET_RADIUS,
                crate::sky::CLOUD_RADIUS,
                strike.strength,
            )
            .map(|heard| (strike, heard))
        })
        .min_by(|a, b| a.1.near_m.total_cmp(&b.1.near_m))
    else {
        return;
    };
    if heard.take == pbd_core::audio::ThunderTake::Roll && *quiet_s < mix.thunder_gap_s {
        return;
    }
    *quiet_s = 0.0;
    let age = air.now.step.saturating_sub(strike.step) as f32 * dt + since_state;
    // Placed at the bolt's nearest point, along the line to it.
    let toward = strike.direction * PLANET_RADIUS - listener.eye;
    let point = listener.eye + toward.normalize_or_zero() * heard.near_m;
    sfx.play_since(
        heard.take.cue(),
        Where::At(point),
        6.0 * strike.strength.clamp(0.0, 1.0),
        age,
    );
}

/// A short call placed around the listener now and then, so a bed is not one
/// loop heard again (design section 5, scatters).
fn scatters(
    time: Res<Time>,
    listener: Res<Listener>,
    env: Res<Environment>,
    mut sfx: ResMut<Sfx>,
    mut state: Local<(f32, u64)>,
) {
    let _spent = super::Spent::new("scatters");
    let Some(beds) = env.beds else {
        return;
    };
    let (wait, dice) = &mut *state;
    if *dice == 0 {
        *dice = 0x2545_F491_4F6C_DD1D;
        *wait = 12.0;
    }
    *wait -= time.delta_secs();
    if *wait > 0.0 || !listener.valid {
        return;
    }
    let mut roll = || {
        *dice ^= *dice << 13;
        *dice ^= *dice >> 7;
        *dice ^= *dice << 17;
        (*dice >> 40) as f32 / (1u64 << 24) as f32
    };
    let next = 10.0 + roll() * 25.0;
    let cue = if beds.weight(Bed::Gulls) > 0.2 {
        Some("gull")
    } else if beds.weight(Bed::NightCrickets) > 0.1 {
        Some("owl")
    } else if beds.weight(Bed::TownDay) + beds.weight(Bed::TownNight) > 0.2 {
        Some("dog_far")
    } else {
        None
    };
    if let Some(cue) = cue {
        let up = listener.eye.normalize_or(Vec3::Y);
        let angle = roll() * std::f32::consts::TAU;
        let side = Quat::from_axis_angle(up, angle) * up.any_orthonormal_vector();
        let metres = 30.0 + roll() * 170.0;
        let point = listener.eye + side * metres + up * 8.0;
        sfx.play(cue, Where::At(point));
    }
    *wait = next;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Design "Water sounds read the walker's water" (owner, 2026-11-16:
    /// "water sfx also plays when I'm standing on land (the blue stuff
    /// peeking above water)"). On the real column tier at a version 10 sea
    /// shore, a seabed block 1 m under the sea level, the sea's water in its
    /// column over it: the drawn sheet stands half a metre over the block,
    /// so feet on it are wet; feet a hand over the sheet, in the same
    /// column's sea, are dry to the steps. The column's own sea, which tops
    /// at the sea level, would call both wet. (The sea has no tide,
    /// `no-tides`: the sheet is one level.)
    #[test]
    fn feet_over_the_drawn_sheet_are_dry_underfoot() {
        use pbd_core::column::layer_altitude;
        let d = pbd_core::geo::direction(pbd_core::geo::LatLon {
            lat: 18.86_f32.to_radians(),
            lon: 27.54_f32.to_radians(),
        });
        let set = std::sync::Arc::new(crate::planet::lod::generate_fine(
            d,
            &crate::config::ColumnSettings::default(),
            &pbd_core::edits::Edits::new(),
        ));
        let sea_level = crate::planet::terrain_config().sea_level_m;
        let block = (0..set.finest_records().len())
            .find(|&i| {
                set.columns.column(i).is_some_and(|c| {
                    c.surface().is_some_and(|top| {
                        layer_altitude(top) + 1.0 == sea_level - 1.0 && c.water_over(top)
                    })
                })
            })
            .expect("a seabed block 1 m under the sea level at the shore");
        let up = Vec3::from_slice(&set.finest_records()[block].direction_height[..3]).normalize();
        let water = crate::config::WaterSettings::default();
        let sheet = sea_level - water.depth_offset_m;
        // On the block, under the sheet: wet. A hand over the sheet: dry.
        for (feet, wet) in [(sea_level - 1.0 + 0.01, true), (sheet + 0.05, false)] {
            let position = up * (PLANET_RADIUS + feet + HALF_HEIGHT);
            let mut contact = PlanetContact::test_planet(5);
            contact.set_fine(&set);
            let mut world = World::new();
            world.insert_resource(contact);
            world.insert_resource(water.clone());
            world.insert_resource(crate::sea::Sea::new(&water));
            let heard = bevy::ecs::system::RunSystemOnce::run_system_once(
                &mut world,
                move |sea: crate::walking::Sea| {
                    water_underfoot(&sea, &WalkingConfig::default(), position)
                },
            )
            .unwrap();
            assert_eq!(
                heard.feet_wet,
                wet,
                "feet at {feet:+.2} m: {}",
                if heard.feet_wet { "wet" } else { "dry" }
            );
        }
    }

    fn book() -> pbd_core::audio::CueBook {
        ron::from_str(include_str!("../../../../assets/config/audio/cues.ron")).unwrap()
    }

    /// Task 5.7: every cue the code names is in `cues.ron`.
    #[test]
    fn every_cue_the_code_names_is_in_the_book() {
        let book = book();
        let mut missing = Vec::new();
        let mut names: Vec<String> = NAMED.iter().map(|s| s.to_string()).collect();
        names.extend(family_cues());
        for take in [
            pbd_core::audio::ThunderTake::Crack,
            pbd_core::audio::ThunderTake::Close,
            pbd_core::audio::ThunderTake::Roll,
        ] {
            names.push(take.cue().into());
        }
        for material in Material::ALL {
            if let Some(cue) = pbd_core::audio::place_cue(material) {
                names.push(cue);
            }
        }
        for name in names {
            if !book.cues.contains_key(&name) {
                missing.push(name);
            }
        }
        assert!(
            missing.is_empty(),
            "named in code, not in cues.ron: {missing:?}"
        );
        assert!(book.problems().is_empty(), "{:?}", book.problems());
    }

    /// Task 5.7: every file `cues.ron` names exists, every placed cue is
    /// mono and every menu cue too (the menus are unplaced but mono files),
    /// and every bed exists and is stereo.
    #[test]
    fn every_file_exists_and_placed_files_are_mono() {
        let book = book();
        let dir = super::super::engine::audio_dir();
        let mut problems = Vec::new();
        for (name, cue) in &book.cues {
            for file in cue.files(name) {
                let path = dir.join("sfx").join(&file);
                match ogg_channels(&path) {
                    Some(1) => {}
                    Some(n) => problems.push(format!("{file}: {n} channels, placed cues are mono")),
                    None => problems.push(format!("{file}: missing or not Ogg")),
                }
            }
        }
        for bed in Bed::ALL {
            let path = dir.join("ambience").join(format!("{}.ogg", bed.name()));
            match ogg_channels(&path) {
                Some(2) => {}
                other => problems.push(format!(
                    "{}: {other:?} channels, beds are stereo",
                    bed.name()
                )),
            }
        }
        let levels: pbd_core::audio::Ambience =
            ron::from_str(include_str!("../../../../assets/config/audio/ambience.ron")).unwrap();
        for bed in Bed::ALL {
            if !levels.beds.contains_key(bed.name()) {
                problems.push(format!("{} has no level in ambience.ron", bed.name()));
            }
        }
        let mix: pbd_core::audio::MixConfig =
            ron::from_str(include_str!("../../../../assets/config/audio/mix.ron")).unwrap();
        problems.extend(mix.problems());
        assert!(problems.is_empty(), "{problems:#?}");
    }

    /// Design "Eating munches": a cooked dish eaten asks for `eat_soft`, a
    /// crisp food (the glowing mushroom, eaten raw) for `eat_crisp`, each at
    /// the eye and nowhere else.
    #[test]
    fn eating_asks_for_a_munch_at_the_eye() {
        use pbd_core::cooking::Effect;
        use pbd_core::goods::Good;
        let eye = Vec3::new(0.0, 4801.6, 0.0);
        // A fresh world for each meal: a reader run once starts from the
        // first message in the queue.
        let heard = |dish: &str, effect| {
            let mut world = World::new();
            world.init_resource::<Messages<crate::hotbar::Gained>>();
            world.init_resource::<Messages<crate::hotbar::Planted>>();
            world.init_resource::<Messages<crate::husbandry::Deed>>();
            world.init_resource::<Messages<crate::kitchen::Cooked>>();
            world.init_resource::<Messages<crate::kitchen::Ate>>();
            world.init_resource::<Messages<crate::missions::Did>>();
            world.init_resource::<Messages<crate::missions::MissionDone>>();
            world.init_resource::<Sfx>();
            world.insert_resource(Listener {
                eye,
                valid: true,
                ..Listener::default()
            });
            world.insert_resource(AudioData {
                cues: book(),
                mix: ron::from_str(include_str!("../../../../assets/config/audio/mix.ron"))
                    .unwrap(),
                ambience: Default::default(),
            });
            world.write_message(crate::kitchen::Ate {
                dish: Good::by_code(dish).unwrap(),
                effect,
            });
            bevy::ecs::system::RunSystemOnce::run_system_once(&mut world, deeds).unwrap();
            let mut sfx = world.resource_mut::<Sfx>();
            std::mem::take(&mut sfx.requests)
                .into_iter()
                .map(|r| (r.cue, r.at))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            heard("grilled-fish", Effect::Steady),
            vec![("eat_soft", Where::At(eye))]
        );
        assert_eq!(
            heard("glowing-mushroom", Effect::Glow),
            vec![("eat_crisp", Where::At(eye))]
        );
        let book = book();
        for cue in ["eat_crisp", "eat_soft"] {
            assert_eq!(book.cues[cue].bus, pbd_core::audio::Bus::Effects, "{cue}");
        }
    }

    /// Design "Eating munches": every munch take is a file that decodes,
    /// mono, about a second long.
    #[test]
    fn every_munch_take_decodes() {
        let book = book();
        let dir = super::super::engine::audio_dir().join("sfx");
        for cue in ["eat_crisp", "eat_soft"] {
            let files = book.cues[cue].files(cue);
            assert!(files.len() >= 3, "{cue}: {} takes", files.len());
            for file in files {
                let data = kira::sound::static_sound::StaticSoundData::from_file(dir.join(&file))
                    .unwrap_or_else(|e| panic!("{file}: {e}"));
                let seconds = data.duration().as_secs_f64();
                assert!((0.5..=1.5).contains(&seconds), "{file}: {seconds:.2} s");
                assert_eq!(ogg_channels(&dir.join(&file)), Some(1), "{file}");
            }
        }
    }

    /// The channel count from an Ogg Vorbis identification header.
    fn ogg_channels(path: &std::path::Path) -> Option<u8> {
        let bytes = std::fs::read(path).ok()?;
        let at = bytes.windows(7).position(|w| w == b"\x01vorbis")?;
        bytes.get(at + 11).copied()
    }
}
