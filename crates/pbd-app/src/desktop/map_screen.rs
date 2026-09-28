//! The world map (`world-map` task groups 3 to 5): M opens it, M or Escape
//! closes it, and the world keeps running behind it.
//!
//! It is a menu screen of its own, as the pack is, so it holds the pointer
//! through `MenuOpen` and closes on Escape with no code of its own for
//! either. What it draws is `pbd_app::world_map`'s (decision 10):
//! - the BASE, the whole planet at 11.3 m a pixel, as image nodes placed by
//!   the view, three of them so the antimeridian never shows an edge;
//! - the finer levels' TILES over it, where the view is close enough to need
//!   them, built on the pool and kept in an LRU;
//! - a grey veil and a raster layer (the biomes), when one is chosen;
//! - one UI material for the live layers, which turns every pixel back into
//!   a direction and looks up the weather overlay, the clouds and rain and
//!   the night there (`map_live.wgsl`);
//! - the markers: the player and heading, the ship and every craft.
//!
//! A capture (the clock pinned) builds what it shows in place, so its
//! picture is a function of its flags.

use super::menu::{self, EDGE, INK, MINT, PANEL_FILL, Panel, Screen};
use avian3d::prelude::Position;
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderType, TextureDimension, TextureFormat, TextureViewDescriptor,
    TextureViewDimension,
};
use bevy::shader::ShaderRef;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future};
use bevy::ui::FocusPolicy;
use bevy::ui_render::prelude::{MaterialNode, UiMaterial, UiMaterialPlugin};
use bevy::window::PrimaryWindow;
use pbd_app::atmosphere::{Air, MAP_SIZE};
use pbd_app::flight_view::PilotShip;
use pbd_app::overlay::{OverlayMode, RAMPS, overlay_rgba, overlay_texels};
use pbd_app::planet::{PLANET_RADIUS, terrain_config};
use pbd_app::saves::WorldSave;
use pbd_app::sky::Sun;
use pbd_app::vehicles::Vehicle;
use pbd_app::walking::{Walker, WalkingReadout};
use pbd_app::world_map::{
    self as raster, BASE, LEVELS, MapLayers, Palette, RasterLayer, TILE, TileKey, TileLru,
};
use pbd_core::geo::{self, LatLon};
use pbd_core::map::Texel;
use pbd_core::overlay::Overlay;
use std::collections::HashMap;
use std::sync::Arc;

/// How the map opens, from the launch flags (`--map-*`), for its captures.
#[derive(Resource, Clone, Debug, Default)]
pub struct MapLaunch {
    pub metres_per_pixel: Option<f32>,
    pub overlay: Option<Overlay>,
    pub layer: Option<String>,
    pub night: bool,
    pub clouds: bool,
    pub at: Option<(f32, f32)>,
}

/// Where the map is looking: the map position at the centre of the screen
/// (`geo::project`'s u and v), and how many screen pixels one turn of
/// longitude takes, which is the zoom.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct MapView {
    pub centre: Vec2,
    pub px_per_turn: f32,
}

impl Default for MapView {
    fn default() -> Self {
        Self {
            centre: Vec2::splat(0.5),
            // About a quarter of the planet across a 1440-pixel window.
            px_per_turn: 6000.0,
        }
    }
}

/// The equator's length, metres.
fn equator_m() -> f32 {
    std::f32::consts::TAU * PLANET_RADIUS
}

/// Screen pixels per turn at the closest zoom: a finest-level pixel four
/// screen pixels wide, a cell about four pixels across.
fn closest() -> f32 {
    LEVELS[LEVELS.len() - 1] as f32 * 4.0
}

impl MapView {
    /// The zoom that shows `metres` a screen pixel at the equator.
    pub fn at_metres_per_pixel(centre: Vec2, metres: f32) -> Self {
        Self {
            centre,
            px_per_turn: equator_m() / metres.max(1e-3),
        }
    }

    /// Metres a screen pixel at the equator.
    pub fn metres_per_pixel(self) -> f32 {
        equator_m() / self.px_per_turn
    }

    /// The view kept on the map: no wider than the whole planet on screen,
    /// no closer than [`closest`], the poles no further in than the window's
    /// edge, and the longitude wrapped.
    pub fn clamped(self, size: Vec2) -> Self {
        let widest = size.x.min(2.0 * size.y).max(1.0);
        let px_per_turn = self.px_per_turn.clamp(widest, closest());
        let tall = px_per_turn / 2.0;
        let half = size.y / 2.0 / tall;
        let v = if half >= 0.5 {
            0.5
        } else {
            self.centre.y.clamp(half, 1.0 - half)
        };
        Self {
            centre: Vec2::new(self.centre.x.rem_euclid(1.0), v),
            px_per_turn,
        }
    }

    /// Where a map position lands on a screen of `size`, pixels from its top
    /// left, taking the copy of it nearest the centre in longitude.
    pub fn to_screen(self, uv: Vec2, size: Vec2) -> Vec2 {
        let du = (uv.x - self.centre.x + 0.5).rem_euclid(1.0) - 0.5;
        let dv = uv.y - self.centre.y;
        size / 2.0 + Vec2::new(du * self.px_per_turn, dv * self.px_per_turn / 2.0)
    }

    /// The map position under a screen pixel.
    pub fn to_map(self, at: Vec2, size: Vec2) -> Vec2 {
        let d = at - size / 2.0;
        self.centre + Vec2::new(d.x / self.px_per_turn, d.y / (self.px_per_turn / 2.0))
    }

    /// Zoom by `factor` about a screen pixel, which keeps the place under it.
    pub fn zoom_about(self, at: Vec2, size: Vec2, factor: f32) -> Self {
        let under = self.to_map(at, size);
        let zoomed = Self {
            px_per_turn: self.px_per_turn * factor,
            ..self
        }
        .clamped(size);
        let moved = zoomed.to_map(at, size);
        Self {
            centre: zoomed.centre + (under - moved),
            ..zoomed
        }
        .clamped(size)
    }

    /// How much of the map a screen of `size` spans, in u across and v down.
    pub fn span(self, size: Vec2) -> Vec2 {
        Vec2::new(size.x / self.px_per_turn, size.y / (self.px_per_turn / 2.0))
    }

    /// The level the view needs: the coarsest whose pixel is no bigger than a
    /// screen pixel, else the finest.
    pub fn level(self) -> usize {
        LEVELS
            .iter()
            .position(|&width| width as f32 >= self.px_per_turn)
            .unwrap_or(LEVELS.len() - 1)
    }
}

/// What the legend has chosen.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Default)]
pub struct MapChoice {
    /// The overlay on the base, at most one: a raster layer by its index in
    /// `MapLayers`, or a weather overlay.
    pub overlay: MapOverlay,
    /// The chosen weather overlay painted on the 3D globe too.
    pub on_globe: bool,
    pub night: bool,
    pub clouds: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MapOverlay {
    #[default]
    None,
    Layer(usize),
    Weather(Overlay),
}

impl MapOverlay {
    fn weather(self) -> Option<Overlay> {
        match self {
            MapOverlay::Weather(overlay) => Some(overlay),
            _ => None,
        }
    }
}

/// The map's live layers as `map_live.wgsl` reads them.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct MapLiveParams {
    pub view: Vec4,
    pub sun: Vec4,
    pub layers: Vec4,
}

/// The one material the live layers are drawn with.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct MapLiveMaterial {
    #[uniform(0)]
    pub params: MapLiveParams,
    #[texture(1, dimension = "cube")]
    #[sampler(2)]
    pub cloud: Handle<Image>,
    #[texture(3, dimension = "cube")]
    #[sampler(4)]
    pub overlay: Handle<Image>,
}

impl UiMaterial for MapLiveMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/map_live.wgsl".into()
    }
}

/// The base and a finer tile, drawn as built or greyed out beneath an
/// overlay as the mockup greys the planet (`world-map` decision 11).
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct MapImageMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub image: Handle<Image>,
    /// x 1 to grey the image out.
    #[uniform(2)]
    pub grey: Vec4,
}

impl UiMaterial for MapImageMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/map_image.wgsl".into()
    }
}

/// How opaque a weather overlay is drawn over the greyed base: the mockup's
/// 0.9 (decision 11). The globe keeps its own `overlay_opacity`.
pub const MAP_OVERLAY_OPACITY: f32 = 0.9;
/// How strongly the night side is drawn over an overlay, as a share of its
/// full strength: the mockup's 0.3.
pub const NIGHT_UNDER_OVERLAY: f32 = 0.3;

