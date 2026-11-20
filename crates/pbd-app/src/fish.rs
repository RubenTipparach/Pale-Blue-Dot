//! Fish in the app: schools spawned in the water round the player by the
//! species' own rule, stepped, drawn, and fished with the rod.
//!
//! Every rule is the core's (`pbd_core::fauna`, `pbd_core::fishing`). This
//! adapts them to the world: the sea surface is the one wave table the hulls
//! float on, the bed is the ground rule the craft use (`vehicles::ground_under`),
//! the water's class comes from the terrain generator and its temperature
//! from the simulated atmosphere, now. A catch lands in the hotbar and in the
//! save on the frame it is landed.
//!
//! The fish are not entities. A school is one entity carrying one mesh that
//! is rebuilt from the school's packed arrays each frame, which is the
//! repository's rule for what there are many of.

use crate::atmosphere::Air;
use crate::config::FaunaConfig;
use crate::hotbar::Hotbar;
use crate::planet::PlanetContact;
use crate::saves::WorldSave;
use crate::sea::Sea;
use crate::walking::{HALF_HEIGHT, Walker, WalkingCamera, WalkingReadout};
use avian3d::prelude::Position;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use pbd_core::fauna::{self, HOME_BODY, School, Species, Water, WaterClass};
use pbd_core::fishing::{Angler, Controls, Event, Line, Phase};
use pbd_core::goods::Good;
use pbd_core::inventory::{Equipment, Item, Tool};
use pbd_core::locale::{Localized, tr, tr_with};
use pbd_core::sea::LocalSea;
use pbd_core::world_frame::SimOrigin;
use std::collections::HashMap;
use std::sync::Mutex;

/// The tool slot, as a resource: the core's `Equipment`, which the desktop's
/// picker changes and the save records.
#[derive(Resource, Default, Deref, DerefMut, Clone, Copy, Debug)]
pub struct ToolSlot(pub Equipment);

impl ToolSlot {
    /// The tool slot a world opens with: the save's; or, in a world that
    /// never recorded one, nothing for a world made with tool crafting and
    /// all four at the stone tier for one made before (`tool-crafting`
    /// decision 8).
    pub fn restore(save: &WorldSave) -> Self {
        Self(save.equipment.unwrap_or_else(|| {
            if save.identity.crafts_its_tools() {
                Equipment::new_world()
            } else {
                Equipment::default()
            }
        }))
    }
}

/// The water round the player: the schools in it, the line in it, and the
/// clocks that pace them.
#[derive(Resource)]
pub struct Fishery {
    pub schools: Vec<School>,
    pub line: Line,
    step_s: f32,
    spawn_s: f32,
    spawned: u64,
    rng: fauna::Rng,
    /// The bed under points already asked about, on a half-metre grid. The
    /// ground rule runs the terrain's noise, several microseconds a call, and
    /// a school asks it for every fish every step.
    beds: Mutex<HashMap<(i32, i32, i32), f32>>,
    /// The cast a refusal was last said for, so it is said once a cast.
    told_refusal: bool,
    /// Each school's fade by its seed, 0 drawn whole to 1 gone, and whether
    /// it is on its way out: a school fades in when spawned and out before
    /// it is dropped, never there or gone in one frame
    /// (`ocean-environments` B18).
    fades: HashMap<u64, (f32, bool)>,
    /// Whether the line out was cast with bait in the pack: its bites come
    /// more readily, and a fish landed on it uses one (`starter-missions`
    /// G1).
    pub baited: bool,
    /// Where the angler stood this frame, local: which way the fish on the
    /// line is drawn running (`fish-behaviour`).
    pub feet: Option<Vec3>,
}

/// How long a school takes to fade in when spawned, or out before it is
/// dropped, s: the sea places' landing fade.
const SCHOOL_FADE_S: f32 = 0.8;

impl Fishery {
    /// How far a school has faded out, 0 whole to 1 gone; one not yet
    /// stepped is gone, so it fades in from its first frame.
    /// How far the schools' clock has run from its last step to the next,
    /// 0 to 1: where between their two steps the fish are drawn
    /// (`movement-smoothness`).
    pub fn step_alpha(&self, step_hz: f32) -> f32 {
        if step_hz > 0.0 && step_hz.is_finite() {
            (self.step_s * step_hz).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }

    pub fn fade_of(&self, school: &School) -> f32 {
        self.fades.get(&school.seed).map_or(1.0, |f| f.0)
    }

    /// Fade every school a frame on: in, or out if it is leaving; forget
    /// the ones no longer there.
    fn fade(&mut self, dt: f32) {
        let step = dt / SCHOOL_FADE_S;
        let mut kept = HashMap::with_capacity(self.schools.len());
        for school in &self.schools {
            let (fade, leaving) = self
                .fades
                .get(&school.seed)
                .copied()
                .unwrap_or((1.0, false));
            let fade = if leaving {
                (fade + step).min(1.0)
            } else {
                (fade - step).max(0.0)
            };
            kept.insert(school.seed, (fade, leaving));
        }
        self.fades = kept;
    }
}

impl Default for Fishery {
    fn default() -> Self {
        Self {
            schools: Vec::new(),
            line: Line::new(0x0f15_4a11),
            step_s: 0.0,
            spawn_s: 0.0,
            spawned: 0,
            rng: fauna::Rng::new(0x5c4001),
            beds: Mutex::new(HashMap::new()),
            told_refusal: false,
            fades: HashMap::new(),
            baited: false,
            feet: None,
        }
    }
}

/// What the rod is doing, for the HUD: a line of text and, while a fish is on
/// or a cast is charging, a meter.
#[derive(Resource, Default, Clone, Debug, PartialEq)]
pub struct FishingStatus {
    pub text: String,
    pub meter: Option<(&'static str, f32)>,
    /// The fishing buttons, bottom left, while the rod is out at fishable
    /// water: what the left and the right click do now, and whether each does
    /// anything in this phase (owner, 2026-11-18: "need to show both right
    /// and left click for fishing, and they both need to be on the left
    /// corner of the screen"; `fishing-and-equipment`, "Both buttons, bottom
    /// left").
    pub keys: Option<[(&'static str, bool); 2]>,
    /// The bite: the left click's row turns yellow and shakes (owner,
    /// 2026-11-18: "when fish bites, the left click buton prompt should
    /// highlight yellow and vibrate").
    pub bite: bool,
    /// Seconds a one-off message (a catch, a snap) stays up.
    hold_s: f32,
}

impl FishingStatus {
    pub(crate) fn say(&mut self, text: impl Into<String>, hold_s: f32) {
        self.text = text.into();
        self.hold_s = hold_s;
    }
}

/// The body whose roster the water round the player holds.
#[derive(Resource, Clone, Debug)]
pub struct Body(pub String);

impl Default for Body {
    fn default() -> Self {
        Self(HOME_BODY.to_string())
    }
}

/// The fishing systems, so a reader of the same buttons can be ordered
/// against them: digging runs first and sees the line as the click found it.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FishSet;

pub struct FishPlugin;

/// The fishery's local positions move with a rebase (`world-frames`): the
/// line's float, every school, the drawn float's last place and the
/// under-water watch's eye.
fn rebase_fishery(world: &mut World, shift: Vec3) {
    if let Some(mut fishery) = world.get_resource_mut::<Fishery>() {
        fishery.line.rebase(shift);
        if let Some(feet) = fishery.feet.as_mut() {
            *feet -= shift;
        }
        for school in &mut fishery.schools {
            school.rebase(shift);
        }
    }
    if let Some(mut drawn) = world.get_resource_mut::<draw::DrawnFloat>() {
        drawn.rebase(shift);
    }
    if let Some(mut trail) = world.get_resource_mut::<draw::CastTrail>() {
        trail.rebase(shift);
    }
    if let Some(mut swim) = world.get_resource_mut::<draw::DrawnSwim>() {
        swim.rebase(shift);
    }
    if let Some(mut watch) = world.get_resource_mut::<WatchHook>()
        && let Some((eye, target)) = watch.view.as_mut()
    {
        *eye -= shift;
        *target -= shift;
    }
}

impl Plugin for FishPlugin {
    fn build(&self, app: &mut App) {
        crate::sim_origin::add_rebase_hook(app, rebase_fishery);
        app.add_plugins(places::SeaPlacesPlugin)
            .init_resource::<Fishery>()
            .add_message::<crate::missions::Did>()
            .init_resource::<FishingStatus>()
            .init_resource::<ToolSlot>()
            .init_resource::<Body>()
            .init_resource::<draw::DrawnFloat>()
            .init_resource::<draw::CastTrail>()
            .init_resource::<draw::DrawnSwim>()
            .init_resource::<draw::DrawnHeadings>()
            .add_systems(Startup, draw::spawn)
            .add_systems(
                Update,
                (
                    life_script.run_if(resource_exists::<LifeScript>),
                    fish,
                    draw::ease_float,
                    draw::schools,
                    draw::hud,
                )
                    .chain()
                    .in_set(FishSet),
            )
            // The float and the line are placed where the rod is drawn this
            // frame: after the camera follows the walker, before transforms
            // propagate (`fishing-and-equipment` design 4, the line's far
            // end).
            .add_systems(
                PostUpdate,
                draw::tackle
                    .after(crate::walking::follow_walker)
                    .after(crate::planet::update_planet_frame)
                    .before(bevy::transform::TransformSystems::Propagate),
            )
            .add_systems(
                First,
                watch_hook_restore.run_if(resource_exists::<WatchHook>),
            )
            .add_systems(
                PostUpdate,
                watch_hook_view
                    .run_if(resource_exists::<WatchHook>)
                    .after(draw::tackle)
                    .before(bevy::transform::TransformSystems::Propagate),
            );
    }
}

/// The water as a school or the line sees it: the sea's surface from one
/// local wave table, and the bed from the craft's ground rule, remembered.
pub(crate) struct PlanetWater<'a> {
    sea: &'a Sea,
    local: LocalSea,
    contact: Option<&'a PlanetContact>,
    beds: &'a Mutex<HashMap<(i32, i32, i32), f32>>,
}

impl<'a> PlanetWater<'a> {
    fn at(
        sea: &'a Sea,
        air: Option<&Air>,
        contact: Option<&'a PlanetContact>,
        beds: &'a Mutex<HashMap<(i32, i32, i32), f32>>,
        point: Vec3,
        seconds: f64,
    ) -> Self {
        let direction = point.normalize_or(Vec3::Y);
        let state = sea.state_at(air, direction);
        let local = sea
            .table
            .local(&state, direction, sea.depth_at(direction), seconds);
        Self {
            sea,
            local,
            contact,
            beds,
        }
    }
}

impl PlanetWater<'_> {
    /// The fresh water's top over the column at a point, radius from the
    /// planet's centre: a river, a lake, a pond or poured water standing
    /// over the sea's own cells (`fishing-anywhere` design 1). `None` at
    /// sea, on dry ground or off the column tier.
    fn fresh(&self, point: Vec3) -> Option<f32> {
        let contact = self.contact?;
        let up = point.normalize_or(Vec3::Y);
        let floor = self.bed(point);
        fresh_top(contact, up, floor)
    }
}

/// The fresh water standing in the column at `up` just over its ground
/// (`floor`, a radius): its top as a radius, if it stands more than a
/// quarter metre over the sea's own cells' top (`fishing-anywhere` design 1,
/// the rule the small boats float by).
pub fn fresh_top(contact: &PlanetContact, up: Vec3, floor: f32) -> Option<f32> {
    let top = contact.column_water(up * (floor + 0.3)).flatten()?;
    let sea_top = pbd_core::column::layer_altitude(crate::planet::column::sea_layer());
    (top > sea_top + 0.25).then_some(crate::planet::PLANET_RADIUS + top)
}

/// How far over a sea-level flat the line counts the sea standing, m.
const FLAT_WATER_M: f32 = 0.05;

impl Water for PlanetWater<'_> {
    fn surface(&self, point: Vec3) -> f32 {
        if let Some(fresh) = self.fresh(point) {
            return fresh;
        }
        let sea = self.sea.radius + self.local.height(point, self.sea.radius);
        // A flat at the sea's level is drawn under the sea, half a metre over
        // the sheet's rest (the sheet sits low so a beach stands clear of the
        // swell): a float coming down on it is on water, not dry ground
        // (owner, 2026-11-19, a float on open water: "The float came down on
        // dry ground"). Ground over the sea's level stays dry.
        let bed = self.bed(point);
        if bed <= crate::planet::PLANET_RADIUS + FLAT_WATER_M && bed > sea {
            return bed + FLAT_WATER_M;
        }
        sea
    }

