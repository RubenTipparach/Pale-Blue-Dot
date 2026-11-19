# Design: planet sizes and the planet viewer

## Context

- A cell is 2.833 m across on every body, 1 m tall (CLAUDE.md, "Hex size
  is fixed"). Tile width is `1.2087 * R / 2^L`, so the radius and the
  subdivision level move together: `R = 300 m * 2^(L - 7)`.
- The home world, Tenebris, is 4,800 m at L11 (`star-systems`, "The home
  world is Tenebris, unchanged").
- An edited cell is saved by its key (`cell_key.rs`): face, `level - 8` in
  2 bits, and `i` and `j` in 12 bits each.
- `star-systems` design 2 already places every planned body on the ladder.
  Gas giants have no cells, so their size is free.

## 1. The sizes the engine can make

| Rung | Radius | Cells on the surface (`10 * 4^L + 2`) | Circumference | Today |
| --- | ---: | ---: | ---: | --- |
| L7 | 300 m | 163,842 | 1.9 km | **Not keyable** (see 1.2) |
| L8 | 600 m | 655,362 | 3.8 km | Keyable |
| L9 | 1,200 m | 2,621,442 | 7.5 km | Keyable |
| L10 | 2,400 m | 10,485,762 | 15.1 km | Keyable |
| L11 | 4,800 m | 41,943,042 | 30.2 km | **Shipped**: the home world |
| L12 | 9,600 m | 167,772,162 | 60.3 km | **Not keyable** (see 1.3) |

The cell count is per layer of the surface. Every rung up is four times
the cells and twice the walk around.

### 1.1 L8 to L11: carried today

The key's level field holds `level - 8` in 2 bits, so it addresses L8 to
L11, and `i` and `j` reach 4,095, which covers L11's side of 2,048. A body
at 600, 1,200 or 2,400 m needs no change to the key. It does still need a
body of its own: its generator, its frame and its render, which is
`star-systems` and `per-body-rendering`.

### 1.2 L7 (300 m): the floor `star-systems` asks for

`star-systems` puts Sprout and the ten asteroids at 300 m (L7). The key
cannot address L7: its level field starts at 8. An edit on a 300 m body
has no key to be saved under. To fix it, a new key format that also
covers L7 is needed, read beside the old one. That is a save-format
change (CLAUDE.md, "Saved games survive every change"), and is its own
step.

### 1.3 L12 (9,600 m): bigger than the home world

`i` and `j` would need 13 bits (side 4,096), and the level field a third
value. Both are a key format change. Four times the cells also needs
everything that is planet-wide measured first (section 2).
`star-systems` already chose not to go there: Sequoia stays at 4,800 m
"since 9,600 m (L12) would quadruple its cells". Recommendation taken
(ask only with screenshots): L11 stays the largest walkable body until
section 2's numbers say L12 is affordable.

## 2. What size costs

Not measured yet. These are what to measure, by what scales with the
radius.

**Near the player, size changes almost nothing.** The terrain is built
near the player in chunks at 2.833 m cells on every body, so walking on
a 600 m moon and on Tenebris draws about the same ground. Two things
change near the player:
- the horizon is closer on a small body, so less far terrain is in view;
- the ground curves more: it drops about 3 m over 60 m on a 600 m body,
  against 0.4 m on Tenebris (`d^2 / 2R`).

**Planet-wide work scales with the area (four times per rung):**
- cloud puff sites (`cloud_puffs::sites`, `4 * pi * R^2 / spacing^2`):
  about 4,600 at 4,800 m, and 18,500 at 9,600 m;
- sites and towns (`sites.rs`): counts are configured per body, but the
  search covers the whole surface;
- the world map and any whole-planet texture;
- the worm (cave) field (`worms.rs`, sized by `radius_m`);
- the far terrain seen from orbit, which is drawn coarser than the cells,
  so it scales with what is on screen, not with the cell count.

**More bodies at once.** A system adds its bodies' far renders and their
clouds and atmospheres. That is `per-body-rendering`'s budget, not one
body's.

To measure:
- **In a cloud session (CPU, real numbers):** time and memory to generate
  each rung's planet-wide data, at L8 to L11. Use a measurement instrument
  in `tools/`: a bench that changes no behaviour (CLAUDE.md, "Write it up
  before touching code").
- **On the owner's desktop (GPU):** `perf_suite.py` on a body at each rung
  once one exists. A cloud session cannot measure frame time
  (CLAUDE.md), and says so.

## 3. The planet viewer

The owner suggested "a seperate build system in some javacript electron
app or somethign". Recommendation taken (ask only with screenshots): a
**web page served locally**, running **the game's own generator compiled
to WebAssembly**. Not Electron, and not a generator written again in
JavaScript.

- **Same rules, one copy.** `pbd-core` has no engine dependencies (glam,
  serde, ron), and `cargo check -p pbd-core --target
  wasm32-unknown-unknown` passes as it stands (2026-11-19). The viewer
  calls the real `surface_altitude`, `biome`, `top_material` and
  `TerrainConfig::for_version`. A JavaScript copy would be a second
  implementation of a world rule, which CLAUDE.md forbids, and it would
  drift.
- **No Electron.** A browser already runs WebAssembly and WebGL. Electron
  adds a 100 MB install and a second build for nothing a page cannot do.
  The page can also be published as an artifact, with the `.wasm` as a
  supporting file, so it opens from the tracker like every mockup.
- **Layout:**
  - a crate `tools/planet-viewer/` (wasm-bindgen, outside the game's
    workspace build) exposes a few functions over `pbd-core`;
  - `tools/planet-viewer/index.html` (three.js, as the mockups use) draws
    with them;
  - one script builds the `.wasm`.
- **What it shows** (one 3D view, with tabs as CLAUDE.md asks):
  - **Globe:** the whole body at a rung, coloured by biome and height, lit
    by day or at night, dragged to turn and never spinning on its own.
  - **Ground:** a patch at the real 2.833 m cells and 1 m layers, with the
    atlas's own tiles, so a rung's curvature and horizon can be seen
    standing on it.
  - **Controls:** rung (L7 to L12), seed, generator version (every version
    a save can name), and a side-by-side of two rungs.
  - **A size card** per rung: radius, cells, circumference, gravity at the
    game's g, and how long a walk round takes at walking speed.
  - **Planet types:** whatever `TerrainConfig` carries today (the
    versions). Airless, ice and gas giant types arrive with
    `star-systems`; until then the viewer says which ones exist.
- **Cost:** it runs on the viewer's machine and touches no game code.

## Risks

- **The wasm build drifts from the game.** It calls the same crate at the
  same commit, so a change to the generator shows in the viewer on the
  next build. A test compares a few altitudes from the wasm build with the
  native ones.
- **The whole globe at L11 is 42 million cells.** The viewer samples a
  coarser grid for the globe (as the far render does) and builds real
  cells only in the Ground tab's patch.
