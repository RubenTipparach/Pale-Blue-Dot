# Design: gathering the sea floor (sea-gathering)

## Context

The proposal lists what exists. This design says what can be taken, how,
what it gives and what it is for. Every choice the owner did not make is
marked **recommendation taken (ask only with screenshots)**: none of them is
a look the owner has to judge before it is built, since every model, colour
and light is the reef's own, already approved in `ocean-environments`.

### Why a change of its own, not a section of `ocean-environments`

`ocean-environments` is the sea's places and life as they are drawn and
generated: 3,000 lines of design about light, zones, models and generator
versions. Gathering is a play loop that crosses farming (fertiliser),
the economy (prices), cooking (dishes), placement (ornaments) and the
save (two record kinds). It changes nothing about how the sea is drawn or
generated. As its own change its tasks, specs and archive stand on their
own; it builds on `ocean-environments`' decisions D5 (living coral and
kelp are pieces; reef rock and brain-coral blocks are cells) and D9
(protected creatures are refused), which it cites rather than reopens.

## Decisions

### 1. What can be gathered: the models the sea already draws

One item per model (owner: "Reefs have specific models ... collect and
sell them"). The item is named after the model, not after a generic
"coral".

| Model (`places::Kind`) | Where | Item (code) | Regrows, world days | Value, coins | Grinder | Ornament |
| --- | --- | --- | ---: | ---: | --- | --- |
| `Branch` | reef | `staghorn-coral` | 4 | 5 | 1 to 2 fertiliser | yes |
| `Finger` | reef | `finger-coral` | 4 | 5 | 1 to 2 | yes |
| `Boulder` (on a reef) | reef | `brain-coral` | 8 | 7 | 1 to 3 | yes |
| `Table` | reef, 3.5 m down | `table-coral` | 8 | 9 | 1 to 3 | yes |
| `Soft` | reef | `soft-coral` | 3 | 4 | 1 to 1 | yes |
| `Tube` | reef | `tube-sponge` | 3 | 4 | 1 to 1 | yes |
| `Barrel` | reef | `barrel-sponge` | 5 | 6 | 1 to 2 | yes |
| `Fan` | reef | `sea-fan` | 4 | 6 | 1 to 1 | yes |
| `Anemone` | reef | `sea-anemone` | 3 | 5 | 1 to 1 | yes |
| `Turf` | reef | `sea-lettuce` | 2 | 1 | 2 to 1 | no |
| `Rubble` | reef | `coral-rubble` | 2 | 1 | 2 to 1 | no |
| kelp stand | kelp bed | `kelp` | 2 | 2 | 2 to 1 | no |
| `Mussels` | round a vent | `mussels` | 3 | 3 | no | no |

**Not taken**, each argued:
- the big anemone (`Hero`): it is the clownfish's home, drawn with them,
  one reef tile in three; taking it would take the fish too;
- a kelp bed's rocks (`Boulder` laid by the kelp rule): rock, not coral;
- the black smoker and the bacterial mats: a chimney of rock and a film
  on it, not something a hand takes;
- floes: the sea ice is cells on a version-14 world, dug as cells;
- reef rock and brain-coral **blocks** (D5): cells, dug as ground already.

The bed's animals are the crawlers the game has (`Mover::Crawler`): the
shore crab, the lobster, the giant clam, the octopus and the tube worm.
A taken one is the `Item::Fish` of its species, the same item a catch is,
so its icon, its trade value (`fish:<id>`) and its cooking (any fish at
the campfire) are the ones the game already has.

**Not in the world yet**, so not gathered: sea urchins, sea cucumbers,
starfish, scallops, oysters and abalone. Each needs a body in
`fish::bodies` and a species row before it can be taken. They are the
next rows of this table, a separate request. Recommendation taken (ask
only with screenshots).

### 2. By hand, while swimming or wading

Every row is taken by hand, F, the empty or any hand: there is no knife or
net in the game, and the reef is reached by swimming, where the tools are
not swung. Recommendation taken (ask only with screenshots).

**Aim**: the piece as it is drawn. Each gatherable piece in a tile built at
the near level carries its foot, its up and its drawn height and width.
F offers `Gather <item>` while the eye ray passes within the piece's
half-width (at least 0.2 m) of the segment from its foot to its top, within
2.5 m of the eye; the nearest along the ray wins. A crawler is aimed the
same way, as a 0.35 m sphere round where it is drawn. The offer names the
item ("Gather staghorn coral", "Gather lobster").

A far-level tile's pieces are not offered: the player is never within
reach of one (the near level reaches 12 m).

### 3. A take is a stored record; the piece is gone and grows back

Pieces are derived (`plan`), never saved; a take is. Each piece gets an
**id**, a 64-bit hash of the tile, its lattice point and its kind, worked
out by the same `plan` that places it, so the same piece has the same id
in every build of the tile and every session. A take writes

`sea-take <id> 1 (day: <world day>, code: "<item>")`

with the pack after it, in one journal line (`store_with_hands`), and is
acknowledged (the item toast, the piece gone) only once the save accepts
it.

**Gone on that frame.** The tile the piece stood in is rebuilt from its
own kept pieces, less the taken one, and swapped in at the fade it had, so
nothing else in the tile moves, fades or appears. The piece is left out
**after** the tile's trim to its budget, so a take never lets a piece the
budget had left out take its place.

**Grows back** after its kind's days of world time (the sun's clock, which
runs only while the world is played: CLAUDE.md, world persistence), the
next time its tile is built: a tile is built as it comes back into reach
(52 m), so a piece grows back out of the player's sight. A take that comes
due while its tile stands is not drawn again until then, not even when
another take in the same tile rebuilds it: the rebuild keeps out what its
build kept out. Recommendation taken (ask only with screenshots): the
regrowth days are the table's, wild plants' two days for weed and kelp,
up to eight for the slow, big corals.

