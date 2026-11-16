//! The starter missions on screen (`starter-missions` design 12-14): one HUD
//! line for the current mission, and the Missions tab beside the pack.
//!
//! The counting and the record are `pbd_app::missions`; this only draws them.
//!
//! - **The HUD line** sits top right under the compass strip, in the kit's
//!   face: `CATCH FISH 1/3`, then `DONE: CATCH FISH` for a few seconds when
//!   one closes. The player's "Mission line" setting hides it.
//! - **The tab** is a menu screen of its own (`Screen::Missions`), opened
//!   with O, as the pack is with I; a strip of two oak tabs over both
//!   switches between them. The tab draws the tree (`mission_tree`).

use super::menu::Screen;
use bevy::prelude::*;
use pbd_app::missions::{MissionChain, Missions};
use pbd_app::settings::PlayerSettings;
use pbd_app::ui_kit::{self, KitText, Plaque, Set, UiKit};

/// The HUD line.
#[derive(Component)]
pub struct MissionLine;

/// The strip of tabs over the pack and the missions, and what each opens.
#[derive(Component)]
pub struct TabStrip;
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct TabTo(pub Screen);

/// The strip over the pack and the missions, left to right: their own two
/// tabs, then links to the map, the guide and the pause menu
/// (`inventory-grid` decision 9; owner, 2026-11-15: "pack should buttons to
/// other menus like the pause menu or the map").
pub const TABS: [(&str, Screen); 5] = [
    ("PACK  I", Screen::Pack),
    ("MISSIONS  O", Screen::Missions),
    ("MAP  M", Screen::Map),
    ("GUIDE  J", Screen::Guide),
    ("MENU  ESC", Screen::Pause),
];

pub fn spawn(mut commands: Commands, kit: Res<UiKit>) {
    let kit = &*kit;
    // The HUD line, under the compass strip at the right.
    commands.spawn((
        KitText::new("", ui_kit::TEXT).shadow(ui_kit::INK),
        Node {
            position_type: PositionType::Absolute,
            right: px(6),
            top: px(36),
            ..default()
        },
        Visibility::Hidden,
        MissionLine,
    ));
    // The tabs, over whichever of the two panels is open.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(10),
                width: percent(100),
                justify_content: JustifyContent::Center,
                column_gap: px(4),
                display: Display::None,
                ..default()
            },
            GlobalZIndex(12),
            TabStrip,
        ))
        .with_children(|strip| {
            for (label, to) in TABS {
                strip
                    .spawn((ui_kit::plaque(kit, Set::Oak, 70.0), TabTo(to)))
                    .with_children(|p| {
                        p.spawn(ui_kit::plaque_label(Set::Oak, label));
                    });
            }
        });
}

/// The HUD line: the current mission, or the one just done, unless the
/// player hid it or a menu is open.
pub fn show_line(
    missions: Res<Missions>,
    chain: Res<MissionChain>,
    screen: Res<Screen>,
    settings: Res<PlayerSettings>,
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    mut line: Query<(&mut KitText, &mut Visibility), With<MissionLine>>,
) {
    let (text, colour) = match &missions.notice {
        Some((title, _)) => (format!("DONE: {title}"), ui_kit::RUNE),
        None => (
            missions.progress.line(&chain.0).unwrap_or_default(),
            ui_kit::TEXT,
        ),
    };
    let shown = settings.mission_line
        && *screen == Screen::Playing
        && walking.is_some_and(|w| w.active)
        && !text.is_empty();
    for (mut words, mut visibility) in &mut line {
        let want = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != want {
            *visibility = want;
        }
        if words.text != text || words.colour != colour {
            words.text.clone_from(&text);
            words.colour = colour;
        }
    }
}

/// Keep the tab strip current (the tab itself is `mission_tree`).
pub fn paint(
    screen: Res<Screen>,
    mut strip: Query<&mut Node, With<TabStrip>>,
    mut tabs: Query<(&TabTo, &mut Plaque)>,
) {
    let open = matches!(*screen, Screen::Pack | Screen::Missions);
    for mut node in &mut strip {
        let want = if open { Display::Flex } else { Display::None };
        if node.display != want {
            node.display = want;
        }
    }
    for (to, mut plaque) in &mut tabs {
        let latched = to.0 == *screen;
        if plaque.latched != latched {
            plaque.latched = latched;
        }
    }
}

