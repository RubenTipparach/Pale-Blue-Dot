# Proposal: the blocks audited, and the cave without a sky in it

## Why

Four things the owner saw in one session of digging, and one they remembered
from Tenebris:

1. **"Why is the top of the snow surface rock?"** Because it is: the snow cap
   samples tile `(3,0)` of the tundra sheet, which that sheet names "cold
   granite". Sand does the same - beach, desert and seabed all sample `(3,2)`,
   "packed path" on every sheet. The grass fix made caps draw their tile's
   colours, and the moment they did, every code whose tile had only ever
   contributed brightness showed what it was actually pointing at.
2. **A blue haze in the caves.** The terrain shader's distance haze and limb
   rim are not gated by the voxel field, so a wall forty metres underground
   fogs toward the sky colour exactly as a ridge at the same distance does.
   Tenebris gates its rim by `v_sky_light`.
3. **Water droplets on the lens run UP.** The composite's drop math is
   Tenebris's, whose full-screen `v_uv` is y-up; Bevy's `FullscreenVertexOutput`
   is y-down, so the same "monotonic downward fall" climbs.
4. **A dithered face beside placed stone.** Not reproduced headless - the
   scripted place draws clean - so the design records the diagnosis and what
   would settle it rather than a fix.
5. **The corner dimming "when a block is on top of or below another".** It is
   here, and it is the reference's: the pit floor's corners measure 63 of 255
   against 110 at the centre. What is NOT here, and is not in Tenebris either,
   is the vertical half of that idea - a wall's foot where it meets a floor is
   exactly as bright as its middle (92 against 92, measured), and a wall's top
   under an overhang is too. That is the picture the owner drew.

## What

- **Every cap draws the sheet's own ground, and the audit proves it.**
  `tools/block_audit.py` cuts, for every material code and every sheet, the
  tile the cap samples and the tile the side samples, straight from the atlas
  and the shader's own table, labelled with the sheet's name for that tile. A
  test reads the same table and the same names and holds snow to a snow tile,
  sand to a sand tile, stone to `#3`, grass to `#0`.
- **The haze and the rim take the field.** Multiplied by the same `skylight`
  every other sky term already takes; a cave at sky 0 has no atmosphere in it.
- **The droplets fall.** The lens works in the reference's y-up frame.
- **A wall darkens at its foot and under a lid.** The corner ladder gains the
  two cells beyond the wall's edge - across and beside, one layer past the
  edge - which is Minecraft's third-neighbour rule adapted to a lattice where
  three columns meet at a corner. One rule, in the core and transcribed to the
  shader, with a fourth rung on the ladder.

## What this is NOT

- Not a change to the sky, the sun or the clock: that is `planet-spin`.
- Not new art. Every tile it points at is already in the sheets.
