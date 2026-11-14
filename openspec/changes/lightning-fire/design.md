## Context

- **The atmosphere's strikes** are cell events: a unit direction at a cell's
  centre, the step it struck on and a strength 0..1. They come from charge
  in storm cells (`atmosphere/storms.rs`).
- **The flash** (`Weather::flash`) is one Vec4: the strike's point at the
  cloud base and its flickering brightness. The ground adds
  `albedo * cool white * flash.w * skylight` (`planet_surface.wgsl`): every
  open pixel brightens evenly, wherever the strike was.
- **The bolt** (`push_bolt`) is nine hashed kinks from the cloud base to
  `ground`, drawn unlit in the shower's quad mesh so it blooms.
- **Flames** already exist in towns (`TownFlame`, unlit, `FLAME_RGB`), and
  lamps light the ground through the column tier's light field, which is
  baked (`baked-light-ease`). A fire that lasts seconds cannot go through a
  bake.

## Goals / Non-Goals

**Goals:**
- A strike visibly hits something: a tree, a roof or the ground.
- The impact lights the place it hits.
- A fire that reads as fire for a few seconds, lighting its surroundings, and
  goes out under rain.
- It costs nothing between strikes.

**Non-Goals:**
- **Spreading fire, burnt trees or scorch marks.** Fire that spreads or
  leaves a mark changes the world, and that would have to be saved
  (`world-persistence`). Not asked for: "localized ... would go out in a few
  seconds".
- **Damage to the player, the ship or towns.** A later question for the
  owner, with a mockup.
- **Dry-weather fires.** Strikes only happen in storms here, and storms
  rain.

## Decisions

Every decision is a recommendation taken (ask only with screenshots); the
owner judges the shots and the video.

1. **The landing point is the tallest solid point within 24 m.** Lightning
   takes the shortest path to the ground, and that means tall things.
   - Search the column tier's loaded cells (`ColumnTier::sample`, CPU,
     authoritative) in a 24 m disc round the strike's ground point. Where
     the tier is not loaded, use the height field (`surface_height`).
   - Take the highest solid top. Ties go to the nearest the strike's centre.
   - Skip water and anything under a roof: a cell with solid above it is not
     a top.
   - The result is a deterministic function of the strike and the terrain,
     so a strike lands the same way every time it is replayed.
2. **What it hit decides the fire.** The top block's material says what
   burns:
   - wood or leaves: a tree fire at the crown, 1.5 m across;
   - grass, a plant or a flower: a ground fire, 1 m;
   - sand, stone, snow or anything else: no fire, only the impact burst.
3. **The fire's life is set by the rain.**
   `life_s = mix(dry_s, wet_s, rain)`, with `dry_s` = 12 and `wet_s` = 3,
   plus or minus 1 s hashed from the strike. It flares in the first 0.3 s,
   holds, then shrinks over its last 40%, and gives a puff of steam as it
   dies. All of these are config values in `weather.ron`.
4. **The fire is a few billboards and one light.**
   - 6 to 10 flame quads hashed off the strike, unlit and over white, in the
     same quad mesh as the bolt and the shower, so it needs no new pipeline.
   - Smoke and steam quads rise and fade.
   - At most 4 fires at once (`MAX_FIRES`); the oldest goes first.
5. **The light: from one light to five.** The terrain's lightning term
   becomes a short array of point lights (`STORM_LIGHTS` = 5): the cloud
   flash and up to four fires. Each is a position, a radius and a colour
   times intensity. The ground pass sums them with the same falloff
   `lamp_light` uses for a lantern, gated by line of sight to the sky as the
   flash is today.
   - The cloud flash keeps today's look.
   - The impact flash is the same flicker, centred on the landing point, at
     `lightning_ground_burst` (config), so the ground near the strike blazes
     and the rest of the view brightens as it does now.
   - Rain shells and the fog see the same lights (the rain pass already
     reads the flash).
   - This is a uniform, not a bake, so a light lasting seconds costs no
     rebuild.
6. **More strikes in sight.** The atmosphere's cells are coarse, so a strike
   is far more often over the horizon than in view. A strike within 3 km
   takes the landing point, and the flash reach stays 3 km. `--strike-at LAT
   LON [SECONDS]` makes a strike there (after the delay) through the same
   path, for captures. Whether the natural rate near the player needs
   raising is decided from a measurement (task 1.2), not assumed.
7. **Logs:** a `STRIKE` line for each strike within reach gives where it
   landed, what it hit, the distance from the player and the fire's life.
   A `FIRE_OUT` line comes when the fire ends. These are for the video and
   for the audio change's events.

## Risks / Trade-offs

- **The ground pass's lighting is already the frame's largest cost.** Four
  more point lights, each a distance check and a falloff, are cheap. They
  are skipped when their intensity is 0, so a clear day pays nothing.
  Measure the storm scenario before and after (perf rule).
- **A fire on a tree that is not drawn yet**, landed on from the height
  field: the fire floats where the tree will be. Use the tier only where it
  is loaded, and the height field (ground fire) elsewhere.
- **A bolt to a rooftop in a town** is not a fire (stone, clay). The burst is
  enough.

## Verification

- **Unit tests:**
  - the landing search picks a tree's crown over the ground beside it;
  - it never lands in water;
  - it is deterministic;
  - the fire's life follows the rain;
  - a stone top makes no fire.
- **Captures** from `--strike-at` (fixed step), in
  `docs/screenshots/lightning-fire/`:
  - a strike on a tree at night in rain: the bolt, the burst, then the fire
    at 1 s and 3 s, and out;
  - a ground strike by day;
  - a strike on sand: a burst, no fire.
- **A real-time video** with `obs-record` in a storm (P to force one), the
  frame graph on, and sound once `audio-and-ambience` lands.
- **The storm perf scenario**, before and after.