/// The greying switch as the material's uniform reads it.
fn grey_of(on: bool) -> Vec4 {
    Vec4::new(if on { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0)
}

/// The base, the tiles and the layers as they are built.
#[derive(Resource)]
pub struct MapRaster {
    palette: Option<Arc<Palette>>,
    pub base: Option<Handle<Image>>,
    base_texels: Option<Arc<Vec<Texel>>>,
    building: Option<Task<(Vec<Texel>, Vec<u8>)>>,
    tiles: TileLru<Handle<Image>>,
    pending: HashMap<TileKey, Task<(TileKey, Vec<u8>)>>,
    layers: HashMap<usize, Handle<Image>>,
    /// Whether a new base was built this run, rather than read from the
    /// cache: the cache test's witness.
    pub built_fresh: bool,
}

impl Default for MapRaster {
    fn default() -> Self {
        Self {
            palette: None,
            base: None,
            base_texels: None,
            building: None,
            tiles: TileLru::new(TILES_KEPT),
            pending: HashMap::new(),
            layers: HashMap::new(),
            built_fresh: false,
        }
    }
}

/// How many finer tiles are kept, about 18 MB of pixels at 333 x 333.
const TILES_KEPT: usize = 120;
/// How many tiles are built at once.
const TILES_BUILDING: usize = 8;

/// The live layers' cubes and the builds that fill them.
#[derive(Resource)]
pub struct MapLive {
    pub material: Handle<MapLiveMaterial>,
    /// The base's material, shared by its three copies.
    pub base: Handle<MapImageMaterial>,
    cloud: Handle<Image>,
    overlay: Handle<Image>,
    cloud_generation: u64,
    overlay_built: Option<(u64, Overlay)>,
    task: Option<Task<(u64, Overlay, Vec<u8>)>>,
}

#[derive(Component)]
pub struct MapRoot;
#[derive(Component)]
pub struct MapCanvas;
/// One of the base's three copies, by which turn of longitude it is.
#[derive(Component)]
pub struct BaseCopy(i32);
/// One of the raster layer's three copies.
#[derive(Component)]
pub struct LayerCopy(i32);
#[derive(Component)]
pub struct TileLayer;
/// A tile on screen, by its unwrapped column and its row.
#[derive(Component)]
pub struct TileNode {
    key: TileKey,
    column: i32,
}
#[derive(Component)]
pub struct LiveNode;
#[derive(Component)]
pub struct MarkerLayer;
/// A marker: the player's, the ship's, or a craft's.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapMarker {
    Player,
    Ship,
    Craft(Entity),
}
/// The legend's buttons.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapButton {
    Overlay(MapOverlay),
    Night,
    Clouds,
    OnGlobe,
}
#[derive(Component)]
pub struct MapReadout;
#[derive(Component)]
pub struct MapNote;
/// The legend's colour key: the chosen layer's swatches, or the chosen
/// weather's ramp (`world-map` decision 11).
#[derive(Component)]
pub struct MapKey;

/// The marker pictures: an arrow for the player, pointing north until turned,
/// and a diamond for a craft.
#[derive(Resource)]
pub struct MarkerIcons {
    arrow: Handle<Image>,
    craft: Handle<Image>,
}

fn icon(size: u32, inside: impl Fn(f32, f32) -> Option<[u8; 4]>) -> Image {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let (fx, fy) = (
                (x as f32 + 0.5) / size as f32 * 2.0 - 1.0,
                (y as f32 + 0.5) / size as f32 * 2.0 - 1.0,
            );
            data.extend(inside(fx, fy).unwrap_or([0; 4]));
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::linear();
    image
}

/// The player's arrow: a white arrowhead with a dark rim, its tip up, drawn
/// at twice the size it is shown so its edges stay clean when it turns.
fn arrow_icon() -> Image {
    icon(64, |x, y| {
        // Inside the arrowhead: below the two edges from the tip, above the
        // notch in its base.
        let edge = |x: f32, y: f32| y > -0.92 + 2.0 * x.abs() && y < 0.85 - 0.9 * x.abs();
        if edge(x, y) {
            let inner = edge(x * 1.3, (y + 0.05) * 1.22);
            Some(if inner {
                [255, 250, 235, 255]
            } else {
                [20, 24, 30, 255]
            })
        } else {
            None
        }
    })
}

/// A craft: an amber diamond with a dark rim.
fn craft_icon() -> Image {
    icon(24, |x, y| {
        let r = x.abs() + y.abs();
        if r < 0.9 {
            Some(if r < 0.62 {
                [240, 170, 60, 255]
            } else {
                [20, 24, 30, 255]
            })
        } else {
            None
        }
    })
}

/// A picture of `width x height` RGBA8 sRGB pixels, drawn filtered
/// (decision 9) whatever the app's pixel-art default.
fn map_image(width: usize, height: usize, rgba: Vec<u8>) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::linear();
    image
}

/// A cube of the weather maps' size, filtered.
fn cube_image(format: TextureFormat) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: MAP_SIZE as u32,
            height: MAP_SIZE as u32,
            depth_or_array_layers: 6,
        },
        TextureDimension::D2,
        vec![0; 6 * MAP_SIZE * MAP_SIZE * 4],
        format,
        RenderAssetUsages::default(),
    );
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
    image.sampler = ImageSampler::linear();
    image
}

/// The clouds as the map draws them, one texel per weather map texel: the
/// cover, and the rain and the snow at up to 8 mm an hour.
pub fn cloud_bytes(cloud: &[[f32; 4]]) -> Vec<u8> {
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    cloud
        .iter()
        .flat_map(|t| [byte(t[0]), byte(t[2] / 8.0), byte(-t[2] / 8.0), 255])
        .collect()
}

/// The map's layers: the biomes, as the mockup had them. Added through the
/// registry like any other layer.
pub fn biomes_layer() -> RasterLayer {
    RasterLayer {
        name: "Biomes",
        note: "Each place's biome over the greyed base (survey M3).",
        paint: raster::biome_colour,
        key: raster::BIOME_KEY,
        class: raster::biome_class,
    }
}

/// Where the player is: the walker when on foot, else the ship they fly,
/// body-local and planet-centred, as the readouts read them.
#[derive(bevy::ecs::system::SystemParam)]
pub struct PlayerPlace<'w, 's> {
    walking: Option<Res<'w, WalkingReadout>>,
    walker: Query<'w, 's, &'static Position, With<Walker>>,
    ship: Query<'w, 's, &'static Position, (With<PilotShip>, Without<Walker>)>,
}

impl PlayerPlace<'_, '_> {
    pub fn get(&self) -> Option<Vec3> {
        let on_foot = self.walking.as_ref().is_some_and(|w| w.active);
        let at = if on_foot {
            self.walker.iter().next()
        } else {
            self.ship.iter().next()
        };
        at.map(|p| p.0).filter(|p| p.length_squared() > 1.0)
    }

    /// Whether the walker is out: the place is then the walker's, which a
    /// launch's first frames do not yet have.
    pub fn walking(&self) -> bool {
        self.walking.as_ref().is_some_and(|w| w.active)
    }
}

/// M opens the map from the world, on foot or at the controls, centred on the
/// player, and closes it again. Escape closes it through the menu's own step.
pub fn open(
    keys: Res<ButtonInput<KeyCode>>,
    mut screen: ResMut<Screen>,
    mut view: ResMut<MapView>,
    player: PlayerPlace,
) {
    if !keys.just_pressed(KeyCode::KeyM) {
        return;
    }
    match *screen {
        Screen::Playing => {
            if let Some(at) = player.get() {
                view.centre = geo::project(at);
            }
            *screen = Screen::Map;
        }
        Screen::Map => *screen = Screen::Playing,
        _ => {}
    }
}

