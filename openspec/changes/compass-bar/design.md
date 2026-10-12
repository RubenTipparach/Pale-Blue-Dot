# Design: compass-bar

## Context

`pbd_core::geo` is the one place latitude and longitude are worked out
(`world-map` decision 5). It names +Y the north pole, puts longitude 0 on +X,
and puts longitude 90 degrees east on +Z. The proposal measures what that
means on screen: facing `geo` north, east is on the left, and the map is the
mirror of the planet. The sun and the atmosphere both turn the planet about
-Y.

The HUD is deliberately bare (`menus-and-a-quiet-screen`): a crosshair, the
slot row, and one instrument line. A compass bar is the first readout to come
back. It earns its place as a navigation control, the way the crosshair is an
aiming one.

## Decisions

### 1. North is the -Y pole (recommendation taken: ask only with screenshots)

Two ways to make the compass, the map and the sky agree:

| | North | The sun rises | What changes |
| --- | --- | --- | --- |
| A (taken) | -Y | in the east, on your right facing north | which pole is called north: latitude's sign, the map drawn flipped top to bottom, the season's hemisphere |
| B | +Y | in the west | east and west swapped: longitude's sign, the map drawn flipped left to right |
| C | +Y | in the east | the planet's spin reversed: the sun's path, the Coriolis sign, every settled climate remade |

A matches Earth and Skyrim: the sun rises in the east, and east is on your
right when you face north. B leaves the sun rising in the west. C changes
the climate model and its shipped settled states, which is a generation
change for a labelling problem. A it is. The owner judges it at the gate on
the screenshots: the bar at sunrise, and the map beside the planet seen from
orbit.

### 2. The frame keeps its names; the player reads the compass's

`geo::lat_lon`, `geo::north_east`, `geo::heading` and `geo::project` are
frame functions, and things that must not move are built on them:
- a town's layout and its stored record orient by `north_east`;
- the site placer reads the map rasters through `pixel_direction`;
- the map's raster cache is keyed by `project`.

Renaming them would touch all of that for no gain. They keep their meaning,
and their doc comments say plainly that their north is the frame's +Y and not
the compass's.

What the player reads goes through new functions beside them:

```rust
/// The pole a compass points to: the one the planet turns
/// counter-clockwise about, which the sun and the winds agree on.
pub const COMPASS_NORTH: Vec3 = Vec3::NEG_Y;
pub fn compass_north_east(direction: Vec3) -> (Vec3, Vec3); // (-frame north, frame east)
pub fn compass_heading(direction: Vec3, forward: Vec3) -> f32; // clockwise from compass north
pub fn bearing(from: Vec3, to: Vec3) -> f32; // compass heading of the great circle from one place to another
pub fn compass_lat_lon(direction: Vec3) -> LatLon; // north positive: -frame lat, same lon
```

Compass east comes out the same vector as frame east. The probe's numbers say
so: facing frame north, right is -east. So facing compass north (the
opposite), right is +east. Longitude therefore keeps its sign, and only
latitude flips.

### 3. The bar

- **Where**: top centre, 36% of the window's width, held between 320 and
  640 px, 32 px tall (letters and ticks above, a row for the towns along its foot), with a 14 px line under it for a place's name.
- **Look**: a dark band at 0.45 alpha with a hairline above and below, and a
  notch at the centre.
- **What it spans**: 180 degrees, 90 either side of the heading.
- **What it shows**:
  - N, E, S and W in 15 px capitals, with N in the bar's warm accent;
  - NE, SE, SW and NW in 11 px at 0.7 alpha;
  - a 6 px tick every 15 degrees that has no letter.
- **Edges**: every mark fades over the outer quarter of each half, so marks
  slide in and out rather than being cut by the edge.
- **Layout**: a pure function in `pbd_app::compass`, so a test can place the
  marks without a window:

```rust
pub fn marks(heading: f32, places: &[Place]) -> Vec<Mark> // offset in -1..1 of the half width, fade 0..1
```

### 4. The heading, steady when looking up or down

The bar reads the active camera. It takes the forward vector off the local
vertical (`up`, the camera's direction from the planet's centre). When the
view pitches toward straight down or straight up, the camera's own up vector
stands in for the part of forward that is lost:

```text
h = flat(forward) - flat(camera_up) * forward.dot(up)
```

`flat` takes the vertical part out. Level, `h` is forward. Looking straight
down, it is the camera's up, which points where the player was facing. Looking
straight up, it is minus the camera's up, which again points the way the
player was facing. In between, it always points along the facing direction.
So the bar never spins when the player looks at their feet. It is the same
formula walking, flying and aboard a craft.

### 5. Places on the bar

