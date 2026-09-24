# Tasks

## 0. Mockup (done, awaiting the owner)
- [x] Three.js prototype of the Kestrel, Tern and Loon, on the engine's sphere,
      gravity and tick rate, with adjustable wind, gusts, rain and current,
      and a toggle between today's swell and the proposed sea
      (`docs/mockups/vehicles.html`, published).
- [x] Headless checks of settling, hover takeoff, sailing, paddling in a squall
      and the unattended Kestrel; three bugs found and fixed in the design
      (design section 8).
- [ ] The owner's verdict on the feel (design section 10). Nothing below starts
      before it.

## 1. The sea (`pbd-core::sea`)
- [ ] Fixed 30 + 1 component table; deep-water ω from the planet's gravity;
      phase against the saved world clock.
- [ ] Pierson-Moskowitz amplitudes with cos² spreading, scaled to
      Hs = 0.21 U²/g, steepness cap 0.1; sea-state lag; shoaling by depth.
- [ ] Height and orbital velocity at a point and depth, plus the current.
- [ ] Tests: a change of wind never moves a crest by more than its amplitude
      change; Hs matches the formula within 5 %; amplitude at zero depth is
      zero; the velocity is the time derivative of the height.
- [ ] Sea state as a slow per-cell field in the atmosphere step, saved with the
      weather.

## 2. Drawing it
- [ ] WaterView uniform carries the table; `water.wgsl` loops over it; the
      literal sines are removed; per-vertex filter at 2.5× the cap spacing.
- [ ] Sea-state channel on the weather maps; the spectrum function in WGSL.
- [ ] Compute-shader test of the WGSL height against `pbd_core::sea` to 1 mm
      on a headless adapter; layout test of the uniform.
- [ ] The camera's water state reads the local wave height (spec delta,
      `planet/water`).

## 3. The air at a point (`pbd-core::atmosphere::gust`)
- [ ] Log-law shear, frozen-turbulence gusts, rain downdraft; blend to `upper`
      above the deck. Tests: deterministic for equal inputs; mean of the gust
      over a long window is zero; shear is 1 at 10 m.

## 4. Physics core (`pbd-core::vehicle`)
- [ ] Foil function with stall and control deflection; tests against thin
      aerofoil slope and the flat-plate limit.
- [ ] Hull from a shape function: cells for buoyancy, the same sections for
      the loft; buoyancy and heave damping over the cells.
- [ ] Hull resistance with the hull-speed wall; lateral drag.
- [ ] Rotor model (inflow, ground effect, edgewise drag), Kestrel mixer and
      assist (attitude, vertical speed, ground velocity).
- [ ] Sail with a free boom to the sheet; keel and rudder; crew hiking.
- [ ] Paddle strokes as blade drag; stern rudder.
- [ ] Water aboard: rain, weir inflow, drains, bailing, slosh.
- [ ] Mooring and anchor lines scaled to mass.
- [ ] `vehicles.ron` with units and validation; code-defaults test on the
      shipped file.
- [ ] Headless scenario tests mirroring the prototype's checks (design
      section 8), with the prototype's numbers as the reference band.

## 5. The app
- [ ] Vehicle plugin: Avian bodies from the craft data; force application in
      `PhysicsSchedule`; contact against `PlanetContact`.
- [ ] Meshes from the craft data (hull loft, foils, rotors, sail, paddle);
      nearest-point textures where textured.
- [ ] Weather at each craft from `Air`; the same function for every craft,
      occupied or not.

## 6. Boarding and persistence
- [ ] G boards and leaves; the walker parks in the craft's frame; exit points
      resolved against `PlanetContact`.
- [ ] Vehicle record in `WorldSave` with a version bump; written through the
      durable path on spawn, board, leave, moor, anchor, cast off and rest.
- [ ] Unattended policy (design section 6) and the coarse out-of-region step.
- [ ] Test: board, fly, leave in the air, reload, the same ID is there and
      boardable.

## 7. Camera, controls, HUD
- [ ] `VehicleCamera`: seat view on raw mouse look, chase view; generalised
      `set_active_mode`.
- [ ] KESTREL, TERN and LOON groups in `BINDINGS`; reader test extended.
- [ ] Instrument strip and state chips per the mockup.

## 8. Check
- [ ] fmt, clippy, workspace tests, `openspec validate --all`.
- [ ] Captures: each craft at rest, underway in a breeze and in a squall, on
      an offset planet as well as the origin one.
- [ ] Owner's in-game check; requirements move into `openspec/specs` as each
      becomes true.