/// Build the map's screen, hidden, and its live material. `Startup`.
pub fn spawn(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<MapLiveMaterial>>,
    mut bases: ResMut<Assets<MapImageMaterial>>,
    layers: Res<MapLayers>,
    launch: Option<Res<MapLaunch>>,
) {
    let base = bases.add(MapImageMaterial {
        image: Handle::default(),
        grey: grey_of(false),
    });
    let cloud = images.add(cube_image(TextureFormat::Rgba8Unorm));
    let overlay = images.add(cube_image(TextureFormat::Rgba8UnormSrgb));
    let material = materials.add(MapLiveMaterial {
        params: MapLiveParams::default(),
        cloud: cloud.clone(),
        overlay: overlay.clone(),
    });
    commands.insert_resource(MapLive {
        material: material.clone(),
        base: base.clone(),
        cloud,
        overlay,
        cloud_generation: 0,
        overlay_built: None,
        task: None,
    });
    let icons = MarkerIcons {
        arrow: images.add(arrow_icon()),
        craft: images.add(craft_icon()),
    };
    // The launch's choices, for a capture of a layer.
    let launch = launch.map(|l| l.clone()).unwrap_or_default();
    let mut choice = MapChoice {
        night: launch.night,
        clouds: launch.clouds,
        ..default()
    };
    if let Some(overlay) = launch.overlay {
        choice.overlay = MapOverlay::Weather(overlay);
    } else if let Some(name) = &launch.layer
        && let Some(index) = layers
            .rasters
            .iter()
            .position(|l| l.name.eq_ignore_ascii_case(name))
    {
        choice.overlay = MapOverlay::Layer(index);
    }
    commands.insert_resource(choice);
    let full = || Node {
        position_type: PositionType::Absolute,
        left: px(0),
        top: px(0),
        width: percent(100.0),
        height: percent(100.0),
        ..default()
    };
    let hidden = |node: Node| Node {
        display: Display::None,
        ..node
    };
    commands
        .spawn((
            MapRoot,
            Node {
                display: Display::None,
                ..full()
            },
            GlobalZIndex(9),
            Panel(Screen::Map),
            BackgroundColor(Color::srgb(0.03, 0.07, 0.13)),
        ))
        .with_children(|root| {
            root.spawn((
                MapCanvas,
                Node {
                    overflow: Overflow::clip(),
                    ..full()
                },
                Interaction::default(),
                FocusPolicy::Block,
            ))
            .with_children(|canvas| {
                for copy in -1..=1 {
                    canvas.spawn((BaseCopy(copy), MaterialNode(base.clone()), hidden(full())));
                }
                canvas.spawn((TileLayer, full(), FocusPolicy::Pass));
                for copy in -1..=1 {
                    canvas.spawn((LayerCopy(copy), ImageNode::default(), hidden(full())));
                }
                canvas.spawn((LiveNode, MaterialNode(material), full(), FocusPolicy::Pass));
                canvas
                    .spawn((MarkerLayer, full(), FocusPolicy::Pass))
                    .with_children(|markers| {
                        markers.spawn((
                            MapMarker::Ship,
                            ImageNode::new(icons.craft.clone()),
                            hidden(Node {
                                position_type: PositionType::Absolute,
                                width: px(18),
                                height: px(18),
                                ..default()
                            }),
                        ));
                        markers.spawn((
                            MapMarker::Player,
                            ImageNode::new(icons.arrow.clone()),
                            hidden(Node {
                                position_type: PositionType::Absolute,
                                width: px(34),
                                height: px(34),
                                ..default()
                            }),
                        ));
                    });
            });
            legend(root, &layers);
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: px(12),
                    bottom: px(12),
                    padding: UiRect::axes(px(10), px(6)),
                    border: UiRect::all(px(1)),
                    ..default()
                },
                BorderColor::all(EDGE),
                BackgroundColor(PANEL_FILL),
            ))
            .with_children(|corner| {
                corner.spawn((
                    MapReadout,
                    Text::new(""),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(INK),
                ));
            });
        });
    commands.insert_resource(icons);
}

fn legend(root: &mut ChildSpawnerCommands, layers: &MapLayers) {
    let heading = |panel: &mut ChildSpawnerCommands, text: &str| {
        panel.spawn((
            Text::new(text.to_string()),
            TextFont {
                font_size: 11.0,
                ..default()
            },
            TextColor(MINT),
            Node {
                margin: UiRect::top(px(6)),
                ..default()
            },
        ));
    };
    let button = |row: &mut ChildSpawnerCommands, label: &str, what: MapButton| {
        row.spawn((
            Button,
            what,
            Node {
                padding: UiRect::axes(px(8), px(4)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(EDGE),
            BackgroundColor(Color::srgba(0.06, 0.13, 0.15, 0.9)),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(INK),
            ));
        });
    };
    let wrap = Node {
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(4),
        row_gap: px(4),
        ..default()
    };
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(12),
            top: px(12),
            width: px(300),
            flex_direction: FlexDirection::Column,
            row_gap: px(5),
            padding: UiRect::all(px(12)),
            border: UiRect::all(px(1)),
            ..default()
        },
        BorderColor::all(EDGE),
        BackgroundColor(PANEL_FILL),
        FocusPolicy::Block,
        Interaction::default(),
    ))
    .with_children(|panel| {
        panel.spawn(menu::title("MAP"));
        panel.spawn((
            Text::new("M close  Esc close  wheel zoom  drag pan  Home find me"),
            TextFont {
                font_size: 10.0,
                ..default()
            },
            TextColor(INK),
        ));
        heading(panel, "OVERLAY");
        panel.spawn(wrap.clone()).with_children(|row| {
            button(row, "The planet", MapButton::Overlay(MapOverlay::None));
            for (index, layer) in layers.rasters.iter().enumerate() {
                button(
                    row,
                    layer.name,
                    MapButton::Overlay(MapOverlay::Layer(index)),
                );
            }
        });
        heading(panel, "WEATHER, AT THE HOUR ON THE CLOCK");
        panel.spawn(wrap.clone()).with_children(|row| {
            for overlay in Overlay::ALL {
                button(
                    row,
                    overlay.name(),
                    MapButton::Overlay(MapOverlay::Weather(overlay)),
                );
            }
            button(row, "Show on globe", MapButton::OnGlobe);
        });
        heading(panel, "ON THE MAP");
        panel.spawn(wrap).with_children(|row| {
            button(row, "Night side", MapButton::Night);
            button(row, "Clouds and rain", MapButton::Clouds);
        });
        panel.spawn((
            MapNote,
            Text::new(""),
            TextFont {
                font_size: 10.0,
                ..default()
            },
            TextColor(INK),
        ));
        panel.spawn((
            MapKey,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(3),
                ..default()
            },
        ));
    });
}

/// A line of the key's small text.
fn key_text(text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text.into()),
        TextFont {
            font_size: 10.0,
            ..default()
        },
        TextColor(INK),
    )
}

/// The key's labels for a weather overlay's two ends and middle: the rain's
/// negative end is snow, as the mockup labels it.
fn ramp_labels(overlay: Overlay) -> [String; 3] {
    let (low, high) = overlay.range();
    let label = |v: f32| {
        let text = if v.abs() < 1.0 && v != 0.0 {
            format!("{v:.2}")
        } else {
            format!("{}", v.round())
        };
        if overlay == Overlay::Rain && v < 0.0 {
            format!("{} snow", text.trim_start_matches('-'))
        } else {
            text
        }
    };
    [label(low), label((low + high) / 2.0), label(high)]
}

