# Proposal: industry tiers, oil at sea, freighters and the carriers

Status: **planning only**. Nothing here is built. The mockup is M20, Pits,
Plants and Power (`docs/mockups/industry.html`, on the tracker). M20 had no
change of its own; its oil was written up in `steer-and-sail` (decisions 5
and 6), which keeps it. This change takes everything else on the page from
here on.

## Why

The owner, 2026-11-14, verbatim, with five reference pictures (a BuildCraft
quarry frame over a pit, the Bagger 288 bucket-wheel excavator, a pumpjack, an
offshore oil platform in a storm, a container ship):

> Pits, plants and power:
>
> different levels of strip mining
>
> * humans digging (you hire townsfolk)
> * an excavator
> * quary (buildcraft/minecraft mod)
> * super excavator
> * mining drones
>
> oil drilling:
> -oil rig
> -oil platform
>
> * frieghters (hired humans, evenetullay automated)
>
> keep in mind for the power system, 3d printers, mining setups, conveyor
> belts, launch pads, fuel systems

and, in the same session:

> oil platforms are kind of both a power plant and a rig in one

> be sure to add pipes, pneumatic tubes and conveyor belts, and wires to
> transport resouirces like water, electricity, fuel and goods

## What changes (once built)

- **Five tiers of strip mining**, each a different author of the same edits
  on marked cells: a hired crew of townsfolk, a diesel excavator, a quarry
  frame, a bucket-wheel excavator, and a hive of mining drones. Each later
  tier is faster and reaches harder rock; none replaces marking.
- **Oil, on land and at sea**: a drilling rig that drills a well and leaves a
  pumpjack on it; an offshore platform that is a rig and a power plant in
  one (it drills several wells on the shelf sea, burns their gas in its own
  turbines and sends the surplus ashore down a sea cable).
- **Freighters**: ships on stored routes that carry goods and crude between
  docks, crewed by hired townsfolk at first and automated later. The same
  route record the ship-tiers change uses for space freighters.
- **Four carriers** for everything that moves: wires (electricity), pipes
  (water, crude, fuels, propellant), conveyor belts (bulk: ore, coal, stone,
  crates) and pneumatic tubes (small goods: parts, plates, circuits).
- **The power ladder's loads**, so the grid is planned for them: 3D printers
  (the construction yard, M22), mining setups, conveyors, pipe pumps and
  tubes, launch pads (M21) and the fuel system from the column to a pad, a
  dock or a filling station.

## What does not change

- The oil field, pumpjack, column and oil plant stay as `steer-and-sail`
  decision 5 has them; this change adds to them.
- Every dug cell stays an edit on the durable path, now naming a crew or a
  machine as its author (CLAUDE.md, "The player is not the only author of
  change"). Nothing digs what is not marked.
- World time runs only while the world is played; wages, fuel and power are
  spent only then.
- Saves: crews, machines, wells, platforms, routes and carrier networks are
  stored records; the oil field stays derived. Nothing here changes
  generation, so old worlds keep every cell.

## Impact

- Specs: a new `world/industry` capability (a delta in this change, none of
  it true yet).
- Mockups: M20 gains the mining tiers, the drilling rig, the offshore
  platform, freighters and a Transport section; its power grid gains the new
  loads and the platform.
- Mockups it touches by reference: `townsfolk` (who is hired), M22
  construction (the printer's load), M21 ship-tiers (pads, propellant, space
  freighters), M23 drydock (the ships a freighter is).
- Code: none until asked (CLAUDE.md, "Write it up before touching code").