- **Which places**: the world's sites (`WorldSites::ready`), within 3 km of
  the player along the ground.
- **Marker**: a 7 px diamond at the site's `bearing`, faded by distance from
  1.0 at 1 km to 0.4 at 3 km. The world is 30 km round, so 3 km is roughly
  what the eye can pick out from a hilltop or a low flight.
- **Name**: the site nearest the middle, within 7 degrees, puts its name and
  distance on the line under the bar, for example "Copan  1.2 km".
- **Towns not yet stood up**: the list is the save's sites, so a town shows
  before it has been stood up. A site that is not built yet is still a place
  to go.

### 6. When the bar shows

- **Only in the world**: the bar shows only while the screen is `Playing`.
  Menus, the pack and the map hide it.
- **High up**: above the ground it fades out between 2 and 3 km. There the
  planet is a globe, and its shape is the better compass.
- **Captures**: the bar is drawn in them like the rest of the HUD, so a
  screenshot can show it.

### 7. The map is drawn compass-north up

The map keeps its coordinates (`geo::project`'s u and v), its rasters and its
cache. What changes is how they reach the screen, which now runs v upward:

- **The transform**: `MapView::to_screen`/`to_map` negate the vertical, and
  so does a drag.
- **The images**: the base, the layer copies and the tiles are placed by
  their flipped rows. The images themselves are drawn flipped: `flip_y` on
  the layer image, and a `1 - v` in `map_image.wgsl`.
- **The live layer**: `map_live.wgsl` turns a node's uv into map v with the
  same flip.
- **The readout and the arrow**: the cursor readout gives `compass_lat_lon`.
  The player's arrow turns by the map's own drawing of the heading (decision
  7a).

Clockwise on screen stays clockwise in the world. Before, both the arrow and
the map were mirrored, so they agreed with each other. Now neither is.

### 7a. The arrow points where a step goes on the map (the eight-wind calibration)

The owner, 2026-10-08: "When I'm facing East in the world, am I facing East
on the map? Calibrate for all other directions too."

`map_screen::tests::facing_each_wind_the_bar_the_map_arrow_and_a_step_agree`
does this through the real systems. At six places it faces each of the eight
winds in turn:
- the equator;
- 28.64° either side, where the spawn is;
- 45° on the antimeridian;
- 60° and 75°.

For each, it reads three things: the compass bar's middle letter, the way the
player's arrow points on the map's screen (`markers`, turned clockwise from
its tip-up picture by Bevy's UI rotation), and the way a step forward moves the
player on the map. With the arrow turned by the true compass heading:

- **The bar** reads the right wind at every place.
- **N, E, S and W** agree exactly. Facing east in the world, the arrow points
  right on the map and a step moves right.
- **The diagonals do not.** An equirectangular map stretches east-west by
  1 / cos(latitude), so a step to the north-east is drawn leaning toward
  east. The arrow stayed at 45°:

| Latitude | Facing NE: the arrow | A step moves on the map | Apart |
| --- | ---: | ---: | ---: |
| 0° | 45.0° | 45.0° | 0.0° |
| 28.64° (the spawn) | 45.0° | 48.7° | 3.7° |
| 45° | 45.0° | 54.7° | 9.7° |
| 60° | 45.0° | 63.5° | 18.5° |
| 75° | 45.0° | 75.5° | 30.5° |

So the arrow turns by the heading as the map draws it:
`geo::map_heading(direction, heading)`, which is
`atan2(sin h, cos h * cos lat)`. Walking the way the arrow points is then
walking the way the player faces. The cardinals do not move. A town the
compass bar has dead ahead is under the arrow's tip on the map, for as far as
a great circle and the map's straight line agree. The bar keeps the true
heading, which is what a compass reads.

### 8. The season

`Clock::season` names the hemispheres in compass words. The year opens on
the summer of the +Y hemisphere, so that is now "southern summer". Its doc
says so. The climate's own comments that call +Y "northern" are about the
frame and are left alone.

### 9. What does not move

- **Generation and saves**: generator versions, settled climates, sites,
  towns, records and saves are untouched.
- **The harness**: `--at` and `--map-at` take frame latitude, and the log
  lines that print an `--at` for a town print frame latitude. Every capture
  command in the screenshot READMEs keeps working. A readout the player sees
  never prints frame latitude.

## Risks

- **Two norths in the code.** A future readout written with `geo::lat_lon`
  would be mirrored again. To catch that, a test walks `src/desktop` and
  fails if a player-facing module calls a frame function. The map module is
  exempt only for `project` and `unproject`, which it uses as coordinates.
- **The mockup.** The published world-map mockup stays mirrored until it is
  redrawn (task 4.3). The game's map is the reference from here on.
