# Design: townsfolk

## Context

See `proposal.md` for the owner's words. What the design has to work with
(2026-09-27):

- **The mockup's people** (`docs/mockups/towns.html`, `person()` and
  `animateNpc()`):
  - A person is a group of boxes: two legs (0.18 × 0.8 m), a body (0.5 ×
    0.72 m), a belt, a head (0.3 m), hair, and two arms hinged at the
    shoulder. An optional spear is held in the right hand.
  - A walker ping-pongs along its path at about 1.2 m/s, facing its way,
    with arms and legs swinging at 7 rad/s.
  - A worker animates its right arm: a hammer stroke, or a wipe.
  - An idler turns its head slowly.
  - Each person's solid is a circle of radius 0.28 m from the feet to 1.9 m,
    labelled with its role. It is moved with the person every frame and
    pushed out of like a wall.
- **The game, after the changes this follows:**
  - `tenebris-towns` gives thin-solid contact and the walker's slide;
  - `lamps-and-lanterns` gives the light-field sampler for moving things;
  - `cities-in-the-world` gives settlement records and active settlement
    entities;
  - `world-persistence` gives the rule that what has no state is derived.

## Goals / Non-Goals

**Goals:**
- Parity with the mockup's people: the same roles, movement, solidity and
  numbers.
- The owner's look: hexagonal blocks that read as cubes.

**Non-Goals:**
- **Behaviour beyond the mockup:** schedules, sleeping at night, following
  the player, talking, trading.
- **Consequences:** stealing, a house taken apart, reputation. That is a
  later change, after the roadmap (the owner, T5 and P2).
- **Crowds.** The counts are the mockup's, per settlement.

## Decisions

**1. People are derived from the settlement record.** Each template carries
its people as roles with posts or paths in the layout's axial coordinates.
The settlement's variation (rotation and mirror) charts them onto the sphere
with the buildings. Clothes, skin and pace are rolled from the seed and the
person's index, so a town's people are the same on every visit and on every
machine.

**2. A figure is prisms, animated on the CPU, drawn instanced.**
- Each body part is a hexagonal prism, flat side forward, sized like the
  mockup's boxes. Across the flats it matches the box's width, so the
  silhouette reads as the mockup's.
- The parts' poses are computed on the CPU each frame: the swing, the stroke
  or the look. They are drawn as instances of one prism mesh with the
  terrain's atlas, so a hundred people are one draw.
- The pixel texture is the terrain's, sampled nearest (CLAUDE.md).

**3. Solidity reuses the thin-solid contact.** A person's circle is a thin
solid in the settlement's contact index. The index updates the circle's
centre each tick, as the mockup does, and the walker's push-and-slide handles
it with no special case.

**4. Lighting reuses the sampler.** Each person samples the field at the
corners of their bounds, like the player's body, so no second lighting path
exists.

## Risks / Trade-offs

- [Moving solids in a contact index built for static pieces] → A person's
  circle is kept in a small list apart from the static index, since there are
  at most a few dozen per town. The walker checks both.
- [Figures pop in at the edge of range] → They fade with the settlement's
  pieces (`cities-in-the-world`, decision 6).
- [Frame cost] → Not measured in the cloud (CLAUDE.md). The town scenario in
  `perf_suite.py` covers it.

## Migration Plan

- There is no save change, because people are derived.
- Rollback is the previous build: the towns are empty of people again.
