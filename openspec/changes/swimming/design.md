# Design: two things were in the way, not one

## The wall, and the floor under it

The obvious blocker was one clause in the swept ground resolution: a wet
footprint was rejected exactly as a cliff is. Removing it is not enough, and
the second half is the more interesting bug.

`SurfaceContact::radius` is computed at `PLANET_RADIUS + height.max(0)`. Over a
water cap that clamp puts the contact plane at **sea level**, so the walker's
footprint reports the sheet as its support: with the rejection gone, a walker
would simply walk out onto the top of the sea. The clamp is right for its other
two callers, which is why it is there: assisted flight must clear the water,
and rain must land on it.

So the contact answers both questions now. `radius` is the surface you fly over
and rain falls on; `floor_radius` is the solid ground, the seabed under a water
cap and the same value everywhere else. The walker's footprint takes the floor.
One record, two fields, no second sampler.

## The probe, and where it lives

`Sea` is a `SystemParam` holding the terrain and the water settings, because
answering "is the walker in water" needs both and two systems need the same
answer. It also keeps the sheet radius in one place: `PLANET_RADIUS -
depth_offset_m`, the radius the water pass DRAWS at, so the physics and the
picture cannot disagree about where the sea is.

It arrived for a second reason, which this project's rules predict: adding the
probe to `drive_walker` pushed it to eight arguments and Clippy said so. The
rule is that the allow is the smell and the struct is the fix.

## Held, not tapped

A ground jump is an impulse on an edge; a swim thrust is an acceleration while
the control is down. The walker only had the edge, and `just_pressed` cannot
express holding. `WalkingState::jump_held` is the other half, and the two are
kept separate rather than one being derived from the other, because they mean
different things at the same instant: the frame you press space in the shallows
is both an edge and a hold.

## The exit, which is where we diverge

The reference gates the swim thrust on the **eyes** and gives a player floating
at the surface the **jetpack** to climb out with, unlocked the moment the eyes
clear the water. This project has no jetpack, so gated on the eyes alone a
swimmer beside a bank would bob there for ever. The thrust is gated on the
**body** while the feet are off the bottom, which covers the last metre. It is
recorded here because it is the one number that is not the reference's.

## What a scripted capture taught us about input

Two real defects, both found only because a headless capture had to press a key:

- **Scripted keys must be written where the real ones are.** Pressed in
  `Update`, they are wiped by the next frame's input clear before
  `RunFixedMainLoop` reads them. The walker stood still and nothing said why.
- **Pointer capture follows the window's focus, and a headless window never
  reports any.** So the input path dropped capture every frame and zeroed the
  movement axes. `WalkingState::scripted` is what a capture sets to say the
  keys are coming from a script.

Both are the same shape as the tests: a test that sets the movement axes
directly passes while the real input path is broken, because the axes are
rewritten from the keyboard every frame. The tests drive keys now.

## Auto-climb out of the water (owner, 2026-11-16)

The owner: *"introduce auto climb when in water and trying to climb up a ledge
of 1 or 2 blocks"*.

### What is there today, measured off `walking.rs`

- A swimmer leaves the water only by holding jump: the swim thrust
  (`water_swim_force` 20 m/s², drag 3/s) rises at about 4.2 m/s until the body
  probe clears the surface, and the swimmer then has to be moving forward at
  the instant the feet clear the ledge. In practice it is a bob against the
  bank.
- Wading (water up to `water_wade_depth`, two layers plus 0.1 m, over the
  feet) is grounded, so auto step (`step_height`, one layer, 1.05 m) already
  walks up a ONE-layer bank out of a shallow; a two-layer bank stops the wader.
- The sweep (`sweep` in `resolve_ground`) refuses a move into a rise it cannot
  step and reports the face's normal along the ground; the slide takes the part
  of the move along the face.

### The rule

While the walker's **body is in water** (the body probe, 0.5 m over the feet,
under the surface: swimming or wading), the **forward control is held**, and
the sweep **refused the move against a face** the heading points into, the
walker climbs onto the ledge beyond the face when:

