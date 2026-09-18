# Tasks

## 1. The probes

- [ ] One helper answering the three states off the rendered sheet radius
      (`PLANET_RADIUS - depth_offset_m`, the same radius the water pass draws
      at), called by both the drive and the ground-resolution system so the
      two can never disagree about whether the player is in water.
- [ ] Probe heights: the feet, the body at +0.50 m and the eyes at
      +`EYE_HEIGHT`, as the reference does.

## 2. The model

- [ ] Remove the water rejection from `resolve_ground`.
- [ ] `grounded` is false whenever the eyes are under.
- [ ] Speed, gravity and vertical drag multipliers on the body probe; the
      continuous swim thrust and the weakened seabed jump on the jump control.
- [ ] The knobs live beside the walker's existing ones in `WalkingConfig`, with
      their units, and the defaults are the reference's.

## 3. Prove it

- [ ] A test that walks from land into the sea and asserts the walker crosses
      the waterline rather than stopping at it.
- [ ] A test for the terminal sink and rise speeds against the closed form,
      and one that a submerged walker is never grounded.
- [ ] A test that the seabed is still walkable while wading.
- [ ] Captures: standing in the shallows, swimming at the surface, and under
      it, which is also the first time the composite pass's underwater path is
      reachable by playing rather than by a capture preset.
- [ ] Ask the owner to swim in the running game. The feel of water is exactly
      the sort of thing a green test cannot settle.