    fn bed(&self, point: Vec3) -> f32 {
        let on = point.normalize_or(Vec3::Y) * self.sea.radius;
        let key = (
            (on.x * 2.0).round() as i32,
            (on.y * 2.0).round() as i32,
            (on.z * 2.0).round() as i32,
        );
        let mut beds = self
            .beds
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(bed) = beds.get(&key) {
            return *bed;
        }
        let centre = Vec3::new(key.0 as f32, key.1 as f32, key.2 as f32) * 0.5;
        let bed = crate::vehicles::ground_under(self.contact, centre.as_dvec3()) as f32;
        // A bounded memo: a walk across the world would otherwise keep every
        // cell it ever passed.
        if beds.len() > 200_000 {
            beds.clear();
        }
        beds.insert(key, bed);
        bed
    }
}

/// Everything the fishing step reads about the world.
#[derive(bevy::ecs::system::SystemParam)]
struct Surroundings<'w, 's> {
    fauna: Res<'w, FaunaConfig>,
    body: Res<'w, Body>,
    sea: Option<Res<'w, Sea>>,
    air: Option<Res<'w, Air>>,
    contact: Option<Res<'w, PlanetContact>>,
    frame: Res<'w, crate::planet::PlanetRenderFrame>,
    sun: Option<Res<'w, crate::sky::Sun>>,
    walking: Option<Res<'w, WalkingReadout>>,
    menu: Option<Res<'w, crate::controls::MenuOpen>>,
    key: Option<Res<'w, crate::controls::PickerKey>>,
    buttons: Option<Res<'w, ButtonInput<MouseButton>>>,
    time: Res<'w, Time>,
    walkers: Query<'w, 's, &'static Position, With<Walker>>,
    cameras: Query<'w, 's, &'static GlobalTransform, With<WalkingCamera>>,
    /// Aboard a paddled boat the seat is the angler (`small-boats` design 6).
    aboard: Option<Res<'w, crate::vehicles::Aboard>>,
    vehicles: Query<'w, 's, &'static crate::vehicles::Vehicle>,
    vehicle_view: Option<Res<'w, crate::vehicles::view::VehicleView>>,
    vehicle_cameras: Query<'w, 's, &'static GlobalTransform, With<crate::vehicles::VehicleCamera>>,
    /// Steady hands (`cooking` design S5).
    meals: Option<Res<'w, crate::kitchen::MealEffects>>,
    /// Which side of the water the eye is on: no cast from under it
    /// (`mission-playthrough` 3.6).
    eye_water: Option<Res<'w, crate::planet::EyeWaterState>>,
    water_settings: Option<Res<'w, crate::config::WaterSettings>>,
}

/// One frame of fishing: the line, the schools, and spawning round the
/// player.
#[allow(clippy::too_many_arguments)]
fn fish(
    world: Surroundings,
    tools: Res<ToolSlot>,
    mut fishery: ResMut<Fishery>,
    mut status: ResMut<FishingStatus>,
    mut hotbar: Option<ResMut<Hotbar>>,
    mut save: Option<ResMut<WorldSave>>,
    mut did: MessageWriter<crate::missions::Did>,
    mut heard: MessageWriter<crate::audio::hooks::FishHeard>,
    (tip_now, mut cue): (
        Option<Res<crate::held::RodTipNow>>,
        Option<ResMut<crate::held::RodCue>>,
    ),
    mut was_aboard: Local<Option<Entity>>,
) {
    let Some(sea) = world.sea.as_deref() else {
        return;
    };
    let dt = world.time.delta_secs().min(0.1);
    status.hold_s = (status.hold_s - dt).max(0.0);
    // Aboard a paddled boat (`small-boats` design 6): the craft, its seat
    // as the angler's place, and whether the seat view is the camera.
    let aboard = world.aboard.as_ref().and_then(|a| a.0);
    let afloat = aboard
        .and_then(|e| world.vehicles.get(e).ok())
        .filter(|v| v.craft.kind.paddled())
        .map(|v| &v.craft);
    // The fishery lives in local space (`world-frames`): the feet, the eye,
    // the line and the schools; the water and the air are asked at planet
    // points.
    let o = crate::sim_origin::from_render_frame(&world.frame);
    let home = |p: bevy::math::DVec3| {
        o.to_local(pbd_core::world_frame::WorldPos::new(
            crate::sim_origin::HOME,
            p,
        ))
    };
    let feet = match afloat {
        Some(craft) => Some(home(craft.exit())),
        None => world
            .walkers
            .iter()
            .next()
            .map(|p| p.0 - o.up_at(p.0) * HALF_HEIGHT),
    };
    let Some(feet) = feet else {
        return;
    };
    pbd_core::world_frame::check_local("angler", feet);
    fishery.feet = Some(feet);
    let up = o.up_at(feet);
    let roster = world.fauna.0.roster(&world.body.0);
    let settings = &world.fauna.0;
    let seconds = world.sun.as_ref().map_or(0.0, |sun| sun.clock.seconds);
    let air = world.air.as_deref();
    let contact = world.contact.as_deref();
    let fishery = &mut *fishery;

    // The line. Only the rod fishes, on foot or from a paddled boat's seat,
    // only with the pointer held and no menu or picker open; put the rod
    // away, or board or step off, and the line comes in.
    let walking =
        (world.walking.as_ref().is_some_and(|w| w.active) && aboard.is_none()) || afloat.is_some();
    let rod = tools.held() == Some(Tool::Rod);
    let moved = *was_aboard != aboard;
    *was_aboard = aboard;
    if (!rod || !walking || moved) && fishery.line.phase != Phase::Ready {
        let flock = settings.flock;
        // A fish on the line is lost with it, and the player is told so.
        if matches!(
            fishery.line.phase,
            Phase::Nibble | Phase::Bite | Phase::Hooked
        ) && let Some(school) = fishery
            .line
            .target
            .and_then(|t| fishery.schools.get(t.school))
        {
            let species = school.species;
            let _ = report(
                &mut status,
                Event::Lost { species },
                roster,
                None,
                None,
                o.query_point(fishery.line.float),
                false,
            );
        }
        fishery.line.wind_in(&mut fishery.schools, &flock);
    }
    let captured = match afloat {
        Some(_) => world.vehicle_view.as_ref().is_some_and(|v| v.captured),
        None => world.walking.as_ref().is_some_and(|w| w.captured),
    };
    let listening = walking
        && rod
        && captured
        && !world.menu.as_ref().is_some_and(|m| m.0)
        && !world.key.as_ref().is_some_and(|k| k.holding);
    let camera = match afloat {
        // The vehicle camera's look, from the seat's own eye: in the seat
        // view they are one place, and from the chase view the line still
        // leaves the paddler, not the camera behind them.
        Some(craft) => world.vehicle_cameras.iter().next().map(|t| {
            (
                home(craft.eye()),
                t.forward().as_vec3(),
                t.right().as_vec3(),
                t.up().as_vec3(),
            )
        }),
        None => world.cameras.iter().next().map(|t| {
            let eye = crate::sim_origin::local_from_render(&world.frame, t.translation());
            (
                eye,
                t.forward().as_vec3(),
                t.right().as_vec3(),
                t.up().as_vec3(),
            )
        }),
    };
    let mut under = false;
    let has_bait = hotbar
        .as_deref()
        .is_some_and(|h| h.count(Item::Good(Good::BAIT)) > 0);
    if let (Some((eye, look, right, cam_up)), true) = (camera, rod && walking) {
        let buttons = world.buttons.as_deref();
        let pressed = |b| listening && buttons.is_some_and(|i| i.just_pressed(b));
        // A line is not cast from under the water (owner, 2026-11-19: "the
        // line casting shouldnt work underwater").
        let sheet = crate::planet::PLANET_RADIUS
            - world
                .water_settings
                .as_deref()
                .map_or(0.5, |w| w.depth_offset_m);
        under = crate::audio::hooks::eye_under_water(
            o.query_point(eye),
            world.eye_water.as_deref(),
            sheet,
        );
        // Said when a cast is tried with nothing to put on the hook.
        if !has_bait && fishery.line.phase == Phase::Ready && pressed(MouseButton::Left) {
            status.say(tr("fishing.status.needs_bait"), 2.0);
        }
        let controls = Controls {
            // And never without a worm to put on the hook (owner,
            // 2026-11-19: "do not allow fishing without bait").
            pressed: pressed(MouseButton::Left)
                && !((under || !has_bait) && fishery.line.phase == Phase::Ready),
            held: listening && buttons.is_some_and(|i| i.pressed(MouseButton::Left)),
            released: buttons.is_some_and(|i| i.just_released(MouseButton::Left)),
            reel_in: pressed(MouseButton::Right),
        };
        let angler = Angler {
            // The rod tip as last drawn (`held.rs`, `RodTipNow`): eye
            // space, -z ahead. A cast leaves from where the drawn-back tip is
            // when the button comes up.
            tip: {
                let t = tip_now
                    .as_deref()
                    .map_or_else(crate::held::rod_tip, |t| t.0);
                eye + right * t.x + cam_up * t.y - look * t.z
            },
            look,
            feet,
        };
        let around = if fishery.line.is_out() {
            fishery.line.float
        } else {
            feet
        };
        let water = PlanetWater::at(
            sea,
            air,
            contact,
            &fishery.beds,
            o.query_point(around),
            seconds,
        );
        let factor = air.map_or(1.0, |air| {
            let sample = air.now.sample(o.up_at(around));
            settings
                .fishing
                .bite_factor(sample.cover, sample.rain_rate * 3600.0)
        });
        // A baited line draws bites more readily (`starter-missions` G1).
        let factor = settings
            .fishing
            .baited(factor, fishery.baited && fishery.line.is_out());
        // Steady hands: the line's rules eased while the effect runs.
        let fishing = pbd_core::cooking::steadied(
            &settings.fishing,
            world.meals.as_deref().map_or(1.0, |m| m.steady),
        );
        let events = fishery.line.step(
            &o,
            dt,
            controls,
            &angler,
            &mut fishery.schools,
            roster,
            &fishing,
            &settings.flock,
            &water,
            crate::sea::surface_gravity(),
            factor,
        );
        if let Some(cue) = cue.as_deref_mut() {
            cue.reeling = controls.held && fishery.line.phase == Phase::Hooked;
            let to_float = (fishery.line.float - angler.tip).normalize_or_zero();
            cue.pull = to_float.dot(right).clamp(-1.0, 1.0);
        }
        for event in events {
            // The rod's sounds (`audio-and-ambience` build decision 5).
            heard.write(crate::audio::hooks::FishHeard {
                event,
                at: o.query_point(fishery.line.float),
            });
            let float = o.query_point(fishery.line.float);
            if let Some(cue) = cue.as_deref_mut() {
                cue.events.push(event);
            }
            if let Event::Dry = event {
                // What the line read where it came down (`mission-playthrough`:
                // a float on open water said dry ground).
                let q = o.query_point(fishery.line.float);
                let up = q.normalize_or(Vec3::Y);
                info!(
                    "FISH_DRY at r {:.2} (alt {:.2}): surface {:.2}, bed {:.2}, fresh {:?}, sea radius {:.2}, column water {:?}",
                    q.length(),
                    q.length() - crate::planet::PLANET_RADIUS,
                    water.surface(q) - crate::planet::PLANET_RADIUS,
                    water.bed(q) - crate::planet::PLANET_RADIUS,
                    water.fresh(q).map(|f| f - crate::planet::PLANET_RADIUS),
                    sea.radius - crate::planet::PLANET_RADIUS,
                    contact
                        .and_then(|c| c.column_water(up * (crate::planet::PLANET_RADIUS + 50.0))),
                );
                info!(
                    "FISH_DRY ground: generated {:.2}, stand from above {:?}, stand at the sheet {:?}, column water at the bed {:?}",
                    crate::planet::surface_height(up),
                    contact.map(|c| c
                        .stand(up * (crate::planet::PLANET_RADIUS + 20.0))
                        .floor_radius
                        - crate::planet::PLANET_RADIUS),
                    contact
                        .map(|c| c.stand(up * sea.radius).floor_radius
                            - crate::planet::PLANET_RADIUS),
                    contact.and_then(|c| c.column_water(up * (crate::planet::PLANET_RADIUS - 2.0))),
                );
            }
            if let Event::Cast { .. } = event {
                // The bait goes on the hook as the line goes out.
                fishery.baited = hotbar
                    .as_deref()
                    .is_some_and(|h| h.count(Item::Good(Good::BAIT)) > 0);
            }
            for happened in report(
                &mut status,
                event,
                roster,
                hotbar.as_deref_mut(),
                save.as_deref_mut(),
                float,
                fishery.baited,
            ) {
                did.write(happened);
            }
        }
    }
    if status.hold_s <= 0.0 {
        status.text = idle_text(&fishery.line, rod && walking, roster);
    }
    status.keys = (rod && walking && !roster.is_empty()).then(|| {
        if under && fishery.line.phase == Phase::Ready {
            [("fishing.key.underwater", false), ("", false)]
        } else if !has_bait && fishery.line.phase == Phase::Ready {
            [("fishing.key.need_bait", false), ("", false)]
        } else {
            keys_for(fishery.line.phase)
        }
    });
    status.bite = status.keys.is_some() && fishery.line.phase == Phase::Bite;
    // The rod in hand moves with the line (`held.rs`, `rod-motion`).
    if let Some(cue) = cue.as_deref_mut() {
        cue.phase = fishery.line.phase;
        cue.power = fishery.line.power;
        cue.tension = fishery.line.tension;
        if fishery.line.phase != Phase::Hooked {
            cue.reeling = false;
        }
    }
    status.meter = match fishery.line.phase {
        Phase::Charging => Some(("cast", fishery.line.power)),
        Phase::Hooked => Some(("tension", fishery.line.tension)),
        _ => None,
    };

    // A float cast over a protected creature: the catch is refused, with the
    // reason, once a cast (`ocean-environments` 7.3).
    if fishery.line.is_out() {
        let float = fishery.line.float;
        let refused = fishery.schools.iter().find_map(|school| {
            let species = roster.get(school.species as usize)?;
            let reason = species.catch_refusal()?;
            (species.status == fauna::Status::Protected
                && school.centre().distance(float) <= species.sense_m.max(3.0))
            .then(|| {
                tr_with(
                    "fishing.refused",
                    &[("species", &species.name()), ("reason", &reason)],
                )
                .into_string()
            })
        });
        if let Some(text) = refused
            && !fishery.told_refusal
        {
            status.say(text, 3.0);
            fishery.told_refusal = true;
        }
    } else {
        fishery.told_refusal = false;
    }

    // The schools, on their own clock. A shy creature near the eye is
    // startled first: a clam shuts, a duck paddles off (7.2).
    let eye = camera.map_or(feet + up * 1.6, |c| c.0);
    let step = 1.0 / settings.spawn.step_hz;
    fishery.step_s = (fishery.step_s + dt).min(4.0 * step);
    while fishery.step_s >= step {
        fishery.step_s -= step;
        let splash = fishery.line.splash();
        let beds = &fishery.beds;
        for school in &mut fishery.schools {
            let Some(species) = roster.get(school.species as usize) else {
                continue;
            };
            let water = PlanetWater::at(
                sea,
                air,
                contact,
                beds,
                o.query_point(school.anchor),
                seconds,
            );
            school.notice(eye, species, SHY_HOLD_S);
            school.step(&o, step, species, &settings.flock, &water, splash);
        }
    }
    // `PBD_LIFE_TRACE=1`, a measurement instrument (`ocean-environments`
    // B18): each frame, each school's nearest creature to the eye, how far
    // under it is drawn and how far its school has faded, so an approach
    // can be read off the log frame by frame. Changes nothing drawn.
    if std::env::var_os("PBD_LIFE_TRACE").is_some() {
        let night = world
            .sun
            .as_ref()
            .is_some_and(|sun| sun.direction().dot(up) < -0.05);
        for school in &fishery.schools {
            let Some(species) = roster.get(school.species as usize) else {
                continue;
            };
            let nearest = (0..school.len()).min_by(|&a, &b| {
                school.positions[a]
                    .distance(eye)
                    .total_cmp(&school.positions[b].distance(eye))
            });
            if let Some(i) = nearest {
                info!(
                    "LIFE_TRACE {} eye {:.2} m under {:.3} fade {:.3} spook {:.1}",
                    species.id,
                    school.positions[i].distance(eye),
                    fauna::under_share(species, school, i, seconds, night),
                    fishery.fade_of(school),
                    school.spook[i],
                );
            }
        }
    }

    // Spawning and dropping round the player; a school fades as it comes
    // and goes.
    fishery.fade(dt);
    fishery.spawn_s += dt;
    let every = 1.0 / settings.spawn.attempts_per_s;
    if fishery.spawn_s >= every {
        fishery.spawn_s = 0.0;
        drop_far(fishery, feet, settings.spawn.despawn_m);
        if let Some(air) = air {
            try_spawn(
                &o, fishery, feet, sea, air, contact, roster, settings, seconds,
            );
        }
    }
}

/// `--watch-hook`, a recording instrument (`movement-smoothness` 7): the
/// frame is drawn from an eye under the water beside the hook, so a bite can
/// be watched from below. When the float lands an eye is placed to the
/// side of the lure (0.35 m under the float), 0.85 m off, a little under it, and looks
/// at the lure; it stays there for the cast. Only the drawn frame moves: the
/// walking camera is put back at the start of every frame, so the line, the
/// cast, the fish's shyness and everything else in `Update` see the angler's
/// own eye, and the line is placed from the rod before the view moves. The
/// rod in hand is hidden while the view is under water. Changes nothing a
/// player meets.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct WatchHook {
    /// The eye and the point it looks at, local (`world-frames`), for this
    /// cast.
    view: Option<(Vec3, Vec3)>,
    /// The angler's own camera, put back at the start of the next frame.
    angler: Option<(Transform, GlobalTransform)>,
}

