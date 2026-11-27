mod arrival;
mod boat;
mod breeding;
mod build_mode;
mod compass_bar;
mod construction;
mod cpu_times;
mod cracks;
mod craft;
mod digging;
mod drops_view;
mod equipment;
mod eye_path;
mod farm;
mod fire_play;
mod forge;
mod frame_graph;
mod fruit;
mod furniture;
mod glow_plants;
mod guide;
mod hud;
mod kitchen;
mod map_herds;
mod map_screen;
mod map_sites;
mod menu;
mod mission_tree;
mod missions;
mod overlay_ui;
mod pack;
mod path_picker;
mod planner;
mod repro;
mod scene;
mod scroll_list;
mod sea_gather;
mod showcase;
mod slots;
mod sounds;
mod survival_kit;
// Survival's HUD, death, drinking and wear (`survival-mode`).
mod survival_panels;
mod survival_places;
mod survival_play;
mod talk;
mod time_ui;
mod toasts;
mod water;
mod weather_ui;

use avian3d::prelude::*;
use bevy::{
    app::AppExit,
    asset::AssetPlugin,
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
    render::render_resource::TextureUsages,
    render::view::{
        Hdr,
        screenshot::{Screenshot, ScreenshotCaptured},
    },
    time::TimeUpdateStrategy,
    window::PresentMode,
};
use pbd_app::{
    CelestialScene, FIXED_HZ, PaleBlueDotPlugin, PhysicsFrame,
    config::ConfigPlugin,
    flight_view::{FlightViewConfig, FlightViewPlugin, FlyMode, TourProgress},
    planet::{
        FINEST_LEVEL, PLANET_RADIUS, PlanetPlugin, surface_code, surface_height, terrain_config,
        terrain_radius, tile_width_m,
    },
    saves::{self, Pose, WorldSave},
    sky::SkyPlugin,
    walking::{EYE_HEIGHT, RestoredPose, WalkingConfig, WalkingPlugin},
    weather::WeatherPlugin,
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Resource, Clone)]
pub struct Launch {
    pub capture: Option<PathBuf>,
    /// Sound (`audio-and-ambience`): `--no-audio`, `--audio-log` and
    /// `--audio-wav PATH`.
    pub audio: pbd_app::audio::AudioLaunch,
    /// `--frame-log PATH`: a measurement instrument. Every frame's wall time,
    /// written as CSV with the fine set's state on the same clock, so a hitch
    /// can be attributed rather than guessed at (`far-side-flight`).
    pub frame_log: Option<PathBuf>,
    /// `--walk-distance METRES`: a measurement instrument. The walker sprints
    /// forward in real time, hopping when it stalls, logs `WALK_DONE` and quits
    /// once it has walked that far: a repeatable stretch of streaming for the
    /// frame log to measure.
    pub walk_distance: Option<f32>,
    /// `--stroll SECONDS`: a recording instrument. The walker walks (no
    /// sprint) straight ahead, turns round every leg and walks back, hops what
    /// it stalls on, never digs, turns a quarter after three seconds stuck,
    /// logs `STROLL_START` and `STROLL_DONE` and quits after that long: a
    /// walk up and down a town's lane for a video (`--walk --at` puts it
    /// there).
    pub stroll: Option<f32>,
    /// `--stand SECONDS`: a measurement instrument (`cascade-perf`). The
    /// walker stands where `--walk --at` put it, looking the way `--yaw` and
    /// `--pitch` say, touching nothing; logs `STAND_START` once it stands
    /// and `STAND_DONE` and quits that long after: a held view for the frame
    /// log to measure.
    pub stand: Option<f32>,
    /// `--coast-walk METRES`: a recording instrument (`coast-walk`). The
    /// walker walks (no sprint) along the shore with the sea on one side,
    /// crosses the rivers it meets, keeps off cliffs and never digs; logs
    /// `COAST_START`, `COAST_RIVER` and `COAST_DONE` and quits after that many
    /// metres along the ground (`--walk --at` puts it on the shore).
    pub coast_walk: Option<f32>,
    /// `--walk-river`: walk the river nearest the start from its source to
    /// the sea on its bank (`river-cache` design 8).
    pub walk_river: bool,
    /// `--walk-river right`: keep to the right bank.
    pub walk_river_right: bool,
    /// `--clock-runs`: `--time` sets the hour the clock starts from but does
    /// not stop it (a recording instrument: a walk from golden hour into the
    /// night).
    pub clock_runs: bool,
    /// `--stroll-leg SECONDS`: how long the stroll walks before it turns
    /// round (half the stroll when not given): a short leg paces one lane.
    pub stroll_leg: Option<f32>,
    /// `--frame-graph`: start with the frame graph shown (`F3` toggles it).
    pub frame_graph: bool,
    /// `--no-vsync`: present unpaced, a measurement instrument, so a frame's
    /// time is its work and not the display's refresh.
    pub no_vsync: bool,
    /// `--vsync`: present with vsync in a `--frame-log` run, which otherwise
    /// runs without it (owner, 2026-11-17: "to avoid confusion vsync just
    /// shouldnt run when we benchmark"), so a log's frame time is the work and
    /// never the wait for the display.
    pub vsync: bool,
    /// `--frame-latency 1|2`: how many frames the swapchain may queue
    /// (`crab-breeze-lessons` 1.1). The default is 2 (1.3): measured on the
    /// owner's desktop it took 3.6 ms off p50 in flight and on foot in the
    /// rain, for up to one frame more between input and photon. 1 stays for
    /// a player who prefers the lower latency.
    pub frame_latency: u32,
    /// `--frame-limit 0|60|120`, `--render-scale PERCENT`, `--no-auto-step`:
    /// the player's settings for this run, over the file in a plain run and
    /// over the defaults in a capture or a benchmark (`player-settings`).
    pub settings: pbd_app::settings::Overrides,
    /// `--upscale nearest`: stretch a scaled view with nearest rather than
    /// bilinear filtering, for the owner's comparison.
    pub upscale_nearest: bool,
    /// `--capture-loading`: photograph the loading screen to `--capture`'s
    /// path while the planet builds, then exit (`loading-screen`).
    pub capture_loading: bool,
    pub view: String,
    pub frames: u32,
    pub tour: bool,
    /// `--route far-side`: take off, climb, cruise round the planet and land
    /// on the far side (`far-side-flight`). Implies the tour's scripted setup.
    pub route: bool,
    /// Which route `--route` named.
    pub route_kind: pbd_app::flight_view::RouteKind,
    /// `--route clouds`: the scenic route through several separate clouds.
    pub route_broken: bool,
    pub fixed: bool,
    /// `--real-time`: a `--capture` on the real clock, after the normal
    /// warm-up, instead of fixed steps from `READY` (an instrument: what play
    /// shows, frame pacing and all, where a fixed-step capture shows what a
    /// fixed frame shows).
    pub real_time: bool,
    pub fly: bool,
    pub walk: bool,
    /// Walk mode, placed at the shoreline and holding forward, so a capture can
    /// photograph the water being entered. A walker with no input never moves.
    pub swim: bool,
    /// `--climb-out`: put the walker in the sea a tile off a bank one or two
    /// layers over the water, facing it, and hold forward once the column
    /// tier is under it, so a capture can photograph the climb out of the
    /// water (`swimming` 4). A capture instrument.
    pub climb_out: bool,
    /// `--fish`: stand on a warm shore facing the sea with the rod, and cast
    /// once, scripted, so the float, the line and the schools can be
    /// photographed and the fishery's numbers logged. A capture instrument,
    /// like `--swim`: a headless run has no hand on the mouse.
    pub fish: bool,
    /// `--fish-play`: with `--fish`, play the bite out through the mouse as
    /// a player would: click when the float goes under, then hold to reel
    /// and ease off when the tension climbs, logging every phase and the
    /// fish in the slots (`fishing-and-equipment` tasks 11). A measurement
    /// instrument.
    pub fish_play: bool,
    /// `--watch-hook`: with `--fish`, the frame is drawn from under the water
    /// beside the hook while the line is in, and the HUD and readout are
    /// hidden, so a bite can be filmed from below (`movement-smoothness` 7).
    /// A recording instrument.
    pub watch_hook: bool,
    /// `--cast`: take up the rod and cast once, from a boat's seat
    /// (`--aboard tube|raft|loon`) or on foot where the walker
    /// stands (`--at`), scripted as `--fish` casts on its shore
    /// (`small-boats` design 6). A capture instrument.
    pub cast: bool,
    /// `--rod-cue nibble|bite|hooked`: once a cast's float has been on the
    /// water a second, the rod in hand is drawn as if the line were in that
    /// phase, for a still of the rod's motion (`rod-motion` 2.5). Only the
    /// drawn rod sees it: the line, its rules and any fish do not. A capture
    /// instrument.
    pub rod_cue: Option<pbd_core::fishing::Phase>,
    /// `--load-hold`: aboard a raft, put twelve logs in its hold through the
    /// pack's own move and save (`small-boats` design 5). A capture
    /// instrument.
    pub load_hold: bool,
    /// `--carts` (`land-transport` 5.6): a hand cart and a hitched horse cart
    /// set down ahead; `--cart-pull` takes the handles, `--cart-drive` the seat.
    pub carts: Option<pbd_app::vehicles::carts::CartScript>,
    /// `--husbandry SCENE`: set a farm scene in front of the walker through
    /// the player's own paths (`wild`, `tame`, `pen` or `collect`), so the
    /// animals can be photographed (`animal-husbandry` S12). A capture
    /// instrument, like `--fish`.
    pub husbandry: Option<String>,
    /// `--life ID`: a capture instrument that puts a group of that water
    /// species a few metres in front of the eye (`ocean-environments` 7.1).
    pub life: Option<String>,
    /// `--aboard KIND` boards a craft of that kind once the fleet is in (a
    /// tube or a raft is launched for it), and `--seat` takes the
    /// seat rather than the chase view: a headless run has nobody to walk up
    /// to a craft and press F. Implies `--walk`.
    pub aboard: Option<pbd_core::vehicle::Kind>,
    /// `--pole-run`: aboard a raft, pole it up the river and back down
    /// (`vehicles::pole_run`).
    pub pole_run: bool,
    pub seat: bool,
    /// Test instrument: hold the render origin this far from the camera's
    /// cell, so the planet's centre and the eye are both away from 0 in
    /// render space (`movement-smoothness` design 6.6). Works in play.
    pub render_offset: Vec3,
    /// Capture instrument for the `shore` view: camera height above the last
    /// land cell in metres. Absent means standing eye height.
    pub height: Option<f32>,
    /// `--dig N` digs N blocks straight down from the camera on the frame the
    /// tier is ready, and `--place N` stacks N stones on the layer above the last
    /// hole. A headless run has no mouse, and a picture of a hole is the only
    /// thing that says the verb works end to end.
    pub dig: u32,
    /// `--break SECONDS` holds the use button on the block under the reticle
    /// for that long, with `--tool` in hand (the shovel if none), so a capture
    /// can photograph the cracks at a chosen stage. Implies `--walk`.
    pub break_s: Option<f32>,
    /// `--tool rod|shovel|pickaxe|axe|hand`: the tool in hand for a capture,
    /// with or without `--break`; `hand` is bare hands. A new world owns no
    /// tool, so the rig gives it the tool first, at `--tier` (stone when
    /// none follows). Implies nothing else.
    pub tool: Option<pbd_core::inventory::Tool>,
    /// `--tool hand`: bare hands.
    pub bare: bool,
    /// `--tier stone|bronze|iron|steel`, for `--tool`.
    pub tier: pbd_core::inventory::Tier,
    /// `--craft pack|pack-empty|pack-pick|pack-locked|bench [metres]`
    /// (`tool-crafting` tasks 4.2 and 5.5, a capture instrument): the pack's
    /// Craft panel (mid-craft, with nothing chosen, on a recipe, or with a
    /// locked row hovered), or a workbench that far ahead with its panel
    /// open mid-craft.
    pub craft: Option<(String, f32)>,
    /// `--place N` stacks N stones on the last hole, so a tower somebody built
    /// can be photographed wearing the stone it is made of.
    pub place: u32,
    /// `--spawn mouth` puts the spawn, and so the column tier, at the nearest
    /// cave mouth to the default spawn. Mouth patches cover a few percent of
    /// the land and the default spawn has none, so without this a walker has
    /// nothing to walk into for the first few hundred metres.
    pub spawn: Option<String>,
    /// `--at <lat> <lon>`: a new world's walker starts exactly there, in
    /// degrees, not at the level ground the start search finds, and is set
    /// back on the ground once the towns are built, so a capture can stand
    /// in a town (`cities-in-the-world`).
    pub at: Option<(f32, f32)>,
    /// Rain intensity at launch, 0..1.
    pub rain: f32,
    /// `--gale <m/s>`: the sea held at a gale (`ocean-environments` 4.3), an
    /// instrument for filming the spray.
    pub gale: Option<f32>,
    /// `--weather-at SECONDS` starts the weather field that far into its own
    /// time, so a capture can be taken under a chosen sky: the spawn is a wet
    /// meadow whose cover sits near 0.6 at launch, and the body's average is
    /// 0.18. Moves the field only, not the sun or the sea.
    pub weather_at: f32,
    /// `--open-doors` opens every door of the towns when they are built, as
    /// the player would, without writing it to the save: a capture that
    /// looks through a doorway (`cities-in-the-world` slice 2b). Doors start
    /// shut.
    pub open_doors: bool,
    /// `--no-shadows` draws no sun cascades and lights everything as if in
    /// the sun: the same build's picture without them (`sun-shadows`).
    pub no_shadows: bool,
    /// `--room-sky OPEN SHUT` sets the share of the sky a town's rooms take
    /// with a door open and with all shut (`sun-shadows` decision 7), for
    /// tuning against captures.
    pub room_sky: Option<(f32, f32)>,
    /// `--room-bounce K`: how much of the sun bounces round a room, times
    /// `field_lit.wgsl`'s `ROOM_BOUNCE`, for tuning.
    pub room_bounce: Option<f32>,
    /// `--up M` stands a capture's walker on the highest town floor within M
    /// metres of the ground at `--at`: upstairs in a house.
    pub up: f32,
    /// `--dig-ahead` digs along the camera's LOOK rather than straight down.
    /// Digging down is right for proving the verb and useless for judging the
    /// result: the walker falls into its own pit and the eye ends up inside
    /// the wall, so every capture of a hole is a screen of dirt. Digging ahead
    /// leaves them standing on the rim looking at what they made.
    pub dig_ahead: bool,
    /// `--take-trees`: a capture's instrument that marks the nearest trees
    /// picked bare and the next felled (`flora-v2` 12.5).
    pub take_trees: bool,
    /// `--pick-trees`: the same instrument picking only, felling nothing, so a
    /// picked tree is seen bare among the trees it stood with.
    pub pick_only: bool,
    /// `--pick-fruit FRAME`: a pick played by the player's own key
    /// (`flora-v2` 9.6). Once the fine set stands round the walker, the look
    /// turns onto the nearest hanging fruit in reach and F is pressed on the
    /// script's FRAME-th frame (counted from the aim), as a player would.
    pub pick_fruit: Option<u32>,
    /// `--pick-glow FRAME`: the same for the nearest glowing mushroom
    /// (`glowing-mushrooms` 2.3), F pressed by the player's own key.
    pub pick_glow: Option<u32>,
    /// `--hold <item>`: that item in the selected hotbar slot from the
    /// start, as a player holds it (`glowing-mushrooms` part B): a block as
    /// `block:torch`, a good by its code.
    pub hold: Option<String>,
    /// `--survival-kit`: one of each survival good and both new stations in
    /// the pack (`survival-mode` 4.10, a capture instrument).
    pub survival_kit: bool,
    /// `--lay-weapons <m>`: each weapon laid as a drop in a row that far
    /// ahead (`survival-mode` 4.10, a capture instrument).
    pub lay_weapons: Option<f32>,
    /// `--pitch <degrees>` starts the walker looking that far below (negative)
    /// or above the horizon. A capture that digs along the look needs to look
    /// DOWN to dig a pit the way a player does - a slanted run of cells, each
    /// taken from a different column at a different layer - and the headless
    /// walker otherwise looks dead level at the horizon.
    pub pitch: Option<f32>,
    /// `--yaw <degrees>` turns the walker's starting heading that far to the
    /// right of the default, which faces compass west (`Y x up`): 90 faces
    /// north, 180 east (`compass-bar`). With `--time` the launch log says where the sun
    /// stands from the spawn, as the yaw and pitch that would centre it, so a
    /// sky capture is aimed off the clock rather than guessed.
    pub yaw: Option<f32>,
    /// `--turn DEG_PER_S`: the walker's view turns right at this rate, a
    /// measurement instrument for the clouds' history (`cloud-ghosting`).
    pub turn: f32,
    /// `--pour N` pours N water sources where the capture camera looks, a
    /// cell apart across the view (`water-flow` E5).
    pub pour: u32,
    /// `--water-ticks N` runs N ticks of the water's rule at once after the
    /// pour, so a capture shows water that has had N seconds to move.
    pub water_ticks: u32,
    /// `--pour-ahead M`: pour straight down M metres ahead of the eye.
    pub pour_ahead: f32,
    /// `--pit DEPTH`: dig a pit that deep two cells short of the pour.
    pub pit: u32,
    /// `--pour-for SECONDS`: a measurement instrument (`water-flow` 5.3,
    /// `perf_suite.py`'s `pour`). The `--pour`/`--pit` script runs in real
    /// time without a capture, then the poured sources are stopped and
    /// started again in turn so water flows, falls and dries the whole
    /// while; logs `POUR_START` and `POUR_DONE` and quits after that long.
    pub pour_for: Option<f32>,
    /// `--water-tick-frac F` (0..1): the water drawn F of the way through the
    /// last of `--water-ticks`, and held there: the rule does not tick on
    /// its own, so a capture shows smooth water mid-tick (`water-flow` 5.3).
    pub water_tick_frac: Option<f32>,
    /// `--arrows` turns the flow-arrow overlay on (`water-flow` 3.4).
    pub arrows: bool,
    /// `--torch` puts one torch on the ground under the capture camera. A
    /// headless run has no hands, and a lamp is the one thing in this world
    /// whose whole point is what it does to a dark place.
    pub torch: bool,
    /// `--lamps [metres [across]]`: one of every light in a row across the
    /// view, that far ahead of the camera (5 m when no distance follows) and
    /// centred that far to the right (0 when none follows), for the lights'
    /// captures (`lamps-and-lanterns` task 5.2). Close up shows the lights;
    /// further out shows how far each one's light goes; a row moved aside
    /// brings its end into a close-up.
    pub lamps: bool,
    pub lamps_at: f32,
    pub lamps_across: f32,
    /// `--place-row <block,block,...> [metres]` (`placed-blocks-keep-material`,
    /// a capture instrument): one of each named block (`stone`, `dirt`, ...)
    /// placed on the ground in a row across the view, that far ahead of the
    /// camera (5 m when no distance follows), through the edit path a
    /// player's place takes, so a capture shows what a placed block is drawn
    /// as on the ground it stands on.
    pub place_row: Vec<pbd_core::terrain::Material>,
    /// `--farm [metres]` (`farming-and-smelting` task 5.1, a capture
    /// instrument): tills two rows of four cells that far ahead, sows wheat
    /// and turnip in them at staggered world times so the rows show all four
    /// stages, tills one grassy cell so its seed floats, and stands a street
    /// lantern by the field for a night shot.
    pub farm: Option<f32>,
    /// `--farm-harvest`: with `--farm`, harvests the ripe cells and picks the
    /// nearest wild plant, so the hotbar shows what came of it.
    pub farm_harvest: bool,
    /// `--farm-ripe`: with `--farm`, every crop of the field is sown ripe
    /// (a capture's instrument, `stone-and-wood-ui` 4.8: the crop the eye
    /// is on is one F harvests).
    pub farm_ripe: bool,
    /// `--hold-pointer`: a capture's instrument (`stone-and-wood-ui` 4.8).
    /// The walker holds the pointer as a player's does after a click, so the
    /// aim and the interact prompt show in a still capture, whose window
    /// never has the focus.
    pub hold_pointer: bool,
    /// `--forge [metres]`: a forge that far ahead, stocked and given two jobs,
    /// the first part done; with `--forge-panel`, its panel open.
    pub forge: Option<f32>,
    /// `--gather-around <metres>` (`gather-by-hand`, a capture instrument):
    /// once the ground is in, every stone and shrub that near the player is
    /// taken through the same path F takes one, so a still shows them gone.
    pub gather_around: Option<f32>,
    /// `--pick-wild-ago <days>` (playtest-fixes design 3, a capture
    /// instrument): once the ground round the player is in, every wild plant
    /// within 20 m is recorded as picked that many game days ago, as a pick
    /// writes it, so a still shows one gone and one grown back.
    pub pick_wild_ago: Option<f64>,
    /// `--paving [metres]` (`townsfolk` design 39 to 41, a capture
    /// instrument): a grinder and a mixer that far ahead either side, and a
    /// plan staked from beyond them along the view, its nearer half paved,
    /// through the same paths a player's stake and shovel take.
    pub paving: Option<f32>,
    /// `--road-plan <scene>` with `--paving` (`townsfolk` 14, a capture
    /// instrument): the road planner's scenes, `two` (a road by two
    /// clicks, an open start whose ghost follows the aim), `stake` (a second
    /// road from the first's stake), `erased` (that second road erased),
    /// `wide` (a three-wide road up the view).
    pub road_plan: Option<String>,
    pub forge_panel: bool,
    /// `--campfire [metres]` (`cooking` task 9.5, a capture instrument): a
    /// campfire that far ahead, stocked, flatbread done on it and a fish
    /// grilling; with `--campfire-panel`, its panel open.
    pub campfire: Option<f32>,
    pub campfire_panel: bool,
    /// `--eat <dish,dish>`: each dish given and eaten, for the effects row.
    pub eat: Vec<String>,
    /// `--missions <n>`: the starter missions capture instrument raises the
    /// first `n` steps of its scripted chain (`starter-missions` design 17).
    pub missions: Option<usize>,
    /// `--tree-select <id>`: the Missions tab opens with this mission's card
    /// in the side panel (a capture instrument, `starter-missions` TS.2).
    pub tree_select: Option<String>,
    /// `--tree-zoom <level>`: the Missions tab opens at this zoom level,
    /// 0 (farthest) to 6, on the selected card or the current mission (a
    /// capture instrument, `starter-missions` TS.3).
    pub tree_zoom: Option<usize>,
    /// `--tree-hover <id>`: zoomed out, show that mission's hover card as if
    /// the pointer were on its tile (a capture instrument).
    pub tree_hover: Option<String>,
    /// `--tree-town`: the Missions tab shows the town tree once it is open;
    /// `--town-focus <id>`: that town mission is made the one the player is
    /// on once it opens, as a pick on the tree would (capture instruments,
    /// `town-missions-tree` design 6).
    pub tree_town: bool,
    pub town_focus: Option<String>,
    /// `--mission-mode survival`: the missions follow a Survival world's
    /// tree whatever the world's own mode (a capture instrument,
    /// `survival-mode` 6.5, until worlds carry their mode).
    pub mission_mode: Option<pbd_core::missions::Mode>,
    /// `--combat <scene>`: a capture's combat scene (`survival-mode` combat
    /// C1 to C3), a measurement instrument.
    pub combat: Option<String>,
    /// `--pick-path <branch>`: once the path picker is up, that path is
    /// picked as a click on its card would (a capture instrument,
    /// `survival-mode` 6.5).
    pub pick_path: Option<String>,
    /// `--tree-jobs`: the Missions tab shows the taken jobs once the town
    /// tree is open; `--track-job`: the first job taken is tracked, as its
    /// TRACK would (capture instruments, playtest-fixes design 2).
    pub tree_jobs: bool,
    pub track_job: bool,
    /// `--no-readout`: no debug readout at the top left, as in a showcase
    /// (a capture instrument).
    pub no_readout: bool,
    /// `--time <hour>` pins the clock, 0..24, and STOPS it. A capture whose
    /// world has a day in it is a different picture every run, and a harness
    /// cannot wait six minutes for dusk.
    pub time: Option<f32>,
    /// `--day N` puts the clock on that day of the year, for a season. It
    /// pins the clock the way `--time` does.
    pub day: Option<u32>,
    /// `--overlay <name>` opens with that weather overlay showing (`wind`,
    /// `jet`, `currents`, `cloud`, `rain`, `humidity`, `sunlight`,
    /// `temperature`), as M would.
    pub overlay: Option<pbd_core::overlay::Overlay>,
    /// `--world <name>` opens that save, creating it if it is not there.
    /// Absent, an interactive run opens the one played most recently and a
    /// capture writes to no world at all.
    pub world: Option<String>,
    /// `--load <name>`: a capture instrument. A third of the way to the shot,
    /// load that save as the saves page's LOAD does, so a capture shows what
    /// a load does to a running game: an old world's planet switched in
    /// place, say (`bigger-biomes` task 2.2c). The save must already exist.
    pub load: Option<String>,
    /// `--new-world <name>`: a capture instrument (`new-world-arrival`). A
    /// third of the way to the shot, make a world of that name and enter it as
    /// the saves page's NEW WORLD does, so a capture shows what making a new
    /// world while one is open does to a running game.
    pub new_world: Option<String>,
    /// `--menu pause|settings|saves|pack|map|guide` opens that screen at startup. A headless run has
    /// no pointer and no keyboard, so a screen a player reaches with `Escape`
    /// has to be reachable by a flag or it can never be photographed.
    pub menu: Option<String>,
    /// `--settings-tab picture|weather|world|movement|input` and
    /// `--input-tab on-foot|hands|flying|vehicles|menus`: the tab and the
    /// Input sub-tab the settings open on, for a capture of each.
    pub settings_tab: u8,
    pub input_tab: u8,
    /// `--rebind SLOT=KEY` (repeatable, e.g. `on_foot:ShiftLeft=KeyE`): a
    /// rebind for this run only, so a capture, which reads no settings file,
    /// can show one and its clash. An instrument; nothing is kept.
    pub rebinds: Vec<(String, String)>,
    /// `--cursor X Y`: hold the pointer at this window position (logical px)
    /// every frame, so a capture can show what hovering shows (a slot's
    /// tooltip). An instrument for a headless run, which has no mouse.
    pub cursor: Option<Vec2>,
    /// How the map opens, for its captures: `--map-mpp <metres>` is its zoom
    /// as metres a screen pixel at the centre, `--map-overlay <name>` a
    /// weather overlay, `--map-layer <name>` a raster layer such as
    /// `biomes`, `--map-night` and `--map-clouds` those live layers, and
    /// `--map-at <lat> <lon>` the centre in degrees (the player otherwise);
    /// `--map-mark-east <metres>`, repeatable, adds a waypoint that far
    /// from the player, the first due east (a capture instrument).
    pub map: map_screen::MapLaunch,
    /// `--showcase <name>` (`step-videos` 2.1, 5.1): a bot plays
    /// `assets/showcase/<name>.ron` toward its goal through the player's
    /// own inputs, in real time, in a new world, and quits.
    pub showcase: Option<(String, showcase::file::Showcase)>,
    /// `--showcase-stills <dir>`: a showcase's `Still` acts save a PNG there.
    pub showcase_stills: Option<PathBuf>,
    /// `--showcase-from N`: start at shot N, a development aid with a
    /// `--world` that already holds the shots before it.
    pub showcase_from: usize,
    /// `--generator <v>` (`repro-capture`): the terrain generator a run with
    /// no `--world` is made by, so a repro of an older world stands on its
    /// own ground. Refused with `--world`, whose generator is its own.
    pub generator: Option<u32>,
    /// `--host <port>` (`multiplayer` design 20): play this world and let
    /// others join it on the network.
    pub host: Option<u16>,
    /// `--join <address:port>` (`multiplayer` design 20): play the world
    /// another game hosts.
    pub join: Option<std::net::SocketAddr>,
}