/// Fill the colour key for the chosen overlay: a swatch and a share of the
/// land for each row of a layer's key, counted off the base once it is
/// built; a weather overlay's ramp as a bar with its range under it; nothing
/// for the planet.
pub fn paint_key(
    mut commands: Commands,
    choice: Res<MapChoice>,
    layers: Res<MapLayers>,
    raster: Res<MapRaster>,
    keys: Query<Entity, With<MapKey>>,
    mut painted: Local<Option<(MapOverlay, bool)>>,
    mut shares: Local<HashMap<usize, Vec<f32>>>,
) {
    let counted = raster.base_texels.is_some();
    let now = (choice.overlay, counted);
    if *painted == Some(now) {
        return;
    }
    let Ok(key) = keys.single() else {
        return;
    };
    *painted = Some(now);
    commands.entity(key).despawn_related::<Children>();
    match choice.overlay {
        MapOverlay::None => {}
        MapOverlay::Layer(index) => {
            let Some(layer) = layers.rasters.get(index) else {
                return;
            };
            let share = match (&raster.base_texels, shares.get(&index)) {
                (_, Some(done)) => Some(done.clone()),
                (Some(texels), None) => {
                    let (width, _) = raster::level_size(BASE);
                    let done = raster::layer_shares(layer, texels, width);
                    shares.insert(index, done.clone());
                    Some(done)
                }
                (None, None) => None,
            };
            commands.entity(key).with_children(|rows| {
                for (row, (name, [r, g, b])) in layer.key.iter().enumerate() {
                    rows.spawn(Node {
                        column_gap: px(6),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|line| {
                        line.spawn((
                            Node {
                                width: px(14),
                                height: px(10),
                                border: UiRect::all(px(1)),
                                ..default()
                            },
                            BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.4)),
                            BackgroundColor(Color::srgb_u8(*r, *g, *b)),
                        ));
                        line.spawn((
                            key_text(*name),
                            Node {
                                width: px(90),
                                ..default()
                            },
                        ));
                        if let Some(share) = share.as_ref().and_then(|s| s.get(row)) {
                            line.spawn(key_text(format!("{:.0}%", share * 100.0)));
                        }
                    });
                }
            });
        }
        MapOverlay::Weather(overlay) => {
            let stops = RAMPS[overlay.ramp().index()]
                .iter()
                .map(|&[r, g, b]| ColorStop::auto(Color::srgb(r, g, b)))
                .collect();
            let [low, middle, high] = ramp_labels(overlay);
            commands.entity(key).with_children(|rows| {
                rows.spawn((
                    Node {
                        height: px(10),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BorderColor::all(EDGE),
                    BackgroundGradient::from(LinearGradient::to_right(stops)),
                ));
                rows.spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|line| {
                    line.spawn(key_text(low));
                    line.spawn(key_text(middle));
                    line.spawn(key_text(high));
                });
            });
        }
    }
}

/// The legend's clicks.
pub fn press(
    screen: Res<Screen>,
    buttons: Query<(&Interaction, &MapButton), Changed<Interaction>>,
    mut choice: ResMut<MapChoice>,
) {
    if *screen != Screen::Map {
        return;
    }
    for (interaction, button) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *button {
            MapButton::Overlay(overlay) => {
                choice.overlay = if choice.overlay == overlay && overlay != MapOverlay::None {
                    MapOverlay::None
                } else {
                    overlay
                };
            }
            MapButton::Night => choice.night = !choice.night,
            MapButton::Clouds => choice.clouds = !choice.clouds,
            MapButton::OnGlobe => choice.on_globe = !choice.on_globe,
        }
    }
}

/// "Show on globe" sets the globe's overlay to the map's: one mode, set from
/// the legend (decision 6). Only a change to the switch or to the map's
/// weather overlay writes it, so the launch's `--overlay` and the pause
/// panel's row keep theirs until the map is asked.
pub fn to_globe(
    choice: Res<MapChoice>,
    mut mode: ResMut<OverlayMode>,
    mut last: Local<(bool, Option<Overlay>)>,
) {
    let now = (choice.on_globe, choice.overlay.weather());
    if now == *last {
        return;
    }
    let was_on = last.0;
    *last = now;
    if !now.0 && !was_on {
        return;
    }
    let want = if now.0 { now.1 } else { None };
    if mode.0 != want {
        mode.0 = want;
    }
}

/// The legend shows what is chosen.
pub fn paint_legend(
    choice: Res<MapChoice>,
    layers: Res<MapLayers>,
    mut buttons: Query<(&MapButton, &mut BackgroundColor, &mut BorderColor)>,
    mut notes: Query<&mut Text, With<MapNote>>,
) {
    if !choice.is_changed() {
        return;
    }
    for (button, mut fill, mut border) in &mut buttons {
        let on = match *button {
            MapButton::Overlay(overlay) => choice.overlay == overlay,
            MapButton::Night => choice.night,
            MapButton::Clouds => choice.clouds,
            MapButton::OnGlobe => choice.on_globe,
        };
        fill.0 = if on {
            Color::srgba(0.35, 0.28, 0.1, 0.95)
        } else {
            Color::srgba(0.06, 0.13, 0.15, 0.9)
        };
        *border = BorderColor::all(if on {
            Color::srgb(0.95, 0.75, 0.3)
        } else {
            EDGE
        });
    }
    for mut text in &mut notes {
        text.0 = match choice.overlay {
            MapOverlay::None => {
                "The planet as it looks: each place its ground's top block, shaded by \
                 relief, the sea by its depth."
                    .into()
            }
            MapOverlay::Layer(index) => layers
                .rasters
                .get(index)
                .map_or(String::new(), |l| l.note.to_string()),
            MapOverlay::Weather(overlay) => {
                let (low, high) = overlay.range();
                format!(
                    "{} now, {} to {} {}, over the greyed base.",
                    overlay.name(),
                    low,
                    high,
                    overlay.unit()
                )
            }
        };
    }
}

/// Pan, zoom and find me, while the map is open.
#[allow(clippy::too_many_arguments)]
pub fn steer(
    screen: Res<Screen>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut wheel: MessageReader<MouseWheel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    canvas: Query<&Interaction, With<MapCanvas>>,
    mut view: ResMut<MapView>,
    mut dragging: Local<Option<Vec2>>,
    player: PlayerPlace,
) {
    if *screen != Screen::Map {
        wheel.clear();
        *dragging = None;
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let size = Vec2::new(window.width(), window.height());
    let pointer = window.cursor_position();
    let over_map = canvas.iter().any(|i| *i != Interaction::None);
    let mut next = view.clamped(size);
    for event in wheel.read() {
        let steps = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 40.0,
        };
        if over_map {
            let at = pointer.unwrap_or(size / 2.0);
            next = next.zoom_about(at, size, 1.25f32.powf(steps));
        }
    }
    if mouse.just_pressed(MouseButton::Left) && over_map {
        *dragging = pointer;
    }
    if !mouse.pressed(MouseButton::Left) {
        *dragging = None;
    }
    if let (Some(from), Some(to)) = (*dragging, pointer) {
        let d = to - from;
        next.centre -= Vec2::new(d.x / next.px_per_turn, d.y / (next.px_per_turn / 2.0));
        next = next.clamped(size);
        *dragging = Some(to);
    }
    if keys.just_pressed(KeyCode::Home)
        && let Some(at) = player.get()
    {
        next.centre = geo::project(at);
        next = next.clamped(size);
    }
    if next != *view {
        *view = next;
    }
}

