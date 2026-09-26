# Proposal: clouds without a speckled rim, and without salt and streaks on entry

## Why

The owner, on the merged build's recordings (2026-09-25), with screenshots:

- "bad outline in high altitudes on clouds near horizons (viewing
  underside)": from high up, a cloud near the limb is a flat grey shape ringed
  with bright, stair-stepped speckle (far-side recording, 45-55 s; a capture
  of `--route far-side --time 10 --frames 3000` shows it).
- "lots of streaks still when we first enter a cloud": at the moment of entry
  the frame fills with white salt, and curtains drag down the screen
  (cloud-hop recording, 18 s and 28-29 s).

Priorities 3 and 4 in `CLAUDE.md`.

## What was found

Both come from the clouds' half-resolution history (`water.wgsl`), read off
the code and the frames.

1. **A texel whose cloud comes from its history is never hazed.** The march
   writes the texel's cloud distance only when THIS frame's march found cloud
   (`depth.x = select(0, marched.depth, marched.cloud.w > 1e-3)`), but the
   texel's colour is the blended history. At a cloud's fringe the coverage of
   the jittered steps flickers, so many edge texels hold most of a cloud with
   a distance of zero. The composite measures the haze only to texels with a
   distance, falls back to zero when none has one, and `cloud_air(0)` is no
   haze at all. From 1.5-3 km up the haze leaves 11-22% of a limb cloud, so
   the unhazed edge texels are 5-9 times brighter than the hazed inside: the
   bright rim, stair-stepped at the march texel. A texel with no distance
   also passes the silhouette test's near side unconditionally, which keeps
   exactly those texels along the limb's depth break.
2. **Entering, the history is thrown away over the ground.** The history is
   read where this frame's cloud point was on the previous screen, and kept
   only if the previous march reached as far as THIS ray's ground. With the
   cloud 30 m away and the ground 250 m, the parallax between the two moves
   the ground by more than the test's 3% + 10 m, so the history is rejected
   over ground and sea (the sky, capped at 60 km, passes). A rejected texel
   shows one raw, white-noise-jittered march sample: the salt.
3. **The streaks are the history dragged.** A texel with no cloud this frame
   is read from the history at the middle of the march's span, hundreds of
   metres from where the history's cloud was, and keeps 94% of what it reads.
   Each frame pulls the cloud a little further along the screen's motion:
   curtains below the point the camera flies toward.

## What changes

1. **Every texel that holds cloud keeps a distance.** Where this frame's march
   found none, the texel's distance is where the history's cloud is; and the
   composite, where no texel round a pixel has one, measures the haze to this
   pixel's own entry into the cloud layer, never to zero.
2. **The history is kept if the previous march reached the cloud**, not this
   ray's ground: an occluder in front of the cloud still rejects it, and
   parallax against the ground far behind no longer does.
3. **A texel with no cloud this frame reads its history at the history's own
   cloud distance** (looked up where the span's middle falls, then read again
   there), so what it holds fades in place instead of being dragged.

## Out of scope

- The march's raw noise itself (white-noise jitter; the finest detail octave
  kept above what a 12 m step can sample) and a resolve pass with a
  neighbourhood clamp (`cloud-close-up` design section 2, dropped in the
  merge): taken up if salt or streaks remain after this.
- A near-field full-resolution march (priority 3's up-close half).
