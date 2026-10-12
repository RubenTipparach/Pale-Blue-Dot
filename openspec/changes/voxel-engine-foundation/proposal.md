# Proposal: the volumetric voxel engine foundation

## Why

The desktop explorer is a surface-column preview. Every column carries one
height, so the planet has no interior: no cave, no overhang, no independently
removable block, and no durable edit. Every gameplay verb this game is
eventually about - mining, building, digging out a base - needs voxel occupancy
that a height cannot express, and needs it to survive a reload.

It is also the only way to metre-scale ground. The measured preview tile is
18.88 m across with a 6 m elevation step (see the `planet/scale` capability),
which is 16x coarser laterally and 6x coarser vertically than this design
targets, and the gap cannot be closed by raising the subdivision level: level 12
on a 4 km body is 167,772,162 cells, about 21.5 GB of topology alone. The whole
detailed globe must never be allocated. Sparse streamed chunks are the answer,
and they are what this change is.

## What changes

The production engine described in `design.md`: streamed radial chunk slabs over
the existing spherical dual topology, CPU-authoritative voxel occupancy with
durable edits, GPU face extraction and baked light as derived state, and the
capability tiers and budgets that bound all of it.

## What does not change

The topology, the reference-frame rules, the assisted-flight contract and the
walking contract are already specified and already hold. This change builds
underneath them; it does not renegotiate them.

## Non-goals

- Multiplayer. The single-player bubble is the scope; the frame rules are
  written so a second bubble is possible later, and that is all.
- Replacing the surface-column preview before its replacement renders. The
  preview stays the far tier.
- Any performance claim that has not been measured on hardware.

## Status

This is the standing design, not scheduled work. It was `docs/engine-architecture.md`
and is a proposal because most of it is unbuilt: putting it in `openspec/specs/`
would assert that the engine does things it does not do. What the prototype
actually satisfies today lives in `openspec/specs/` and is pinned by tests.
