# The chest and the workbench as models

## Why

The owner, 2026-11-26: "chest needs a custom model", then "the work bench
could use a better model too". Both are terrain cells, so today each is
drawn as a whole hex of the local trees' wood (2.8 m across, 1 m tall), with
an iron strap or a vise stood on top (`storage-chests` design 7,
`tool-crafting` decision 5). The material word has no room for a tile of
their own, and a hex of bark is not a chest or a bench.

## What changes

- **The terrain stops drawing them.** A cell holding a chest or a workbench
  is still solid: it is walked on, it blocks light, it is aimed at, dug and
  saved exactly as today. Only the terrain's own draw skips it, as air.
- **A model stands in the cell instead**, built as the towns' furniture is
  (`furniture` design 2): bevelled forms, smooth normals, the town atlas's
  plank, timber, boards and iron textures sampled nearest, lit by the field
  and the terrain's sun.
  - **The chest** is the towns' chest (`furniture::Kind::Chest`): a boarded
    body with its painted bands and lock, and a crowned lid.
  - **The workbench** is a new piece: a thick plank top on four legs that
    taper into it, a low stretcher frame with a boards shelf, a wooden vise
    at one end with an iron screw, and a mallet and a chisel on the top.
- **Nothing a player made changes.** The materials, their codes, the saves
  and every rule are the same; this is how two blocks are drawn.

## Not in this change

The other stations (forge, carpenter's bench, drafting table, grinder,
mixer) keep their wood hex until asked. A dropped chest or workbench item
floating in the world keeps its block look.
