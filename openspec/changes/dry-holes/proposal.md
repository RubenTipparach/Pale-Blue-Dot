# Proposal: dry holes, and a sea that flows in

## Why

The owner, 2026-11-18: "We should be able to dig holes below water line and
not have it filled with water too". And straight after: "Ocean needs to work
like rivers. In the sense that the water flows."

A hole dug on land whose floor goes under the sea's level, and that touches
no water, should stay dry: no water in it, drawn or simulated. Water should
only enter a dug hole through a neighbouring water cell, and the sea should
come in the way a river's water does: along a channel at the rule's pace,
pouring over the lip into a pit it is let into.

**Finding (design 1).** The cells of such a hole are air, before and after
the rule runs and after a reload: neither the liquid rule nor the sea's
flood puts water in them. What fills it is the **sea's reach**
(`ColumnTier::sea_reach`, `ocean-environments` B22), the walk that decides
where the sea's analytic sheet is drawn. It walks from the open sea through
air up to the high-tide line (1.5 m over the mean sea) and down again
through any air under it. Low ground near the coast (a beach at +1 m, a
river bank near the sea) joins a pit dug beside it to the sea through the
air over that ground, and the walk then runs down the pit however deep it
goes. The pit is marked reached, and three readers believe it:
- the sheet is listed over it and drawn at the tided sea level;
- the eye in it is "under the sea" (blue fog, distortion);
- the walker swims in it.

The column holds air the whole time: the water is a picture painted over a
dry hole.

Today the sea also comes in all at once: an edit that opens a cell to the
sea fills everything joined to it under sea level in the same frame
(`flood_from`), with no flow and no fall at the lip.

## What changes

- **Dug holes are dry until water flows into them.** The sea's reach never
  runs down through air under the mean sea level in a column a player has
  dug or built in. Such a column is the sea's only when the sea's water
  stands in it at the sea's top layer. A dug pit that joins no water is dry
  at every depth: no sheet, no underwater view, no swimming. Generated caves
  and pits behind a beach keep B22's look (the sheet over them at high tide).
- **The sea flows in by the water rule.** The edit no longer floods. The
  cell the edit opened fills at once if the sea's own water is beside or
  over it; the rest is the liquid rule. A new clause makes the rule's water
  the sea's below the sea's level: where the sea is joined to a cell under
  its level, water that stands there fed by a source or a fall becomes a
  source. So:
  - a channel cut from the shore fills cell by cell, a cell a tick;
  - a pit whose wall is breached fills by the sea pouring over the lip as
    falling water (the falls' look unchanged), then filling up from the
    floor until it stands at sea level;
  - water poured into a sealed dry pit is still only the bucket's.
- **The far sea is unchanged.** The open sea's surface, its waves and tides
  stay analytic. The tier's build still floods what generation opened onto
  the sea, as before.

## Impact

- `pbd-core/src/liquid.rs`: the rule's sea clause (`Grid::sea`,
  `transform_at`); `transform` unchanged for every caller and fixture.
- `pbd-app/src/planet_column.rs`: `sea_reach` (dug columns), the sea's
  joined set (`TierWater::joined`, `dug`), `water_after_edit`.
- Saves: no format change and no generator version. The rule's sea cells
  are saved as edits authored by the water, like any flowing cell. An old
  save's holes beside the sea are flooded at the build exactly as before.
- Rendering: no shader change. The sheet's listing reads the reach mark it
  already reads.
- Not changed: how falling water looks (CLAUDE.md, "Waterfalls are designed
  in a mockup first"); the open sea's sheet and tides; rivers.
