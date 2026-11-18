# Proposal: craft lit as the terrain is

## Why

The owner, 2026-11-18, after `drop-lighting` merged: "can you properly
light the raft? moving forward we need to apply proper lighting to stuff".

Every craft's root carries `LitByField` (the cell light field: sky and
lamps), but not `LitLikeTerrain`. So its faces take Bevy's own sun, with no
sun shadow from the terrain's cascades. This is the glow `drop-lighting`
removed from dropped items: a raft in a canyon's shade, or at dusk, stands
out brighter than the river and banks round it.

## What Changes

- **Every craft is lit as the terrain is.** The raft, the inner tube, the
  Loon and the authored models take `LitLikeTerrain` beside `LitByField` on
  their root (`vehicles::draw::build`). That gives them the terrain's own
  sun with its shadows and the field the ground is shaded from. It covers
  every mesh under the root: hull, deck, cargo and the crew.
- **Moving is already handled.** A moving craft's field is read again
  whenever its bounds move (`field_light`, the corners' cache).
- **A standing rule in CLAUDE.md:** everything placed in the world is lit as
  the terrain is.

## Impact

- `vehicles/draw.rs`: one marker.
- A test that a built craft's root is lit as the terrain is.
- Captures of the raft on a river at day and dusk, before and after.