impl Launch {
    fn parse(args: &[String]) -> Self {
        let mut result = Self {
            capture: None,
            audio: pbd_app::audio::AudioLaunch::default(),
            frame_log: None,
            walk_distance: None,
            stroll: None,
            stand: None,
            coast_walk: None,
            walk_river: false,
            walk_river_right: false,
            clock_runs: false,
            stroll_leg: None,
            frame_graph: false,
            no_vsync: false,
            vsync: false,
            frame_latency: 2,
            settings: pbd_app::settings::Overrides::default(),
            upscale_nearest: false,
            capture_loading: false,
            view: "coast".into(),
            frames: 180,
            dig: 0,
            break_s: None,
            tool: None,
            place: 0,
            tour: false,
            route: false,
            route_kind: pbd_app::flight_view::RouteKind::FarSide,
            route_broken: false,
            fixed: false,
            real_time: false,
            fly: false,
            walk: false,
            swim: false,
            climb_out: false,
            fish: false,
            fish_play: false,
            watch_hook: false,
            cast: false,
            rod_cue: None,
            load_hold: false,
            carts: None,
            husbandry: None,
            life: None,
            aboard: None,
            pole_run: false,
            seat: false,
            render_offset: Vec3::ZERO,
            height: None,
            spawn: None,
            at: None,
            rain: 0.0,
            gale: None,
            weather_at: 0.0,
            open_doors: false,
            no_shadows: false,
            room_sky: None,
            room_bounce: None,
            up: 0.0,
            world: None,
            load: None,
            new_world: None,
            time: None,
            day: None,
            overlay: None,
            map: map_screen::MapLaunch::default(),
            showcase: None,
            showcase_stills: None,
            showcase_from: 1,
            generator: None,
            host: None,
            join: None,
            torch: false,
            pour: 0,
            water_ticks: 0,
            pour_ahead: 0.0,
            pit: 0,
            pour_for: None,
            water_tick_frac: None,
            arrows: false,
            lamps: false,
            lamps_at: 5.0,
            lamps_across: 0.0,
            place_row: Vec::new(),
            farm: None,
            farm_harvest: false,
            hold_pointer: false,
            farm_ripe: false,
            forge: None,
            gather_around: None,
            pick_wild_ago: None,
            paving: None,
            road_plan: None,
            forge_panel: false,
            bare: false,
            tier: pbd_core::inventory::Tier::Stone,
            craft: None,
            campfire: None,
            campfire_panel: false,
            eat: Vec::new(),
            missions: None,
            tree_select: None,
            tree_zoom: None,
            tree_hover: None,
            tree_town: false,
            town_focus: None,
            mission_mode: None,
            combat: None,
            pick_path: None,
            tree_jobs: false,
            track_job: false,
            no_readout: false,
            dig_ahead: false,
            take_trees: false,
            pick_only: false,
            pick_fruit: None,
            pick_glow: None,
            hold: None,
            survival_kit: false,
            lay_weapons: None,
            pitch: None,
            yaw: None,
            turn: 0.0,
            menu: None,
            settings_tab: 0,
            input_tab: 0,
            rebinds: Vec::new(),
            cursor: None,
        };
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--capture" => {
                    i += 1;
                    result.capture =
                        Some(args.get(i).expect("--capture requires a PNG path").into());
                }
                "--break" => {
                    i += 1;
                    result.break_s = Some(
                        args.get(i)
                            .and_then(|n| n.parse().ok())
                            .filter(|s: &f32| s.is_finite() && *s >= 0.0)
                            .expect("--break requires seconds"),
                    );
                    result.walk = true;
                }
                "--tool" => {
                    i += 1;
                    result.walk = true;
                    let name = args.get(i).expect("--tool requires a tool").to_lowercase();
                    // Named exactly: "axe" is inside "pickaxe", and a
                    // match on containing picked the pickaxe for it.
                    use pbd_core::inventory::Tool;
                    result.tool = Some(match name.as_str() {
                        "rod" => Tool::Rod,
                        "shovel" => Tool::Shovel,
                        "pickaxe" => Tool::Pickaxe,
                        "axe" => Tool::Axe,
                        "hand" => {
                            result.bare = true;
                            Tool::Shovel
                        }
                        _ => panic!("--tool {name}: rod, shovel, pickaxe, axe or hand"),
                    });
                }
                "--tier" => {
                    i += 1;
                    let name = args.get(i).expect("--tier requires a tier").to_lowercase();
                    result.tier = pbd_core::inventory::Tier::ALL
                        .into_iter()
                        .find(|t| t.code() == name)
                        .expect("--tier takes stone, bronze, iron or steel");
                }
                "--craft" => {
                    i += 1;
                    let scene = args.get(i).expect("--craft requires pack or bench").clone();
                    assert!(
                        matches!(
                            scene.as_str(),
                            "pack"
                                | "pack-empty"
                                | "pack-pick"
                                | "pack-locked"
                                | "bench"
                                | "bench-view"
                                | "carpenter"
                        ),
                        "--craft takes pack, pack-empty, pack-pick, pack-locked, bench, bench-view or carpenter"
                    );
                    let mut metres = 4.0;
                    if let Some(m) = args.get(i + 1).and_then(|next| next.parse::<f32>().ok()) {
                        metres = m;
                        i += 1;
                    }
                    result.craft = Some((scene, metres));
                }
                "--dig" => {
                    i += 1;
                    result.dig = args
                        .get(i)
                        .and_then(|n| n.parse().ok())
                        .expect("--dig requires a count");
                }
                "--place" => {
                    i += 1;
                    result.place = args
                        .get(i)
                        .and_then(|n| n.parse().ok())
                        .expect("--place requires a count");
                }
                "--torch" => result.torch = true,
                "--pour" => {
                    i += 1;
                    result.pour = args
                        .get(i)
                        .and_then(|n| n.parse().ok())
                        .expect("--pour requires a count");
                }
                "--water-ticks" => {
                    i += 1;
                    result.water_ticks = args
                        .get(i)
                        .and_then(|n| n.parse().ok())
                        .expect("--water-ticks requires a count");
                }
                "--arrows" => result.arrows = true,
                "--pit" => {
                    i += 1;
                    result.pit = args
                        .get(i)
                        .and_then(|n| n.parse().ok())
                        .expect("--pit requires a depth");
                }
                "--pour-for" => {
                    i += 1;
                    let seconds: f32 = args
                        .get(i)
                        .and_then(|n| n.parse().ok())
                        .expect("--pour-for requires seconds");
                    assert!(
                        seconds.is_finite() && seconds > 0.0,
                        "--pour-for takes seconds"
                    );
                    result.pour_for = Some(seconds);
                }
                "--water-tick-frac" => {
                    i += 1;
                    let frac: f32 = args
                        .get(i)
                        .and_then(|n| n.parse().ok())
                        .expect("--water-tick-frac requires a fraction");
                    assert!(
                        (0.0..=1.0).contains(&frac),
                        "--water-tick-frac takes 0 to 1"
                    );
                    result.water_tick_frac = Some(frac);
                }
                "--pour-ahead" => {
                    i += 1;
                    result.pour_ahead = args
                        .get(i)
                        .and_then(|n| n.parse().ok())
                        .expect("--pour-ahead requires metres");
                }
                "--lamps" => {
                    result.lamps = true;
                    if let Some(metres) = args.get(i + 1).and_then(|next| next.parse::<f32>().ok())
                    {
                        assert!(
                            metres.is_finite() && metres > 0.0,
                            "--lamps takes metres ahead"
                        );
                        result.lamps_at = metres;
                        i += 1;
                        if let Some(across) =
                            args.get(i + 1).and_then(|next| next.parse::<f32>().ok())
                        {
                            assert!(across.is_finite(), "--lamps takes metres across");
                            result.lamps_across = across;
                            i += 1;
                        }
                    }
                }
                "--place-row" => {
                    i += 1;
                    let names = args.get(i).expect("--place-row requires blocks");
                    result.place_row = names
                        .split(',')
                        .map(|name| {
                            pbd_core::terrain::Material::ALL
                                .into_iter()
                                .find(|&m| pbd_core::inventory::material_name(m) == name)
                                .unwrap_or_else(|| panic!("--place-row: no block named {name}"))
                        })
                        .collect();
                    if let Some(metres) = args.get(i + 1).and_then(|next| next.parse::<f32>().ok())
                    {
                        assert!(
                            metres.is_finite() && metres > 0.0,
                            "--place-row takes metres ahead"
                        );
                        result.lamps_at = metres;
                        i += 1;
                    }
                }
                "--farm" | "--forge" => {
                    let farm = args[i] == "--farm";
                    let mut metres = 6.0;
                    if let Some(m) = args.get(i + 1).and_then(|next| next.parse::<f32>().ok()) {
                        assert!(
                            m.is_finite() && m > 0.0,
                            "--farm and --forge take metres ahead"
                        );
                        metres = m;
                        i += 1;
                    }
                    if farm {
                        result.farm = Some(metres);
                    } else {
                        result.forge = Some(metres);
                    }
                }
                "--paving" => {
                    let mut metres = 6.0;
                    if let Some(m) = args.get(i + 1).and_then(|next| next.parse::<f32>().ok()) {
                        assert!(m.is_finite() && m > 0.0, "--paving takes metres ahead");
                        metres = m;
                        i += 1;
                    }
                    result.paving = Some(metres);
                }
                "--road-plan" => {
                    i += 1;
                    result.road_plan = Some(
                        args.get(i)
                            .expect("--road-plan takes a scene: two, stake, erased, wide, lamps, slope, bridge")
                            .clone(),
                    );
                }
                "--gather-around" => {
                    i += 1;
                    result.gather_around = Some(
                        args.get(i)
                            .and_then(|v| v.parse().ok())
                            .filter(|m: &f32| m.is_finite() && *m > 0.0)
                            .expect("--gather-around requires metres"),
                    );
                }
                "--farm-harvest" => result.farm_harvest = true,
                "--hold-pointer" => result.hold_pointer = true,
                "--farm-ripe" => result.farm_ripe = true,
                "--forge-panel" => result.forge_panel = true,
                "--campfire" => {
                    let mut metres = 4.0;
                    if let Some(m) = args.get(i + 1).and_then(|next| next.parse::<f32>().ok()) {
                        assert!(m.is_finite() && m > 0.0, "--campfire takes metres ahead");
                        metres = m;
                        i += 1;
                    }
                    result.campfire = Some(metres);
                }
                "--campfire-panel" => result.campfire_panel = true,
                "--eat" => {
                    i += 1;
                    let dishes = args
                        .get(i)
                        .expect("--eat takes dish codes, comma separated");
                    result.eat = dishes.split(',').map(str::to_string).collect();
                }
                "--tree-zoom" => {
                    i += 1;
                    result.tree_zoom = Some(
                        args.get(i)
                            .and_then(|v| v.parse().ok())
                            .expect("--tree-zoom takes a level, 0 to 9"),
                    );
                }
                "--tree-hover" => {
                    i += 1;
                    result.tree_hover = Some(
                        args.get(i)
                            .expect("--tree-hover takes a mission id")
                            .clone(),
                    );
                }
                "--tree-select" => {
                    i += 1;
                    result.tree_select = Some(
                        args.get(i)
                            .expect("--tree-select takes a mission id")
                            .clone(),
                    );
                }
                "--tree-town" => result.tree_town = true,
                "--tree-jobs" => result.tree_jobs = true,
                "--track-job" => result.track_job = true,
                "--no-readout" => result.no_readout = true,
                "--town-focus" => {
                    i += 1;
                    result.town_focus = Some(
                        args.get(i)
                            .expect("--town-focus takes a town mission id")
                            .clone(),
                    );
                }
                "--combat" => {
                    i += 1;
                    result.walk = true;
                    result.combat = Some(
                        args.get(i)
                            .cloned()
                            .expect("--combat takes a scene: fox-stalk, fox-warn, hound-warn, swing-hit, spear-throw or guard"),
                    );
                }
                "--mission-mode" => {
                    i += 1;
                    result.mission_mode = Some(match args.get(i).map(String::as_str) {
                        Some("survival") => pbd_core::missions::Mode::Survival,
                        Some("explorer") => pbd_core::missions::Mode::Explorer,
                        _ => panic!("--mission-mode takes survival or explorer"),
                    });
                }
                "--pick-path" => {
                    i += 1;
                    result.pick_path = Some(
                        args.get(i)
                            .expect("--pick-path takes a path's branch id")
                            .clone(),
                    );
                }
                "--missions" => {
                    i += 1;
                    result.missions = Some(
                        args.get(i)
                            .and_then(|n| n.parse().ok())
                            .expect("--missions takes how many scripted steps to raise"),
                    );
                }
                "--dig-ahead" => result.dig_ahead = true,
                "--take-trees" => result.take_trees = true,
                "--pick-fruit" => {
                    i += 1;
                    result.pick_fruit = Some(
                        args.get(i)
                            .and_then(|n| n.parse().ok())
                            .expect("--pick-fruit takes the frame to press F on"),
                    );
                }
                "--pick-wild-ago" => {
                    i += 1;
                    result.pick_wild_ago = Some(
                        args.get(i)
                            .and_then(|n| n.parse().ok())
                            .expect("--pick-wild-ago takes game days"),
                    );
                }
                "--pick-glow" => {
                    i += 1;
                    result.pick_glow = Some(
                        args.get(i)
                            .and_then(|n| n.parse().ok())
                            .expect("--pick-glow takes the frame to press F on"),
                    );
                }
                "--hold" => {
                    i += 1;
                    result.hold = Some(
                        args.get(i)
                            .cloned()
                            .expect("--hold takes an item: block:torch, glowing-mushroom"),
                    );
                }
                "--survival-kit" => result.survival_kit = true,
                "--lay-weapons" => {
                    i += 1;
                    result.lay_weapons = Some(
                        args.get(i)
                            .and_then(|m| m.parse().ok())
                            .filter(|m: &f32| m.is_finite() && *m > 0.0)
                            .expect("--lay-weapons takes metres ahead"),
                    );
                    result.walk = true;
                }
                "--pick-trees" => {
                    result.take_trees = true;
                    result.pick_only = true;
                }
                "--pitch" => {
                    i += 1;
                    let degrees: f32 = args
                        .get(i)
                        .and_then(|d| d.parse().ok())
                        .expect("--pitch requires degrees");
                    assert!(
                        (-89.0..=89.0).contains(&degrees),
                        "--pitch takes degrees in -89..89"
                    );
                    result.pitch = Some(degrees);
                }
                "--yaw" => {
                    i += 1;
                    let degrees: f32 = args
                        .get(i)
                        .and_then(|d| d.parse().ok())
                        .expect("--yaw requires degrees");
                    assert!(degrees.is_finite(), "--yaw takes finite degrees");
                    result.yaw = Some(degrees);
                }
                "--time" => {
                    i += 1;
                    let hour: f32 = args
                        .get(i)
                        .and_then(|h| h.parse().ok())
                        .expect("--time requires an hour");
                    assert!(
                        (0.0..=24.0).contains(&hour),
                        "--time takes an hour in 0..24"
                    );
                    result.time = Some(hour);
                }
                "--day" => {
                    i += 1;
                    let day: u32 = args
                        .get(i)
                        .and_then(|d| d.parse().ok())
                        .expect("--day requires a day number");
                    result.day = Some(day);
                }
                "--host" => {
                    i += 1;
                    result.host = Some(
                        args.get(i)
                            .and_then(|v| v.parse().ok())
                            .expect("--host requires a port"),
                    );
                }
                "--join" => {
                    i += 1;
                    result.join = Some(
                        args.get(i)
                            .and_then(|v| v.parse().ok())
                            .expect("--join requires an address:port"),
                    );
                }
                "--generator" => {
                    i += 1;
                    result.generator = Some(
                        args.get(i)
                            .and_then(|v| v.parse().ok())
                            .expect("--generator requires a version number"),
                    );
                }
                "--world" => {
                    i += 1;
                    result.world = Some(args.get(i).expect("--world requires a name").clone());
                }
                "--load" => {
                    i += 1;
                    result.load = Some(args.get(i).expect("--load requires a name").clone());
                }
                "--new-world" => {
                    i += 1;
                    result.new_world =
                        Some(args.get(i).expect("--new-world requires a name").clone());
                }
                "--menu" => {
                    i += 1;
                    let screen = args.get(i).expect(
                        "--menu requires title, pause, settings, saves, pack, map, guide, missions or breeding",
                    );
                    assert!(
                        matches!(
                            screen.as_str(),
                            "title"
                                | "pause"
                                | "settings"
                                | "saves"
                                | "pack"
                                | "map"
                                | "guide"
                                | "missions"
                                | "breeding"
                        ),
                        "--menu takes title, pause, settings, saves, pack, map, guide, missions or breeding"
                    );
                    result.menu = Some(screen.clone());
                }
                "--settings-tab" => {
                    i += 1;
                    let name = args.get(i).expect("--settings-tab requires a tab");
                    result.settings_tab = menu::SETTINGS_TABS
                        .iter()
                        .position(|t| t.eq_ignore_ascii_case(name))
                        .expect("--settings-tab takes picture, weather, world, movement, input or sound")
                        as u8;
                }
                "--input-tab" => {
                    i += 1;
                    let name = args.get(i).expect("--input-tab requires a sub-tab");
                    result.input_tab = pbd_app::controls::Tab::ALL
                        .iter()
                        .position(|t| t.label().eq_ignore_ascii_case(&name.replace('-', " ")))
                        .expect("--input-tab takes on-foot, hands, flying, vehicles or menus")
                        as u8;
                }
                "--cursor" => {
                    let at = |k: usize| -> f32 {
                        args.get(i + k)
                            .and_then(|v| v.parse().ok())
                            .expect("--cursor takes X Y in logical px")
                    };
                    result.cursor = Some(Vec2::new(at(1), at(2)));
                    i += 2;
                }
                "--rebind" => {
                    i += 1;
                    let pair = args.get(i).expect("--rebind requires SLOT=KEY");
                    let (slot, key) = pair.split_once('=').expect("--rebind takes SLOT=KEY");
                    result.rebinds.push((slot.into(), key.into()));
                }
                "--map-mpp" => {
                    i += 1;
                    result.map.metres_per_pixel = Some(
                        args.get(i)
                            .and_then(|v| v.parse().ok())
                            .expect("--map-mpp requires metres a pixel"),
                    );
                }
                "--map-overlay" => {
                    i += 1;
                    let name = args.get(i).expect("--map-overlay requires a name");
                    result.map.overlay = Some(
                        pbd_core::overlay::Overlay::ALL
                            .into_iter()
                            .find(|o| o.name().eq_ignore_ascii_case(name))
                            .unwrap_or_else(|| panic!("no overlay called {name}")),
                    );
                }
                "--map-layer" => {
                    i += 1;
                    result.map.layer =
                        Some(args.get(i).expect("--map-layer requires a name").clone());
                }
                "--map-mark-east" => {
                    i += 1;
                    result.map.mark_east_m.push(
                        args.get(i)
                            .and_then(|v| v.parse().ok())
                            .filter(|m: &f32| m.is_finite() && *m > 0.0)
                            .expect("--map-mark-east requires metres"),
                    );
                }
                "--map-night" => result.map.night = true,
                "--map-clouds" => result.map.clouds = true,
                "--at" => {
                    let lat: f32 = args
                        .get(i + 1)
                        .and_then(|v| v.parse().ok())
                        .expect("--at requires a latitude");
                    let lon: f32 = args
                        .get(i + 2)
                        .and_then(|v| v.parse().ok())
                        .expect("--at requires a longitude");
                    i += 2;
                    result.at = Some((lat, lon));
                }
                "--map-at" => {
                    let lat: f32 = args
                        .get(i + 1)
                        .and_then(|v| v.parse().ok())
                        .expect("--map-at requires a latitude");
                    let lon: f32 = args
                        .get(i + 2)
                        .and_then(|v| v.parse().ok())
                        .expect("--map-at requires a longitude");
                    i += 2;
                    result.map.at = Some((lat, lon));
                }
                "--turn" => {
                    i += 1;
                    let degrees: f32 = args
                        .get(i)
                        .and_then(|d| d.parse().ok())
                        .expect("--turn requires degrees a second");
                    assert!(degrees.is_finite(), "--turn takes finite degrees");
                    result.turn = degrees;
                }
                "--view" => {
                    i += 1;
                    result.view = args.get(i).expect("--view requires a view name").clone();
                }
                "--spawn" => {
                    i += 1;
                    let spawn = args.get(i).expect("--spawn requires a place").clone();
                    assert!(
                        spawn == "mouth" || spawn == "snow" || spawn_biome(&spawn).is_some(),
                        "--spawn knows mouth, snow and the biomes (beach, fields, desert, \
                         jungle, swamp, mountains, tundra)"
                    );
                    result.spawn = Some(spawn);
                }
                "--frame-graph" => result.frame_graph = true,
                "--showcase" => {
                    i += 1;
                    let name = args.get(i).expect("--showcase requires a name").clone();
                    let file = showcase::file::load(&name).unwrap_or_else(|e| panic!("{e}"));
                    // A new world, on foot, where and when the shot list
                    // says, with the clock running from its hour.
                    result.walk = true;
                    result.spawn = file.spawn.clone().or(result.spawn.take());
                    result.at = file.at.or(result.at);
                    result.yaw = file.yaw.or(result.yaw);
                    result.time = Some(file.hour);
                    result.day = file.day.or(result.day);
                    result.clock_runs = true;
                    result.showcase = Some((name, file));
                }
                "--showcase-from" => {
                    i += 1;
                    result.showcase_from = args
                        .get(i)
                        .and_then(|n| n.parse().ok())
                        .filter(|n| *n >= 1)
                        .expect("--showcase-from takes a shot number from 1");
                }
                "--showcase-stills" => {
                    i += 1;
                    result.showcase_stills = Some(PathBuf::from(
                        args.get(i).expect("--showcase-stills requires a directory"),
                    ));
                }
                "--no-vsync" => result.no_vsync = true,
                "--vsync" => result.vsync = true,
                "--frame-latency" => {
                    i += 1;
                    let frames: u32 = args
                        .get(i)
                        .expect("--frame-latency requires 1 or 2")
                        .parse()
                        .expect("invalid frame latency");
                    assert!((1..=2).contains(&frames), "frame latency must be 1 or 2");
                    result.frame_latency = frames;
                }
                "--frame-limit" => {
                    i += 1;
                    result.settings.frame_limit = Some(
                        args.get(i)
                            .and_then(|n| n.parse().ok())
                            .and_then(pbd_app::settings::FrameLimit::from_fps)
                            .expect("--frame-limit takes 0, 60 or 120"),
                    );
                }
                "--render-scale" => {
                    i += 1;
                    let percent: u32 = args
                        .get(i)
                        .and_then(|n| n.parse().ok())
                        .expect("--render-scale requires a percentage");
                    assert!(
                        (50..=100).contains(&percent),
                        "--render-scale takes 50 to 100"
                    );
                    result.settings.render_scale = Some(percent);
                }
                "--no-auto-step" => result.settings.auto_step = Some(false),
                "--rain-shadows" => {
                    i += 1;
                    let state = args.get(i).expect("--rain-shadows requires on or off");
                    result.settings.rain_shadows = Some(match state.as_str() {
                        "on" => true,
                        "off" => false,
                        _ => panic!("--rain-shadows takes on or off"),
                    });
                }
                "--reflections" => {
                    i += 1;
                    let level = args
                        .get(i)
                        .expect("--reflections requires off, low, medium or high");
                    result.settings.reflections = Some(
                        pbd_app::config::Reflections::parse(level)
                            .expect("--reflections takes off, low, medium or high"),
                    );
                }
                "--antialiasing" => {
                    i += 1;
                    let state = args.get(i).expect("--antialiasing requires on or off");
                    result.settings.antialiasing = Some(match state.as_str() {
                        "on" => true,
                        "off" => false,
                        _ => panic!("--antialiasing takes on or off"),
                    });
                }
                "--shadows" => {
                    i += 1;
                    let level = args
                        .get(i)
                        .expect("--shadows requires low, medium, high or ultra");
                    result.settings.shadows = Some(
                        pbd_app::config::ShadowQuality::parse(level)
                            .expect("--shadows takes low, medium, high or ultra"),
                    );
                }
                "--grass" => {
                    i += 1;
                    // Tufts are the only grass (owner, 2026-11-16); the flag
                    // stays so old capture recipes still run.
                    let look = args.get(i).expect("--grass requires tufts");
                    assert!(look == "tufts", "--grass takes tufts: the blades are gone");
                }
                "--upscale" => {
                    i += 1;
                    let filter = args.get(i).expect("--upscale requires bilinear or nearest");
                    assert!(
                        matches!(filter.as_str(), "bilinear" | "nearest"),
                        "--upscale takes bilinear or nearest"
                    );
                    result.upscale_nearest = filter == "nearest";
                }
                "--capture-loading" => result.capture_loading = true,
                "--walk-distance" => {
                    i += 1;
                    let metres: f32 = args
                        .get(i)
                        .expect("--walk-distance requires metres")
                        .parse()
                        .expect("invalid walk distance");
                    assert!(
                        metres.is_finite() && metres > 0.0,
                        "walk distance must be positive"
                    );
                    result.walk_distance = Some(metres);
                    result.walk = true;
                }
                "--stroll" => {
                    i += 1;
                    let seconds: f32 = args
                        .get(i)
                        .and_then(|s| s.parse().ok())
                        .expect("--stroll requires seconds");
                    assert!(
                        seconds.is_finite() && seconds > 0.0,
                        "--stroll takes seconds"
                    );
                    result.stroll = Some(seconds);
                    result.walk = true;
                }
                "--stand" => {
                    i += 1;
                    let seconds: f32 = args
                        .get(i)
                        .and_then(|s| s.parse().ok())
                        .expect("--stand requires seconds");
                    assert!(
                        seconds.is_finite() && seconds > 0.0,
                        "--stand takes seconds"
                    );
                    result.stand = Some(seconds);
                    result.walk = true;
                }
                "--coast-walk" => {
                    i += 1;
                    let metres: f32 = args
                        .get(i)
                        .and_then(|s| s.parse().ok())
                        .expect("--coast-walk requires metres");
                    assert!(
                        metres.is_finite() && metres > 0.0,
                        "--coast-walk takes metres"
                    );
                    result.coast_walk = Some(metres);
                    result.walk = true;
                }
                "--walk-river" => {
                    result.walk_river = true;
                    result.walk = true;
                    // An optional bank, looking downstream: left (the
                    // default) or right.
                    if let Some(side) = args
                        .get(i + 1)
                        .filter(|s| matches!(s.as_str(), "left" | "right"))
                    {
                        result.walk_river_right = side == "right";
                        i += 1;
                    }
                }
                "--stroll-leg" => {
                    i += 1;
                    let seconds: f32 = args
                        .get(i)
                        .and_then(|s| s.parse().ok())
                        .expect("--stroll-leg requires seconds");
                    assert!(
                        seconds.is_finite() && seconds > 0.0,
                        "--stroll-leg takes seconds"
                    );
                    result.stroll_leg = Some(seconds);
                }
                "--no-audio" => result.audio.off = true,
                "--audio-log" => result.audio.log = true,
                "--audio-wav" => {
                    i += 1;
                    result.audio.wav = Some(PathBuf::from(
                        args.get(i).expect("--audio-wav requires a path"),
                    ));
                }
                "--frame-log" => {
                    i += 1;
                    result.frame_log = Some(PathBuf::from(
                        args.get(i).expect("--frame-log requires a path"),
                    ));
                }
                "--frames" => {
                    i += 1;
                    result.frames = args
                        .get(i)
                        .expect("--frames requires a number")
                        .parse()
                        .expect("invalid frame count");
                }
                "--tour" => result.tour = true,
                "--route" => {
                    i += 1;
                    let name = args.get(i).expect("--route requires a route name");
                    result.route_kind = route_kind(name);
                    result.route_broken = name == "clouds";
                    result.route = true;
                    result.tour = true;
                }
                "--fly" => result.fly = true,
                "--walk" => result.walk = true,
                "--swim" => {
                    result.walk = true;
                    result.swim = true;
                }
                "--climb-out" => {
                    result.walk = true;
                    result.climb_out = true;
                }
                "--fish" => {
                    result.walk = true;
                    result.fish = true;
                }
                "--fish-play" => {
                    result.walk = true;
                    result.fish = true;
                    result.fish_play = true;
                }
                "--watch-hook" => result.watch_hook = true,
                "--cast" => result.cast = true,
                "--rod-cue" => {
                    i += 1;
                    use pbd_core::fishing::Phase;
                    let name = args.get(i).expect("--rod-cue requires a phase");
                    result.rod_cue = Some(match name.to_lowercase().as_str() {
                        "nibble" => Phase::Nibble,
                        "bite" => Phase::Bite,
                        "hooked" => Phase::Hooked,
                        other => panic!("--rod-cue: nibble, bite or hooked, not {other}"),
                    });
                }
                "--load-hold" => result.load_hold = true,
                "--carts" => {
                    result.walk = true;
                    result.carts.get_or_insert_default();
                }
                "--cart-pull" => result.carts.get_or_insert_default().pull = true,
                "--cart-drive" => result.carts.get_or_insert_default().drive = true,
                "--cart-close" => {
                    i += 1;
                    result.walk = true;
                    let code = args
                        .get(i)
                        .expect("--cart-close requires hand, horse, plodder or ride");
                    let script = result.carts.get_or_insert_default();
                    script.plodder = code == "plodder";
                    script.ride = code == "ride";
                    script.close = if script.plodder {
                        Some(pbd_core::vehicle::cart::CartKind::HorseCart)
                    } else {
                        pbd_core::vehicle::cart::CartKind::ALL
                            .into_iter()
                            .find(|k| k.code().starts_with(code.as_str()))
                    };
                }
                "--life" => {
                    i += 1;
                    result.walk = true;
                    result.life = Some(
                        args.get(i)
                            .expect("--life requires a species id from fauna.ron")
                            .clone(),
                    );
                }
                "--husbandry" => {
                    i += 1;
                    result.walk = true;
                    result.husbandry = Some(
                        args.get(i)
                            .expect("--husbandry requires a scene: wild, tame, pen or collect")
                            .clone(),
                    );
                }
                "--aboard" => {
                    i += 1;
                    let key = args.get(i).expect("--aboard requires a craft");
                    result.aboard = Some(
                        pbd_core::vehicle::Kind::from_key(key)
                            .expect("--aboard knows loon, tube and raft"),
                    );
                    result.walk = true;
                }
                "--seat" => result.seat = true,
                "--pole-run" => result.pole_run = true,
                "--fixed-dt" => result.fixed = true,
                "--clock-runs" => result.clock_runs = true,
                "--real-time" => result.real_time = true,
                "--render-offset" => {
                    let mut components = [0.0; 3];
                    for component in &mut components {
                        i += 1;
                        *component = args
                            .get(i)
                            .expect("--render-offset requires x y z in metres")
                            .parse::<f32>()
                            .expect("invalid render offset");
                    }
                    result.render_offset = Vec3::from_array(components);
                    assert!(
                        result.render_offset.is_finite(),
                        "render offset must be finite"
                    );
                }
                "--height" => {
                    i += 1;
                    let height: f32 = args
                        .get(i)
                        .expect("--height requires metres above the shore")
                        .parse()
                        .expect("invalid height");
                    assert!(
                        height.is_finite() && height >= 0.0,
                        "height must be finite and non-negative"
                    );
                    result.height = Some(height);
                }
                "--overlay" => {
                    i += 1;
                    let name = args.get(i).expect("--overlay requires a name");
                    result.overlay = Some(
                        pbd_core::overlay::Overlay::ALL
                            .into_iter()
                            .find(|o| o.name().eq_ignore_ascii_case(name))
                            .unwrap_or_else(|| panic!("no overlay called {name}")),
                    );
                }
                "--gale" => {
                    i += 1;
                    let mps: f32 = args
                        .get(i)
                        .expect("--gale requires a wind in m/s")
                        .parse()
                        .expect("invalid gale wind");
                    assert!((0.0..=60.0).contains(&mps), "gale must be within 0..60 m/s");
                    result.gale = Some(mps);
                }
                "--rain" => {
                    i += 1;
                    let rain: f32 = args
                        .get(i)
                        .expect("--rain requires an intensity 0..1")
                        .parse()
                        .expect("invalid rain intensity");
                    assert!((0.0..=1.0).contains(&rain), "rain must be within 0..1");
                    result.rain = rain;
                }
                "--weather-at" => {
                    i += 1;
                    let seconds: f32 = args
                        .get(i)
                        .expect("--weather-at requires seconds")
                        .parse()
                        .expect("invalid weather seconds");
                    assert!(seconds.is_finite(), "weather seconds must be finite");
                    result.weather_at = seconds;
                }
                "--open-doors" => result.open_doors = true,
                "--no-shadows" => result.no_shadows = true,
                "--room-sky" => {
                    let mut share = || {
                        i += 1;
                        let v: f32 = args
                            .get(i)
                            .and_then(|v| v.parse().ok())
                            .expect("--room-sky requires two shares, open and shut");
                        assert!((0.0..=1.0).contains(&v), "--room-sky shares are 0..1");
                        v
                    };
                    result.room_sky = Some((share(), share()));
                }
                "--room-bounce" => {
                    i += 1;
                    let k: f32 = args
                        .get(i)
                        .and_then(|v| v.parse().ok())
                        .expect("--room-bounce requires a weight");
                    assert!(k.is_finite() && k >= 0.0, "--room-bounce is 0 or more");
                    result.room_bounce = Some(k);
                }
                "--up" => {
                    i += 1;
                    result.up = args
                        .get(i)
                        .and_then(|v| v.parse().ok())
                        .filter(|m: &f32| m.is_finite() && *m >= 0.0)
                        .expect("--up requires metres");
                }
                "--verify-flight" => {}
                "--verify-route" => {
                    i += 1;
                }
                unknown => panic!("unknown argument {unknown}; use --help"),
            }
            i += 1;
        }
        assert!(
            result.generator.is_none() || result.world.is_none() && result.load.is_none(),
            "--generator is for a run with no --world: a world keeps its own generator"
        );
        assert!(
            !(result.walk && (result.fly || result.tour)),
            "--walk cannot be combined with --fly or --tour"
        );
        assert!(
            [
                "orbit",
                "coast",
                "surface",
                "seam",
                "night",
                "nightshore",
                "midnight",
                "meadow",
                "river",
                "pole",
                "shore",
                "wade",
                "dive",
                "cave",
                "overhang",
                "mouth",
                "seacave",
                "column"
            ]
            .contains(&result.view.as_str()),
            "unknown capture view"
        );
        assert!(
            result.frames >= 60 && result.frames <= 100_000,
            "capture frames must be 60..100000"
        );
        assert!(
            result.height.is_none()
                || ([
                    "shore",
                    "nightshore",
                    "midnight",
                    "meadow",
                    "river",
                    "dive",
                    "column"
                ]
                .contains(&result.view.as_str())
                    && result.capture.is_some())
                || result.fly,
            "--height requires --fly, or --view shore or dive with a static --capture"
        );
        result
    }
}

