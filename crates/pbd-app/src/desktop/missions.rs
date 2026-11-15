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
//!   switches between them. It lists the chain in order: done, the current
//!   mission with each objective's count, open, locked and pending.

use super::menu::{self, Panel, Screen};
use bevy::prelude::*;
use pbd_app::missions::{MissionChain, Missions};
use pbd_app::settings::PlayerSettings;
use pbd_app::ui_kit::{self, KitText, Plaque, Set, UiKit};
use pbd_core::missions::{Mission, Progress, Standing};

/// The HUD line.
#[derive(Component)]
pub struct MissionLine;

/// One mission's row in the tab, by its place in the chain: its title, its
/// standing, and the line of counts under it.
#[derive(Component)]
pub struct RowTitle(pub usize);
#[derive(Component)]
pub struct RowStanding(pub usize);
#[derive(Component)]
pub struct RowCounts(pub usize);

/// The current mission's line of help, at the foot of the tab.
#[derive(Component)]
pub struct HelpText;

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

/// The tab's width, native pixels.
const WIDTH: f32 = 300.0;

pub fn spawn(mut commands: Commands, kit: Res<UiKit>, chain: Res<MissionChain>) {
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
    // The tab's panel: the stone moulding, the lintel, a row per mission.
    commands
        .spawn((menu::layer(), GlobalZIndex(10), Panel(Screen::Missions)))
        .with_children(|layer| {
            layer
                .spawn(menu::framed(kit, Set::Stone, WIDTH))
                .with_children(|panel| {
                    menu::lintel(panel, kit, "MISSIONS");
                    for (index, _) in chain.0.missions.iter().enumerate() {
                        panel
                            .spawn(Node {
                                flex_direction: FlexDirection::Column,
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn(Node {
                                    justify_content: JustifyContent::SpaceBetween,
                                    width: percent(100),
                                    ..default()
                                })
                                .with_children(|line| {
                                    line.spawn((
                                        KitText::new("", ui_kit::TEXT).shadow(ui_kit::INK),
                                        RowTitle(index),
                                    ));
                                    line.spawn((
                                        KitText::new("", ui_kit::TEXT_DIM).shadow(ui_kit::INK),
                                        RowStanding(index),
                                    ));
                                });
                                row.spawn((
                                    KitText::new("", ui_kit::RUNE).shadow(ui_kit::INK),
                                    Node {
                                        margin: UiRect::new(px(8), px(0), px(2), px(1)),
                                        display: Display::None,
                                        ..default()
                                    },
                                    RowCounts(index),
                                ));
                            });
                    }
                    panel.spawn((
                        menu::body("", 12.0, ui_kit::TEXT_DIM),
                        Node {
                            margin: UiRect::top(px(5)),
                            max_width: px(WIDTH - 28.0),
                            ..default()
                        },
                        HelpText,
                    ));
                });
        });
}

/// What a row's right column says.
fn standing_word(standing: Standing, current: bool) -> &'static str {
    match standing {
        Standing::Done => "DONE",
        Standing::Open if current => "NOW",
        Standing::Open => "OPEN",
        Standing::Locked => "LOCKED",
        Standing::Pending => "NOT YET IN THE GAME",
    }
}