A crawler is not stored: taking it removes it from its school, and the
spawner fills the water again by its own rule, as a caught fish's school
does.

### 4. Protected creatures are refused (D9)

A reef turtle, a sea otter, a seal, a duck, a beaver or a river otter
aimed at offers `Gather <name>`, and F on it says the game's existing
refusal (`fauna.refusal.protected`, "Protected") and takes nothing. The
rule is the core's (`sea_gather::hand_refusal`): a protected species is
refused whatever reaches it; a crawler is taken; a schooling fish is
caught with the rod, not the hand.

### 5. Selling: a value per kind

Every new good has a row in `values.ron` (the table above, and
fertiliser 2, steamed mussels 8, kelp crisps 5). Corals, sponges, fans and
anemones are `goods` (decoration a town buys and sells as made things),
sea lettuce and kelp `crop`, mussels `fish`, fertiliser `goods`, the dishes
`dish`. Every town trader already buys any good with a value in a traded
group at its town's price, so nothing in the trade code changes.
Recommendation taken (ask only with screenshots).

### 6. Crushing: the grinder makes fertiliser

There was no fertiliser, so this defines one (owner: "crush them for
fertilizer"). **`fertilizer`**, a good, is made at the grinder (the one
station that crushes) from each reef piece, kelp and sea lettuce, one
recipe a kind (the table's yields: a big coral more than a soft one).

**What it does: M8's compost**, the rule `animal-husbandry` already wrote
down for fertiliser and never built: "half again as fast, one more at
harvest". F on a sown crop that is not yet ripe, with fertiliser in the
pack, offers `Fertilize <crop>`; F uses one and writes the plot:

- the growth left runs half again as fast: the plot's `planted_s` moves
  back by a third of the time left, so it ripens in two thirds of it, and
  its stage reads on from where it was;
- `fed: true`, so the harvest gives one more produce, and a crop is fed
  once a sowing (a fed crop offers `Check` again).

Speed and yield both, because either alone is a weak reason to swim out
to a reef: a faster crop saves days, the extra one pays the trip back
when sold. The field is `#[serde(default)]`: a plot written before reads
as unfed, and an older build reading a fed plot ignores the field.

### 7. Cooking

Two campfire dishes from what the sea gives, plus what the game already
cooks:

- **steamed mussels**: 3 mussels, 20 s, gives 1; Steady for 6 minutes;
- **kelp crisps**: 2 kelp, 15 s, gives 2; Stride for 4 minutes;
- a crab, lobster, clam or octopus is a fish to the campfire (grilled
  fish, chili fish), as it already would be if caught.

Recommendation taken (ask only with screenshots).

### 8. Decorating: the reef's own model, set on any ground

Owner: "use them as decoration". A coral, sponge, sea fan or anemone is
an **ornament**. With one held, right-click sets it on the aimed ground
within reach: on land, or on the sea floor while swimming. One ornament
to a cell; a cell with one says "Something already stands there". It is
kept as an `ornament` record by the cell's key:

`ornament <cell key> 1 (code: "<item>", at: (x, y, z), yaw: <rad>, standing: true)`

with `at` the body-relative point in `f64` (CLAUDE.md: a stored position
is body-relative `f64`, named and converted at the boundary). F on it
offers `Take up <item>`: it goes back to the pack and the record is
rewritten with `standing: false`.

**Drawn** with the reef's own model of its kind (`places::library`), at
the scale a mid-sized reef piece has, in its kind's first palette colour,
in the reef's material (the water life's atlas, nearest), with
`LitByField` and `LitLikeTerrain` on its root, so it is lit as the ground
under it is, by the terrain's sun and shadows and the cell light field,
and as dark as the ground in a cave or at night. Under water it takes
the depth tint the reef's pieces take (`UnderWater::albedo`), on land
none. It does not sway (a coral out of water is still).