#[derive(Resource)]
struct CaptureState {
    pub(crate) frame: u32,
    requested: bool,
    saved: bool,
    finish_frame: u32,
}

#[derive(Resource)]
pub struct FrameStats {
    pub frame_ms: f32,
    pub samples: Vec<f64>,
    /// Frames measured since the start: the frame log's frame number, which
    /// `samples` (trimmed in a long session) no longer is.
    frames: u64,
    previous: Instant,
    /// When the app started: the frame log's clock, to line a slow frame up
    /// with what the log says happened then.
    started: Instant,
}

pub fn run(args: &[String]) {
    // The `READY` line counts from here (`loading-screen` decision 4).
    let launched = Instant::now();
    if let Some(i) = args.iter().position(|s| s == "--verify-route") {
        verify_route(route_kind(
            args.get(i + 1).map_or("far-side", String::as_str),
        ));
        return;
    }
    if args.iter().any(|s| s == "--verify-flight") {
        verify_flight(args.windows(2).any(|w| w[0] == "--view" && w[1] == "pole"));
        return;
    }
    let launch = Launch::parse(args);
    // The river cache's file lives beside the worlds (`river-cache` design
    // 11): set before anything asks an altitude, so the drainage map is read
    // from it rather than built.
    pbd_core::river_cache::set_dir(Some(pbd_app::paths::saves_dir().join("cache")));
    if let Some(path) = &launch.capture
        && let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).expect("capture directory");
    }
    let photo = launch.capture.is_some() && !launch.tour && !launch.walk && !launch.fly;
    // The world is opened BEFORE the app is built, because where the player
    // was standing decides where the planet's fine set and the column tier are
    // anchored. Restoring the pose afterwards would build the world around the
    // spawn and then teleport away from it.
    let mut world = open_world(&launch);
    if let Some(version) = launch.generator {
        world.identity.generator = version;
    }
    // Joining another's world (`multiplayer` design 20): the world is the
    // host's, opened in memory from its welcome, before anything is built.
    let mut joined = None;
    if let Some(addr) = launch.join {
        // Two minutes: a host still building its world answers when its first
        // frame runs.
        match pbd_app::net_play::join(addr, std::time::Duration::from_secs(120)) {
            Ok(joining) => {
                world = saves::WorldSave::memory_only();
                world.identity.seed = joining.seed;
                world.identity.generator = joining.generator;
                world.identity.topology = joining.topology;
                world.world_seconds = Some(joining.clock_s);
                world.join_remote();
                let mut early = Vec::new();
                for message in joining.early {
                    pbd_app::net_play::receive(joining.player, message, &mut world, &mut early);
                }
                eprintln!(
                    "joined {addr} as player {} ({} edits so far)",
                    joining.player,
                    early.len()
                );
                joined = Some((joining.client, joining.player));
            }
            Err(why) => {
                eprintln!("could not join {addr}: {why}");
                std::process::exit(2);
            }
        }
    }
    // The planet is made by the generator the world was made by (an old
    // world keeps its biomes, survey B3), chosen here, before anything builds
    // it (`bigger-biomes` decision 6). The logger is not up yet.
    if let Err(why) = pbd_app::planet::switch_generator(world.identity.generator) {
        eprintln!("terrain generator: {why}");
    }
    let saved_seconds = world.world_seconds;
    let hotbar = slots::Hotbar::restore(&mut world);
    let tools = pbd_app::fish::ToolSlot::restore(&world);
    let drops = pbd_app::drops::Drops::restore(&world);
    // The saves page is the front door of a plain launch: a player picks the
    // world rather than being put in the last one.
    let opening = menu::opening_screen(
        launch.menu.as_deref(),
        // A joined game plays the host's world (`multiplayer` design 20):
        // the title screen's CONTINUE would open a saved world instead.
        launch.world.is_some() || launch.showcase.is_some() || launch.join.is_some(),
        // A recording instrument plays the world it was launched on, as a
        // capture does: the title screen would stand in front of the film.
        launch.capture.is_some() || launch.walk_river,
    );
    let restored = world.pose;
    // Where and when a world with no pose starts, from the launch alone, for
    // a new world made later from the saves page (`new-world-arrival`).
    let new_world_start = NewWorldStart {
        direction: spawn_direction(&launch),
        clock: pbd_core::daylight::Clock::at(
            launch.day.unwrap_or(0),
            launch.time.unwrap_or(pbd_core::daylight::START_HOUR),
        ),
    };
    // The world's name when the player is first in it; not in a capture,
    // whose shots would change.
    let arrival = world
        .slot()
        .filter(|_| launch.capture.is_none())
        .map(|slot| arrival::Arrival::new(&slot.file.name, restored.is_none()));
    let step = Duration::from_secs_f64(1.0 / FIXED_HZ);
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: pbd_app::paths::assets_dir().to_string_lossy().into_owned(),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(bevy::log::LogPlugin {
                custom_layer: cpu_times::span_layer,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: pbd_core::locale::tr("window.title").into(),
                    resolution: window_size().into(),
                    present_mode: if launch.capture.is_some()
                        || launch.no_vsync
                        || (launch.frame_log.is_some() && !launch.vsync)
                    {
                        PresentMode::AutoNoVsync
                    } else {
                        PresentMode::AutoVsync
                    },
                    // Two frames in flight by default (`crab-breeze-lessons`
                    // 1.3): with one, the render world waited on the GPU's
                    // previous frame before it could encode the next.
                    desired_maximum_frame_latency: if launch.capture.is_some() {
                        None
                    } else {
                        std::num::NonZeroU32::new(launch.frame_latency)
                    },
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins((
        PhysicsPlugins::default(),
        PaleBlueDotPlugin,
        ConfigPlugin,
        WeatherPlugin {
            rain: launch.rain,
            weather_at: launch.weather_at,
        },
        PlanetPlugin,
        FlightViewPlugin,
        SkyPlugin,
        pbd_app::tone::TonePlugin,
        pbd_app::bloom::LiteBloomPlugin,
    ))
    .insert_resource(CelestialScene::planet_at_origin(PLANET_RADIUS as f64, 1.0))
    .insert_resource(PhysicsFrame::default())
    // The render origin starts at -offset, so a camera placed at a
    // planet-local eye plus the offset is in render space from its first
    // frame (`movement-smoothness` design 6.6).
    .insert_resource(pbd_app::walking::DrawBetweenSteps(launch.capture.is_none()))
    .insert_resource(pbd_app::render_origin::RenderOffset(
        launch.render_offset.as_dvec3(),
    ))
    .insert_resource(pbd_app::planet::PlanetRenderFrame {
        center: launch.render_offset.as_dvec3(),
        origin: -launch.render_offset.as_dvec3(),
    })
    .insert_resource(Time::<Fixed>::from_duration(step))
    .insert_resource(SubstepCount(4))
    .insert_resource(FlightViewConfig {
        route_kind: launch.route_kind,
        route_scenic_broken: launch.route_broken,
        mode: if launch.route {
            FlyMode::Route
        } else if launch.tour {
            FlyMode::Tour
        } else {
            FlyMode::Manual
        },
        // The route always starts from the walker's dry-land spawn, not
        // wherever the world's last flight ended.
        spawn_direction: if launch.route {
            spawn_direction(&launch)
        } else {
            restored
                .map(|pose| pose.position.normalize_or(Vec3::Y))
                .unwrap_or_else(|| spawn_direction(&launch))
        },
        // `--fly --height M` starts that far over the ground (a capture's
        // view from the air: `water-flow`'s rivers); 240 m otherwise.
        // `--fly --yaw/--pitch` aims a flight's first view, so a capture can
        // look down at a place from the air.
        view_pitch_down: match (launch.fly, launch.pitch) {
            (true, Some(pitch)) => -pitch.to_radians(),
            _ => FlightViewConfig::default().view_pitch_down,
        },
        view_yaw_deg: if launch.fly { launch.yaw } else { None },
        spawn_altitude: if launch.fly {
            launch.height.unwrap_or(240.0)
        } else {
            240.0
        },
        // The tour keeps 45 m above the ground; the route lands, so it keeps
        // only the walker's eye height.
        minimum_clearance: if launch.route {
            1.6
        } else if launch.tour {
            45.0
        } else {
            1.6
        },
        startup_camera: !photo,
        ..default()
    })
    // What the save recorded, or the starting kit in a new world. The hotbar
    // rides the edit log rather than a timer, so what comes back is what was
    // held when the last block moved.
    .insert_resource(hotbar)
    // And the tool in hand, which the save records whenever it changes.
    .insert_resource(tools)
    // And the dug blocks still floating where they were cut.
    .add_plugins(pbd_app::drops::DropsPlugin)
    // Bark dust and falling leaves on a hit or felled tree (`tree-punch-shake`).
    .add_plugins(pbd_app::tree_fx::TreeFxPlugin)
    // `PBD_MOTION_TRACE` (`movement-smoothness`): absent, nothing.
    .add_plugins(pbd_app::motion_trace::MotionTracePlugin)
    .add_plugins(pbd_app::step_trace::StepTracePlugin)
    // `PBD_MEMORY_TRACE` (`memory-leak`): absent, nothing.
    .add_plugins(pbd_app::memory_trace::MemoryTracePlugin)
    .add_plugins(pbd_app::sim_origin::SimOriginPlugin)
    // The world map on M (`world-map`), and how a capture asks it to open.
    .add_plugins(map_screen::MapScreenPlugin)
    // Each world's city sites, made once and kept in its save (`city-sites`).
    .add_plugins(pbd_app::sites::SitesPlugin)
    .add_plugins(pbd_app::compass::CompassPlugin)
    // The towns standing at their sites (`cities-in-the-world`).
    .add_plugins(pbd_app::towns::TownsPlugin)
    // The player's buildings and build mode (`player-building`).
    .add_plugins(pbd_app::builds::BuildsPlugin)
    // Logs on the water (`log-drives`).
    .add_plugins(pbd_app::logs::LogsPlugin)
    // Furniture: seats, beds and containers (`furniture`).
    .add_plugins(furniture::plugin)
    // The building planner and construction sites (`building-planner`).
    .add_plugins(construction::plugin)
    .init_resource::<build_mode::BuildMode>()
    .init_resource::<build_mode::Blueprints>()
    .init_resource::<pbd_app::build_view::BuildView>()
    .add_systems(Startup, build_mode::spawn_ghost)
    .add_systems(
        PreUpdate,
        build_mode::take_input
            .after(bevy::input::InputSystems)
            .after(pbd_app::controls::remap_keys),
    )
    .add_systems(
        Update,
        (
            build_mode::load_blueprints,
            build_mode::drive_view,
            build_mode::aim_and_build.run_if(
                resource_exists::<pbd_app::planet::PlanetFine>
                    .and(resource_exists::<pbd_app::planet::PlanetContact>),
            ),
            build_mode::draw_ghost,
            build_mode::paint_ui,
        )
            .chain()
            .before(pbd_app::builds::sync),
    )
    // `PBD_BUILD_SHOT`, a capture instrument (`player-building` captures).
    .add_systems(
        Update,
        build_mode::scripted
            .run_if(
                resource_exists::<CaptureState>
                    .and(resource_exists::<pbd_app::planet::PlanetFine>)
                    .and(pbd_app::loading::ready),
            )
            .before(build_mode::aim_and_build),
    )
    // The roads between them (`townsfolk` design 10 and 11).
    .add_plugins(pbd_app::roads::RoadsPlugin)
    // Their people (`townsfolk`).
    .add_plugins(pbd_app::townsfolk::TownsfolkPlugin)
    .insert_resource(launch.map.clone())
    .insert_resource(drops)
    .configure_sets(
        Update,
        pbd_app::drops::DropsSet.after(digging::dig_and_place),
    )
    .init_resource::<equipment::Picker>()
    // Whose roster the guide and the icons read, in every mode; the fish
    // themselves come with the walker.
    .init_resource::<pbd_app::fish::Body>()
    .init_resource::<guide::GuidePage>()
    // The breeding screen's choices, in every mode: a still capture has no
    // husbandry plugin, and the screen's systems still run.
    .init_resource::<pbd_app::husbandry::BreedingPick>()
    .init_resource::<pack::PackHand>()
    .init_resource::<boat::BoatMenu>()
    .init_resource::<pack::HoldView>()
    .init_resource::<pack::OpenChest>()
    // Farming and the forge (`farming-and-smelting`).
    .init_resource::<farm::FarmNote>()
    .init_resource::<water::PourSources>()
    .init_resource::<farm::StationAt>()
    .init_resource::<craft::CraftPick>()
    .add_message::<pbd_app::hotbar::Gained>()
    .add_message::<pbd_app::hotbar::Planted>()
    // Cooking (`cooking` design S5, S6).
    .init_resource::<pbd_app::kitchen::MealEffects>()
    .add_message::<pbd_app::kitchen::Cooked>()
    .add_message::<pbd_app::kitchen::Ate>()
    // The starter missions (`starter-missions`): the chain, its counting
    // and the player's record; the HUD line and the tab are `missions`.
    .add_plugins(pbd_app::missions::MissionsPlugin)
    .insert_resource(pbd_app::missions::ForcedMode(launch.mission_mode))
    // Coins, trade and town jobs (`townsfolk` design 5 to 12): the rules'
    // state and counting, then the coin count and the talk, trade and
    // board panels.
    .add_plugins((pbd_app::economy::EconomyPlugin, talk::TalkPlugin))
    .add_plugins(survival_play::SurvivalPlayPlugin)
    // Fire in the player's hands, in Survival (`wildfire` 4.1, 4.3, 4.4).
    .add_plugins(fire_play::FirePlayPlugin)
    // Combat (`survival-mode` combat C1 to C4): the swing's rule and its
    // claim on the click run before the digging reads it.
    .add_plugins(pbd_app::combat::CombatPlugin)
    .configure_sets(
        Update,
        pbd_app::combat::CombatSet.before(digging::dig_and_place),
    )
    // `--craft pack-locked`'s pointer, after the UI's focus pass (a capture
    // instrument, `tool-crafting` 5.5).
    .add_systems(
        PreUpdate,
        craft::hover_locked.after(bevy::ui::UiSystems::Focus),
    )
    .add_systems(
        Update,
        (
            (
                missions::press_tabs,
                missions::paint,
                missions::show_line,
                mission_tree::tree_tabs,
                mission_tree::switch_tree,
                mission_tree::paint_jobs,
                mission_tree::press_track,
                mission_tree::steer,
                mission_tree::relayout,
                mission_tree::place,
                mission_tree::paint,
                mission_tree::hover_tip,
            )
                .chain(),
            missions::scripted.run_if(pbd_app::loading::ready),
            missions::scripted_town.run_if(pbd_app::loading::ready),
            mission_tree::scripted_track
                .run_if(pbd_app::loading::ready)
                .run_if(|launch: Res<Launch>| launch.track_job),
            showcase::hide_readout.run_if(|launch: Res<Launch>| launch.no_readout),
            missions::night_watch.run_if(pbd_app::loading::ready),
            missions::about_watch.run_if(pbd_app::loading::ready),
        ),
    )
    // Survival's paths (`survival-mode` design 18): the picker over the tree,
    // and the watches that raise the deeds the game already sees.
    .init_resource::<path_picker::Picker>()
    .add_systems(
        Update,
        (
            (
                path_picker::offer,
                path_picker::press,
                path_picker::paint,
                path_picker::light_lane,
            )
                .chain()
                .after(mission_tree::hover_tip),
            path_picker::scripted_pick.run_if(pbd_app::loading::ready),
            missions::dawn_watch.run_if(pbd_app::loading::ready),
            missions::water_watch.run_if(pbd_app::loading::ready),
        ),
    )
    // The world, loaded before the planet is built: `create_planet` reads its
    // edits for the first tier, so a save's holes are there on the first frame
    // rather than appearing when the player first walks.
    .insert_resource(world)
    // The network, hosting or joined (`multiplayer` design 20).
    .add_systems(
        Update,
        pbd_app::net_play::play.after(digging::dig_and_place),
    )
    // The clock: pinned and stopped where a capture asked for an hour, so a
    // picture is a function of its flags rather than of when it was taken.
    .insert_resource(pbd_app::sky::Sun {
        clock: sun_clock(&launch, saved_seconds),
        // `PBD_CLOCK_RUNS` (`baked-light-ease`, a measurement switch) keeps it
        // running in a capture too, at its own rate.
        running: ((launch.time.is_none() && launch.day.is_none() || launch.clock_runs)
            && launch.capture.is_none())
            || pbd_app::sky::clock_runs().is_some(),
    })
    .init_resource::<digging::Aim>()
    .init_resource::<digging::Mining>()
    .insert_resource(ClearColor(if std::env::var("PBD_NO_SKY").is_ok() {
        // The hole detector's background: nothing in the palette is near it,
        // so a magenta pixel is a pixel with no world behind it.
        Color::srgb(1.0, 0.0, 1.0)
    } else {
        Color::srgb(0.002, 0.004, 0.012)
    }))
    .insert_resource(CaptureState {
        frame: 0,
        requested: false,
        saved: false,
        finish_frame: 0,
    })
    .insert_resource(FrameStats {
        frame_ms: 0.0,
        samples: Vec::new(),
        frames: 0,
        previous: Instant::now(),
        started: Instant::now(),
    })
    .insert_resource(launch.clone())
    .insert_resource(pbd_app::overlay::OverlayMode(launch.overlay))
    .add_plugins(pbd_app::place::PlacePlugin)
    // Water that flows (`water-flow`): the rule's tick, the mesher, the arrows.
    .add_plugins(pbd_app::water_flow::WaterFlowPlugin)
    .add_plugins(MaterialPlugin::<scene::StarMaterial>::default())
    .add_systems(
        Startup,
        (
            scene::setup,
            hud::setup,
            compass_bar::spawn,
            frame_graph::setup,
            photo_camera,
            overlay_ui::spawn,
            cracks::spawn,
            farm::make_looks,
            farm::spawn_note,
            pbd_app::interact::spawn_prompt.run_if(resource_exists::<pbd_app::ui_kit::UiKit>),
            pbd_app::interact::spawn_fish_keys.run_if(resource_exists::<pbd_app::ui_kit::UiKit>),
        ),
    )
    // The interact key's offers from the fruit and the farm (the husbandry
    // hands, the doors and the boats offer from their own plugins), and the
    // prompt that names the one F will do (`stone-and-wood-ui`, "Interact on
    // F").
    .add_systems(
        PostUpdate,
        (
            fruit::offer.in_set(pbd_app::interact::InteractSet::Offer),
            glow_plants::offer.in_set(pbd_app::interact::InteractSet::Offer),
            sea_gather::offer.in_set(pbd_app::interact::InteractSet::Offer),
            farm::offer
                .in_set(pbd_app::interact::InteractSet::Offer)
                .run_if(resource_exists::<pbd_app::planet::PlanetFine>)
                .run_if(resource_exists::<pbd_app::saves::WorldSave>),
            pbd_app::interact::show_prompt.after(pbd_app::interact::InteractSet::Resolve),
            pbd_app::interact::show_fish_keys,
        ),
    )
    // The column tier is built by a startup system and its records land when
    // that schedule's commands apply, so a camera that wants to stand inside a
    // cave has to be placed a schedule later.
    .add_systems(PostStartup, cave_camera)
    .insert_resource(opening)
    .insert_resource(menu::FrontDoor(matches!(
        opening,
        menu::Screen::Title | menu::Screen::Saves
    )))
    .init_resource::<menu::NameField>()
    .init_resource::<equipment::HandListOpen>()
    .init_resource::<toasts::Toasts>()
    .init_resource::<mission_tree::TreeView>()
    .insert_resource(if launch.tree_jobs {
        mission_tree::ShownTree::Jobs
    } else if launch.tree_town {
        mission_tree::ShownTree::Town
    } else {
        mission_tree::ShownTree::Starter
    })
    .insert_resource(menu::SettingsTab(launch.settings_tab))
    .insert_resource(menu::InputTab(launch.input_tab))
    .init_resource::<menu::Rebinding>()
    .add_systems(Startup, menu::spawn)
    .add_systems(
        PostStartup,
        (
            slots::spawn,
            build_mode::spawn_ui,
            (
                slots::load_icons,
                (
                    equipment::spawn,
                    toasts::spawn,
                    guide::spawn,
                    drops_view::dress,
                    pack::spawn,
                    forge::spawn,
                    kitchen::spawn,
                    kitchen::spawn_effects,
                    missions::spawn,
                    breeding::spawn,
                    mission_tree::spawn,
                    path_picker::spawn,
                ),
                boat::spawn,
            )
                .chain(),
        ),
    )
    // Escape is read before either of the world's input readers, which live in
    // `RunFixedMainLoop`, and is cleared there so neither ever sees it.
    .add_systems(
        PreUpdate,
        (
            menu::capture_rebind,
            menu::name_input,
            menu::toggle,
            guide::open,
            breeding::open,
            pack::open,
            boat::open,
            map_screen::open,
        )
            .chain()
            .after(bevy::input::InputSystems)
            .after(pbd_app::controls::remap_keys)
            .after(repro::read_key),
    )
    // The repro key (`repro-capture`): read before the menus and every F
    // reader, saved in an exclusive system that sees the whole world.
    .init_resource::<repro::Asked>()
    .add_systems(
        PreUpdate,
        repro::read_key
            .after(bevy::input::InputSystems)
            .after(pbd_app::controls::remap_keys),
    )
    .add_systems(Update, repro::save)
    .add_systems(
        First,
        (repro::scripted_press, repro::scripted_keys).run_if(resource_exists::<CaptureState>),
    )
    .add_systems(Update, (breeding::press, breeding::paint).chain())
    .add_systems(Update, (boat::press, boat::paint).chain())
    .add_systems(
        Update,
        (
            menu::press_graphics,
            menu::label_graphics,
            menu::drag_settings,
            menu::show_settings,
            menu::settings_tabs,
            menu::input_tabs,
            menu::press_bindings,
            menu::rebuild_input,
            menu::wheel_scroll,
        )
            .chain(),
    )
    .add_systems(
        Update,
        (
            configure_camera,
            scene::move_moon,
            scene::turn_stars,
            scene::follow_sun,
            (
                equipment::pick,
                slots::input,
                equipment::pack_hand,
                equipment::paint,
                menu::hide_hud_on_title,
                menu::show_splash,
                toasts::gather,
                toasts::paint,
            )
                .chain(),
            (
                pack::find_hold,
                furniture::press,
                pack::press,
                pack::drop_key,
                slots::update,
                pack::paint_hold,
                furniture::paint,
                pack::paint,
            )
                .chain(),
            guide::press,
            guide::paint,
            hud::near_field,
            compass_bar::update,
            (
                frame_graph::toggle,
                frame_graph::update,
                frame_graph::breakdown,
            )
                .chain(),
            (menu::press, menu::paint, menu::rebuild_saves).chain(),
            (weather_ui::drag, weather_ui::show).chain(),
            (time_ui::press, time_ui::toggle, time_ui::show).chain(),
            overlay_ui::show,
            autosave,
            save_weather,
            // A dig press on a survey stake takes it back, and is spent
            // (`townsfolk` design 39): before the digging.
            (
                digging::server_worms,
                pbd_app::roads::take_stakes,
                digging::dig_and_place,
                cracks::show,
                digging::shake_on_impact,
            )
                .chain()
                .before(pbd_app::fish::FishSet),
            digging::scripted_dig,
            // A photograph is of the world, not of the loading screen.
            // After the screen lifts, so a capture's frame 1 is the first
            // open frame, as it was before the screen existed.
            capture
                .run_if(pbd_app::loading::ready)
                .after(pbd_app::loading::warm),
        ),
    )
    .add_systems(
        Update,
        (
            // Picking fruit from a tree (`flora-v2` 12.5), ahead of the
            // farm's F, after the husbandry hands'.
            fruit::scripted
                .after(digging::scripted_dig)
                .run_if(pbd_app::loading::ready),
            fruit::pick
                .before(farm::use_key)
                .after(digging::dig_and_place)
                .after(pbd_app::husbandry::HusbandryHands),
            // The glowing mushrooms (`glowing-mushrooms` design 3, 4): a
            // pick, then the resident set brought round to it.
            (glow_plants::pick, glow_plants::follow)
                .chain()
                .after(fruit::pick)
                .before(farm::use_key),
            // The sea floor (`sea-gathering`): the takes read from the save,
            // F on a piece, a creature or an ornament, right-click to set one.
            (
                sea_gather::refresh_takes,
                sea_gather::use_key,
                sea_gather::place,
            )
                .chain()
                .after(glow_plants::pick)
                .before(farm::use_key),
            (
                farm::use_key,
                forge::work,
                forge::grey,
                forge::paint,
                craft::press,
                craft::collect_hands,
                craft::paint,
                craft::paint_need_counts,
            )
                .chain()
                .after(digging::dig_and_place)
                .after(pbd_app::husbandry::HusbandryHands),
            // The campfire's panel opens from `farm::use_key` (after the
            // husbandry hands, which clear an F they used).
            (
                kitchen::work,
                kitchen::grey,
                kitchen::paint,
                kitchen::eat,
                craft::paint_locks.after(forge::paint),
            )
                .chain()
                .after(farm::use_key)
                .after(pbd_app::husbandry::HusbandryHands),
            (pbd_app::kitchen::refresh_effects, kitchen::show_effects),
            kitchen::scripted.after(farm::scripted),
            glow_plants::hold_script.run_if(pbd_app::loading::ready),
            (survival_kit::give_kit, survival_kit::lay_weapons).run_if(pbd_app::loading::ready),
            (farm::show, farm::show_note),
            farm::scripted.after(digging::scripted_dig),
            farm::scripted_gather.after(digging::scripted_dig),
            farm::scripted_pick_wild.after(digging::scripted_dig),
            craft::scripted.after(digging::scripted_dig),
            // Water that flows (`water-flow` E5): F5, F6 and the capture rig.
            water::scripted
                .after(digging::scripted_dig)
                .run_if(resource_exists::<pbd_app::planet::PlanetFine>)
                .run_if(resource_exists::<pbd_app::planet::PlanetContact>)
                .run_if(resource_exists::<pbd_app::drops::Drops>),
            water::pour_stop
                .after(water::scripted)
                .run_if(|launch: Res<Launch>| launch.capture.is_some())
                .run_if(resource_exists::<pbd_app::planet::PlanetFine>)
                .run_if(resource_exists::<pbd_app::planet::PlanetContact>)
                .run_if(resource_exists::<pbd_app::drops::Drops>),
            water::pour_cycle
                .after(water::scripted)
                .run_if(|launch: Res<Launch>| launch.pour_for.is_some())
                .run_if(resource_exists::<pbd_app::planet::PlanetFine>)
                .run_if(resource_exists::<pbd_app::planet::PlanetContact>)
                .run_if(resource_exists::<pbd_app::drops::Drops>),
            water::keys
                .after(digging::dig_and_place)
                .run_if(resource_exists::<pbd_app::planet::PlanetFine>)
                .run_if(resource_exists::<pbd_app::planet::PlanetContact>)
                .run_if(resource_exists::<pbd_app::drops::Drops>),
        ),
    )
    // MEASUREMENT: `PBD_EYE_PATH`, the waterline videos' camera (`eye_path`).
    .add_systems(Startup, eye_path::setup)
    .add_systems(
        PostUpdate,
        eye_path::drive
            .after(pbd_app::flight_view::follow_flight_camera)
            .before(bevy::transform::TransformSystems::Propagate)
            .run_if(resource_exists::<eye_path::EyePathNow>),
    )
    .add_systems(
        PostUpdate,
        eye_path::label
            .after(eye_path::drive)
            .before(bevy::ui::UiSystems::Layout)
            .run_if(resource_exists::<eye_path::EyePathNow>),
    )
    .add_systems(
        Update,
        landing_shots
            .run_if(|launch: Res<Launch>| {
                launch.capture.is_some() && std::env::var_os("PBD_LANDING_SHOTS").is_some()
            })
            .run_if(pbd_app::loading::ready)
            .after(capture),
    )
    .add_systems(
        Last,
        rebase_shots
            .run_if(|| std::env::var_os("PBD_REBASE_SHOTS").is_some())
            .run_if(pbd_app::loading::ready),
    )
    .init_resource::<menu::SaveIndex>()
    .init_resource::<menu::LoadRequest>()
    .insert_resource(new_world_start)
    .add_systems(Update, arrival::banner.run_if(pbd_app::loading::ready))
    .add_systems(
        PreUpdate,
        (load_on_cue, load_world, arrive)
            .chain()
            .after(menu::toggle),
    )
    .insert_resource(FrameLog::open(
        launch.frame_log.as_deref(),
        launch.frame_latency,
    ))
    .add_systems(
        Last,
        (
            (cpu_times::main_end, measure_frames, write_frame_log).chain(),
            // The save is the loading thread's until the world opens.
            drain_saves.run_if(pbd_app::world_open),
        )
            // The frame limiter holds the loop after these.
            .in_set(pbd_app::settings::PacingSet),
    );
    cpu_times::add(&mut app);
    if launch.frame_log.is_some() {
        // MEASUREMENT (`cascade-perf`): a measured run is never paced by
        // focus. Bevy holds an unfocused window to 60 Hz, which hid every
        // frame under 16.7 ms when the window could not take focus (a
        // locked screen); the run still logs it lost focus.
        app.insert_resource(bevy::winit::WinitSettings {
            focused_mode: bevy::winit::UpdateMode::Continuous,
            unfocused_mode: bevy::winit::UpdateMode::Continuous,
        });
        // A route that ends the run when it lands, like `--walk-distance`.
        app.add_systems(
            Last,
            (
                quit_after_route,
                log_window_attention,
                cpu_times::log_shadow_spans,
            ),
        );
    }
    // GPU time per render pass, for the frame log (`perf-rig`) and the F3
    // breakdown. Not on macOS: its spans write timestamps inside a pass,
    // which Apple GPUs cannot, so they read nought there, and the counter
    // sample buffers it still makes for every frame in flight ran out under
    // slow frames ("Cannot allocate sample buffer", a crash). The clouds'
    // boundary timer measures them there instead (`cloud-puffs`).
    if !cfg!(target_os = "macos") {
        app.add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin);
    }
    // Towns are lit by the field in every run, a still from above as much as
    // a walk (`cities-in-the-world` slice 4a): without it a photo drew them
    // with Bevy's own sun and ambient, near black on every face in shade.
    app.add_plugins(pbd_app::field_light::FieldLightPlugin);
    app.add_plugins(pbd_app::river_fill::RiverFillPlugin);
    // The player's settings (`player-settings`): the file in a plain run and
    // in a bot playing the game (owner, 2026-11-19: "it shouldnt skip
    // settings save", after a volume set in a showcase's window was lost);
    // in a capture or a benchmark the defaults, so its picture or its numbers
    // are its flags' alone. Read here, once the logger is up.
    // A bot playing climbs with auto step (owner, 2026-11-19: "when in bot
    // run also just turn on autostep").
    let mut overrides = launch.settings;
    if launch.showcase.is_some() {
        overrides.auto_step = Some(true);
    }
    let (player, mut keys, kept) = pbd_app::settings::for_launch(
        launch.capture.is_some() || launch.frame_log.is_some(),
        &overrides,
    );
    info!(
        "PLAYER_SETTINGS frame_limit={} render_scale={}% auto_step={} rain_shadows={} pixel_grass={} file={}",
        player.frame_limit.label(),
        player.render_scale,
        player.auto_step,
        player.rain_shadows,
        player.pixel_grass,
        kept.path
            .as_ref()
            .map_or("none".into(), |p| p.display().to_string()),
    );
    if !launch.rebinds.is_empty() {
        let mut names = keys.to_names();
        names.extend(launch.rebinds.iter().cloned());
        let (laid, skipped) = pbd_app::controls::KeyBinds::from_names(&names);
        assert!(
            skipped.is_empty(),
            "--rebind names no slot or key: {skipped:?}"
        );
        keys = laid;
    }
    if !keys.is_empty() {
        info!("PLAYER_KEYS {:?}", keys.to_names());
    }
    app.insert_resource(player)
        .insert_resource(keys)
        .insert_resource(kept)
        .insert_resource(if launch.upscale_nearest {
            pbd_app::settings::Upscale::Nearest
        } else {
            pbd_app::settings::Upscale::Bilinear
        })
        .add_plugins(pbd_app::settings::PlayerSettingsPlugin {
            capture: launch.capture.is_some(),
        });
    // `--no-shadows` in every run, a still as much as a walk: read only when
    // walking, a photo from above drew its shadows regardless.
    if launch.no_shadows {
        app.insert_resource(pbd_app::planet::shadow::ShadowSettings {
            enabled: false,
            ..default()
        });
    }
    // Every door of every town open, for a capture, with a walker or in a
    // still view (the column view's tree hut).
    if launch.open_doors {
        app.insert_resource(pbd_app::towns::OpenDoors);
    }
    if !photo && !launch.tour {
        app.insert_resource(WalkingConfig {
            start_walking: !launch.fly,
            restored: restored.map(|pose| RestoredPose {
                position: pose.position,
                heading: pose.heading,
                pitch: pose.pitch,
            }),
            pitch: launch.pitch.unwrap_or(0.0).to_radians(),
            yaw: launch.yaw.unwrap_or(0.0).to_radians(),
            turn: launch.turn.to_radians(),
            exact_start: launch.at.is_some(),
            start_up_m: launch.up,
            auto_step: player.auto_step,
            ..default()
        })
        .add_plugins((
            WalkingPlugin,
            pbd_app::vehicles::VehiclePlugin,
            pbd_app::fish::FishPlugin,
            pbd_app::held::HeldPlugin,
            pbd_app::husbandry::HusbandryPlugin,
            pbd_app::beaver::BeaverPlugin,
            // Forest fires and torches in the rain, in Survival (`wildfire`).
            pbd_app::wildfire::WildfirePlugin,
            // The beavers' and otters' skinned bodies (`ocean-environments` B29).
            pbd_app::critters::CrittersPlugin,
            // The light a player carries (`glowing-mushrooms` part B).
            pbd_app::carried_light::CarriedLightPlugin,
        ))
        .add_systems(
            Update,
            glow_plants::hand_item.before(pbd_app::held::HoldSet),
        )
        .insert_resource(pbd_app::vehicles::VehicleScript {
            board: launch.aboard,
            seat: launch.seat,
            at: launch.at.map(|_| spawn_direction(&launch)),
            pole_run: launch.pole_run,
            pitch: launch.pitch.map(f32::to_radians),
        });
        if launch.at.is_some() {
            app.insert_resource(pbd_app::towns::RespawnInTown);
        }
        if launch.room_sky.is_some() || launch.room_bounce.is_some() {
            let base = pbd_app::towns::RoomSky::default();
            let (open, shut) = launch.room_sky.unwrap_or((base.open, base.shut));
            app.insert_resource(pbd_app::towns::RoomSky {
                open,
                shut,
                bounce: launch.room_bounce.unwrap_or(base.bounce),
            });
        }
        if launch.pick_fruit.is_some() {
            app.add_systems(
                PreUpdate,
                fruit::pick_script.after(bevy::input::InputSystems),
            );
        }
        if launch.pick_glow.is_some() {
            app.add_systems(
                PreUpdate,
                glow_plants::pick_script.after(bevy::input::InputSystems),
            );
        }
        if sea_gather::shot_wanted().is_some() {
            app.add_systems(
                PreUpdate,
                sea_gather::shot_script.after(bevy::input::InputSystems),
            );
        }
        if launch.break_s.is_some() || launch.tool.is_some() {
            app.add_systems(PreUpdate, break_script.after(bevy::input::InputSystems));
        }
        if let Some(mps) = launch.gale {
            app.insert_resource(pbd_app::sea::SeaGale(mps));
        }
        if let Some(id) = launch.life.clone() {
            // `--life ID:METRES` puts the group that far ahead, for a close
            // look at something small.
            let (id, metres) = match id.split_once(':') {
                Some((id, m)) => (id.to_string(), m.parse().ok()),
                None => (id, None),
            };
            app.insert_resource(pbd_app::fish::LifeScript {
                id,
                metres,
                placed: false,
            });
        }
        if let Some(scene) = launch.husbandry.clone() {
            app.insert_resource(pbd_app::husbandry::HusbandryScript { scene });
        }
        if let Some(scene) = launch.combat.clone() {
            app.insert_resource(pbd_app::combat::CombatScript::new(scene));
        }
        if launch.cast {
            app.add_systems(
                PreUpdate,
                cast_afloat_script.after(bevy::input::InputSystems),
            );
        }
        if let Some(phase) = launch.rod_cue {
            app.insert_resource(RodCueScript(phase)).add_systems(
                Update,
                rod_cue_script
                    .after(pbd_app::fish::FishSet)
                    .before(pbd_app::held::HoldSet),
            );
        }
        if launch.load_hold {
            app.add_systems(Update, load_hold_script.before(pack::find_hold));
        }
        if let Some(script) = launch.carts {
            app.world_mut()
                .get_resource_or_insert_with(pbd_app::vehicles::carts::Carts::default)
                .script = Some(script);
        }
        if launch.fish {
            if launch.fish_play {
                app.insert_resource(FishPlay);
            }
            if launch.watch_hook {
                app.insert_resource(pbd_app::fish::WatchHook::default())
                    .add_systems(
                        PostUpdate,
                        hide_in_play_hud
                            .before(
                                bevy::camera::visibility::VisibilitySystems::VisibilityPropagate,
                            )
                            .run_if(pbd_app::loading::ready),
                    );
            }
            // In a real-time run (no `--capture`) the script's frames count
            // from `READY`, so its cast is not spent under the loading
            // screen (`movement-smoothness`'s fishing trace).
            let live = launch.capture.is_none();
            if live {
                app.insert_resource(FishLive);
            }
            app.add_systems(
                PreUpdate,
                (swim_script, fish_script)
                    .chain()
                    .after(bevy::input::InputSystems)
                    .run_if(
                        move |ready: Option<Res<pbd_app::loading::Warming>>,
                              loading: Option<Res<pbd_app::WorldLoading>>| {
                            !live || (ready.is_none() && loading.is_none())
                        },
                    ),
            );
        }
        if launch.hold_pointer {
            app.add_systems(
                PreUpdate,
                (|mut state: ResMut<pbd_app::walking::WalkingState>| {
                    state.scripted = true;
                    state.captured = true;
                })
                .after(bevy::input::InputSystems),
            );
        }
        if launch.walk_distance.is_some() {
            // Where the swim's keys go, for the same reason (below).
            app.init_resource::<WalkProgress>()
                .add_systems(PreUpdate, walk_script.after(bevy::input::InputSystems));
        }
        if launch.stroll.is_some() {
            app.add_systems(PreUpdate, stroll_script.after(bevy::input::InputSystems));
        }
        if launch.stand.is_some() {
            app.add_systems(PreUpdate, stand_script.after(bevy::input::InputSystems));
        }
        if launch.walk_river {
            app.add_systems(
                PreUpdate,
                river_walk_script
                    .after(bevy::input::InputSystems)
                    .run_if(pbd_app::loading::ready),
            );
        }
        if launch.coast_walk.is_some() {
            app.add_systems(
                PreUpdate,
                coast_script
                    .after(bevy::input::InputSystems)
                    .run_if(pbd_app::loading::ready),
            );
        }
        if let Some((name, file)) = launch.showcase.clone() {
            if let Some(dir) = &launch.showcase_stills {
                std::fs::create_dir_all(dir).expect("stills directory");
            }
            // The live control channel (`bot-control`): local only.
            if let Some(control) = showcase::control::open() {
                app.insert_resource(control);
            }
            app.insert_resource(
                showcase::Player::new(name, file, launch.showcase_stills.clone())
                    .starting_at(launch.showcase_from),
            )
            .init_resource::<showcase::Tally>()
            .init_resource::<showcase::PendingClick>()
            .add_systems(Startup, showcase::spawn_caption)
            .add_systems(Update, (showcase::hide_readout, showcase::warm_stills))
            .add_systems(
                PreUpdate,
                (showcase::tally, showcase::drive)
                    .chain()
                    .after(bevy::input::InputSystems)
                    .before(pbd_app::controls::remap_keys)
                    .run_if(pbd_app::loading::ready),
            )
            .add_systems(
                PreUpdate,
                showcase::press_controls
                    .after(bevy::ui::UiSystems::Focus)
                    .after(showcase::drive),
            );
        }
        if launch.swim {
            // The scripted keys have to be written where the real ones are:
            // after the input clear and before the walking input reads them,
            // which is in RunFixedMainLoop. Pressed in `Update` they were
            // wiped by the next frame's clear before anything looked, and the
            // walker stood still. Registered only beside the walker it drives:
            // a plain photo has no `WalkingState`, and a system that asks for
            // one unconditionally panics the first frame.
            app.add_systems(PreUpdate, swim_script.after(bevy::input::InputSystems));
        }
        if launch.climb_out {
            app.add_systems(PreUpdate, climb_out_script.after(bevy::input::InputSystems));
        }
    }
    if launch.fixed || (launch.capture.is_some() && !launch.real_time) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(step));
    }
    if let Some(at) = launch.cursor {
        app.add_systems(
            First,
            move |mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>| {
                for mut window in &mut windows {
                    if window.cursor_position() != Some(at) {
                        window.set_cursor_position(Some(at));
                    }
                }
            },
        );
    }
    // Sound (`audio-and-ambience`): the device, or the mock backend stepped
    // with a fixed-step capture; default volumes in a capture, a benchmark
    // or a take (`PBD_TAKE_SOUND=1`).
    {
        let mut audio = launch.audio.clone();
        audio.capture = launch.capture.is_some() && !launch.real_time;
        audio.default_volumes = launch.capture.is_some()
            || launch.frame_log.is_some()
            || std::env::var_os("PBD_TAKE_SOUND").is_some_and(|v| v == "1");
        app.add_plugins(pbd_app::audio::AudioPlugin(audio));
        sounds::register(&mut app);
    }
    // The UI kit (`stone-and-wood-ui`): its pieces, fonts and scale are in
    // place in `finish`, before the loading screen's first frame.
    app.add_plugins(pbd_app::ui_kit::UiKitPlugin);
    // The loading screen (`loading-screen`): added last, because it takes
    // over `Main` from the plugins that made it.
    app.add_plugins(pbd_app::loading::LoadingPlugin {
        launched,
        capture: launch.capture.is_some() && !launch.real_time,
    });
    // A launch onto the title loads on the splash (`title-screen`, "The
    // splash"): no world is seen before the menu.
    if app.world().get_resource::<menu::Screen>() == Some(&menu::Screen::Title) {
        app.insert_resource(pbd_app::loading::SplashLaunch);
    }
    if launch.capture_loading
        && let Some(path) = &launch.capture
    {
        app.insert_resource(pbd_app::loading::CaptureLoading(path.clone()));
    }
    if let Some(arrival) = arrival {
        app.insert_resource(arrival);
    }
    // `--new-world` in a capture: the world being left, the switch's loading
    // screen and the arrival, each photographed beside the shot.
    if launch.new_world.is_some()
        && let Some(path) = &launch.capture
    {
        let beside = |what: &str| {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("shot");
            path.with_file_name(format!("{stem}-{what}.png"))
        };
        app.insert_resource(pbd_app::loading::CaptureSwitch(beside("loading")));
        app.insert_resource(arrival::ArrivalShot(beside("arrival")));
        app.insert_resource(LeavingShot(beside("leaving")));
    }
    // On the network (`multiplayer` design 20): joined, or hosting.
    if let Some((client, player)) = joined {
        app.insert_resource(pbd_app::net_play::NetPlay::Joined {
            client,
            player,
            pending: Vec::new(),
        });
    } else if let Some(port) = launch.host {
        match pbd_net::Host::bind(port) {
            Ok(host) => {
                eprintln!("hosting on {}", host.local_addr());
                app.insert_resource(pbd_app::net_play::NetPlay::Host {
                    host,
                    players: Default::default(),
                });
            }
            Err(why) => eprintln!("could not host on port {port}: {why}"),
        }
    }
    app.run();
}

