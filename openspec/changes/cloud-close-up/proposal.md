# Proposal: clouds without ghosts, without columns, and fading into the haze

## Why

The owner's priorities 1 to 3 (`CLAUDE.md`), in their words:

- "theres lots of ghosting when objects are in front of it, and it looks
  messed up when I fly up close to it"; "A lot of ghosting with clouds still".
- Of a frame entering a cloud (the `--route clouds` recording, 18 s): "fix
  this weird effect that happens when you enter a cloud": stacked, scalloped
  sheets, and white columns hanging under every cloud.
- "notice how clouds in the horizon dont blend with the background?", with
  reference photos: an ocean horizon, cumulus from an airliner, a sunset over
  a deck. In the game the layer stands at the horizon as a flat grey shell
  with a hard edge.

## What was found

1. **The columns are the clouds' own shape, not the rain.** The same moment of
   the route was captured three times (`--route clouds --time 10 --capture
   --frames 1500`, with `PBD_PASS_OFF`): the columns stay with the rain pass
   off and vanish with the clouds pass off. `cloud_density` reads its noise at
   `radial*22 + height*1.7`, where `radial` is the point's DIRECTION: height
   never enters the noise as a third dimension, it only slides the same
   two-dimensional pattern along a fixed diagonal. Every lump is therefore the
   same shape at every height, drawn out into a slanted prism from base to
   top. From afar that reads as puffs; from inside the layer or just over it,
   looking down through many prisms, it reads as columns and stacked sheets.
   The fine octaves inherit the same prisms.
2. **The ghosting is the history.** Each frame's march is blended into a
   history that keeps 94% of the old (`CLOUD_BLEND = 0.06`), reprojected by
   the cloud's single coverage-weighted depth. Nothing checks that what the
   history holds is still plausible: where a hill, a tree or a ship moves in
   front of a cloud, the old cloud stays in the history and fades out over
   about sixteen frames, in the object's shape. Where the one depth is wrong
   (inside a cloud, where the cloud is spread along the whole ray) the history
   lands a little off every frame and accumulates offset copies. And the
   upsample, when all four texels around a pixel are rejected, divides their
   near-zero weights back up to a full cloud.
3. **The clouds are never hazed.** The ground and the water sheet fade with
   distance (`distance_fog`); the cloud's light is composited as if it were a
   metre away at any distance, so a cloud 20 km out at the horizon is as
   crisp and as opaque as one overhead.

## What changes

1. **True 3D shape noise**: the noise coordinate is the point itself in
   noise cells (`p * 22/base radius`), so the shape varies with height as it
   does across; the wind shear and the flow drift are unchanged.
2. **A resolve pass** between the march and the composite (the standard
   temporal anti-aliasing resolve):
   - the march writes only this frame's cloud and distances;
   - the resolve reprojects the history, clips it to the range of this
     frame's 3 x 3 neighbourhood (variance clipping), drops it where the
     previous frame's march was stopped by something nearer (disocclusion),
     and blends;
   - the composite's upsample returns no cloud when every texel is rejected.
3. **Haze on the clouds**: a cloud's contribution fades with the air between
   the eye and the cloud, integrated through the same exponential atmosphere
   the ground's haze uses, so a distant cloud dissolves into the sky or the
   hazed ground behind it, and one seen from orbit through thin air does not.
   Its strength is a validated value in `weather.ron`.

## Out of scope

- A near-field full-resolution march and finer detail up close (the rest of
  priority 2): after this lands and is judged.
- The ground-to-space fog transition (priority 4) and the sky model itself.
