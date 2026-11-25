## Why

The owner, 2026-11-18, two asks for the player in the water:
- "Slow down swimming by 50% for the player";
- "add a small push force for water current against the player swimming.
  Currents don't affect the player if their feet are grounded".

Today a swimmer moves at the walk's speed through water (8 m/s x 0.5, the
meal's stride on top; sprinting 14 x 0.5) and the water carries nothing: a
river's flow moves boats (`vehicles::fresh_water`) but not a swimmer.

## What Changes

- **Swimming is half as fast.** A swimmer (eyes under and not wading, or in
  the water with its feet off the bottom) moves at half its old speed, at
  the surface and under it, sprinting too: `swim_speed_mult` 0.5 in a new
  `assets/config/swimming.ron`. Wading, feet on the bed and head out, is
  untouched.
- **The water's current pushes a swimmer.** While the walker is in the
  water with its feet off the bottom, it is carried toward half the water's
  own velocity there, capped at 1 m/s, easing in and out over about two
  thirds of a second, so entering, leaving and crossing cells never jolts.
  The water's velocity is the one boats already float on: a river's or a
  lake's flow from the water rule's own levels (`vehicles::fresh_water`),
  and at sea the ocean's surface current from the weather.
- **Feet down, no push.** A walker whose feet touch the bed or the ground is
  not pushed, and what it carried eases away.

## Capabilities

### Modified Capabilities
- `player/walking`: a swimmer's pace, and the water's push on it.

## Impact

- Code: `pbd-app::walking` (`drive_walker`, `GroundState`, a
  `SwimSettings`), `pbd-app::config` (load `swimming.ron`),
  `pbd-app::vehicles` (`fresh_water` shared), tests.
- Config: `assets/config/swimming.ron`, every field with its unit.
- Saves: nothing.
- Frame cost: one column-water probe a tick for a walker in the water (the
  probe a boat makes). Not measured.
