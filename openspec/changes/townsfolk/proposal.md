# Proposal: townsfolk

## Why

**The owner (2026-09-27): "can you also make sure all features in the mockup
are implemented too? make sure its in the roadmap".** And on what they look
like (survey T2): **"this is fine, instead of cubes I prefer hexagonal blocks
that look like cubes, keeps the spirit of the game".**

The towns mockup has people. Each settlement has its own, 16 in the walled
town:
- some walk a path between two points, swinging arms and legs;
- some work in place: the smith hammers, the innkeeper wipes the bar;
- the rest stand and look about;
- a guard on the gate carries a spear;
- every one of them is solid to the walker.

`tenebris-towns` and `cities-in-the-world` left them out as "a separate
change". This is that change, so every feature of the approved mockup has a
place on the roadmap.

## What Changes

- **People in every settlement, as the mockup has them.**
  - Walkers go back and forth along a path between two points.
  - Workers work in place (hammering, wiping, stirring).
  - Idlers look about.
  - A guard stands at each gate.
- **Built from hexagonal blocks that look like cubes** (the owner, T2). Each
  part of the body (legs, body, belt, head, hair, arms) is a hexagonal prism
  in the terrain's pixel style, and clothes and skin take their colours from
  the settlement's people and biome.
- **Solid to the walker.** Each person is a circle of 0.28 m from the feet to
  1.9 m, pushed out of like any thin solid. The walker slides round them and
  never passes through.
- **Lit like everything else that moves.** They take the light field through
  `lamps-and-lanterns`' sampler, so a smith by the forge is lit by it at night.
- **Derived, not stored.** A townsperson with no state is a law, following
  `world-persistence`'s rule. Who stands where, and what they do, comes from
  the settlement record and the seed. When people gain state (the owner's
  "you can steal from people lol, but there will be consequences"), they
  become records in a later change.
- **Out of scope:** schedules, talk, trade, reactions to theft or to a house
  taken apart, and walking off their paths. Those are the consequences change
  after the roadmap.

## Capabilities

### New Capabilities
- `world/townsfolk`: the people of a settlement:
  - who they are and where they stand;
  - how they move;
  - how they look;
  - that they are solid and lit;
  - that they follow from the settlement and the seed.

### Modified Capabilities
- None.

## Impact

- **`pbd-core`:** a `townsfolk` module that derives each settlement's people
  from its record and the seed: role, path or post, clothing and pace.
- **`pbd-app`:**
  - people drawn as hexagonal-prism figures with simple animation (walk swing,
    work loop, idle look);
  - their circles joined to the walker's thin-solid pushes;
  - their lighting from the field's sampler.
- **Order:** after `cities-in-the-world`, because they live in its settlements
  and use its thin-solid contact. Before `player-building`.
- **Performance:** a few hundred figures across the towns in range, each a
  handful of prisms. It is not measured in a cloud session (CLAUDE.md). The
  owner's `perf_suite.py` town scenario covers it.
