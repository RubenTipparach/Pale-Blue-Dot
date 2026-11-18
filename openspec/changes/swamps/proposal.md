# Proposal

## Why

The owner, 2026-11-18: "the swamps do need to be on this planet", and "right
now I dont see any swamps reall".

Measured on version 10 (`design.md`, "Measured"): the swamp is 1.2% of the
land, every cell of it a strip 2 to 5 m over the sea on the wet coast, and two
thirds of that strip's top is bare dirt, which grows no tree and reads as a
muddy beach. It is there; a player does not see it as a place.

## What Changes

- **Generator version 11** (`TENEBRIS_V11`, `GENERATOR_VERSION = 11`), beside
  versions 4 to 10, which keep their ground cell for cell (version 10 pinned
  first, in its own commit). New worlds are made on it.
- **Swamps where the land is low and wet** (decision 1): on the coast up to
  12 m over the sea where it is wettest, lower toward the drier land, and
  along the rivers up to 3 m over the river's water. Mostly in the jungle's
  wet third (82%), the rest on the wet side of the fields. About 6.6% of the
  land, where it was 1.2%: from the jungle (24.96 to 20.77%) and the fields
  (26.10 to 24.90%). No height moves, so the rivers, the sea, the caves and
  the sites are version 10's.
- **Swamp ground** (decision 3): standing water wherever the ground round
  a cell holds a pool, mud in the wettest patches, swamp moss between; the
  coast's lowest layer as version 10 has it; no cave under a swamp's top
  6 m, so no pool drains. Measured: moss 66%, water 9%, mud 25%.
- **Willows** on the moss at the table's swamp rate (version 10's, unchanged).
- **Reeds** (decision 5): the swamp's ferns drawn as M13's reeds, as the
  approved foliage mockup draws them. Draw-time, so every version's swamps
  show them.
- **The map's biome layer** shows the new swamps (it already has a swamp
  colour and share in its key).
- **`mountain-ground`'s version moves to 12**: it planned version 11 and is
  not built.

## Impact

- `pbd-core`: `planet_gen.rs` (`Swamps`, `swamp_holds`, `swamp_patch`,
  `swamp_pool_held`, V11, `climate_ground`, `site_ground`), `column.rs`
  (the swamp's cave seal), `rivers.rs` (`beside`),
  `terrain.rs` (`GENERATOR_VERSION = 11`), `atmosphere/mod.rs` (reads
  `climate_ground`).
- `pbd-app`: `planet_surface.wgsl` (reeds); the map needs no change.
- Assets: `assets/climate/settled-g11-l{3,5}` (version 10's bytes, decision
  7).
- Saves: no existing world moves (CLAUDE.md, "Saved games survive every
  change"); a world names its version and keeps it.
- Look: before-and-after map captures and an eye-level swamp shot in
  `docs/screenshots/swamps/`. Frame cost not measured (cloud container,
  lavapipe).
