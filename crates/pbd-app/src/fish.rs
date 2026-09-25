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
use pbd_core::inventory::{Equipment, Item, Tool};
use pbd_core::sea::LocalSea;
use std::collections::HashMap;
use std::sync::Mutex;

/// The tool slot, as a resource: the core's `Equipment`, which the desktop's
/// picker changes and the save records.
#[derive(Resource, Default, Deref, DerefMut, Clone, Copy, Debug)]
pub struct ToolSlot(pub Equipment);

impl ToolSlot {
    /// The tool slot a world opens with: the save's, or the new world's kit.
    pub fn restore(save: &WorldSave) -> Self {
        Self(save.equipment.unwrap_or_default())
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
        }
    }
}

/// What the rod is doing, for the HUD: a line of text and, while a fish is on
/// or a cast is charging, a meter.
#[derive(Resource, Default, Clone, Debug, PartialEq)]
pub struct FishingStatus {
    pub text: String,
    pub meter: Option<(&'static str, f32)>,
    /// Seconds a one-off message (a catch, a snap) stays up.
    hold_s: f32,
}

impl FishingStatus {
    fn say(&mut self, text: impl Into<String>, hold_s: f32) {
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
        app.init_resource::<Fishery>()
            .init_resource::<FishingStatus>()
            .init_resource::<ToolSlot>()
            .init_resource::<Body>()
            .add_systems(Startup, draw::spawn)
            .add_systems(
                Update,
                (fish, draw::schools, draw::tackle).chain().in_set(FishSet),
            );
    }
}

/// The water as a school or the line sees it: the sea's surface from one
/// local wave table, and the bed from the craft's ground rule, remembered.
struct PlanetWater<'a> {
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

impl Water for PlanetWater<'_> {
    fn surface(&self, point: Vec3) -> f32 {
        self.sea.radius + self.local.height(point, self.sea.radius)
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
}

/// One frame of fishing: the line, the schools, and spawning round the
/// player.
fn fish(
    world: Surroundings,
    tools: Res<ToolSlot>,
    mut fishery: ResMut<Fishery>,
    mut status: ResMut<FishingStatus>,
    mut hotbar: Option<ResMut<Hotbar>>,
    mut save: Option<ResMut<WorldSave>>,
) {
    let Some(sea) = world.sea.as_deref() else {
        return;
    };
    let dt = world.time.delta_secs().min(0.1);
    status.hold_s = (status.hold_s - dt).max(0.0);
    let Some(feet) = world
        .walkers
        .iter()
        .next()
        .map(|p| (p.0.as_dvec3() - world.frame.center).as_vec3())
    else {
        return;
    };
    let up = feet.normalize_or(Vec3::Y);
    let feet = feet - up * HALF_HEIGHT;
    let roster = world.fauna.0.roster(&world.body.0);
    let settings = &world.fauna.0;
    let seconds = world.sun.as_ref().map_or(0.0, |sun| sun.clock.seconds);
    let air = world.air.as_deref();
    let contact = world.contact.as_deref();
    let fishery = &mut *fishery;

    // The line. Only the rod fishes, only on foot, only with the pointer
    // held and no menu or picker open; put the rod away and the line comes in.
    let walking = world.walking.as_ref().is_some_and(|w| w.active);
    let rod = tools.held() == Tool::Rod;
    if (!rod || !walking) && fishery.line.phase != Phase::Ready {
        let flock = settings.flock;
        fishery.line.wind_in(&mut fishery.schools, &flock);
    }
    let listening = walking
        && rod
        && world.walking.as_ref().is_some_and(|w| w.captured)
        && !world.menu.as_ref().is_some_and(|m| m.0)
        && !world.key.as_ref().is_some_and(|k| k.holding);
    let camera = world.cameras.iter().next().map(|t| {
        let eye = (t.translation().as_dvec3() - world.frame.center).as_vec3();
        (
            eye,
            t.forward().as_vec3(),
            t.right().as_vec3(),
            t.up().as_vec3(),
        )
    });
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
            tip: eye + right * ROD_TIP.x + cam_up * ROD_TIP.y - look * ROD_TIP.z,
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
        let events = fishery.line.step(
            dt,
            controls,
            &angler,
            &mut fishery.schools,
            roster,
            &settings.fishing,
            &settings.flock,
            &water,
            crate::sea::surface_gravity(),
            factor,
        );
        for event in events {
            report(
                &mut status,
                event,
                roster,
                hotbar.as_deref_mut(),
                save.as_deref_mut(),
            );
        }
    }
    if status.hold_s <= 0.0 {
        status.text = idle_text(&fishery.line, rod && walking, roster);
    }
    status.meter = match fishery.line.phase {
        Phase::Charging => Some(("CAST", fishery.line.power)),
        Phase::Hooked => Some(("TENSION", fishery.line.tension)),
        _ => None,
    };

