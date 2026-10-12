# Design

## What Tenebris actually does

Two functions in `planet.rs`, and the split is by job rather than by
approximation quality.

**`gravity_at(world_pos, full_default, falloff_default) -> GravityQuery`**
walks every body and keeps the strongest pull:

```text
full = radius * gravity_full_mult      (default 1.4)
fall = radius * gravity_falloff_mult   (default 1.8, guarded to exceed full)

dist <= full          -> multiplier 1
full < dist < fall    -> multiplier 1 - (dist - full) / (fall - full)
dist >= fall          -> this body contributes nothing
```

It returns the winning body, the multiplier and the altitude. No body winning
is `is_in_space`. Tenebris uses zero as an inheritance sentinel; this port uses
explicit optional overrides instead, as required by `CLAUDE.md`. A zero inner
multiplier is valid and starts the taper at the centre. An outer multiplier
must be finite and strictly larger than the inner multiplier.

Its comment records why strongest-pull rather than nearest: Crag's well overlaps
Quartz's, and the moon you are hovering above should win even when the parent
planet's centre is nearer. The multiplier is monotonic in `radius - distance`
within the band, so the largest multiplier is the right tie-break.

**`orbital_gravity_at(world_pos) -> DVec3`** sums, over every body:

```text
a = SURFACE_GRAVITY_MPS2_PER_G * gravity_g * radius^2 / max(d, 0.5 * radius)^2
```

plus the star's `mu / d^2` at the heliocentric origin, with `mu == 0` disabling
it. No cutoff, because a transfer arc needs the far field.

**`SURFACE_GRAVITY_MPS2_PER_G = 25.0`** anchors both, and that is stated as the
reason it is one constant: the walker has always used 25 m/s^2, and the orbital
field is anchored to it so the two agree at the surface.

## Starting point before this change

`GravityWell::acceleration_at` is one well: inverse square outside the radius,
linear to zero inside it (uniform density, continuous at the surface), guarded
at the centre. `CelestialScene::planet_at_origin(PLANET_RADIUS, 9.0)` supplies
9.0 m/s^2. `walking.rs` computes the same falloff inline for the walker.

Two notes on that. The interior model is the more principled of the two, and
neither is reachable in play. And the walker's inline copy is a second place the
falloff is written, which is the divergent-path defect this project's own rules
name; whatever else changes, that should collapse to one call.

## The band shape is the interesting part

A hard edge is what a *mode* boundary wants. Being on a planet and being in
space are different simulations here - different basis, different camera,
different controls - so the transition needs a definite radius rather than an
asymptote. An uncapped inverse-square field has no such radius, so any handoff
built on it is a threshold chosen elsewhere, and a threshold chosen elsewhere is
a number that can disagree with the field it is supposed to describe.

The taper between `1.4 R` and `1.8 R` is what stops that edge being a step: pull
fades over a band you can fly through rather than switching off at a line.

## Settled implementation policy

`docs/HANDOFF.md` settles the constant at **25 m/s^2 per g** and the default
bands at **1.4 R / 1.8 R**. Gravity is felt at human scale; changing a body's
radius does not change the fall through the same metre of height. Per-body
`gravity_g` is the explicit way to change that feel, including zero gravity.

Core owns the two mathematical fields and the anchor query. A body carries its
stable ephemeris index, centre, radius, gravity_g, and optional band overrides.
Selection compares band multipliers, not centre distance or acceleration;
equal multipliers choose the lower stable body index, independent of iteration
order. The query returns body, multiplier, altitude and acceleration, and its
absence alone defines space. Invalid non-finite configuration is rejected.
The exact centre has zero acceleration because no inward direction exists.

Walking, ship control, and hover compensation all consume that same anchor
query through `CelestialScene`. Flight readout exposes the query's space state.
The inverse-square field remains available as a core reference, finite inside
the body and sharing the 25 m/s^2 constant, but is not summed into ship motion.
No star force, n-body ship integrator, trajectory solver, or automatic
boarding/disembarking is introduced. The manual F occupancy switch is distinct
from whether an actor is spatially inside a body's anchor field.

Tests pin exact band edges, overlap selection and deterministic ties, explicit
zero overrides, finite centre behavior, equal surface acceleration in both
fields, and actual Avian walker/ship acceleration and hover. Jump and one-metre
fall tests measure the new 2.88 m apex / approximately 0.28 s fall at 1 g. The
current six-metre terrain steps are unchanged by this gravity change; falling
six metres takes approximately 0.69 s at 25 m/s^2. The rescale remains separate.
Until that rescale, a full six-metre terrace is higher than the restored jump;
the unchanged walk/fly switch remains available for traversing those rises.
The 9 m/s^2 comparison runs the same motor with a lower g multiplier to isolate
the acceleration change; it is a controlled baseline, not a replay of the old
inverse-square implementation at an arbitrary terrain altitude.
