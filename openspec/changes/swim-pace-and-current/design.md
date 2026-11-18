## Context

`walking::drive_walker` writes the walker's planar velocity outright each
tick from the input: `speed` is `walk_speed` (8 m/s) or `sprint_speed`
(14 m/s, times the meal's stride), times `water_movement_mult` (0.5) with
the body in water. `swimming` is `(eyes under && !wading) || (body in water
&& !grounded)`. `GroundState::drift`, a way along the ground added to the
planar velocity, was the moving deck's (retired with the cog,
`retire-rowboat-and-cog`) and is free.

The water's velocity is already worked out for boats:
`vehicles::fresh_water(contact, point)` reads the column tier's water
levels (the water rule's state, `PlanetContact::column_water`) round a
point, and gives a river's or lake's surface current, downhill along its
slope at 1.6 m/s per 0.12 of slope, capped at 2 m/s, in the planet's
body-local frame. At sea a boat takes the weather sample's ocean surface
current (`air.now.sample(direction).current`). This is the sim's, not the
shader's flow lane (the UV moved to a north/east frame in #115; nothing here
reads it).

## Decisions

### 1. Swim pace

`swim_speed_mult` = 0.5 multiplies the speed when `swimming`, after the
water's own `water_movement_mult`: a swimmer goes 2 m/s (sprinting 3.5 m/s),
each times the meal's stride as before. A wader (feet down, head out) keeps
4 m/s. The number is in `assets/config/swimming.ron` (`SwimSettings`), with
its unit, validated in 0 < m <= 1.

### 2. The push

While the body is in the water and its feet are off the bottom
(`GroundState::feet_down` false, the sweep's own contact, before the
submerged rule clears `grounded`), the target drift is

    target = clamp(current_share * flow, current_max_mps)

along the ground (the up component removed), with `current_share` 0.5 and
`current_max_mps` 1.0. Otherwise the target is zero. The drift eases toward
the target at `current_rate_per_s` 1.5 (time constant 0.67 s):

    drift += (target - drift) * (1 - exp(-rate * dt))

and is added to the planar velocity as today's drift is. So:
- a swimmer floating with no input drifts downstream at up to 1 m/s (half a
  2 m/s river); swimming upstream against the fastest river it still
  makes 2 - 1 = 1 m/s over the bed at the meal-free stride;
- the push grows from zero when the walker enters the water and fades when
  it leaves or puts its feet down, over the same time constant: no jolt;
- `fresh_water` samples the surface's slope 1.5 m either side, so the flow
  changes smoothly across cells, and the ease smooths what is left.

All in the floating origin: the probe is asked at the walker's planet point
(`SimOrigin::query_point`, as the walker's own water test is), and the
current is a direction with a speed, the same in the local frame. Nothing
is rounded at the planet's scale.

- *Recommendation taken (ask only with screenshots):* the numbers above.
  The owner asked for "a small push"; half the flow, capped at 1 m/s, is
  felt in a fast river and never stronger than a swimmer.

### 3. Feet down

The resolve step records `feet_down`, whether the sweep found a floor under
the feet this tick, before the submerged rule. A walker standing on a
river's bed with its eyes under is not grounded for the jump, but its feet
are down, so the current does not push it.

## Found: half the pace breaks two earlier swim rules' tests

Measured with the unit tests (2026-11-18), the pace alone (the current off
gives the same): a slower swimmer holding forward sinks further before it
reaches a bank or a shelf, so
- `walking::tests::a_swimmer_steps_from_two_layer_water_onto_a_one_layer_shelf`
  (`swimming` 5, the owner: "my feet hits the floor when swimming to depth
  1"): starting 1.5 m under, 0.5 m from the shelf, the feet now touch the
  bed (1.95 m under) before the step up;
- `walking::tests::a_swimmer_climbs_out_onto_a_one_or_two_layer_bank`
  (`swimming` 4): the swimmer reaches the bank deeper, so the 0.4 s climb
  lifts it further, 0.40 m in a tick against the test's 0.35 m.
At `swim_speed_mult` 0.75 the shelf case passes and the climb still steps
0.36 m; at 1.0 both pass. Both are the owner's earlier asks against this
one, so the pace is shipped as asked (0.5) and the two tests left failing
for the owner's call. Ways out: make the climb's time grow with its height
(a step capped per tick), or let a swimmer pressing forward sink slower.

## Risks / Trade-offs

- **Ocean currents move a swimmer at sea** too, at half their speed. Most
  are a few tenths of a metre a second. A swimmer is never carried faster
  than it swims.
- **Halving the swim slows every swim mission's metres**, which count
  distance, not time: they stay possible.
