# Design: early power

Everything here is proposed. Numbers are first guesses to tune, not
measurements of the game. Where nothing can be compared side by side, the
recommendation is taken (CLAUDE.md, "Ask only with screenshots") and marked
so; the owner judges the tree and the mills on M33's POWER and MILLS tabs.

## Context

- The starter tree (`starter-missions`) ends its mining branch at "The iron
  age": iron and copper ingots come from the forge (`mixed-ore`, 2 copper : 1
  iron : 1 tin per 4 ore).
- The town tree (`town-missions-tree`) builds the **grinder** (stone, planks,
  2 iron) and the **mixer** (`townsfolk` design 41). Both are stations whose
  recipes run on world time once queued.
- The wind at a place is `pbd_core::wind::wind_at` (the atmosphere's mean,
  sheared with height, with gusts); a river's flow is sampled by
  `log_drive`'s water query (`water-flow`).
- M20 (`industry-tiers`) has, at tier II, wind turbines rated 150 kW (cut in
  3 m/s, rated 12, cut out 25; on a rise: the cell 2 layers above every cell
  within 3) and a hydro dam (5 kW a percent of flow); and wire on wood poles
  4 cells apart, 2,000 kW a line.

## Goals / Non-Goals

**Goals**: a small tree that teaches mechanical power (the windmill), then
electric power (the water mill), lines and appliances, with the lesson that
power is used as it is made. It leads into M20's tier II without changing
any of its numbers.

**Non-Goals**: batteries or any storage (owner: "no battery storage for
awhile"); electric windmills (they come with M20's turbines); a grid tier
past this one; anything that changes an existing world.

## Decisions

### 1. A tree of its own, opened by the iron age

POWER is a third tree in the Missions screen, as the town tree is (its own
file `power_missions.ron`, its own saved record `power_missions`, its own
tab). It opens when the starter tree's `iron` ("The iron age") is done,
because the first mission needs an iron axle. Opening it unlocks the two
recipes its first mission asks for (`wooden-gear`, `axle`). No mission of one
tree is a parent of another tree's; a mission that needs another tree's
station says so in its help line and waits on it with a hidden `after` gate
across trees (`after: ["town:grinder"]`), the one new rule here
(recommendation taken).

### 2. The tree

A strict tree, as the starter tree's rules have it: one parent each (the
only line drawn), hidden `after` gates for extra needs, unlocks as rewards.
Objective kinds marked **new** are not raised by the game yet.

| id | Branch | Mission | Parent (after) | Objectives | Unlocks |
| --- | --- | --- | --- | --- | --- |
| `gears` | 0 Millwright | Cut gears | (tree opens) | craft 4 `wooden-gear`; craft 1 `axle` | `millstone`, `copper-wire`, `dynamo` |
| `millstones` | 0 Millwright | Dress millstones | `gears` | craft 2 `millstone` | `windmill` |
| `windmill` | 1 Wind | Raise a windmill | `millstones` | craft a windmill; set it on a rise (**new** kind `rise`) | `flour` |
| `grind` | 1 Wind | Grind grain | `windmill` | grind 12 wheat at the windmill (**new** kind `grind`) | `loaf` |
| `loaf` | 1 Wind | Bake a loaf | `grind` | cook 2 `loaf` at the campfire | `feed-mash` |
| `mash` | 1 Wind | Mash for the herd | `loaf` | grind 6 `feed-mash`; feed it to the herd 3 times | - |
| `wind-grinder` | 1 Wind | Stone by the wind | `windmill` (`town:grinder`) | set a grinder on the mill floor (**new** kind `drive`); crush 9 stone with it | - |
| `dynamo` | 2 Water | Wind a dynamo | `gears` | craft 8 `copper-wire`; craft a `dynamo` | `water-mill` |
| `water-mill` | 2 Water | Build a water mill | `dynamo` | craft a water mill; set it on flowing water (**new** kind `flow`) | `power-pole` |
| `turn` | 2 Water | Turn the wheel | `water-mill` | make 6 kWh at the mill (**new** kind `power`) | - |
| `poles` | 3 Lines | Set poles | `turn` | craft 4 `power-pole`; set 4 in a run from the mill (**new** kind `pole`) | `electric-lamp` |
| `line` | 3 Lines | String a line | `poles` | join the mill to a pole 12 cells away (**new** kind `line`) | `motor`, `bellows` |
| `lamps` | 3 Lines | Lamps on the line | `line` | light 3 lamps from the line at night | - |
| `sluice` | 3 Lines | When the wheel stops | `lamps` | shut the sluice at night (**new** kind `sluice`); see the lamps go out | - |
| `motor-grinder` | 4 Appliances | A grinder on the line | `line` (`town:grinder`) | fit a motor to a grinder; crush 20 stone on the line | - |
| `bellows` | 4 Appliances | Bellows at the forge | `motor-grinder` | fit bellows to a forge; smelt 4 ingots on the line | - |
| `motor-mixer` | 4 Appliances | A mixer on the line | `bellows` (`town:mixer`) | fit a motor to a mixer; mix 16 gravel on the line | - |

