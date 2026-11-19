# Proposal

## Why

The owner, 2026-11-16, on a mountain range seen from the air (one dark slate
texture over the whole flank, crossed by regular rows of tan stripes, with no
grass or snow on its flats): "mountains should have dirt/grass or snow up
where it is flat, or portions wherever possible, it should also consist of
some various rock patterns so its not all the same rock on a mountain".

Measured in version 10 (`design.md`, "Measured"):
- a Mountains cell's top is Stone below 200 m and Snow above, and nothing
  else, because `top_material` reads altitude and latitude but never slope;
- every rock (Stone, desert Rock, Ore) draws as one render code, so a
  mountain shows exactly one rock picture (slate) of the sixteen its sheet
  holds.

## What Changes

- **Generator version 13** (`TENEBRIS_V13`, beside V4-V12, which stay cell
  for cell; version 11 went to `swamps`, 2026-11-18, so this builds on
  version 11's ground): a Mountains cell that is flat (no neighbour more than a metre
  off) takes alpine grass or dirt below a ragged snow line and snow above
  it; steep cells stay rock; the snow line comes down onto the flats.
- **Mountain grass and dirt drawn as alpine soil**, not as the mountains
  sheet's scree at (0,0).
- **Rock variety at the draw**: the rock picture chosen per cell from the
  sheet's rocks (slate, granite, fractured slate, dark shale, lichen stone)
  by bands of altitude and a coarse noise, so a flank shows strata and
  patches. Draw-time only, so it holds for every version's worlds.
- **The tan rows**: the far fade of a riser under snow or grass takes the
  mountain's own mean, not the lowland ground's tan.

## Impact

- `pbd-core`: `planet_gen.rs` (V11, a slope test in `top_material`),
  `terrain.rs` (`GENERATOR_VERSION = 12`), a digest test of V11 taken first.
- `pbd-app`: `planet_surface.wgsl` (rock tile per cell, alpine grass and
  soil, risers' far colour).
- Saves: no change to any existing world (CLAUDE.md, "Saved games survive
  every change"); new worlds are made on version 13.
- Look changes go to the owner with before-and-after captures.

## Mountain flora and features (added 2026-11-19)

The owner, 2026-11-19, on M13: "nice, can you add more stuff to the mountain
biomes? add stuff to foliage detail playground".

Read in the code (design 5): a mountain today grows nothing of its own. Its
trees are 0 in 256 cells in every generator version (`TreeTable::V4`,
`V10`), it has no fruit, and its clutter rolls only pebbles and, on snow,
dead shrubs, because the green pieces need grass and a version 12 mountain
has none. Its tufts take the scree tile's greys.

**Recommended: one change, not two.** The flora goes into this change rather
than a new `mountain-flora`, because what grows where on a mountain hangs on
what version 13 lays down: the flats, the alpine grass and the ragged snow
line. Version 13 is not built yet, so the generated parts of the flora (the
trees, the tree line, the scoured ridges) ride in the same version and the
same digest, instead of a version 15 that repeats its rules
(`ocean-environments` holds 14). Recommendation taken (ask only with
screenshots); the look goes to the owner on M13's new Mountains tab.

What changes, by altitude band (design 5 to 14):

- **Montane forest edge**: firs and spruce on the lower flats, thinning
  upward to a tree line; **krummholz**, stunted wind-bent conifers, at the
  line. Version 13 (whether a tree stands on a cell is generation).
- **Alpine meadow** on the flats: an alpine turf tile and its pixel tufts,
  cushion plants, small alpine flowers in four low-saturation colours
  (gentian blue-violet, edelweiss white, yellow, pink), dwarf juniper,
  heather and bilberry bushes. Draw-time, in the existing clutter slots, so
  no new vertices.
- **Rock**: scree below cliffs with boulders, lichen patches (orange,
  yellow-green, grey) drawn into the rock texels, moss in seeps. Draw-time.
- **Snow zone**: snow patches lingering on shaded flats below the line, ice
  on the high flats, wind-scoured bare rock on the ridges, a rare hardy flower
  at the snow edge. The patches and ice are draw-time; the scoured ridges are
  version 13.
- **Proposed, not in this change's build**: mountain tarns (version 13,
  still water in closed basins, if `water-flow` agrees), cairns on summits,
  bilberries as a forage item for cooking, and mountain goats, marmots and
  eagles (a note for animal work). No falling water: a waterfall's look is
  designed only in its own mockup (CLAUDE.md).

Old worlds keep their ground and their trees; the draw-time parts reach every
world, as decision 3's rock variety does, and are a look change shown first.
