# Design

## Measured

- Every gain but one raises `hotbar::Gained` after its durable write: finds
  when digging (seeds, bait, flint), tilling, picking, harvesting, crafting,
  cooking, collecting from animals and mission rewards.
- The one that does not: a dug block. It floats where it was cut
  (`inventory-grid` decision 4) and `drops::gather` puts it in the pack when
  the magnet brings it in; nothing is raised there.
- Notes (`FarmNote`) say some gains in text ("Found worm", "+1 log").

## Decisions

1. `PickedUp { item, count }` is raised by `drops::gather` for each pickup
   the save accepted, with the count taken.
2. The toasts read `Gained` and `PickedUp`. One list: a toast per item,
   oldest first. A gain of an item whose toast is shown adds to its count and
   restarts its time; otherwise a new toast joins the end.
3. Shown for 3 s, then a 0.4 s fade. The first ten toasts are shown; the
   rest wait, and their time starts only when they come into the ten.
4. Placed at the right of the screen above the hotbar's line, newest at the
   bottom. Each row is the item's 16 px icon (the slot's own, by
   `slots::paint_icon`; a tool's from `ItemIcons::tool`) and its name and
   count ("Dirt x2"). Plain text, no notes.
5. Toasts show only in play; a menu over the world hides them, and their
   time still runs.
6. The gain notes ("Found {}", "+1 log", "+N {fruit}") go; the notes that say
   why something did not happen stay.
