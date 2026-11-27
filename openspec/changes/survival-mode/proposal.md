## Why

The owner, 2026-11-18: "3c will add survival mode, falling from high places,
dangeroous animals (we'll introduce predators), fighting with villagers,
hunger, and oxygen - underwater can t breath, or in space".

Planned first as phase 3c, it moved to phase 4, with town building, civil
engineering and power generation (owner, 2026-11-22: "bump surivival to
phase 4 s well"). It still comes after multiplayer (3b).

The owner started phase 4 with it on 2026-11-26, and widened it:

> "start working on phase 4, we're gonna work on survival mode which will
> add new branch to starter missions to handle survival.
> Survival: you can start foresr fires during dry seasons. Torches can light
> up stuff or can be out out by rain.
> Hunger is a thing , hunting is a thing too. Making spears, arrows , hunting
> axe, swords will all be a thing.
> Stamina regen relies on food levels. It'll consume stored energy when it
> regens. Half stamina consumes less food. Pushing or exerting yourself
> further, the stamina bar will decrease slowly but also consume food at the
> same time and console [consume] more food to regen back up.
> Underwater will consume oxygen. Temperature will cause stress, like too hot
> or too cold. When in hot weather the body will need to consume more water
> to avoid heat strokes. Heat strokes will cause damage to health and cause
> players to pass out. You could craft straws to get juice out of cactus.
> Swimming will consume stamina.
> Falling will damage health.
> Jumping/climbing will consume stamina (slightly)"

Forest fires and torches that the rain puts out are a world process with
saved records of their own, so they are written up in their own change,
`wildfire` (its design 1 says why); everything else the owner listed is
here.
Today the game has no health, no death, no hunger and no breath:
- `cooking` decision 1 chose buffs over a hunger bar, on the owner's answer
  to Q1, which was asked with screenshots;
- `animal-husbandry` keeps predators wild-only, and nothing attacks the player
  (M7's recommendation);
- `swimming` gives unlimited breath;
- `docs/GAME_DESIGN.md` lists "survival through constant meter maintenance"
  as a non-goal.

**Survival is therefore a mode, not a change to the game everyone plays.**
A world is made in Explorer (today's calm game, unchanged) or Survival. Every
earlier decision stays true in Explorer, and Survival is where the owner's
new pressures live. Recommendation taken (ask only with screenshots); the
HUD and the danger look go to the owner as mockups.

## What Changes

- **A world's mode**: Explorer or Survival, chosen when the world is made and
  shown in its world list entry. Every existing save opens as Explorer.
- **Health**: one health bar for the player. It refills slowly while fed, and
  reaching zero means death and a respawn.
- **Falling**: landing too fast hurts. Damage comes from the speed at impact
  in the planet's own gravity, so a low-gravity world is a safer place to
  jump. Deep water breaks a fall.
- **Hunger**: a bar that empties slowly over play time. Meals refill it as
  well as giving their buffs. An empty bar stops health from refilling, then
  slowly drains it. It never kills quickly, and never while the game is
  closed.
- **Breath and oxygen**: with the head under water, a breath bar empties and
  then health drains. On foot in thin air or vacuum, a suit's air reserve
  does the same. Inside a sealed craft's cabin the player breathes the
  craft's air.
- **Predators**: the wild-only predators of `animal-husbandry` (red fox, dust
  hound and their kin) hunt in Survival. They stalk, warn, attack, and run
  when hurt, and are bolder at night. The player fights back with what is in
  their hand.
- **Fighting villagers**: a townsperson can be struck. They and the town's
  guards fight back, and the town remembers it (`townsfolk`'s reputation).
- **Tool wear**: in Survival every tool has a number of uses, set by its
  tier, and each finished action spends one. A worn-out tool breaks but
  stays in its slot. It can be repaired at the workbench or by a town's
  smith for coins. The owner, 2026-11-22: "for planning survival: can you
  add tool degredation?".
- **Death**: the player's pack drops where they died as a marked bag that
  stays until it is picked up, and they wake at their bed or the world's
  spawn.
- **Stamina** (2026-11-26): its own bar. Sprinting, swimming and climbing
  spend it, a jump a little. It refills on its own, and every point it
  refills draws on the food the player has eaten: the bottom half costs
  less food a point than the top half, and points spent pushing on (a
  sprint, a swim) drain the bar slowly, burn food while they are spent and
  cost half as much again in food to refill. Empty, the player is winded
  until a quarter is back. Design 12.
- **Thirst and body temperature** (2026-11-26): a water bar beside hunger.
  Body temperature comes from the air the weather already models at the
  player (biome, season, hour, altitude), the sun and shade, wind, rain,
  being wet, a fire, shelter and what is worn. Hot, the body needs more
  water; hotter still and dry, it heats past 40 C into heat stroke, which
  hurts and makes the player pass out for 30 s. Cold slows stamina, burns
  more food and, in the end, hurts. Design 13 and 14.
- **Drinking**: fresh water by hand or bucket, milk and fruit, and a
  cactus straw (two fibre, by hand) that drinks juice from a desert cactus.
  Design 15.
- **Hunting and weapons** (2026-11-26): wild game is hunted with a spear
  (held or thrown), a bow and arrows, a hunting axe and swords, made at
  three tiers on the existing station and recipe tables. A kill drops raw
  meat, hide and bone: meat for the campfire (`roast-meat` has waited for
  it), hide for leather at a tanning rack, bone for arrows and fertiliser.
  Protected and kept animals cannot be hunted. Design 16 and 17.
- **Worn things**: a straw hat and a wool cloak, two worn slots, for the
  heat and the cold. Design 13.
- **Survival's missions** (2026-11-26, reshaped 2026-11-27): the starter
  tree as the game has it up to "Make a fire", then paths the player picks
  (owner, 2026-11-27: "Survival: consist of the starter tree today then (in
  the game) then branch off to all survival stuff like eating and drinking,
  hunting etc."): Eat and drink, Hunting, Fishing, Farming, Shelter and
  weather, In the water, and the starter tree's other branches. When the
  fire is made the Missions screen opens with a path picker ("after making a
  fire present players with the tree and show them options on what branch
  to take"); the path picked is the current goal, the others stay open, and
  the picker shows again when the path is done. Survival missions show only
  in Survival worlds. Drink missions count fresh water only (owner,
  2026-11-27: "Keep in mind salt water is undrinkable"). Design 18.
- **The HUD**: health, stamina, hunger, water and body temperature show
  only in Survival, stacked at the left edge, and only when they matter: a
  full bar is hidden, and hunger and water show only at 50% or lower (owner,
  2026-11-26, survey G7: "Don't need to show food or water till it's 50% or
  lower. You can check in pack these stats at any time..if stats are full
  hide them."). Every stat is always in the pack's new stats panel. Heat
  stroke is one short warning above the hotbar (G8: "Just show stroke
  warning above hot bar"). Breath only while it is not full. A mockup first
  (`docs/mockups/survival.html`, task 1.1).
- **The owner's answers of 2026-11-26** (survey G6 to G11): the stamina and
  food numbers stay as drawn, shown in action by an animation on the
  mockup's Stamina tab (G6: "Some sort of js animation would be nice to see
  this in action. Add an expand button to lazy load"); heat stroke as drawn
  (G8); the weapons' looks approved for the animation step (G10); the
  Survival branch off "Open the pack" (G11: "Go right into survival after
  pack is opened (if survival mode is active)"), superseded on 2026-11-27 by
  the paths after the fire above.

Not in this change: PvP (still off in multiplayer), armour, disease, sleep
needs, food spoiling, difficulty levels beyond one tuning file, and craft
crash damage. Forest fires and torches going out are `wildfire`.

## Capabilities

### New Capabilities
- `player/survival`: the world's mode, the player's health, damage from
  falls, hunger, stamina and its food cost, thirst, body temperature and
  heat stroke, breath and suit air, hunting and weapons, tool wear and
  repair, death and the dropped pack, and respawn.
- `player/missions` (added to, as `starter-missions` adds it): the Survival
  paths after the fire and the path picker, shown only in Survival worlds.

### Modified Capabilities
<!-- None in openspec/specs: predators (`animal-husbandry`), townsfolk,
swimming and cooking are changes not yet synced. This change's tasks name
what each gains in Survival, and each keeps its Explorer behaviour. -->

## Impact

- **`pbd-core`**: health, hunger, breath and damage as rules (engine-free and
  tested). They run on the server, because multiplayer (3b) comes first and
  the server owns damage (`multiplayer` design 8, `docs/GAME_DESIGN.md` 15).
- **Fauna**: predators gain a hunt state in Survival, and their attacks are
  server requests like any other damage.
- **Townsfolk**: guards and villagers gain a fight-back state. Reputation
  records a fight.
- **Saves**: the world records its mode, the player record gains health,
  hunger, stamina, water, body temperature, breath and spawn, and each owned
  tool gains its uses left, with full values for every older save. A dropped
  pack, a hunted herd member and a tapped cactus are records. Additive only.
- **Weather**: body temperature reads the atmosphere's `Sample` at the
  player (`air_c`, `humidity`, `wind`, `sunlight`, `rain_rate`, `snow`), on
  the server, as the weather already is.
- **Recipes and goods**: new goods (`raw-meat`, `hide`, `bone`, `feather`,
  `straw`, `arrow`, the weapons, `straw-hat`, `wool-cloak`, `bedroll`) and
  recipes, append-only, names from the locale file.
- **Fauna**: the home roster gains wild game and predators, appended
  (`husbandry.ron` is append-only), each with art and motion by the
  `animal-husbandry` and `animation` routes.
- **Animation**: the swing becomes an attack too (`held-swing`), and
  predators gain stalk, lunge and flee motions, mockup first by the
  `animation` skill.
- **Mockups first**: `docs/mockups/survival.html` (the HUD, the stamina and
  food curve with sliders, a forest fire at night, the weapons at true
  scale), M33's Survival tab for the paths and the picker, then a predator hunt by day and
  lit at night, and death with its dropped pack.
