# Design: cull cloud puffs outside the view

## The bound

The vertex stage (`cloud_puff_vertex`, `water.wgsl`) places a billboard's
corners at `c + f0*(±across) + f1*(±up)`, where `f0`, `f1` turn with the eye
and `up = mix(radius_up, radius_across, k²)`, so `up <= max(radius_up,
radius_across)`. Every corner therefore lies within

```text
R = sqrt(across² + max(across, up_radius)²)
```

of the centre, whatever the eye. The bound is the axis-aligned box of half
size `R` round the centre (body-local): it holds that sphere, so it holds the
billboard in every orientation. `R` is grown by 1% and 1 m against roundoff.

## The test

The planes come from the frame's `clip_from_body` (the matrix the vertex
stage draws with, `WaterView.clip_from_local`), by Gribb-Hartmann rows. Bevy's
projection is reverse-Z and infinite, so there are five: left `w+x`, right
`w-x`, bottom `w+y`, top `w-y`, near `w-z`; the far plane is at infinity and
is not tested.

A box is culled only if, for some plane, its corner furthest along the
plane's normal (the "positive vertex") is on the outside. That is the
standard conservative test: it never culls a box that intersects the
frustum. It may keep a few boxes near a frustum corner that miss it, which
costs a few billboards and is accepted.

An eye inside a puff's box keeps it (the box straddles the near plane).

## Where

- `pbd_core::frustum` (new): `Planes::from_clip(Mat4)`, and
  `Planes::touches_box(centre, half) -> bool`. Pure, engine-free, tested.
- `pbd_core::cloud_puffs::bound_radius(&Puff) -> f32`.
- `PuffSites::build` takes the planes and filters after `over_horizon`,
  before the sort; it returns the three counts.
- `planet_water.rs` passes `clip_from_body`.

## Tests

- A box straddling each of the five planes is kept; one wholly outside each
  is culled.
- A box behind the eye is culled; a box round the eye is kept.
- A puff whose centre is off screen but whose billboard reaches into it is
  kept: the bound's billboard corners, projected, are checked against the
  screen for a sweep of eyes and puffs (every kept-or-culled decision agrees
  with "some corner or edge on screen" whenever that is true).
- The sort order of the kept puffs is unchanged by the cull.

## Measuring it

- Puff counts and the build's CPU time in the frame log, read on the
  `clouds` and `storm` flights.
- Before against after, interleaved, 2 repeats, `clouds` and `storm`, GPU
  apps closed. No visual change is expected; stills of the same frame before
  and after are diffed to show it.

Recommendation taken (ask only with screenshots): a box round the bounding
sphere, the five-plane conservative test, no far plane.

## As built (2026-10-29)

Measured in `docs/benchmarks/2026-10-29-cull-wet`: the flights are 2.5 ms
faster at p50 (clouds 19.74 -> 17.22, storm 19.70 -> 17.12), and the GPU
time up to the composite falls by the same 2.45 ms. The cull keeps about half
of what passes the horizon (250-400 of 450-900 puffs, from 2,080 built); the
build costs the CPU 0.15-0.18 ms. Why the culled billboards cost the GPU that
much is not established (a GPU capture would show it). The same cloudy frame
before and after (`docs/screenshots/puff-frustum-cull/`) is identical across
the sky; the only band that differs is far wet grass, which is
`wet-near-detail`.
