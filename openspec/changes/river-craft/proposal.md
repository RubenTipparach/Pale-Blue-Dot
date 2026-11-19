# Proposal: river craft (rafts, a tug and its tow, a log drive, net boats, anchoring)

**Mockup:** M34 River Craft, https://claude.ai/artifact/GZTZJ4Vg1bSbg4SH2rYY97
(`docs/mockups/river-craft.html`), published 2026-11-16; on the tracker.

## Why

The owner, 2026-11-16, playing the new flowing rivers (`water-flow`):

> dude the river stuff is so much fun, can you make a mock up of stuff like
> rafting physics? or design a boat that can tug other boats? log hauling?
> boats with nets, also anchoring

The rivers flow now, but nothing on them feels the flow. The game's craft
(`pbd_core::vehicle`: the Tern, the Loon, the Cog) float on the sea table and
take a sample of `Surroundings` each tick: the sea, its state, the tide, the
wind, the depth. There is no water velocity in it, so a canoe on a running
river sits still. A line between two craft does not exist (`Mooring` ties a
craft to a fixed point), an anchor holds wherever it lands, and logs, booms,
bundles and nets are not things yet.

## What Changes

A mockup first, as for every look and feel change (CLAUDE.md, "Write it up
before touching code"):

- **`docs/mockups/river-craft.html`, mockup M34 "River Craft"**, a playable
  three.js toy on a river, one 3D view at a time in five tabs:
  1. **Raft** (`#raft`): a log raft carried by the flow, rowed with two oars
     (version 4; it was poled and paddled before), spun by shear, held in the eddy
     behind a rock, bumping off rocks and banks, over a one-layer fall. Flow
     arrows on a toggle.
  2. **Tug** (`#tug`): a steam tug towing two barges on lines that go slack
     and taut, the barges following with their own drag.
  3. **Log drive** (`#logs`): logs rolled into the river from a rollway, a
     jam at a narrows that can be broken, a boom that catches the drive, a
     bundle made from the caught logs and hauled away by a boom boat.
  4. **Nets** (`#nets`): a skiff dragging a net (a cloth of points, corks on
     top), fish schools caught by it, the full net dragging harder, hauling
     it in.
  5. **Anchor** (`#anchor`): one anchor per boat (version 6, owner: "also
     only really need one anchor per boat"), the rode paying out only to the
     bed (version 5), the anchor setting and then holding the boat within
     its swing on the rode (version 4, owner: "anchor in mockup should stop
     you from moving"), weighing anchor; the bed changes only how fast it
     sets.
- **The boat's menu** (version 6, owner, 2026-11-19: "rope length can be
  controlled in the boat menu, yes boats have menus for inventory and
  various stuff"): opened with C aboard or aiming at a boat, it holds the
  boat's hold beside the pack, its anchor (drop or weigh, the rope's length
  from "to the bed" up to the rode's maximum) and its lantern; on the
  Anchor, Raft and Rowing tabs; design 10.
- **Rowing** (`#rowing`, added 2026-11-19 on "mockup better rowing on a
  raft"): today's raft stroke beside the proposed one (two oars on a rowing
  frame in five stages on one beat, no pole (version 4), splash and
  swirls, first-person hands), and the raft's own lantern lit at night
  ("Rafts should also have a light that comes with it at night"); design 9.
- **The design of the port** (`design.md`): a current in `Surroundings`
  derived from the flowing water, craft-to-craft lines generalising
  `Mooring`, anchor holding by bed, logs and booms as floating bodies, nets
  beside the rod.

No game code changes in this change until the owner approves the mockup and
asks for the port.

## Capabilities

### New Capabilities

- `world/river-craft`: craft on flowing water feel its current; lines between
  craft; anchors that set and hold; rafts, logs, booms and nets.

### Modified Capabilities

None until the port.

## Impact

- New: `docs/mockups/river-craft.html` and `docs/mockups/river-craft/`
  (`engine.js`, `physics.js`, `scenes.js`, `app.js`, and the stone-and-oak UI
  pieces copied from `mission-tree/ui/`).
- New: screenshots in `docs/screenshots/mockup-river-craft/`.
- The port, when asked for: `pbd-core` (`vehicle` gains a current, lines,
  anchor holding, new kinds), `pbd-app` (`vehicles.rs` samples the current
  from `water_flow.rs`), the durable records of craft, lines, booms and
  bundles. Saves: new records only; old saves have none and open unchanged.
