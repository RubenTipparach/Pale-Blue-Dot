# Proposal: ship tiers, from hex-stack rockets to antimatter landers

Status: **planning only**. Nothing here is built. One decision is the owner's
(G4 in the survey, the orbital-mechanics invariant); the plan is written for
the recommendation and marked provisional until it is answered.

Mockup: `docs/mockups/ship-tiers.html`, published at
https://claude.ai/artifact/4e3jLcjN4yH6NhJmyWKuY5 (link recorded in `docs/mockups/mockup-work.md`,
id M21 on the tracker). Screenshots: `docs/screenshots/mockup-ship-tiers/` (the
assembly building's four so far; the GPU screenshot gate, including G4's
orbit shots for the survey, was not run: mockup work moved off the desktop
during perf tuning).

## Why

The owner, 2026-11-12, verbatim:

> different tiers of ships:
> nuclear/chemical rockets: build hexagon stacks, the VAB will only do hex stacks
> * kerbal style stack building. but with hexagons, a 3d hex grid, thrust:weight ratios to get into space.
> * once in space, we can do some orbital stuff (the orbital mechanics in tenebris rust was pretty cool)
> fusion engines:
> * faster than chemicals, more fuel so less stuff to care about regarding orbits, more direct routes possible.
> * navigation computers plot courses
> * automations - freighters can ferry goods for you
> * transitions to rockets for transporting between planets, landers can lift off into low orbit.
> antimater engines:
> * landers are the standard to traveling anywhere
> * starships are now possible - they can travel between stars using warp drives. Unlocks the ability to travel to other solar systems.

Withdrawn the same day: "Let's not do warp." Starships, warp and other star
systems are out of this change.

The owner, 2026-11-14: "update the rocket sim to have a larger ground area,
and maybe a small planet to test orbit transitions around, and a solar system
with navigation nodes to plot flight paths and rocket burns". The mockup gets a
launch field 2.4 km across (the game's cells near the pad, a cheap far ring),
Taper, a 600 m test moon 14 km out, and a planner tab: the whole system, a
predicted line integrated forward, and maneuver nodes. Nodes contradict the
flight invariant; they are in the mockup only, and the invariant is asked
about (survey G4a, design 11.4).

Today a ship is one craft flown by assisted Newtonian flight with a bounded
gravity field, and the earlier mockups (`ship-crafting.html`, `travel.html`)
give every ship the same abilities from the start. The tiers turn space travel
into a progression tied to the power ladder the industry mockup is drawing
(fission, then fusion, then antimatter), and give the first tier the
thrust-to-weight and delta-v play of Kerbal, on this game's hex cell.

## The conflict, stated rather than resolved

CLAUDE.md, "Non-negotiable world and flight invariants":

> Ships use local gravity falloff, Newtonian motion, configurable linear/angular
> limits, and inertial/rotational dampeners. Do not restore rocket patched
> conics, maneuver nodes, transfer-window requirements, or n-body ship
> simulation.

`openspec/specs/player/flight/spec.md`, "No orbital rocket mechanics", says the
same and is pinned by a test. The gravity-model change took Tenebris's bounded
anchor field (full pull to 1.4 R, nothing past 1.8 R) for ships and left its
orbital field out. In that field orbits only fit inside 1.8 R and do not close,
and the moon at 26 km is reached on a straight coast. "Some orbital stuff" in
the Tenebris sense pulls against all three.

What Tenebris actually does is in `design.md` ("What Tenebris does"). In short:
rails for planets (parked for physics), Newtonian rockets in a summed
inverse-square field, patched conics drawn from Kepler solutions, maneuver
nodes, SAS hold modes. No burn autopilot.

**Recommendation (B, provisional on G4):** rockets fly in Tenebris's orbital
field (inverse-square from every on-rails body and the star, integrated the
same way the game integrates today), with a predicted path drawn by running
that integration forward and Ap/Pe/T read off it. No conic solver, no
sphere-of-influence hops, no maneuver nodes, no transfer-window requirement, no
ship-to-ship gravity. The fusion tier's nav computer is the planner. Assisted
ships, hover and walkers keep the anchor field. CLAUDE.md is **not** edited by
this change: if the owner picks B, the invariant's wording changes in the
commit that builds it, with its test.

## What changes (once built)

- **The assembly building (VAB)** builds rockets only as stacks of hex cells
  (2.833 m across, 1 m layers): a core column plus the six cells round it, with
  radial symmetry 1x/2x/3x/6x, decouplers splitting stages, and live mass,
  thrust, TWR and delta-v per stage.
- **Tier 1, chemical and nuclear rockets.** TWR over 1 to leave the pad,
  delta-v to reach orbit; solid boosters, a nuclear thermal upper stage fed by
  the industry chain's fuel rods. Orbit Pale Blue, reach Lantern on an arc.
- **Tier 2, fusion.** Ships built in orbit that never land; a nav computer that
  plots and flies burn-flip-brake courses; automated freighters on routes;
  landers that lift to low orbit to meet them.
- **Tier 3, antimatter.** Landers that fly surface to surface anywhere in the
  system. No warp and no other star systems (the owner, 2026-11-12: "Let's
  not do warp").
- **Bigger rockets you walk inside** (the owner, 2026-11-12: "Make me some
  more rocket building presets. Allow larger rockets with more leg room to
  walk around in. (3-4 radius? Blocks)"): the assembly building's grid
  reaches four rings out from the core, a rocket up to nine cells (25 m)
  across, with crew decks, ladders and hatches inside, and more presets.
- Engine work the tiers need: an orbital field for rockets (if B), and
  freighter routes as stored records that move goods through the durable
  save path.

- **The planner (mockup, 2026-11-14):** maneuver nodes with prograde, normal
  and radial handles on the integrated line, chained, flown by hand; Taper to
  practise launch, orbit, transfer, capture, landing and return; the Aubade
  system to scale. Provisional on G4a; not in the spec deltas.

## What does not change

- Planets, moons and stations stay on analytic rails.
- f64 system coordinates, bounded f32 local frames, rebasing at tick
  boundaries (CLAUDE.md). There is one star system.
- Saved games: a ship design, a rocket in flight and a freighter route are
  records that migrate forward; nothing a player built
  is dropped (CLAUDE.md, "Saved games survive every change").
- The Kestrel, Tern and Loon are untouched; they are the craft before the
  tiers.

## Impact

- Specs: a new `player/rockets` capability (the VAB rules, staging, TWR and
  delta-v) and, under B, a modified "No orbital rocket mechanics" requirement
  in `player/flight`. Both are deltas in this change; neither is true yet.
- Mockups it touches by reference, not by edit: `ship-crafting.html` (its
  grid moves onto hex cells and becomes the tier-2 ship builder),
  `travel.html` (its transit drive is the tier-2 nav course; its free hover is
  a tier-2 lander and tier-3 feature), `industry.html` (the power ladder),
  `stations.html` (where fusion ships are built and dock).
- Code: `pbd-core` (part catalogue, stage maths, orbital field, path
  prediction, nav-computer solver), `pbd-app` (the VAB, the rocket flight
  mode, the HUD readouts, freighters). None of it is
  started.