/// Which save this run opens.
///
/// A CAPTURE with no `--world` writes to nothing, which is the LOD camera's
/// own lesson in another costume: a harness that pins a pose every frame and
/// then saves it poisons the world for every later run. A capture that means
/// to write says which world.
fn open_world(launch: &Launch) -> WorldSave {
    let root = pbd_app::paths::saves_dir();
    // The world's seed, which every generator version shares. Not read off
    // `terrain_config()`, which would fix this run's generator before the
    // world that chooses it is open.
    let seed = pbd_core::planet_gen::TerrainConfig::TENEBRIS.seed;
    let asked = launch.world.clone();
    // A showcase plays a new world, kept in memory unless `--world` names
    // one: the owner's saves are not its to fill.
    // A Survival showcase's world is a Survival one (`survival-mode` 6.4).
    let mode = if launch.showcase.as_ref().is_some_and(|(_, f)| f.survival) {
        pbd_core::survival::WorldMode::Survival
    } else {
        pbd_core::survival::WorldMode::Explorer
    };
    if asked.is_none() && (launch.capture.is_some() || launch.showcase.is_some()) {
        let mut save = WorldSave::memory_only();
        if mode.is_survival() {
            save.identity.mode = Some(mode);
        }
        return save;
    }
    let listed = saves::list(&root);
    let slot = match asked.as_deref() {
        Some(name) => {
            let id = saves::slot_id(name);
            listed
                .into_iter()
                .find(|slot| slot.id == id || slot.file.name == name)
                .or_else(|| saves::create_in(&root, name, seed, mode).ok())
        }
        // No name: the one played most recently, which is what logging back
        // on means. A first run has none, and nothing is made behind the
        // player's back: the saves page opens and offers NEW WORLD.
        None => listed.into_iter().next(),
    };
    let Some(slot) = slot else {
        if asked.is_some() {
            warn!("no world could be opened; this run will not be saved");
        } else {
            info!("no worlds yet; the saves page will make the first");
        }
        return WorldSave::memory_only();
    };
    if slot.file.seed != seed {
        // A save is OF a world. Loading it into a different one would make it
        // silently become somebody else's.
        warn!(
            "world '{}' was made in seed {} and this is {seed}; it will not be loaded",
            slot.file.name, slot.file.seed
        );
        return WorldSave::memory_only();
    }
    if let Some(why) = saves::refusal(&slot) {
        warn!(
            "world '{}' was made with {why}; it will not be loaded",
            slot.file.name
        );
        return WorldSave::memory_only();
    }
    WorldSave::open(root, slot)
}

/// Ask for the `--load` world's load a third of the way to the shot, as the
/// saves page's LOAD would, once.
/// `--new-world` in a capture: the world being left, photographed on the
/// frame the new one is asked for.
#[derive(Resource, Clone)]
struct LeavingShot(std::path::PathBuf);

fn load_on_cue(
    mut commands: Commands,
    leaving: Option<Res<LeavingShot>>,
    launch: Res<Launch>,
    state: Res<CaptureState>,
    mut load: ResMut<menu::LoadRequest>,
    mut asked: Local<bool>,
    mut photographed: Local<bool>,
) {
    // The world being left, a few frames before it is left, so the shot is
    // not of the loading screen that replaces it.
    if let Some(shot) = leaving.filter(|_| !*photographed)
        && state.frame + 10 >= launch.frames / 3
    {
        *photographed = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(shot.0.clone()));
    }
    if *asked || state.frame < launch.frames / 3 {
        return;
    }
    if let Some(name) = launch.new_world.as_deref() {
        *asked = true;
        let seed = pbd_core::planet_gen::TerrainConfig::TENEBRIS.seed;
        match saves::create(&pbd_app::paths::saves_dir(), name, seed) {
            Ok(slot) => {
                info!(
                    "--new-world: made '{name}' ({}) at frame {}",
                    slot.id, state.frame
                );
                load.0 = Some(slot);
            }
            Err(error) => warn!("--new-world: could not make '{name}': {error}"),
        }
        return;
    }
    let Some(name) = launch.load.as_deref() else {
        return;
    };
    *asked = true;
    let id = saves::slot_id(name);
    match saves::list(&pbd_app::paths::saves_dir())
        .into_iter()
        .find(|slot| slot.id == id || slot.file.name == name)
    {
        Some(slot) => {
            info!("--load: loading '{name}' at frame {}", state.frame);
            load.0 = Some(slot);
        }
        None => warn!("--load: there is no world '{name}'"),
    }
}

/// Where and when a new world starts, as the launch decided before anything
/// was restored: the launch's spawn direction (not the restored pose's) and
/// its new-world clock (`new-world-arrival` decisions 3 and 5).
#[derive(Resource, Clone, Copy)]
struct NewWorldStart {
    direction: Vec3,
    clock: pbd_core::daylight::Clock,
}

/// What a switch begun by [`load_world`] still has to do on arrival.
#[derive(Resource)]
struct SwitchNote {
    name: String,
    /// The planet was switched to another generator.
    switched: bool,
}

/// Begin a load the saves screen asked for.
///
/// An exclusive system, because a load touches more of the world than one set
/// of borrows can hold. The leaving world is written and drained here; the
/// world asked for is opened and prepared behind the loading screen, as a
/// launch opens one (`new-world-arrival`), and [`arrive`] finishes it on the
/// first open frame. It happens between frames rather than inside the press
/// that asked for it, which is also why `LoadRequest` exists at all.
fn load_world(world: &mut World) {
    if world.contains_resource::<pbd_app::WorldLoading>() {
        // One world opening at a time: the one in hand finishes first, and
        // a request waits for it.
        return;
    }
    let Some(slot) = world.resource_mut::<menu::LoadRequest>().0.take() else {
        return;
    };
    info!("WORLD_SWITCH before {}", switch_line(world));
    let name = slot.file.name.clone();
    // The world being left is written as the exit path writes it: where the
    // player stands and the hour, its weather, and its craft (which leave the
    // scene before the next world's arrive); then everything queued for it
    // goes down before the writer is let go. A half-written world is the one
    // thing a save must never leave behind.
    let pose = {
        let state = world.get_resource::<pbd_app::walking::WalkingState>();
        let view = state.map(pbd_app::walking::WalkingState::view);
        // Saved in the planet's frame, never local (`world-frames`).
        let o = pbd_app::sim_origin::origin_of(world);
        let mut walkers =
            world.query_filtered::<&avian3d::prelude::Position, With<pbd_app::walking::Walker>>();
        let position = walkers.iter(world).next().map(|p| o.query_point(p.0));
        view.zip(position).map(|((heading, pitch), position)| Pose {
            position,
            heading,
            pitch,
            selected: world.resource::<slots::Hotbar>().selected(),
        })
    };
    let seconds = world.resource::<pbd_app::sky::Sun>().clock.seconds;
    let weather = world
        .get_resource::<pbd_app::atmosphere::Air>()
        .map(|air| air.now.to_bytes());
    let craft = pbd_app::vehicles::put_away(world);
    let root = {
        let mut open = world.resource_mut::<WorldSave>();
        if let Some(pose) = pose {
            open.snapshot(pose, seconds);
        }
        if let Some(bytes) = weather {
            open.snapshot_weather(bytes);
        }
        if let Some(file) = craft {
            open.snapshot_vehicles(&file);
        }
        open.drain();
        open.root().to_path_buf()
    };
    // A world made by another generator than the planet's: the planet is
    // switched to it here, in place, before the thread builds it, and never by
    // relaunching (`bigger-biomes` decision 8; the owner: "Should not
    // restart"). The saves page refuses a version this build lacks, so a
    // refusal here keeps the planet as it is.
    let switched = match pbd_app::planet::switch_generator(saves::generator_of(&slot)) {
        Ok(switched) => switched,
        Err(why) => {
            warn!("{name}: {why}; the planet stays as it is");
            false
        }
    };
    let direction = world.resource::<NewWorldStart>().direction;
    world.insert_resource(SwitchNote { name, switched });
    pbd_app::loading::begin_switch(world, root, slot, direction);
}

/// Finish a switch on the first open frame, behind the loading screen: the
/// new world's save, planet, sites and towns are in place, and what the
/// binary owns is restored from the save, or dealt new.
fn arrive(world: &mut World) {
    let Some(arrived) = world.remove_resource::<pbd_app::loading::Arrived>() else {
        return;
    };
    let SwitchNote { name, switched } =
        world.remove_resource::<SwitchNote>().unwrap_or(SwitchNote {
            name: String::new(),
            switched: false,
        });
    let start = *world.resource::<NewWorldStart>();
    let hotbar =
        world.resource_scope(|_, mut save: Mut<WorldSave>| slots::Hotbar::restore(&mut save));
    let (tools, drops, pose, seconds) = {
        let save = world.resource::<WorldSave>();
        (
            pbd_app::fish::ToolSlot::restore(save),
            pbd_app::drops::Drops::restore(save),
            save.pose,
            save.world_seconds,
        )
    };
    // The world resumes in its season and at its hour; a new one starts on
    // the new-world clock, not the last world's (decision 5).
    world.resource_mut::<pbd_app::sky::Sun>().clock =
        seconds.map_or(start.clock, |seconds| pbd_core::daylight::Clock { seconds });
    // And under its own sky: the saved weather, or a new one spun up.
    let air = {
        let config = world.resource::<pbd_app::config::AtmosphereConfig>().0;
        let sun = *world.resource::<pbd_app::sky::Sun>();
        let mut air = pbd_app::atmosphere::Air::open(
            config,
            pbd_app::planet::terrain_config().seed,
            world.resource::<WorldSave>().weather.as_deref(),
            sun.clock.seconds,
        );
        air.in_place = !sun.running;
        air
    };
    world.insert_resource(air);
    world.insert_resource(hotbar);
    world.insert_resource(tools);
    world.insert_resource(drops);
    if switched && world.contains_resource::<map_screen::MapRaster>() {
        // The map's picture of the other planet goes with it.
        world.insert_resource(map_screen::MapRaster::default());
    }
    // The water of the world being left is not this world's water.
    if let Some(mut fishery) = world.get_resource_mut::<pbd_app::fish::Fishery>() {
        *fishery = pbd_app::fish::Fishery::default();
    }
    if world
        .get_resource::<pbd_app::walking::WalkingState>()
        .is_some()
    {
        match pose {
            Some(pose) => pbd_app::walking::restore(
                world,
                RestoredPose {
                    position: pose.position,
                    heading: pose.heading,
                    pitch: pose.pitch,
                },
            ),
            // A world nobody has played yet starts where a new world starts:
            // the launch's spawn, not where the last world's player stood.
            None => pbd_app::walking::arrive_new(world, start.direction),
        }
    }
    if let Some(pose) = pose {
        world.resource_mut::<slots::Hotbar>().select(pose.selected);
    }
    // The list shows which world is open, so it is drawn again now one is.
    world.resource_mut::<menu::SaveIndex>().set_changed();
    if world.resource::<WorldSave>().slot().is_some() {
        world.insert_resource(arrival::Arrival::new(&name, arrived.fresh));
    }
    info!(
        "loaded world '{name}' ({})",
        if arrived.fresh { "new" } else { "resumed" }
    );
    info!("WORLD_SWITCH after {}", switch_line(world));
}

/// One line of what world is open and where the player is in it, for the
/// `WORLD_SWITCH` log (`new-world-arrival`, a measurement instrument).
fn switch_line(world: &mut World) -> String {
    let save = world.resource::<WorldSave>();
    let slot = save.slot().map_or_else(
        || "memory-only".to_string(),
        |s| format!("'{}' ({})", s.file.name, s.id),
    );
    let saved = save.pose.map_or_else(
        || "none".to_string(),
        |p| {
            format!(
                "{:.1},{:.1},{:.1}",
                p.position.x, p.position.y, p.position.z
            )
        },
    );
    let head = format!(
        "world {slot} seed {} generator {} edits {} saved_pose {saved}",
        save.identity.seed,
        save.identity.generator,
        save.edits.len()
    );
    let o = pbd_app::sim_origin::origin_of(world);
    let mut walkers =
        world.query_filtered::<&avian3d::prelude::Position, With<pbd_app::walking::Walker>>();
    let walker = walkers.iter(world).next().map_or_else(
        || "none".to_string(),
        |p| {
            // As `--at` takes it, through the one place the screen reads.
            let p = o.query_point(p.0);
            let place = pbd_app::place::Place::of(p);
            format!(
                "{:.1},{:.1},{:.1} (lat {:.4} lon {:.4})",
                p.x, p.y, p.z, place.lat, place.lon
            )
        },
    );
    let spawn = world
        .get_resource::<pbd_app::walking::WalkingState>()
        .map_or_else(
            || "none".to_string(),
            |s| format!("{:.4?}", s.spawn_direction()),
        );
    let clock = world.resource::<pbd_app::sky::Sun>().clock.seconds;
    format!("{head} walker {walker} spawn_dir {spawn} clock_s {clock:.0}")
}