    // The schools, on their own clock.
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
            school.step(step, species, &settings.flock, &water, splash);
        }
    }

    // Spawning and dropping round the player.
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

/// Where the rod tip is off the eye, m: right, up, and forward (Tenebris's
/// `client fishing.rs:85-91`).
const ROD_TIP: Vec3 = Vec3::new(0.26, 0.05, -0.95);
/// And the grip.
const ROD_GRIP: Vec3 = Vec3::new(0.18, -0.30, -0.22);

/// Drop the schools that have gone too far or been fished out, keeping the
/// line's target pointing at the same school.
fn drop_far(fishery: &mut Fishery, feet: Vec3, despawn_m: f32) {
    let target = fishery.line.target.map(|t| t.school);
    let mut index = 0;
    let mut remap = Vec::with_capacity(fishery.schools.len());
    fishery.schools.retain(|school| {
        let keep = Some(remap.len()) == target
            || (!school.is_empty() && school.centre().distance(feet) <= despawn_m);
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
    if roster.is_empty() || fishery.schools.len() >= settings.spawn.max_schools as usize {
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
    let depth = sea.depth_at(direction);
    let Some(class) =
        fauna::water_class(&crate::planet::TERRAIN, direction, depth, &settings.water)
    else {
        return;
    };
    let temperature = air.now.sample(direction).temperature;
    let candidates: Vec<usize> = roster
        .iter()
        .enumerate()
        .filter(|(i, sp)| {
            sp.lives_in(class, temperature, &settings.water)
                && sp.fits(depth)
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
    let anchor = direction * sea.radius;
    let water = PlanetWater::at(sea, Some(air), contact, &fishery.beds, anchor, seconds);
    let school = School::spawn(pick as u16, &roster[pick], anchor, seed, &water);
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

/// What a line event means to the player, and what a catch does to the
/// hotbar and the save.
fn report(
    status: &mut FishingStatus,
    event: Event,
    roster: &[Species],
    hotbar: Option<&mut Hotbar>,
    save: Option<&mut WorldSave>,
) {
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
        Event::Snapped { species } => status.say(
            format!(
                "The line snapped - the {} is gone",
                name(species).to_lowercase()
            ),
            2.5,
        ),
        Event::Caught { species, length_cm } => {
            status.say(format!("{} - {length_cm} cm", name(species)), 3.0);
            land(species, length_cm, hotbar, save, status);
        }
        _ => {}
    }
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
        Phase::Floating => "Watch the float - right click winds in".into(),
        Phase::Nibble => "Something is nibbling - wait for the float to go under".into(),
        Phase::Bite => "Bite! Click now".into(),
        Phase::Hooked => "Hold to reel - ease off when it runs".into(),
    }
}

/// A tool's icon, as an asset path.
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
        fauna::water_class(&crate::planet::TERRAIN, direction, depth, limits),
        air.map(|air| air.now.sample(direction).temperature),
    )
}

mod draw {
    //! The fish, the rod, the float and the line, drawn.
    use super::*;

    #[derive(Component)]
    pub struct SchoolMesh(pub usize);

    #[derive(Component)]
    pub struct Rod;

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
    ) {
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
                bottom: px(110),
                left: percent(50),
                width: px(420),
                margin: UiRect::left(px(-210)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(5),
                ..default()
            })
            .with_children(|hud| {
                hud.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.83, 0.52)),
                    TextShadow::default(),
                    StatusText,
                ));
                hud.spawn((
                    Node {
                        width: px(260),
                        height: px(8),
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

    /// One mesh per school, rebuilt from its fish each frame; entities made
    /// and dropped as schools come and go.
    #[allow(clippy::too_many_arguments)]
    pub fn schools(
        mut commands: Commands,
        fishery: Res<Fishery>,
        fauna: Res<FaunaConfig>,
        body: Res<Body>,
        frame: Res<crate::planet::PlanetRenderFrame>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut drawn: Query<(Entity, &SchoolMesh, &Mesh3d, &mut Transform)>,
        mut palette: Local<HashMap<u16, Handle<StandardMaterial>>>,
    ) {
        let roster = fauna.0.roster(&body.0);
        let mut seen = vec![false; fishery.schools.len()];
        for (entity, school_mesh, mesh, mut transform) in &mut drawn {
            let Some(school) = fishery.schools.get(school_mesh.0) else {
                commands.entity(entity).despawn();
                continue;
            };
            let Some(species) = roster.get(school.species as usize) else {
                continue;
            };
            seen[school_mesh.0] = true;
            transform.translation = (frame.center + school.anchor.as_dvec3()).as_vec3();
            if let Some(mesh) = meshes.get_mut(&mesh.0) {
                build(mesh, school, species);
            }
        }
        for (index, school) in fishery.schools.iter().enumerate() {
            if seen[index] {
                continue;
            }
            let Some(species) = roster.get(school.species as usize) else {
                continue;
            };
            let material = palette
                .entry(school.species)
                .or_insert_with(|| {
                    let [r, g, b] = species.colour;
                    materials.add(StandardMaterial {
                        base_color: Color::srgb(r, g, b),
                        perceptual_roughness: 0.45,
                        reflectance: 0.6,
                        cull_mode: None,
                        ..default()
                    })
                })
                .clone();
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            );
            build(&mut mesh, school, species);
            commands.spawn((
                Name::new(format!("{} school", species.name)),
                SchoolMesh(index),
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material),
                Transform::from_translation((frame.center + school.anchor.as_dvec3()).as_vec3()),
                Visibility::default(),
            ));
        }
    }

    /// A school's fish as one triangle list: each a five-sided cone nosed
    /// along its own velocity with a tail fin, scaled by the species' shape.
    fn build(mesh: &mut Mesh, school: &School, species: &Species) {
        const SIDES: usize = 5;
        let mut positions = Vec::with_capacity(school.len() * SIDES * 9);
        let [sx, sy, sz] = species.shape;
        let length = species.length_m;
        for (i, p) in school.positions.iter().enumerate() {
            let up = p.normalize_or(Vec3::Y);
            let v = school.velocities[i];
            let forward = (v - up * v.dot(up) * 0.5).normalize_or(up.any_orthonormal_vector());
            let side = forward.cross(up).normalize_or(up.any_orthonormal_vector());
            let top = side.cross(forward);
            let at = *p - school.anchor;
            let nose = at + forward * length * sz * 0.5;
            let tail = at - forward * length * sz * 0.5;
            let ring: Vec<Vec3> = (0..SIDES)
                .map(|k| {
                    let a = k as f32 / SIDES as f32 * std::f32::consts::TAU;
                    at - forward * length * sz * 0.1
                        + side * a.cos() * length * sx * 0.25
                        + top * a.sin() * length * sy * 0.25
                })
                .collect();
            for k in 0..SIDES {
                let (a, b) = (ring[k], ring[(k + 1) % SIDES]);
                positions.extend_from_slice(&[nose, a, b, tail, b, a]);
            }
            // The tail fin, a flat triangle across the body.
            let fin = tail - forward * length * sz * 0.2;
            positions.extend_from_slice(&[
                tail,
                fin + top * length * sy * 0.3,
                fin - top * length * sy * 0.3,
            ]);
        }
        let count = positions.len();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.remove_indices();
        mesh.compute_flat_normals();
        if count == 0 {
            mesh.remove_attribute(Mesh::ATTRIBUTE_NORMAL);
        }
    }

    /// The rod in hand, the float on the water, the line between them, and the
    /// HUD line.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn tackle(
        mut commands: Commands,
        fishery: Res<Fishery>,
        tools: Res<ToolSlot>,
        status: Res<FishingStatus>,
        walking: Option<Res<WalkingReadout>>,
        frame: Res<crate::planet::PlanetRenderFrame>,
        sea: Option<Res<Sea>>,
        cameras: Query<(Entity, &GlobalTransform), With<WalkingCamera>>,
        mut rods: Query<(Entity, &mut Visibility), (With<Rod>, Without<Float>, Without<LineMesh>)>,
        mut floats: Query<
            (&mut Transform, &mut Visibility),
            (With<Float>, Without<LineMesh>, Without<Rod>),
        >,
        mut lines: Query<
            (&Mesh3d, &mut Transform, &mut Visibility),
            (With<LineMesh>, Without<Float>, Without<Rod>),
        >,
        mut texts: Query<&mut Text, With<StatusText>>,
        mut meter: Query<&mut Node, (With<Meter>, Without<MeterFill>)>,
        mut fill: Query<(&mut Node, &mut BackgroundColor), (With<MeterFill>, Without<Meter>)>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        let walking = walking.is_some_and(|w| w.active);
        let rod_out = walking && tools.held() == Tool::Rod;
        let camera = cameras.iter().next();
        if rods.is_empty()
            && let Some((camera, _)) = camera
        {
            // Tenebris's rod: a tapered stick from the grip to the tip, in
            // the eye's own frame so it moves with the look and nothing else.
            let along = ROD_TIP - ROD_GRIP;
            let span = along.length();
            let rod = commands
                .spawn((
                    Name::new("Fishing rod"),
                    Rod,
                    Mesh3d(meshes.add(ConicalFrustum {
                        radius_top: 0.006,
                        radius_bottom: 0.016,
                        height: span,
                    })),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.54, 0.35, 0.17),
                        perceptual_roughness: 0.7,
                        ..default()
                    })),
                    Transform::from_translation(ROD_GRIP + along * 0.5)
                        .with_rotation(Quat::from_rotation_arc(Vec3::Y, along / span)),
                    Visibility::Hidden,
                ))
                .id();
            commands.entity(camera).add_child(rod);
        }
        for (_, mut visibility) in &mut rods {
            *visibility = if rod_out {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
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
            let Some((_, eye)) = camera else {
                continue;
            };
            if !out {
                continue;
            }
            let tip = eye.transform_point(ROD_TIP);
            transform.translation = tip;
            let slack = match line.phase {
                Phase::Hooked => 0.1 + 0.5 * (1.0 - line.tension.min(1.0)),
                _ => 0.6,
            };
            let tip_local = (tip.as_dvec3() - frame.center).as_vec3();
            let down = -tip_local.normalize_or(Vec3::Y);
            // A slack line lies on the water rather than hanging through it:
            // the mockup's sag ran under the surface and looped back up.
            let surface = sea.as_ref().map_or(0.0, |sea| sea.radius);
            let points: Vec<Vec3> = (0..LINE_POINTS)
                .map(|k| {
                    let u = k as f32 / (LINE_POINTS - 1) as f32;
                    let offset = (float_at - tip) * u + down * slack * 4.0 * u * (1.0 - u);
                    let at = tip_local + offset;
                    let r = at.length();
                    if r > 0.0 && r < surface + 0.01 {
                        at * ((surface + 0.01) / r) - tip_local
                    } else {
                        offset
                    }
                })
                .collect();
            if let Some(mesh) = meshes.get_mut(&mesh.0) {
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, points);
            }
        }
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