| test | value | knob |
| --- | ---: | --- |
| facing the face: heading · (into the face) | at least **0.7** (within about 45°; a hex bank's faces meet the line of the bank at 30°, so swimming along it is 0.5 and does not climb) | `water_climb_facing` |
| ledge top over the water surface | at most **2.7 m** (two layers, 2 × 1.05, plus the sea sheet's 0.5 m under its layer top and a 0.1 m skin) | `water_climb_m` |
| or, standing on the bed in wading water, ledge top over the feet | at most **2.2 m** (two layers plus 0.1, the wade depth), whether or not the top is above the surface | `water_climb_wade_m` |
| otherwise, ledge top against the surface | **at or above it** (within 0.1 m; a step still under water is taken by "Stepping up a submerged step" below) | - |
| ledge top over the feet | at most **4.5 m** (a body's length under the surface plus two layers; a swimmer deeper than that swims up first) | `water_climb_reach_m` |
| room on the ledge | the body stands there under any ceiling, and no town solid holds it | - |
| how long | **0.4 s**: the body rises to stand on the ledge over the first 60 % and moves **0.8 m** forward onto it over the rest, both eased | `water_climb_secs`, `water_climb_forward_m` |

Why two references: the brief's "2 layers above the water surface, or above
the feet when wading". Measured, the surface test is the looser of the two
for any ledge above the surface (a wader's feet are at least 0.5 m under it),
so the wade test matters in one case: a wader on the bed of a two-layer
shallow facing a bank whose top is level with the water, which the surface
test would call under water. A wader passes either test. The sea's drawn sheet sits 0.5 m under
the top of its layer (`depth_offset_m`), so a beach cap two layers over sea
level is 2.5 m over the sheet; the 0.6 m over two layers is what keeps "two
blocks" meaning two blocks at sea, in a river (whose surface is a layer top)
and in a flowing cell (whose surface sits a fraction into its layer). A cap
three layers up is 3.0 to 3.5 m over any of them and does not climb.

During the climb the motor and gravity stand down and the body is placed on the
eased path each tick (no velocity carried in, none left over); it lands
grounded. It is never a teleport: each tick moves the body at most the path's
share of 0.4 s.

What does not climb: a ledge more than the table allows (the swimmer stays in
the water or uses the swim rise, unchanged); a walker moving away from the bank
or along it (no refused move, or a heading within 60° of along the face); a
swimmer more than 4.5 m under the ledge top; a walker with the forward control
up.

### Shared with auto step

The climb follows the **auto step** player setting (`player-settings` 5):
on (the default) it climbs; off, it does not, and the swim rise is the way out
as before. Auto step is the setting that says "the walker goes up a rise
without a key"; a player who turned it off to have rises stop them asked for
that in water too. Recommendation taken (ask only with screenshots).

### Where it is

Water of every kind: the probe is `Sea::state`, whose surface is the column's
water (sea at the tided sheet, a river, a flowing cell, a poured pool) on the
tier and the height field's sea off it, so the climb is measured against the
surface the walker is actually swimming in.

### Built (2026-11-16)

As the table says, in `walking.rs`: `water_climb` decides, from the face the
sweep first refused the move at, and `resolve_ground` starts the `Climb` and
places the body on its path each tick; `drive_walker` stands the motor and
gravity down while it runs. Tests (`walking::tests`, a pool and bank built on a
generated column tier): a swimmer climbs a one- and a two-layer bank in 19 to
32 ticks at 64 Hz with no step over 0.35 m; a three-layer bank is not climbed;
swimming along a bank is not; a wader climbs a bank two layers over its feet
(one-layer shallow) and one level with the surface (two-layer shallow); auto
step off, no climb.

In the game (`--climb-out`, a capture instrument that finds a sea bank on the
column tier and holds forward): the swimmer reaches the face 0.55 s after
letting go, rises from 2.3 m under sea level to the bank's top 2 m over it
(2.5 m over the drawn sheet) in 0.23 s, and steps on over the next 0.15 s.
Shots in `docs/screenshots/water-autoclimb/`.

**Found on the way, and fixed after (`wall-snap`, 2026-11-16):** a swimmer
sinking beside a three-layer bank was put 5.5 m up it in one tick. This
write-up first blamed the footprint's frame (`up.any_orthonormal_vector()`,
which flips where `up.z` changes sign). That was wrong, and the measurements
below say why.

### Wall snap: what it was

- **Not the frame.** The footprint is a pure function of the position: a
  position the sweep accepted is read the same way the next tick, whichever
  way its frame points. The frame flip only changes the footprint's shape
  between one side of z = 0 and the other. Walkers and swimmers pressed into a
  wall across z = 0, from starts 5 cm apart, were never lifted on the old
  code. The lift seen on that circle was a pool test placing its swimmer, at
  1.2 m toward the bank, with a footprint point already inside the zig-zag.
  Placed like that it is lifted on either side of the circle.
- **The rounding.** A position on a 4.8 km planet holds about 0.5 mm in an
  `f32`. A body pressed into a wall rests with a footprint point within a
  millimetre of the wall's edge. The refused move then moves the body
  radially (`old_up * destination.length().max(floor)`): a swimmer sinking,
  a walker held at its floor. That re-rounds its direction, and every so
  often the point lands inside the wall. The refusal took the footprint's
  support there as the floor, which is the wall's top, and stood the body on
  it in one tick, at any height.
- **Measured** on the old code: two seconds pressing into a three-layer wall
  at 240 spots 14 cm apart along it, walking and swimming, on and off z = 0.
  40 runs ended on the wall: a walker 3 m up, a swimmer 4.4 to 7 m.

**The fix:** a refused move raises the body only for a footprint point
truly inside a wall, never for one that grazes it. The refusal's floor is
still the footprint's support where that is no higher than the body stands.
Where it is higher, the body is raised only to the least support of the
footprint nudged 1 cm (`SLIVER_M`) eight ways round it:
- a point a millimetre inside a wall is clear of it in at least one nudge,
  so the body stays where it stands;
- a body really in a wall (a pillar built beside it) is in it in every
  nudge, and is put on its top as it always was.

The footprint's frame is unchanged, so every footprint samples exactly where
it did. Measured after: none of the 240 runs. An earlier version, which never
raised the body at all, also took none of a denser sweep of 480. It was
narrowed to this one so that a body truly in a wall keeps today's behaviour.

Found with it: the refusal called a walker grounded only when its vertical
velocity was at most exactly zero. After a slide that velocity is ±1e-9 of
rounding, so a walker standing still against a wall could lose its footing.
The tolerance is the hold's 1e-4 now.

### The slide along a hex bank

Swimming or walking along a terrain bank and leaning into it, the walker
stopped dead at the zig-zag's first inner corner. It walked 2.8 m and then
nothing, leaning 0.3 rad, where the open ground gave 7.4 m. The slide moves
along the face's normal, which `rise_normal` takes as the support's fall over
four probes 0.25 m round the body. At a corner that is neither face's normal,
so the move along it is refused again, three times, and the tick ends with
no motion.

**The fix:** after the three slides, if the walker made under half the move,
the move is tried turned away from the wish, 10° at a time up to 80° either
way. Each try is shortened to its part along the wish (cos θ), and the first
the sweep does not refuse is taken. A grounded walker must stay grounded
taking it, so the fallback never walks anyone off a ledge. A wall met head on
refuses every try, as it refused the slide. Measured: leaning 0.3 rad, 6.5 m
in a second against 7.4 m in the open; leaning 0.6, 5.0 m. Before, 2.8 and
2.2 m.

## Stepping up a submerged step (owner, 2026-11-16)

The owner, with a sketch (side view; P the player, w water, x solid):

```
P > w w x
    w x x
    x x x x
```

*"Often times I'm swimming in 2 depth water and my feet hits the floor when
swimming to depth 1. I expect auto climb to work here too."*

The climb above only takes a ledge whose top is at or above the water's
surface. A step in the bed whose top is still under water was left to the
swimmer. A swimmer is not grounded, so auto step does not take it either:
the feet stop against the step, and the swimmer bobs there.

### The rule

Everything in the table above holds: the body in water, forward held, the
move refused against a face within about 45° of the heading, room to stand,
and following auto step. One more case climbs:

| test | value | knob |
| --- | ---: | --- |
| a step whose top is **under the water's surface** (more than 0.1 m under it) | its top at most **2.2 m** over the feet: one or two layers, the same as a wader's | `water_climb_wade_m` |

So a swimmer whose feet meet a one- or two-layer step in the bed rises onto
it in the same eased 0.4 s, and goes on wading or swimming from there. If the
next step is the bank above the water, the climb above takes it. Deep water
to the shelf to the bank, with no stop. The step's height is measured from the
feet, not the surface, because the surface is not what the feet meet. Two
layers is the most a step-up takes anywhere: auto step's one on land, and the
water climb's two.

What still does not climb: a step more than two layers over the feet (swim up
first); a step met swimming along it (the facing test); auto step off.

### Built (2026-11-16)

The rule is one more case in `water_climb`, `steps_under_water`. Measured on
the pool, where the bed or bank in each row of cells is set by `pool_rows`:

- **Before:** a swimmer in mid-water over a two-layer pool, its feet 1.3 m
  under the surface, held forward into a one-layer shelf. Its feet stopped
  at the shelf, and it sank to the bed (1.95 m under) before the wader's rule
  took it up.
- **After:** it steps onto the shelf from mid-water and carries on past the
  shelf's first row.

A swimmer floating with its feet at or over the shelf's top glides onto it,
as before. The owner's staircase (two deep, a one-deep shelf, the bank level
with the surface) ends standing on the bank both ways. It passed before as
well, by the slower way: sinking to the bed, then the wader's rule.


## Swimming up out of shallow water (owner, 2026-11-16)

The owner: *"in river, I can't swim to the surface if I start at a lower
block on a waterfall"*. Starting on the bed under a step, holding swim-up
never brings the head out.

### What it is, measured

The suspect was the surface: read from the upper reach's column or from the
sea's level. It is not. A probe on the real column tier ran a walker from the
bed of every river cell with a higher reach beside it, at three version 10
rivers (`23.70 -2.80`, `28.25 0.75`, `24.00 1.00`), holding the swim control
for five seconds. `Sea::state` reads this column's own water every time. The
walker still fails:
- **Water under 2.1 m deep:** the walker never leaves the bed. It stands
  1.76 to 1.98 m under the surface, eyes under, not grounded, at 0 m/s.
- **Deeper water** (a fall's pool, three or four layers): the walker rises
  until the water over its feet is 2.1 m, then sinks back and hangs there,
  its eyes about 0.5 m under.

**The cause is the wading jump** (`drive_walker`).
- Water up to `water_wade_depth` (two layers and 0.1 m; owner,
  2026-11-16: "change jumpable water height to 2 blocks") is waded, "eyes
  under or not", so a jump from it rises at full strength.
- The rule took ANY rise in wading water for that jump: `wading_jump =
  wading && vertical > 0`. While it holds, the water's drag and its lighter
  gravity are off.
- A swimmer holding the control is rising, so in water two layers deep the
  swim thrust (20 m/s²) pulls against full gravity (25 m/s²) and loses.
- A river is two layers deep (12.10: its bed is carved two layers under its
  water), and a source's surface is its layer's top. So a walker on a river's
  bed has its eyes 0.4 m under and is always in the trap.
- The sea hides it: its sheet is 0.5 m under its layer's top, so two layers
  of sea is 1.5 m, and the eyes are out.

### The rule

A wading jump is a jump: the rise after a jump taken from the ground in
wading water, at full strength, with land gravity and no drag, while the
walker keeps rising. It ends when the walker stops rising or lands.
- Rising any other way is swimming, with the water's drag and gravity: the
  swim thrust from the bed, from mid-water, or from deeper water into the
  wading band.
- What a jump from wading water does is unchanged, held or tapped.

It is one flag on `WalkingState`, set by the jump and cleared when the rise
ends.

### Verify

A test beside the pool tests (`pool_rows`): a pool two layers deep and a
fall's pool four layers deep beside a higher reach. A walker on the bed
holding the swim control brings its eyes over the surface within three
seconds. It fails before: the eyes stay under. The wading tests
(`a_wading_walker_climbs_a_two_layer_bank`, the jump tests) still pass.