**Why not the furniture tool.** Furniture stands in a cell's hexagon
layout in a room, against walls and clear of doorways, cut into the
town's meshes in the town atlas; an ornament stands on open ground and
on the sea floor, where building is not offered, and wears the reef's
own model and atlas. The fence and incubator's way (a good with a reader
of its own) fits it. Recommendation taken (ask only with screenshots).

The reef's models are flat-shaded vertex-coloured pieces on the water
life's atlas (`ocean-environments` B14), from before the smooth-shading
rule (2026-11-23). An ornament is the same model, as the owner asked
("uses the same model"); smoothing the reef's models is a look change for
the reef as a whole, not for this change.

### 9. Icons drawn from the models

Each new item's 16 px icon is rendered from its model, not drawn apart:

- `cargo test -p pbd-app --lib -- --ignored dump_sea_models` writes
  `tools/art/sea_models.json`: each gatherable model's triangles, vertex
  colours and atlas UVs (`places::library`), its kind's palette, and the
  kelp card;
- `tools/art/sea_gather_icons.py` renders each at 64 px (a three-quarter
  view from above, lit from the top left, the atlas sampled at the UVs and
  tinted the kind's first palette colour), reduces it to 16 px by area
  average snapped to the render's own colours, and adds the 1 px dark
  outline icons carry (CLAUDE.md: outlines are for UI icons only);
- the fertiliser sack and the two dishes have no model, and kelp is a
  card, not a model (reduced, the card read as a ladder): they are pixel
  maps in the same script, kelp's in the card's own colours.

`assets/items/goods/PROVENANCE.md` gets their rows.

### 10. Saves

- New record kinds `sea-take` and `ornament` (schema 1 each), noted in the
  world's identity on first store, as every record kind is.
- `farming::Plot` gains `fed` with `#[serde(default)]`.
- New goods are appended rows, saved by code.
- No generator change: every world places exactly the pieces it did.

An old save opens unchanged: nothing taken, no ornaments, every plot
unfed.

### 11. Frame time (not measured)

This change does no performance work. What it adds: a take rebuilds one
tile once (a tile landing's cost); the aim walks the near pieces of the
tiles within 3 m of the eye each frame (a few hundred distance tests);
each ornament is one mesh of a few hundred triangles. The owner measures
it on real hardware.

## As built (2026-11-25, `claude/phase-2-1`)

- Built as designed, in `pbd_core::sea_gather`, `fish::places` and
  `desktop::sea_gather`; F's acts `gather_sea`, `take_ornament` and
  `fertilize` sit in the one interact table (`Reader::Sea`, the farm's).
- A crawler taken by hand goes through the catch's own durable write
  (`WorldSave::record_catch`), so the field guide counts it as caught.
- The aim for a capture: `PBD_SEA_SHOT=<code|any>:<frame>[:<held good>]`
  (a capture instrument), used by `tools/capture_sea_gathering.sh`.
- Captures (lavapipe, fixed step; looks only) in
  `docs/screenshots/sea-gathering/`: `reef-before`/`reef-after` (a
  staghorn taken at 18.816 92.682), `kelp-before`/`kelp-after` (a kelp
  stand taken at 70.343 -3.462; its rock stays), `pack` (every new icon),
  `ornament-night` (a brain coral set on sand at 22:30, as dark as the sand).
- Not verified in the game: a regrowth seen happen (it needs days of world
  time and a tile rebuilt out of sight; the rule is tested), the crawler
  take and the protected refusal on screen, fertilising on screen (the rule
  is tested), the ornament under water.