/// Keep the base, the tiles and the chosen layer built for the view.
#[allow(clippy::too_many_arguments)]
pub fn keep_raster(
    screen: Res<Screen>,
    sun: Res<Sun>,
    view: Res<MapView>,
    choice: Res<MapChoice>,
    layers: Res<MapLayers>,
    windows: Query<&Window, With<PrimaryWindow>>,
    save: Option<Res<WorldSave>>,
    mut raster: ResMut<MapRaster>,
    mut images: ResMut<Assets<Image>>,
) {
    if *screen != Screen::Map {
        return;
    }
    let in_place = !sun.running;
    let palette = raster
        .palette
        .get_or_insert_with(|| {
            Arc::new(Palette::load(&raster::tileset_dir()).unwrap_or_else(|e| panic!("{e}")))
        })
        .clone();
    // The base: read from the world's cache, or built and cached.
    if raster.base.is_none() {
        if raster.building.is_none() {
            let dir = save
                .as_ref()
                .and_then(|s| s.slot().map(|slot| s.root().join(&slot.id)));
            let palette = palette.clone();
            let job = move || {
                let cfg = *terrain_config();
                let cached = dir
                    .as_ref()
                    .and_then(|d| raster::load_base(d, cfg.seed, &cfg));
                let fresh = cached.is_none();
                let texels = cached.unwrap_or_else(|| raster::build_base(&cfg));
                if fresh
                    && let Some(d) = &dir
                    && let Err(error) = raster::save_base(d, cfg.seed, &texels)
                {
                    warn!("the map's base could not be cached: {error}");
                }
                let (w, h) = raster::level_size(BASE);
                let rgba =
                    raster::colour_block(&raster::bordered(&texels, w, h), BASE, 0, w, h, &palette);
                (texels, rgba, fresh)
            };
            if in_place {
                let (texels, rgba, fresh) = job();
                raster.built_fresh = fresh;
                raster.base_texels = Some(Arc::new(texels));
                let (w, h) = raster::level_size(BASE);
                raster.base = Some(images.add(map_image(w, h, rgba)));
            } else {
                raster.building = Some(AsyncComputeTaskPool::get().spawn(async move {
                    let started = std::time::Instant::now();
                    let (texels, rgba, fresh) = job();
                    info!(
                        "the map's base: {} in {:.1} s",
                        if fresh {
                            "built"
                        } else {
                            "read from the cache"
                        },
                        started.elapsed().as_secs_f32()
                    );
                    (texels, rgba)
                }));
            }
        } else if let Some(task) = raster.building.as_mut()
            && let Some((texels, rgba)) = block_on(future::poll_once(task))
        {
            raster.building = None;
            raster.base_texels = Some(Arc::new(texels));
            let (w, h) = raster::level_size(BASE);
            raster.base = Some(images.add(map_image(w, h, rgba)));
        }
    }
    // The chosen raster layer, painted from the base's texels once.
    if let MapOverlay::Layer(index) = choice.overlay
        && !raster.layers.contains_key(&index)
        && let (Some(texels), Some(layer)) = (raster.base_texels.clone(), layers.rasters.get(index))
    {
        let (w, h) = raster::level_size(BASE);
        let image = images.add(map_image(w, h, raster::paint_layer(layer, &texels)));
        raster.layers.insert(index, image);
    }
    // The tiles the view needs, at the level it needs.
    let level = view.level();
    if level == BASE {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let size = Vec2::new(window.width(), window.height());
    for key in wanted_tiles(*view, size, level)
        .into_iter()
        .map(|(key, _)| key)
    {
        if raster.tiles.contains(key) || raster.pending.contains_key(&key) {
            continue;
        }
        if in_place {
            let rgba = key.build(terrain_config(), &palette);
            let image = images.add(map_image(TILE, TILE, rgba));
            for (_, gone) in raster.tiles.insert(key, image) {
                images.remove(&gone);
            }
        } else if raster.pending.len() < TILES_BUILDING {
            let palette = palette.clone();
            raster.pending.insert(
                key,
                AsyncComputeTaskPool::get()
                    .spawn(async move { (key, key.build(terrain_config(), &palette)) }),
            );
        }
    }
    let mut done = Vec::new();
    for (key, task) in raster.pending.iter_mut() {
        if let Some(built) = block_on(future::poll_once(task)) {
            done.push((*key, built.1));
        }
    }
    for (key, rgba) in done {
        raster.pending.remove(&key);
        let image = images.add(map_image(TILE, TILE, rgba));
        for (_, gone) in raster.tiles.insert(key, image) {
            images.remove(&gone);
        }
    }
}

/// The tiles of `level` a view shows, each with the unwrapped column it is
/// drawn at, nearest the centre first.
pub fn wanted_tiles(view: MapView, size: Vec2, level: usize) -> Vec<(TileKey, i32)> {
    let (width, height) = raster::level_size(level);
    let across = (width / TILE) as i32;
    let down = (height / TILE) as i32;
    let span = view.span(size);
    let per_u = width as f32 / TILE as f32;
    let per_v = height as f32 / TILE as f32;
    let (u0, u1) = (view.centre.x - span.x / 2.0, view.centre.x + span.x / 2.0);
    let (v0, v1) = (view.centre.y - span.y / 2.0, view.centre.y + span.y / 2.0);
    let (c0, c1) = ((u0 * per_u).floor() as i32, (u1 * per_u).floor() as i32);
    let (r0, r1) = (
        ((v0 * per_v).floor() as i32).max(0),
        ((v1 * per_v).floor() as i32).min(down - 1),
    );
    let middle = Vec2::new(view.centre.x * per_u, view.centre.y * per_v);
    let mut tiles = Vec::new();
    for row in r0..=r1 {
        for column in c0..=c1 {
            tiles.push((
                TileKey {
                    level: level as u8,
                    x: column.rem_euclid(across) as u16,
                    y: row as u16,
                },
                column,
            ));
        }
    }
    tiles.sort_by(|a, b| {
        let d = |t: &(TileKey, i32)| {
            Vec2::new(t.1 as f32 + 0.5, f32::from(t.0.y) + 0.5).distance_squared(middle)
        };
        d(a).total_cmp(&d(b))
    });
    tiles
}

fn place(node: &mut Node, left: f32, top: f32, width: f32, height: f32) {
    node.left = px(left);
    node.top = px(top);
    node.width = px(width);
    node.height = px(height);
    node.position_type = PositionType::Absolute;
}

/// The tiles on screen, apart from the base's and the layer's copies.
type TileQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static TileNode,
        &'static MaterialNode<MapImageMaterial>,
        &'static mut Node,
    ),
    (Without<BaseCopy>, Without<LayerCopy>),
>;

/// Set a map image's picture and greying, touching the asset only when one
/// of them changes, so an unchanged frame uploads nothing.
fn set_image(
    materials: &mut Assets<MapImageMaterial>,
    handle: &Handle<MapImageMaterial>,
    image: &Handle<Image>,
    grey: Vec4,
) {
    let stale = materials
        .get(handle)
        .is_some_and(|m| m.image != *image || m.grey != grey);
    if stale && let Some(material) = materials.get_mut(handle) {
        material.image = image.clone();
        material.grey = grey;
    }
}