The **sluice** mission is the tier's lesson on purpose: there is no battery,
so shutting the wheel at night puts the lamps out at once.

### 3. Recipes, from the game's goods

New goods and blocks are proposals; every input is a good the game has.

| Code | Station | Secs | Inputs | Outputs |
| --- | --- | ---: | --- | --- |
| `wooden-gear` | workbench | 3 | 3 planks | 1 wooden gear |
| `axle` | workbench | 4 | 1 log, 1 iron-ingot | 1 axle |
| `millstone` | workbench | 6 | 2 block:stone, 4 stone | 1 millstone |
| `windmill` | workbench | 10 | 24 planks, 6 log, 4 rope, 4 wooden gear, 1 axle, 2 millstone, 6 fibre (thatch) | 1 block:windmill |
| `flour` | windmill | 6 | 3 wheat | 2 flour |
| `feed-mash` | windmill | 6 | 1 wheat, 1 turnip | 2 feed mash |
| `loaf` | campfire | 20 | 2 flour | 1 loaf |
| `copper-wire` | workbench | 2 | 1 copper-ingot | 4 copper wire |
| `dynamo` | workbench | 8 | 8 copper wire, 2 iron-ingot, 1 axle | 1 dynamo |
| `water-mill` | workbench | 10 | 20 planks, 4 log, 2 wooden gear, 1 axle, 1 dynamo, 8 block:stone | 1 block:water mill |
| `power-pole` | workbench | 2 | 1 log, 2 copper wire | 1 power pole |
| `electric-lamp` | workbench | 3 | 1 iron-ingot, 1 copper wire | 2 electric lamp |
| `motor` | workbench | 6 | 6 copper wire, 2 iron-ingot | 1 motor |
| `bellows` | workbench | 4 | 2 leather, 3 planks, 1 motor | 1 bellows |

- **Flour and the loaf.** A loaf is a dish with the flatbread's effect and
  twice its fill (`cooking.ron`, recommendation taken). The flatbread from
  wheat stays as it is.
- **Feed mash** is a feed for every kept animal (`husbandry.ron`'s feed
  list); fed mash, an animal's produce is ready in half its time for that day
  (recommendation taken).

### 4. The windmill: food, wind and a rise

- **Mechanical, not electric.** The windmill's shaft turns its millstones or
  a grinder set on its floor (one at a time). It is never on a line in this
  tier: electric wind is M20's turbine, tier II.
- **Footprint**: 1 cell, 4 layers tall, a timber tower on a fieldstone foot,
  four sails 9 m across that turn to face the wind.
- **On a rise**, M20's turbine rule: the cell stands 2 layers above every
  cell within 3, and the sails are clear of trees within 2 cells. Placing
  elsewhere is refused with "Needs higher ground" or "Trees in the way".
- **Wind**: the 10 m wind at the mill (`wind_at`, gusts in it). Below 3 m/s
  the sails stand; from 3 to 8 m/s the work runs at (v - 3) / 5 of its speed;
  8 to 20 m/s at full speed; above 20 m/s the sails are furled and stand.
  Recipes take `secs / share` of world time. Calm stops the queue, it does not
  empty it.
- **A grinder on its floor** runs on the shaft instead of by hand: the town
  grinder's recipes at the same wind share (and twice as fast at full wind,
  recommendation taken).

### 5. The water mill: hydro power by the river's flow

