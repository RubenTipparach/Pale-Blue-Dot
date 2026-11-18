# Proposal: a toast for every item gained

## Why

The owner, 2026-11-18: "when getting items from digging dirt or sand
(theres missions that want you get bait or grass tufts) you should have a
toast that pops up telling you an item was rewarded, when showing item
rewards, make sure to show the icon sprite too, multiple items stack them,
like dirt x1, dirt x2 etc, toasts should appear for about 3 seconds then fade
out, also vertically stack different items too, up to like 10 items, then
page them".

## What Changes

- A toast for each item that comes into the pack: its icon and "Dirt x1".
  Another of the same item while its toast is up adds to it ("Dirt x2") and
  holds it up again.
- Each toast stays about 3 s and fades out. Different items stack in a
  column, at most ten at once; more wait and come in as others fade.
- The short "Found bait", "+1 log" and "+3 apples" notes are dropped: the
  toast says it, with the picture.

## Impact

- `hotbar.rs`: a `PickedUp` message for a dug block the magnet brings in
  (finds, harvests, picks, crafts and mission rewards already raise `Gained`).
- `drops.rs`: raises it on a pickup the save accepted.
- `desktop/toasts.rs` (new): the stack, painted from `slots::paint_icon`.
- No save change: toasts are a view of what was already saved.
