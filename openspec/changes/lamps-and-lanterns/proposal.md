# Proposal: lamps and lanterns (the lights, step 1 of the cities plan)

## Why

**The owner's plan (2026-09-27): "step 1 implement lights, step 2 implement
cities, step 3 implement ways for me to make cities."** Lights come first
because a city at night is lamps, lanterns and lit windows. The towns mockup
showed what the owner wants; the owner's verdict on its lighting was "that
lighting is just excellent". The game can light a place already, but only
partly.

The game has a day and a night, a sky field that darkens caves, and one light
you can place, the torch. The verification run on 2026-09-27 found what is
still missing from the owner's original request for lighting (`night-and-lamps`):
- **Moving things are not lit.** The player, the ship and anything else that
  moves take no light from the field, although the owner asked that "if an
  object is non static like player, items or entities like animals they should
  receive dynamic lighting from these sources".
- **Bioluminescent flowers** are not built.
- **The torch has no icon of its own**; it shows a tinted wood tile.
- **One call site keeps its own copy of the noon sun**, which the requirement
  "no second copy of the sun" rules out.
- **The torch's tint and gain live only in the shader**; nothing in the core
  holds them.

A city needs more than a torch on the ground: lanterns hung on walls and
posts, braziers, candles behind windows, and street lights that come on at
dusk. It also needs the lit requirements to be recorded, since none of the
built lighting has reached `openspec/specs/`.

## What Changes

- **Finish `night-and-lamps` and `voxel-light`.**
  - Sync their built, tested requirements into a new `planet/light` main spec.
  - Move their unfinished items here: the sampler for moving things, the
    glowing flowers, the torch icon, and the stray copy of the sun.
  - Archive both. Their remaining "held" items (coloured light, an incremental
    relight, light on the clutter) come here too, as non-goals with their
    reasons.
- **Moving things take the field's light.** A point sampler answers both
  channels anywhere in the lit tier. The player's body and held tool, the ship,
  and anything else drawn outside the baked terrain read it every frame, so a
  ship in a cave is dark and a player by a torch is lit.
- **New light sources, each a material like the torch:**
  - a lantern on a post, a lantern on a wall bracket, a hanging lantern;
  - a brazier;
  - a candle, which is small and is what lights a room behind a window.
  Each has its own emission level, its own icon and its own model, and each
  goes through the one edit, save and relight path the torch already uses.
- **Lamps that come on at dusk.** An emitter can be always lit (a torch you
  placed) or lit from dusk to dawn (a city's street lanterns). The field
  re-bakes when the clock crosses dusk or dawn, which costs one bake of about
  6 ms twice a day.
- **Glowing flowers**, as `night-and-lamps` designed them: a share of the
  flower cells, chosen once on the CPU, lit from dusk to dawn.
- **The torch's tint and gain in the core**, held against the shader by a
  test, like the light constants already are.

## Capabilities

### New Capabilities
- `planet/light`: the light field and everything that feeds or reads it. It
  covers:
  - the sky channel and the block channel;
  - which materials emit and when;
  - the sampler for things that move.

  It takes in the requirements `voxel-light` and `night-and-lamps` built, and
  adds the lantern family, dusk-lit emitters and lit moving things.

### Modified Capabilities
- None. The lighting that exists was never synced into a main spec, so its
  requirements arrive as new ones in `planet/light`.

## Impact

- **`pbd-core`:**
  - `light` gains a dusk-gated emitter class and a point sampler that
    interpolates between cells;
  - `terrain::Material` gains the lantern family, a brazier and a candle, each
    with an emission level;
  - `daylight` answers whether it is dusk-lit;
  - the torch's tint and gain move into the core.
- **`pbd-app`:**
  - the column tier samples the field for the player, the held tool and the
    ship;
  - the tier re-bakes when the clock crosses dusk or dawn;
  - models and icons for the new materials;
  - the hotbar kit gains a new versioned grant;
  - the one remaining `SUN_DIRECTION` call site is removed.
- **Shaders:** `planet_surface.wgsl` draws the new emitter geometry; the
  ship's and the player's materials read a light uniform set from the sampler.
- **Saves:** new materials are new ids in the existing edit log. An old save
  loads unchanged, and the kit grant deals the new lights once, as the torch
  grant did.
- **Specs:** `planet/light` is created with the requirements that are true
  today. `voxel-light` and `night-and-lamps` are archived.
- **Performance:** each moving thing samples the field once a frame (eight
  cell reads each), and there is one extra 6 ms bake at dusk and one at dawn.
  `tools/perf_suite.py` runs before the push that adds the per-frame sampling.