- **Placed** on a river cell whose surface flow is at least 0.4 m/s
  (`water-flow`'s flow sample), the wheel in the water, the house on the bank
  (2 cells, 2 layers). Placing on still water is refused: "Needs moving
  water".
- **Output**: `rated_kw * clamp(flow / full_flow, 0, 1)`, rated 6 kW at a
  flow of 1.2 m/s or more. A frozen river gives 0. Rain that raises the flow
  raises the output, as M20's dam does.
- **The sluice** (F at the house) lifts the wheel out of the race: 0 kW
  while shut. It is the only way to stop a mill short of breaking it.
- **Leads into M20**: 6 kW a mill against M20's 500 kW dam at full flow and
  150 kW turbine; the dam is the same rule (flow) at a step it needs.

### 6. Lines

- **Poles** are set 4 cells apart at most (M20's wood poles, the same
  model). A pole joins every pole within 4 cells; a mill, appliance or lamp
  joins a pole within 1 cell. One network per joined run.
- **Capacity**: 20 kW a light line (copper wire), far above what this tier
  makes, so it never limits here; M20's heavier wire on the same poles
  carries 2,000 kW later. No losses in this tier (recommendation taken).
- **Saved** as records: each pole a record naming its author; the network is
  derived from the poles and never saved.

### 7. No batteries: power is used as it is made

- **Each tick**, a network's supply is what its mills make that tick, and its
  demand is what its running appliances draw. Nothing is stored.
- **Supply at or above demand**: everything runs at full speed. **Below**:
  every appliance on the network runs at `supply / demand` of its speed, and
  every lamp dims to that share. **Zero**: everything stops, lamps go dark,
  queues wait (nothing is lost).
- One rule in `pbd-core`, run by the server in multiplayer, so every client
  sees the same lamps dark (CLAUDE.md, one authoritative implementation).

### 8. The first appliances (proposals)

| Appliance | Draws | What it does on power |
| --- | ---: | --- |
| Electric lamp | 0.1 kW | A light where it stands, in the cell light field like a lantern; dims with the share |
| Motor on a grinder | 2.0 kW | The grinder's recipes in half the time |
| Motor on a mixer | 2.5 kW | The mixer's recipes in half the time |
| Bellows on a forge | 1.5 kW | The forge's recipes in 0.6 of the time, coal as before |

One mill (6 kW at full flow) runs a motor grinder, a forge's bellows and
twenty lamps (5.5 kW); add the mixer and the line asks 8 kW of 6, so
everything on it slows to 75% and the lamps dim to match.
That is the shape on purpose: a second mill or a choice of what to run.

### 9. `assets/config/power.ron`, the proposed shape

```ron
(
    windmill: (
        cut_in_m_s: 3.0,      // sails stand below
        full_m_s: 8.0,        // full speed from here
        furl_m_s: 20.0,       // furled above
        rise_layers: 2,       // above every cell within rise_radius_cells
        rise_radius_cells: 3,
        tree_clear_cells: 2,
        grinder_speed: 2.0,   // a grinder on its floor at full wind
    ),
    water_mill: (
        min_flow_m_s: 0.4,    // placement
        full_flow_m_s: 1.2,
        rated_kw: 6.0,
    ),
    line: (
        pole_reach_cells: 4,
        join_reach_cells: 1,
        capacity_kw: 20.0,
    ),
    appliances: [
        (code: "electric-lamp", draw_kw: 0.1),
        (code: "motor-grinder", draw_kw: 2.0, time_scale: 0.5),
        (code: "motor-mixer", draw_kw: 2.5, time_scale: 0.5),
        (code: "bellows", draw_kw: 1.5, time_scale: 0.6),
    ],
)
```

Validated at load: speeds ascending (`cut_in < full < furl`), every draw and
capacity positive and finite, every `time_scale` in (0, 1]. Zero is a valid
value, not a sentinel; nothing falls back silently.

### 10. Game text

Plain and short, from the locale file: "Needs higher ground", "Trees in the
way", "Needs moving water", "No wind", "No power", "Sluice shut". Help lines
as the mockup's cards have them ("Set the windmill on a rise. It needs the
wind.").

### 11. Saves

The tree's ids are a new record, append-only; no world from before has it,
so it opens at its root when "The iron age" is done in that world, and a world
that already has iron opens it on load. Mills, poles and appliances are
stored records naming their author. Nothing existing changes.

### 12. The mockup

M33 version 10:

- **POWER** (`#power`): the tree as the Missions screen shows it, with its
  bot run (Step and Play, off until pressed), opening partway through
  "String a line"; the RECIPES side tab lists each unlock with its inputs.
- **MILLS** (`#mills`): one 3D view, made when the tab first opens, stopped
  when it closes. A windmill on a rise and a water mill on a river joined by
  a pole line with lamps; by day, or at night lit by the line's lamps and the
  mills' windows, with a dim moon. The settlement set's textures (plank,
  timber, fieldstone, thatch, iron, grass, dirt) sampled nearest; sails, wheel
  and wire cloth from `tools/art/early_power_art.py`. Sails and wheel stand
  still until **Turn** is pressed; **Sluice** shuts the wheel and puts the
  lamps out (no batteries).
- Stand-in icons from `tools/art/early_power_icons.py`.

## Risks / Trade-offs

- A cross-tree `after` gate is new to the tree's rules; the alternative, a
  copy of the grinder mission in this tree, double-counts it.
- Mechanical-only windmills may read as odd beside M20's electric turbines;
  the owner's words put food on the windmill and power on the water mill.
- 6 kW is small; it is meant to make a second mill worth building.

## Decisions taken

- The tree, its parents and its recipes: recommendation taken (ask only with
  screenshots); the owner judges them on the POWER tab.
- Windmill food, water mill power, lines, no batteries: the owner's words.
- Every number in decision 9: recommendation taken.
