# Pale Blue Dot

Design repository for **Tenebris**, a procedural, multi-planet hex-voxel exploration and survival game.

This repository currently defines the contract for the Rust rewrite before production code is imported:

- [`AGENTS.md`](AGENTS.md) — repository guidance for coding agents, translated into tool-neutral instructions.
- [`docs/ENGINE_ARCHITECTURE.md`](docs/ENGINE_ARCHITECTURE.md) — Bevy and Avian engine architecture, coordinate spaces, simulation boundaries, and delivery plan.
- [`docs/GAME_DESIGN.md`](docs/GAME_DESIGN.md) — product pillars, world generation, traversal, progression, multiplayer, and content requirements.

## Non-negotiable direction

- Bevy owns application scheduling, ECS, rendering integration, assets, and states.
- Avian owns local collision detection, rigid bodies, and character/vehicle contacts.
- Planets, moons, and stations use deterministic on-rails ephemerides.
- Player ships do **not** use patched conics or orbital maneuver planning. They use approachable Newtonian flight with gravity falloff, configurable speed/acceleration limits, and inertial/rotational dampeners.
- Terrain is made from streamable hexagonal-prism voxels and must support worlds several kilometers across without requiring the whole planet to be resident.

## Status

This is an architecture and pre-production baseline. Version pins, minimum supported Rust version, and implementation crates should be chosen in the first executable vertical slice and then recorded here.
