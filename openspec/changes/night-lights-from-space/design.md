# Design: night lights from space

## 1. Drawn by the planet pass, as light added to the ground

The far lights are added to the terrain's colour in `planet_surface.wgsl`
(`shade_terrain`), and folded in by `finish` before its haze. No new pass, no
new pipeline, no new binding:

- **No pipeline after `READY`.** The terrain pipelines are already compiled by
  the loading screen's prewarm; a new point-sprite pass would be one more to
  prewarm (`CLAUDE.md`, "A loading screen on boot").
- **Clouds and haze for free.** The clouds are composited over the main pass
  (`planet_water.rs`), so a cloud deck over a town hides its lights by the
  cloud's own opacity. The ground's haze (`finish`) dims them by distance.
  On top of that, the lights at a pixel are thinned by the weather map's cover
  there (`weather_cloud`'s x, which the ground's cloud shadows read too), so a
  town under a deck is dark even where the clouds pass draws a thin far shell
  of it.
- **The night side only.** Light is added, never lit: the sun does not touch
  it, so it shows by contrast only where the ground is dark.

Rejected: billboards at the sites (a pipeline of their own, no haze, drawn
through cloud unless sorted against it); baking a glow per coarse hex (the
`cities-in-the-world` design 7 sketch): it needs a per-hex buffer rebuilt as
the sites load, and the per-pixel glow below gives the same lit patch.

## 2. The data: a uniform list, built on the CPU each frame

`PlanetParams` gains, at its end (so the visibility pass, which reads a prefix
of the same uniform, is untouched):

```text
night: vec4<f32>                       x lights in the list, yzw spare
night_lights: array<vec4<f32>, 128>    two vec4 per light, 64 lights
  [2i]   xyz the light's direction (planet frame, unit), w cos(reach / R)
  [2i+1] x radius m, y strength now, z seed, w point spacing m
```

A light's **strength now** is decided on the CPU, per light, by
`pbd_core::night_lights::strength`:

```text
strength = brightness(kind or count) * night(e) * near(d)
```

