# Design

## 1. Drawn, not solid

`Material::modelled()` (pbd-core): `Chest | Workbench`. It is a draw fact
only. `Column::solid` is unchanged, so collision (the contact tier reads
cells, not drawn runs), the light field, aiming, digging and the save are
untouched.

- `Column::drawn_runs` builds its runs from `solid && !modelled`, so the GPU
  record has no cap or walls for the cell: the run under it ends at its
  floor and draws a cap there, and a neighbour's walls face into it.
- `planet_column::pack_materials` writes the cell's render code as air (0),
  so the shader's neighbour reads (a wall's open stretch, the cap's
  occluders) see it as open, the same as the runs.
- Measured callers of `drawn_runs` outside the draw (the cave-spawn search
  in `desktop.rs`, two tests) only look for air gaps two metres high or
  more; a one-metre modelled cell is never one.

## 2. The models

Both in `pbd_core::furniture::model`, in the furniture's frame (x across, y
up, z to the front, origin on the floor under the middle):

- `station_chest()`: `polys(Kind::Chest)` scaled 1.25 (1.1 m by 0.62 m,
  0.6 m tall), so it reads at the size of a cell's contents.
- `workbench()`: top 1.6 m by 0.7 m, 0.09 m thick, its face 0.88 m up; four
  legs 0.09 m square tapering to 0.07 at the foot; stretchers 0.15 m up on
  all four sides; a boards shelf on them; a vise jaw block at the right end
  of the front with a turned iron screw and a timber handle through it; a
  mallet (turned handle, timber head) and a chisel (iron blade, timber
  handle) on the top.

## 3. Standing it

`farm::show` already spawns a view per placed chest and workbench near the
eye. For a modelled cell it stands the model at the cell's FLOOR (layer
altitude, not the top), turned to face the player who set it down, or the
cell's own north when unknown, with `LitByField` and `LitLikeTerrain`. The
strap, hasp and vise dressing on the old hex goes.

## 4. Known limits

- Standing on a chest: the collision top is still the cell's, 1 m up, so a
  player stood on a 0.6 m chest stands 0.4 m above its lid. Rare; a lower
  collision top would be a change to the contact tier, asked for separately.
- The coarse heightfield tiers far off still draw the cell as a bump of
  wood. A chest is never far enough off to be seen there.

## 5. Checks

- A test: a column with a chest on its top layer has its drawn runs end
  below it and its `solid` still true there.
- Close-up captures of each model in the game, before and after, from 2 m
  and 4 m, at day and at night by a lamp (CLAUDE.md: models are looked at
  close up before they are shown), in `docs/screenshots/station-models/`.
