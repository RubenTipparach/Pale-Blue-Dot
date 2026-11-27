# Proposal: gathering the sea floor's plants and animals (sea-gathering)

## Why

The owner, 2026-11-25: "I want to be able to gather specific aquatic life.
The various plants and animals that live on sea floor, including reefs."
And the same day: "Reefs have specific models. It will be nice to collect
and sell them, crush them for fertilizer, or use them as decoration."

What the game has today (read in the code, 2026-11-25):

- **The reef's pieces** (`crates/pbd-app/src/fish/places.rs`, the
  `ocean-environments` design 5, 11, B14 to B16, B30 and `reef-perf`):
  derived like trees and never saved, laid tile by tile round the eye from
  `pbd_core::ocean::places::environment`. Each is one of a set of models
  (`Kind`): staghorn (`Branch`), finger coral, brain (boulder) coral, table
  coral, soft coral, tube and barrel sponges, sea fans, anemones, the big
  anemone with its clownfish (`Hero`), algal turf, coral rubble; on a kelp
  bed the kelp stands (M25's cards) and their rocks; round a vent a black
  smoker, bacterial mats and mussel beds. Every one is the same model in
  every tile, tinted from its kind's palette.
- **The bed's animals** (`pbd_core::fauna`, `Mover::Crawler`): the shore
  crab, the lobster, the giant clam, the octopus and the tube worm walk or
  sit on the bed in schools round the player. Each already has a trade
  value (`fish:<id>` in `values.ron`) and an icon, and a caught one is an
  `Item::Fish` that the campfire cooks as fish. None can be taken: a line
  refuses them (`Species::catch_refusal`), and nothing else reaches them.
- **Protected creatures** (`ocean-environments` D9): reef turtles, sea
  otters, seals, ducks, beavers and river otters are refused.
- **The grinder** (`townsfolk` design 41) crushes stone. **No fertiliser**
  exists in the game; M8's compost ("half again as fast, one more at
  harvest", `animal-husbandry` design) is written up and not built.
- **Placing a good of its own** (the fence, the incubator): right-click with
  it held sets it on the aimed ground as a stored record; F takes it up.

## What Changes

- **F gathers a sea-floor piece by hand** while swimming or wading, aimed
  at the piece as it is drawn, within reach. Each reef model is its own
  item (eleven of them), and so are kelp and the vent's mussels. The piece
  goes out of the world on that frame and grows back after its kind's days
  of world time, when its tile is next built. A take is a stored
  `sea-take` record by the piece's id, written with the pack in one line.
- **F takes a crawler** (crab, lobster, clam, octopus, tube worm) by hand:
  the same `Item::Fish` a catch would be. Protected creatures are refused
  with the game's existing refusal.
- **Sell**: every new item has a value and a group, so every town trader
  buys it at a price per kind.
- **Crush**: the grinder turns every reef piece, kelp and sea lettuce into
  **fertiliser**, a new good. F on a growing crop with fertiliser in the
  pack feeds it once: the rest of its growth runs half again as fast and
  its harvest gives one more. An old plot reads as unfed.
- **Cook**: two campfire dishes, steamed mussels and kelp crisps. Crabs,
  lobsters, clams and octopus cook as any fish already does.
- **Decorate**: right-click with a coral, sponge, sea fan or anemone held
  sets it on the aimed ground, on land or on the sea floor, one to a cell:
  the same model the reef draws, lit as the terrain is, kept as an
  `ornament` record. F takes it back up.
- **Icons**: each new item's 16 px pack icon is rendered from its own
  model by a committed script (`tools/art/sea_gather_icons.py`, reading
  the models dumped by a test), with a 1 px outline (UI only).

## Capabilities

### New Capabilities

- `world/sea-gathering`: what can be gathered from the sea floor, by what,
  what it gives, how it grows back, what is refused, and what the items
  are for (selling, crushing, cooking, decorating).

### Modified Capabilities

None. The reef, kelp and vents are drawn and placed as they were; no world
generates differently (no generator version).

## Impact

- `pbd-core`: a new `sea_gather` module (the roster, the records, the
  feeding rule); `goods` rows; `farming::Plot` gains an optional `fed`
  field; `values.ron`, `recipes.ron`, `cooking.ron` rows; locale keys.
- `pbd-app`: `fish::places` gives each piece an id and a gatherable kind,
  leaves taken ones out of a tile and exposes the aimed piece; a new
  `desktop::sea_gather` reader (gather, crawlers, ornaments, feeding).
- Saves: two new record kinds and one optional field; an old save opens as
  it was, with nothing taken and every plot unfed.
- Frame time: picking rebuilds one tile once (the cost of a tile landing);
  the aim walks the pieces of tiles within a few metres each frame; each
  placed ornament is one small mesh. Not measured (this session does no
  performance work); the owner measures it.
