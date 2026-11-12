# Design: loading-screen

## What opening a world does today (read from the code, timed in the 11-12 report)

| When | Where | What | Cost |
| --- | --- | --- | --- |
| before the window | `desktop::run` → `open_world` | the save's log replayed | small |
| `Startup` (window blank) | `planet::create_planet` | base records, contact tier, fine set round the spawn | 5.2 s |
| first frames | Bevy render | pipelines queued and compiled on the async pool as each is first drawn | 9-11 s warm, 50-55 s cold to the first frame |
| `Update` | `sites::keep_sites` | the sites read, or surveyed on the pool (new world) | read: small |
| `Update`, one frame | `towns::start_towns` | every templated site's town read, or laid and stored | 2.6-3.0 s |
| `Update`, one frame | `towns::settle` | grounds made and installed, `rebuild_planet` round the viewer | 6.7-7.5 s, of it 5.3 s the rebuild |

`ground::install` is process-wide and the planet reads it through
`column::surface_m`, so every height the first build made is wrong for a world
with towns, and the build is thrown away. A saved world's sites and towns are
already on disk, yet the order is the same: build, read, rebuild.

## Decision 1: prepare the world before the planet, off the main thread

The loading thread does, in order, what `keep_sites`, `start_towns` and
`settle` do, with the **same functions**, then builds the planet once:

1. **Sites**: `sites::stored(save)`, or, when the world names this build's
   generator, `pbd_core::sites::generate` from `sites::list_spawn()` and
   `sites::store`. A world of another generator gets none, as today.
2. **Towns**: for each site with a template, in site-id order,
   `towns::ensure` (read, or lay and store, or mark unsettled). The towns laid
   now are read back from the save once the writer's mark passes their last
   line (decision 8 of `cities-in-the-world`: written before shown), as
   `build_towns` does.
3. **Ground**: `towns::grounds_of` (the part of `settle` before the rebuild,
   moved into a function both call) and `ground::install`.
