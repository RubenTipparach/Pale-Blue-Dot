# Proposal: a tool in hand, fish that school, and a rod to catch them with

## Why

**The owner's request: "setup a flocking system for fishing. by default give
the player a fishing rod. Reserve a spot on the hotbar for inventory items. if
player hold G, show options next to it, and they can scroll to change
equipment", then "this equipment being fishing rod, vs shovel, axe, pickaxe",
then "yea and the mockup", then "one more thing to add to the design, is
different species of fish, you may reuse fish from tenebris, but make sure each
fish has a wiki entry, and a thumbnail to represent them in the inventory".**

Today:

- **Nothing is alive in the water.** The workspace has no fauna of any kind
  (`grep -r "fish\|fauna\|species" crates/` returns nothing). The sea floats
  boats, but nothing swims in it.
- **The slots cannot hold equipment.** `pbd_core::inventory::Tool` has one
  variant, `Pick`, and its own comment says nothing constructs it. The
  hotbar-and-game-hud change left "the first tool" open on purpose, because a
  tool with no verb is a control for a mechanic that does not exist.
- **Digging ignores the tool.** `desktop/digging.rs::dig_and_place` takes the
  targeted layer on the frame the left button goes down, whatever is in hand.
  A shovel, a pickaxe and an axe can only mean something once the dig asks
  what is held.
- **G is taken.** It boards and leaves a craft (`vehicles.rs::board_or_leave`,
  on `just_pressed`), and the vehicles spec requires one interaction key for
  both.

## What changes

- **A mockup came first, as asked.** `docs/mockups/fishing.html`, published at
  <https://claude.ai/artifact/9XiEvkxSFJMtqaktg4eovj>, is a three.js
  prototype. It uses the engine's gravity (25 m/s²) and a small sea standing in
  for `pbd_core::sea`. It has eight fish species in three kinds of water, each with a
  field-guide entry, swimming as schools of boids, and a rod you charge,
  cast, hook with and reel under line tension. It has the tool slot with its
  hold-G picker, and the shovel, pickaxe and axe working on sample blocks and a
  tree. It was driven headless end to end: cast, a school turns toward the
  lure, one fish breaks off, nibbles, bites, is hooked and reeled in, and lands
  in the hotbar. Separately, a shovel dug a block of dirt. No errors.
- **The hotbar gets a tool slot.** It sits to the left of the ten existing
  slots and holds the one tool in hand. The ten slots stay for items: blocks,
  and now fish.
- **Holding G opens the tool picker beside that slot.** The mouse wheel moves
  the highlight, and releasing G equips the highlighted tool. A tap of G, under
  0.18 s, still boards or leaves a craft. On foot the two are told apart by how
  long the key is held; aboard, G only boards and leaves.
- **Four tools: fishing rod, shovel, pickaxe and axe.** A new world starts with
  all four owned and the rod in hand.
- **Digging asks what is in hand.** Breaking a block becomes a hold: a time per
  material, multiplied up for the wrong tool. The rod cannot break anything.
  The shovel is for soil, the pickaxe for stone and ore, and the axe for trees.
- **The axe fells trees.** A tree today is drawn by the GPU from a hash of the
  cell ID, and the CPU has no record of one. Felling needs one rule for where
  trees stand, shared by the core and the shader. It also needs a durable
  "felled" edit and a wood item.
- **Fish school.** Each school is a flock of boids (separation, alignment,
  cohesion and a wandering goal), bounded by the sea surface, the seabed and
  its species' depth band. Schools spawn in a ring around the player, only in
  water deep enough for them. They are simulated on the CPU in `pbd-core`, in
  packed arrays rather than as one entity per fish, and drawn in one instanced
  draw.
- **Fishing is cast, wait, bite, hook, reel.**
  - The bobber rides the same sea function the boats float on.
  - Schools that scent the lure swim to it, and one fish breaks off to bite.
  - The hook window is Tenebris's: 0.9 s for the weakest fish, 13% shorter per
    step of strength, never under 0.25 s.
  - Reeling is held against a tension meter; too much tension snaps the line.
  - The catch goes into the slots with its own icon.
