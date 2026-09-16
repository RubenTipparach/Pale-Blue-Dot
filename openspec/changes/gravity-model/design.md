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
is `is_in_space`. The per-body multipliers are config (`gravity_full_mult`,
`gravity_falloff_mult`), zero meaning "use the global default".

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

## What we have

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

## Open: the constant

25 m/s^2 is right for a 300 m planet and is explicitly arcade-scaled. Whether it
is right for a 4 km one, or for whatever radius `preview-scale-and-shader-parity`
settles on, is not something Tenebris can answer for us - its own comment ties
the number to its planet size. Two coherent positions:

- **Take it as spec**, as the hex size was taken, so a walker feels identical on
  both. Then the body radius question and this one are the same question.
- **Take the shape, retune the constant**, keeping the two fields, the bands and
  the selection rule, and choosing the surface value for the bodies this project
  actually has - with the reason recorded, which is the part that is missing
  today.

The measurement that decides it is the fall time through one cell height, since
that is what reads as float: 0.28 s in Tenebris against 1.15 s here.
