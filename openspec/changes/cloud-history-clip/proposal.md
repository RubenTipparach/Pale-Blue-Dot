# Proposal: clip the clouds' history to what this frame shows

## Why

Two fixes for the clouds' ghosting were built at once on two machines:
`cloud-ghosting` (on `main`: a history texel is blended only when its march
saw what this frame's does) and this repository's `cloud-close-up` resolve
pass (the history clipped to the range of this frame's neighbourhood). The
merge kept `cloud-ghosting` and dropped the resolve, because both edited the
same blend. The owner, told they can be combined: "Combine your stuff, do it
on main."

They catch different faults. `cloud-ghosting`'s test is geometric: it drops a
history texel whose march was stopped by something else, or whose cloud lies
beyond what this ray can see. That is every silhouette. What it cannot catch
is a history texel that saw the same depth range and is still wrong:

- **inside and next to a cloud**, where the texel's one coverage-weighted
  depth cannot place a cloud spread along the whole ray, so reprojection lands
  it off by a little every frame and the offsets pile up as smears and
  stacked copies while the camera flies through;
- **the cloud itself changing** (drift, the flow map's phase, lighting), which
  a 94% history holds for about eleven frames whatever the depth says.

The standard answer to both is the one temporal anti-aliasing uses: hold the
history inside the range of colours this frame shows round the texel.

## What changes

The march writes this frame's cloud alone; a resolve pass, between the march
and the composite, does what the march's blend did plus one step:

1. reproject and read the previous history exactly as `cloud-ghosting` does
   (`cloud_previous`: four taps, each kept only when it saw this ray's range,
   renormalised; none kept, the texel stands alone);
2. **clip** what it reads to this frame's 3 x 3 neighbourhood: its mean plus
   or minus 1.25 standard deviations per channel, widened to include this
   texel (Salvi 2016);
3. blend at `CLOUD_BLEND` (0.06, unchanged).

`cloud-ghosting`'s test, its distances pair, its `cloud_prev_eye` and its
blend rate are all kept as they are. Only where its blend happens moves.

## Cost

One more full-screen pass at the march's resolution (0.4 of the view per
axis): nine texel loads for the neighbourhood and one more RGBA16F target.
The march it follows samples dozens of noise texels a texel.
