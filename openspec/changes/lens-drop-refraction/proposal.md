# Proposal: drops on the lens bend only what is behind them

## Why

The owner (2026-11-13), standing on a beach in heavy rain at map 12.948S
8.135E: "something seems off about these rain drops, almost like they are
flipped upside down", and "or perhaps the refraction index is not calibrated
correctly, since I see lots of black drops on top". Drops on the lens over the
grey sky were dark blobs.

## What was found

The lens (`water.wgsl` `lens`, `refract_through_drops`, the Heartfelt drops
Tenebris ported) samples the scene at `uv - n * rain_lens_refract /
rain_lens_scale`, with `n` the raw difference of the drop coverage across
0.001 of drop space. Three things are wrong with that offset, measured in
`design.md`:

1. **It reaches far outside the drop.** `n` is a difference, not a slope,
   and nothing ties it to the drop's size. At 1440x900 a drop is 6 to 27 px
   across, and the median drop pixel samples the scene **70 px** away (p90
   220 px, p99 450 px, up to 1224 px). **Two thirds** of the pixels in drops
   sample a point that no drop covers. A drop over the sky near the
   treeline shows the trees, and that is the black drop.
2. **It points away from the drop's centre.** The top of a drop shows what
   is above it and the bottom what is below. On the sky that gives a dark
   crescent under each drop, and on the trees a bright crescent over it.
   A real drop on glass is a convex lens and does the opposite: what is
   above shows at its bottom, so a drop against a bright sky is bright
   underneath. This is the "flipped" look. BigWings' original adds the
   normal; Tenebris's port subtracts it, and we carried that over.
3. **It is stretched sideways.** The slope is per screen height, but its x
   part is applied as a share of the screen width, so sideways offsets are
   1.6 times the vertical ones (the aspect ratio).

## What changes

1. **Toward the centre, within the drop.** The offset becomes the slope of a
   lens height built from the drop field, each kind of drop weighted by the
   square of its own radius:
   - the static drops (their height over `l0`'s);
   - a layer's drops;
   - a layer's trail beads, now kept apart from the drops.

   Then every drop, small or large, bends a pixel near its centre by the
   same share of the pixel's distance from that centre, and nothing at its
   rim. The offset points toward the centre and is carried to the screen
   with the aspect ratio. A cap holds it within one drop's radius, because
   the seams between drop cells step the field.
2. **`rain_lens_bend`** replaces `rain_lens_refract` (Tenebris's
   unnormalised 1.7). It is that share at a drop's centre, validated to
   0..=2:
   - below 1, a drop is an upright magnifier of what is behind it;
   - from 1 to 2, it inverts within its own outline;
   - at 2 or less, no sample ever leaves the drop.

   The default is 0.5, which magnifies a drop's middle twice over. About 1%
   of the pixels that move are inverted, where drops overlap.
3. **A drop's edge and the light under it.** A drop that shows only what is
   behind it vanishes against a flat sky, and the old lens's drops read as
   water over the ground only through the far-off sky they sampled. Instead:
   - the edge, where the drop's alpha passes a half, loses
     `rain_lens_rim` (0.2) of its light;
   - the lower part of that edge gathers `rain_lens_glint` (1.0) of the
     sky's mean radiance, the light a drop on glass collects underneath.

   Over the sky a drop is a faint ring of sky, darker above and lighter
   below. Over dark ground it is a magnified patch with a light crescent
   under it.

## Out of scope

The drop field itself is unchanged: where drops appear, their size and
count, their trails, the mist they wipe, and the emerge drips (which use the
same refraction and are corrected with it).
