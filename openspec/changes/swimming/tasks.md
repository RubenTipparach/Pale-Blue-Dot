# Tasks

## 1. The probes

- [x] One helper answering the three states off the rendered sheet radius
      (`PLANET_RADIUS - depth_offset_m`, the same radius the water pass draws
      at), called by both the drive and the ground-resolution system so the
      two can never disagree about whether the player is in water.
- [x] Probe heights: the feet, the body at +0.50 m and the eyes at
      +`EYE_HEIGHT`, as the reference does.

## 2. The model

- [x] Remove the water rejection from `resolve_ground`, and with it the other
      half nobody had noticed: the contact reported SEA LEVEL as the walkable
      radius over a water cap, so even with the rejection gone a walker stood
      on the sheet as if it were a floor. `SurfaceContact::floor_radius` is the
      solid ground, the seabed under water; `radius` stays the surface an
      aircraft clears and the rain lands on.
- [x] `grounded` is false whenever the eyes are under.
- [x] Speed, gravity and vertical drag multipliers on the body probe; the
      continuous swim thrust and the weakened seabed jump on the jump control.
- [x] The knobs live beside the walker's existing ones in `WalkingConfig`, with
      their units, and the defaults are the reference's.

## 3. Prove it

- [x] A test that walks from land into the sea and asserts the walker crosses
      the waterline rather than stopping at it, driven through the real key
      path: the input is rewritten from the keyboard every frame, so a test
      that sets the movement axes directly tests nothing.
- [x] A test for the terminal sink and rise speeds against the closed form,
      and that a submerged walker is never grounded. Measured: sinking settles
      at -2.56 m/s against the predicted -2.5.
- [x] The seabed stays walkable while wading: the existing terrace and ground tests still pass, and the scripted swim wades before it swims.
- [x] Captures, through a `--swim` script that places the walker at the
      shoreline and holds forward, since a walker with no input never moves.
      Two things had to be fixed before it photographed anything: scripted keys
      pressed in `Update` are wiped by the next frame's input clear before
      `RunFixedMainLoop` reads them, and pointer capture follows the window's
      focus, which a headless window never reports.
- [ ] Ask the owner to swim in the running game. The feel of water is exactly
      the sort of thing a green test cannot settle.