/// Write the pose on a timer, and whenever a menu opens.
///
/// Pose is the one part of a save that is genuinely cheap to lose, which is
/// exactly why it is the only part on a clock: the edits and the hotbar are
/// written per edit and are already down. A menu opening counts because the
/// player who opens one is usually the player about to quit.
#[allow(clippy::too_many_arguments)]
fn autosave(
    time: Res<Time>,
    screen: Res<menu::Screen>,
    walking: Option<Res<pbd_app::walking::WalkingState>>,
    slots: Res<slots::Hotbar>,
    walkers: Query<&avian3d::prelude::Position, With<pbd_app::walking::Walker>>,
    sun: Res<pbd_app::sky::Sun>,
    mut save: ResMut<WorldSave>,
    mut due: Local<f32>,
    space: pbd_app::SimSpace,
) {
    let opening = screen.is_changed() && *screen != menu::Screen::Playing;
    *due -= time.delta_secs();
    if !opening && *due > 0.0 {
        return;
    }
    *due = saves::AUTOSAVE_S;
    let (Some(state), Ok(position)) = (walking, walkers.single()) else {
        return;
    };
    let (heading, pitch) = state.view();
    save.snapshot(
        Pose {
            // The planet's frame, never local (`world-frames`).
            position: space.origin().query_point(position.0),
            heading,
            pitch,
            selected: slots.selected(),
        },
        sun.clock.seconds,
    );
}

/// Wait for the disk on the way out.
///
/// The one place blocking is right is the place the player is already
/// waiting: losing the last two digs to a quit would be the whole feature
/// failing at its most visible moment.
fn drain_saves(
    exits: MessageReader<AppExit>,
    air: Option<Res<pbd_app::atmosphere::Air>>,
    fleet: Option<Res<pbd_app::vehicles::Fleet>>,
    vehicles: Query<&pbd_app::vehicles::Vehicle>,
    mut save: ResMut<WorldSave>,
) {
    if !exits.is_empty() {
        if let Some(air) = air {
            save.snapshot_weather(air.now.to_bytes());
        }
        if let Some(file) = fleet.and_then(|f| pbd_app::vehicles::fleet_file(&f, &vehicles)) {
            save.snapshot_vehicles(&file);
        }
        save.drain();
    }
}

/// Write the atmosphere on its own, slower timer, and whenever a menu opens:
/// half a megabyte is not a thing to write every five seconds, and losing a
/// minute of weather costs a minute of sky.
fn save_weather(
    time: Res<Time>,
    screen: Res<menu::Screen>,
    air: Option<Res<pbd_app::atmosphere::Air>>,
    mut save: ResMut<WorldSave>,
    mut due: Local<f32>,
) {
    let opening = screen.is_changed() && *screen != menu::Screen::Playing;
    *due -= time.delta_secs();
    if !opening && *due > 0.0 {
        return;
    }
    *due = saves::WEATHER_SAVE_S;
    if let Some(air) = air {
        save.snapshot_weather(air.now.to_bytes());
    }
}

/// The clock a launch starts on: pinned where `--time` asked, and then the
/// log says where the sun stands from the spawn, as the `--yaw` and `--pitch`
/// that would centre it, so a sky capture is aimed rather than guessed.
fn sun_clock(launch: &Launch, saved_seconds: Option<f64>) -> pbd_core::daylight::Clock {
    use pbd_core::daylight::{Clock, START_HOUR};
    let clock = match (launch.day, launch.time, saved_seconds) {
        (None, None, Some(seconds)) => Clock { seconds },
        // A saved world asked for an hour: on to that hour, today or the next
        // day, never back. The world's own days stand (`mission-playthrough`:
        // a bot run started at 10:00 put the world back to day 0, so the wheat
        // sown on a later day could never ripen).
        (None, Some(hour), Some(seconds)) => {
            let now = Clock { seconds };
            let today = Clock::at(now.day() as u32, hour);
            if today.seconds >= seconds {
                today
            } else {
                Clock::at(now.day() as u32 + 1, hour)
            }
        }
        (day, hour, _) => Clock::at(day.unwrap_or(0), hour.unwrap_or(START_HOUR)),
    };
    if launch.day.is_some() {
        info!(
            "day {} of {}: the sun stands {:.1} deg off the equator",
            clock.day(),
            pbd_core::daylight::YEAR_DAYS,
            clock.declination().to_degrees()
        );
    }
    if launch.time.is_some() {
        let up = spawn_direction(launch);
        let heading = Vec3::Y.cross(up).normalize_or(Vec3::X);
        let right = heading.cross(up);
        let sun = clock.sun();
        let yaw = sun.dot(right).atan2(sun.dot(heading)).to_degrees();
        let pitch = sun.dot(up).clamp(-1.0, 1.0).asin().to_degrees();
        info!("sun from the spawn at --time: --yaw {yaw:.1} --pitch {pitch:.1}");
    }
    clock
}

/// The biome a `--spawn` name asks for, if it names one.
fn spawn_biome(name: &str) -> Option<pbd_core::planet_gen::Biome> {
    use pbd_core::planet_gen::Biome;
    Some(match name {
        "beach" => Biome::Beach,
        "fields" => Biome::Fields,
        "desert" => Biome::Desert,
        "jungle" => Biome::Jungle,
        "swamp" => Biome::Swamp,
        "mountains" => Biome::Mountains,
        "tundra" => Biome::Tundra,
        _ => return None,
    })
}

/// `count` directions spread evenly over the sphere: a Fibonacci sweep.
fn fibonacci_sphere(count: usize) -> impl Iterator<Item = Vec3> {
    let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
    (0..count).map(move |i| {
        let y = 1.0 - 2.0 * (i as f32 + 0.5) / count as f32;
        let r = (1.0 - y * y).max(0.0).sqrt();
        let a = golden * i as f32;
        Vec3::new(a.cos() * r, y, a.sin() * r)
    })
}

/// Where the walker, and with it the column tier, is anchored.
fn spawn_direction(launch: &Launch) -> Vec3 {
    let default = Vec3::new(0.8776, 0.4794, 0.0).normalize();
    // The river run starts in the river (owner, 2026-11-18: "You should
    // start out spawn in middle of a river. You know the map already"): the
    // drainage map's river nearest `--at` (or the default spawn), 15 m down
    // its line from its source, in its water.
    if launch.walk_river {
        let near = launch.at.map_or(default, |(lat, lon)| {
            pbd_core::geo::direction(pbd_core::geo::LatLon {
                lat: lat.to_radians(),
                lon: lon.to_radians(),
            })
        });
        let cfg = pbd_app::planet::terrain_config();
        if let Some(settings) = cfg.rivers.as_ref()
            && let Some(source) = pbd_core::rivers::map(cfg, settings)
                .course(near, 40)
                .first()
            && let Some(path) = pbd_app::river_walk::RiverPath::through(source.centre, 1.0)
        {
            let start = path.line_at(15.0);
            info!(
                "RIVER_WALK spawn in the river 15 m from its source, {}",
                pbd_app::place::Place::of(start * PLANET_RADIUS).line()
            );
            return start;
        }
    }
    if let Some((lat, lon)) = launch.at {
        return pbd_core::geo::direction(pbd_core::geo::LatLon {
            lat: lat.to_radians(),
            lon: lon.to_radians(),
        });
    }
    if launch.tour && launch.view == "pole" {
        return Vec3::Y;
    }
    if launch.view == "seacave" {
        // The tier has to be somewhere that HAS caves below sea level, and
        // the spawn is seventy-four metres up. The shore is where the ground
        // meets the waterline; `nearest_ground_near` is the same locator the
        // change's measuring test uses, so the picture and the numbers are
        // taken of one place.
        if let Some(shore) = pbd_app::planet::nearest_ground_near(default, 0.5..4.0) {
            info!(
                "seacave spawn moved {:.0} m to the shore",
                shore.dot(default).clamp(-1., 1.).acos() * PLANET_RADIUS
            );
            return shore;
        }
    }
    if launch.spawn.as_deref() == Some("snow") {
        // The nearest dry land where the field's precipitation is snow, so a
        // capture can photograph a snowfall: a Fibonacci sweep of the sphere,
        // nearest first. A measurement instrument, like `--weather-at`.
        let found = fibonacci_sphere(20_000)
            .filter(|d| {
                pbd_app::planet::surface_height(*d) > 1.0
                    && matches!(
                        pbd_core::planet_gen::biome(pbd_app::planet::terrain_config(), *d),
                        pbd_core::planet_gen::Biome::Tundra
                            | pbd_core::planet_gen::Biome::Mountains
                    )
            })
            .max_by(|a, b| a.dot(default).total_cmp(&b.dot(default)));
        if let Some(snow) = found {
            info!(
                "spawn moved {:.0} m to the nearest snowfield",
                snow.dot(default).clamp(-1., 1.).acos() * PLANET_RADIUS
            );
            return snow;
        }
        warn!("no snowfield found; spawning at the default");
    }
    if let Some(wanted) = launch.spawn.as_deref().and_then(spawn_biome) {
        // The nearest dry land inside the named biome, with the same biome
        // 40 m round it on four sides where there is such a place, so a
        // capture photographs the biome from inside it rather than from its
        // edge (`bigger-biomes` 4.1). A measurement instrument, like
        // `--spawn snow`.
        let config = pbd_app::planet::terrain_config();
        // At or above the water, not a metre over it as `snow` asks: a beach
        // is the sand within two metres of the sea.
        let inside = |d: Vec3| {
            pbd_app::planet::surface_height(d) >= 0.0
                && pbd_core::planet_gen::biome(config, d) == wanted
        };
        let around = |d: Vec3| {
            let (a, b) = d.any_orthonormal_pair();
            let step = 40.0 / PLANET_RADIUS;
            [a, -a, b, -b]
                .into_iter()
                .all(|t| inside((d + t * step).normalize()))
        };
        // A swamp is a strip of low wet ground, rarely 80 m across, so when
        // no place has room round it the nearest one of any size will do.
        let nearest = |roomy: bool| {
            fibonacci_sphere(20_000)
                .filter(|d| inside(*d) && (!roomy || around(*d)))
                .max_by(|a, b| a.dot(default).total_cmp(&b.dot(default)))
        };
        let found = nearest(true).or_else(|| nearest(false));
        if let Some(place) = found {
            info!(
                "spawn moved {:.0} m into the nearest {wanted:?}",
                place.dot(default).clamp(-1., 1.).acos() * PLANET_RADIUS
            );
            return place;
        }
        warn!("no {wanted:?} found; spawning at the default");
    }
    if launch.spawn.as_deref() == Some("mouth") || launch.view == "mouth" {
        // The nearest worm that starts at the surface within a kilometre.
        let columns = pbd_app::config::ColumnSettings::default();
        let found = pbd_core::worms::gather(
            &columns.worms(),
            pbd_app::planet::terrain_config(),
            default,
            1_000.0,
        )
        .nearest_opening(default);
        match found {
            Some(mouth) => {
                info!(
                    "spawn moved {:.0} m to the nearest cave mouth",
                    mouth.dot(default).clamp(-1., 1.).acos() * PLANET_RADIUS
                );
                return mouth;
            }
            None => warn!("no cave mouth within a kilometre of the spawn; spawning at the default"),
        }
    }
    default
}

/// The window's size in points: 1440 x 900, or `PBD_WINDOW=<w>x<h>`, a
/// measurement switch (`frame-budget` 5c) that prices the pixel count with the
/// same view. It is not a render scale: the UI shrinks with it.
fn window_size() -> (u32, u32) {
    std::env::var("PBD_WINDOW")
        .ok()
        .and_then(|v| {
            let (w, h) = v.trim().split_once('x')?;
            Some((w.parse().ok()?, h.parse().ok()?))
        })
        .filter(|&(w, h): &(u32, u32)| (64..=8192).contains(&w) && (64..=8192).contains(&h))
        .unwrap_or((1440, 900))
}

/// The camera's samples: none, or `PBD_MSAA=<1|2|4>`, a measurement switch
/// (`frame-budget` 5c). Off by default (owner, 2026-11-20: "AA should be a
/// final composite thing, but I didnt even ask for AA to be implemented"):
/// the water's passes run after the resolve on blended edge pixels, which
/// put a pale line round everything seen under the water
/// (`ocean-environments` B24), and the pixel look wants hard edges. The
/// player's ANTIALIASING is SMAA on the finished frame instead
/// (`settings::follow_antialiasing`), and is never added to a camera that
/// multisamples.
fn msaa_samples() -> Msaa {
    match std::env::var("PBD_MSAA").ok().as_deref().map(str::trim) {
        Some("2") => Msaa::Sample2,
        Some("4") => Msaa::Sample4,
        _ => Msaa::Off,
    }
}

fn configure_camera(
    mut commands: Commands,
    mut cameras: Query<(Entity, &mut Camera3d), Added<Camera3d>>,
) {
    for (entity, mut camera) in &mut cameras {
        // The water composite reads the main pass depth; Bevy only allocates
        // a sampleable depth texture when a camera asks for one.
        let usages =
            TextureUsages::from(camera.depth_texture_usages) | TextureUsages::TEXTURE_BINDING;
        camera.depth_texture_usages = usages.into();
        commands.entity(entity).insert((
            Hdr,
            Tonemapping::TonyMcMapface,
            msaa_samples(),
            Projection::Perspective(PerspectiveProjection {
                near: 0.05,
                far: 500_000.0,
                fov: 60.0_f32.to_radians(),
                ..default()
            }),
        ));
    }
}

/// Stand inside the world: the two frames the voxel column tier exists to
/// produce, and neither could be taken before it.
///
/// It reads the LIVE TIER rather than calling the generator at a direction of
/// its own. That is not tidiness, it is the whole reason the first ten captures
/// were wrong: a column belongs to a CELL and is generated at that cell's own
/// centre, so a chamber found at an arbitrary point 1.4 m away need not exist
/// in the cell that is actually drawn. The camera stood inside solid rock, and
/// from inside rock nothing draws a face toward you, so the frame came back
/// showing the whole world from impossible angles - which reads exactly like a
/// renderer full of holes.
/// Whether this capture view's camera is placed a schedule later, off the
/// LIVE column tier, rather than from a direction and an altitude.
///
/// One list, because there were two and they disagreed: `seacave` was added
/// to the one `cave_camera` reads and not to the one `capture_camera` reads,
/// so BOTH spawned a camera and the frame came out of whichever won - a wide
/// shot of the shore from four hundred metres up, with the cave camera
/// standing in a chamber nobody photographed. `mouth` had been in the same
/// state since it landed.
fn placed_off_the_tier(view: &str) -> bool {
    ["cave", "overhang", "mouth", "seacave"].contains(&view)
}

/// The chamber a cave capture looks down: open air two and a half to twelve
/// metres tall, four to forty metres under the ground, and of those the one
/// with the longest open tunnel ahead of it. Its cell's direction, floor and
/// roof altitudes and the tunnel's heading, with how many cells it runs.
fn cave_chamber(
    fine: &pbd_app::planet::PlanetFine,
    dry_cave_below_the_sea: bool,
) -> Option<(usize, Vec3, f32, f32, Vec3)> {
    use pbd_core::column::{layer_altitude, layer_at};
    let tier = &fine.set.columns;
    let records = fine.set.finest_records();
    // Air at this altitude in this cell's column, which is the one question
    // both the pick and the aim ask.
    let air = |cell: usize, altitude: f32| -> bool {
        let slot = tier.slots.get(cell).copied().unwrap_or(usize::MAX);
        if slot == usize::MAX {
            return false;
        }
        layer_at(altitude).is_some_and(|layer| !tier.columns[slot].solid(layer))
    };
    // The chamber a player can SEE DOWN, not the one with the most rock over
    // it. Burial was the sheet carve's pick and it was right for a slab, where
    // every chamber is the same two metres across and depth is all that is
    // left to choose by; with worms the frames differ by whether the tunnel
    // carries on, so the pick is the sight line, which is the instrument the
    // carve is measured with applied at capture time. Walking cell to cell,
    // always taking the neighbour most nearly ahead.
    let walk = |start: usize, heading: Vec3, altitude: f32| -> (usize, Vec3) {
        let mut cell = start;
        let mut at = Vec3::from_slice(&records[start].direction_height[..3]);
        let mut ahead = heading;
        for step in 1..=24 {
            let mut next: Option<(f32, usize, Vec3)> = None;
            for (side, &neighbor) in fine.set.finest_neighbors[cell]
                .iter()
                .enumerate()
                .take(records[cell].degree())
            {
                if neighbor == u32::MAX {
                    continue;
                }
                let there = Vec3::from_slice(&records[neighbor as usize].direction_height[..3]);
                let toward = (there - at).normalize_or_zero();
                let score = toward.dot(ahead);
                let _ = side;
                if next.is_none_or(|(had, ..)| score > had) {
                    next = Some((score, neighbor as usize, toward));
                }
            }
            let Some((_, neighbor, toward)) = next else {
                return (step - 1, ahead);
            };
            if !air(neighbor, altitude) {
                return (step - 1, ahead);
            }
            cell = neighbor;
            at = Vec3::from_slice(&records[neighbor].direction_height[..3]);
            ahead = toward;
        }
        (24, ahead)
    };
    let mut best: Option<(usize, Vec3, f32, f32, Vec3)> = None;
    for (index, &slot) in tier.slots.iter().enumerate() {
        if slot == usize::MAX {
            continue;
        }
        let column = &tier.columns[slot];
        let runs = column.drawn_runs();
        let surface = column
            .surface()
            .map_or(0.0, |top| layer_altitude(top) + 1.0);
        for pair in runs.windows(2) {
            let floor = layer_altitude(pair[0].to);
            let roof = layer_altitude(pair[1].from);
            let gap = roof - floor;
            let buried = surface - roof;
            if !(2.5..=12.0).contains(&gap) || !(4.0..=40.0).contains(&buried) {
                continue;
            }
            if dry_cave_below_the_sea && (roof >= 0.0 || surface <= 0.0) {
                continue;
            }
            let here = Vec3::from_slice(&records[index].direction_height[..3]);
            let eye_altitude = floor + EYE_HEIGHT.min(gap - 0.5);
            let tangent = Vec3::Y.cross(here).normalize_or_zero();
            let bitangent = here.cross(tangent);
            for turn in 0..6 {
                let angle = turn as f32 * std::f32::consts::TAU / 6.0;
                let heading = tangent * angle.cos() + bitangent * angle.sin();
                let (reach, _) = walk(index, heading, eye_altitude);
                if best.is_none_or(|(had, ..)| reach > had) {
                    best = Some((reach, here, floor, roof, heading));
                }
            }
        }
    }
    best
}

fn cave_camera(
    mut commands: Commands,
    launch: Res<Launch>,
    fine: Res<pbd_app::planet::PlanetFine>,
) {
    if launch.capture.is_none() || launch.tour || launch.walk || launch.fly {
        return;
    }
    if !placed_off_the_tier(&launch.view) {
        return;
    }
    // The volumetric-water case is a chamber below sea level UNDER LAND: the
    // column holds no water at all, so the pocket is air, and only a height
    // field would call it sea. A pocket under the SEA is not it - the column
    // there really does hold water over the rock - and the first cut of this
    // pick took one, which came out as a frame of open ocean.
    let best = cave_chamber(&fine, launch.view == "seacave");
    let tier = &fine.set.columns;
    let records = fine.set.finest_records();
    if launch.view == "mouth" {
        // Stand on the ground outside a tunnel opening and look into it. What
        // counts as an opening is `planet_column::mouth_of`, the same function
        // the mouth count and the walker read, so a frame cannot be taken of
        // something the count does not call a mouth.
        let mut best: Option<(f32, usize, usize, f32, f32)> = None;
        for (index, &slot) in tier.slots.iter().enumerate() {
            if slot == usize::MAX {
                continue;
            }
            let runs = tier.columns[slot].drawn_runs();
            let cell = &records[index];
            let Some((floor, roof, side)) = pbd_app::planet::mouth_of(cell, &runs) else {
                continue;
            };
            let neighbor = fine.set.finest_neighbors[index][side];
            if neighbor == u32::MAX || roof - floor < 1.8 {
                continue;
            }
            // The deepest tunnel behind the opening is the one worth looking
            // into: a one-cell notch is a doorway with a wall behind it.
            let depth = roof - floor;
            if best.is_none_or(|(had, ..)| depth > had) {
                best = Some((depth, index, neighbor as usize, floor, roof));
            }
        }
        let Some((depth, index, outside, floor, roof)) = best else {
            panic!(
                "no tunnel opening in the column tier; use --spawn mouth or raise worm_surface_share"
            )
        };
        let inside = Vec3::from_slice(&records[index].direction_height[..3]);
        let ground = Vec3::from_slice(&records[outside].direction_height[..3]);
        info!(
            "mouth capture: a {depth:.0} m opening, floor {floor:.0} m, roof {roof:.0} m, from ground at {:.0} m",
            records[outside].direction_height[3]
        );
        // Three cells back from the opening on the outside ground, at eye
        // height THERE: the ground a cell back is another cell's, and a frame
        // that stood at the outside cell's height from a cell back was inside
        // the rock of that cell, seeing the world from below through its cap.
        let back = (ground - inside).normalize_or_zero();
        let vantage = (ground + back * (4.0 / PLANET_RADIUS)).normalize();
        let there = pbd_app::planet::surface_height(vantage);
        let eye = vantage * (PLANET_RADIUS + there + EYE_HEIGHT);
        // Aim at the middle of the opening, which is where a walker's eyes go.
        let target = inside * (PLANET_RADIUS + (floor + roof) * 0.5 + EYE_HEIGHT * 0.5);
        let mut transform = Transform::from_translation(eye).looking_at(target, vantage);
        transform.translation += launch.render_offset;
        commands.spawn((Camera3d::default(), transform));
        return;
    }

    let Some((reach, here, floor, roof, along)) = best else {
        panic!("no chamber in the column tier to photograph; raise reach_m or worm_density")
    };
    let gap = roof - floor;
    info!(
        "cave capture: a {gap:.0} m chamber with {reach} cells of open tunnel ahead, floor \
         {floor:.0} m, roof {roof:.0} m, {:.0} m from the tier anchor",
        here.dot(fine.set.anchor).clamp(-1., 1.).acos() * PLANET_RADIUS
    );
    // `cave` stands on the chamber floor at eye height and looks along the
    // tunnel; `overhang` looks UP at the rock over it, which is the frame that
    // says the world has a ceiling.
    let eye_altitude = floor + EYE_HEIGHT.min(gap - 0.5);
    let eye = here * (PLANET_RADIUS + eye_altitude);
    let target = if launch.view == "overhang" {
        // The roof EIGHT metres down the tunnel, not the one overhead: a
        // ceiling a metre above the lens is one flat plane at a grazing angle,
        // which is a grey wash rather than a picture of a roof. Eight metres
        // out it recedes and the walls either side give it depth.
        (here + along * (8.0 / PLANET_RADIUS)).normalize() * (PLANET_RADIUS + roof - 0.3)
    } else {
        (here + along * (12.0 / PLANET_RADIUS)).normalize() * (PLANET_RADIUS + eye_altitude)
    };
    let mut transform = Transform::from_translation(eye).looking_at(target, here);
    transform.translation += launch.render_offset;
    commands.spawn((Camera3d::default(), transform));
}

fn photo_camera(
    mut commands: Commands,
    launch: Res<Launch>,
    water_settings: Res<pbd_app::config::WaterSettings>,
    clock: Res<pbd_app::sky::Sun>,
) {
    if launch.capture.is_none() || launch.tour || launch.walk || launch.fly {
        return;
    }
    // The column-tier views are placed a schedule later, off the live tier.
    if placed_off_the_tier(&launch.view) {
        return;
    }
    if launch.view == "river" {
        // A river is a channel pulled to a bed below the sea on lowland only,
        // so it is water with LAND on both sides a short way off - which is
        // what tells one from a bay. Walk the spawn's latitude for a wet cell
        // whose banks stand clear within sixty metres, and stand on one.
        // One latitude circle does not reliably cross a channel, so search the
        // sphere and take the one nearest the spawn: a wet cell whose banks
        // both stand clear thirty metres off, on each of two axes, which is a
        // watercourse rather than the edge of a bay.
        let seed = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        let bank = 30.0 / PLANET_RADIUS;
        let golden = std::f32::consts::PI * (3.0 - 5_f32.sqrt());
        let samples = 300_000;
        let mut best: Option<(f32, Vec3, Vec3)> = None;
        for i in 0..samples {
            let y = 1.0 - 2.0 * (i as f32 + 0.5) / samples as f32;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let t = golden * i as f32;
            let here = Vec3::new(r * t.cos(), y, r * t.sin());
            let toward = here.dot(seed);
            let depth = surface_height(here);
            // The generator's own carve, not a guess from heights: a shallow
            // wet cell where the river field fires. Both banks must stand
            // clear on both axes without being a cliff, or this is a canyon
            // floor or the edge of a bay.
            if toward < 0.85 || !(-4.0..0.0).contains(&depth) {
                continue;
            }
            if !pbd_core::planet_gen::in_river(terrain_config(), here) {
                continue;
            }
            let (a, b) = here.any_orthonormal_pair();
            let bankside =
                |axis: Vec3, sign: f32| surface_height((here + axis * bank * sign).normalize());
            if [(a, 1.0), (a, -1.0), (b, 1.0), (b, -1.0)]
                .into_iter()
                .any(|(axis, sign)| !(0.5..10.0).contains(&bankside(axis, sign)))
            {
                continue;
            }
            if best.is_none_or(|(near, _, _)| toward > near) {
                best = Some((toward, here, a));
            }
        }
        let (_, channel, across_axis) = best.unwrap_or((1.0, seed, Vec3::X));
        // Stand back on the near bank and look across the water.
        let here = (channel - across_axis * bank * 0.8).normalize();
        let across = (channel + across_axis * bank * 0.9).normalize();
        let height = launch.height.unwrap_or(EYE_HEIGHT);
        let eye = here * (terrain_radius(here) + height);
        let mut transform =
            Transform::from_translation(eye).looking_at(across * terrain_radius(across), here);
        transform.translation += launch.render_offset;
        commands.spawn((Camera3d::default(), transform));
        return;
    }
    if launch.view == "meadow" {
        // Every other ground preset stands at the spawn, and the spawn is in
        // jungle: a closed canopy at the reference's 115-of-256, which is
        // under three percent of this world. Pasture is a third of it and no
        // preset could photograph it. Walk east along the spawn's latitude to
        // the first pasture cell well clear of the shore and stand there.
        let seed = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        let lat = seed.y.clamp(-1.0, 1.0).asin();
        let at = |lon: f32| Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
        let step = 4.0 * tile_width_m(FINEST_LEVEL) / PLANET_RADIUS;
        let mut lon = seed.z.atan2(seed.x);
        let start = lon;
        while lon < start + std::f32::consts::TAU {
            let here = at(lon);
            let height = surface_height(here);
            if height > 8.0 && surface_code(here, height) & 0xff == 2 {
                break;
            }
            lon += step;
        }
        let ground = at(lon);
        let height = launch.height.unwrap_or(EYE_HEIGHT);
        let east = (at(lon + step) - ground).normalize_or_zero();
        let eye = ground * (terrain_radius(ground) + height);
        let look = at(lon + 20.0 * step);
        let mut transform = Transform::from_translation(eye)
            .looking_at(look * terrain_radius(look) + east * height, ground);
        transform.translation += launch.render_offset;
        commands.spawn((Camera3d::default(), transform));
        return;
    }
    if ["shore", "nightshore", "midnight", "wade", "dive"].contains(&launch.view.as_str()) {
        // A capture instrument, nothing more: the eye-height polar shoreline the
        // owner asked to see. Above ~70 N the polar snow line reaches the sea,
        // so walk east from 72 N until land meets water, stand on the last land
        // cell at eye height, and look at the sea surface on the first water
        // cell. Bounded to one circuit so a latitude with no coast cannot spin.
        // `--height` lifts the eye and pushes the aim point out to sea by the
        // same distance, so every height in a series looks down at about 45
        // degrees instead of straight down.
        // `nightshore` is the same instrument on the night side, found from
        // the sun where the clock puts it - the one sun, pinned by `--time` -
        // so the night it frames is the night the sky and the lamps are
        // drawn for. The equator's antisolar longitude is the deepest night
        // the equator has, so that is where this one starts. `midnight` is
        // the antisolar POINT itself: off the equator the sun's latitude
        // leaves the equator's antisolar longitude short of the sun's
        // opposite, its sky lit by the upper atmosphere over the limb; only
        // at the antisolar point is the sky black in every direction, which
        // is the frame the owner's night report was taken in.
        let night = launch.view == "nightshore" || launch.view == "midnight";
        let sun = clock.clock.sun();
        let lat = if launch.view == "midnight" {
            (-sun.y).clamp(-1.0, 1.0).asin()
        } else if night {
            0.0
        } else {
            72_f32.to_radians()
        };
        let at = |lon: f32| Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
        let step = 2.0 * tile_width_m(FINEST_LEVEL) / PLANET_RADIUS;
        let mut lon = if night { (-sun.z).atan2(-sun.x) } else { 0.0 };
        let start = lon;
        while surface_height(at(lon)) < 0.0 && lon < start + std::f32::consts::TAU {
            lon += step;
        }
        while surface_height(at(lon)) >= 0.0 && lon < start + 2.0 * std::f32::consts::TAU {
            lon += step;
        }
        let land = at(lon - step);
        let shore = at(lon);
        // `dive` needs water deep enough to hold the camera. The rescaled
        // shelf is one metre deep and stays under three for hundreds of
        // metres, so a fixed descent below the first water cell puts the eye
        // inside the seabed and the frame is flat deep-water colour with no
        // seabed, no surface and no Snell's window in it. Keep walking out
        // until the floor clears the requested depth. Bounded like the walk
        // above, and it falls back to the deepest cell the walk found.
        let depth = launch.height.unwrap_or(3.0);
        let water = if launch.view == "dive" {
            let clearance = -(depth + 1.0);
            let mut out = lon;
            let mut deepest = lon;
            while out < lon + std::f32::consts::TAU {
                let here = surface_height(at(out));
                if here <= clearance {
                    break;
                }
                if here < surface_height(at(deepest)) {
                    deepest = out;
                }
                out += step;
            }
            at(if surface_height(at(out)) <= clearance {
                out
            } else {
                deepest
            })
        } else {
            shore
        };
        let east = (shore - land).normalize_or_zero();
        // The sheet sits below sea level by the configured offset; `wade` puts
        // the eye inside the surface band over the first water cell, and
        // `dive` `--height` metres under over the first cell deep enough,
        // both looking out to sea.
        let sheet = PLANET_RADIUS - water_settings.depth_offset_m;
        let (eye, sea) = match launch.view.as_str() {
            "wade" => (water * (sheet + 0.1), water * sheet + east * 40.0),
            "dive" => (
                water * (sheet - depth),
                water * (sheet - depth - 1.0) + east * 40.0,
            ),
            _ => {
                let height = launch.height.unwrap_or(EYE_HEIGHT);
                (
                    land * (terrain_radius(land) + height),
                    water * terrain_radius(water) + east * height,
                )
            }
        };
        let mut transform = Transform::from_translation(eye).looking_at(sea, land);
        transform.translation += launch.render_offset;
        commands.spawn((Camera3d::default(), transform));
        return;
    }
    if launch.view == "column" {
        // A camera standing straight over the spawn at `--height` metres
        // above its ground, facing compass west (`Y x up`, as a walker's
        // default heading does; `compass-bar`) and tilted by `--pitch` (-89 looks
        // straight down): a descent through the weather is this view at a
        // run of heights. The weather's "here" is the camera's own direction,
        // so a forced storm (`--rain`) brews directly under it at any height.
        // The spawn is `spawn_direction`'s, so `--spawn desert` stands it over
        // a desert (`bigger-biomes` 4.1); without `--spawn` it is the default.
        // `--yaw` turns it to the right as it turns a walker: 90 faces
        // compass north, 180 east.
        let direction = spawn_direction(&launch);
        let height = launch.height.unwrap_or(EYE_HEIGHT);
        let position = direction * (terrain_radius(direction) + height);
        let yaw = launch.yaw.unwrap_or(0.0).to_radians();
        let east =
            Quat::from_axis_angle(direction, -yaw) * Vec3::Y.cross(direction).normalize_or_zero();
        let pitch = launch.pitch.unwrap_or(-30.0).to_radians();
        let forward = east * pitch.cos() + direction * pitch.sin();
        let mut transform = Transform::from_translation(position).looking_to(forward, direction);
        transform.translation += launch.render_offset;
        commands.spawn((Camera3d::default(), transform));
        return;
    }
    let (direction, altitude, look_down) = match launch.view.as_str() {
        "orbit" => (Vec3::new(0.85, 0.48, 1.05).normalize(), 7800.0, true),
        "night" => (Vec3::new(-0.75, 0.12, 0.8).normalize(), 6200.0, true),
        "pole" => (Vec3::new(0.01, 1.0, 0.05).normalize(), 4200.0, true),
        "surface" => (Vec3::new(0.8776, 0.4794, 0.0).normalize(), 90.0, false),
        // A low eye at the spawn, looking down across the 300 m band boundary
        // at about eleven degrees: the still frame the hexagon-lod change asks
        // for. Higher than eye level so the spawn's own terraces and trees do
        // not fill the frame.
        "seam" => (Vec3::new(0.8776, 0.4794, 0.0).normalize(), 60.0, false),
        _ => (Vec3::new(0.8776, 0.4794, 0.0).normalize(), 420.0, false),
    };
    let position = direction * (terrain_radius(direction) + altitude);
    let tangent = Vec3::Y.cross(direction).normalize_or_zero();
    let target = if look_down {
        Vec3::ZERO
    } else if altitude < 80.0 {
        position + tangent * 300.0 - direction * altitude
    } else {
        position + tangent * 1600.0 - direction * if altitude > 300.0 { 900.0 } else { 150.0 }
    };
    let up = if look_down { Vec3::Y } else { direction };
    let mut transform = Transform::from_translation(position).looking_at(target, up);
    transform.translation += launch.render_offset;
    commands.spawn((Camera3d::default(), transform));
}