- **Eight species, each with a field-guide entry and a thumbnail.**
  - Five come from Tenebris: minnow, ray, eel, reef fish and sea serpent. They
    bring Tenebris's strengths, its hook window, and its own 16×16 icons,
    copied with provenance.
  - Three are new: silverfin, banded perch and deepback.
  - Each lives in some kinds of water (rivers, shallows, shelf or deep, from
    the terrain) at a temperature window, read live from the climate
    simulation. Their ranges therefore follow the climate. A measured range
    atlas maps them all on the real planet (design section 7).
  - Measuring them found that **the simulated sea does not hold its climate**.
    It starts from a climatology averaging +13 °C and falls to −17 °C within
    80 days (about 64 hours of play), then settles at −23.5 °C with every
    point of water frozen. That is an atmosphere problem, and the plan
    depends on it being fixed there.
  - Each species' entry is its own record in `fauna.ron`: its numbers, its
    text and a tip. It opens in game with J or a click on a caught fish, and a
    `docs/wiki/fish.md` page is generated from the same data.
  - A test fails if any species lacks an entry or an icon.
- **The rosters are planet data.** Each body lists its species in `fauna.ron`.
  An empty roster, as on an airless, frozen or barren world, means no fish
  spawn and the rod says so.
- **Every catch, tool change and felled tree is written the frame it happens.**

## Owner questions

These come with defaults, and the mockup already shows each default.

1. **"Reserve a spot on the hotbar for inventory items."** I read this as a
   dedicated tool slot, separate from the ten item slots, whose options open
   beside it when G is held. If you meant something else, such as reserving one
   of the ten slots, that changes the HUD and the save line but nothing else.
2. **G as tap versus hold.** The default splits them at 0.18 s. The cost is
   that boarding moves from the key going down to the key coming up, which
   delays it by about a tap's length. The alternative is a different key for
   the picker, which keeps the vehicles unchanged.
3. **The starting kit.** The default is all four tools owned, rod in hand. You
   said "by default give the player a fishing rod", so the alternative is the
   rod alone, with the other three found or made later. There is no crafting
   yet, so owning only the rod would make digging impossible.
4. **Digging becomes a hold.** Today a click takes the block instantly. The
   defaults are 0.25 s for soil, 0.6 s for stone, 0.8 to 1.0 s for rock and
   ore, 1.0 s for a tree, and four times as long with the wrong tool.
   Tenebris's times are much longer (2.5 s, 1.5 s and 5 s at its lowest tier)
   because it has tool tiers, and this change does not.
5. **Tension reeling.** Tenebris's reel is one click followed by a 0.9 s pull.
   The mockup adds a tension meter so that a strong fish is a fight. Say if you
   would rather keep Tenebris's click.
6. **Fishing from a boat.** This is out of scope by default; the rod works on
   foot, like digging. Fishing from the Loon or the Tern is an obvious next
   step.

7. **Where the wiki lives.** The default is both an in-game field guide and a
   generated `docs/wiki/fish.md`, from one source. If "wiki" means a web page
   like Tenebris's `wiki.html` linked from the menu, the generator writes HTML
   instead. The data does not change.

8. **The sea's drift.** The spawn plan assumes the sea holds roughly the
   climate a world starts with, and today it freezes within days of play
   (design section 7). The fix belongs in `atmospheric-circulation` (the sun
   value and the clouds' energy balance). Should that come before fishing is
   built?

## Non-goals

- Tool tiers, durability and crafting. There is one of each tool, and it never
  wears out.
- Cooking, eating or selling fish. A fish is an item and nothing more yet.
- Land fauna, birds, and anything that attacks. Tenebris's turtle stays
  behind for the same reason: Tenebris does not let it be caught.
- A bag behind the ten slots. The owner questions assume the ten are enough
  for now.
