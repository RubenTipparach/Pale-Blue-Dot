# Proposal: night lights from space, and fewer, dimmer glowing flowers

## Why

The owner, 2026-11-14:

1. "also city lights should be visible from space, no need to do anything
   detailed, just that you have the map of where cities are, so you could
   probably plot the lights from space on the night side"
2. "and if man made (player made) structures reach a certain number of
   lights, it could also be visible from space"
3. "1/4 the flower lights at night, and 1/4 brightness on the flower lights,
   since I dont expect to see those fom space"

This is the far half of the owner's survey answer C3 (2026-09-27, "City lights
light the far terrain at night") and of `cities-in-the-world` tasks 4.2 ("the
night points, drawn by the planet pass") and 4.2a ("the night lights light the
coarse LOD hexes under each settlement"). Today a town's lamps and windows
exist only where the town stands (within `towns::STAND_M`, 1.2 km) and its
street lamps light the ground only inside the column tier. From altitude the
night side is dark everywhere: nothing says where anyone lives.

The glowing flowers (`lamps-and-lanterns` decision 8) are one cell in 25 of
every meadow, each lit at level 6 with a bright cyan head. From the air at
night the meadows read as fields of light that compete with the towns.

## What changes

- **Far night lights.** Every site on the world's map (`WorldSites`, the
  saved list) is drawn on the night side as a warm cluster of points of light
  with a soft glow under them, sized and brightened by its kind: a walled town
  (the capital most) over a harbour over a village and the biome towns. A cave
  town shows nothing: it is under the rock. It is drawn by the planet pass
  (`planet_surface.wgsl`), as light added to the ground before the ground's
  haze, so the clouds pass draws over it and the haze dims it. It comes on
  with the sky's own night term at the site and fades out as the camera comes
  within the distance where the real town stands, so up close the real lamps
  are all there is.
- **The player's lights from space.** The player's light blocks (torches,
  lanterns, braziers, candles, from the save's edits on cells the player has
  touched) are grouped into clusters by distance. A cluster of at least 12
  lights becomes a far light of its own, sized and brightened by its count.
  The clusters are derived, never saved, and are recomputed when the edits
  change.
- **Glowing flowers: a quarter as many, a quarter as bright.** The share drops
  from 0.04 to 0.01; because the choice is a threshold on a fixed hash of the
  cell's key, the new glowers are exactly a quarter of the old ones (a subset),
  not a reshuffle. The head is drawn at a quarter of its brightness, and the
  light it puts on the field drops from level 6 to level 3 (a third of the
  flood, the nearest whole level to a quarter).

## Saved games

Nothing here is saved or names anything saved. The far lights are derived from
the saved site list and the saved edits. Which cells grow a glowing flower is a
render and light-field choice made from the cell key by `ColumnSettings`
(`column.ron`), not part of the terrain generator or any generator version's
digest (`planet_gen` digests altitudes, biomes and top blocks); the flower is
not an edit and nothing in a save refers to it. So the glow share is changed in
place, with no new generator version (design, decision 6).

## Not in this change

- No light under the far lights' haze is baked per hex (`cities-in-the-world`
  design 7's "per-hex glow value"); the soft glow is drawn per pixel instead.
- A town's real lamp count does not size its far light; its kind does.
- Frame cost is not measured: no perf sitting in this change (said in the
  write-up and the report).
