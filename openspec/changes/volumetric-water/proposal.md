# Proposal: water is where the column says it is

## Why

**The owner's words: "water height fog should not apply in cave area, we
need some understanding of volumetric water"**, with a picture of a dry cave
whose floor is tinted sea-blue, and the Minecraft water article as the model
of what water as a BLOCK means.

The columns already know. `column::generate_solid` fills every layer above
the ground and under sea level with `Material::Water`, and every layer above
the ground on land with `Material::Air`, so a cave carved under land below
sea level is air in the data, which is right: a sealed pocket is dry. What
tints it is three consumers that never ask the column and read the RADIUS
instead:

- the terrain fragment's submerged term, `water_depth = sea_radius -
  length(position)`, which fogs every fragment under sea level whether water
  stands over it or not;
- the camera's `submersion`, which is the eye's radius against the sea
  radius, so a walker in that cave is drawn underwater;
- the walker's `water_depth`, which is the heightfield's `-height`, so the
  same walker swims in air.

`water-composite`'s design saw this coming and wrote it down: "A cave below
sea level cannot exist in a heightfield, so the 'caves stay dry' clause of
Tenebris's rule is satisfied by construction and is noted here so it is not
lost when the voxel engine arrives." The worms arrived, and it was lost.

There is also the opposite hole. A worm carving under the seabed turns
`Water` layers into `Air`, so a tunnel that breaks into the ocean is a dry
tube with the sea standing over its mouth. And on the GPU, `render_code`
gives `Water` and `Air` the same code, nought, so the material buffer the
faces already read cannot tell them apart at all.

## What

Three phases, and only the first is this change's work:

1. **Water is a material the renderer and the walker can see.** Water gets
   its own render code, and every consumer that asks "is this under water"
   asks the column in the tier and the heightfield only outside it: the
   fragment's submerged term, the camera's submersion, the walker's water
   depth. A dry cave under sea level is dry on screen, underfoot and in the
   composite; the sea is unchanged, because over the sea the column's answer
   IS the sea.
2. **Water floods by connectivity.** At tier build, air that a carve opened
   onto water fills to the water's level (a bounded flood within the tier
   from every `Water` layer into air that touches it, at or below that
   level); a sealed pocket stays dry. An edit that opens a pocket to water
   floods it the same way, on the edit. This is the aquifer-less half of
   Minecraft's rule: oceans are sources and water fills what is open to it.
3. **Flow and levels**, Minecraft's own: source and flowing water, eight
   levels falling one per block, seven blocks of spread on the flat and
   unbounded downward, a source made where two sources meet over solid, the
   surface drawn lower for lower levels. That is the field `water-flow`
   left empty and the fluid state `voxel-engine-foundation` reserves; it
   is named here so the shape of phase 1 does not fight it.

## Measured before anything moves

How much of the spawn tier the bug covers: the columns whose ground stands
above sea level yet carry carved air below it (the dry-cave case in the
picture), the columns under the sea whose carve turned water into air (the
dry-tube case), and the layers of each. An ignored test measures it off the
real tier at the default spawn; the design carries the numbers.