4. **Planet**: `planet::PreparedPlanet::build` (today's `build_planet`) round
   the flight view's spawn direction, with the save's edits.

The thread owns the `WorldSave` while it works (it is removed from the world
at the first loading frame and put back when the thread is joined), so the
order of every `store` is the order today's systems use, and nothing else can
write in between. `Startup` then runs: `create_planet` takes the prepared
planet instead of building one, `WorldSites` starts `Ready` for the world's
key, and `Towns` starts holding its towns with their ground installed, so
`build_towns` has nothing left to do and `settle` is not called. The planet is
built once.

The parts of `settle` after the rebuild (stand the towns in range, respawn a
`--at` walker on the terrace) are what `stand_in_range` does over the next
frames, behind the screen; the walker already spawns on the installed ground.

**Byte identity.** The old path builds round the viewer after the rebuild; the
new one round the spawn direction, which is what the viewer is at the first
frame. For the same direction, the planet built once with the ground equals
the planet built without it and then rebuilt with it. That is pinned by an
integration test in its own process (`tests/loading_terrain.rs`), because the
ground is process-wide and would leak into every other test in the lib binary:
it prepares a fresh memory-only world through the loading path, then builds
the old way (no ground, install, rebuild), and compares base records, every
fine level's records and the column tier byte for byte. It also checks that
the records the loading path stores equal what `sites::ensure` and
`towns::ensure` store, and that the ground changes the planet at all (so the
comparison means something).

## Decision 2: hold the frame's schedules back while the thread works

Every game system takes the planet's resources as given, so they cannot run
before the planet exists. Bevy's `Main` schedule is replaced by one that, while
the world is loading, runs only `First`, a `Loading` schedule (poll the thread,
draw the screen, honour a window close) and `PostUpdate` and `Last`, which
lay out and draw the UI and keep the frame log. `Startup`, `PreUpdate`,
`Update` and the fixed loop run from the frame after the thread is joined,
exactly as before.

The few game systems in `PostUpdate` and `Last` that need what `Startup`
makes are given `run_if(world_open)`; a run without the loading plugin (every
test app) has no `WorldLoading` marker and runs them as before. The render
world's `prepare_views` takes the planet's art as an option.

The screen during this part draws on its own 2D camera, the only camera that
exists. When the world opens, that camera goes and the overlay moves onto the
game's UI camera above everything (`GlobalZIndex(i32::MAX)`).

## Decision 3: warm the pipelines by drawing the world behind the screen

Bevy 0.18 creates a pipeline when a material, mesh layout or pass is first
queued for a view, on the async compute pool (`synchronous_pipeline_compilation`
is false by default). Nothing is compiled ahead of being drawn; there is no
list to warm from. So the world is drawn: after `Startup` the game runs
normally under an opaque overlay, and the overlay stays until

- at least 20 frames have run,
- the active 3D camera has looked round four headings (three frames each)
  and been put back, then held still 30 frames so the cloud and SSR history is
  of the view the player will see,
- `PipelineCache` has had nothing waiting (queued, being created, or waiting
  on a shader asset) for 15 frames in a row,
- the towns in range stand whole (none cutting, none fading in),

or 90 s pass, when it lifts anyway and logs what was still waiting. The world
clock does not run behind the screen (`Sun.running` is held off and given
back), so loading spends no world time.

**`synchronous_pipeline_compilation = true` is not used**: it would make every
compile block the render thread, so the screen would freeze instead of
counting. Compiling async behind an overlay keeps it drawing.

**wgpu's pipeline cache (`Features::PIPELINE_CACHE`, Vulkan)** would let a run
hand compiled pipelines to the next. Bevy 0.18.1 cannot use it: its
`PipelineCache::start_create_render_pipeline` and `..._compute_pipeline` build
every wgpu descriptor with `cache: None`
(`bevy_render-0.18.1/src/render_resource/pipeline_cache.rs`), and expose no
hook. Using it means patching `bevy_render`. It is also mostly redundant here:
on Vulkan the NVIDIA driver keeps its own on-disk shader cache, which is why a
second run of the same exe reaches its first frame in 9-11 s against 50-55 s
the first time. The loading screen cannot make a cold driver cache warm; it
makes the wait visible and moves it from play to loading.

## Decision 4: the `READY` line and late pipelines

When the overlay lifts the game logs

```text
READY playable 14.21 s after launch at frame 512 (world 6.02 s, warm 3.80 s, 61 pipelines)
```

The perf suite counts its warm-up from this line (the frame number lines up
with the frame log's `frame` column). After it, a render-world system logs each
new pipeline by label as `LATE_PIPELINE <label>`: the list of what the next
prewarm must reach.

## Decision 5: a capture loads with time stopped and does not warm

A `--capture` must draw the same picture every time it is run with the same
exe, assets and arguments; before the screen, two runs matched to 0-14 pixels
at 1/255. The screen broke that (Holbrook lane at `--frames 300`: 73% of the
pixels differ; a shore view at 30 m: 26%), because a capture steps time by a
fixed amount every frame (`TimeUpdateStrategy::ManualDuration`), and loading
takes a different number of frames on every run: the thread's build time,
then however many frames the pipelines take to compile behind the screen.
Each loading frame moved virtual time, and each warm frame ran the whole
simulation (weather, clouds, waves, town fades, fixed steps).

So in a capture:

- virtual time is paused from launch and unpaused as the world opens, just
  before `Startup`, so loading advances nothing that reads time;
- there is no warm-up: the screen lifts on the first open frame, and the
  capture system runs after the lift that frame. `--frames` and every timed
  system count from `READY`, as they counted from the first frame before the
  screen existed. Pipelines compile during the counted frames, as they did
  then.

A capture is not a warm-up measurement; real-time runs keep decision 3.

Measured with `tools/capture_repeat.py` (each scene captured twice,
`--frames 300`, 1440x900, RTX 3070, release, 2026-11-12): before the fix,
Holbrook lane 0 pixels differed and a harbour from 30 m (rain, night) 44.6%
(578,454 of 1,296,000, worst 77/255); with it, 0 and 0.

## Found in the build: a new world started on a town's terrace

Before the screen, a new world's walker was set down at `Startup` on a
planet with no towns' ground, and the towns stood later. Built once with
their ground (decision 1), the start's search for a level plain found
Holbrook's terrace, 470 m from where the same seed had always started and
under a passing storm: the perf suite's `walk` read cover 1.00, rain 1.00
where it had read 0.69 and 0.00. The start is now chosen on the
generator's own ground (`column::natural_surface_m`, `place::level_plain`),
and `tests/new_world_start.rs` holds it, and the weather over it, to the
path before the screen. A craft's berths and a saved world's pose do not
search, and are unchanged.

## The screen

Full window, the game's clear colour; a panel with the menu's edge, `PALE BLUE
DOT` in ink, the step in mint (`READING THE WORLD`, `SURVEYING SITES`,
`LAYING OUT TOWNS`, `THE TOWNS' GROUND`, `BUILDING THE PLANET`, `COMPILING
SHADERS 37/61`, `STANDING THE TOWNS`), a 32-block pixel bar, and the seconds
so far. The default font is FiraMono, the HUD's. Step weights for the bar are
fixed fractions; the planet build reports no progress of its own, so its part
of the bar fills by elapsed time against the last measured 5 s and never
reaches its end before it is done.

## Measured (filled in by task 4)

See `tasks.md` 4.x and `docs/benchmarks/2026-11-12-loading-screen/`.