/// How far the `--walk-distance` walk has come, for the frame log.
#[derive(Resource, Default)]
struct WalkProgress {
    walked_m: f32,
    started: Option<Instant>,
    last: Option<Vec3>,
    /// Where the walker was when the stall clock last reset, and when.
    anchor: Option<(Vec3, Instant)>,
    turns: u32,
    done: bool,
}

/// The `--walk-distance` instrument: sprint forward, measure the path actually
/// walked, jump when stalled, dig ahead after 2.5 s, turn 45 degrees after 7, and
/// quit at the distance.
// Eight parameters: the instrument drives keys, mouse and tool, reads the
// walker, and quits; a struct of them would be a struct with one caller.
#[allow(clippy::too_many_arguments)]
fn walk_script(
    launch: Res<Launch>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut state: ResMut<pbd_app::walking::WalkingState>,
    mut progress: ResMut<WalkProgress>,
    walkers: Query<&Position, With<pbd_app::walking::Walker>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut tools: ResMut<pbd_app::fish::ToolSlot>,
    mut exit: MessageWriter<AppExit>,
    space: pbd_app::SimSpace,
) {
    let Some(goal) = launch.walk_distance else {
        return;
    };
    if !state.active || progress.done {
        return;
    }
    let Ok(position) = walkers.single() else {
        return;
    };
    // The instrument steers by planet points (`world-frames`).
    let position = &Position(space.origin().query_point(position.0));
    let here = position.0;
    let now = Instant::now();
    state.captured = true;
    state.scripted = true;
    let started = *progress.started.get_or_insert_with(|| {
        info!("WALK_START {goal:.0} m");
        now
    });
    // The pickaxe in hand, so a stall can dig its way out.
    tools.hold(pbd_core::inventory::Tool::Pickaxe);
    if let Some(last) = progress.last {
        // Ground covered: the great-circle distance between the two points'
        // directions, so a fall or a jump is not walking. A teleport (a rebase
        // or a respawn) is not walking either.
        let step = here
            .normalize_or(Vec3::Y)
            .angle_between(last.normalize_or(Vec3::Y))
            * pbd_app::planet::PLANET_RADIUS;
        if step < 5.0 {
            progress.walked_m += step;
        }
    }
    progress.last = Some(here);
    keys.press(KeyCode::KeyW);
    // MEASUREMENT: a slow walk (`PBD_WALK_AXIS_SCALE`) walks, not sprints.
    if std::env::var_os("PBD_WALK_AXIS_SCALE").is_none() {
        keys.press(KeyCode::ShiftLeft);
    }
    let (anchor, since) = *progress.anchor.get_or_insert((here, now));
    if here.distance(anchor) > 4.0 {
        progress.anchor = Some((here, now));
        mouse.release(MouseButton::Left);
    } else {
        let stalled = now.duration_since(since).as_secs_f32();
        // Stuck, as a player gets unstuck: jump (a tap every 0.7 s, released
        // between so each counts), then dig through what is in front with the
        // pickaxe, and only then turn. (Fly is F, not the jump key.) Breaking
        // is a HOLD (`dig_and_place`): the button stays down from 2.5 s on, as
        // the owner put it, "gotta hold mouse down to break blocks". Tapped,
        // it only ever started a crack and let go.
        let pulse = |period: f32| (stalled % period) < 0.05;
        if stalled > 0.6 && pulse(0.7) {
            keys.press(KeyCode::Space);
        } else {
            keys.release(KeyCode::Space);
        }
        if stalled > 2.5 {
            mouse.press(MouseButton::Left);
        } else {
            mouse.release(MouseButton::Left);
        }
        if stalled > 7.0 {
            let up = here.normalize_or(Vec3::Y);
            let ahead = state.view().0;
            let turned = Quat::from_axis_angle(up, std::f32::consts::FRAC_PI_4) * ahead;
            state.face(up, turned);
            progress.turns += 1;
            progress.anchor = Some((here, now));
            info!("WALK_TURN stalled 7 s at {:.0} m walked", progress.walked_m);
        }
    }
    if progress.walked_m >= goal {
        progress.done = true;
        info!(
            "WALK_DONE {:.0} m in {:.1} s, {} turns",
            progress.walked_m,
            now.duration_since(started).as_secs_f32(),
            progress.turns
        );
        exit.write(AppExit::Success);
    }
}

/// Metres along the ground between two points' directions. From the chord
/// between the unit directions, not `angle_between`: its `acos` near 1 in
/// `f32` reads a step under a metre as 0 or 1.66 m, which made a walk's count
/// two to three times the ground it covered.
fn ground_m(a: Vec3, b: Vec3) -> f32 {
    let chord = a.normalize_or(Vec3::Y).distance(b.normalize_or(Vec3::Y));
    2.0 * (chord * 0.5).min(1.0).asin() * pbd_app::planet::PLANET_RADIUS
}

/// The `--stand` instrument: hold still where the walker was put, for the
/// frame log, and quit after that long.
fn stand_script(
    launch: Res<Launch>,
    mut state: ResMut<pbd_app::walking::WalkingState>,
    walkers: Query<(), With<pbd_app::walking::Walker>>,
    mut exit: MessageWriter<AppExit>,
    mut started: Local<Option<Instant>>,
    mut done: Local<bool>,
) {
    let Some(seconds) = launch.stand else {
        return;
    };
    if !state.active || *done || walkers.single().is_err() {
        return;
    }
    state.captured = true;
    state.scripted = true;
    let since = *started.get_or_insert_with(|| {
        info!("STAND_START {seconds:.0} s");
        Instant::now()
    });
    if since.elapsed().as_secs_f32() >= seconds {
        *done = true;
        info!("STAND_DONE {seconds:.0} s");
        exit.write(AppExit::Success);
    }
}

/// The `--stroll` instrument: walk ahead at walking pace for a video, hop
/// what stalls it, turn a quarter after three seconds stuck, never dig.
// Ten parameters, `walk_script`'s reason: an instrument that drives keys,
// reads the walker and the towns, and quits.
#[allow(clippy::too_many_arguments)]
fn stroll_script(
    launch: Res<Launch>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut state: ResMut<pbd_app::walking::WalkingState>,
    walkers: Query<&Position, With<pbd_app::walking::Walker>>,
    mut exit: MessageWriter<AppExit>,
    mut clock: Local<Option<(Instant, Vec3, Instant)>>,
    mut legs: Local<u32>,
    towns: Option<Res<pbd_app::towns::Towns>>,
    mut stood: Local<Option<Instant>>,
    mut stood_log: Local<Option<Instant>>,
    space: pbd_app::SimSpace,
) {
    let stood_log = stood_log.get_or_insert_with(Instant::now);
    let Some(seconds) = launch.stroll else {
        return;
    };
    if !state.active {
        return;
    }
    let Ok(position) = walkers.single() else {
        return;
    };
    // The instrument steers by planet points (`world-frames`).
    let position = &Position(space.origin().query_point(position.0));
    // Not before the walker stands in a town that stands (within 150 m of
    // its anchor), and a second after: towns stand from kilometres off, and
    // the walker is put at `--at` only once its town is up.
    if clock.is_none() {
        let near = towns.as_ref().is_some_and(|t| {
            t.standing
                .iter()
                .any(|s| ground_m(s.anchor, position.0) < 150.0)
        });
        if !near {
            *stood = None;
            if towns.as_ref().is_none_or(|t| t.standing.is_empty())
                && stood_log.elapsed().as_secs_f32() > 5.0
            {
                *stood_log = Instant::now();
                info!("STROLL_WAIT no town stands yet, {:?}", position.0);
            }
            if let Some(t) = towns.as_ref()
                && let Some((d, name)) = t
                    .standing
                    .iter()
                    .map(|s| (ground_m(s.anchor, position.0), s.name.as_str()))
                    .min_by(|a, b| a.0.total_cmp(&b.0))
                && stood_log.elapsed().as_secs_f32() > 5.0
            {
                *stood_log = Instant::now();
                info!("STROLL_WAIT {d:.0} m from {name}");
            }
            return;
        }
        let since = *stood.get_or_insert_with(Instant::now);
        if since.elapsed().as_secs_f32() < 1.0 {
            return;
        }
    }
    let here = position.0;
    let now = Instant::now();
    state.captured = true;
    state.scripted = true;
    let nearest = towns
        .as_ref()
        .and_then(|t| {
            t.standing
                .iter()
                .map(|s| (ground_m(s.anchor, here), s.name.clone(), s.anchor))
                .min_by(|a, b| a.0.total_cmp(&b.0))
        })
        .unwrap_or((f32::NAN, String::new(), Vec3::ZERO));
    if clock.is_none() {
        // Face the town's middle, whatever way the walker was put down.
        let up = here.normalize_or(Vec3::Y);
        let toward = nearest.2 - up * nearest.2.dot(up);
        if toward.length() > 1e-6 {
            state.face(up, toward.normalize());
        }
    }
    let (started, anchor, since) = *clock.get_or_insert_with(|| {
        info!(
            "STROLL_START {seconds:.0} s, {:.0} m from {}'s anchor",
            nearest.0, nearest.1
        );
        (now, here, now)
    });
    keys.press(KeyCode::KeyW);
    let leg = launch.stroll_leg.unwrap_or(seconds * 0.5);
    let walked = now.duration_since(started).as_secs_f32();
    if walked >= leg * (*legs + 1) as f32 {
        let up = here.normalize_or(Vec3::Y);
        let back = Quat::from_axis_angle(up, std::f32::consts::PI) * state.view().0;
        state.face(up, back);
        *legs += 1;
        info!("STROLL_TURN");
    }
    if here.distance(anchor) > 1.5 {
        *clock = Some((started, here, now));
    } else {
        let stalled = now.duration_since(since).as_secs_f32();
        if stalled > 0.5 && (stalled % 0.7) < 0.05 {
            keys.press(KeyCode::Space);
        } else {
            keys.release(KeyCode::Space);
        }
        if stalled > 3.0 {
            let up = here.normalize_or(Vec3::Y);
            let turned = Quat::from_axis_angle(up, std::f32::consts::FRAC_PI_2) * state.view().0;
            state.face(up, turned);
            *clock = Some((started, here, now));
        }
    }
    if now.duration_since(started).as_secs_f32() >= seconds {
        info!("STROLL_DONE");
        exit.write(AppExit::Success);
    }
}

/// Where the coast walk is (`coast-walk`).
#[derive(Default)]
struct CoastWalk {
    started: Option<Instant>,
    /// The sea's side: +1 the left.
    side: f32,
    /// The heading the view is turning toward, and when it was chosen.
    target: Option<Vec3>,
    steered: Option<Instant>,
    last: Option<(Vec3, Instant)>,
    /// Where the stall clock last reset, and when.
    anchor: Option<(Vec3, Instant)>,
    walked_m: f32,
    rivers: u32,
    in_river: bool,
    turns: u32,
    /// Turns made without getting 4 m from where the first of them was.
    trapped: u32,
    trap_at: Option<Vec3>,
    /// Backing out of a wedge until then.
    backing: Option<Instant>,
    /// The last `COAST_PACE` line: where, when, and the count then.
    pace: Option<(Vec3, Instant, f32)>,
    path_m: f32,
    max_step_m: f32,
    done: bool,
}

/// The `--coast-walk` instrument (`coast-walk`): walk the shore at walking
/// pace, steering by `coast_walk::steer` every 0.2 s and turning the view
/// toward its heading at no more than 50 degrees a second, hopping what
/// stalls it and turning 40 degrees away from the sea after 1.5 s stuck;
/// never sprints or digs. It starts when the loading screen lifts, swims at
/// the surface in water (holding jump, which also climbs out at the bank)
/// rather than walking the bed into a bank it cannot step, and counts only
/// ground it gets 1.5 m on from, so shuffling against a bank is not walking.
fn coast_script(
    launch: Res<Launch>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut state: ResMut<pbd_app::walking::WalkingState>,
    walkers: Query<&Position, With<pbd_app::walking::Walker>>,
    mut exit: MessageWriter<AppExit>,
    mut walk: Local<CoastWalk>,
    space: pbd_app::SimSpace,
) {
    use pbd_app::coast_walk;
    let Some(goal) = launch.coast_walk else {
        return;
    };
    if !state.active || walk.done {
        return;
    }
    let Ok(position) = walkers.single() else {
        return;
    };
    // The instrument steers by planet points (`world-frames`).
    let position = &Position(space.origin().query_point(position.0));
    let here = position.0;
    let up = here.normalize_or(Vec3::Y);
    let now = Instant::now();
    state.captured = true;
    state.scripted = true;
    if walk.started.is_none() {
        let (heading, side) = coast_walk::shore_start(up).unwrap_or((state.view().0, 1.0));
        walk.side = side;
        state.face(up, heading);
        walk.started = Some(now);
        walk.anchor = Some((here, now));
        info!(
            "COAST_START {goal:.0} m, the sea on the {}",
            if side > 0.0 { "left" } else { "right" }
        );
    }
    let started = walk.started.unwrap_or(now);
    keys.release(KeyCode::ShiftLeft);
    // Backing out of a wedge: walk backwards with a hop, then a quarter turn.
    if let Some(until) = walk.backing {
        if now < until {
            keys.release(KeyCode::KeyW);
            keys.press(KeyCode::KeyS);
            keys.press(KeyCode::Space);
            return;
        }
        walk.backing = None;
        keys.release(KeyCode::KeyS);
        keys.release(KeyCode::Space);
        let turned =
            Quat::from_axis_angle(up, -walk.side * std::f32::consts::FRAC_PI_2) * state.view().0;
        state.face(up, turned);
        walk.target = Some(turned);
        walk.steered = Some(now);
        walk.anchor = Some((here, now));
    }
    keys.press(KeyCode::KeyW);
    // Ground covered, as the walk measures it: a fall, a jump or a rebase is
    // not walking.
    if let Some((last, at)) = walk.last {
        let step = ground_m(here, last);
        walk.path_m += step;
        walk.max_step_m = walk.max_step_m.max(step);
        // Turn toward the chosen heading, no faster than a walker looks round.
        if let Some(target) = walk.target {
            let dt = now.duration_since(at).as_secs_f32();
            let heading = state.view().0;
            let target = (target - up * target.dot(up)).normalize_or(heading);
            let angle = heading.angle_between(target);
            let sign = up.dot(heading.cross(target)).signum();
            let turn = angle.min(50f32.to_radians() * dt) * sign;
            if turn.abs() > 1e-5 {
                state.face(up, Quat::from_axis_angle(up, turn) * heading);
            }
        }
    }
    walk.last = Some((here, now));
    if walk
        .steered
        .is_none_or(|t| now.duration_since(t).as_secs_f32() >= 0.2)
    {
        walk.steered = Some(now);
        walk.target = match coast_walk::steer(up, state.view().0, walk.side) {
            Some(choice) => Some(choice.heading),
            None => Some(
                Quat::from_axis_angle(up, -walk.side * std::f32::consts::FRAC_PI_2)
                    * state.view().0,
            ),
        };
    }
    let in_river = pbd_app::planet::surface_height(up) < 0.0 && coast_walk::in_river(up);
    if in_river && !walk.in_river {
        walk.rivers += 1;
        info!(
            "COAST_RIVER {} at {:.0} m walked",
            walk.rivers, walk.walked_m
        );
    }
    walk.in_river = in_river;
    // Out of a trap once 4 m from where it began.
    if walk.trap_at.is_some_and(|at| ground_m(here, at) > 4.0) {
        walk.trapped = 0;
        walk.trap_at = None;
    }
    let (anchor, since) = walk.anchor.unwrap_or((here, now));
    let swimming = pbd_app::planet::surface_height(up) < 0.0;
    // Along the ground: a hop against a wall rises 3 m and is not progress.
    let step = ground_m(here, anchor);
    if step > 1.5 {
        if step < 5.0 {
            walk.walked_m += step;
        }
        walk.anchor = Some((here, now));
        if swimming {
            keys.press(KeyCode::Space);
        } else {
            keys.release(KeyCode::Space);
        }
    } else {
        let stalled = now.duration_since(since).as_secs_f32();
        // Trapped (a bank the height field said was lower, a tree): three
        // turns in, hold jump, which climbs out of water; four, back out.
        if swimming || walk.trapped >= 3 || (stalled > 0.5 && (stalled % 0.7) < 0.05) {
            keys.press(KeyCode::Space);
        } else {
            keys.release(KeyCode::Space);
        }
        if walk.trapped >= 4 {
            walk.backing = Some(now + std::time::Duration::from_millis(1500));
            walk.trapped = 0;
            walk.trap_at = None;
            info!("COAST_BACK backing out at {:.0} m walked", walk.walked_m);
        }
        // A tree or a step the height field cannot see: turn away from the
        // sea and walk on, quickly, as a walker steps round it.
        if stalled > 1.5 {
            let away = Quat::from_axis_angle(up, -walk.side * 40f32.to_radians()) * state.view().0;
            state.face(up, away);
            walk.target = Some(away);
            walk.steered = Some(now);
            walk.turns += 1;
            walk.trapped += 1;
            walk.trap_at.get_or_insert(here);
            walk.anchor = Some((here, now));
            info!(
                "COAST_TURN stalled 1.5 s at {:.0} m walked, {}, {} in this spot",
                walk.walked_m,
                pbd_app::place::Place::of(here).line(),
                walk.trapped
            );
        }
    }
    // Every 10 s, the pace: ground counted against the straight line, and
    // whether the walker is in water, so the count can be checked.
    let walked = walk.walked_m;
    let (at, then, counted) = *walk.pace.get_or_insert((here, now, walked));
    let dt = now.duration_since(then).as_secs_f32();
    if dt >= 10.0 {
        info!(
            "COAST_PACE {:.1} m/s counted, {:.1} m/s straight ({:.1} m/s through space), {:.1} m from the centre, {}",
            (walk.walked_m - counted) / dt,
            ground_m(here, at) / dt,
            here.distance(at) / dt,
            here.length(),
            if swimming { "swimming" } else { "on land" }
        );
        info!(
            "COAST_PATH {:.1} m/s frame by frame, the largest frame step {:.2} m",
            walk.path_m / dt,
            walk.max_step_m,
        );
        walk.pace = Some((here, now, walk.walked_m));
        walk.path_m = 0.0;
        walk.max_step_m = 0.0;
    }
    if walk.walked_m >= goal {
        walk.done = true;
        info!(
            "COAST_DONE {:.0} m in {:.1} s, {} rivers, {} turns",
            walk.walked_m,
            now.duration_since(started).as_secs_f32(),
            walk.rivers,
            walk.turns
        );
        exit.write(AppExit::Success);
    }
}

/// Where the river walk is (`river-cache` design 8).
#[derive(Default)]
struct RiverWalk {
    path: Option<pbd_app::river_walk::RiverPath>,
    started: Option<Instant>,
    /// When the next once-a-second line is due.
    next_log: Option<Instant>,
    /// The heading the walker moves toward.
    target: Option<Vec3>,
    last: Option<(Vec3, Instant)>,
    walked_m: f32,
    /// The furthest along the river the walker has been, m.
    best_m: f32,
    /// When `best_m` last grew by 5 m.
    gained: Option<(f32, Instant)>,
    /// The last second and a bit of where the walker was: seconds since the
    /// start and the unit direction (`river-cache` design 12).
    trail: std::collections::VecDeque<(f32, Vec3)>,
    /// The hex route being walked, cell centre to cell centre.
    route: Option<pbd_app::river_walk::HexRoute>,
    /// Cells that stuck it, kept out of the next routes.
    blocked: Vec<[u32; 3]>,
    /// Stuck this many times in a row near `stuck_at`.
    stuck: u32,
    stuck_at: Option<Vec3>,
    /// When it last found itself stuck: the next check waits a second.
    stuck_when: Option<Instant>,
    stuck_total: u32,
    routes: u32,
    /// Where `--walk-river` spawned it.
    spawn: Option<Vec3>,
    /// When a route was last planned: at most twice a second.
    planned: Option<Instant>,
    hop_until: Option<Instant>,
    /// The jump key went down last frame: on land it is let up between taps.
    jump_down: bool,
    /// The main thread's cache lookups at the last `RIVER_CACHE` line.
    main_lookups: u64,
    backing: Option<Instant>,
    done: bool,
}

