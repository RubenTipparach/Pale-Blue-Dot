# Proposal: three vehicles, a sea they can float on, and weather that pushes them

## Why

**The owner's request: "lets get some vehicles going, first I'd like a plane
that can vtol. then Id like a boat that I can ride on the ocean. I expect their
physics to be somewhat competent and react to weather wind and rain
accordingly", then "a sail boat vs paddle boat would be awesome", then "I
expect mockups in js first".**

The game has one craft and it is not a vehicle. The "Survey skiff" is an
invisible Avian sphere with a camera on it. F swaps the player between walking
and being that sphere. Nothing boards it, nothing records who is in it, and
nothing saves it. Its flight model is assisted Newtonian thrust with
dampeners: no wing, no lift, no stall, and no reaction to the wind the
atmosphere simulation already computes at every place on the planet.

There are three gaps, and a vehicle needs all three closed:

1. **Nothing applies weather to a body.** `Atmosphere::sample(dir)` returns
   the wind, the ocean current and the rain rate for any direction. The rain
   renderer, the clouds and the overlays read them. No force does.
2. **The sea cannot float anything.** Measured in `design.md` section 1: the
   swell in `water.wgsl` is three sines 1.05 to 2.26 m long, at most 0.18 m
   high, moving at 0.22 m/s. The water mesh has a vertex every ~1.64 m, so it
   can only draw waves longer than ~3.3 m. None of the three qualify, so the
   sea on screen is an alias of the function rather than the function. The
   function only exists on the GPU, by design: the water spec says it is
   "not evaluated a second time on the CPU". A hull floated on it would ride
   ripples the player cannot see.
3. **A vehicle is not an entity with a life of its own.** CLAUDE.md is
   explicit: "Exiting a ship changes its occupancy, never its existence.
   Persist its ID, pose/frame, inventory, damage, and ownership so it remains
   boardable." `WorldSave` holds the walker's pose and nothing else that moves.

## What changes

- **A mockup came first, as asked.** `docs/mockups/vehicles.html`, published
  at <https://claude.ai/artifact/UJebv7EdJUg1NLpvfVHi3D>, is a three.js
  prototype of all three craft. They sit on the engine's own sphere
  (sea radius 4,799.5 m, gravity 25 m/s²) in adjustable wind, gusts, rain and
  current, and you can switch between today's swell and the proposed sea.
  Its physics is the design below, written in JS so its feel can be approved
  before any Rust exists. What it measured is in `design.md` section 8.
- **One sea function, in `pbd-core`, that the shader draws and the hulls float
  on.** It is a fixed table of 30 components (5 wavelengths × 6 headings) plus
  one swell. Their wavenumbers and deep-water frequencies are constant; the
  wind sets only their heights, from a Pierson–Moskowitz spectrum with
  Hs = 0.21 U²/g. Because the table is fixed, a change of wind never makes the
  sea jump. The renderer drops any component shorter than 2.5× its own vertex
  spacing instead of drawing its alias. Waves shoal where the water is
  shallow.
- **Wind at a point** is the atmosphere's mean wind with a log-law height
  profile, plus deterministic gusts: frozen turbulence carried downwind, a
  pure function of place, world time and the local storminess. Rain adds a
  downdraft and stronger gusts.
- **Three craft**, each built from data in `assets/config/vehicles.ron`:
  - **Kestrel**, a VTOL tiltrotor. Every lifting surface is a foil with a
    stall. The rotors tilt through 90°, lose thrust with inflow, gain it in
    ground effect and drag sideways in a crosswind. Assist is the flight
    model's dampeners, restated for a winged craft: attitude hold and
    vertical-speed hold in the hover, and ground-velocity hold with the stick
    centred.
  - **Tern**, a cat-rigged keelboat. The sail is a foil whose boom swings free
    up to the sheet. The keel and rudder are hydrofoils. Resistance climbs a
    wall near hull speed. The crew hikes out to windward.
  - **Loon**, a canoe. Each stroke is blade drag against the moving water, so
    a stroke on one side turns the boat. Rain and waves over the gunwale fill
    it, and the water sloshes to the low side, which is what swamps a canoe.
- **Hulls are cells.** Buoyancy is summed over the voxels of the hull
  envelope, against the same sea function the water mesh draws. Heave, pitch
  and roll therefore come from the waves, and a vehicle is built the way
  everything else in this world is.
- **Boarding, occupancy and persistence.** G is the one interaction key: it
  boards the craft in reach and leaves the one you are in. A craft keeps its
  ID, pose and frame, velocity, water aboard, mooring and occupancy in a
  vehicle record. The record goes through the durable save path when the
  craft is spawned, boarded, left, moored or anchored, and when it comes to
  rest.
- **An unattended craft follows a written policy.** A Kestrel left in flight
  converts to the hover, holds its position and lets itself down. A boat
  eases its sheet, centres its tiller and drifts, unless it is moored or
  anchored.
- **A third camera**, the vehicle camera, with a seat view on raw mouse look
  and a chase view. It is the active camera, so the weather, the water state
  and the LOD anchor follow it with no further work.

## Impact

- **Specs.** `planet/water` gains the shared sea function and MODIFIES the
  requirement that forbade evaluating it on the CPU. `world/weather` gains
  gusts and the rain downdraft. `player/vehicles` is new.
- **Code.**
  - `pbd-core`: new `sea`, `vehicle::{foil, hull, craft, sail, paddle, rotor}`
    and `atmosphere::gust` modules.
  - `pbd-app`: a vehicle plugin (bodies, forces, contact, the camera and a
    HUD), and a `water.wgsl` that reads the sea table from the uniform
    instead of literals.
  - Saves: a `vehicles` record with a save version bump.
- **Nothing here changes the skiff or F.** The assisted-flight spec stands as
  it is. Vehicles are a second kind of craft, not a replacement for the first.
- **Out of scope:** damage, fuel, cargo, building a craft, multiplayer seats,
  swimmers bobbing on the new waves (a follow-up once the sea is shared), and
  landing the Kestrel on a moving deck.

## Status

Proposed. The mockup is built and published for approval. No Rust has been
written. The next step is the owner's verdict on the mockup's feel, then
`/opsx:apply`.
