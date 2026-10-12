# Proposal: smaller rain drops on the lens, and condensation in cloud

## Why

The owner (2026-09-25): "We'll need to add condensation to the lenses as
well" (flying into clouds), and "make the rain drops smaller. They are kinda
big on the lens right now."

## What was found

1. **The drops are Tenebris's size, and big at this resolution.** The lens
   (`water.wgsl` `lens`, the Heartfelt-style rain on glass Tenebris ported)
   takes its knobs from `weather.ron` (`rain_lens_size` 0.7,
   `rain_lens_density` 1.4, exactly Tenebris's `weather.yaml`), in drop-space
   units of the screen height. At 1440x900 a big falling drop is about 55 px
   across (86 px of support), a small one about 30 px, a static drop 12-15 px.
   Raising `rain_lens_size` alone shrinks the drops but not their trail beads,
   whose height follows the layer, and the refraction offset is an
   unnormalised finite difference that reaches 150-290 px: several drop radii,
   which is why each drop reads as a ring.
2. **There is no condensation, and Tenebris removed it.** Its composite says
   "refraction only - NO lens fog"; our port still computes Heartfelt's trail
   channel (the clear tracks drops cut through misted glass) and throws it
   away. A mist brings that half back.
3. **Whether the eye is in cloud is the GPU's to say.** The CPU has the
   weather maps and the layer's heights, but the cloud's shape (the noise, the
   flow, the cells, the erosion) exists only in `cloud_density`; on the
   cloud-hop route the cover is 0.3-0.75 in separate puffs, so a mist driven
   by the map alone would fog in the clear air between them, and porting the
   noise to Rust would be two hand-written copies of one rule.

## What changes

1. **`rain_lens_scale`** (unitless, 1 = Tenebris, default 2): the lens's drop
   space is scaled by it, and the refraction divided by it, so drops, beads,
   trails and the static drops shrink together and each drop still bends the
   same share of its own width. About 27 / 15 / 6 px at 1440x900.
2. **Lens mist**: a one-pixel probe pass samples the one `cloud_density` at
   the eye each frame and integrates a mist amount in a 1x1 pair (fogging over
   `lens_mist_fog_s`, clearing over `lens_mist_clear_s`, both faster with
   airspeed). The lens draws it as fine beads over a slightly blurred,
   lifted image, clearing from the edges inward in patches; rain drops and
   their trails cut clear tracks through it. None under water.

## Out of scope

Frost for snow cloud; wipers.
