# Proposal: glowing mushrooms to pick and eat, and a carried light

## Why

The owner, 2026-11-19: "Players also asked to pick up glowing mushrooms. You
can eat them and glow light from your body temporarily." Narrowed the same
day: "Allow players to pick up the glowing mushrooms that exist in the game
today." Then: "Holding a light source, torches lamps or mushrooms or anything
that is a light should illuminate surrounding area", and "Need to make this
fast so this would be realtime for the light emanating from player."

What the game has today (read in the code, 2026-11-19):

- **The glowing plant** of `lamps-and-lanterns` decision 8. A finest cell grows
  one where `pbd_core::flora::glows(key, glow_flower_chance)` says so, on its
  exact key, with `glow_flower_chance` 0.01 in `assets/config/column.ron`.
  `planet_column.rs` sets `GLOW_FLOWER_BIT` (bit 0 of the record's
  `spare[2]`). The shader (`planet_surface.wgsl`, the flower slot) draws it on
  any green cell, whatever its own flower roll says. At dusk the tier adds it
  as an emitter at `GLOW_FLOWER_LEVEL` 3 and sets `GLOW_LIT_BIT` (bit 29 of
  `more[3]`), and the head is drawn as its own cool light. The code and the
  write-ups call it a flower. It is drawn as a 32 cm stalk under a low,
  four-sided cap 14 cm across and 3 cm high, which is a small mushroom's
  silhouette, and the owner calls it a mushroom (design 1).
- **Nothing carried gives light.** Torches and lanterns are placeable blocks
  (`Material::Torch`, `LanternPost`, `LanternWall`, `LanternHanging`, with
  their levels in `Material::emission`). `held.rs`'s `hand_light` only shades
  the hand mesh. The cooking change already lists a Glow effect, "a 4 m light
  on the player", and holds it back: "glow waits for a light that follows the
  player" (`pbd_core::cooking`, `cooking` design S5).

## What Changes

- **The glowing plant becomes the glowing mushroom**, in names the player
  reads and in this change. Its shape and where it grows are as they are. A
  rounder cap is recommended as a separate look change, asked with
  screenshots (design 1).
- **F picks one you aim at**, the way F picks fruit (`fruit-aim`). It gives
  one `glowing-mushroom` good, with a 16 px icon and the usual item toast. The
  pick is a stored `glow-plant` record keyed by the cell's exact key. A picked
  cell draws no plant and gives no light. It grows back after 8 world days,
  and only while its cell is out of sight (design 2 to 4).
- **No new generator version.** Where the mushrooms grow is already a pure
  function of the cell key and a config chance. A world that has never been
  picked from saves nothing new, and an old save opens as it did (design 5).
- **Eating one** (right click with it in hand, as a dish is eaten) runs the new
  `Glow` effect for 3 minutes of play. There is no hunger. Eating another
  while it runs sets the time back to the full 3 minutes. It never stacks
  (design 6).
- **One carried light for everything that glows**, held or eaten. A torch,
  any lantern or a glowing mushroom in the selected hotbar slot lights the
  world around the player, as the body glow does. The level and colour of
  each come from the placed thing's own values (`Material::emission`,
  `GLOW_FLOWER_LEVEL`, `TORCH_TINT`), through one table. The light moves every
  frame with no lag and no steps from cell to cell. Rock blocks it. It lights
  the ground, walls, trees, clutter, water, and everything `LitByField`
  (drops, craft, creatures, the hand and the held item). Nothing about it is
  saved (design 7 to 12).
- **Speed is the first requirement.** The carried light is a point light that
  the terrain, field-lit and water shaders evaluate beside the baked field.
  Its shadow is a short walk through the column records, from the exact
  light position to the pixel. The field is not re-baked as the player moves
  (design 8). Three approaches are compared, with a cost estimate for each.
  The owner measures the cost on the RTX 3070 and the M4 Max with a
  `perf_suite` A/B. A cloud session cannot measure it (design 13).
- **Mockup:** a new tab, Glow and carried light, on the Foliage Detail
  Playground (M13, `docs/mockups/foliage.html`, version 2). It shows a night
  patch of the plants as the game draws them, picking with F and the toast,
  eating and the body glow with its timer and fade, a held torch, lantern and
  mushroom compared with Today, a wall and a doorway that block the light, and
  a day view.

## Capabilities

- `planet/flora`: picking a glowing mushroom; a picked cell draws none and
  lights nothing; regrowth.
- `planet/light`: the carried light, its sources, its shadow, what it lights.
- `player/glow`: eating a glowing mushroom, the Glow effect, its fade.

## Impact

- `pbd-core`: `flora.rs` (`glow_plant_point`, the record, regrowth),
  `light.rs` (`CarriedLight`, `carried_light(item)`), `cooking.rs`
  (`Effect::Glow`), `goods.rs` (`glowing-mushroom`).
- `pbd-app`: `planet_column.rs` (picked cells), `desktop/glow_plants.rs` (aim
  and pick), `kitchen.rs` (`MealEffects::glow`), `field_light.rs` (carried
  lights to field-lit materials), a `carried_light.rs` system,
  `planet_surface.wgsl`, `field_lit.wgsl`, `water.wgsl`.
- Assets: `assets/config/cooking.ron` (the dish), `assets/config/light.ron` or
  `column.ron` (the carried light's tunables), the icon
  `assets/items/goods/glowing-mushroom.png`.
- Saves: a new record kind, `glow-plant`, at schema 1. `Effect::Glow` is a new
  value in the existing `meal` record. Nothing else in a save changes.
- Frame cost: not measured here (a cloud container with no GPU). The
  estimates are in design 13, with the owner's measuring plan.