/// Put the angler's camera back before anything reads it.
fn watch_hook_restore(
    mut watch: ResMut<WatchHook>,
    mut cameras: Query<(&mut Transform, &mut GlobalTransform), With<WalkingCamera>>,
) {
    if let Some((transform, global)) = watch.angler.take() {
        for (mut t, mut g) in &mut cameras {
            *t = transform;
            *g = global;
        }
    }
}

/// Draw this frame from under the water beside the hook.
#[allow(clippy::type_complexity)]
fn watch_hook_view(
    mut watch: ResMut<WatchHook>,
    fishery: Res<Fishery>,
    frame: Res<crate::planet::PlanetRenderFrame>,
    mut cameras: Query<(&mut Transform, &GlobalTransform), With<WalkingCamera>>,
    mut held: Query<&mut Visibility, With<crate::held::Held>>,
) {
    let line = &fishery.line;
    let wet = matches!(
        line.phase,
        Phase::Floating | Phase::Nibble | Phase::Bite | Phase::Hooked
    );
    if !wet {
        watch.view = None;
        return;
    }
    let Ok((mut transform, global)) = cameras.single_mut() else {
        return;
    };
    let o = crate::sim_origin::from_render_frame(&frame);
    let view = *watch.view.get_or_insert_with(|| {
        let float = line.float;
        let up = o.up_at(float);
        let eye = crate::sim_origin::local_from_render(&frame, transform.translation);
        let toward = eye - float;
        let toward = (toward - up * toward.dot(up)).normalize_or(up.any_orthonormal_vector());
        let side = up.cross(toward).normalize_or(up.any_orthonormal_vector());
        let lure = line.lure(&o);
        let target = lure + up * 0.1;
        (target + side * 0.85 + toward * 0.2 - up * 0.15, target)
    });
    watch.angler = Some((*transform, *global));
    let (eye, target) = view;
    let up = o.up_at(eye);
    let at = crate::render_origin::render_from_physics(&frame, eye);
    let look = crate::render_origin::render_from_physics(&frame, target);
    *transform = Transform::from_translation(at).looking_at(look, up);
    for mut v in &mut held {
        *v = Visibility::Hidden;
    }
}

/// `--life ID`, a capture instrument (`ocean-environments` 7.1): once the
/// walker is up, one group of that species is put in the water a few metres
/// ahead of the eye, where the spawner's ring (20 to 60 m out) never puts
/// one, so a capture can show it close. It changes nothing a player meets.
#[derive(Resource, Clone, Debug)]
pub struct LifeScript {
    pub id: String,
    /// How far ahead to put it, m; 3 to 14 m when `None`.
    pub metres: Option<f32>,
    pub placed: bool,
}

/// `PBD_SCHOOL_DEPTH=sea`: a capture instrument (`ocean-environments`
/// task 18.2) that reads a school's depth off the sea's mean sphere, the
/// rule before design B23, so the before frame of a school in a lake can be
/// taken beside the after. Unset, a school's depth is read from the surface
/// of the water it is in. It changes nothing a player meets.
fn school_depth_from_sea() -> bool {
    static FROM_SEA: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *FROM_SEA.get_or_init(|| std::env::var("PBD_SCHOOL_DEPTH").is_ok_and(|v| v == "sea"))
}

#[allow(clippy::too_many_arguments)]
fn life_script(
    mut script: ResMut<LifeScript>,
    mut fishery: ResMut<Fishery>,
    world: Surroundings,
    mut waited: Local<u32>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    life: Option<Res<draw::LifeMaterial>>,
    settings: Option<Res<crate::config::WaterSettings>>,
) {
    if script.placed || !world.walking.as_ref().is_some_and(|w| w.active) {
        return;
    }
    // Five seconds on foot first, so a walker over water has sunk to the bed
    // and the group is put where the eye then is.
    *waited += 1;
    if *waited < 300 {
        return;
    }
    let (Some(sea), Some(camera)) = (world.sea.as_deref(), world.cameras.iter().next()) else {
        return;
    };
    let roster = world.fauna.0.roster(&world.body.0);
    if script.id == "lineup" {
        let eye = (camera.translation().as_dvec3() - world.frame.center).as_vec3();
        let look = camera.forward().as_vec3();
        let water = PlanetWater::at(
            sea,
            world.air.as_deref(),
            world.contact.as_deref(),
            &fishery.beds,
            eye,
            0.0,
        );
        let seconds = world.sun.as_ref().map_or(0.0, |sun| sun.clock.seconds);
        let above = eye.length() > water.surface(eye);
        let under = places::UnderWater::of(settings.as_deref());
        let mesh = lineup(eye, look, &water, seconds, above, under);
        if let Some(life) = life {
            commands.spawn((
                Name::new("Water life lineup"),
                crate::field_light::LitByField,
                crate::field_light::LitLikeTerrain,
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(life.0.clone()),
                Transform::from_translation(world.frame.center.as_vec3()),
                Visibility::default(),
            ));
            info!("life script: the lineup of every body put ahead");
        }
        script.placed = true;
        return;
    }
    let Some(index) = roster.iter().position(|sp| sp.id == script.id) else {
        warn!("--life {}: no such species", script.id);
        script.placed = true;
        return;
    };
    let eye = (camera.translation().as_dvec3() - world.frame.center).as_vec3();
    let up = eye.normalize_or(Vec3::Y);
    let look = camera.forward().as_vec3();
    let ahead = (look - up * look.dot(up)).normalize_or(up.any_orthonormal_vector());
    // The first point ahead deep enough for it, 3 to 14 m out.
    let species = &roster[index];
    let seconds = world.sun.as_ref().map_or(0.0, |sun| sun.clock.seconds);
    let first = script.metres.unwrap_or(3.0);
    for step in 0..=22 {
        let metres = first + step as f32 * 0.5;
        let direction = (eye + ahead * metres).normalize_or(up);
        // Fresh water over the sea's level (a lake, a river, a pond) is
        // asked as the spawner asks it (task 18.2); the sea otherwise.
        let contact = world.contact.as_deref();
        let floor =
            crate::vehicles::ground_under(contact, direction.as_dvec3() * f64::from(sea.radius))
                as f32;
        let fresh = contact.and_then(|c| fresh_top(c, direction, floor));
        let depth = fresh.map_or_else(|| sea.depth_at(direction), |top| top - floor);
        if !species.fits(depth) {
            continue;
        }
        let top = fresh.unwrap_or(sea.radius);
        let anchor = direction * top;
        let water = PlanetWater::at(
            sea,
            world.air.as_deref(),
            world.contact.as_deref(),
            &fishery.beds,
            anchor,
            seconds,
        );
        let o = crate::sim_origin::from_render_frame(&world.frame);
        let local = o.at(direction.as_dvec3(), f64::from(top));
        let school = School::spawn(&o, index as u16, species, local, 0x11fe, &water);
        info!(
            "life script: {} x{} put {metres} m ahead in {} water {depth:.1} m deep",
            species.id,
            school.len(),
            if fresh.is_some() { "fresh" } else { "sea" }
        );
        fishery.schools.push(school);
        script.placed = true;
        return;
    }
}

