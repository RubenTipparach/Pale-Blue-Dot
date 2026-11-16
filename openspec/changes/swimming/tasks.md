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
- [x] The flight-to-walk toggle no longer walks to the nearest land first.
      That was the only sane thing while the sea was a wall; with a swimmer it
      snapped a pilot over the ocean to a shore they were nowhere near. The
      walker now arrives under the ship, floating at the sheet and not grounded
      (`toggling_to_walk_over_the_sea_lands_in_the_water_not_on_a_shore`).
- [x] **F leaves flight where the ship is, and the walker FALLS.** The owner's
      words: "change F to toggle off fly, therefore I must fall to the ground
      instead of snapping to it." The toggle used to find the ground under
      the ship and stand the walker on it; now the walker starts with its eye
      where the ship's was, carrying the ship's velocity, not grounded, and
      gravity and the ground contact take it from there, into the sea if
      that is what is under it (`drop_walker`;
      `toggling_to_walk_in_the_air_falls_rather_than_snapping` and the sea
      test, which now falls thirty metres into the water). Tenebris's own
      `toggle_fly` does the same: fly off is the same body with gravity back.
      The spawn and the reset key keep the ground rule, since a new world
      has no ship to fall from.
- [x] Found on the way: the `--swim` script was registered unconditionally and
      asked for `WalkingState`, which a plain photo never has, so every
      `--capture --view <preset>` run since the swim landed panicked on its
      first frame. It is registered beside the walker it drives now.
- [ ] Ask the owner to swim in the running game. The feel of water is exactly
      the sort of thing a green test cannot settle.

## 4. Auto-climb out of the water (owner, 2026-11-16)

"introduce auto climb when in water and trying to climb up a ledge of 1 or 2
blocks". The rule and its numbers are in the design, "Auto-climb out of the
water".

- [x] The knobs in `WalkingConfig`: `water_climb_m`, `water_climb_wade_m`,
      `water_climb_reach_m`, `water_climb_facing`, `water_climb_secs`,
      `water_climb_forward_m`.
- [x] The trigger in `resolve_ground` on a refused move, and the eased climb,
      with the motor and gravity standing down while it runs. Off with auto
      step.
- [x] Tests on the test planet: a swimmer facing a 1-layer bank climbs out;
      so does a 2-layer bank; a 3-layer bank does not; moving along the bank
      does not climb; a wader at a 2-layer bank climbs.
- [x] Captures, a short sequence of swimming up to a bank and climbing out, in
      `docs/screenshots/water-autoclimb/` (`--climb-out --time 21`).
- [x] The lift up a wall (design, "Wall snap: what it was"), `wall-snap`.
      Measured, it was not the footprint's flip on z = 0, which this task
      named. It was the rounding of a position pressed against a wall. On
      the old code, 40 of 240 runs of a walker or a swimmer pressed into a
      three-layer wall were stood on its top, on and off that circle. A
      refused move now raises the body only for a footprint point truly
      inside a wall, never for one that grazes it (`support_past_sliver`).
      The frame is unchanged. Tests:
      `a_body_pressed_against_a_tall_wall_is_never_stood_on_its_top` (fails
      before, 40 of 240) and
      `walking_a_full_circle_round_z_0_beside_a_wall_never_snaps_up` (fails
      before).
- [x] The slide along a hex bank stopped at the zig-zag's first corner. The
      move is now tried turned up to 80° from the wish when the slides make
      under half of it. Test:
      `a_walker_leaning_into_a_hex_bank_slides_along_its_zig_zag` (before:
      2.8 m against 7.4 m in the open; after: 6.5 m).
- [x] Found with it: a walker standing still against a wall lost its footing
      to the rounding of its vertical velocity. The refusal's grounded test
      takes the hold's 1e-4 now.
- [ ] The owner swims up to a bank in the running game.

## 5. Stepping up a submerged step (owner, 2026-11-16)

"Often times I'm swimming in 2 depth water and my feet hits the floor when
swimming to depth 1. I expect auto climb to work here too." The rule is in
the design, "Stepping up a submerged step".

- [x] `water_climb` takes a step whose top is under the surface, up to two
      layers over the feet (`water_climb_wade_m`), with the same gates
      (`steps_under_water`).
- [x] Tests on the pool (`pool_rows`):
      `a_swimmer_steps_from_two_layer_water_onto_a_one_layer_shelf` (fails
      before: the swimmer sinks to the bed first);
      `a_swimmer_climbs_the_owners_staircase_onto_the_bank`;
      `a_swimmer_moving_along_a_submerged_step_does_not_climb`.
- [ ] The owner swims the staircase in the running game.

## 6. Swimming up out of shallow water (owner, 2026-11-16)

"in river, I can't swim to the surface if I start at a lower block on a
waterfall". The finding and the rule are in the design, "Swimming up out of
shallow water".

- [ ] 6.1 The wading jump is only a jump: a flag on `WalkingState` set by a
      jump taken in wading water, cleared when the rise ends; a held swim
      in wading water takes the water's drag and gravity.
- [ ] 6.2 A test on the pool (`pool_rows`): a walker on the bed of two- and
      four-layer water beside a higher reach, holding the swim control,
      brings its eyes over the surface (fails before).
- [ ] 6.3 The owner swims up from a river's bed in the running game.
