# Tasks

Two of the five pieces are built and pushed; three are not. The split is stated
here rather than in prose, because this change is the one place a reader can see
which is which.

## 1. A day and a night - DONE
- [x] `pbd_core::daylight::Clock`: a day in seconds, a start hour, an axial
      tilt, and a direction from a time.
- [x] `sky::Sun` as a resource, extracted to the render world, with the arc
      built about the old `SUN_DIRECTION` so noon is where noon was.
- [x] Every consumer reads it: the sky, the terrain, the water, the clutter.
      The `const` and its six normalising call sites are gone.
- [x] `--time <hour>` pins and stops the clock for a capture.

## 2. The block channel - DONE
- [x] `Light` carries two nibbles, sky and block, and `flood` is shared by
      both channels rather than written twice.
- [x] `emitters()` derives the emitter list from the columns rather than
      keeping a second list beside them.
- [x] Tests: a lamp lights a dark place, falls off by one a cell, does not
      light through a wall, and is dropped when it is inside rock.
- [x] The shader adds the lamp term over the sky term, with the warm tint and
      the gain named as constants a test holds against the core's.

## 3. Torches - DONE
- [x] A save made before torches joined the kit has none, and never would:
      the hotbar rides the edit log and the kit is dealt only to a new world.
      The kit is a VERSIONED GRANT now (`slots::KIT_VERSION`, one table of
      what each version added), the log records the version dealt as a
      `kit` line beside the hotbar, and a save opened behind the version is
      dealt what it missed once, on the frame it opens, through the same
      durable writer an edit uses. Tests: an old hotbar reopened gains the
      torches once; a new world's kit is the base plus every grant.
- [x] A `Material`, so the hotbar, `aim`, the edit path, the save and the
      relight all carry it with no second path.
- [x] The column record carries the torch layer, so the shader knows where to
      draw the geometry.
- [x] A geometry band in the vertex shader for the torch itself.
- [x] Non-solid to the walker and transparent to light.
- [ ] A real icon. What ships today is a tinted wood tile, which is a
      stand-in, and this repository's rule says an item gets its art in the
      change that adds it. This is the outstanding half of a done piece.

## 4. Bioluminescent flowers - NOT BUILT
- [ ] A glowing species: a share of the flower cells, chosen on the CPU, with
      a bit in the cell record so the shader draws the glowing head exactly
      where the bake lit one.
- [ ] Fed into the emitter list, so the field carries them.
- [ ] Their emission follows the clock, because a flower that glows at noon is
      a flower nobody can see glowing.
- [ ] A capture at night over a meadow, and the same meadow by day as the
      control.

## 5. The sampler for what moves - NOT BUILT
- [ ] `light_at(point)` on the tier, public, answering both channels.
- [ ] Applied to the ship, which is the only moving drawn thing there is.
- [ ] Tests: a point in a dark cave reads dark, a point by a torch reads lit.
- [ ] Named honestly in the proposal: there is no dropped item and no animal
      yet, so this ships the function and one consumer.

## 6. Held
- [ ] Coloured light. A torch is warm by the shader's tint over one intensity
      channel; RGB is three nibbles and a widening of the field.
- [ ] A moon, moving stars, seasons.
- [ ] Fuel, burning out, fire spread.