/// `--life lineup`, a capture instrument (`ocean-environments` B8): every
/// body side by side on the bed ahead of the eye, each 0.5 m long (a worm
/// 0.8 m tall) whatever its species' size, in rows of five: first as each
/// swims, then the poses (a clam shut, a dabbler upended and asleep, a worm
/// drawn in, an octopus inking, a seal hauled out), so the bodies can be
/// judged beside the mockup's. Static: posed at the clock it was put at.
fn lineup(
    eye: Vec3,
    look: Vec3,
    water: &impl fauna::Water,
    seconds: f64,
    floaters_up: bool,
    under: places::UnderWater,
) -> Mesh {
    use bodies::{Look, Place, emit, model_for};
    use fauna::Pose;
    let up = eye.normalize_or(Vec3::Y);
    let ahead = (look - up * look.dot(up)).normalize_or(up.any_orthonormal_vector());
    let right = ahead.cross(up);
    let mut entries: Vec<(&str, u32, Pose)> = bodies::MODELS
        .iter()
        .filter(|m| !m.starts_with("dabbler"))
        .map(|m| (*m, 0, Pose::Moving))
        .collect();
    entries.insert(13, ("reed_dabbler", 0, Pose::Moving));
    entries.insert(14, ("reed_dabbler", 1, Pose::Moving));
    entries.extend([
        ("giant_clam", 0, Pose::Shut),
        ("reed_dabbler", 0, Pose::TippedUp),
        ("reed_dabbler", 1, Pose::Asleep),
        ("tube_worm", 0, Pose::Shut),
        ("octopus", 0, Pose::Inking),
        ("seal", 0, Pose::HauledOut),
    ]);
    let t = seconds.rem_euclid(3600.0) as f32;
    let (mut positions, mut colours, mut uvs) = (Vec::new(), Vec::new(), Vec::new());
    for (k, (model, id, pose)) in entries.into_iter().enumerate() {
        let (row, col) = ((k / 5) as f32, (k % 5) as f32);
        let at = eye + ahead * (2.2 + row * 0.9) + right * ((col - 2.0) * 0.75);
        let dir = at.normalize_or(up);
        let bed = water.bed(dir * water.surface(at));
        let body = model_for(model, id);
        let size = if body.kind == bodies::Kind::TubeWorm {
            0.8
        } else {
            0.5
        };
        // From above the water the floaters ride the surface.
        let floats = matches!(
            body.kind,
            bodies::Kind::Bird | bodies::Kind::Otter | bodies::Kind::Seal
        );
        let radius = if floaters_up && floats {
            water.surface(at) - 0.03
        } else {
            bed + 0.02
        };
        let place = Place {
            origin: dir * radius,
            // Each turned three-quarters to the eye, so its side and its
            // face both show.
            forward: (-right * 0.8 - ahead * 0.6).normalize(),
            up: dir,
            scale: size / body.length,
        };
        let look = Look {
            t,
            phase: k as f32,
            pose,
        };
        // Tinted by the water over it as the bed it lies on is (B23), so a
        // body beside the sand can be read against it.
        let tint = under.albedo([1.0; 3], water.surface(at) - radius);
        emit(
            body,
            &look,
            &place,
            tint,
            &mut positions,
            &mut colours,
            &mut uvs,
        );
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.compute_flat_normals();
    mesh
}

/// How long a startled shy creature stays so after the eye has gone, s.
const SHY_HOLD_S: f32 = 6.0;

/// Drop the schools that have been fished out, and the ones gone too far
/// once they have faded out (a far one starts fading here), keeping the
/// line's target pointing at the same school.
fn drop_far(fishery: &mut Fishery, feet: Vec3, despawn_m: f32) {
    let target = fishery.line.target.map(|t| t.school);
    for (k, school) in fishery.schools.iter().enumerate() {
        if Some(k) != target
            && school.centre().distance(feet) > despawn_m
            && let Some(fade) = fishery.fades.get_mut(&school.seed)
        {
            fade.1 = true;
        }
    }
    let fades = &fishery.fades;
    let mut index = 0;
    let mut remap = Vec::with_capacity(fishery.schools.len());
    fishery.schools.retain(|school| {
        let gone = fades.get(&school.seed).is_some_and(|f| f.1 && f.0 >= 1.0);
        let keep = Some(remap.len()) == target || (!school.is_empty() && !gone);
        remap.push(keep.then_some(index));
        if keep {
            index += 1;
        }
        keep
    });
    if let Some(t) = fishery.line.target.as_mut()
        && let Some(Some(new)) = remap.get(t.school)
    {
        t.school = *new;
    }
}

/// One attempt to spawn a school in the ring round the player: a random
/// place, its water class and temperature now, and a species that lives
/// there and is under its cap.
#[allow(clippy::too_many_arguments)]
fn try_spawn(
    o: &pbd_core::world_frame::SimOrigin,
    fishery: &mut Fishery,
    feet: Vec3,
    sea: &Sea,
    air: &Air,
    contact: Option<&PlanetContact>,
    roster: &[Species],
    settings: &fauna::FaunaSettings,
    seconds: f64,
) {
    // Fish and the rest of the water life have their own caps, so the life
    // never crowds out what a line is cast for (`ocean-environments` 7.1).
    let biting = |school: &School| {
        roster
            .get(school.species as usize)
            .is_some_and(Species::can_bite)
    };
    let fish_now = fishery.schools.iter().filter(|s| biting(s)).count();
    let life_now = fishery.schools.len() - fish_now;
    let fish_room = fish_now < settings.spawn.max_schools as usize;
    let life_room = life_now < settings.spawn.max_life as usize;
    if roster.is_empty() || (!fish_room && !life_room) {
        return;
    }
    // The place is found by planet directions; the school is put in local
    // space (`world-frames`).
    let up = o.up_at(feet);
    let east = Vec3::Y.cross(up).normalize_or(Vec3::X);
    let north = up.cross(east);
    let feet = o.query_point(feet);
    let bearing = fishery.rng.range(0.0, std::f32::consts::TAU);
    let distance = fishery
        .rng
        .range(settings.spawn.ring_m.0, settings.spawn.ring_m.1);
    let direction =
        (feet + (east * bearing.cos() + north * bearing.sin()) * distance).normalize_or(up);
    // Fresh water the world holds (a river, a lake, a pond, poured water)
    // is river water at its own top (`fishing-anywhere` design 2); the sea
    // keeps its classes.
    let floor =
        crate::vehicles::ground_under(contact, direction.as_dvec3() * f64::from(sea.radius)) as f32;
    let fresh = contact.and_then(|c| fresh_top(c, direction, floor));
    let (depth, class) = match fresh {
        Some(top) => (top - floor, WaterClass::River),
        None => {
            let depth = sea.depth_at(direction);
            let Some(class) = fauna::water_class(
                crate::planet::terrain_config(),
                direction,
                depth,
                &settings.water,
            ) else {
                return;
            };
            (depth, class)
        }
    };
    let temperature = air.now.sample(direction).temperature;
    // The place, from the one environment function the generator and the
    // map read (`ocean-environments` 6.1); unknown on a world made before
    // the sea's places, whose life spawns as it always did.
    let terrain = crate::planet::terrain_config();
    let here = terrain
        .ocean
        .is_some()
        .then(|| pbd_core::ocean::places::environment(terrain, direction))
        .flatten();
    let candidates: Vec<usize> = roster
        .iter()
        .enumerate()
        .filter(|(i, sp)| {
            sp.lives_in(class, temperature, &settings.water)
                && sp.fits(depth)
                && sp.env_allows(here)
                && if sp.can_bite() { fish_room } else { life_room }
                && fishery
                    .schools
                    .iter()
                    .filter(|s| s.species as usize == *i)
                    .count()
                    < sp.max_schools as usize
        })
        .map(|(i, _)| i)
        .collect();
    if candidates.is_empty() {
        return;
    }
    let pick = candidates[(fishery.rng.next_u64() % candidates.len() as u64) as usize];
    fishery.spawned += 1;
    let seed = fauna::hash2(
        fishery.spawned,
        direction
            .to_array()
            .iter()
            .fold(0u64, |h, v| fauna::hash2(h, u64::from(v.to_bits()))),
    );
    let anchor = direction * fresh.unwrap_or(sea.radius);
    let water = PlanetWater::at(sea, Some(air), contact, &fishery.beds, anchor, seconds);
    let local = o.at(direction.as_dvec3(), f64::from(fresh.unwrap_or(sea.radius)));
    let mut school = School::spawn(o, pick as u16, &roster[pick], local, seed, &water);
    // Smaller water, smaller fish (`fishing-anywhere` design 3).
    if let (Some(top), Some(contact)) = (fresh, contact) {
        school.scale = body_scale(direction, top - floor, |p| {
            let floor = crate::vehicles::ground_under(Some(contact), p.as_dvec3()) as f32;
            fresh_top(contact, p.normalize_or(Vec3::Y), floor).is_some()
        });
    }
    debug!(
        "{} school of {} spawned in {} at {:.1} C, {:.0} m out",
        roster[pick].id,
        school.len(),
        class.name(),
        temperature,
        distance
    );
    fishery.schools.push(school);
}

/// How big a body of fresh water is, as the size its fish grow to, 0.35 to
/// 1 (`fishing-anywhere` design 3): how far it reaches round `up` (the
/// furthest of 3, 6, 12 and 24 m at which at least half of eight bearings
/// are still water) against 24 m, and its depth against 3 m, the smaller of
/// the two.
pub fn body_scale(up: Vec3, depth_m: f32, wet: impl Fn(Vec3) -> bool) -> f32 {
    let up = up.normalize_or(Vec3::Y);
    let east = Vec3::Y.cross(up).normalize_or(Vec3::X);
    let north = up.cross(east);
    let radius = crate::planet::PLANET_RADIUS;
    let mut reach = 0.0_f32;
    for r in [3.0_f32, 6.0, 12.0, 24.0] {
        let wet_bearings = (0..8)
            .filter(|k| {
                let a = *k as f32 * std::f32::consts::TAU / 8.0;
                let at = (up * radius + (east * a.cos() + north * a.sin()) * r).normalize_or(up)
                    * radius;
                wet(at)
            })
            .count();
        if wet_bearings < 4 {
            break;
        }
        reach = r;
    }
    let size = (reach / 24.0).min(depth_m.max(0.0) / 3.0);
    (0.35 + 0.65 * size).clamp(0.35, 1.0)
}

/// What a line event means to the player, and what a catch does to the
/// hotbar and the save. What the starter missions count comes back: the
/// float on water, and a fish landed (`starter-missions` design 15).
fn report(
    status: &mut FishingStatus,
    event: Event,
    roster: &[Species],
    hotbar: Option<&mut Hotbar>,
    save: Option<&mut WorldSave>,
    float_at: Vec3,
    baited: bool,
) -> Vec<crate::missions::Did> {
    use pbd_core::missions::Kind;
    let name = |species: u16| {
        roster
            .get(species as usize)
            .map_or_else(|| tr("inventory.fish"), Species::name)
    };
    // The name mid-sentence ("the minnow").
    let lower = |species: u16| {
        roster
            .get(species as usize)
            .map_or_else(|| tr("inventory.fish"), Species::lower_name)
    };
    // The name missions and their saved progress count by.
    let data_name = |species: u16| {
        roster
            .get(species as usize)
            .map_or_else(|| "fish".to_string(), Species::data_name)
    };
    match event {
        Event::Dry => status.say(tr("fishing.status.dry"), 1.5),
        Event::Bite { .. } => status.say(tr("fishing.status.bite"), 0.3),
        Event::TooEarly { species } => status.say(
            tr_with("fishing.status.too_early", &[("species", &lower(species))]),
            2.0,
        ),
        Event::Missed { species } => status.say(
            tr_with("fishing.status.missed", &[("species", &lower(species))]),
            2.0,
        ),
        Event::Lost { species } => status.say(
            tr_with("fishing.status.lost", &[("species", &lower(species))]),
            2.0,
        ),
        Event::Snapped { species } => status.say(
            tr_with("fishing.status.snapped", &[("species", &lower(species))]),
            2.5,
        ),
        Event::Caught { species, length_cm } => {
            status.say(
                tr_with(
                    "fishing.status.caught",
                    &[("species", &name(species)), ("length", &length_cm)],
                ),
                3.0,
            );
            // A baited line's catch uses one bait, taken in the catch's own
            // durable write (`starter-missions` G1); given back if the fish
            // is let go.
            let bait = Item::Good(Good::BAIT);
            let mut hotbar = hotbar;
            let used_bait = baited && hotbar.as_deref_mut().is_some_and(|h| h.take_item(bait, 1));
            if land(species, length_cm, hotbar.as_deref_mut(), save, status) {
                use crate::missions::Did;
                let kind = data_name(species);
                let mut did = vec![Did::new(Kind::Catch, kind.clone(), 1)];
                if used_bait {
                    did.push(Did::new(Kind::Baited, kind.clone(), 1));
                }
                // Each kind, counted once a mission ("Three kinds of fish",
                // G4).
                did.push(Did::new(Kind::Species, kind, 1));
                // The sea or fresh water: water standing above the sea's
                // level is a river or a lake ("Off the shore").
                let fresh = float_at.length() - crate::planet::PLANET_RADIUS > 0.6;
                did.push(Did::new(
                    Kind::Waters,
                    if fresh { "fresh" } else { "sea" },
                    1,
                ));
                return did;
            }
            if used_bait && let Some(h) = hotbar {
                h.give(bait, 1);
            }
        }
        Event::Splash => return vec![crate::missions::Did::new(Kind::Cast, "water", 1)],
        _ => {}
    }
    Vec::new()
}

/// A landed fish goes into the hotbar and the catch goes into the save, on
/// the frame it is landed. A hotbar with no room releases it rather than
/// losing it silently.
pub fn land(
    species: u16,
    length_cm: u32,
    hotbar: Option<&mut Hotbar>,
    save: Option<&mut WorldSave>,
    status: &mut FishingStatus,
) -> bool {
    let Some(hotbar) = hotbar else {
        return false;
    };
    if hotbar.give(Item::Fish(species), 1) > 0 {
        status.say(tr("fishing.status.no_room"), 3.0);
        return false;
    }
    if let Some(save) = save
        && !save.record_catch(species, length_cm, hotbar)
    {
        warn!("catch not saved: the save writer has failed");
    }
    true
}

/// What the left and the right click do in `phase`, and whether each is
/// drawn lit (`fishing-and-equipment`, "Both buttons, bottom left"). The
/// right click's CANCEL is never lit (owner, 2026-11-18: "right click cancle
/// doesnt need to be highlighted"); the left lights when it does something.
pub fn keys_for(phase: Phase) -> [(&'static str, bool); 2] {
    // The right click shows only while the line is out (owner, 2026-11-18:
    // "right click doesnt show up if the line isnt cast").
    let out = matches!(
        phase,
        Phase::Flying | Phase::Floating | Phase::Nibble | Phase::Bite
    );
    let left = match phase {
        Phase::Ready => ("fishing.key.hold_to_cast", true),
        Phase::Charging => ("fishing.key.let_go_to_cast", true),
        Phase::Hooked => ("fishing.key.hold_to_reel", true),
        Phase::Bite => ("fishing.key.hook", true),
        Phase::Flying | Phase::Floating | Phase::Nibble => ("fishing.key.hook", false),
    };
    [left, (if out { "fishing.key.cancel" } else { "" }, false)]
}

/// The standing line under the crosshair while the rod is out.
fn idle_text(line: &Line, rod: bool, roster: &[Species]) -> String {
    if !rod {
        return String::new();
    }
    if roster.is_empty() {
        return tr("fishing.status.nothing_lives").into();
    }
    match line.phase {
        // The buttons in the corner say these (`keys_for`).
        Phase::Ready | Phase::Charging | Phase::Flying | Phase::Floating => String::new(),
        Phase::Nibble => tr("fishing.status.nibble").into(),
        Phase::Bite => tr("fishing.status.bite").into(),
        Phase::Hooked => tr("fishing.status.hooked").into(),
    }
}

/// A tool's icon, as an asset path.
/// Bare hands' icon in the tool slot and the picker (`inventory-grid`
/// decision 8), drawn by `tools/gen_item_icons.py`.
pub const HAND_ICON: &str = "items/tools/hand.png";

pub fn tool_icon(tool: Tool) -> &'static str {
    match tool {
        Tool::Rod => "items/tools/rod.png",
        Tool::Shovel => "items/tools/shovel.png",
        Tool::Pickaxe => "items/tools/pickaxe.png",
        Tool::Axe => "items/tools/axe.png",
    }
}

/// A species' icon, as an asset path.
pub fn species_icon(species: &Species) -> String {
    format!("items/{}.png", species.icon)
}

/// The numbers a field-guide entry shows, each read off the species record the
/// simulation and the hook use, so the guide cannot disagree with the water.
pub fn guide_facts(
    species: &Species,
    fishing: &fauna::FishingSettings,
) -> Vec<(Localized, Localized)> {
    let water = species
        .water
        .iter()
        .map(|class| class.label().into_string())
        .collect::<Vec<_>>()
        .join(", ");
    let (lo, hi) = (
        format!("{:.0}", species.depth_m.0),
        format!("{:.0}", species.depth_m.1),
    );
    let depth = if species.bed {
        tr_with("fishing.fact.on_bed", &[("from", &lo), ("to", &hi)])
    } else {
        tr_with("fishing.fact.down", &[("from", &lo), ("to", &hi)])
    };
    let pips =
        "#".repeat(species.strength as usize) + &"-".repeat(5 - species.strength.min(5) as usize);
    vec![
        (tr("fishing.fact.water"), Localized::verbatim(water)),
        (
            tr("fishing.fact.temperature"),
            tr_with(
                "fishing.fact.temperature_range",
                &[
                    ("from", &format!("{:.0}", species.temp_c.0)),
                    ("to", &format!("{:.0}", species.temp_c.1)),
                ],
            ),
        ),
        (tr("fishing.fact.depth"), depth),
        (
            tr("fishing.fact.found"),
            tr_with(
                "fishing.fact.schools",
                &[("fewest", &species.school.0), ("most", &species.school.1)],
            ),
        ),
        (
            tr("fishing.fact.length"),
            tr_with(
                "fishing.fact.about_cm",
                &[("length", &format!("{:.0}", species.length_m * 100.0))],
            ),
        ),
        (
            tr("fishing.fact.strength"),
            Localized::verbatim(format!("{pips} {}/5", species.strength)),
        ),
        (
            tr("fishing.fact.hook_window"),
            tr_with(
                "fishing.fact.seconds",
                &[(
                    "seconds",
                    &format!("{:.2}", fishing.hook_window(species.strength)),
                )],
            ),
        ),
        (
            tr("fishing.fact.speed"),
            tr_with(
                "fishing.fact.speed_range",
                &[
                    ("from", &format!("{:.1}", species.speed_mps.0)),
                    ("to", &format!("{:.1}", species.speed_mps.1)),
                ],
            ),
        ),
    ]
}

/// The water class and temperature a school of this world would spawn in at
/// a place, for a capture or a log.
pub fn water_here(
    sea: &Sea,
    air: Option<&Air>,
    direction: Vec3,
    limits: &fauna::WaterLimits,
) -> (Option<WaterClass>, Option<f32>) {
    let depth = sea.depth_at(direction);
    (
        fauna::water_class(crate::planet::terrain_config(), direction, depth, limits),
        air.map(|air| air.now.sample(direction).temperature),
    )
}

pub mod bodies;
pub mod places;

/// A mesh's attribute buffers taken back to be laid into again
/// (`memory-leak`): emptied, their capacity kept, so a mesh rebuilt every
/// frame stops allocating once it has reached its size.
pub(crate) mod kept {
    use bevy::mesh::{Mesh, MeshVertexAttribute, VertexAttributeValues, triangle_normal};

    /// The attribute's buffer out of `mesh`, empty, or a new one if it has
    /// none of that type.
    pub fn float2(mesh: &mut Mesh, attribute: MeshVertexAttribute) -> Vec<[f32; 2]> {
        match mesh.remove_attribute(attribute) {
            Some(VertexAttributeValues::Float32x2(mut v)) => {
                v.clear();
                v
            }
            _ => Vec::new(),
        }
    }

    /// As [`float2`], three floats a vertex.
    pub fn float3(mesh: &mut Mesh, attribute: MeshVertexAttribute) -> Vec<[f32; 3]> {
        match mesh.remove_attribute(attribute) {
            Some(VertexAttributeValues::Float32x3(mut v)) => {
                v.clear();
                v
            }
            _ => Vec::new(),
        }
    }

    /// As [`float2`], four floats a vertex.
    pub fn float4(mesh: &mut Mesh, attribute: MeshVertexAttribute) -> Vec<[f32; 4]> {
        match mesh.remove_attribute(attribute) {
            Some(VertexAttributeValues::Float32x4(mut v)) => {
                v.clear();
                v
            }
            _ => Vec::new(),
        }
    }

    /// The flat normals of an unindexed triangle list, into `out`: what
    /// `Mesh::compute_flat_normals` works out, without a new buffer.
    pub fn flat_normals(positions: &[[f32; 3]], out: &mut Vec<[f32; 3]>) {
        out.clear();
        out.reserve(positions.len());
        for p in positions.chunks_exact(3) {
            let normal = triangle_normal(p[0], p[1], p[2]);
            out.extend([normal; 3]);
        }
    }
}

pub(crate) mod draw {
    //! The fish, the rod, the float and the line, drawn.
    use super::*;

    /// A school's drawn root: which school, and its mesh (on a child, so the
    /// root's [`crate::field_light::Faded`] reaches it).
    #[derive(Component)]
    pub struct SchoolMesh {
        pub index: usize,
        pub mesh: Handle<Mesh>,
    }

    #[derive(Component)]
    pub struct Float;

    #[derive(Component)]
    pub struct LineMesh;

    /// The bait on the hook, at the lure under the float (`fish-behaviour`):
    /// what the fish pecks at and takes.
    #[derive(Component)]
    pub struct Hook;

    /// Where the float is drawn (`movement-smoothness` design 7.1): the
    /// core's [`fauna::swim::DrawnFloat`], as a resource. Draw only.
    #[derive(Resource, Clone, Copy, Debug, Default, Deref, DerefMut)]
    pub struct DrawnFloat(pub fauna::swim::DrawnFloat);

    /// Every swimmer's drawn pose and the line's fish (`fish-behaviour`
    /// design 5): the core's [`fauna::swim::Drawing`], as a resource. Draw
    /// only, but for a bolting fish handed back to its school.
    #[derive(Resource, Clone, Debug, Default, Deref, DerefMut)]
    pub struct DrawnSwim(pub fauna::swim::Drawing);

    /// Ease the float's jumps (`movement-smoothness` design 7.1), after the
    /// line has stepped and before anything is drawn from it.
    pub fn ease_float(
        time: Res<Time>,
        fishery: Res<Fishery>,
        mut drawn: ResMut<DrawnFloat>,
        mut trail: ResMut<CastTrail>,
        space: crate::SimSpace,
    ) {
        let line = &fishery.line;
        let o = space.origin();
        trail.update(line.phase, line.float, time.delta_secs().min(0.1));
        drawn.update(
            &o,
            time.delta_secs().min(0.1),
            line.float,
            line.phase,
            line.run,
        );
    }

    /// How far the float flies between the places the cast's trail keeps,
    /// m (the mockup's).
    pub(crate) const TRAIL_STEP_M: f32 = 0.15;

    /// How long the line takes to ease from the cast's trail to its hanging
    /// curve once the float is down, s.
    pub(crate) const TRAIL_EASE_S: f32 = 0.6;

    /// The cast's line as drawn (`rod-motion` design 8): while the float
    /// flies the line lies along the path it took, from the rod tip back to
    /// where it was let go and out along its arc; once it is down the line
    /// eases to its hanging curve. Drawing only: the line's rule never reads
    /// it.
    #[derive(Resource, Clone, Debug, Default, PartialEq)]
    pub struct CastTrail {
        /// The float's places since it was let go, planet-local, m: the
        /// first where it left the tip, then one every `TRAIL_STEP_M`.
        pub points: Vec<Vec3>,
        flying: bool,
        /// How long the float has been down, s.
        landed_s: Option<f32>,
    }

    impl CastTrail {
        /// One frame: the line's phase and float after this frame's step.
        pub fn update(&mut self, phase: Phase, float: Vec3, dt: f32) {
            match phase {
                Phase::Flying => {
                    if !self.flying {
                        *self = Self {
                            points: vec![float],
                            flying: true,
                            landed_s: None,
                        };
                    } else if self
                        .points
                        .last()
                        .is_some_and(|last| last.distance(float) > TRAIL_STEP_M)
                    {
                        self.points.push(float);
                    }
                }
                Phase::Floating | Phase::Nibble | Phase::Bite if !self.points.is_empty() => {
                    if self.flying {
                        self.flying = false;
                        self.landed_s = Some(0.0);
                    } else if let Some(t) = self.landed_s.as_mut() {
                        *t += dt.max(0.0);
                    }
                }
                _ => *self = Self::default(),
            }
        }

        /// How much of the line lies along the trail: all of it while the
        /// float flies, easing to none over `TRAIL_EASE_S` once it is down.
        pub fn blend(&self) -> f32 {
            if self.points.is_empty() {
                return 0.0;
            }
            if self.flying {
                return 1.0;
            }
            match self.landed_s {
                Some(t) if t < TRAIL_EASE_S => {
                    let u = t / TRAIL_EASE_S;
                    1.0 - u * u * (3.0 - 2.0 * u)
                }
                _ => 0.0,
            }
        }

        /// The origin was rebased: the trail moves with it.
        pub fn rebase(&mut self, shift: Vec3) {
            for p in &mut self.points {
                *p -= shift;
            }
        }
    }

    /// `n` points evenly along the cast's path (the mockup's `trailPoints`):
    /// from the rod tip back to where the float was let go, along the
    /// float's places, to `end`, where the line is tied.
    pub(crate) fn trail_points(tip: Vec3, trail: &[Vec3], end: Vec3, n: usize) -> Vec<Vec3> {
        let path: Vec<Vec3> = std::iter::once(tip)
            .chain(trail.iter().copied())
            .chain(std::iter::once(end))
            .collect();
        let mut acc = vec![0.0f32];
        for k in 1..path.len() {
            acc.push(acc[k - 1] + path[k].distance(path[k - 1]));
        }
        let total = *acc.last().unwrap();
        let total = if total > 0.0 { total } else { 1.0 };
        (0..n)
            .map(|k| {
                let d = total * k as f32 / (n.max(2) - 1) as f32;
                let mut j = 1;
                while j < acc.len() - 1 && acc[j] < d {
                    j += 1;
                }
                let seg = acc[j] - acc[j - 1];
                let seg = if seg > 0.0 { seg } else { 1.0 };
                path[j - 1].lerp(path[j], (d - acc[j - 1]) / seg)
            })
            .collect()
    }

    #[derive(Component)]
    pub struct StatusText;

    #[derive(Component)]
    pub struct Meter;

    #[derive(Component)]
    pub struct MeterFill;

    /// How quickly a creature's drawn heading follows its step's, s: the
    /// time constant of its easing (`movement-smoothness` design 8.2), a
    /// little over two of the school's 30 Hz steps.
    pub(crate) const HEADING_EASE_S: f32 = 0.08;

    /// The fastest a creature's drawn heading turns, rad/s (540 degrees a
    /// second: right round in a third of a second), so even a step that
    /// reverses it is drawn as a turn.
    pub(crate) const HEADING_TURN_MAX: f32 = 3.0 * std::f32::consts::PI;

    /// A step slower than this, m/s, says nothing about which way a creature
    /// faces: it keeps the heading it was drawn with.
    const HEADING_MIN_MPS: f32 = 0.01;

    /// How each creature is drawn (`movement-smoothness` design 8): the
    /// heading it was drawn along last frame, eased toward its latest step's
    /// so a turn is spread over the frames between steps; the heading a fish
    /// at the lure took the lure with, which it keeps, level, while it is
    /// held; and this frame's forward and up as the mesh was built, which the
    /// motion trace reads. Draw only: nothing in the rules reads it.
    #[derive(Resource, Default, Debug)]
    pub struct DrawnHeadings {
        /// This frame's drawn forward and up, by school seed and id.
        pub map: HashMap<(u64, u32), Heading>,
        eased: HashMap<(u64, u32), Eased>,
        frame: u64,
    }

    /// One creature's drawn heading.
    #[derive(Clone, Copy, Debug)]
    pub struct Heading {
        pub forward: Vec3,
        pub up: Vec3,
        /// Where its mouth is drawn, local (`fish-behaviour`'s trace): the
        /// model's nose, laid rigid.
        pub mouth: Vec3,
    }

    #[derive(Clone, Copy, Debug)]
    struct Eased {
        forward: Vec3,
        /// Held at the lure: the level heading it took the lure with.
        held: Option<Vec3>,
        seen: u64,
    }

    impl DrawnHeadings {
        /// A new frame: what was drawn last frame is cleared from `map`.
        pub(crate) fn begin(&mut self) {
            self.frame += 1;
            self.map.clear();
        }

        /// Forget the creatures not drawn this frame.
        pub(crate) fn end(&mut self) {
            let frame = self.frame;
            self.eased.retain(|_, e| e.seen == frame);
        }

        /// The heading creature `key` is drawn along this frame, unit:
        /// `step` is the heading its latest step gives (any length), `up`
        /// the local up at it, `held` whether it is held at the lure.
        pub(crate) fn ease(
            &mut self,
            key: (u64, u32),
            step: Vec3,
            up: Vec3,
            held: bool,
            dt: f32,
        ) -> Vec3 {
            let level = |f: Vec3| (f - up * f.dot(up)).normalize_or(up.any_orthonormal_vector());
            let moving = step.length() >= HEADING_MIN_MPS && step.is_finite();
            let frame = self.frame;
            let Some(e) = self.eased.get_mut(&key) else {
                let f = if moving {
                    step.normalize()
                } else {
                    up.any_orthonormal_vector()
                };
                let f = if held { level(f) } else { f };
                self.eased.insert(
                    key,
                    Eased {
                        forward: f,
                        held: held.then_some(f),
                        seen: frame,
                    },
                );
                return f;
            };
            let last = e.forward;
            let target = if held {
                level(*e.held.get_or_insert_with(|| level(last)))
            } else {
                e.held = None;
                if moving { step.normalize() } else { last }
            };
            let (t, max) = if dt.is_finite() && dt > 0.0 {
                (1.0 - (-dt / HEADING_EASE_S).exp(), HEADING_TURN_MAX * dt)
            } else {
                (0.0, 0.0)
            };
            let f = turn_toward(last, target, up, t, max);
            e.forward = f;
            e.seen = frame;
            f
        }
    }

    /// `from` turned `t` (0 to 1) of the way to `to`, both unit, and never
    /// more than `max` radians; a reversal turns about `up`, never about an
    /// arbitrary axis.
    pub(crate) fn turn_toward(from: Vec3, to: Vec3, up: Vec3, t: f32, max: f32) -> Vec3 {
        let angle = from.angle_between(to);
        if !angle.is_finite() || angle < 1e-6 {
            return to;
        }
        let by = (angle * t.clamp(0.0, 1.0)).min(max.max(0.0));
        let axis = if from.dot(to) < -0.9999 {
            up
        } else {
            from.cross(to).normalize_or(up)
        };
        (Quat::from_axis_angle(axis, by) * from).normalize_or(to)
    }

    /// The float, the line and the HUD line; the rod is hung on the walking
    /// camera once there is one.
    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut images: ResMut<Assets<Image>>,
        fauna: Option<Res<FaunaConfig>>,
        rows: Option<ResMut<crate::field_light::FieldRows>>,
    ) {
        // The body-wave table the schools' vertex stage reads
        // (`fish-behaviour` design 8.4), from `fauna.ron`.
        if let (Some(fauna), Some(mut rows)) = (fauna, rows) {
            rows.set_swim_rows(&bodies::wave_table(&fauna.0.behaviour));
        }
        // The creatures' material, with one tiny creature-shaped mesh held
        // hidden from the start so the loading screen warms its pipeline
        // before play (no schools exist until the walker is in the water).
        // `PBD_PASS_OFF=life_texture` draws them as before the atlas, for a
        // before-and-after capture of the same frame (`ocean-environments`
        // B14).
        let untextured = std::env::var("PBD_PASS_OFF")
            .is_ok_and(|list| list.split(',').any(|p| p.trim() == "life_texture"));
        let mut desc = life_material_desc(images.add(life_atlas()));
        if untextured {
            desc.base_color_texture = None;
        }
        let life = materials.add(desc);
        let mut warm = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        warm.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[0.0, 0.0, 0.0], [0.01, 0.0, 0.0], [0.0, 0.01, 0.0]],
        );
        warm.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0f32; 4]; 3]);
        warm.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![bodies::white_uv(); 3]);
        warm.compute_flat_normals();
        // A school's mesh carries the body-wave attributes and is drawn
        // through the swim vertex stage: warmed too, so no school's first
        // frame compiles a pipeline after `READY`.
        let mut warm_school = warm.clone();
        warm_school.insert_attribute(bodies::ATTRIBUTE_SWIM_AXES, vec![[0.0f32; 4]; 3]);
        warm_school.insert_attribute(bodies::ATTRIBUTE_SWIM_WAVE, vec![[0.0f32; 4]; 3]);
        warm_school.insert_attribute(bodies::ATTRIBUTE_SWIM_UP, vec![[0.0f32; 4]; 3]);
        commands.spawn((
            Name::new("Water life (warm)"),
            crate::field_light::LitByField,
            Mesh3d(meshes.add(warm)),
            MeshMaterial3d(life.clone()),
            Transform::default(),
            Visibility::Hidden,
        ));
        commands.spawn((
            Name::new("Water life school (warm)"),
            crate::field_light::LitByField,
            crate::field_light::LitLikeTerrain,
            Transform::default(),
            Visibility::Hidden,
            children![(
                Mesh3d(meshes.add(warm_school)),
                MeshMaterial3d(life.clone()),
                Transform::default(),
                Visibility::Inherited,
            )],
        ));
        commands.insert_resource(LifeMaterial(life));
        let red = materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.2, 0.15),
            perceptual_roughness: 0.5,
            ..default()
        });
        let white = materials.add(StandardMaterial {
            base_color: Color::srgb(0.93, 0.93, 0.9),
            perceptual_roughness: 0.6,
            ..default()
        });
        commands
            .spawn((
                Name::new("Fishing float"),
                Float,
                crate::field_light::LitByField,
                Transform::default(),
                Visibility::Hidden,
            ))
            .with_children(|float| {
                float.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.06).mesh().uv(12, 8))),
                    MeshMaterial3d(red),
                    Transform::from_xyz(0.0, 0.03, 0.0),
                ));
                float.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.055).mesh().uv(12, 8))),
                    MeshMaterial3d(white),
                    Transform::from_xyz(0.0, -0.02, 0.0),
                ));
            });
        // The bait on the hook: a worm's end and the shank over it, the
        // mockup's (M37).
        let bait = materials.add(StandardMaterial {
            base_color: Color::srgb_u8(0xb0, 0x60, 0x5a),
            perceptual_roughness: 0.9,
            ..default()
        });
        let shank = materials.add(StandardMaterial {
            base_color: Color::srgb_u8(0x5a, 0x5f, 0x63),
            metallic: 0.6,
            perceptual_roughness: 0.4,
            ..default()
        });
        commands
            .spawn((
                Name::new("Fishing hook"),
                Hook,
                crate::field_light::LitByField,
                Transform::default(),
                Visibility::Hidden,
            ))
            .with_children(|hook| {
                hook.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.018, 0.035, 0.018))),
                    MeshMaterial3d(bait),
                    Transform::default(),
                ));
                hook.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.004, 0.03, 0.004))),
                    MeshMaterial3d(shank),
                    Transform::from_xyz(0.0, 0.02, 0.0),
                ));
            });
        let mut line = Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::default());
        line.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![Vec3::ZERO; LINE_POINTS + 1]);
        commands.spawn((
            Name::new("Fishing line"),
            LineMesh,
            Mesh3d(meshes.add(line)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.92, 0.95, 0.95),
                unlit: true,
                ..default()
            })),
            Transform::default(),
            Visibility::Hidden,
        ));
        commands
            .spawn(Node {
                position_type: PositionType::Absolute,
                bottom: px(55),
                left: percent(50),
                width: px(210),
                margin: UiRect::left(px(-105)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(3),
                ..default()
            })
            .with_children(|hud| {
                hud.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.83, 0.52)),
                    crate::ui_kit::text_shadow(),
                    StatusText,
                ));
                hud.spawn((
                    Node {
                        width: px(130),
                        height: px(4),
                        border: UiRect::all(px(1)),
                        display: Display::None,
                        ..default()
                    },
                    BorderColor::all(Color::srgba(0.55, 0.75, 0.74, 0.6)),
                    BackgroundColor(Color::srgba(0.02, 0.06, 0.08, 0.72)),
                    Meter,
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            width: percent(0),
                            height: percent(100),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.94, 0.71, 0.24)),
                        MeterFill,
                    ));
                });
            });
    }

    const LINE_POINTS: usize = 24;

    /// The one material every creature and the sea's places are drawn with:
    /// the water life's atlas (`ocean-environments` B14) times the colours in
    /// the vertices, both sides lit (a fin is one face).
    #[derive(Resource)]
    pub struct LifeMaterial(pub Handle<StandardMaterial>);

    /// The water life's atlas (`tools/water_life_atlas.py`): pixel art,
    /// sampled nearest with no mipmaps, decoded at startup so it is there
    /// before the first creature or reef tile is drawn.
    pub fn life_atlas() -> Image {
        let pixels = image::load_from_memory(include_bytes!(
            "../../../assets/textures/water_life/atlas.png"
        ))
        .expect("assets/textures/water_life/atlas.png")
        .to_rgba8();
        let mut image = Image::from_dynamic(
            image::DynamicImage::ImageRgba8(pixels),
            true,
            RenderAssetUsages::default(),
        );
        image.sampler = bevy::image::ImageSampler::nearest();
        image
    }

    pub fn life_material_desc(atlas: Handle<Image>) -> StandardMaterial {
        StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(atlas),
            perceptual_roughness: 0.85,
            reflectance: 0.3,
            cull_mode: None,
            double_sided: true,
            ..default()
        }
    }

    /// One mesh per school, rebuilt from its fish each frame, under a root
    /// that carries its fade (`ocean-environments` B18): made and dropped as
    /// schools come and go, and drawn through the terrain's screen-door mask
    /// as a school fades in on spawning and out before it is dropped.
    #[allow(clippy::too_many_arguments)]
    pub fn schools(
        mut commands: Commands,
        mut fishery: ResMut<Fishery>,
        fauna: Res<FaunaConfig>,
        body: Res<Body>,
        frame: Res<crate::planet::PlanetRenderFrame>,
        float: Res<DrawnFloat>,
        (mut headings, mut swim): (ResMut<DrawnHeadings>, ResMut<DrawnSwim>),
        time: Res<Time>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut drawn: Query<(
            Entity,
            &SchoolMesh,
            &mut Transform,
            &mut crate::field_light::Faded,
        )>,
        life: Res<LifeMaterial>,
        water: Option<Res<crate::config::WaterSettings>>,
        sea: Option<Res<Sea>>,
        air: Option<Res<Air>>,
        contact: Option<Res<PlanetContact>>,
        sun: Option<Res<crate::sky::Sun>>,
    ) {
        let roster = fauna.0.roster(&body.0);
        // The clock and the sun the creatures' poses read (7.2).
        let now = sun
            .as_deref()
            .map(|sun| (sun.clock.seconds, sun.direction()));
        // What the water does to a creature's colour at its depth
        // (`ocean-environments` B12, B23): the depth under the surface of
        // the water its school swims in, a lake's or a river's own top, the
        // sea's otherwise.
        let under = super::places::UnderWater::of(water.as_deref());
        let seconds = now.map_or(0.0, |(seconds, _)| seconds);
        let o = crate::sim_origin::from_render_frame(&frame);
        let alpha = fishery.step_alpha(fauna.0.spawn.step_hz);
        // The fish on the line is drawn at the lure under the drawn float
        // (`movement-smoothness` 7.1).
        let held = float.shift(&o, fishery.line.float);
        let dt = time.delta_secs().min(0.1);
        // Every swimmer turned and its wave stepped, and the line's fish
        // moved by its choreography (`fish-behaviour`): before the meshes
        // are built from them.
        {
            let lure = drawn_lure(&o, &fishery, &float);
            let fishery = &mut *fishery;
            let water = sea.as_deref().map(|sea| {
                PlanetWater::at(
                    sea,
                    air.as_deref(),
                    contact.as_deref(),
                    &fishery.beds,
                    o.query_point(fishery.line.float),
                    seconds,
                )
            });
            let view = match (fishery.line.is_out(), fishery.feet) {
                (true, Some(feet)) => Some(fauna::swim::LineView {
                    line: &fishery.line,
                    lure,
                    feet,
                }),
                _ => None,
            };
            swim.frame(
                &o,
                &mut fishery.schools,
                roster,
                &fauna.0.behaviour,
                view,
                alpha,
                dt,
                |school, i, species| {
                    let id = school.ids.get(i).copied().unwrap_or(0);
                    let body = bodies::model_for(&species.id, id);
                    let kind = body.wave_kind()?;
                    let length_m = school.length_cm(i, species) as f32 / 100.0;
                    let scale = length_m / body.length;
                    Some((
                        fauna::swim::Build {
                            length_m,
                            mouth_m: body.nose * scale,
                            speeds: species.speed_mps,
                        },
                        kind,
                    ))
                },
                |p| {
                    let Some(water) = water.as_ref() else {
                        return p;
                    };
                    let q = o.query_point(p);
                    let (top, bottom) = (
                        f64::from(water.surface(q) - 0.25),
                        f64::from(water.bed(q) + 0.25),
                    );
                    if top > bottom {
                        o.with_radius(p, o.radius_at(p).clamp(bottom, top))
                    } else {
                        p
                    }
                },
            );
        }
        let fishery = &*fishery;
        let dim = |school: &School| {
            sea.as_deref().map(|sea| {
                let anchor = o.query_point(school.anchor);
                let water = PlanetWater::at(
                    sea,
                    air.as_deref(),
                    contact.as_deref(),
                    &fishery.beds,
                    anchor,
                    seconds,
                );
                // `PBD_SCHOOL_DEPTH=sea`, a capture instrument (task 18.2):
                // the depth read off the sea's mean sphere, as before B23, so
                // a before frame can be taken beside the after.
                let surface = if school_depth_from_sea() {
                    sea.radius
                } else {
                    water.surface(anchor)
                };
                (surface, under)
            })
        };
        headings.begin();
        let mut seen = vec![false; fishery.schools.len()];
        for (entity, school_mesh, mut transform, mut faded) in &mut drawn {
            let Some(school) = fishery.schools.get(school_mesh.index) else {
                commands.entity(entity).despawn();
                continue;
            };
            let Some(species) = roster.get(school.species as usize) else {
                continue;
            };
            seen[school_mesh.index] = true;
            transform.translation =
                crate::render_origin::render_from_physics(&frame, school.anchor);
            let fade = fishery.fade_of(school);
            if faded.0 != fade {
                faded.0 = fade;
            }
            if let Some(mesh) = meshes.get_mut(&school_mesh.mesh) {
                build(
                    &o,
                    mesh,
                    school,
                    species,
                    now,
                    dim(school),
                    alpha,
                    held,
                    (&mut headings, &swim, dt),
                );
            }
        }
        for (index, school) in fishery.schools.iter().enumerate() {
            if seen[index] {
                continue;
            }
            let Some(species) = roster.get(school.species as usize) else {
                continue;
            };
            let material = life.0.clone();
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            );
            build(
                &o,
                &mut mesh,
                school,
                species,
                now,
                dim(school),
                alpha,
                held,
                (&mut headings, &swim, dt),
            );
            let handle = meshes.add(mesh);
            commands
                .spawn((
                    Name::new(format!("{} school", species.id)),
                    SchoolMesh {
                        index,
                        mesh: handle.clone(),
                    },
                    crate::field_light::LitByField,
                    // Lit as the seabed and the shore are, by the
                    // terrain's fill, sun and lamps (`ocean-environments`
                    // B23), not by Bevy's picture.
                    crate::field_light::LitLikeTerrain,
                    crate::field_light::Faded(fishery.fade_of(school)),
                    Transform::from_translation(crate::render_origin::render_from_physics(
                        &frame,
                        school.anchor,
                    )),
                    Visibility::default(),
                ))
                .with_children(|root| {
                    root.spawn((
                        Mesh3d(handle),
                        MeshMaterial3d(material),
                        Transform::default(),
                        Visibility::default(),
                    ));
                });
        }
        headings.end();
    }

    /// Where creature `i` of school `s` is drawn this frame, planet-local
    /// (before the school's own wobble and pose), between the school's last
    /// two steps: the point the mesh is built round, and the one the motion
    /// trace (`movement-smoothness`) follows.
    pub(crate) fn creature_drawn(
        o: &SimOrigin,
        fishery: &Fishery,
        float: &DrawnFloat,
        swim: Option<&DrawnSwim>,
        s: usize,
        i: usize,
        step_hz: f32,
    ) -> Option<Vec3> {
        let school = fishery.schools.get(s)?;
        // A swimmer is drawn at its pose (`fish-behaviour`).
        if let Some(pose) = swim.and_then(|w| w.poses.get(&(school.seed, *school.ids.get(i)?))) {
            return Some(pose.pos);
        }
        (i < school.len()).then(|| {
            drawn_at(
                school,
                i,
                fishery.step_alpha(step_hz),
                float.shift(o, fishery.line.float),
            )
        })
    }

    /// Creature `i` between its school's steps; the fish on the line, held
    /// at the lure, moved by `held` with the drawn float.
    fn drawn_at(school: &School, i: usize, alpha: f32, held: Vec3) -> Vec3 {
        let p = school.drawn(i, alpha);
        if school.held.is_some_and(|(h, _)| h == i) {
            p + held
        } else {
            p
        }
    }

    /// The lure as drawn this frame, local: a hand's width under the drawn
    /// float (`fish-behaviour`'s trace reads it).
    /// Before what the fish does to it: add [`fauna::swim::Drawing::lure_offset`].
    pub(crate) fn drawn_lure(o: &SimOrigin, fishery: &Fishery, float: &DrawnFloat) -> Vec3 {
        let drawn = float.drawn(o, fishery.line.float);
        drawn - o.up_at(drawn) * 0.35
    }

    /// How far under a diving duck is drawn when wholly dived, m.
    const DUCK_UNDER_M: f32 = 0.8;

    /// A school's creatures as one triangle list: each its own low-poly body
    /// (`bodies`, `ocean-environments` design B8), posed for what it is doing
    /// (7.2), swayed, scaled to its length and turned along its heading.
    ///
    /// A crawler stands on the bed and walks it (a crab sideways); a floater
    /// rides the sea (a seal lies out, and swims under when startled); a
    /// swimmer pitches with its heading. A shut clam's lid is down, an
    /// upended dabbler points its nose down, a sleeping duck tucks its head,
    /// a diver under the water is not drawn and a hidden octopus is small in
    /// its den.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build(
        o: &SimOrigin,
        mesh: &mut Mesh,
        school: &School,
        species: &Species,
        now: Option<(f64, Vec3)>,
        dim: Option<(f32, super::places::UnderWater)>,
        alpha: f32,
        held: Vec3,
        (headings, swim, dt): (&mut DrawnHeadings, &DrawnSwim, f32),
    ) {
        // Laid into the buffers the mesh already holds (`memory-leak`): a
        // school is laid every frame, and new buffers grown from empty each
        // time were most of the game's large allocations.
        let mut positions = kept::float3(mesh, Mesh::ATTRIBUTE_POSITION);
        let mut colours = kept::float4(mesh, Mesh::ATTRIBUTE_COLOR);
        let mut uvs = kept::float2(mesh, Mesh::ATTRIBUTE_UV_0);
        let mut waves = bodies::SwimAttributes {
            axes: kept::float4(mesh, bodies::ATTRIBUTE_SWIM_AXES),
            wave: kept::float4(mesh, bodies::ATTRIBUTE_SWIM_WAVE),
            up: kept::float4(mesh, bodies::ATTRIBUTE_SWIM_UP),
        };
        let mut normals = kept::float3(mesh, Mesh::ATTRIBUTE_NORMAL);
        let t = now.map_or(0.0, |(seconds, _)| seconds.rem_euclid(3600.0) as f32);
        for i in 0..school.len() {
            let id = school.ids.get(i).copied().unwrap_or(0);
            // A swimmer's pose (`fish-behaviour`): turned, banked, its body
            // wave; else between the school's last two steps
            // (`movement-smoothness`).
            let posed = swim.poses.get(&(school.seed, id));
            let p = &posed.map_or_else(|| drawn_at(school, i, alpha, held), |pose| pose.pos);
            let up = o.up_at(*p);
            let pose = now.map_or(fauna::Pose::Moving, |(seconds, sun)| {
                fauna::pose(species, school, i, seconds, sun.dot(up) < -0.05)
            });
            // How far under it has gone: a dive is drawn, never cut
            // (`ocean-environments` B18).
            let under = now.map_or(0.0, |(seconds, sun)| {
                fauna::under_share(species, school, i, seconds, sun.dot(up) < -0.05)
            });
            let body = bodies::model_for(&species.id, id);
            let phase = (fauna::hash2(school.seed, u64::from(id)) >> 40) as f32
                / (1u64 << 24) as f32
                * std::f32::consts::TAU;
            let v = school.velocities[i];
            let along = v - up * v.dot(up);
            // The heading its step gives (a swimmer pitches with it), eased
            // to over the frames between steps; a fish at the lure keeps the
            // level heading it took the lure with, never turned by the
            // lure's bob (`movement-smoothness` design 8.2).
            let step = match species.mover {
                fauna::Mover::School => v - up * v.dot(up) * 0.5,
                _ => along,
            };
            let held_here = school.held.is_some_and(|(h, _)| h == i);
            let length = school.length_cm(i, species) as f32 / 100.0;
            let mut scale = length / body.length;
            let mouth = body.nose * scale;
            let framed = posed.map(|pose| pose.frame(up, mouth));
            let eased = match framed {
                Some((forward, _, _)) => forward,
                None => headings.ease((school.seed, id), step, up, held_here, dt),
            };
            let flat = (eased - up * eased.dot(up)).normalize_or(up.any_orthonormal_vector());
            let mut forward = match species.mover {
                fauna::Mover::School => eased,
                _ => flat,
            };
            // A crab walks sideways.
            if body.kind == bodies::Kind::Crab {
                forward = up.cross(flat).normalize_or(flat);
            }
            let side = forward.cross(up).normalize_or(up.any_orthonormal_vector());
            let mut top = side.cross(forward);
            let at = *p - school.anchor;
            let mut origin = match species.mover {
                fauna::Mover::Crawler => at - up * fauna::crawl_lift(species),
                _ => at,
            };
            // Posed: banked, flashing, its head shaken about its mouth.
            if let Some((_, posed_top, posed_origin)) = framed {
                top = posed_top;
                origin = posed_origin - school.anchor;
            }
            match (body.kind, pose) {
                (bodies::Kind::Seal, _) => {
                    origin -= up * (0.1 * scale + (1.2 + 0.03 * scale) * under);
                }
                (bodies::Kind::Otter, _) => origin -= up * 0.03 * scale,
                (bodies::Kind::Bird, _) => {
                    origin -= up * (0.02 + DUCK_UNDER_M * under);
                    // Nose first as it goes.
                    let tip = Quat::from_axis_angle(side, -0.9 * under.min(1.0 - under) * 2.0);
                    forward = tip * forward;
                    top = tip * top;
                }
                (bodies::Kind::Octopus, fauna::Pose::Hidden) => scale *= 0.5,
                _ => {}
            }
            // An octopus takes the rock's colour when still and flushes when
            // it moves (decision 6).
            let mut tint = if body.kind == bodies::Kind::Octopus {
                let h = (along.length() / species.speed_mps.1.max(1e-3)).clamp(0.0, 1.0);
                [0.55 + 0.45 * h, 0.55 + 0.25 * h, 0.5 + 0.25 * h]
            } else {
                [1.0; 3]
            };
            if let Some((surface, water)) = dim {
                tint = water.albedo(tint, (f64::from(surface) - o.radius_at(*p)) as f32);
            }
            let look = bodies::Look { t, phase, pose };
            headings.map.insert(
                (school.seed, id),
                Heading {
                    forward,
                    up: top,
                    mouth: school.anchor + origin + forward * body.nose * scale,
                },
            );
            // Its body wave: the pose's; a swimmer drawn without one (a seal)
            // cruises on the world clock.
            let wave = match posed {
                Some(pose) => Some(pose.wave),
                None => body.wave_kind().map(|_| bodies::cruising(body, &look)),
            };
            let place = bodies::Place {
                origin,
                forward,
                up: top,
                scale,
            };
            bodies::emit_swimmer(
                body,
                &look,
                &place,
                tint,
                wave,
                &mut positions,
                &mut colours,
                &mut uvs,
                &mut waves,
            );
        }
        // A school with nobody drawn (all under, all gone) keeps one hidden
        // triangle, so its mesh keeps the layout the loading screen warmed
        // and never asks for a pipeline of its own.
        if positions.is_empty() {
            positions.extend([[0.0; 3]; 3]);
            colours.extend([[0.0; 4]; 3]);
            uvs.extend([bodies::white_uv(); 3]);
            waves.axes.extend([[0.0; 4]; 3]);
            waves.wave.extend([[0.0; 4]; 3]);
            waves.up.extend([[0.0; 4]; 3]);
        }
        // Flat normals of the rigid body: the vertex stage turns them with
        // the bend. `Mesh::compute_flat_normals`'s arithmetic, into the
        // buffer the mesh held.
        kept::flat_normals(&positions, &mut normals);
        mesh.remove_indices();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(bodies::ATTRIBUTE_SWIM_AXES, waves.axes);
        mesh.insert_attribute(bodies::ATTRIBUTE_SWIM_WAVE, waves.wave);
        mesh.insert_attribute(bodies::ATTRIBUTE_SWIM_UP, waves.up);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    }

    /// The float on the water and the line from the rod's tip to it, placed
    /// in `PostUpdate` after the camera has moved, so the rod, the line and
    /// the float are drawn from one frame's state.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn tackle(
        fishery: Res<Fishery>,
        tools: Res<ToolSlot>,
        walking: Option<Res<WalkingReadout>>,
        frame: Res<crate::planet::PlanetRenderFrame>,
        sea: Option<Res<Sea>>,
        air: Option<Res<Air>>,
        contact: Option<Res<PlanetContact>>,
        sun: Option<Res<crate::sky::Sun>>,
        tip_now: Option<Res<crate::held::RodTipNow>>,
        (drawn, swim, trail): (Res<DrawnFloat>, Res<DrawnSwim>, Res<CastTrail>),
        cameras: Query<&Transform, With<WalkingCamera>>,
        rods: Query<
            (&crate::held::Held, &Transform),
            (
                Without<WalkingCamera>,
                Without<Float>,
                Without<LineMesh>,
                Without<Hook>,
            ),
        >,
        mut floats: Query<
            (&mut Transform, &mut Visibility),
            (
                With<Float>,
                Without<LineMesh>,
                Without<WalkingCamera>,
                Without<Hook>,
            ),
        >,
        mut lines: Query<
            (&Mesh3d, &mut Transform, &mut Visibility),
            (
                With<LineMesh>,
                Without<Float>,
                Without<WalkingCamera>,
                Without<Hook>,
            ),
        >,
        mut hooks: Query<
            (&mut Transform, &mut Visibility),
            (
                With<Hook>,
                Without<Float>,
                Without<LineMesh>,
                Without<WalkingCamera>,
            ),
        >,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        let walking = walking.is_some_and(|w| w.active);
        let rod_out = walking && tools.held() == Some(Tool::Rod);
        let camera = cameras.iter().next();
        // The rod itself is the held tool's model (`held.rs`); this draws
        // what leaves it.
        let line = &fishery.line;
        let out = rod_out && line.is_out();
        // The float as drawn: the line's, its hooked jumps eased out
        // (`movement-smoothness` 7.1).
        let o = crate::sim_origin::from_render_frame(&frame);
        let float = drawn.drawn(&o, line.float);
        // The lure, where the fish has it (`fish-behaviour`: a peck's tug,
        // the take's drag), and the float leaning toward a lure pulled
        // aside and down after it. Drawn only.
        let up_here = o.up_at(float);
        let lure_off = swim.lure_offset();
        let lure = float - up_here * 0.35 + lure_off;
        let aside = lure_off - up_here * lure_off.dot(up_here);
        let float = float + aside * 0.45 - up_here * (-lure_off.dot(up_here)).max(0.0) * 0.3;
        let float_at = crate::render_origin::render_from_physics(&frame, float);
        for (mut transform, mut visibility) in &mut hooks {
            *visibility = if out {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            transform.translation = crate::render_origin::render_from_physics(&frame, lure);
            transform.rotation = Quat::from_rotation_arc(Vec3::Y, up_here);
        }
        for (mut transform, mut visibility) in &mut floats {
            *visibility = if out {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            transform.translation = float_at;
            transform.rotation = Quat::from_rotation_arc(Vec3::Y, o.up_at(float));
        }
        for (mesh, mut transform, mut visibility) in &mut lines {
            *visibility = if out {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            let Some(eye) = camera else {
                continue;
            };
            if !out {
                continue;
            }
            // The camera is a top-level entity: its `Transform`, which
            // `follow_walker` has just set, is where the rod is drawn this
            // frame. (Its `GlobalTransform` is still last frame's here.)
            // The tip is where the rod is drawn this frame (`held.rs` poses
            // it in `Update`, rod-motion), else the swayed rest tip
            // (`movement-smoothness` 7.3).
            let tip = tip_now.as_deref().map_or_else(
                || {
                    rods.iter()
                        .find(|(held, _)| held.tool() == Tool::Rod)
                        .map_or(crate::held::rod_tip(), |(held, pose)| {
                            held.posed(pose, crate::held::rod_tip())
                        })
                },
                |t| t.0,
            );
            // The line's shape is laid in local points (`world-frames`
            // 10.1), from the drawn rod tip to the drawn float; the water is
            // asked at planet query points.
            let tip_render = eye.transform_point(tip);
            let tip_local = crate::sim_origin::local_from_render(&frame, tip_render);
            transform.translation = tip_render;
            // With a fish on the line it is taut: nearly straight at the bite
            // and through the fight, a touch of sag only at the lowest
            // tension (owner, 2026-11-18: "I dont think the line in real life
            // would slack wheren theres a fish connected to it").
            let slack = match line.phase {
                Phase::Bite => 0.03,
                Phase::Hooked => 0.01 + 0.05 * (1.0 - line.tension.min(1.0)),
                _ => 0.6,
            };
            // A slack line is held just over the water rather than hanging
            // through it (the mockup's sag ran under the surface and looped
            // back up): the water the float rides, with its waves.
            // It ends on the top of the float, where it is tied.
            let end = float + o.up_at(float) * FLOAT_TOP_M;
            let end_point = o.query_point(end);
            // While the float flies the line follows its path, easing to the
            // hanging curve once it is down (`rod-motion` 8).
            let blend = trail.blend();
            let cast = (blend > 0.0).then_some((trail.points.as_slice(), blend));
            let points = match sea.as_deref() {
                Some(sea) => {
                    let seconds = sun.as_ref().map_or(0.0, |sun| sun.clock.seconds);
                    let water = PlanetWater::at(
                        sea,
                        air.as_deref(),
                        contact.as_deref(),
                        &fishery.beds,
                        end_point,
                        seconds,
                    );
                    line_points(&o, tip_local, end, slack, cast, |p| {
                        line_floor(&water, end_point, o.query_point(p))
                    })
                }
                None => line_points(&o, tip_local, end, slack, cast, |_| 0.0),
            };
            // On down through the float to the hook: the leader.
            let mut points = points;
            points.push(lure + up_here * 0.03 - tip_local);
            if let Some(mesh) = meshes.get_mut(&mesh.0) {
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, points);
            }
        }
    }

    /// Where the line is tied: the top of the float's red half, m over the
    /// float's own point (the red sphere's centre 0.03 m up, radius 0.06).
    pub(crate) const FLOAT_TOP_M: f32 = 0.09;

    /// How far over the water the slack of the line is held, m: enough that
    /// the drawn waves never cover it, so it reads white all the way to the
    /// float.
    pub(crate) const LINE_CLEARANCE_M: f32 = 0.05;

    /// How far round a point the drawn sea can take its height from, m. The
    /// sea is drawn as a heightfield sampled at the cells' corners
    /// (`water.wgsl`, `vertex`) and flat between them, so between corners it
    /// can stand over the true wave by up to the wave's height. A corner is
    /// never further than this from a point on the sea (a cell is about
    /// 2.8 m across).
    const DRAWN_REACH_M: f32 = 1.6;

    /// The highest the drawn sea can stand over a point: the most of the
    /// surface at the point and round it at `DRAWN_REACH_M`. The slack of the
    /// line is held over this, so the drawn waves never cover it.
    pub(crate) fn drawn_surface(water: &impl Water, point: Vec3) -> f32 {
        let up = point.normalize_or(Vec3::Y);
        let (a, b) = up.any_orthonormal_pair();
        (0..6)
            .map(|k| {
                let angle = k as f32 * std::f32::consts::TAU / 6.0;
                let off = (a * angle.cos() + b * angle.sin()) * DRAWN_REACH_M;
                water.surface(point + off)
            })
            .fold(water.surface(point), f32::max)
    }

    /// Over how much of its last stretch the line comes down from the wave
    /// crests onto the float, m across the water.
    const APPROACH_M: f32 = 0.5;

    /// The radius the slack of the line is held over at `point`, before
    /// `LINE_CLEARANCE_M`; `end` is where it is tied on the float.
    ///
    /// - Never under the drawn sea ([`drawn_surface`]).
    /// - Never under the waterline the frame's composite tints
    ///   (`water.wgsl` `compose`): the sea's mean surface, without the
    ///   waves. A line under it is drawn as seen through water, dark.
    /// - Over the crests: every wave at its crest. The eye
    ///   looks down on the line from the rod, so a line over every crest has
    ///   no wave between it and the eye. A line lying low on the water in
    ///   front of the float was seen through the crests nearer the eye, dark
    ///   and broken, and read as stopping short of the float.
    /// - The crest hold eases off over the last `APPROACH_M` so the line comes
    ///   down onto the float where it is tied.
    pub(crate) fn line_floor(water: &PlanetWater, end: Vec3, point: Vec3) -> f32 {
        let sea = water.sea;
        let under = drawn_surface(water, point).max(sea.radius);
        // Every wave at its crest.
        let crests = sea.radius + water.local.reach().max(0.0);
        let up = end.normalize_or(Vec3::Y);
        let rel = point - end;
        let across = (rel - up * rel.dot(up)).length();
        let t = (across / APPROACH_M).min(1.0);
        under + (crests - under).max(0.0) * t
    }

    /// The line's points, as offsets from the rod tip, from `tip` to
    /// `float` (both local, relative to `origin`; `float` is where the line
    /// is tied, the top of the float): a curve that sags by `slack` toward
    /// the planet, every point between the ends held `LINE_CLEARANCE_M` over
    /// `surface` (the radius of the water under a local point), lifted
    /// through the origin in f64. The first point is the tip
    /// and the last is the float, exactly: a float pulled under on a bite
    /// is still where the line ends (`fishing-and-equipment` design 4, the
    /// line's far end). With a cast's trail (its places, local, and how
    /// much of the line lies along it) each point between is moved that far
    /// toward the trail's (`rod-motion` 8).
    pub(crate) fn line_points(
        origin: &SimOrigin,
        tip: Vec3,
        float: Vec3,
        slack: f32,
        cast: Option<(&[Vec3], f32)>,
        surface: impl Fn(Vec3) -> f32,
    ) -> Vec<Vec3> {
        let down = -origin.up_at(tip);
        let last = LINE_POINTS - 1;
        let along =
            cast.map(|(trail, blend)| (trail_points(tip, trail, float, LINE_POINTS), blend));
        (0..LINE_POINTS)
            .map(|k| {
                if k == 0 {
                    return Vec3::ZERO;
                }
                if k == last {
                    return float - tip;
                }
                let u = k as f32 / last as f32;
                let mut offset = (float - tip) * u + down * slack * 4.0 * u * (1.0 - u);
                if let Some((trail, blend)) = &along {
                    offset = offset.lerp(trail[k] - tip, *blend);
                }
                let at = tip + offset;
                let r = origin.radius_at(at);
                let floor = f64::from(surface(at) + LINE_CLEARANCE_M);
                if r > 0.0 && r < floor {
                    origin.with_radius(at, floor) - tip
                } else {
                    offset
                }
            })
            .collect()
    }

    /// The HUD line under the crosshair, and the cast or tension meter.
    #[allow(clippy::type_complexity)]
    pub fn hud(
        status: Res<FishingStatus>,
        mut texts: Query<&mut Text, With<StatusText>>,
        mut meter: Query<&mut Node, (With<Meter>, Without<MeterFill>)>,
        mut fill: Query<(&mut Node, &mut BackgroundColor), (With<MeterFill>, Without<Meter>)>,
    ) {
        for mut text in &mut texts {
            if text.0 != status.text {
                text.0 = status.text.clone();
            }
        }
        for mut node in &mut meter {
            node.display = if status.meter.is_some() {
                Display::Flex
            } else {
                Display::None
            };
        }
        if let Some((label, value)) = status.meter {
            for (mut node, mut colour) in &mut fill {
                node.width = percent(value.clamp(0.0, 1.0) * 100.0);
                colour.0 = match label {
                    "tension" if value > 0.8 => Color::srgb(1.0, 0.42, 0.34),
                    "tension" if value > 0.55 => Color::srgb(0.94, 0.71, 0.24),
                    "tension" => Color::srgb(0.47, 0.88, 0.63),
                    _ => Color::srgb(0.94, 0.71, 0.24),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests;
