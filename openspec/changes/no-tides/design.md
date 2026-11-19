# Design: no tides

## Context

The owner, 2026-11-19: "Remove literal tide feature."

The tide is `ocean-environments` task 5.1 (built 2026-11-15): an offset of
the sea's surface from the moon, `pbd_core::sea::tide_m(p, tide)`, where
`tide` is the moon's unit direction in the planet's frame and the range in
`w` (3 m, `water.ron`'s `tide_range_m`). It is the second Legendre term of
the moon's angle, so at one place the water rises and falls by the range
twice a day, up to two thirds of it (+2 m) over the mean and a third (-1 m)
under it. `desktop/scene.rs::move_moon` set it every frame from the moon's
orbit. It was never saved.

What remains after it: the sea's mean surface, the drawn sheet, at the
sea's level less `depth_offset_m` (0.5 m), with the waves of the one sea
table on it. Nothing else about the sea changes.

## 1. Remove the code paths, not the range

Setting `tide_range_m` to nought would leave every reader carrying a term
that is always zero, a uniform lane that is always zero, and a capture flag
that does nothing. The tide is removed from the code instead, and each
reader is left reading what it read with no tide: the mean sheet plus the
waves. Where a reader took "the tide's highest reach" as a margin, it takes
the sea's level, which is the mean sheet plus the waves' margin at a shore
(design 3).

## 2. Every reader, and what it does now

| Reader | What it did with the tide | What it does now |
| --- | --- | --- |
| `pbd_core::sea::tide_m`, `tide_high_m` | the offset, and its highest (2/3 of the range) | removed |
| `LocalSea::tide`, `with_tide`, `at`, `height` | added `tide_m` to the waves' height | the waves' height alone |
| `LocalSea::reach` | every crest plus the tide's highest | every crest |
| `SeaGpu::tide`, `sea.wgsl` `SeaView.tide`, `sea_tide` | the moon and range for the shader, added in `sea_surface` | removed; the uniform loses its last `vec4`, the Rust `planet_water::SeaView` the same, so the two layouts still agree |
| `pbd_app::sea::Sea::tide`, `tide_range_m`, `set_moon`, `tide_at` | the one tide every reader read | removed |
| `SeaNow::tide_here` | the tide under the camera | removed |
| `desktop/scene.rs::move_moon` | set the tide from the moon; `--tide high\|low` put the moon over the camera | moves the drawn moon only; the flag is removed (an unknown flag stops the capture, as any other) |
| `config.rs` `WaterSettings::tide_range_m`, `water.ron` | the range, validated non-negative | removed; an old file's key is read and ignored (design 4) |
| `vehicle::Surroundings::tide` (`pbd-core`) | the hulls floated on the tided sea | removed: the hulls float on the waves |
| `vehicles.rs` | the tide at sea, none on a river | the sea's surface at sea, the river's on a river, as before |
| `vehicles/place.rs::on_the_tide`, `harbour.rs`, the `--aboard` launch | lifted a new craft to the tide's height | removed: a new craft is made on the mean surface, which is the water's |
| `vehicles/launch.rs::water_along` | the sea's surface with the tide | the sea's mean surface |
| `walking::Sea::sheet_radius` | the drawn sheet plus the tide there | the drawn sheet |
| `audio/hooks.rs` `water_underfoot` | (reads the walker's answer) | unchanged; its test held a seabed block wet at high tide and dry at low; it now holds the block 1 m under the sea's level wet, and a body over the sheet dry |
| `planet_water::submersion`, `eye_surface_radius`, `composite_surface_radius` | the sea's mean plus the tide at the eye | the sea's mean (the sheet radius) |
| `planet_water::eye_water_at` (`high_water` argument) | air a reached cell holds under `sea level + tide_high_m` is the sea's | air a reached cell holds under the high-water line (`terrain::high_water_m`, the sea's level) is the sea's; the argument goes |
| `planet.rs` `water_deep.w` | the tide at the camera, lifting the ground's underwater tint | spare (nought); `planet_surface.wgsl` tints under the sheet radius `water_absorption.w` |
| `planet.rs` `lod_spare.w`, `planet_visibility.wgsl` | the sheet listed over ground under `tide_high - offset + 1` m | the sheet listed over ground under `terrain::high_water_m()`, the CPU's own line, so `sheet_listed` and the shader read one number |
| `planet_terrain::high_water_m`, `HIGH_WATER`, `set_high_water` | the tide's highest less the offset (+1.5 m), set from the loaded config | the sea's level; the static and its setter go (design 3) |
| `fish.rs` `PlanetWater` | the float and the line's waves with the tide | the waves alone |
| `fish.rs` `line_floor` | the line held over the tided waterline and the crests on the tide | over the drawn sea and the mean (the composite's waterline), and over every crest |
| `fish/places.rs` floes | a floe laid where it floats at low water (mean less a third of the range), its mesh lifted by the tide each frame | laid where it floats at the mean; not lifted |
| `fish/places.rs` floe seals, ambient life | drawn on the tided floe; tinted by depth under the tided surface | on the floe; by depth under the mean |
| `desktop/eye_path.rs` | measured the eye path's water from the tided sheet; printed the tide | from the sheet; the tide is gone from its label |
| `water.wgsl` `SEA_EYE_RANGE_M` | "the tide's whole range and some", 3 m | the same 3 m, said as the waves' reach and some: it decides which surface the eye's is, and a smaller one would be a look change for no reason |
| comments in `water.wgsl`, `sky_atmosphere.wgsl`, `water_flow.rs`, `planet_contact.rs`, `shader_tests.rs` | named the tide | say the waves or the sea's surface |

Tests that were about the tide go with it:
`the_tide_ranges_twice_a_turn_and_keeps_the_seas_volume` (`pbd-core`),
`every_boat_floats_on_the_tide` (`pbd-core` vehicles), and the parity
test's tide cases (`sea_gpu_tests`, which keeps its three sea states, the
tide off). Tests that used the tide to make a case are rewritten to the
case with no tide: the swim, the audio underfoot, the jump over a beach, the
line to the float, the B22 reach and the `dry-holes` beach.

## 3. The high-water line is the sea's level

`terrain::high_water_m()` is the line under which a cell's ground has water
on it. It was the tide's highest less the sheet's offset, `2.0 - 0.5 = 1.5`
m over the sea's level. With no tide it is the sea's level, `sea_level_m`,
not the sheet's `-0.5`: the column's own sea stands to the sea's level, and
the half metre between it and the drawn sheet is where the waves break at a
shore (a wave there is held to `depth_limit`, 0.6, of the water's depth, so
no crest passes the sea's level over ground at or above it). Its readers:

- **The flooded-ground bit** (`flora-v2` 12.11, `surface_code`,
  `reconcile_surface`): set on a top under the line, or with its column's
  water over it. Ground from the sea's level to 1.5 m up is no longer
  flooded. The bit is a draw-time fact, never saved
  (`pbd_core::flora::FLOODED_BIT`'s own doc), so no generator version
  moves; `TreeTable::grows` and the generator's beach rules are untouched.
- **The sheet's listing** (`sheet_listed`, `planet_visibility.wgsl`): a
  record whose ground is under the line, or that the sea reaches. With the
  sheet at -0.5 m, ground between -0.5 and 0 hides it, as ground over the
  sheet always did.
- **The sea's reach** (`ColumnTier::sea_reach`, B22): its walk goes up to
  the line's layer, and never less than the first layer over the sea's level
  (`.max(sea)`, unchanged). That layer is the air just over the sea's top;
  a beach at the sea's level is still crossed, and a pit behind it is still
  reached, as B22 asked. A beach a metre up is no longer crossed: only the
  tide went over it.
- **`mark_sea`'s dry mark** and **the eye's water** read the same line.

## 4. An old `water.ron`

`WaterSettings` denies unknown fields, so a `water.ron` that still names
`tide_range_m` would stop the game at launch. The game ships its config
with it, but a newer game beside an older config (an unpacked update over a
kept copy, a hand-tuned file) should still start. So the struct keeps a
private field named `tide_range_m` of type `serde::de::IgnoredAny`: the key
is read and thrown away, the shipped file no longer has it, and no code
reads it. It is documented as retired.

## 5. Saves and generation

- The tide was derived from the clock and the moon, never stored: no save
  format change, and an old save opens to the same world with a still sea.
- No generator's code or value changes. `pbd_core::flora` (the
  flooded-ground bit's definition, the beach and beach-back rules),
  `planet_gen`, terrain, biomes and sites are untouched but for wording in a
  comment; `cargo test -p pbd-core planet_gen` passes unchanged.

## 6. Captures

A before and after at the beach of `sea-hole-beach` (`39.725 15.339`, the
owner's spot), at the high tide before (`--tide high`, the sheet 1.5 m over
the mean there) and after, side by side, in `docs/screenshots/no-tides/`.
Lavapipe in a cloud container at a fixed step: they show what the shore
looks like, not how smoothly it runs. Frame time is not measured here.

## Risks

- **Plants at the waterline.** The flora rules may stand a tree on a beach
  cell just over the sea's level that the tide used to keep bare. That is
  the generator's own placement on dry ground; the owner judges it in the
  captures.
- **A capture script with `--tide`** stops with "unknown argument". The
  READMEs that name it describe captures already taken.