/// Place the base, the tiles and the chosen layer for the view, and grey the
/// base and tiles out while an overlay shows (decision 11).
#[allow(clippy::too_many_arguments)]
pub fn lay_out(
    mut commands: Commands,
    screen: Res<Screen>,
    view: Res<MapView>,
    choice: Res<MapChoice>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut raster: ResMut<MapRaster>,
    live: Res<MapLive>,
    mut images: ResMut<Assets<MapImageMaterial>>,
    mut bases: Query<(&BaseCopy, &mut Node), Without<LayerCopy>>,
    mut copies: Query<(&LayerCopy, &mut ImageNode, &mut Node), Without<BaseCopy>>,
    tile_layer: Query<Entity, With<TileLayer>>,
    mut tiles: TileQuery,
) {
    if *screen != Screen::Map {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let size = Vec2::new(window.width(), window.height());
    let view = view.clamped(size);
    let (w, h) = (view.px_per_turn, view.px_per_turn / 2.0);
    // Where map position (u, 0) is on screen: the copy of the map a whole
    // turn either side of the one under the centre is `u` plus or minus 1.
    let x_of = |u: f32| size.x / 2.0 + (u - view.centre.x) * w;
    let top = size.y / 2.0 - view.centre.y * h;
    let grey = grey_of(choice.overlay != MapOverlay::None);
    if let Some(base) = &raster.base {
        set_image(&mut images, &live.base, base, grey);
        for (copy, mut node) in &mut bases {
            node.display = Display::Flex;
            place(&mut node, x_of(copy.0 as f32), top, w, h);
        }
    }
    let layer = match choice.overlay {
        MapOverlay::Layer(index) => raster.layers.get(&index).cloned(),
        _ => None,
    };
    for (copy, mut image, mut node) in &mut copies {
        match &layer {
            Some(handle) => {
                image.image = handle.clone();
                node.display = Display::Flex;
                place(&mut node, x_of(copy.0 as f32), top, w, h);
            }
            None => node.display = Display::None,
        }
    }
    // The finer tiles: one node for each wanted tile that is built.
    let level = view.level();
    let wanted = if level == BASE {
        Vec::new()
    } else {
        wanted_tiles(view, size, level)
    };
    let (lw, lh) = raster::level_size(level);
    let side_x = TILE as f32 / lw as f32 * view.px_per_turn;
    let side_y = TILE as f32 / lh as f32 * view.px_per_turn / 2.0;
    let mut shown: HashMap<(TileKey, i32), Entity> = HashMap::new();
    for (entity, tile, _, _) in &tiles {
        if wanted.contains(&(tile.key, tile.column))
            && !shown.contains_key(&(tile.key, tile.column))
        {
            shown.insert((tile.key, tile.column), entity);
        } else {
            commands.entity(entity).despawn();
        }
    }
    let Ok(parent) = tile_layer.single() else {
        return;
    };
    for (key, column) in wanted {
        let Some(handle) = raster.tiles.get(key).cloned() else {
            continue;
        };
        // Half a pixel over each edge, so rounding never leaves a hairline
        // of the base between two tiles.
        let at_left = x_of(column as f32 * TILE as f32 / lw as f32);
        let at_top = top + f32::from(key.y) * side_y;
        match shown.get(&(key, column)) {
            Some(&entity) => {
                if let Ok((_, _, material, mut node)) = tiles.get_mut(entity) {
                    set_image(&mut images, &material.0, &handle, grey);
                    place(
                        &mut node,
                        at_left - 0.5,
                        at_top - 0.5,
                        side_x + 1.0,
                        side_y + 1.0,
                    );
                }
            }
            None => {
                let mut node = Node::default();
                place(
                    &mut node,
                    at_left - 0.5,
                    at_top - 0.5,
                    side_x + 1.0,
                    side_y + 1.0,
                );
                let material = images.add(MapImageMaterial {
                    image: handle,
                    grey,
                });
                let child = commands
                    .spawn((
                        TileNode { key, column },
                        MaterialNode(material),
                        node,
                        FocusPolicy::Pass,
                    ))
                    .id();
                commands.entity(parent).add_child(child);
            }
        }
    }
}

/// Keep the live layers' cubes and uniform in step with the weather, the
/// clock and the view, only while the map is open.
#[allow(clippy::too_many_arguments)]
pub fn live(
    screen: Res<Screen>,
    view: Res<MapView>,
    choice: Res<MapChoice>,
    sun: Res<Sun>,
    air: Option<Res<Air>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut live: ResMut<MapLive>,
    mut materials: ResMut<Assets<MapLiveMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if *screen != Screen::Map {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let size = Vec2::new(window.width(), window.height());
    let view = view.clamped(size);
    let span = view.span(size);
    let want = choice.overlay.weather();
    if let Some(air) = air.as_ref() {
        // The clouds, from the same maps the globe draws.
        if choice.clouds && live.cloud_generation != air.generation {
            live.cloud_generation = air.generation;
            if let Some(image) = images.get_mut(&live.cloud) {
                image.data = Some(cloud_bytes(&air.maps.cloud));
            }
        }
        // The overlay, coloured from the same texels the globe's overlay is.
        if let Some(task) = live.task.as_mut()
            && let Some((generation, overlay, bytes)) = block_on(future::poll_once(task))
        {
            live.task = None;
            live.overlay_built = Some((generation, overlay));
            if let Some(image) = images.get_mut(&live.overlay) {
                image.data = Some(bytes);
            }
        }
        if let Some(overlay) = want
            && live.overlay_built != Some((air.generation, overlay))
            && live.task.is_none()
        {
            let atmosphere = air.now.clone();
            let generation = air.generation;
            let opacity = MAP_OVERLAY_OPACITY;
            let job = move || {
                overlay_texels(&atmosphere, overlay)
                    .into_iter()
                    .flat_map(|t| overlay_rgba(overlay, t, opacity))
                    .collect::<Vec<u8>>()
            };
            if air.in_place {
                let bytes = job();
                live.overlay_built = Some((generation, overlay));
                if let Some(image) = images.get_mut(&live.overlay) {
                    image.data = Some(bytes);
                }
            } else {
                live.task = Some(
                    AsyncComputeTaskPool::get().spawn(async move { (generation, overlay, job()) }),
                );
            }
        }
    }
    let overlay_ready = want.is_some() && live.overlay_built.map(|(_, o)| o) == want;
    let sun_dir = sun.clock.sun();
    // Under an overlay the live clouds are not drawn and the night is only a
    // hint, as the mockup's `draw()` has it (decision 11): an overlay is data,
    // and cloud or night laid over it would hide what it says.
    let data = choice.overlay != MapOverlay::None;
    let night = match (choice.night, data) {
        (false, _) => 0.0,
        (true, false) => 1.0,
        (true, true) => NIGHT_UNDER_OVERLAY,
    };
    if let Some(material) = materials.get_mut(&live.material) {
        material.params = MapLiveParams {
            view: Vec4::new(view.centre.x, view.centre.y, span.x, span.y),
            sun: sun_dir.extend(night),
            layers: Vec4::new(
                if choice.clouds && !data { 1.0 } else { 0.0 },
                if overlay_ready { 1.0 } else { 0.0 },
                0.0,
                0.0,
            ),
        };
    }
}

/// The markers and the readout.
#[allow(clippy::too_many_arguments)]
pub fn markers(
    mut commands: Commands,
    screen: Res<Screen>,
    view: Res<MapView>,
    icons: Res<MarkerIcons>,
    windows: Query<&Window, With<PrimaryWindow>>,
    player: PlayerPlace,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    ship: Query<&Position, With<PilotShip>>,
    crafts: Query<(Entity, &Vehicle)>,
    layer: Query<Entity, With<MarkerLayer>>,
    mut marks: Query<(Entity, &MapMarker, &mut Node, &mut UiTransform)>,
    mut readout: Query<&mut Text, With<MapReadout>>,
) {
    if *screen != Screen::Map {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let size = Vec2::new(window.width(), window.height());
    let view = view.clamped(size);
    let player = player.get();
    let forward = cameras
        .iter()
        .find(|(camera, _)| camera.is_active)
        .map(|(_, t)| t.forward().as_vec3());
    let ship_at = ship.iter().next().map(|p| p.0);
    let mut seen_crafts = Vec::new();
    for (entity, mark, mut node, mut turn) in &mut marks {
        let (at, heading) = match *mark {
            MapMarker::Player => match player {
                Some(up) => (Some(up), forward.map(|f| geo::heading(up, f))),
                None => (None, None),
            },
            MapMarker::Ship => (ship_at, None),
            MapMarker::Craft(craft) => {
                seen_crafts.push(craft);
                match crafts.get(craft) {
                    Ok((_, vehicle)) => (Some(vehicle.craft.body.position.as_vec3()), None),
                    Err(_) => {
                        commands.entity(entity).despawn();
                        continue;
                    }
                }
            }
        };
        let Some(at) = at else {
            node.display = Display::None;
            continue;
        };
        let pixel = view.to_screen(geo::project(at), size);
        let (w, h) = match (node.width, node.height) {
            (Val::Px(w), Val::Px(h)) => (w, h),
            _ => (20.0, 20.0),
        };
        node.left = px(pixel.x - w / 2.0);
        node.top = px(pixel.y - h / 2.0);
        node.display = Display::Flex;
        if let Some(heading) = heading {
            turn.rotation = Rot2::radians(heading);
        }
    }
    if let Ok(parent) = layer.single() {
        for (entity, _) in &crafts {
            if !seen_crafts.contains(&entity) {
                let child = commands
                    .spawn((
                        MapMarker::Craft(entity),
                        ImageNode::new(icons.craft.clone()),
                        Node {
                            position_type: PositionType::Absolute,
                            width: px(14),
                            height: px(14),
                            display: Display::None,
                            ..default()
                        },
                    ))
                    .id();
                commands.entity(parent).add_child(child);
            }
        }
    }
    if let Ok(mut text) = readout.single_mut() {
        let under = window
            .cursor_position()
            .map(|p| view.to_map(p, size))
            .filter(|uv| (0.0..=1.0).contains(&uv.y))
            .map(|uv| geo::lat_lon(geo::unproject(uv)).degrees());
        let place = |(lat, lon): (f32, f32)| {
            format!(
                "{:.2}{} {:.2}{}",
                lat.abs(),
                if lat >= 0.0 { "N" } else { "S" },
                lon.abs(),
                if lon >= 0.0 { "E" } else { "W" }
            )
        };
        let mut line = format!("{:.1} m a pixel", view.metres_per_pixel());
        if let Some(at) = under {
            line = format!("{}   {line}", place(at));
        } else if let Some(p) = player {
            line = format!("you {}   {line}", place(geo::lat_lon(p).degrees()));
        }
        text.0 = line;
    }
}

/// The launch's view: `--map-mpp`, and `--map-at` or the player. A launch's
/// first frames have the ship but not yet the walker, so the view follows the
/// player until the walker is out, or for half a second in a ship.
pub fn first_view(
    launch: Res<MapLaunch>,
    mut view: ResMut<MapView>,
    player: PlayerPlace,
    mut frames: Local<u32>,
    screen: Res<Screen>,
) {
    if *frames > 30 || *screen != Screen::Map {
        return;
    }
    *frames += 1;
    let centre = match launch.at {
        Some((lat, lon)) => Some(geo::direction(LatLon {
            lat: lat.to_radians(),
            lon: lon.to_radians(),
        })),
        None => player.get(),
    };
    let Some(centre) = centre else {
        return;
    };
    if player.walking() || launch.at.is_some() {
        *frames = u32::MAX;
    }
    view.centre = geo::project(centre);
    if let Some(metres) = launch.metres_per_pixel {
        *view = MapView::at_metres_per_pixel(view.centre, metres);
    }
}

/// The map's plugin: its material, its resources and its systems.
pub struct MapScreenPlugin;

impl Plugin for MapScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<MapLiveMaterial>::default())
            .add_plugins(UiMaterialPlugin::<MapImageMaterial>::default())
            .init_resource::<MapView>()
            .init_resource::<MapRaster>()
            .init_resource::<MapLayers>()
            .init_resource::<MapLaunch>()
            .add_systems(PreStartup, |mut layers: ResMut<MapLayers>| {
                layers.add(biomes_layer());
            })
            .add_systems(Startup, spawn)
            .add_systems(
                Update,
                (
                    first_view,
                    steer,
                    press,
                    to_globe,
                    paint_legend,
                    keep_raster,
                    paint_key,
                    lay_out,
                    live,
                    markers,
                )
                    .chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Vec2 = Vec2::new(1440.0, 900.0);

    /// Zoomed all the way out the whole planet is on screen; zoomed all the
    /// way in a cell is more than a pixel (the spec's two scenarios).
    #[test]
    fn the_map_shows_the_planet_and_zooms_to_the_cell() {
        let out = MapView {
            centre: Vec2::new(0.3, 0.2),
            px_per_turn: 1.0,
        }
        .clamped(SCREEN);
        assert!(out.px_per_turn <= SCREEN.x && out.px_per_turn / 2.0 <= SCREEN.y);
        assert_eq!(out.centre.y, 0.5, "both poles on screen");
        let span = out.span(SCREEN);
        assert!(span.x >= 1.0 && span.y >= 1.0, "{span}");
        let near = MapView {
            centre: Vec2::splat(0.5),
            px_per_turn: 1e9,
        }
        .clamped(SCREEN);
        let cell_px = 2.833 / near.metres_per_pixel();
        assert!(cell_px >= 1.0, "a cell is {cell_px} pixels");
        assert_eq!(near.level(), LEVELS.len() - 1);
    }

    /// Zooming about a pixel keeps the place under it, and a screen pixel
    /// goes to the map and back.
    #[test]
    fn zooming_keeps_the_place_under_the_pointer() {
        let view = MapView {
            centre: Vec2::new(0.42, 0.37),
            px_per_turn: 20_000.0,
        };
        let at = Vec2::new(300.0, 700.0);
        let under = view.to_map(at, SCREEN);
        let zoomed = view.zoom_about(at, SCREEN, 1.7);
        assert!(zoomed.to_map(at, SCREEN).distance(under) < 1e-5);
        assert!(view.to_screen(under, SCREEN).distance(at) < 1e-2);
        // Across the antimeridian: a place just east of it, seen from just
        // west, lands to the right of the centre, not a whole turn away.
        let west = MapView {
            centre: Vec2::new(0.999, 0.5),
            px_per_turn: 20_000.0,
        };
        let east = west.to_screen(Vec2::new(0.001, 0.5), SCREEN);
        assert!(
            east.x > SCREEN.x / 2.0 && east.x < SCREEN.x / 2.0 + 60.0,
            "{east}"
        );
    }

    /// The view asks for the tiles it shows, at the level it needs, the one
    /// under the centre first, across the antimeridian too.
    #[test]
    fn the_view_asks_for_the_tiles_it_shows() {
        let finest = LEVELS.len() - 1;
        let view = MapView {
            // Off the row boundary at the equator, so one tile is nearest.
            centre: Vec2::new(0.0005, 0.52),
            px_per_turn: LEVELS[finest] as f32,
        };
        assert_eq!(view.level(), finest);
        let tiles = wanted_tiles(view, SCREEN, finest);
        assert_eq!(tiles[0].0, TileKey::under(finest, view.centre));
        let across = (LEVELS[finest] / TILE) as u16;
        assert!(
            tiles.iter().any(|(k, c)| k.x == across - 1 && *c == -1),
            "the far side of the antimeridian"
        );
        assert!(tiles.len() <= 8 * 5, "{} tiles", tiles.len());
        // Zoomed out to the base there are none to ask for.
        let wide = MapView {
            centre: Vec2::splat(0.5),
            px_per_turn: 1500.0,
        };
        assert_eq!(wide.level(), BASE);
    }

    /// M opens the map from the world, centred on the player, and closes it;
    /// while it is open the menu holds the pointer, which is what the walker
    /// and the pilot read to take no input (`controls::MenuOpen`), and the
    /// clock keeps running (task 3.5).
    #[test]
    fn m_opens_and_closes_the_map_and_the_world_runs_behind_it() {
        use bevy::time::TimeUpdateStrategy;
        use pbd_app::controls::MenuOpen;
        use pbd_core::daylight::Clock;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(250),
            ))
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(Screen::Playing)
            .init_resource::<MapView>()
            .init_resource::<MenuOpen>()
            .init_resource::<menu::SaveIndex>()
            .init_resource::<menu::NameField>()
            .insert_resource(WalkingReadout {
                active: true,
                ..default()
            })
            .insert_resource(Sun {
                clock: Clock::at_hour(9.0),
                running: true,
            })
            .add_systems(Update, (open, menu::paint, pbd_app::sky::run_clock).chain());
        let east = Vec3::new(0.0, 0.0, PLANET_RADIUS);
        app.world_mut().spawn((Walker, Position(east)));
        let tap_m = |app: &mut App| {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::KeyM);
            keys.clear();
            keys.press(KeyCode::KeyM);
            app.update();
        };
        app.update();
        tap_m(&mut app);
        assert_eq!(*app.world().resource::<Screen>(), Screen::Map);
        assert!(
            app.world().resource::<MenuOpen>().0,
            "the map holds the pointer"
        );
        let centre = app.world().resource::<MapView>().centre;
        assert!(
            centre.distance(geo::project(east)) < 1e-6,
            "centred on the player"
        );
        let before = app.world().resource::<Sun>().clock.seconds;
        for _ in 0..8 {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
            app.update();
        }
        let after = app.world().resource::<Sun>().clock.seconds;
        assert!(after > before + 1.0, "the clock ran: {before} to {after}");
        tap_m(&mut app);
        assert_eq!(*app.world().resource::<Screen>(), Screen::Playing);
        assert!(!app.world().resource::<MenuOpen>().0, "and gives it back");
    }

    /// A parked ship's marker stays where it was parked while the player
    /// walks away, and the player's own marker follows them (task 4.1).
    #[test]
    fn a_parked_ships_marker_stays_put_while_the_player_walks_away() {
        let mut app = App::new();
        app.insert_resource(Screen::Map)
            .insert_resource(MapView {
                centre: Vec2::new(0.5, 0.5),
                px_per_turn: 60_000.0,
            })
            .insert_resource(WalkingReadout {
                active: true,
                ..default()
            })
            .insert_resource(MarkerIcons {
                arrow: Handle::default(),
                craft: Handle::default(),
            })
            .add_systems(Update, markers);
        app.world_mut().spawn((
            Window {
                resolution: (1440, 900).into(),
                ..default()
            },
            PrimaryWindow,
        ));
        let parked = Vec3::new(PLANET_RADIUS, 0.0, 0.0);
        let walker = app
            .world_mut()
            .spawn((Walker, Position(parked + Vec3::new(0.0, 0.0, 3.0))))
            .id();
        app.world_mut().spawn((PilotShip, Position(parked)));
        let node = || Node {
            position_type: PositionType::Absolute,
            width: px(20),
            height: px(20),
            display: Display::None,
            ..default()
        };
        let player = app.world_mut().spawn((MapMarker::Player, node())).id();
        let ship = app.world_mut().spawn((MapMarker::Ship, node())).id();
        app.world_mut().spawn(MarkerLayer);
        let at = |app: &App, e: Entity| {
            let n = app.world().get::<Node>(e).expect("a marker");
            assert_eq!(n.display, Display::Flex, "shown");
            match (n.left, n.top) {
                (Val::Px(x), Val::Px(y)) => Vec2::new(x, y),
                other => panic!("placed in pixels: {other:?}"),
            }
        };
        app.update();
        let (ship0, player0) = (at(&app, ship), at(&app, player));
        // Walk 60 m east.
        app.world_mut().get_mut::<Position>(walker).unwrap().0 =
            (parked + Vec3::new(0.0, 0.0, 60.0)).normalize() * PLANET_RADIUS;
        app.update();
        assert_eq!(at(&app, ship), ship0, "the ship stays where it was parked");
        let moved = at(&app, player) - player0;
        assert!(moved.x > 50.0, "the player moves east on the map: {moved}");
    }

    /// A layer added through the registry from outside the map's code gets
    /// its button in the legend (task 5.1, decision 7).
    #[test]
    fn a_registered_layer_appears_in_the_legend() {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>()
            .init_resource::<Assets<MapLiveMaterial>>()
            .init_resource::<Assets<MapImageMaterial>>()
            .init_resource::<MapLayers>()
            .add_systems(Startup, spawn);
        app.world_mut()
            .resource_mut::<MapLayers>()
            .add(RasterLayer {
                name: "Test layer",
                note: "added by a test",
                paint: |_| [0; 4],
                key: &[],
                class: |_| None,
            });
        app.update();
        let mut buttons = app.world_mut().query::<(&MapButton, &Children)>();
        let mut texts = Vec::new();
        for (button, children) in buttons.iter(app.world()) {
            if *button == MapButton::Overlay(MapOverlay::Layer(0)) {
                for child in children.iter() {
                    if let Some(text) = app.world().get::<Text>(child) {
                        texts.push(text.0.clone());
                    }
                }
            }
        }
        assert_eq!(texts, vec!["Test layer".to_string()]);
    }

    /// The legend's weather buttons are one group, and "Show on globe" puts
    /// the chosen one on the 3D globe through `OverlayMode`, as M's cycling
    /// used to (task 5.2).
    #[test]
    fn the_legend_sets_the_globes_overlay() {
        let mut app = App::new();
        app.insert_resource(Screen::Map)
            .init_resource::<MapChoice>()
            .init_resource::<OverlayMode>()
            .add_systems(Update, (press, to_globe).chain());
        let wind = app
            .world_mut()
            .spawn((
                MapButton::Overlay(MapOverlay::Weather(Overlay::Wind)),
                Interaction::Pressed,
            ))
            .id();
        let globe = app
            .world_mut()
            .spawn((MapButton::OnGlobe, Interaction::None))
            .id();
        app.update();
        assert_eq!(
            app.world().resource::<MapChoice>().overlay,
            MapOverlay::Weather(Overlay::Wind)
        );
        assert_eq!(
            app.world().resource::<OverlayMode>().0,
            None,
            "not on the globe yet"
        );
        *app.world_mut().get_mut::<Interaction>(globe).unwrap() = Interaction::Pressed;
        app.update();
        assert_eq!(app.world().resource::<OverlayMode>().0, Some(Overlay::Wind));
        // Choosing it again turns it off, on the map and the globe alike.
        *app.world_mut().get_mut::<Interaction>(wind).unwrap() = Interaction::None;
        app.update();
        *app.world_mut().get_mut::<Interaction>(wind).unwrap() = Interaction::Pressed;
        app.update();
        assert_eq!(
            app.world().resource::<MapChoice>().overlay,
            MapOverlay::None
        );
        assert_eq!(app.world().resource::<OverlayMode>().0, None);
    }

    /// The map leaves a globe overlay set elsewhere (the launch's
    /// `--overlay`, the pause panel's row) alone until its own switch is used.
    #[test]
    fn the_map_leaves_the_globes_overlay_alone_until_asked() {
        let mut app = App::new();
        app.insert_resource(Screen::Map)
            .init_resource::<MapChoice>()
            .insert_resource(OverlayMode(Some(Overlay::Cloud)))
            .add_systems(Update, (press, to_globe).chain());
        app.update();
        app.world_mut().resource_mut::<MapChoice>().overlay = MapOverlay::Weather(Overlay::Wind);
        app.update();
        assert_eq!(
            app.world().resource::<OverlayMode>().0,
            Some(Overlay::Cloud)
        );
    }

    /// A map app with the screen built and open: the systems that draw the
    /// layers and the key, a window to measure, and no GPU.
    fn drawn_map() -> App {
        use pbd_core::daylight::Clock;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<MapLiveMaterial>>()
            .init_resource::<Assets<MapImageMaterial>>()
            .insert_resource(Screen::Map)
            .init_resource::<MapView>()
            .init_resource::<MapChoice>()
            .init_resource::<MapRaster>()
            .init_resource::<MapLayers>()
            .insert_resource(Sun {
                clock: Clock::at_hour(12.0),
                running: false,
            })
            .add_systems(PreStartup, |mut layers: ResMut<MapLayers>| {
                layers.add(biomes_layer());
            })
            .add_systems(Startup, spawn)
            .add_systems(Update, (paint_key, live).chain());
        app.world_mut().spawn((Window::default(), PrimaryWindow));
        app.update();
        app
    }

    fn live_params(app: &App) -> MapLiveParams {
        let handle = app.world().resource::<MapLive>().material.clone();
        app.world()
            .resource::<Assets<MapLiveMaterial>>()
            .get(&handle)
            .expect("the live material")
            .params
    }

    /// Every text under an entity, depth first.
    fn texts_under(app: &App, root: Entity) -> Vec<String> {
        let world = app.world();
        let mut out = Vec::new();
        let mut stack = vec![root];
        while let Some(entity) = stack.pop() {
            if let Some(text) = world.get::<Text>(entity) {
                out.push(text.0.clone());
            }
            if let Some(children) = world.get::<Children>(entity) {
                stack.extend(children.iter().rev());
            }
        }
        out
    }

    /// Under an overlay the live clouds are not drawn and the night side is
    /// drawn at 0.3 of itself, as the mockup draws them (decision 11); over
    /// the planet they are drawn in full.
    #[test]
    fn under_an_overlay_the_clouds_hide_and_the_night_dims() {
        let mut app = drawn_map();
        {
            let mut choice = app.world_mut().resource_mut::<MapChoice>();
            choice.night = true;
            choice.clouds = true;
        }
        app.update();
        let planet = live_params(&app);
        assert_eq!(planet.layers.x, 1.0, "clouds over the planet");
        assert_eq!(planet.sun.w, 1.0, "and the night in full");
        for overlay in [MapOverlay::Weather(Overlay::Rain), MapOverlay::Layer(0)] {
            app.world_mut().resource_mut::<MapChoice>().overlay = overlay;
            app.update();
            let data = live_params(&app);
            assert_eq!(data.layers.x, 0.0, "no clouds over {overlay:?}");
            assert_eq!(
                data.sun.w, NIGHT_UNDER_OVERLAY,
                "a dim night over {overlay:?}"
            );
        }
    }

    /// The key shows a swatch per biome for the biome layer, and the rain's
    /// ramp with its range, snow at the negative end, as the mockup's legend
    /// does (decision 11); nothing over the planet.
    #[test]
    fn the_key_shows_the_biomes_and_the_rains_range() {
        let mut app = drawn_map();
        let key = app
            .world_mut()
            .query_filtered::<Entity, With<MapKey>>()
            .single(app.world())
            .expect("the key");
        assert!(texts_under(&app, key).is_empty(), "nothing over the planet");
        app.world_mut().resource_mut::<MapChoice>().overlay = MapOverlay::Layer(0);
        app.update();
        let names: Vec<String> = raster::BIOME_KEY
            .iter()
            .map(|(n, _)| n.to_string())
            .collect();
        assert_eq!(texts_under(&app, key), names, "a row per biome, in order");
        app.world_mut().resource_mut::<MapChoice>().overlay = MapOverlay::Weather(Overlay::Rain);
        app.update();
        assert_eq!(texts_under(&app, key), ["10 snow", "0", "10"]);
        let bar = app
            .world()
            .get::<Children>(key)
            .and_then(|c| c.first().copied())
            .expect("the ramp bar");
        assert!(app.world().get::<BackgroundGradient>(bar).is_some());
    }

    /// The clouds' bytes carry the cover and split the precipitation into
    /// rain and snow.
    #[test]
    fn the_clouds_split_rain_from_snow() {
        let bytes = cloud_bytes(&[[1.0, 0.0, 8.0, 0.0], [0.5, 0.0, -4.0, 0.0]]);
        assert_eq!(&bytes[..4], &[255, 255, 0, 255]);
        assert_eq!(&bytes[4..], &[128, 0, 128, 255]);
    }
}
