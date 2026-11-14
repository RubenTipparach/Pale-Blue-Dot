# Design: Luanti's liquid rule on the hex grid

## Context

**Luanti's rule** (`src/servermap.cpp` at `a10dc8d`,
`transformLiquidsLocal` and `get_max_liquid_level`):
- **Node states.** A node is a liquid *source*, a *flowing* node with a
  3-bit level (7 nearest the source, 0 the thinnest film) and a "flowing
  down" bit, or something else. That something is either *floodable* (air)
  or *neutral* (solid, or another liquid).
- **The queue.** Only nodes in a queue are looked at. An edit queues the
  node, and a changed node queues its neighbours. A tick takes at most the
  queue's length at its start (`liquid_update`, 1 s; `liquid_loop_max`,
  100,000), so water spreads about a node a second.
- **One node's update.** It reads its six neighbours: up, four sides, down.
  - A source stays a source.
  - A node with **two or more same-level or upper sources around it** becomes
    a source if the liquid is renewable. The source below is not counted.
  - Else a node beside a source takes level 7.
  - Else it takes the highest level its neighbours offer:
    - a flowing node above gives its level plus 4 (`WATER_DROP_BOOST`),
      capped at 7;
    - a side neighbour gives its level less 1, if it is not itself falling.
  - It is liquid only if that level is within `liquid_range` (8, so levels
    0 to 7 all count); otherwise it is air again.
- **Falling first.** "Flowing down" is set when the node below is air or
  flowing. A falling node offers nothing sideways, so water drops before it
  spreads.
- **Viscosity** (water 1, lava 7) slows how fast the level changes.

**Here today**: water is one static material at sea level (proposal).

## Decisions

**1. The hex neighbourhood is eight: up, six sides, down.**
- Luanti's order is kept: upper before same level before lower.
- A pentagon cell has five sides. The arrays hold eight and count what is
  there, as the topology's explicit 5/6 neighbour tables already do.

**2. Renewing takes three sources on the hex grid, not two.**
- On Luanti's square grid, two sources side by side share no neighbour, so a
  pair does not spread.
- On the hex grid, two adjacent cells share two neighbours. With Luanti's
  threshold of two, each shared cell becomes a source, which makes new
  pairs, and any two sources poured side by side flood every floodable cell
  on their level. The mockup shows this runaway (the preset "Luanti's
  threshold on hex").
- With three: a cell beside a straight row of sources touches two of them,
  and a cell beside a triangle of three touches at most two, so neither
  grows. A cell in a concave corner touches three and fills. This is the
  behaviour Luanti's two-of-four gives on squares: pools fill their corners,
  and lines stay lines.
- Recommendation taken (ask only with screenshots). The mockup has the
  toggle, so the owner can see both.

**3. Range and levels stay Luanti's: 8 levels, range 8.**
- Seven cells of spread is 20 m on the hex grid (2.833 m cells), where it is
  7 m in Luanti. That is a gentler sheet of water for the same 1 m layer.
- The mockup has a range slider (1 to 8) to judge it. Recommendation taken:
  8, Luanti's default, until the owner says otherwise from the mockup.

**4. A flowing cell's surface stands at (level + 1) / 8 of its 1 m layer.**
- That is Luanti's own drawing rule. A source and a falling cell are full.
- On the hex grid, each cell's top is one flat hexagon at that height, so a
  spreading sheet steps down by eighths. Luanti instead slopes its corners
  between neighbours, which is not a cell look.

**5. Only edits start flow; old worlds do not move.**
- Generated water is all source and is never queued. The queue starts only
  from an edit: a dig, a placed block, or a poured or removed source.
- So opening a world never changes it. A world made before this change has
  the same water cell for cell until someone digs beside it, and then only
  that water moves (CLAUDE.md, "Saved games survive every change").
- No generator change, so no new generator version.

**6. Sources are saved; settled flowing water is derived.**
- Placing or removing a source is a world edit. It takes the durable path
  and names its author. A source the renew rule makes is authored by the
  water. The rule only floods floodable cells (air), never a player's block,
  so the world's change never overwrites the player's.
- Flowing cells are not saved one by one. On load, every saved source
  beside air is queued, and the flow re-runs to the same settled state.
- This relies on settling being a function of the sources and the solids
  alone. The mockup checks it: "Settle from scratch" re-runs every source
  and compares the result with the incremental state, cell for cell.
- Water still moving when the game closes picks up where the sources put
  it. Nothing a player made is lost; at worst a falling sheet restarts.

**7. Save format, rendering and swimming (later, not in the mockup).**
- Save format: flowing water needs its level. Either a material per level or
  a level field. A new save format version is needed, with the old one
  readable: its water is all source.
- Rendering: the water top is today the top of the column's water run
  (`Column::water_surface`, the shader's water word). It must read the
  flowing level instead. That is a shader interface change, so the pipeline
  is validated when it lands.
- Swimming: the player swims in flowing water as in a source. Boats float
  on the sea's table only, as now.
- Cost: the queue is bounded per tick like Luanti's. It is measured on real
  hardware before it ships (perf rule).

**8. The game's code is written from the rule, not translated.** The mockup
is a line-by-line port of LGPL-2.1-or-later code and says so. The Rust
version is written from decisions 1 to 6 and Luanti's rule as described in
Context, and the mockup is the reference for behaviour. Recommendation taken
(ask only with screenshots). The owner can choose to take the LGPL code into
the game instead, and that is the owner's call.

## Risks / Trade-offs

- **Infinite water.** Renewing makes infinite water, as in Luanti. With
  three sources it needs a real pool, not a pair, to start.
- **Large floods are slow.** A flood moves at a cell a second, by design. A
  dam break across a valley takes minutes, which is the look Luanti players
  know.
- **The sea is a source everywhere.** Dig a channel from the sea and it
  fills to sea level, then flows on down any slope below it. Draining the
  sea is impossible, which matches the sea's rails.

## Migration Plan

- Mockup first (tasks section 1). The game waits for the owner's approval of
  the mockup, then a request (CLAUDE.md, the mockup tracker).
