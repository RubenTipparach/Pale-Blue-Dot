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
use pbd_core::sea::LocalSea;
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
}

/// How long a school takes to fade in when spawned, or out before it is
/// dropped, s: the sea places' landing fade.
const SCHOOL_FADE_S: f32 = 0.8;

impl Fishery {
    /// How far a school has faded out, 0 whole to 1 gone; one not yet
    /// stepped is gone, so it fades in from its first frame.
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
        }
    }
}

/// What the rod is doing, for the HUD: a line of text and, while a fish is on
/// or a cast is charging, a meter.
#[derive(Resource, Default, Clone, Debug, PartialEq)]
pub struct FishingStatus {
    pub text: String,
    pub meter: Option<(&'static str, f32)>,
    /// The float is out and a right click reels it in: the interact prompt
    /// shows that button under the crosshair (owner, 2026-11-16: "just have a
    /// button prompt to right click to reel in").
    pub reel_prompt: bool,
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

impl Plugin for FishPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(places::SeaPlacesPlugin)
            .init_resource::<Fishery>()
            .add_message::<crate::missions::Did>()
            .init_resource::<FishingStatus>()
            .init_resource::<ToolSlot>()
            .init_resource::<Body>()
            .add_systems(Startup, draw::spawn)
            .add_systems(
                Update,
                (
                    life_script.run_if(resource_exists::<LifeScript>),
                    fish,
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
            .local(&state, direction, sea.depth_at(direction), seconds)
            .with_tide(sea.tide);
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

impl Water for PlanetWater<'_> {
    fn surface(&self, point: Vec3) -> f32 {
        self.fresh(point)
            .unwrap_or_else(|| self.sea.radius + self.local.height(point, self.sea.radius))
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
    let feet = match afloat {
        Some(craft) => Some(craft.exit().as_vec3()),
        None => world.walkers.iter().next().map(|p| {
            let at = (p.0.as_dvec3() - world.frame.center).as_vec3();
            at - at.normalize_or(Vec3::Y) * HALF_HEIGHT
        }),
    };
    let Some(feet) = feet else {
        return;
    };
    let up = feet.normalize_or(Vec3::Y);
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
                fishery.line.float,
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
                craft.eye().as_vec3(),
                t.forward().as_vec3(),
                t.right().as_vec3(),
                t.up().as_vec3(),
            )
        }),
        None => world.cameras.iter().next().map(|t| {
            let eye = (t.translation().as_dvec3() - world.frame.center).as_vec3();
            (
                eye,
                t.forward().as_vec3(),
                t.right().as_vec3(),
                t.up().as_vec3(),
            )
        }),
    };
    if let (Some((eye, look, right, cam_up)), true) = (camera, rod && walking) {
        let buttons = world.buttons.as_deref();
        let pressed = |b| listening && buttons.is_some_and(|i| i.just_pressed(b));
        let controls = Controls {
            pressed: pressed(MouseButton::Left),
            held: listening && buttons.is_some_and(|i| i.pressed(MouseButton::Left)),
            released: buttons.is_some_and(|i| i.just_released(MouseButton::Left)),
            reel_in: pressed(MouseButton::Right),
        };
        let angler = Angler {
            // The rod model's own tip (`held.rs`): eye space, -z ahead.
            tip: {
                let t = crate::held::rod_tip();
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
        let water = PlanetWater::at(sea, air, contact, &fishery.beds, around, seconds);
        let factor = air.map_or(1.0, |air| {
            let sample = air.now.sample(around.normalize_or(Vec3::Y));
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
        for event in events {
            // The rod's sounds (`audio-and-ambience` build decision 5).
            heard.write(crate::audio::hooks::FishHeard {
                event,
                at: fishery.line.float,
            });
            let float = fishery.line.float;
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
    status.reel_prompt =
        rod && walking && fishery.line.phase == Phase::Floating && !roster.is_empty();
    status.meter = match fishery.line.phase {
        Phase::Charging => Some(("CAST", fishery.line.power)),
        Phase::Hooked => Some(("TENSION", fishery.line.tension)),
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
            .then(|| format!("{}: {reason}", species.name))
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
            let water = PlanetWater::at(sea, air, contact, beds, school.anchor, seconds);
            school.notice(eye, species, SHY_HOLD_S);
            school.step(step, species, &settings.flock, &water, splash);
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
            try_spawn(fishery, feet, sea, air, contact, roster, settings, seconds);
        }
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

#[allow(clippy::too_many_arguments)]
fn life_script(
    mut script: ResMut<LifeScript>,
    mut fishery: ResMut<Fishery>,
    world: Surroundings,
    mut waited: Local<u32>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    life: Option<Res<draw::LifeMaterial>>,
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
        let mesh = lineup(eye, look, &water, seconds, above);
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
        if !species.fits(sea.depth_at(direction)) {
            continue;
        }
        let anchor = direction * sea.radius;
        let water = PlanetWater::at(
            sea,
            world.air.as_deref(),
            world.contact.as_deref(),
            &fishery.beds,
            anchor,
            seconds,
        );
        let school = School::spawn(index as u16, species, anchor, 0x11fe, &water);
        info!(
            "life script: {} x{} put {metres} m ahead",
            species.id,
            school.len()
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
        emit(
            body,
            &look,
            &place,
            [1.0; 3],
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
    let up = feet.normalize_or(Vec3::Y);
    let east = Vec3::Y.cross(up).normalize_or(Vec3::X);
    let north = up.cross(east);
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
    let mut school = School::spawn(pick as u16, &roster[pick], anchor, seed, &water);
    // Smaller water, smaller fish (`fishing-anywhere` design 3).
    if let (Some(top), Some(contact)) = (fresh, contact) {
        school.scale = body_scale(direction, top - floor, |p| {
            let floor = crate::vehicles::ground_under(Some(contact), p.as_dvec3()) as f32;
            fresh_top(contact, p.normalize_or(Vec3::Y), floor).is_some()
        });
    }
    debug!(
        "{} school of {} spawned in {} at {:.1} C, {:.0} m out",
        roster[pick].name,
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
    float: Vec3,
    baited: bool,
) -> Vec<crate::missions::Did> {
    use pbd_core::missions::Kind;
    let name = |species: u16| {
        roster
            .get(species as usize)
            .map_or("fish", |sp| sp.name.as_str())
            .to_string()
    };
    match event {
        Event::Dry => status.say("The float came down on dry ground", 1.5),
        Event::Bite { .. } => status.say("Bite! Click now", 0.3),
        Event::TooEarly { species } => status.say(
            format!("Too early - the {} bolted", name(species).to_lowercase()),
            2.0,
        ),
        Event::Missed { species } => status.say(
            format!(
                "Missed - the {} spat the hook",
                name(species).to_lowercase()
            ),
            2.0,
        ),
        Event::Lost { species } => status.say(
            format!("Wound in - the {} got away", name(species).to_lowercase()),
            2.0,
        ),
        Event::Snapped { species } => status.say(
            format!(
                "The line snapped - the {} is gone",
                name(species).to_lowercase()
            ),
            2.5,
        ),
        Event::Caught { species, length_cm } => {
            status.say(format!("{} - {length_cm} cm", name(species)), 3.0);
            // A baited line's catch uses one bait, taken in the catch's own
            // durable write (`starter-missions` G1); given back if the fish
            // is let go.
            let bait = Item::Good(Good::BAIT);
            let mut hotbar = hotbar;
            let used_bait = baited && hotbar.as_deref_mut().is_some_and(|h| h.take_item(bait, 1));
            if land(species, length_cm, hotbar.as_deref_mut(), save, status) {
                use crate::missions::Did;
                let kind = name(species).to_lowercase();
                let mut did = vec![Did::new(Kind::Catch, kind.clone(), 1)];
                if used_bait {
                    did.push(Did::new(Kind::Baited, kind.clone(), 1));
                }
                // Each kind, counted once a mission ("Three kinds of fish",
                // G4).
                did.push(Did::new(Kind::Species, kind, 1));
                // The sea or fresh water: water standing above the sea's
                // level is a river or a lake ("Off the shore").
                let fresh = float.length() - crate::planet::PLANET_RADIUS > 0.6;
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
        status.say("No room in the hotbar - the fish was let go", 3.0);
        return false;
    }
    if let Some(save) = save
        && !save.record_catch(species, length_cm, hotbar)
    {
        warn!("catch not saved: the save writer has failed");
    }
    true
}

/// The standing line under the crosshair while the rod is out.
fn idle_text(line: &Line, rod: bool, roster: &[Species]) -> String {
    if !rod {
        return String::new();
    }
    if roster.is_empty() {
        return "Nothing lives in this water".into();
    }
    match line.phase {
        Phase::Ready => "Hold the left button to cast".into(),
        Phase::Charging => "Let go to cast".into(),
        Phase::Flying => String::new(),
        // The button prompt says it (`reel_prompt`).
        Phase::Floating => String::new(),
        Phase::Nibble => "Something is nibbling - wait for the float to go under".into(),
        Phase::Bite => "Bite! Click now".into(),
        Phase::Hooked => "Hold to reel - ease off when it runs".into(),
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
) -> Vec<(&'static str, String)> {
    let water = species
        .water
        .iter()
        .map(|class| class.name())
        .collect::<Vec<_>>()
        .join(", ");
    let depth = if species.bed {
        format!(
            "on the bed, {:.0}-{:.0} m",
            species.depth_m.0, species.depth_m.1
        )
    } else {
        format!("{:.0}-{:.0} m down", species.depth_m.0, species.depth_m.1)
    };
    let pips =
        "#".repeat(species.strength as usize) + &"-".repeat(5 - species.strength.min(5) as usize);
    vec![
        ("WATER", water),
        (
            "TEMPERATURE",
            format!("{:.0} to {:.0} C", species.temp_c.0, species.temp_c.1),
        ),
        ("DEPTH", depth),
        (
            "FOUND",
            format!("schools of {}-{}", species.school.0, species.school.1),
        ),
        (
            "LENGTH",
            format!("about {:.0} cm", species.length_m * 100.0),
        ),
        ("STRENGTH", format!("{pips} {}/5", species.strength)),
        (
            "HOOK WINDOW",
            format!("{:.2} s", fishing.hook_window(species.strength)),
        ),
        (
            "SPEED",
            format!("{:.1}-{:.1} m/s", species.speed_mps.0, species.speed_mps.1),
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

    #[derive(Component)]
    pub struct StatusText;

    #[derive(Component)]
    pub struct Meter;

    #[derive(Component)]
    pub struct MeterFill;

    /// The float, the line and the HUD line; the rod is hung on the walking
    /// camera once there is one.
    pub fn spawn(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut images: ResMut<Assets<Image>>,
    ) {
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
        commands.spawn((
            Name::new("Water life (warm)"),
            crate::field_light::LitByField,
            Mesh3d(meshes.add(warm)),
            MeshMaterial3d(life.clone()),
            Transform::default(),
            Visibility::Hidden,
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
        let mut line = Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::default());
        line.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![Vec3::ZERO; LINE_POINTS]);
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
        fishery: Res<Fishery>,
        fauna: Res<FaunaConfig>,
        body: Res<Body>,
        frame: Res<crate::planet::PlanetRenderFrame>,
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
        // sea's tided one otherwise, never the mean sea sphere.
        let under = super::places::UnderWater::of(water.as_deref());
        let seconds = now.map_or(0.0, |(seconds, _)| seconds);
        let dim = |school: &School| {
            sea.as_deref().map(|sea| {
                let water = PlanetWater::at(
                    sea,
                    air.as_deref(),
                    contact.as_deref(),
                    &fishery.beds,
                    school.anchor,
                    seconds,
                );
                (water.surface(school.anchor), under)
            })
        };
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
            transform.translation = (frame.center + school.anchor.as_dvec3()).as_vec3();
            let fade = fishery.fade_of(school);
            if faded.0 != fade {
                faded.0 = fade;
            }
            if let Some(mesh) = meshes.get_mut(&school_mesh.mesh) {
                build(mesh, school, species, now, dim(school));
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
            build(&mut mesh, school, species, now, dim(school));
            let handle = meshes.add(mesh);
            commands
                .spawn((
                    Name::new(format!("{} school", species.name)),
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
                    Transform::from_translation(
                        (frame.center + school.anchor.as_dvec3()).as_vec3(),
                    ),
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
    fn build(
        mesh: &mut Mesh,
        school: &School,
        species: &Species,
        now: Option<(f64, Vec3)>,
        dim: Option<(f32, super::places::UnderWater)>,
    ) {
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut colours: Vec<[f32; 4]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let t = now.map_or(0.0, |(seconds, _)| seconds.rem_euclid(3600.0) as f32);
        for (i, p) in school.positions.iter().enumerate() {
            let up = p.normalize_or(Vec3::Y);
            let pose = now.map_or(fauna::Pose::Moving, |(seconds, sun)| {
                fauna::pose(species, school, i, seconds, sun.dot(up) < -0.05)
            });
            // How far under it has gone: a dive is drawn, never cut
            // (`ocean-environments` B18).
            let under = now.map_or(0.0, |(seconds, sun)| {
                fauna::under_share(species, school, i, seconds, sun.dot(up) < -0.05)
            });
            let id = school.ids.get(i).copied().unwrap_or(0);
            let body = bodies::model_for(&species.id, id);
            let phase = (fauna::hash2(school.seed, u64::from(id)) >> 40) as f32
                / (1u64 << 24) as f32
                * std::f32::consts::TAU;
            let v = school.velocities[i];
            let along = v - up * v.dot(up);
            let flat = along.normalize_or(up.any_orthonormal_vector());
            let mut forward = match species.mover {
                fauna::Mover::School => {
                    (v - up * v.dot(up) * 0.5).normalize_or(up.any_orthonormal_vector())
                }
                _ => flat,
            };
            // A crab walks sideways.
            if body.kind == bodies::Kind::Crab {
                forward = up.cross(flat).normalize_or(flat);
            }
            let side = forward.cross(up).normalize_or(up.any_orthonormal_vector());
            let mut top = side.cross(forward);
            let length = school.length_cm(i, species) as f32 / 100.0;
            let mut scale = length / body.length;
            let at = *p - school.anchor;
            let mut origin = match species.mover {
                fauna::Mover::Crawler => at - up * fauna::crawl_lift(species),
                _ => at,
            };
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
                tint = water.albedo(tint, surface - p.length());
            }
            let look = bodies::Look { t, phase, pose };
            let place = bodies::Place {
                origin,
                forward,
                up: top,
                scale,
            };
            bodies::emit(
                body,
                &look,
                &place,
                tint,
                &mut positions,
                &mut colours,
                &mut uvs,
            );
        }
        // A school with nobody drawn (all under, all gone) keeps one hidden
        // triangle, so its mesh keeps the layout the loading screen warmed
        // and never asks for a pipeline of its own.
        if positions.is_empty() {
            positions = vec![[0.0; 3]; 3];
            colours = vec![[0.0; 4]; 3];
            uvs = vec![bodies::white_uv(); 3];
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.remove_indices();
        mesh.compute_flat_normals();
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
        cameras: Query<&Transform, With<WalkingCamera>>,
        mut floats: Query<
            (&mut Transform, &mut Visibility),
            (With<Float>, Without<LineMesh>, Without<WalkingCamera>),
        >,
        mut lines: Query<
            (&Mesh3d, &mut Transform, &mut Visibility),
            (With<LineMesh>, Without<Float>, Without<WalkingCamera>),
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
        let float_at = (frame.center + line.float.as_dvec3()).as_vec3();
        for (mut transform, mut visibility) in &mut floats {
            *visibility = if out {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            transform.translation = float_at;
            transform.rotation = Quat::from_rotation_arc(Vec3::Y, line.float.normalize_or(Vec3::Y));
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
            let tip_local =
                (eye.transform_point(crate::held::rod_tip()).as_dvec3() - frame.center).as_vec3();
            transform.translation = (frame.center + tip_local.as_dvec3()).as_vec3();
            let slack = match line.phase {
                Phase::Hooked => 0.1 + 0.5 * (1.0 - line.tension.min(1.0)),
                _ => 0.6,
            };
            // A slack line is held just over the water rather than hanging
            // through it (the mockup's sag ran under the surface and looped
            // back up): the water the float rides, with its waves and tide,
            // not the mean sea, which stands a metre over it at low water.
            // It ends on the top of the float, where it is tied.
            let end = line.float + line.float.normalize_or(Vec3::Y) * FLOAT_TOP_M;
            let points = match sea.as_deref() {
                Some(sea) => {
                    let seconds = sun.as_ref().map_or(0.0, |sun| sun.clock.seconds);
                    let water = PlanetWater::at(
                        sea,
                        air.as_deref(),
                        contact.as_deref(),
                        &fishery.beds,
                        line.float,
                        seconds,
                    );
                    line_points(tip_local, end, slack, |p| line_floor(&water, end, p))
                }
                None => line_points(tip_local, end, slack, |_| 0.0),
            };
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
    ///   (`water.wgsl` `compose`): the mean sea lifted by the tide, without
    ///   the waves. A line under it is drawn as seen through water, dark.
    /// - Over the crests: every wave at its crest, on the tide here. The eye
    ///   looks down on the line from the rod, so a line over every crest has
    ///   no wave between it and the eye. A line lying low on the water in
    ///   front of the float was seen through the crests nearer the eye, dark
    ///   and broken, and read as stopping short of the float.
    /// - The crest hold eases off over the last `APPROACH_M` so the line comes
    ///   down onto the float where it is tied.
    pub(crate) fn line_floor(water: &PlanetWater, end: Vec3, point: Vec3) -> f32 {
        let sea = water.sea;
        let under = drawn_surface(water, point).max(sea.radius + sea.tide_at(point));
        // Every wave at its crest: the table's reach less the tide's highest,
        // which it adds; the tide here instead.
        let waves = water.local.reach() - pbd_core::sea::tide_high_m(sea.tide.w.max(0.0));
        let crests = sea.radius + waves.max(0.0) + sea.tide_at(point);
        let up = end.normalize_or(Vec3::Y);
        let rel = point - end;
        let across = (rel - up * rel.dot(up)).length();
        let t = (across / APPROACH_M).min(1.0);
        under + (crests - under).max(0.0) * t
    }

    /// The line's points, as offsets from the rod tip, from `tip` to
    /// `float` (both planet-local; `float` is where the line is tied, the
    /// top of the float): a curve that sags by `slack` toward the planet,
    /// every point between the ends held `LINE_CLEARANCE_M` over `surface`
    /// (the radius of the water under a point). The first point is the tip
    /// and the last is the float, exactly: a float pulled under on a bite
    /// is still where the line ends (`fishing-and-equipment` design 4, the
    /// line's far end).
    pub(crate) fn line_points(
        tip: Vec3,
        float: Vec3,
        slack: f32,
        surface: impl Fn(Vec3) -> f32,
    ) -> Vec<Vec3> {
        let down = -tip.normalize_or(Vec3::Y);
        let last = LINE_POINTS - 1;
        (0..LINE_POINTS)
            .map(|k| {
                if k == 0 {
                    return Vec3::ZERO;
                }
                if k == last {
                    return float - tip;
                }
                let u = k as f32 / last as f32;
                let offset = (float - tip) * u + down * slack * 4.0 * u * (1.0 - u);
                let at = tip + offset;
                let r = at.length();
                let floor = surface(at) + LINE_CLEARANCE_M;
                if r > 0.0 && r < floor {
                    at * (floor / r) - tip
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
                    "TENSION" if value > 0.8 => Color::srgb(1.0, 0.42, 0.34),
                    "TENSION" if value > 0.55 => Color::srgb(0.94, 0.71, 0.24),
                    "TENSION" => Color::srgb(0.47, 0.88, 0.63),
                    _ => Color::srgb(0.94, 0.71, 0.24),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests;
