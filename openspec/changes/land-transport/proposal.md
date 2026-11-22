# Proposal: land-transport

## Why

**The owner, 2026-11-21:** "add cars and cars and carts, and horses and
trains (and tracks) mockups to phase 5". **And 2026-11-22:** "Roads and cars
mockup?". **Approved 2026-11-22** (M38 version 1, in chat: "noth apporoved, I dont see trade panel"): phase 5 waits for the owner's request like the rest.

Phase 5 is mass transportation (CLAUDE.md, "The first game loop", "Then mass
transportation"): airships, sea ships, sailing and navigation, carts and
horses, cars, and trains on their tracks. Phase 3 builds the roads they will
run on (`townsfolk` design 10, 10a, 11, 31 and 34): hex road cells in dirt and
gravel whose top slopes one layer a cell, fords, timber bridges, lamps near
towns, and roads laid by the world between neighbouring towns. Nothing yet
says what a cart, a car or a train needs from those roads, or how a train
runs on a hex world at all.

**Nothing in this change is built, and phase 5 does not start until the owner
asks.** This change is the plan and the mockup (M38, `land-transport.html`),
written before any code as CLAUDE.md requires.

## What Changes

- **A mockup, M38 Land Transport** (`docs/mockups/land-transport.html`), in
  four tabs, each with one 3D view, a Drive button (off by default), orbit by
  drag, scroll and pinch, and day and night lit by its own lamps:
  1. **Carts and horses**: a ridden horse, a horse in the shafts of a
     two-wheeled cart and a horse on the pole of a four-wheeled covered wagon,
     on a dirt road and up a climb; a hitching rail.
  2. **Cars**: an early car and a small truck on gravel, up the climb, over
     the timber bridge and round 60° bends.
  3. **Trains and tracks**: straight and curved track on the hex grid, a
     grade of one layer over six cells on an embankment and in a cutting, a
     small station, a little tank engine with a coach and an open wagon.
  4. **Side by side**: all of them on one road and a track, at one scale.
- **The design answers** (design.md): how track curves on a hex grid; what
  grade a train climbs and how the cells under it are cut and filled; what a
  road vehicle needs from a road cell (slope, bends, width, bridges); how
  vehicles, their animals, trains and track are saved.
- **Spec delta** `world/land-transport`: the requirements as proposed. They
  stay in this change until each is built and pinned by a test.

## Capabilities

### New Capabilities

- `world/land-transport`: road vehicles (horse, cart, wagon, car, truck) on
  road cells, and trains on track laid on the hex grid; how both are stored.

### Modified Capabilities

- None now. When built, `townsfolk`'s roads gain a passing-place rule and
  `world-persistence` gains vehicle and track records.

## Impact

- `docs/mockups/land-transport.html` (new), its screenshots in
  `docs/screenshots/land-transport/`, and its row in
  `docs/mockups/mockup-work.md`.
- No game code. No change to generation, saves or any default.
- Depends on, when built: `townsfolk` (roads, bridges, lamps),
  `animal-husbandry` (the horse as an animal with an ID), `vehicles`
  (boarding, occupancy, a vehicle that outlives its driver leaving),
  `world-persistence` (durable records), `lamps-and-lanterns` and the lit-
  like-terrain rule for everything placed in the world.
