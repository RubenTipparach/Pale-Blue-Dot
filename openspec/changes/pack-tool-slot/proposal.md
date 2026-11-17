# Proposal: the hand in the pack, and every recipe count shown

## Why

The owner, 2026-11-17, on the pack screen:
- "this crafting number flickered and disappeared, this needs fixing": the
  stone axe's line shows no count on its log;
- "I also need a way to swap equipment in the hot bar", and "my hand /current
  equipment needs to show up here" (the pack's HOTBAR row).

## What Changes

- A recipe's input well always shows its count, 1 included. Result and pick
  wells keep a stack's rule (no number for one). Same for the forge's and the
  campfire's input rows.
- The pack's HOTBAR row gets the hand cell at its left, the same cell the HUD
  shows beside the hotbar: the tool in hand, or the bare hand.
- Clicking it opens a short list beside it: BARE HANDS, then each tool owned.
  A pick holds that tool, the same path the G picker takes, and is saved the
  same way.

## Impact

- `desktop/craft.rs`, `forge.rs`, `kitchen.rs` (counts), `desktop/pack.rs`,
  `desktop/equipment.rs` (the shared cell and the list).
- Saves: none new. The `hand` line already holds the tool in hand.