/// A press on a tab switches between the pack and the missions.
pub fn press_tabs(
    tabs: Query<(&Interaction, &TabTo), Changed<Interaction>>,
    mut screen: ResMut<Screen>,
    mut view: ResMut<super::map_screen::MapView>,
    player: super::map_screen::PlayerPlace,
) {
    for (interaction, to) in &tabs {
        if *interaction != Interaction::Pressed || *screen == to.0 {
            continue;
        }
        // The map opens centred on the player, as M does.
        match to.0 {
            Screen::Map => super::map_screen::show(&mut screen, &mut view, &player),
            other => *screen = other,
        }
    }
}

/// One step of the capture instrument's chain: a message the game raises.
#[derive(Clone, Debug)]
enum Scripted {
    Gained(pbd_core::inventory::Item, u16, pbd_app::hotbar::How),
    Did(pbd_core::missions::Kind, &'static str),
    Planted(&'static str),
    Deed(pbd_app::husbandry::Deed),
    Cooked(&'static str, u16),
}

/// The whole chain as the systems would raise it in play, from a fresh
/// world's first dig to the farm meal (`--missions <n>` raises the first
/// `n`, one every few frames). It plays no fishing or farming: it raises
/// the messages, so the counting, the record, the HUD line and the tab are
/// the game's own.
fn script() -> Vec<Scripted> {
    use Scripted as S;
    use pbd_app::hotbar::How::*;
    use pbd_app::husbandry::Deed;
    use pbd_core::goods::Good;
    use pbd_core::inventory::{Item, Tool};
    use pbd_core::missions::Kind;
    use pbd_core::terrain::Material;
    let good = |code| Item::Good(Good::by_code(code).expect("a shipped good"));
    let n = |k: usize, step: Scripted| std::iter::repeat_n(step, k);
    // The tree (`starter-missions` T.2) in the order the bot plays it.
    let mut steps = vec![
        S::Gained(good("fibre"), 8, Found),
        S::Gained(good("stone"), 9, Found),
    ];
    steps.extend(n(6, S::Did(Kind::Gather, "block:dirt")));
    steps.push(S::Did(Kind::Craft, "recipe:stone-axe"));
    steps.extend(n(2, S::Did(Kind::Fell, "tree")));
    steps.extend([
        S::Gained(good("log"), 10, Found),
        S::Gained(good("planks"), 8, Made),
        S::Gained(good("rope"), 2, Made),
        S::Gained(Item::Block(Material::Workbench), 1, Made),
        S::Did(Kind::Place, "block:workbench"),
        S::Gained(Item::Tool(Tool::Pickaxe), 1, Made),
        S::Gained(Item::Tool(Tool::Shovel), 1, Made),
        S::Gained(Item::Block(Material::Campfire), 1, Made),
        S::Did(Kind::Place, "block:campfire"),
    ]);
    steps.extend(n(20, S::Did(Kind::Place, "block:dirt")));
    steps.extend(n(2, S::Did(Kind::Craft, "recipe:torch")));
    steps.extend(n(4, S::Did(Kind::Place, "block:torch")));
    steps.extend([
        S::Did(Kind::Night, "light"),
        S::Gained(Item::Tool(Tool::Rod), 1, Made),
        S::Did(Kind::Cast, "water"),
    ]);
    steps.extend(n(3, S::Did(Kind::Catch, "minnow")));
    steps.push(S::Gained(good("bait"), 3, Found));
    steps.extend(n(3, S::Did(Kind::Baited, "minnow")));
    steps.extend([
        S::Did(Kind::Species, "minnow"),
        S::Did(Kind::Species, "banded perch"),
        S::Did(Kind::Species, "eel"),
    ]);
    steps.extend([S::Did(Kind::Open, "map"), S::Did(Kind::Reach, "marked")]);
    steps.extend(n(3, S::Did(Kind::Waters, "sea")));
    steps.extend(n(3, S::Did(Kind::Waters, "fresh")));
    steps.extend(n(20, S::Did(Kind::Swim, "water")));
    steps.extend([
        S::Did(Kind::Reach, "reef"),
        S::Did(Kind::Reach, "town"),
        S::Did(Kind::Board, "boat"),
    ]);
    steps.extend(n(100, S::Did(Kind::Travel, "boat")));
    steps.extend([
        S::Cooked("grilled-fish", 2),
        S::Did(Kind::Eat, "grilled-fish"),
    ]);
    steps.push(S::Gained(good("wheat-seed"), 10, Found));
    steps.extend(n(6, S::Did(Kind::Till, "block:grass")));
    steps.extend(n(6, S::Planted("wheat")));
    steps.push(S::Did(Kind::Pour, "water"));
    steps.extend([
        S::Gained(good("wheat"), 6, Harvested),
        S::Cooked("flatbread", 2),
        S::Gained(good("turnip"), 3, Harvested),
        S::Gained(good("chili"), 3, Harvested),
        S::Gained(good("frost-kale"), 3, Harvested),
        S::Cooked("roast-turnip", 1),
    ]);
    steps.extend(n(6, S::Did(Kind::Craft, "recipe:fence")));
    steps.extend([
        S::Did(Kind::Craft, "recipe:gate"),
        S::Deed(Deed::Tamed { species: 0 }),
        S::Deed(Deed::PenBuilt { pen: 1, cells: 7 }),
        S::Deed(Deed::Penned { species: 0 }),
        S::Gained(good("egg"), 5, Collected),
        S::Deed(Deed::Tamed { species: 1 }),
        S::Gained(good("wool"), 3, Collected),
        S::Cooked("fried-egg", 1),
        S::Did(Kind::Eat, "grilled-fish"),
        S::Did(Kind::Eat, "flatbread"),
        S::Did(Kind::Eat, "fried-egg"),
    ]);
    steps.extend(n(10, S::Did(Kind::Gather, "block:stone")));
    steps.extend(n(4, S::Did(Kind::Gather, "block:ore")));
    steps.extend([
        S::Gained(good("coal"), 3, Found),
        S::Gained(Item::Block(Material::Forge), 1, Made),
        S::Did(Kind::Place, "block:forge"),
        S::Gained(good("bronze-ingot"), 2, Made),
        S::Did(Kind::Craft, "recipe:bronze-pickaxe"),
    ]);
    steps.extend(n(5, S::Did(Kind::Gather, "block:stone")));
    steps.extend([
        S::Gained(good("iron-ingot"), 3, Made),
        S::Did(Kind::Craft, "recipe:iron-pickaxe"),
    ]);
    steps
}

/// How many frames apart the instrument raises its steps.
const SCRIPT_EVERY: u32 = 2;

/// Write one scripted step as the message its system raises.
fn raise(step: Scripted, world: &mut World) {
    match step {
        Scripted::Gained(item, count, how) => {
            world.write_message(pbd_app::hotbar::Gained { item, count, how });
        }
        Scripted::Did(kind, what) => {
            world.write_message(pbd_app::missions::Did::new(kind, what, 1));
        }
        Scripted::Planted(crop) => {
            if let Some(crop) = pbd_core::farming::Crop::by_code(crop) {
                world.write_message(pbd_app::hotbar::Planted { crop });
            }
        }
        Scripted::Deed(deed) => {
            world.write_message(deed);
        }
        Scripted::Cooked(recipe, count) => {
            if let Some(dish) = pbd_core::goods::Good::by_code(recipe) {
                world.write_message(pbd_app::kitchen::Cooked {
                    recipe: recipe.to_string(),
                    dish: pbd_core::inventory::Item::Good(dish),
                    count,
                    fish: 0,
                });
            }
        }
    }
}

/// `--missions <n>`: raise the first `n` steps of [`script`], one every
/// [`SCRIPT_EVERY`] frames once the world is open (a measurement
/// instrument, `starter-missions` design 17).
pub fn scripted(world: &mut World, mut at: Local<(usize, u32)>) {
    let Some(n) = world.resource::<super::Launch>().missions else {
        return;
    };
    at.1 += 1;
    if at.1 < SCRIPT_EVERY {
        return;
    }
    at.1 = 0;
    let Some(step) = script().into_iter().nth(at.0).filter(|_| at.0 < n) else {
        return;
    };
    at.0 += 1;
    info!("scripted mission step {}: {step:?}", at.0);
    raise(step, world);
}

/// The hour a night by the lights is counted from.
pub const NIGHT_HOUR: f32 = 20.0;
/// How near a light the player placed they have to be, metres.
pub const NIGHT_NEAR_M: f32 = 30.0;

/// `night` (`starter-missions` H.1): the clock passes 20:00 with the
/// player within 30 m of a light they placed. Raised once each evening.
pub fn night_watch(
    sun: Res<pbd_app::sky::Sun>,
    save: Res<pbd_app::saves::WorldSave>,
    fine: Option<Res<pbd_app::planet::PlanetFine>>,
    walkers: Query<&avian3d::prelude::Position, With<pbd_app::walking::Walker>>,
    mut did: MessageWriter<pbd_app::missions::Did>,
    mut was: Local<Option<f32>>,
) {
    let hour = sun.clock.hour();
    let crossed = was.is_some_and(|h| h < NIGHT_HOUR && hour >= NIGHT_HOUR);
    *was = Some(hour);
    if !crossed {
        return;
    }
    let (Some(fine), Ok(feet)) = (fine, walkers.single()) else {
        return;
    };
    let lights: Vec<_> = save
        .edits
        .all()
        .into_iter()
        .filter(|e| pbd_app::hotbar::light_icon(e.material).is_some())
        .collect();
    if lights.is_empty() {
        return;
    }
    let toward = feet.0.normalize_or(Vec3::Y);
    let near = fine.set.finest_records().iter().any(|r| {
        lights.iter().any(|e| e.cell == r.key())
            && Vec3::from_slice(&r.direction_height[..3])
                .normalize_or(Vec3::Y)
                .distance(toward)
                * pbd_app::planet::PLANET_RADIUS
                < NIGHT_NEAR_M
    });
    if near {
        info!("night by the lights at {hour:.2}");
        did.write(pbd_app::missions::Did::new(
            pbd_core::missions::Kind::Night,
            "light",
            1,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The instrument's script walks the shipped chain to its end, through
    /// the same counting the game runs.
    #[test]
    fn the_script_walks_the_shipped_chain() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let (recipes, _) = pbd_app::config::recipe_book();
        app.insert_resource(recipes)
            .insert_resource(pbd_app::config::FaunaConfig(
                ron::from_str(include_str!("../../../../assets/config/fauna.ron")).unwrap(),
            ))
            .insert_resource(pbd_app::config::HusbandryConfig(Default::default()))
            .insert_resource(pbd_app::saves::WorldSave::memory_only())
            .insert_resource(pbd_app::hotbar::Hotbar::default())
            .insert_resource(pbd_app::fish::ToolSlot(
                pbd_core::inventory::Equipment::new_world(),
            ))
            .add_plugins(pbd_app::missions::MissionsPlugin);
        app.update();
        for step in script() {
            raise(step, app.world_mut());
            app.update();
        }
        let chain = &app.world().resource::<MissionChain>().0;
        let missions = app.world().resource::<Missions>();
        assert_eq!(missions.progress.current(chain).map(|m| &m.id), None);
        // Every mission whose kind the game raises; the pending ones wait.
        assert_eq!(
            missions.progress.done.len(),
            chain
                .missions
                .iter()
                .filter(|m| m.pending.is_none())
                .count()
        );
    }

    /// Each plaque in the strip puts its own screen up from the pack
    /// (`inventory-grid` decision 9).
    #[test]
    fn each_strip_tab_opens_its_screen() {
        for (label, to) in TABS {
            let mut app = App::new();
            app.insert_resource(Screen::Pack)
                .init_resource::<super::super::map_screen::MapView>()
                .add_systems(Update, press_tabs);
            app.world_mut()
                .spawn((Button, Interaction::Pressed, TabTo(to)));
            app.update();
            assert_eq!(*app.world().resource::<Screen>(), to, "{label}");
        }
    }
}

/// How near a town's terrace counts as reaching it.
pub const TOWN_NEAR_M: f32 = 60.0;

/// What `about_watch` carries between frames.
#[derive(Default)]
pub struct AboutState {
    /// The screen last frame.
    screen: Option<Screen>,
    /// The mark last counted as reached, so a mark counts once.
    reached_mark: Option<[f32; 3]>,
    /// The last place the walker was counted from, in the water.
    swam_from: Option<Vec3>,
    swum_m: f32,
    /// The craft boarded, and where it was last counted from.
    aboard: Option<(Entity, Vec3)>,
    travelled_m: f32,
    town: Option<u32>,
    reef: bool,
}

/// Getting about (`starter-missions` T.2): the map opened, the player's
/// mark reached (G3), metres swum, a boat boarded and the metres it went, a
/// town and a reef reached. Each is raised where the game already knows it:
/// the screen, the walker's position against the sea and the towns, the
/// craft the player is aboard.
#[allow(clippy::too_many_arguments)]
pub fn about_watch(
    screen: Res<Screen>,
    aboard: Option<Res<pbd_app::vehicles::Aboard>>,
    vehicles: Query<&pbd_app::vehicles::Vehicle>,
    towns: Option<Res<pbd_app::towns::Towns>>,
    walkers: Query<&avian3d::prelude::Position, With<pbd_app::walking::Walker>>,
    save: Option<Res<pbd_app::saves::WorldSave>>,
    mut did: MessageWriter<pbd_app::missions::Did>,
    mut state: Local<AboutState>,
) {
    use pbd_app::missions::Did;
    use pbd_app::planet::{PLANET_RADIUS, surface_height};
    use pbd_core::missions::Kind;
    let at = walkers.single().ok().map(|p| p.0);
    if state.screen != Some(*screen) {
        if *screen == Screen::Map {
            did.write(Did::new(Kind::Open, "map", 1));
        }
        state.screen = Some(*screen);
    }
    let Some(feet) = at else { return };
    let up = feet.normalize_or(Vec3::Y);
    // The player's mark, reached: one set far enough away counts once.
    if let Some(mark) = save.as_deref().and_then(super::map_screen::mark_of)
        && state.reached_mark != Some(mark.at)
        && mark.reached_far(feet, PLANET_RADIUS)
    {
        state.reached_mark = Some(mark.at);
        did.write(Did::new(Kind::Reach, "marked", 1));
    }
    // Swimming: the feet under the sea's level over the sea floor.
    let ground = surface_height(up);
    let altitude = feet.length() - PLANET_RADIUS;
    if ground < -1.0 && altitude < -0.3 {
        if let Some(from) = state.swam_from {
            state.swum_m += (feet - from).length();
        }
        state.swam_from = Some(feet);
        while state.swum_m >= 1.0 {
            state.swum_m -= 1.0;
            did.write(Did::new(Kind::Swim, "water", 1));
        }
        // A reef under the swimmer.
        if !state.reef
            && pbd_core::ocean::places::environment(pbd_app::planet::terrain_config(), up)
                == Some(pbd_core::fauna::Environment::Reef)
        {
            state.reef = true;
            did.write(Did::new(Kind::Reach, "reef", 1));
        }
    } else {
        state.swam_from = None;
    }
    // A town reached.
    if let Some(towns) = towns {
        let near = towns.held.iter().find(|h| {
            h.point(PLANET_RADIUS).normalize_or(Vec3::Y).distance(up) * PLANET_RADIUS < TOWN_NEAR_M
        });
        let id = near.map(|h| h.site.id);
        if id.is_some() && id != state.town {
            did.write(Did::new(Kind::Reach, "town", 1));
        }
        state.town = id;
    }
    // A boat boarded, and the metres it goes.
    let craft = aboard
        .and_then(|a| a.0)
        .and_then(|e| vehicles.get(e).ok().map(|v| (e, v)))
        .filter(|(_, v)| {
            matches!(
                v.craft.kind,
                pbd_core::vehicle::Kind::Rowboat | pbd_core::vehicle::Kind::Cog
            )
        });
    match (craft, state.aboard) {
        (Some((e, v)), Some((was, from))) if e == was => {
            let here = v.craft.body.position.as_vec3();
            state.travelled_m += (here - from).length();
            state.aboard = Some((e, here));
            while state.travelled_m >= 1.0 {
                state.travelled_m -= 1.0;
                did.write(Did::new(Kind::Travel, "boat", 1));
            }
        }
        (Some((e, v)), _) => {
            did.write(Did::new(Kind::Board, "boat", 1));
            state.aboard = Some((e, v.craft.body.position.as_vec3()));
        }
        (None, _) => state.aboard = None,
    }
}
