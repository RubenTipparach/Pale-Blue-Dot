# Proposal: small boats you make, a raft that hauls, and fishing afloat

## Why

**The owner's request (2026-11-16): "Get latest from main, let's build on the
river stuff. Verify innertubes are built. And we have valid crafting recipes
for boats up to a small row boat. Tugs will need a basic dock. So that will be
out of scope for now. Rafts can still haul things tho. I need you to add
fishing to boats, rafts, and tubes as well".**

What the game has today, checked on `main` at `d1191ded`:

- **Inner tubes are not built.** No craft, good, recipe, model or doc names
  one (`grep -rli "inner.\?tube" crates assets docs openspec` finds nothing).
- **No boat is made.** The craft kinds are the Tern, the Loon (canoe), the Cog
  and the harbour's Rowboat (`pbd_core::vehicle::Kind`). A new world places
  the Tern and the Loon at anchor near the player (`vehicles/place.rs`), and
  harbours moor their own boats (`vehicles/harbour.rs`). `recipes.ron` has no
  boat, raft or tube.
- **No raft exists**, so nothing hauls.
- **Fishing is on foot only.** `fish.rs` reads the walker's feet and the
  walking camera and winds the line in the moment the player boards
  ("Only the rod fishes, only on foot"). The tool picker (G) and the pack
  do not open aboard.

## What changes

1. **Two new craft, and the rowboat made by hand:**
   - the **inner tube** (`tube`), a ring you sit in and paddle by hand;
   - the **raft** (`raft`), logs lashed side by side, poled with a paddle;
   - the **small rowboat**, the harbour's existing `Rowboat` craft.

   All three are paddled with the Loon's stroke, on hulls of their own.
2. **Recipes:**
   - INNER TUBE, RAFT and ROWBOAT, all at the workbench.

   Each gives a boat as a good in the slots.
3. **Launching and taking up.** Holding a boat good, the place button on water
   in reach puts the craft on the water. Holding the dig button on a boat you
   made, with nobody in it and its hold empty, takes it back up into the slots.
4. **The raft's hold.** Ten slots, opened with the pack aboard the raft or
   standing beside it, and moved with the pack's own clicks. Each move is one
   durable line: the hold and the slots together.
5. **Fishing afloat.** Aboard the tube, the raft, the canoe or the rowboat:
   - G picks the rod;
   - the left button casts and the right reels, exactly as on foot;
   - the line leaves from the seat;
   - the paddler can still paddle with the line out.

   A cog's deck is fished from on foot already; the Tern's hands are on the
   sheet and the tiller, so it does not fish.

## Out of scope

- **Tugs and barges**, which need a dock (owner).
- **A hold's weight on the raft's float.** The hold carries goods as
  inventory; its mass is not added to the hull in this change.
- **Starter missions for boats.**
- **A mockup.** This is a game change on the existing craft, drawn with the
  towns' boat pieces and simple hexel-free meshes; there is no new look to
  approve beyond the captures.

## Saves

- New craft kinds are new strings (`tube`, `raft`); no old record changes.
- New record fields are written only when set:
  - `made`, for a boat the player made;
  - the hold, as its own `hold` record.
- New goods are appended to the goods table; the recipes are new codes.
- No generator, terrain or topology change.
