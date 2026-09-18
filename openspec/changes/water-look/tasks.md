# Tasks

## 1. Give the sea depth (done)

Brought forward by the swimming change: a walker who cannot submerge cannot
dive, so this stopped being a look question and started being a blocked
feature. A scripted swim walked seven hundred frames out to sea and was still
wading at two metres.

- [x] `OCEAN_RELIEF` 0.12 to **0.45**, and the relief test re-pinned and
      renamed: it asserts the sea within sight of a standing player is deeper
      than their eye, which is the property that actually matters, rather than
      a number. `LAND_RELIEF` does not move: the summits are what the owner
      asked to be climbable.
- [x] The new shelf, measured along the `shore` walk, against the old:

| out from the waterline | was | now |
| ---: | ---: | ---: |
| 10 m | -1 m | -1 m |
| 25 m | -1 m | -2 m |
| 45 m | -1 m | **-3 m** |
| 91 m | -2 m | **-5 m** |
| 181 m | -2 m | **-8 m** |
| 363 m | -3 m | **-11 m** |
| 725 m | -7 m | **-24 m** |

      Wading becomes swimming about forty metres out, which is a beach.
- [x] The `dive` preset walks out until the floor clears the requested depth,
      so a deeper shelf shortens that walk rather than lengthening it.

## 2. Night (done)

- [x] The reflection ramps to `night_sky_color` across the terminator instead of
      mirroring a daytime gradient forever, and the ambient night floor applies
      to the transmitted body and the foam but no longer to the reflection.
- [x] A `nightshore` capture preset, since the existing night view is from
      orbit and the defect is at eye level. It walks the shoreline at the
      antisolar longitude on the equator, because with a fixed sun the old
      preset's 72 N is in permanent daylight.
- [ ] The reflected sky does not vary with the direction the water looks, so
      one authored colour is right toward the terminator and too bright away
      from it. Decide whether that is worth a real sky sample.

## 3. Author for the tone mapper we have

- [ ] Record in `docs/shader-port.md` and the comparison doc that Tenebris
      applies no tone mapping and that every water value here is therefore
      authored against `TonyMcMapface` rather than ported verbatim, with the
      side-by-side table from the proposal.
- [x] Only after the depth lands, re-judge the shine knobs against a capture.
      Done against the owner's own view and a photograph of open ocean, one
      knob at a time (design, "Second round"). The shine knobs came DOWN after
      all, but were worth little; the sea's red is the body colour's, and the
      shallows' red is the sand's, so the body lost its red and the red
      absorption went from 0.60 to 0.90 per metre.

## 4. Prove it

- [x] Shore, wade, dive, coast and night captures before and after, with the
      measured sea RGB and saturation beside them, in the comparison doc
      ("The colour of the sea, against a photograph"). The sky bands are
      byte-identical in every pair.
- [ ] The owner's in-game look at the surface and underwater. A capture on a
      software rasteriser at the owner's view is the best a container can do
      and it is not the game on their monitor.

## 5. The night side and the deep (third round)

- [x] The night reflection follows the reflected ray: lit by the sky's own
      `sun_visibility` rule at the point the ray leaves the atmosphere, scaled
      by how much it faces the sun, and nothing under a black sky.
- [x] The murk is `deep_color` attenuated by the eye's depth and by the
      day/night level, in one function the cap-from-below and the composite
      both call.
- [x] Captures: a new `midnight` preset at the antisolar point (the sun is 48
      degrees north, so `nightshore` on the equator never had a black sky),
      before and after; `dive` at 4 m and 8 m; `wade` byte-identical. In the
      comparison doc, "The night side, and the deep, measured again".
- [ ] The owner's eye at night in the running game.