- `e` is the sun's elevation (a cosine) at the light's direction.
- `night(e) = 1 - smoothstep(-0.12, 0.02, e)`: the sky's own night term
  (`sky_light::sky_radiance`'s moonlit sky), now one function,
  `sky_light::night`, that both read. The lights grow as the sky darkens and
  are whole once the sun is about 7 degrees down. The street lamps come on a
  little earlier (`daylight::DUSK_ON`, 0.05), when they are too faint against
  the dusk ground to be seen from far anyway.
- `d` is the camera's distance to the light's centre, and
  `near(d) = smoothstep(near_out_m, near_in_m, d)`, 1200 m to 2200 m. A
  town stands within 1200 m (`towns::STAND_M`), so the far lights are gone
  where the town's own lanterns and windows are drawn, and are whole well
  before they are needed from altitude. They never double the real lamps.
  (A first try faded them over 700 m to 2000 m; from 900 m above a town its
  far points showed faintly over its own, so the fade starts where the town
  stands.)

Lights of strength nought (the day side, the near ones) are left out, and the
list keeps the 64 strongest, ties by order of making, so the list is stable
frame to frame. A world has about 55 sites; the rest of the room is for the
player's clusters.

## 3. What a far light looks like

For each light, a pixel first tests `dot(radial, direction) > cos(reach)`:
one dot per light per pixel, which is all most pixels pay. Inside the reach:

- **Points.** The tangent plane at the light is cut into a grid of `spacing`
  (12 m for a town, 8 m for the player's). Each grid cell holds at most one
  lit point, by a hash of the cell and the light's seed, with a chance of
  `0.5 x quarter x exp(-2.5 (r/R)^2)` (R the light's radius): likelier toward
  the centre, and likelier in some quarters (blocks of four cells, `quarter`
  a hash between 0.25 and 1.25) than others, so a town is ragged rather than
  a disc. The point sits at a random place in its cell with a random
  brightness. A pixel sums the 3 x 3 cells round it. Each point is a soft
  blob of radius `max(2.5 m, 1.5 x the pixel's footprint)` (the footprint is
  `foot`, taken at the top of `shade_terrain`), and its brightness is divided
  by how far the blob was widened, so from afar a point stays a pixel-sized
  spark rather than a disc. Where a pixel spans several cells, the sum is
  blended into its expected value, so the far town does not shimmer.
- **Glow.** Under the points, a faint warm glow, `0.02 x exp(-4 (r/R)^2)` of a
  point: the "lit patch of ground" of survey C3, with the bloom of the points
  doing most of it. The light is faded out over the last quarter of its reach,
  so it never ends on a ring.
- **Colour and gain.** Sodium-warm, `(1.0, 0.70, 0.36)`, times 3 (HDR: the
  display transform has room above white and the bloom picks it up).
- **Not under the sea.** A pixel below the water sheet draws none: the sea
  drawn over a harbour's quay would tint those lights as seabed.
- **Under a deck.** The weather map's cover at the pixel takes up to 97% of
  it (`1 - 0.97 smoothstep(0.2, 0.85, cover)`), and the clouds pass then draws
  the deck over what is left.

## 4. Brightness by kind

`assets/config/night_lights.ron` (validated, units in the names):

| Kind | Radius m | Brightness |
| --- | ---: | ---: |
| Walled (the capital x1.3 radius, x1.25 brightness) | 150 | 1.0 |
| Harbour | 110 | 0.8 |
| Village | 90 | 0.6 |
| Desert, Tundra, Jungle, Swamp, Cliff | 80 | 0.5 |
| Cave | 0 | 0 (under the rock: nothing shows) |

The site list is the saved one (`WorldSites::ready`), so a world's far lights
are where its map says its towns are, built or not yet built.

## 5. The player's lights

`pbd_core::night_lights::player_clusters(lamps, config)`:

- **Which lights.** Every edit whose material emits (`Material::emission() >
  0`) on a cell the player has touched (`YieldSet::has_cell`), positioned at
  its cell's direction (`cell_key::direction`, new: the key's lattice point by
  the same add-then-normalise midpoint descent `planet_lattice` uses).
- **Grouping.** Each light falls in a cube of a 3D grid of `link_m` (64 m)
  over the planet's frame. Occupied cubes that touch (26-neighbourhood) are one
  group, joined by union-find over a `BTreeMap`, so the grouping is the same
  whatever order the edits came in. No face seams: the grid is the planet's
  frame, not the icosahedron's.
- **Threshold.** A group of at least `player_threshold` (12) lights is a far
  light. Its centre is its lights' mean direction; its radius the furthest
  light from that, at least `player_min_radius_m` (30 m); its brightness
  `player_brightness * min(count / player_full_count, 1)`, full at 60 lights.
- **When.** Recomputed when the edits change, detected by a revision counter
  `Edits::revision`, bumped by every `set`. Placing or removing a lamp is one
  recompute of a list of the player's lights, which is small.
- **Derived, never saved**: nothing is written.

## 6. Glowing flowers: a quarter as many, a quarter as bright

- **How many.** `glow_flower_chance` 0.04 -> 0.01. `flora::glows` compares a
  fixed hash of the key with the chance, so every cell that glows at 0.01
  glowed at 0.04: the survivors are a quarter of the old glowers, picked by
  the hash, with no new ones. A test pins the subset.
- **How bright.** The head's emitted colour is multiplied by
  `GLOW_HEAD_GAIN = 0.25` in the shader. The light the flower puts on the field
  drops from level 6 to level 3: `lamp_strength` (the field's flood curve) is
  0.41 at level 6 and 0.13 at level 3, a factor of 0.32; level 2 would be
  0.15. Level 3 is the nearest whole level to a quarter, and it lights only the
  flower's own cell (a step across costs three).
- **The bake's filter.** `bake_lamps` left the flowers out by `level >
  GLOW_FLOWER_LEVEL`; every real lamp is level 8 or more, so the filter keeps
  the same lamps at level 3.
- **Saved games.** The glow choice is not a generator's: it is decided from
  the cell key by `ColumnSettings` at record build, is not in any generator
  version's digest (`planet_gen`'s digests are altitudes, biomes and top
  blocks), is not an edit, and no save names it. A flower that stops glowing
  is a cell whose clutter roll draws an ordinary flower or none. So it is
  changed in place, without a new generator version, and no world changes
  shape.

## 7. Cost

Not measured: no perf sitting in this change (`CLAUDE.md`: a change that can
move frame time runs `perf_suite.py`; the owner runs it). Expected: one dot
product per light in the list per terrain pixel at night, and 9 hashes per
pixel inside a light's reach; nothing by day (the list is empty), nothing near
a town (faded out). The CPU builds a list of about 60 lights per frame; the
player's clusters are rebuilt only on an edit. The uniform grows by 2 KB.