/// The `--walk-river` instrument (`river-cache` design 8): walk the river
/// nearest the start from its source to the sea on its left bank, through
/// the player's inputs only: W and sprint, the view turned toward a point
/// `STEER_AHEAD_M` along the bank no faster than 70 degrees a second (as a
/// mouse turns it), jump to hop a stall or swim at the surface, and the
/// coast walk's turn away (here from the river) after 1.5 s stuck. It does
/// not move the walker to the source: the start (`--at`) should be at it,
/// and `RIVER_WALK_START` logs where the source is. Each second it logs its
/// progress and `RIVER_AHEAD`, the drawn water ahead under the old listing
/// rule and the new (a CPU model of the visibility pass's frustum test with
/// the camera's own view; the recording is the check). It quits at the
/// mouth, or after five minutes without 5 m of progress.
#[allow(clippy::too_many_arguments)]
fn river_walk_script(
    launch: Res<Launch>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut state: ResMut<pbd_app::walking::WalkingState>,
    walkers: Query<&Position, With<pbd_app::walking::Walker>>,
    cameras: Query<(&GlobalTransform, &Projection), With<pbd_app::walking::WalkingCamera>>,
    fine: Option<Res<pbd_app::planet::PlanetFine>>,
    columns: Option<Res<pbd_app::config::ColumnSettings>>,
    water: Option<Res<pbd_app::config::WaterSettings>>,
    mut exit: MessageWriter<AppExit>,
    mut walk: Local<RiverWalk>,
    space: pbd_app::SimSpace,
    contact: Option<Res<pbd_app::planet::PlanetContact>>,
) {
    use pbd_app::river_walk::{RiverPath, STEER_AHEAD_M};
    let _ = STEER_AHEAD_M;
    /// How far down the river's line the view looks, m.
    const LOOK_AHEAD_M: f32 = 50.0;
    if !state.active || walk.done {
        return;
    }
    let Ok(position) = walkers.single() else {
        return;
    };
    let here = space.origin().query_point(position.0);
    let up = here.normalize_or(Vec3::Y);
    let now = Instant::now();
    state.captured = true;
    state.scripted = true;
    if walk.path.is_none() {
        // Not before the walker stands where it was spawned, in the river:
        // a world opening may hold it elsewhere for a few frames, and a
        // river measured from there is measured from the wrong place.
        let spawn = *walk.spawn.get_or_insert_with(|| spawn_direction(&launch));
        if launch.walk_river && up.angle_between(spawn) * PLANET_RADIUS > 60.0 {
            return;
        }
        let side = if launch.walk_river_right { -1.0 } else { 1.0 };
        let Some(path) = RiverPath::through(up, side) else {
            error!(
                "RIVER_WALK_DONE no river near {}",
                pbd_app::place::Place::of(here).line()
            );
            walk.done = true;
            exit.write(AppExit::Success);
            return;
        };
        let source = pbd_app::place::Place::of(path.line[0] * pbd_app::planet::PLANET_RADIUS);
        info!(
            "RIVER_WALK_START {:.0} m of river, {} cells, from level {} m to the sea; source {}; starting {:.0} m along it, {}",
            path.length_m(),
            path.line.len(),
            path.level[0],
            source.line(),
            path.progress_m(up),
            pbd_app::place::Place::of(here).line()
        );
        let toward = path.bank_at(path.progress_m(up) + STEER_AHEAD_M);
        state.face(up, toward * pbd_app::planet::PLANET_RADIUS);
        walk.started = Some(now);
        walk.next_log = Some(now);
        walk.gained = Some((path.progress_m(up), now));
        walk.path = Some(path);
    }
    let path = walk.path.clone().expect("the river");
    let started = walk.started.unwrap_or(now);
    let progress = path.progress_m(up);
    // Never a jump: a meander's nearest line point can lie far along.
    walk.best_m = walk.best_m.max(progress.min(walk.best_m + 30.0));
    keys.press(KeyCode::ShiftLeft);
    let t = now.duration_since(started).as_secs_f32();
    walk.trail.push_back((t, up));
    while walk.trail.len() > 2 && walk.trail[1].0 <= t - 1.0 {
        walk.trail.pop_front();
    }
    if let Some((last, _)) = walk.last {
        walk.walked_m += ground_m(here, last).min(5.0);
    }
    let heading = state.view().0;
    // The view looks down the river, 50 m along its line, so the river and
    // its distance are in the picture, turning no faster than a mouse does.
    let look = path.line_at(walk.best_m.max(progress) + LOOK_AHEAD_M);
    let look = (look - up * look.dot(up)).normalize_or(heading);
    if let Some((_, at)) = walk.last {
        let dt = now.duration_since(at).as_secs_f32();
        let angle = heading.angle_between(look);
        let sign = up.dot(heading.cross(look)).signum();
        let turn = angle.min(70f32.to_radians() * dt) * sign;
        if turn.abs() > 1e-5 {
            state.face(up, Quat::from_axis_angle(up, turn) * heading);
        }
    }
    walk.last = Some((here, now));
    let backing = walk.backing.is_some_and(|until| now < until);
    if backing {
        for key in [KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyD] {
            keys.release(key);
        }
        keys.press(KeyCode::KeyS);
        keys.press(KeyCode::Space);
    } else {
        if walk.backing.take().is_some() {
            keys.release(KeyCode::KeyS);
            walk.route = None;
        }
        // Hex by hex (`river-cache` design 12): a route over the finest
        // cells to the bank 40 m on (or the channel when the bank has none),
        // walked centre to centre; planned again near its end or off it.
        let due = walk
            .planned
            .is_none_or(|at| now.duration_since(at).as_secs_f32() >= 0.5);
        let replan = walk
            .route
            .as_ref()
            .is_none_or(|r| r.left_m() < 15.0 || r.off_m(up) > 6.0);
        if replan && due {
            walk.planned = Some(now);
            let ahead = progress.min(walk.best_m) + 40.0;
            let blocked = walk.blocked.clone();
            let live = contact.as_deref();
            walk.route =
                pbd_app::river_walk::HexRoute::plan(up, path.line_at(ahead), &blocked, live)
                    .or_else(|| {
                        pbd_app::river_walk::HexRoute::plan(up, path.bank_at(ahead), &blocked, live)
                    });
            walk.routes += 1;
            if walk.route.is_none() {
                info!(
                    "RIVER_WALK_ROUTE none from {:.0} m along: straight for the channel",
                    progress
                );
            }
        }
        let aim = match walk.route.as_mut() {
            Some(route) => route.aim(up),
            None => path.line_at(walk.best_m.max(progress) + 10.0),
        };
        walk.target = Some((aim - up).normalize_or_zero());
        let moving = walk
            .target
            .map(|t| (t - up * t.dot(up)).normalize_or(heading))
            .unwrap_or(heading);
        let left = up.cross(heading).normalize_or_zero();
        let (ahead, aside) = (moving.dot(heading), moving.dot(left));
        for (key, on) in [
            (KeyCode::KeyW, ahead > 0.38),
            (KeyCode::KeyS, ahead < -0.38),
            (KeyCode::KeyA, aside > 0.38),
            (KeyCode::KeyD, aside < -0.38),
        ] {
            if on {
                keys.press(key);
            } else {
                keys.release(key);
            }
        }
        // Jump: swimming, a hop being taken, or the next cell a rise the
        // auto step will not take.
        // In water by the live column at the feet, as the walker's own
        // physics has it; the generator's river says "river" over a dry bed.
        let feet = here - up * pbd_app::walking::HALF_HEIGHT;
        let swimming = contact.as_deref().map_or_else(
            || pbd_app::planet::surface_height(up) < 0.0 || pbd_app::coast_walk::in_river(up),
            |c| c.stand(feet).water_depth > 0.5,
        );
        // A step the auto step will not take, measured from the feet: the
        // next cell's floor, live, over where the walker stands now.
        let feet_m = feet.length() - pbd_app::planet::PLANET_RADIUS;
        let rise = walk.route.as_ref().is_some_and(|r| {
            let next = r.cells[r.next];
            let (floor, _) = pbd_app::river_walk::cell_heights(contact.as_deref(), next);
            floor - feet_m > 0.6 && next.angle_between(up) * pbd_app::planet::PLANET_RADIUS < 4.0
        });
        // In water the jump is held to swim up; on land a jump is a press,
        // so it is tapped (a held key jumps once and never again).
        let want = swimming || rise || walk.hop_until.is_some_and(|until| now < until);
        if want && (swimming || !walk.jump_down) {
            keys.press(KeyCode::Space);
            walk.jump_down = true;
        } else {
            keys.release(KeyCode::Space);
            walk.jump_down = false;
        }
        // Stuck: under a few metres of ground in the last second, measured
        // on the planet with height left out, so a hop or a slide is not
        // motion. Each stuck in one place escalates.
        let threshold = if swimming {
            pbd_app::river_walk::STUCK_SWIM_M
        } else {
            pbd_app::river_walk::STUCK_LAND_M
        };
        let trail: Vec<(f32, Vec3)> = walk.trail.iter().copied().collect();
        if let Some(moved) = pbd_app::river_walk::moved_in_last_second(&trail)
            && moved < threshold
            && walk
                .stuck_when
                .is_none_or(|at| now.duration_since(at).as_secs_f32() >= 1.0)
        {
            if walk.stuck_at.is_some_and(|at| ground_m(here, at) < 8.0) {
                walk.stuck += 1;
            } else {
                walk.stuck = 1;
                walk.stuck_at = Some(here);
            }
            walk.stuck_total += 1;
            walk.stuck_when = Some(now);
            walk.trail.clear();
            // The cells it was walking into stay out of the next routes.
            if let Some(route) = &walk.route {
                let keys: Vec<[u32; 3]> = route.cells[route.next..]
                    .iter()
                    .take(2)
                    .map(|&d| pbd_app::river_walk::cell_key(d))
                    .collect();
                walk.blocked.extend(keys);
            }
            // A swimmer already holds the jump: a hop adds nothing, so it
            // goes straight to a route round the spot.
            if swimming && walk.stuck == 1 {
                walk.stuck = 2;
            }
            let action = match walk.stuck {
                1 => {
                    walk.hop_until = Some(now + std::time::Duration::from_millis(700));
                    "a hop forward"
                }
                2 => {
                    walk.route = None;
                    walk.hop_until = Some(now + std::time::Duration::from_millis(700));
                    "a new route round it"
                }
                _ => {
                    walk.route = None;
                    walk.backing = Some(now + std::time::Duration::from_millis(1200));
                    walk.stuck = 1;
                    "backing out, then a new route"
                }
            };
            if walk.blocked.len() > 64 {
                let drop = walk.blocked.len() - 64;
                walk.blocked.drain(..drop);
            }
            // What it was doing, to find what holds it.
            let held: Vec<&str> = [
                (KeyCode::KeyW, "W"),
                (KeyCode::KeyA, "A"),
                (KeyCode::KeyS, "S"),
                (KeyCode::KeyD, "D"),
                (KeyCode::Space, "jump"),
            ]
            .into_iter()
            .filter(|(k, _)| keys.pressed(*k))
            .map(|(_, n)| n)
            .collect();
            let aiming = walk.route.as_ref().map_or_else(
                || "no route".to_string(),
                |r| {
                    format!(
                        "cell {} of {} at {:.1} m, rise {:.1} m, its ground {:.1} m, here {:.1} m",
                        r.next,
                        r.cells.len(),
                        r.cells[r.next].angle_between(up) * pbd_app::planet::PLANET_RADIUS,
                        r.rise_ahead(),
                        r.heights[r.next],
                        pbd_app::river_walk::walked_on(up)
                    )
                },
            );
            info!(
                "RIVER_WALK_STUCK {moved:.1} m in the last second at {progress:.0} m along, {}, {} in a row: {action}; holding {:?}, swimming {swimming}, {aiming}",
                pbd_app::place::Place::of(here).line(),
                walk.stuck,
                held
            );
        }
    }
    let length = path.length_m();
    if walk.next_log.is_some_and(|t| now >= t) {
        walk.next_log = Some(now + std::time::Duration::from_secs(1));
        let seconds = now.duration_since(started).as_secs_f32();
        info!(
            "RIVER_WALK {seconds:.0} s, {:.0} m walked, {:.0} of {:.0} m along ({:.0}%), {}, {}, --yaw {:.0}",
            walk.walked_m,
            progress,
            length,
            100.0 * progress / length.max(1.0),
            match (
                pbd_app::coast_walk::in_river(up),
                contact
                    .as_deref()
                    .is_none_or(|c| c.column_water(here).is_some()),
            ) {
                (true, true) => "in the river",
                (false, true) => "on the bank",
                (true, false) => "in the river, off the tier",
                (false, false) => "on the bank, off the tier",
            },
            pbd_app::place::Place::of(here).line(),
            pbd_app::place::yaw_of(up, state.view().0).to_degrees()
        );
        // The river cache (`river-cache` design 11): what it holds, and the
        // share of river lookups it answered since the start.
        if let Some(field) = pbd_core::rivers::field(pbd_app::planet::terrain_config()) {
            use std::sync::atomic::Ordering;
            let [carve, top, far, settled] = field.counts();
            let (hits, misses) = (
                field.hits.load(Ordering::Relaxed),
                field.misses.load(Ordering::Relaxed),
            );
            info!(
                "RIVER_CACHE {seconds:.0} s: {:.1}% of {} lookups from the cache; holds {carve} ground, {top} water tops, {far} far sheets, {settled} settled cells; stuck {} times so far; main thread {} lookups since the last line",
                100.0 * hits as f64 / (hits + misses).max(1) as f64,
                hits + misses,
                walk.stuck_total,
                pbd_core::river_cache::thread_lookups() - walk.main_lookups
            );
            walk.main_lookups = pbd_core::river_cache::thread_lookups();
        }
        if let (Ok((camera, projection)), Some(fine)) = (cameras.single(), fine.as_deref()) {
            let fov = match projection {
                Projection::Perspective(p) => Some((p.fov, p.aspect_ratio)),
                _ => None,
            };
            if let Some((fov, aspect)) = fov {
                let eye =
                    here + up * (pbd_app::walking::EYE_HEIGHT - pbd_app::walking::HALF_HEIGHT);
                let inverse = camera.rotation().inverse();
                let (ty, tx) = ((fov * 0.5).tan(), (fov * 0.5).tan() * aspect);
                // The visibility pass's sphere test against the four side
                // planes, its bound a finest cell's reach and the swell's 4 m.
                let in_view = |d: Vec3, radius: f32| {
                    let v = inverse * (d * radius - eye);
                    let depth = -v.z;
                    let r = 1.7 + 4.0;
                    let slack = |t: f32| r * (1.0 + t * t).sqrt();
                    depth > -r
                        && v.x.abs() <= depth * tx + slack(tx)
                        && v.y.abs() <= depth * ty + slack(ty)
                };
                let sea = pbd_app::planet::PLANET_RADIUS
                    - water.as_deref().map_or(0.5, |w| w.depth_offset_m);
                let old = |d: Vec3, _surface: f32| in_view(d, sea);
                let new =
                    |d: Vec3, surface: f32| in_view(d, pbd_app::planet::PLANET_RADIUS + surface);
                let tier = columns.as_deref().map_or(0.0, |c| c.reach_m) - 15.0;
                let anchor = fine.set.anchor;
                // The river measured: to the measure's reach, or to the mouth.
                let measured = (length - progress).clamp(0.0, pbd_app::river_walk::AHEAD_REACH_M);
                info!(
                    "RIVER_AHEAD {seconds:.0} s: drawn to {:.0} m ahead under the old listing, {:.0} m under the new, of {:.0} m measured ({:.0} m from the tier's anchor)",
                    path.drawn_ahead_m(progress, anchor, tier, &in_view, &old)
                        .min(measured),
                    path.drawn_ahead_m(progress, anchor, tier, &in_view, &new)
                        .min(measured),
                    measured,
                    ground_m(here, anchor * pbd_app::planet::PLANET_RADIUS)
                );
            }
        }
    }
    if walk.best_m - walk.gained.map_or(0.0, |g| g.0) >= 5.0 {
        walk.gained = Some((walk.best_m, now));
    }
    let stuck = walk
        .gained
        .is_some_and(|(_, t)| now.duration_since(t).as_secs_f32() > 300.0);
    // The course's last step is the sea's cell, whose centre is half a
    // cell (21 m) and more past the mouth.
    let at_sea = progress >= length - 45.0
        || (pbd_app::planet::surface_height(up) < 0.0
            && !pbd_app::coast_walk::in_river(up)
            && progress > length * 0.8);
    if at_sea || stuck {
        walk.done = true;
        info!(
            "RIVER_WALK_DONE {} at {:.0} of {:.0} m ({:.0}%), {:.0} m walked in {:.1} s, stuck {} times",
            if stuck { "stuck" } else { "at the mouth" },
            progress,
            length,
            100.0 * progress / length.max(1.0),
            walk.walked_m,
            now.duration_since(started).as_secs_f32(),
            walk.stuck_total
        );
        keys.release(KeyCode::KeyW);
        keys.release(KeyCode::ShiftLeft);
        exit.write(AppExit::Success);
    }
}

/// The scripted swim: put the walker at the last dry cell of the `shore` walk
/// and hold forward. It is the only way a headless capture can photograph the
/// water being entered, since a walker with no input stands still, and it is
/// the same shoreline the `shore`, `wade` and `dive` camera presets frame.
fn swim_script(
    launch: Res<Launch>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut state: ResMut<pbd_app::walking::WalkingState>,
    mut placed: Local<bool>,
    terrain: Res<pbd_app::planet::PlanetContact>,
    mut walkers: Query<
        (
            &mut Position,
            &mut pbd_app::walking::GroundState,
            &mut Transform,
        ),
        With<pbd_app::walking::Walker>,
    >,
    space: pbd_app::SimSpace,
) {
    if !state.active {
        return;
    }
    if !*placed {
        // The swim's shore is a cold one; a fishing shore is warm enough
        // for the schools' windows on day one.
        let lat = if launch.fish { 24_f32 } else { 72_f32 }.to_radians();
        let at = |lon: f32| Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
        let step = 2.0 * tile_width_m(FINEST_LEVEL) / PLANET_RADIUS;
        let mut lon = 0.0_f32;
        while surface_height(at(lon)) < 0.0 && lon < std::f32::consts::TAU {
            lon += step;
        }
        while surface_height(at(lon)) >= 0.0 && lon < 2.0 * std::f32::consts::TAU {
            lon += step;
        }
        // A fishing shore is the edge of open water: a river channel is also
        // below the datum, and the first one found at 24 degrees was a dry
        // canyon the float landed in. Twenty wet cells in a row is a sea.
        if launch.fish {
            let open = |lon: f32| (0..20).all(|k| surface_height(at(lon + k as f32 * step)) < -1.0);
            while !(surface_height(at(lon - step)) >= 0.0 && open(lon))
                && lon < 3.0 * std::f32::consts::TAU
            {
                lon += step;
            }
        }
        // The last dry cell, a few cells back from the water, facing the sea.
        // An angler stands at the water's edge rather than up the bank: a
        // cast carries ten to twenty metres.
        let land = at(lon - if launch.fish { 1.0 } else { 4.0 } * step);
        let sea = at(lon);
        let Ok((mut position, mut ground, mut transform)) = walkers.single_mut() else {
            return;
        };
        // A planet point, put into local space (`world-frames`); the origin
        // follows the walker there on the next frame.
        let stand = space.origin().at(
            land.as_dvec3(),
            (terrain.sample(land).floor_radius + 1.0) as f64,
        );
        position.0 = stand;
        transform.translation = stand;
        ground.previous = stand;
        ground.grounded = true;
        state.captured = true;
        state.scripted = true;
        state.face(land, (sea - land).normalize_or_zero());
        *placed = true;
    }
    if launch.swim {
        keys.press(KeyCode::KeyW);
    }
}

/// The frames after the go `--climb-out` photographs: the swim up to the
/// bank, the rise, the step onto it and standing on it.
const CLIMB_SHOTS: [u32; 6] = [2, 30, 38, 44, 52, 75];

/// Where `--climb-out` swims from: along the parallels from 10 to 80
/// degrees, a tile at a time, the first sea cell three layers deep (else
/// two) whose next cell east is a bank one or two layers over sea level
/// (else level with it), with deep water a tile further out. The
/// swimmer's start (the sea cell) and the bank, as directions.
fn climb_out_site() -> Option<(Vec3, Vec3)> {
    let step = tile_width_m(FINEST_LEVEL) / PLANET_RADIUS;
    for (deep, banks) in [(-3.0, 1.0..=2.0), (-2.0, 1.0..=2.0), (-2.0, 0.0..=2.0)] {
        if let Some(site) = climb_out_site_with(step, deep, banks) {
            return Some(site);
        }
    }
    None
}

fn climb_out_site_with(
    step: f32,
    deep: f32,
    banks: std::ops::RangeInclusive<f32>,
) -> Option<(Vec3, Vec3)> {
    for lat in (10..=80).step_by(2) {
        let lat = (lat as f32).to_radians();
        let at = |lon: f32| Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
        let mut lon = 0.0_f32;
        while lon < std::f32::consts::TAU {
            let (out, sea, bank) = (at(lon - step), at(lon), at(lon + step));
            if surface_height(out) <= -3.0
                && surface_height(sea) <= deep
                && banks.contains(&surface_height(bank))
            {
                return Some((sea, bank));
            }
            lon += step / lat.cos().max(0.2);
        }
    }
    None
}

/// The scripted climb out of the water (`swimming` 4): the walker is held in
/// the sea at [`climb_out_site`], its feet a metre under the sheet and facing
/// the bank, until the column tier is under it, and then holds forward and
/// nothing else. It logs `CLIMB_GO <frame>` when it lets go, and each frame
/// after `CLIMB <frame> <feet over the datum> <metres from the bank's centre>`.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn climb_out_script(
    mut commands: Commands,
    launch: Res<Launch>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut state: ResMut<pbd_app::walking::WalkingState>,
    mut coarse_site: Local<Option<Option<(Vec3, Vec3)>>>,
    mut site: Local<Option<(Vec3, Vec3)>>,
    mut go: Local<Option<u32>>,
    mut frame: Local<u32>,
    terrain: Res<pbd_app::planet::PlanetContact>,
    mut walkers: Query<
        (
            &mut Position,
            &mut LinearVelocity,
            &mut pbd_app::walking::GroundState,
            &mut Transform,
        ),
        With<pbd_app::walking::Walker>,
    >,
    space: pbd_app::SimSpace,
) {
    if !state.active {
        return;
    }
    *frame += 1;
    let o = space.origin();
    let Some((coarse, _)) = *coarse_site.get_or_insert_with(|| {
        let found = climb_out_site();
        if found.is_none() {
            warn!("CLIMB_SITE none");
        }
        found
    }) else {
        return;
    };
    let Ok((mut position, mut velocity, mut ground, mut transform)) = walkers.single_mut() else {
        return;
    };
    // Held at the coarse site until the column tier is under it, then at the
    // nearest bank the tier itself has, one or two layers over the water.
    if site.is_none() && terrain.finest_cell(coarse).is_some() {
        *site = terrain
            .bank_near(coarse, 120.0, 3.0, 1.0..=2.0)
            .or_else(|| terrain.bank_near(coarse, 120.0, 2.0, 1.0..=2.0));
        match *site {
            Some((water, _)) => {
                info!("CLIMB_SITE {water}");
                *go = Some(*frame + 60);
            }
            None => warn!("CLIMB_SITE no bank on the tier"),
        }
    }
    let (out, bank) = site.unwrap_or((coarse, coarse));
    if go.is_none_or(|g| *frame < g) {
        let sheet = PLANET_RADIUS - 0.5;
        // Half a tile further out than the water cell's centre, so the
        // first shots see the bank whole.
        let from = if site.is_some() {
            (out + (out - bank) * 0.5).normalize()
        } else {
            out
        };
        let start = o.at(
            from.as_dvec3(),
            (sheet - 1.0 + pbd_app::walking::HALF_HEIGHT) as f64,
        );
        position.0 = start;
        transform.translation = start;
        velocity.0 = Vec3::ZERO;
        ground.previous = start;
        ground.grounded = false;
        state.captured = true;
        state.scripted = true;
        if site.is_some() {
            state.face(from, (bank - from).normalize_or_zero());
        }
        return;
    }
    if *go == Some(*frame) {
        info!("CLIMB_GO {}", *frame);
    }
    // The sequence, frames after letting go, beside the capture's own shot:
    // `<capture>-climb-<frames>.png`. Counted from the go, which follows the
    // tier's landing on the wall clock, so it has to come out of one run.
    if let (Some(path), Some(g)) = (launch.capture.as_ref(), *go) {
        let after = *frame - g;
        if CLIMB_SHOTS.contains(&after) {
            let stem = path.with_extension("");
            let shot = format!("{}-climb-{after:02}.png", stem.display());
            info!("CLIMB_SHOT {after} {shot}");
            commands
                .spawn(Screenshot::primary_window())
                .observe(bevy::render::view::screenshot::save_to_disk(shot));
        }
    }
    keys.press(KeyCode::KeyW);
    info!(
        "CLIMB {} {:.2} {:.2}",
        *frame,
        o.radius_at(position.0) as f32 - pbd_app::walking::HALF_HEIGHT - PLANET_RADIUS,
        (o.up_at(position.0) - bank).length() * PLANET_RADIUS
    );
}

/// `--break SECONDS`: from frame 90, hold the use button on whatever is under
/// the reticle for that long with `--tool` in hand, so the cracks can be
/// photographed at a chosen stage. A capture instrument: a headless run has
/// no hand on the mouse.
fn break_script(
    launch: Res<Launch>,
    time: Res<Time>,
    mut buttons: ResMut<ButtonInput<MouseButton>>,
    mut state: ResMut<pbd_app::walking::WalkingState>,
    mut tools: ResMut<pbd_app::fish::ToolSlot>,
    mut frame: Local<u32>,
    mut held_s: Local<f32>,
) {
    *frame += 1;
    if *frame < 90 || !state.active {
        return;
    }
    if launch.bare {
        tools.hold(None);
    } else {
        let tool = launch.tool.unwrap_or(pbd_core::inventory::Tool::Shovel);
        // A capture's world is new and owns nothing: the rig gives it the
        // tool asked for (its save is memory-only).
        let _ = tools.0.grant(tool, launch.tier);
        tools.hold(tool);
    }
    let Some(hold) = launch.break_s else {
        return;
    };
    state.captured = true;
    state.scripted = true;
    if *held_s < hold {
        buttons.press(MouseButton::Left);
        *held_s += time.delta_secs();
        if *held_s >= hold {
            info!("break script: held the use button {hold:.2} s");
        }
    } else {
        buttons.release(MouseButton::Left);
    }
}

/// `--cast`: once aboard, the rod granted and taken up, the pointer held,
/// and the left button held for a second from 60 frames after boarding, as
/// `fish_script` does on shore; the line's phase logged every second.
#[allow(clippy::too_many_arguments)]
fn cast_afloat_script(
    aboard: Res<pbd_app::vehicles::Aboard>,
    mut walking: ResMut<pbd_app::walking::WalkingState>,
    mut buttons: ResMut<ButtonInput<MouseButton>>,
    mut tools: ResMut<pbd_app::fish::ToolSlot>,
    mut view: ResMut<pbd_app::vehicles::VehicleView>,
    fishery: Option<Res<pbd_app::fish::Fishery>>,
    mut frame: Local<u32>,
    space: pbd_app::SimSpace,
) {
    // Aboard, or on foot (a cast from a bank, `fishing-anywhere`).
    if aboard.0.is_none() {
        if !walking.active {
            return;
        }
        walking.captured = true;
        walking.scripted = true;
    }
    *frame += 1;
    let _ = tools.0.grant(
        pbd_core::inventory::Tool::Rod,
        pbd_core::inventory::Tier::Stone,
    );
    tools.hold(pbd_core::inventory::Tool::Rod);
    view.captured = true;
    view.scripted = true;
    match *frame {
        60 => buttons.press(MouseButton::Left),
        61..=119 => {}
        120 => buttons.release(MouseButton::Left),
        _ => {}
    }
    if frame.is_multiple_of(60)
        && let Some(fishery) = fishery
    {
        info!(
            "cast script frame {}: line {:?}, float at {:.1} m",
            *frame,
            fishery.line.phase,
            space.origin().radius_at(fishery.line.float) as f32 - pbd_app::planet::PLANET_RADIUS
        );
    }
}

/// `--rod-cue`'s phase.
#[derive(Resource, Clone, Copy)]
struct RodCueScript(pbd_core::fishing::Phase);

/// `--rod-cue`: a second after the float lands, the rod in hand is drawn as
/// if the line were in the scripted phase, with that phase's event once.
/// It writes only the rod's cue, which nothing but the drawn rod reads.
fn rod_cue_script(
    script: Res<RodCueScript>,
    fishery: Option<Res<pbd_app::fish::Fishery>>,
    mut cue: ResMut<pbd_app::held::RodCue>,
    mut floating: Local<u32>,
) {
    use pbd_core::fishing::{Event, Phase};
    let Some(fishery) = fishery else {
        return;
    };
    if fishery.line.phase != Phase::Floating {
        return;
    }
    *floating += 1;
    if *floating < 60 {
        return;
    }
    cue.phase = script.0;
    if *floating == 60 {
        let event = match script.0 {
            Phase::Nibble => Event::Nibble { species: 0 },
            Phase::Bite => Event::Bite {
                species: 0,
                window_s: 1.0,
            },
            _ => Event::Hooked { species: 0 },
        };
        cue.events.push(event);
        info!("rod cue script: the rod drawn as {:?}", script.0);
    }
    if script.0 == Phase::Hooked {
        cue.tension = 0.7;
        cue.reeling = true;
        cue.pull = 0.5;
    }
}

/// `--load-hold`: aboard a raft, twelve logs into the slots and then into
/// its hold's first square, by the pack's own move and the one save line.
fn load_hold_script(
    aboard: Res<pbd_app::vehicles::Aboard>,
    vehicles: Query<&pbd_app::vehicles::Vehicle>,
    mut hotbar: ResMut<slots::Hotbar>,
    mut save: ResMut<pbd_app::saves::WorldSave>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    let Some(craft) = aboard
        .0
        .and_then(|e| vehicles.get(e).ok())
        .map(|v| &v.craft)
    else {
        return;
    };
    if craft.kind != pbd_core::vehicle::Kind::Raft {
        return;
    }
    *done = true;
    let mut slots = hotbar.0.clone();
    let log = pbd_core::inventory::Item::Good(pbd_core::goods::Good::LOG);
    slots.give(log, 12);
    let mut hold = pbd_app::vehicles::hold::read(&save, craft.id);
    let Some(from) =
        (0..pbd_core::inventory::CARRIED).find(|&i| slots.get(i).is_some_and(|s| s.item == log))
    else {
        return;
    };
    pbd_core::vehicle::hold::send(
        &mut slots,
        &mut hold,
        pbd_core::vehicle::hold::Place::Carried(from),
    );
    if pbd_app::vehicles::hold::store(&mut save, craft.id, &hold, &slots) {
        hotbar.0 = slots;
        info!("load-hold: twelve logs in the raft's hold");
    }
}

/// `--watch-hook`: no HUD or readout over the view from under the water
/// (the slots, the crosshair, the compass, the prompts and the readout),
/// after everything that shows them.
fn hide_in_play_hud(mut nodes: Query<&mut Visibility, With<Node>>) {
    for mut v in &mut nodes {
        if *v != Visibility::Hidden {
            *v = Visibility::Hidden;
        }
    }
}

/// How long into a bite `--fish-play` strikes, s.
const FISH_PLAY_STRIKE_S: f32 = 0.3;

/// `--fish-play` is on: `fish_script` plays the bite out.
#[derive(Resource)]
struct FishPlay;

/// A real-time `--fish` run (no `--capture`): the first click only takes the
/// pointer, so while the line is in, the script casts again every three
/// seconds (`movement-smoothness`'s fishing trace).
#[derive(Resource)]
struct FishLive;

/// One scripted cast for `--fish`: the left button held for most of a second
/// from frame 90, and the fishery's state logged every second, so a run says
/// what the water held as well as showing it.
#[allow(clippy::too_many_arguments)]
fn fish_script(
    mut buttons: ResMut<ButtonInput<MouseButton>>,
    fishery: Option<Res<pbd_app::fish::Fishery>>,
    status: Option<Res<pbd_app::fish::FishingStatus>>,
    fauna: Res<pbd_app::config::FaunaConfig>,
    play: Option<Res<FishPlay>>,
    live: Option<Res<FishLive>>,
    hotbar: Option<Res<pbd_app::hotbar::Hotbar>>,
    mut tools: ResMut<pbd_app::fish::ToolSlot>,
    mut was: Local<Option<pbd_core::fishing::Phase>>,
    mut frame: Local<u32>,
    time: Res<Time>,
    mut bite_at: Local<Option<f32>>,
) {
    use pbd_core::fishing::Phase;
    *frame += 1;
    // A world made with tool crafting starts with no rod (`tool-crafting`
    // decision 8): the instrument is handed one, as `--cast` is.
    if *frame == 60 {
        let _ = tools.0.grant(
            pbd_core::inventory::Tool::Rod,
            pbd_core::inventory::Tier::Stone,
        );
        tools.hold(pbd_core::inventory::Tool::Rod);
    }
    match *frame {
        90 => buttons.press(MouseButton::Left),
        91..=149 => {}
        150 => buttons.release(MouseButton::Left),
        _ => {}
    }
    if live.is_some()
        && *frame > 150
        && let Some(fishery) = fishery.as_deref()
        && matches!(fishery.line.phase, Phase::Ready | Phase::Charging)
    {
        if *frame % 180 < 60 {
            buttons.press(MouseButton::Left);
        } else {
            buttons.release(MouseButton::Left);
        }
    }
    // `--fish-play`: the bite played out as a player plays it.
    if play.is_some()
        && *frame > 150
        && let Some(fishery) = fishery.as_deref()
    {
        let line = &fishery.line;
        let fish: u16 = hotbar.as_deref().map_or(0, |h| {
            h.iter()
                .flatten()
                .filter(|s| matches!(s.item, pbd_core::inventory::Item::Fish(_)))
                .map(|s| s.count)
                .sum()
        });
        if *was != Some(line.phase) {
            info!(
                "fish play frame {}: {:?} -> {:?}, tension {:.2}, fish in the slots {fish}, status {:?}",
                *frame,
                *was,
                line.phase,
                line.tension,
                status.as_deref().map(|s| s.text.as_str())
            );
            *was = Some(line.phase);
        }
        // Strike 0.3 s into the bite, as a player who sees the float go
        // under does (and as M37's angler does, `fish-behaviour`), so the
        // take is seen before the strike.
        let now = time.elapsed_secs();
        if line.phase == Phase::Bite {
            bite_at.get_or_insert(now);
        } else {
            *bite_at = None;
        }
        match line.phase {
            // The float has gone under: click.
            Phase::Bite
                if !buttons.pressed(MouseButton::Left)
                    && bite_at.is_some_and(|at| now - at >= FISH_PLAY_STRIKE_S) =>
            {
                buttons.press(MouseButton::Left);
            }
            Phase::Bite => {}
            // Reel as a hand does: hold the button down until the tension
            // is high, let it up until it has fallen, never one frame on and
            // one off (`movement-smoothness` 7.2).
            Phase::Hooked => {
                let held = buttons.pressed(MouseButton::Left);
                if !held && line.tension < 0.45 {
                    buttons.press(MouseButton::Left);
                } else if held && line.tension > 0.75 {
                    buttons.release(MouseButton::Left);
                }
            }
            // A live run's recast (above) holds the button itself.
            Phase::Ready | Phase::Charging if live.is_some() => {}
            _ => {
                if buttons.pressed(MouseButton::Left) {
                    buttons.release(MouseButton::Left);
                }
            }
        }
    }
    if frame.is_multiple_of(60)
        && let (Some(fishery), Some(status)) = (fishery, status)
    {
        let roster = fauna.0.roster(pbd_core::fauna::HOME_BODY);
        let schools: Vec<String> = fishery
            .schools
            .iter()
            .map(|s| {
                let name = roster
                    .get(s.species as usize)
                    .map_or("?", |sp| sp.id.as_str());
                format!("{name} x{}", s.len())
            })
            .collect();
        info!(
            "fish script frame {}: line {:?}, {} schools [{}], status {:?}",
            *frame,
            fishery.line.phase,
            schools.len(),
            schools.join(", "),
            status.text
        );
    }
}

