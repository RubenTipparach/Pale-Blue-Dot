# Tasks

## 1. Give the sea depth

- [ ] Raise `OCEAN_RELIEF` from 0.12 so the shelf falls away within sight of a
      standing player, and re-pin
      `relief_is_cut_to_climbable_summits_and_a_shallow_ocean_floor` to the new
      floor. The land's `LAND_RELIEF` does not move: the summits are what the
      owner asked to be climbable.
- [ ] Measure the new shelf profile out from the `shore` walk, the way the
      current one was measured, and put the table in the comparison doc.
- [ ] Re-check the `dive` preset against the new profile: it walks out until the
      floor clears the requested depth, so a deeper shelf should shorten that
      walk rather than lengthen it.

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
- [ ] Only after the depth lands, re-judge the shine knobs against a capture.
      They are already below the reference's on every axis; the expectation is
      that they come back UP, not down.

## 4. Prove it

- [ ] Shore, wade and coast captures before and after, with the measured sea
      and sky RGB and saturation beside them, since "shiny" is a look and the
      numbers are what make a look arguable.