/// A row's line of counts: `SOIL 2/4 · FIBRE 0/3`.
pub fn counts_line(progress: &Progress, mission: &Mission) -> String {
    let counts = progress.counts(mission);
    mission
        .objectives
        .iter()
        .zip(counts)
        .map(|(o, n)| format!("{} {n}/{}", o.label, o.count))
        .collect::<Vec<_>>()
        .join("  ·  ")
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

/// Keep the tab's rows and the tab strip current.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn paint(
    missions: Res<Missions>,
    chain: Res<MissionChain>,
    screen: Res<Screen>,
    mut strip: Query<&mut Node, (With<TabStrip>, Without<RowCounts>)>,
    mut tabs: Query<(&TabTo, &mut Plaque)>,
    mut titles: Query<(&RowTitle, &mut KitText), (Without<RowStanding>, Without<RowCounts>)>,
    mut standings: Query<(&RowStanding, &mut KitText), (Without<RowTitle>, Without<RowCounts>)>,
    mut counts: Query<
        (&RowCounts, &mut KitText, &mut Node),
        (Without<RowTitle>, Without<RowStanding>, Without<TabStrip>),
    >,
    mut help: Query<&mut Text, With<HelpText>>,
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
    if *screen != Screen::Missions || !(missions.is_changed() || screen.is_changed()) {
        return;
    }
    let chain = &chain.0;
    let progress = &missions.progress;
    let current = progress.current(chain).map(|m| m.id.clone());
    let standing = |i: usize| {
        let mission = &chain.missions[i];
        (
            progress.standing(chain, mission),
            current.as_deref() == Some(mission.id.as_str()),
        )
    };
    for (row, mut text) in &mut titles {
        let (s, now) = standing(row.0);
        let colour = match s {
            Standing::Done => ui_kit::TEXT_DIM,
            Standing::Open if now => ui_kit::TITLE,
            Standing::Open => ui_kit::TEXT,
            Standing::Locked | Standing::Pending => ui_kit::LABEL_DIM,
        };
        let title = chain.missions[row.0].title.clone();
        if text.text != title || text.colour != colour {
            text.text = title;
            text.colour = colour;
        }
    }
    for (row, mut text) in &mut standings {
        let (s, now) = standing(row.0);
        let word = standing_word(s, now);
        let colour = if now { ui_kit::RUNE } else { ui_kit::TEXT_DIM };
        if text.text != word || text.colour != colour {
            text.text = word.to_string();
            text.colour = colour;
        }
    }
    for (row, mut text, mut node) in &mut counts {
        let (s, _) = standing(row.0);
        let show = s == Standing::Open;
        let want = if show { Display::Flex } else { Display::None };
        if node.display != want {
            node.display = want;
        }
        let line = counts_line(progress, &chain.missions[row.0]);
        if show && text.text != line {
            text.text = line;
        }
    }
    let words = progress
        .current(chain)
        .map_or("Every starter mission is done.".to_string(), |m| {
            m.help.clone()
        });
    for mut text in &mut help {
        if text.0 != words {
            text.0.clone_from(&words);
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
        S::Gained(good("fibre"), 4, Found),
        S::Gained(good("stone"), 4, Found),
    ];
    steps.extend(n(4, S::Did(Kind::Gather, "block:dirt")));
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
    steps.push(S::Gained(good("wheat-seed"), 2, Found));
    steps.extend(n(4, S::Did(Kind::Till, "block:grass")));
    steps.extend(n(4, S::Planted("wheat")));
    steps.push(S::Gained(good("wheat"), 4, Harvested));
    steps.extend(n(6, S::Did(Kind::Craft, "recipe:fence")));
    steps.extend([
        S::Did(Kind::Craft, "recipe:gate"),
        S::Deed(Deed::Tamed { species: 0 }),
        S::Deed(Deed::PenBuilt { pen: 1, cells: 7 }),
        S::Deed(Deed::Penned { species: 0 }),
        S::Gained(good("egg"), 2, Collected),
    ]);
    steps.extend(n(10, S::Did(Kind::Gather, "block:stone")));
    steps.extend(n(4, S::Did(Kind::Gather, "block:ore")));
    steps.extend([
        S::Gained(good("coal"), 2, Found),
        S::Gained(Item::Block(Material::Forge), 1, Made),
        S::Did(Kind::Place, "block:forge"),
        S::Gained(good("bronze-ingot"), 2, Made),
        S::Did(Kind::Craft, "recipe:bronze-pickaxe"),
    ]);
    steps.extend(n(5, S::Did(Kind::Gather, "block:stone")));
    steps.extend([
        S::Gained(good("iron-ingot"), 1, Made),
        S::Did(Kind::Craft, "recipe:iron-pickaxe"),
        S::Gained(Item::Block(Material::Campfire), 1, Made),
        S::Did(Kind::Place, "block:campfire"),
        S::Cooked("grilled-fish", 2),
        S::Cooked("flatbread", 2),
        S::Did(Kind::Eat, "flatbread"),
        S::Cooked("roast-turnip", 1),
        S::Cooked("fried-egg", 1),
        S::Did(Kind::Eat, "grilled-fish"),
        S::Did(Kind::Eat, "flatbread"),
        S::Did(Kind::Eat, "fried-egg"),
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
        assert_eq!(missions.progress.done.len(), chain.missions.len());
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