/// Frames kept before a watched landing, every frame for this many after it,
/// then every `LANDING_SHOT_STRIDE`th to `LANDING_SHOT_LAST` (`landing_shots`).
const LANDING_SHOT_BEFORE: u32 = 4;
const LANDING_SHOT_EVERY: u32 = 8;
const LANDING_SHOT_STRIDE: u32 = 6;
const LANDING_SHOT_LAST: u32 = 70;

/// A fine set `landing_shots` watches, and the frame it landed on once it has.
type WatchedLanding = (u64, Option<u32>);

/// MEASUREMENT (`two-set-dissolve` design section 5): with
/// `PBD_CAPTURE_RUN`'s stretch: where, the first and the end frame, the stride.
type CaptureRun = (PathBuf, u64, u64, u64);

/// `PBD_LANDING_SHOTS=27,28` in a capture, frames round the landing of fine
/// set 27 (and 28) are saved beside the `--capture` path as
/// `landing-<set>-f<frame>.png`, with a `LANDING_SHOT` line naming the set
/// drawn: the last `LANDING_SHOT_BEFORE` while it builds, every frame for
/// `LANDING_SHOT_EVERY` after it lands, then every `LANDING_SHOT_STRIDE`th to
/// `LANDING_SHOT_LAST` (a second of game time and a little more). A landing
/// comes when its build ends, which is the wall clock's, so the frames round
/// one have to come out of one run. It changes nothing drawn.
fn landing_shots(
    mut commands: Commands,
    launch: Res<Launch>,
    state: Res<CaptureState>,
    fine: Option<Res<pbd_app::planet::PlanetFine>>,
    refresh: Res<pbd_app::planet::LodRefresh>,
    mut watched: Local<Option<Vec<WatchedLanding>>>,
) {
    let (Some(fine), Some(path)) = (fine, launch.capture.as_ref()) else {
        return;
    };
    let watched = watched.get_or_insert_with(|| {
        std::env::var("PBD_LANDING_SHOTS")
            .unwrap_or_default()
            .split(',')
            .filter_map(|set| set.trim().parse().ok())
            .map(|set| (set, None))
            .collect()
    });
    let dir = path.parent().map(PathBuf::from).unwrap_or_default();
    for (set, landed) in watched.iter_mut() {
        if fine.version == *set && landed.is_none() {
            *landed = Some(state.frame);
        }
        let since = landed.map(|frame| state.frame - frame);
        let building = fine.version + 1 == *set && refresh.in_flight_s().is_some();
        let after = since.is_some_and(|since| {
            since < LANDING_SHOT_EVERY
                || (since <= LANDING_SHOT_LAST && since.is_multiple_of(LANDING_SHOT_STRIDE))
        });
        if !(building || after) {
            continue;
        }
        let prefix = format!("landing-{set}-f");
        if building {
            // Only the last few frames of a build are kept: the disk is not
            // the place to hold three seconds of 5 MB frames nobody reads. A
            // frame is saved a few frames after it is asked for, so the older
            // ones are swept each frame rather than deleted by name once.
            let keep_from = state.frame.saturating_sub(LANDING_SHOT_BEFORE);
            for file in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
                let name = file.file_name().to_string_lossy().into_owned();
                let frame = name
                    .strip_prefix(&prefix)
                    .and_then(|rest| rest.strip_suffix(".png"))
                    .and_then(|frame| frame.parse::<u32>().ok());
                if frame.is_some_and(|frame| frame < keep_from) {
                    let _ = std::fs::remove_file(file.path());
                }
            }
        }
        let shot = dir.join(format!("{prefix}{:05}.png", state.frame));
        info!(
            "LANDING_SHOT set {set} frame {} drawing set {} frames since its landing {since:?}",
            state.frame, fine.version
        );
        commands
            .spawn(Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(shot));
    }
}

/// MEASUREMENT (`world-frames` design 7): `PBD_REBASE_SHOTS=<dir>` saves
/// every frame of a real-time run as `f<frame>.png` and keeps only the four
/// frames each side of the first three rebases of the local origin, so the
/// frames across a rebase can be compared with their neighbours for a pop.
/// The frame a rebase is drawn in is logged as `REBASE_SHOT <frame>`.
/// Changes nothing drawn.
fn rebase_shots(
    mut commands: Commands,
    rebases: Option<Res<pbd_app::sim_origin::RebaseCount>>,
    frames: Option<Res<pbd_app::loading::FrameCount>>,
    mut state: Local<(Option<u64>, Vec<u64>)>,
) {
    let (Some(rebases), Some(frames)) = (rebases, frames) else {
        return;
    };
    let Some(dir) = std::env::var_os("PBD_REBASE_SHOTS").map(PathBuf::from) else {
        return;
    };
    let frame = frames.0;
    let (seen, at) = &mut *state;
    if seen.is_some_and(|n| n != rebases.0) && at.len() < 3 {
        at.push(frame);
        info!("REBASE_SHOT {frame} rebase {}", rebases.0);
    }
    *seen = Some(rebases.0);
    if at.len() >= 3 && frame > at[2] + 5 {
        return;
    }
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(format!("f{frame:06}.png"));
    commands
        .spawn(Screenshot::primary_window())
        .observe(move |event: On<ScreenshotCaptured>| {
            if let Ok(image) = event.image.clone().try_into_dynamic() {
                let _ = image.to_rgb8().save(&path);
            }
        });
    // Drop what is not near a rebase.
    let old = frame.saturating_sub(6);
    if !at.iter().any(|r| old + 4 >= *r && old <= r + 4) {
        let _ = std::fs::remove_file(dir.join(format!("f{old:06}.png")));
    }
}

#[allow(clippy::too_many_arguments)]
fn capture(
    mut commands: Commands,
    launch: Res<Launch>,
    input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<CaptureState>,
    stats: Res<FrameStats>,
    progress: Res<TourProgress>,
    mut exit: MessageWriter<AppExit>,
    count: Option<Res<pbd_app::loading::FrameCount>>,
    mut burst: Local<Option<Option<CaptureRun>>>,
) {
    state.frame += 1;
    // MEASUREMENT (`baked-light-ease`): `PBD_CAPTURE_RUN=<dir>,<first>,<count>`
    // saves every frame from the capture's frame `first` (counted from the
    // world's first open frame, as `--frames` is, so a run that loads slowly
    // saves the same stretch) for `count` frames, so a swap can be seen frame
    // by frame. Each is named by the frame log's frame number, which the
    // `BAKED_LIGHT` lines give. An optional fourth field saves every
    // `stride`th frame of the stretch instead (`weather-rain` R5: seconds of
    // a lens drying, not every frame of them).
    let run = burst.get_or_insert_with(|| {
        let spec = std::env::var("PBD_CAPTURE_RUN").ok()?;
        let mut parts = spec.split(',');
        let dir = PathBuf::from(parts.next()?.trim());
        let first = parts.next()?.trim().parse().ok()?;
        let count: u64 = parts.next()?.trim().parse().ok()?;
        let stride: u64 = parts
            .next()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(1)
            .max(1);
        std::fs::create_dir_all(&dir).ok()?;
        Some((dir, first, first + count, stride))
    });
    if let (Some((dir, first, end, stride)), Some(count)) = (run.as_ref(), count.as_ref())
        && (*first..*end).contains(&u64::from(state.frame))
        && (u64::from(state.frame) - *first).is_multiple_of(*stride)
    {
        let path = dir.join(format!("f{:06}.png", count.0));
        commands.spawn(Screenshot::primary_window()).observe(
            move |event: On<ScreenshotCaptured>| {
                if let Ok(image) = event.image.clone().try_into_dynamic() {
                    let _ = image.to_rgb8().save(&path);
                }
            },
        );
    }
    let automatic = launch.capture.is_some() && state.frame >= launch.frames && !state.requested;
    if automatic || input.just_pressed(KeyCode::F12) {
        let path = if automatic {
            launch.capture.clone().unwrap()
        } else {
            let _ = std::fs::create_dir_all("output/captures");
            PathBuf::from(format!("output/captures/flight-{}.png", state.frame))
        };
        let saved_path = path.clone();
        commands.spawn(Screenshot::primary_window()).observe(
            move |event: On<ScreenshotCaptured>,
                  mut state: ResMut<CaptureState>,
                  mut exit: MessageWriter<AppExit>| {
                // Strip the HDR brightness alpha just as Bevy's saver does,
                // but only acknowledge success after the actual PNG write.
                let result = event
                    .image
                    .clone()
                    .try_into_dynamic()
                    .map_err(|error| format!("{error:?}"))
                    .and_then(|image| {
                        image
                            .to_rgb8()
                            .save(&saved_path)
                            .map_err(|error| error.to_string())
                    });
                match result {
                    Ok(()) => {
                        println!("CAPTURE_SAVED {}", saved_path.display());
                        if automatic {
                            state.saved = true;
                            state.finish_frame = state.frame + 4;
                        }
                    }
                    Err(error) => {
                        eprintln!("CAPTURE_FAILED {}: {error}", saved_path.display());
                        if automatic {
                            exit.write(AppExit::error());
                        }
                    }
                }
            },
        );
        if automatic {
            state.requested = true;
        }
        println!("CAPTURE_REQUEST {} frame={}", path.display(), state.frame);
        // Where it was taken (`position-readout`), so the shot can be taken
        // again with `--at`.
        let named = path.display().to_string();
        commands.queue(move |world: &mut World| {
            if let Some(place) = pbd_app::place::of_world(world) {
                info!("PLACE capture {named} {}", place.line());
            }
            // The water under and round the walker (`townsfolk` design 42p's
            // measure): its rule's top and its drawn top.
            if let Some(feet) = pbd_app::walking::walker_feet(world)
                && let Some(contact) = world.get_resource::<pbd_app::planet::PlanetContact>()
            {
                let (feet, round) = contact.water_probe(feet, 3.0);
                info!("WATER_PROBE feet {feet:.2} (rule, drawn) {round:?}");
            }
        });
    }
    if launch.capture.is_some() && state.saved && state.frame >= state.finish_frame {
        if launch.tour {
            println!(
                "RENDERED_TOUR completed={} degrees={:.3} seconds={:.3} min_clearance_m={:.3} max_speed_m_s={:.3} protection_events={}",
                progress.completed,
                progress.angular_distance_rad.to_degrees(),
                progress.elapsed_seconds,
                progress.minimum_clearance,
                progress.maximum_speed,
                progress.protection_events
            );
        }
        // Exclude startup and the screenshot's GPU readback / PNG disk write.
        let mut samples: Vec<f64> = stats
            .samples
            .iter()
            .copied()
            .take(launch.frames.saturating_sub(1) as usize)
            .skip(60)
            .collect();
        samples.sort_by(f64::total_cmp);
        if !samples.is_empty() {
            let at = |q: f64| samples[((samples.len() - 1) as f64 * q) as usize];
            println!(
                "FRAME_WALL_MS p50={:.2} p95={:.2} p99={:.2} samples={}",
                at(0.5),
                at(0.95),
                at(0.99),
                samples.len()
            );
        }
        exit.write(AppExit::Success);
    }
    if launch.capture.is_some() && state.frame > launch.frames + 900 && !state.saved {
        eprintln!("Screenshot did not complete");
        exit.write(AppExit::error());
    }
}

/// The `--frame-log` CSV, open for the run when asked for.
#[derive(Resource)]
struct FrameLog(Option<std::io::BufWriter<std::fs::File>>, u32);

impl FrameLog {
    /// The log, with the swapchain's frame latency as a column of its own
    /// (`frame_latency`, the same in every row) so a run says what it ran.
    fn open(path: Option<&std::path::Path>, frame_latency: u32) -> Self {
        Self(path.map(|path| {
            use std::io::Write;
            let file = std::fs::File::create(path)
                .unwrap_or_else(|error| panic!("--frame-log {}: {error}", path.display()));
            let mut out = std::io::BufWriter::new(file);
            let passes: String = PASS_COLUMNS
                .iter()
                .map(|name| format!(",gpu_{name}_ms"))
                .collect();
            writeln!(
                out,
                "frame,since_start_s,wall_ms,fine_version,rebuild_s,clearance_m,speed_mps,walked_m,gpu_clouds_ms,gpu_total_ms,route_m{passes},gpu_other_ms,frame_latency,cpu_main_ms,cpu_render_ms,acquire_ms,shot,main_between_ms,submit_ms,present_ms"
            )
            .expect("frame log header");
            out
        }), frame_latency)
    }
}

/// One row per frame: the frame's wall time, the fine set drawn, the age of
/// the rebuild in flight (empty when none), and how high and fast the player
/// is, so an over-budget frame lines up with what the streaming was doing.
/// Plus the GPU time of the clouds and of the frame, and the route's distance.
#[allow(
    clippy::too_many_arguments,
    reason = "one read-only input per column of the log"
)]
fn write_frame_log(
    mut log: ResMut<FrameLog>,
    stats: Res<FrameStats>,
    // None while the loading screen prepares the world.
    fine: Option<Res<pbd_app::planet::PlanetFine>>,
    near: Res<pbd_app::planet::NearField>,
    readout: Option<Res<pbd_app::flight_view::FlightReadout>>,
    walk: Option<Res<WalkProgress>>,
    diagnostics: Option<Res<bevy::diagnostic::DiagnosticsStore>>,
    route: Option<Res<pbd_app::flight_view::RouteState>>,
    cpu: Res<cpu_times::CpuTimes>,
    showcase: Option<Res<showcase::Player>>,
) {
    use std::io::Write;
    let latency = log.1;
    let Some(out) = log.0.as_mut() else {
        return;
    };
    let gpu = diagnostics.map_or_else(GpuTimes::unknown, |d| gpu_times(&d));
    let (mut gpu_clouds, gpu_total) = (gpu.clouds, gpu.total);
    // Where the passes' own spans read nothing (Metal: no timestamps inside
    // a pass), the clouds' boundary timer (`cloud-puffs`).
    if (gpu_clouds.is_nan() || gpu_clouds <= 0.0)
        && let Some(ms) = pbd_app::planet::cloud_gpu_ms()
    {
        gpu_clouds = ms;
    }
    let Some(ms) = stats.samples.last() else {
        return;
    };
    let rebuild = near.rebuild_s.map_or(String::new(), |s| format!("{s:.3}"));
    let (clearance, speed) = readout.map_or((f32::NAN, f32::NAN), |r| (r.clearance, r.speed));
    let passes: String = gpu.passes.iter().map(|ms| format!(",{ms:.3}")).collect();
    let _ = writeln!(
        out,
        "{},{:.3},{ms:.3},{},{rebuild},{clearance:.1},{speed:.1},{:.1},{gpu_clouds:.3},{gpu_total:.3},{:.0}{passes},{:.3},{latency},{:.3},{:.3},{:.3},{},{:.3},{:.3},{:.3}",
        stats.frames,
        stats.started.elapsed().as_secs_f64(),
        fine.map_or(0, |fine| fine.version),
        walk.map_or(f32::NAN, |walk| walk.walked_m),
        // Ground distance along the route, in the units the scenic plan's
        // `into N m of cloud ... from M m` line uses.
        route.map_or(f64::NAN, |route| route.flown_rad
            * f64::from(pbd_app::planet::PLANET_RADIUS)),
        gpu.other,
        cpu.main_ms,
        cpu.render_ms(),
        cpu.acquire_ms(),
        showcase.map_or(0, |player| player.shot()),
        cpu.between_ms,
        cpu.submit_ms(),
        cpu.present_ms(),
    );
}

/// The top-level GPU spans the frame log gives a column each
/// (`gpu_<name>_ms`, `lighting-perf` 1.2), in frame order: Bevy's own passes
/// and the game's. The terrain draws in its own pass, `planet_terrain`
/// (`lighting-perf` 2.8; before it, inside `main_transparent_pass_3d`), with
/// wet ground's traces in it at `reflect_render_scale: 1.0`, else in
/// `reflect_half` and `reflect_compose`. A pass that did not run in a frame
/// reads 0; spans not named here are summed in `gpu_other_ms`.
const PASS_COLUMNS: [&str; 22] = [
    "planet_cull",
    "planet_shadow",
    "main_opaque_pass_3d",
    "planet_terrain",
    "reflect_half",
    "reflect_compose",
    "rain_splash",
    "main_transparent_pass_3d",
    "water_compose",
    "ssr_capture",
    "cloud_march",
    "cloud_puffs",
    "cloud_resolve",
    "clouds",
    "rain_half",
    "rain",
    "weather_overlay",
    "water_lens",
    "lens_mist",
    "lite_bloom",
    "tonemapping",
    "upscaling",
];

/// One frame's GPU times, in ms: the clouds, the total, each of
/// [`PASS_COLUMNS`] and the rest.
struct GpuTimes {
    clouds: f64,
    total: f64,
    passes: [f64; PASS_COLUMNS.len()],
    other: f64,
}

impl GpuTimes {
    fn unknown() -> Self {
        Self {
            clouds: f64::NAN,
            total: f64::NAN,
            passes: [f64::NAN; PASS_COLUMNS.len()],
            other: f64::NAN,
        }
    }
}

/// The clouds' GPU time, each named pass's and the whole instrumented
/// frame's, in ms, from the newest batch of render-diagnostic spans. A pass
/// that did not run in that batch (the clouds under `PBD_NO_CLOUDS`) keeps an
/// older measurement in the store, so only the spans stamped with the newest
/// batch's time count. The total sums top-level spans only
/// (`render/<pass>/elapsed_gpu`), so a nested span is not counted twice. The
/// values trail the frame by the few frames a timestamp query takes to read
/// back.
fn gpu_times(store: &bevy::diagnostic::DiagnosticsStore) -> GpuTimes {
    let spans: Vec<(String, f64, Instant)> = store
        .iter()
        .filter_map(|d| {
            let path = d.path().as_str();
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() != 3 || parts[0] != "render" || parts[2] != "elapsed_gpu" {
                return None;
            }
            let m = d.measurement()?;
            Some((parts[1].to_owned(), m.value, m.time))
        })
        .collect();
    let Some(newest) = spans.iter().map(|s| s.2).max() else {
        return GpuTimes::unknown();
    };
    let current = spans.iter().filter(|s| s.2 == newest);
    let clouds = current
        .clone()
        .filter(|s| {
            matches!(
                s.0.as_str(),
                "cloud_march" | "cloud_resolve" | "cloud_puffs" | "clouds"
            )
        })
        .map(|s| s.1)
        .sum();
    let mut passes = [0.0; PASS_COLUMNS.len()];
    let mut other = 0.0;
    for (name, ms, _) in current.clone() {
        match PASS_COLUMNS.iter().position(|p| p == name) {
            Some(i) => passes[i] += ms,
            None => other += ms,
        }
    }
    GpuTimes {
        clouds,
        total: current.map(|s| s.1).sum(),
        passes,
        other,
    }
}

/// With `--frame-log`, every change in whether the window has the user's
/// attention: a window out of focus or hidden is paced by the OS (Bevy updates
/// an unfocused window at 60 Hz; macOS throttles one it cannot show, as under a
/// locked screen), so a run that saw one measured the pacing, not the game.
/// `tools/perf_suite.py` marks such a run invalid (`frame-budget`).
fn log_window_attention(
    mut focus: MessageReader<bevy::window::WindowFocused>,
    mut occluded: MessageReader<bevy::window::WindowOccluded>,
) {
    for event in focus.read() {
        info!("WINDOW_FOCUS {}", event.focused);
    }
    for event in occluded.read() {
        info!("WINDOW_OCCLUDED {}", event.occluded);
    }
}

/// With `--frame-log`, a route run ends itself once the route is complete.
fn quit_after_route(
    route: Option<Res<pbd_app::flight_view::RouteState>>,
    mut exit: MessageWriter<AppExit>,
    mut done: Local<bool>,
) {
    if !*done && route.is_some_and(|route| route.completed) {
        *done = true;
        info!("ROUTE_QUIT the frame log is complete");
        exit.write(AppExit::Success);
    }
}

fn measure_frames(mut stats: ResMut<FrameStats>) {
    // Presentation handles interactive pacing. A second main-thread sleep
    // delayed freshly sampled mouse input without improving displayed motion.
    let now = Instant::now();
    let ms = now.duration_since(stats.previous).as_secs_f64() * 1000.0;
    stats.previous = now;
    stats.frame_ms = stats.frame_ms * 0.9 + ms as f32 * 0.1;
    // Kept to the last 100 000 frames: past that the oldest half goes, so
    // the F3 graph keeps showing the latest frames. It stopped at 100 000
    // and froze half an hour into a session (owner, 2026-11-20: "why did
    // the frame graph not tick forward?").
    if stats.samples.len() >= 100_000 {
        stats.samples.drain(..50_000);
    }
    stats.samples.push(ms);
    stats.frames += 1;
}

/// `--verify-route far-side`: fly the whole route headless and report what the
/// window would show: that it completes and lands where it meant to, its peak
/// height, its clearance while airborne, and the camera rig's fastest turn.
/// A route's name as `--route` and `--verify-route` take it.
fn route_kind(name: &str) -> pbd_app::flight_view::RouteKind {
    match name {
        "far-side" => pbd_app::flight_view::RouteKind::FarSide,
        // `clouds` is the scenic route with its broken-cloud plan
        // (`FlightViewConfig::route_scenic_broken`).
        "scenic" | "clouds" => pbd_app::flight_view::RouteKind::Scenic,
        other => panic!("--route knows far-side, scenic and clouds, not {other}"),
    }
}

fn verify_route(kind: pbd_app::flight_view::RouteKind) {
    let mut app = pbd_app::headless_app();
    app.add_plugins(FlightViewPlugin)
        .insert_resource(CelestialScene::planet_at_origin(PLANET_RADIUS as f64, 1.0))
        .insert_resource(FlightViewConfig {
            mode: FlyMode::Route,
            route_kind: kind,
            spawn_direction: Vec3::new(0.8776, 0.4794, 0.0).normalize(),
            minimum_clearance: 1.6,
            startup_camera: false,
            ..default()
        });
    app.finish();
    app.cleanup();
    app.update();
    for _ in 0..60_000 {
        app.update();
        if app
            .world()
            .resource::<pbd_app::flight_view::RouteState>()
            .completed
        {
            break;
        }
    }
    let config = *app.world().resource::<FlightViewConfig>();
    let r = app
        .world()
        .resource::<pbd_app::flight_view::RouteState>()
        .clone();
    let t = *app.world().resource::<TourProgress>();
    let report = format!(
        "ROUTE {} completed={} seconds={:.1} destination_deg={:.2} touchdown_error_m={:.1} peak_height_m={:.0} min_airborne_clearance_m={:.1} at_m={:.0} max_speed_m_s={:.1} max_camera_rate_rad_s={:.3} protection_events={}\n",
        kind.name(),
        r.completed,
        r.elapsed_s,
        r.destination_rad.to_degrees(),
        r.touchdown_error_m,
        r.peak_height_m,
        r.min_airborne_clearance_m,
        r.min_airborne_at_m,
        r.max_speed_mps,
        r.max_camera_rate_rad_s,
        t.protection_events,
    );
    print!("{report}");
    std::fs::create_dir_all("output/captures").unwrap();
    std::fs::write(format!("output/captures/route-{}.txt", kind.name()), report).unwrap();
    assert!(r.completed, "the route did not land and settle");
    assert!(
        r.touchdown_error_m < 30.0,
        "landed away from the destination"
    );
    match kind {
        pbd_app::flight_view::RouteKind::FarSide => assert!(
            r.peak_height_m > config.route_cruise_height_m * 0.95,
            "never reached cruise height"
        ),
        // Low over the land, never into it.
        pbd_app::flight_view::RouteKind::Scenic => assert!(
            r.min_airborne_clearance_m > 30.0,
            "the scenic route came within 30 m of the ground"
        ),
    }
    assert!(
        r.max_speed_mps <= config.route_speed + 0.5,
        "route speed exceeded"
    );
    assert!(r.max_camera_rate_rad_s < 1.0, "the camera turned too fast");
}

fn verify_flight(polar: bool) {
    let mut app = pbd_app::headless_app();
    app.add_plugins(FlightViewPlugin)
        .insert_resource(CelestialScene::planet_at_origin(PLANET_RADIUS as f64, 1.0))
        .insert_resource(FlightViewConfig {
            mode: FlyMode::Tour,
            spawn_direction: if polar {
                Vec3::Y
            } else {
                Vec3::new(0.8776, 0.4794, 0.0).normalize()
            },
            startup_camera: false,
            ..default()
        });
    app.finish();
    app.cleanup();
    app.update();
    for _ in 0..30_000 {
        app.update();
        if app.world().resource::<TourProgress>().completed {
            break;
        }
    }
    let p = app.world().resource::<TourProgress>();
    let report = format!(
        "CIRCUMNAVIGATION completed={} degrees={:.3} seconds={:.3} min_clearance_m={:.3} max_speed_m_s={:.3} max_command_acceleration_m_s2={:.3} max_angular_speed_rad_s={:.3} max_angular_acceleration_rad_s2={:.3} protection_events={}\n",
        p.completed,
        p.angular_distance_rad.to_degrees(),
        p.elapsed_seconds,
        p.minimum_clearance,
        p.maximum_speed,
        p.maximum_acceleration,
        p.maximum_angular_speed,
        p.maximum_angular_acceleration,
        p.protection_events
    );
    print!("{report}");
    std::fs::create_dir_all("output/captures").unwrap();
    std::fs::write(
        if polar {
            "output/captures/circumnavigation-polar.txt"
        } else {
            "output/captures/circumnavigation.txt"
        },
        report,
    )
    .unwrap();
    assert!(
        p.completed,
        "tour did not complete a full orbit around the surface"
    );
    assert!(p.minimum_clearance >= 45.0, "terrain clearance violated");
    assert!(p.maximum_speed <= 600.1, "flight speed limit violated");
    assert!(
        p.maximum_acceleration <= 80.01,
        "command acceleration limit violated"
    );
    assert!(
        p.maximum_angular_speed <= 1.51,
        "angular speed limit violated"
    );
    assert!(
        p.maximum_angular_acceleration <= 3.01,
        "angular acceleration limit violated"
    );
    assert_eq!(
        p.protection_events, 0,
        "tour needed emergency terrain projection"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::planet_gen::{self, Biome, TerrainConfig};
    use pbd_core::terrain::Material;

    /// What the player reads is the compass's (`compass-bar` decision 2):
    /// nothing on the screen works out a latitude, a heading or north from
    /// the frame's functions, whose north is the compass's south. A readout
    /// written with them would be the mirror of the planet again.
    #[test]
    fn the_screen_reads_the_compass_not_the_frame() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = vec![dir.join("desktop.rs")];
        for entry in std::fs::read_dir(dir.join("desktop")).expect("the desktop modules") {
            let path = entry.expect("an entry").path();
            if path.extension().is_some_and(|e| e == "rs") {
                files.push(path);
            }
        }
        // Spelt out at run time, so this test does not find itself.
        let frame = ["lat_lon", "heading", "north_east"].map(|f| format!("geo::{f}("));
        let mut found = Vec::new();
        for path in files {
            let text = std::fs::read_to_string(&path).expect("a source file");
            for (n, line) in text.lines().enumerate() {
                if frame.iter().any(|f| line.contains(f)) {
                    found.push(format!("{}:{}: {}", path.display(), n + 1, line.trim()));
                }
            }
        }
        assert!(
            found.is_empty(),
            "frame north on the screen:\n{}",
            found.join("\n")
        );
    }

    /// A world made before identities is made by version 4, and the spawn a
    /// player of it logs back into is the ground it always was: 73 m up, on
    /// fields, dry grass on top (`bigger-biomes` task 2.3, `world-persistence`
    /// task 2.2). Version 4's config is the ground from before version 5
    /// existed, pinned on 10,000 directions by
    /// `planet_gen::tests::version_4_makes_the_ground_every_old_world_was_made_on`.
    /// Version 5 moves the moisture, not the heights, and the spawn is on
    /// fields there too.
    #[test]
    fn an_old_worlds_spawn_column_is_the_ground_it_was() {
        let old = pbd_app::saves::Slot {
            id: "old".into(),
            file: pbd_app::saves::format::WorldFile::new("Old".into(), 4242, 0),
            identity: None,
        };
        let spawn = spawn_direction(&Launch::parse(&[]));
        for (version, what) in [
            (saves::generator_of(&old), "an old world"),
            (5, "a new one"),
        ] {
            let cfg = TerrainConfig::for_version(version).expect("carried");
            let surface = pbd_core::column::surface_m(&cfg, spawn);
            assert_eq!(
                (
                    surface,
                    planet_gen::biome(&cfg, spawn),
                    planet_gen::top_material(&cfg, spawn, surface)
                ),
                (73.0, Biome::Fields, Material::DryGrass),
                "{what}, version {version}"
            );
        }
        assert_eq!(saves::generator_of(&old), 4);
    }
}
