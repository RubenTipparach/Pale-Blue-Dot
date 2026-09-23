# Tasks

## 0. Measure what exists
- [x] Field structure over 40,000 directions: latitude bands, ocean against
      land, distribution, correlation length, pattern motion (proposal, design
      section 1).
- [x] The render's global threshold traced (`apply_weather`); the fixed sun
      declination read off `SUN_FIXED`.
- [x] Held-Hou and deformation-radius arithmetic for `coriolis_scale`
      (design section 5).
- [x] Baseline captures from orbit (natural and half); the half-cover globe
      measured at 98-100% cloud in every fifth of the disk.

## 1. Cells in the core
- [ ] `dual_sphere` and its neighbour tables move to `pbd-core`; the app calls
      the core's. The pentagon test moves with them.
- [ ] Per-cell geometry: area, edge lengths, centre distances, tangent basis;
      the gradient and divergence operators, written for any field. Tested on a
      known field (the gradient of a linear field is its slope; the divergence
      of a rigid rotation is zero).

## 2. The simulation
- [ ] `pbd_core::atmosphere`: the state, and the step's seven stages in order
      (design section 4), with core arithmetic for `exp` and the rotation.
- [ ] Surface constants from the terrain generator.
- [ ] Knobs in a validated `atmosphere.ron` with units; code-defaults test.
- [ ] Unit tests: pressure bump, the spin's sense, water budget, finite at the
      extremes, determinism, save and load round trip.
- [ ] The `climate` example; the circulation, water and structure claims
      checked; `coriolis_scale` settled on its output; the numbers recorded
      here. If the three cells do not form, the nudge, and say so.

## 3. The seam
- [ ] `cloud_cell` reads `&Atmosphere`; `rain_trailing` goes; the old field is
      kept only as the start's perturbation.
- [ ] The app owns the atmosphere as a resource, stepped with the world clock
      and held while paused; the player's `Weather` and the rain map read it.
- [ ] Lightning from the strike list; the clock-hash `Lightning` goes.
- [ ] The slider becomes the forcing at the player.

## 4. Saves
- [ ] The state and a simulation version in the world save; written every
      `snapshot_s` and on leaving; spin-up for a new world, timed.

## 5. The clouds
- [ ] The weather maps (A and B, previous and next) filled by the core's
      interpolant and uploaded at `map_hz`; bytes a second measured.
- [ ] `cloud_density` remapped by the local cover, its profile scaled to the
      local top; the slab raised to `cloud_top_max_m`, validated under the
      atmosphere shell.
- [ ] Flow-map advection by the cloud-level wind.
- [ ] `slab.y`, the global cover lane, and the fixed drift removed;
      `CloudLayer` and the water view re-laid out; their size tests moved.
- [ ] `tau_up` handed to `cloud-lighting`'s ambient.

## 6. Check
- [ ] Captures: orbit at three hours, a 60-frame time-lapse of a low, and a
      storm forming under the slider; measured.
- [ ] Step cost at levels 4, 5 and 6; frame A/B on llvmpipe (relative only).
- [ ] fmt, clippy, workspace tests, `openspec validate --all`.
- [ ] Owner's in-game check; then the requirements move to `openspec/specs`,
      and `storm`'s "decided by the field alone" lightning clause is retired
      with them.
