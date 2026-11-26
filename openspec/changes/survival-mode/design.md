# Design

## Context

- **No survival today.** There is no health, no death, no hunger and no
  breath. `cooking` (decision 1, the owner's Q1 answer, asked with
  screenshots) made meals give buffs, with no hunger bar. `swimming` gives
  unlimited breath. `animal-husbandry` keeps predators wild-only, and they
  never attack the player.
- **The design doc says so too.** `docs/GAME_DESIGN.md` section 1 lists
  "survival through constant meter maintenance" as a non-goal. Section 8 says
  "survival creates planning, not chores", and that hazards "telegraph
  cause, trend, threshold, and mitigation".
- **Multiplayer (3b) comes first.** The server owns damage, and every
  survival rule is written for it from the start.
- **Positions and speeds** live in the floating origin. The walker's speed is
  a local `f32`, and "up" and gravity come from the body (`gravity-model`).

## Goals / Non-Goals

**Goals:**
- Survival as an opt-in mode per world, leaving Explorer exactly as it is.
- Pressures that read before they hurt: every damage source shows its cause
  and its trend before the first loss.
- Nothing a player made is lost to death: the pack waits where they fell.
- One set of rules, run by the server in multiplayer and by the in-process
  server alone.

**Non-Goals:**
- PvP, armour, disease, sleep, food spoiling.
- Difficulty levels. There is one tuning file, `assets/config/survival.ron`.
- Damage to craft from crashes.
- Forest fires and torches going out: `wildfire`.

(Ranged weapons and cold and heat were non-goals until the owner's
2026-11-26 message asked for both: design 13 to 16.)

## Decisions

### 1. Mode per world, Explorer by default for old saves

The world's identity record gains `mode: Explorer | Survival`. A save without
it is Explorer, so no world changes under a player (CLAUDE.md, "Saved games
survive every change"). The new-world screen offers both, and so does the
world list.

Recommendation taken (ask only with screenshots): the mode is fixed once
chosen. A switch would let a player step out of Survival at its hardest
moment, and switching back would leave a world with no history of being
survived. If the owner wants a switch, it is one stored field and a menu
entry.

### 2. Health

- One bar, 100 points, kept in the player record. Old saves get 100.
- It refills 1 point every 4 s while hunger is above a quarter, and at half
  that rate below a quarter.
- The numbers are in `survival.ron` with units, validated at load.

### 3. Falls

- Damage comes from the downward speed at impact, along the body's up, in the
  body-local frame: `max(0, v - v_safe) * k`.
- `v_safe = sqrt(2 * g * h_safe)`, with `h_safe` = 3.5 m (three layers and a
  half) and `g` the body's own surface gravity. A step off a two-layer ledge
  never hurts, and on a body with half the gravity the same height hurts
  less, as it should.
- `k` is tuned so a twenty-layer fall onto rock takes about 80 points on the
  home planet. With the home planet's 25 m/s^2 (`gravity::SURFACE_GRAVITY_MPS2_PER_G`
  at 1 g): `v_safe` = 13.2 m/s, a twenty-layer fall lands at 31.6 m/s, so
  `k` = 4.35 points per m/s over; ten layers (22.4 m/s) takes 40, and a
  standing jump (12 m/s, `walking.rs` `jump_speed`) never hurts.
- Landing in a water cell at least 2 layers deep cancels the damage. So does
  a hay or leaf block, if one exists later.

### 4. Hunger

- A bar of 100 food points (fp). Standing about, it empties in 75 minutes
  of play from full (0.0222 fp/s); with the stamina it pays for in normal
  play (design 12) it empties in about 45 minutes, a little under a world
  day (48 minutes). It counts play time only, so a closed game never
  starves anyone (the owner's world time rule).
- Every dish gains a `fills` value, scaled by its stars like its buff. Raw
  food fills a little.
- Empty: health stops refilling, then drains 1 point every 10 s. That gives
  over 16 minutes from empty to death, telegraphed by the bar the whole way.
- The buffs stay exactly as `cooking` made them, in both modes.

### 5. Breath and suit air

- **Breath:** 30 s while the head's cell is water, refilled in 3 s once out.
  Then health drains 5 points a second.
- **Suit air:** 60 s on foot where the body's atmosphere is below a
  breathable density (vacuum, airless bodies, very high altitude). It
  refills inside a sealed craft cabin or a breathable atmosphere. A larger
  suit tank is a crafted item, and waits for the rocketry tiers.
- The bar shows only while it is not full, so a walk on dry land shows
  nothing extra.

### 6. Predators

- The wild-only predators gain, in Survival only, a hunt state:
  - **notice**, by range, scaled up at night;
  - **stalk**;
  - **warn**: a growl and a crouch, held for at least 1.5 s before the first
    attack;
  - **attack**: a lunge, with damage from the species' table;
  - **flee**: below a third of its health, or when the player is out of reach
    for a while.
- They have health. The held tool is the weapon: the axe hits hardest, the
  pickaxe less, bare hands least. That uses the swing's own hit moment
  (`held-swing`).
- A killed predator drops its species' drops.
- In Explorer nothing changes: they never attack the player.
- In multiplayer the server runs them (`multiplayer` design 8), and each
  attack is a server change.

### 7. Townsfolk

- In Survival, a struck townsperson fights or flees by role. A guard with a
  spear fights, and the nearby guards come.
- The fight ends when the player leaves the town's walls or is downed.
  A downed player wakes at their bed, not in a cell (no jail in this phase).
- The town's standing with the player drops, through `townsfolk`'s
  reputation, which is built in phase 3.
- In Explorer, a swing at a townsperson does nothing.

### 8. Death and the dropped pack

- At zero health the whole pack, the hotbar included, drops as one bag
  record at the death place, body-relative `f64` like any drop.
- The bag is shown with an icon and a map marker, and never despawns.
- Walking into it returns everything that fits. What does not fit stays in
  the bag.
- The player wakes at the last bed they slept in (a bed is a placed block that
  sets a spawn; `player-building` makes beds) or at the world's spawn, with
  full health, half hunger and full breath.

### 9. The HUD

- Survival only:
  - health, stamina, hunger and water as four short bars beside the hotbar
    (2026-11-26 adds stamina and water);
  - body temperature as a small thermometer at their end, with its trend
    and its cause (design 13);
  - breath or suit air as a bar, only while not full;
  - "Winded", "Heat stroke" and the pass-out fade as states of those bars,
    not as lines of text;
  - a red edge on the screen for a moment when hit;
  - a plain line on death, "You died", with one button, "Wake up".
- In the stone-and-carved-wood UI (M29), a mockup first, lit at night. No
  explaining text on screen (CLAUDE.md, "Game text is plain").

### 10. Rules in the core, run by the server

- `pbd_core::survival` holds the bars, the damage table and the fall rule, as
  pure functions of state, time step and events, with tests.
- The in-process server (`multiplayer` design 5) ticks them.
- A client predicts its own fall and breath for display, and the server's
  numbers win.
- No player can damage another (PvP stays off).

### 11. Tool wear

The owner, 2026-11-22: "for planning survival: can you add tool degredation?".

- **Survival only.** In Explorer a tool never wears, as today. Every
  recommendation below is taken (ask only with screenshots); the look goes
  to the owner on the HUD mockup (task 1.1).
- **Wear is counted in uses.** Each owned tool carries the uses it has left.
  A use is one finished action: a block broken, a tree felled, a field cell
  tilled, a fish landed, a hit on a creature or a townsperson. A swing at
  nothing, a cast that catches nothing, and a dig given up half way cost
  nothing, so practice and missed swings are free.
- **Harder work costs more.** Breaking a block whose tier is above the
  tool's own (allowed by `tool-crafting` decision 6) costs 2 uses; anything
  else costs 1.
- **The tier sets how long a tool lasts**, in `assets/config/survival.ron`
  (`tool_uses`, by tier): stone 60, bronze 160, iron 320, steel 640. The rod
  has one number per tier as well. These are starting values for the owner
  to play with, not a balance claim.
- **A worn-out tool breaks but is not lost.** At zero uses it becomes a
  broken tool in the same slot: it works as bare hands (`held-swing`'s
  punch), its icon is cracked, and the line "Pickaxe broke" shows once.
  Nothing the player made disappears, in keeping with "Saved games survive".
- **Repair.**
  - At the workbench: half the tool's head materials, rounded up (`tool-crafting`'s recipe for that tool and tier), restore all its uses.
  - In a town: the smith or the trader repairs it for coins, priced from the tool's value (`townsfolk` design 5) times the share worn.
  - A broken tool is repaired the same way.
  - Repairing never raises a tier.
- **What the player sees, Survival only:**
  - a thin bar along the bottom of the tool's slot icon (UI, so the icon's
    1 px outline rule applies), shown once the tool is below full;
  - the bar turns red below a tenth;
  - a short crack sound when it breaks.
  The held model in the world does not change.
- **Saving.** The owned-tool record gains `uses` (additive; a save without
  it reads as full). A use is written in the same durable line as the
  action that spent it (the dig's drop, the catch, the hit), so wear never
  costs an extra write and is never lost on a crash.
- **Multiplayer.** The server counts uses like any damage (decision 10). A
  client shows its own prediction, and the server's number wins.
- **Not in this decision:** tool enchantments or upgrades, wear on armour
  (no armour yet), wear on vehicles and boats (craft damage stays a
  follow-up), and tools wearing in Explorer.

### 12. Stamina, and what it costs in food (owner, 2026-11-26)

The owner: "Stamina regen relies on food levels. It'll consume stored energy
when it regens. Half stamina consumes less food. Pushing or exerting yourself
further, the stamina bar will decrease slowly but also consume food at the
same time and consume more food to regen back up", "Swimming will consume
stamina", "Jumping/climbing will consume stamina (slightly)".

Every number below is a recommendation taken (ask only with screenshots),
except where the owner's words set the shape. The curve goes to the owner on
the mockup's Stamina tab, whose sliders run this exact model
(`docs/mockups/survival.html#stamina`).

- **The bar.** Stamina, 0 to 100 points (pts), its own bar beside health.
- **What spends it.**

  | Action | Stamina | Food while doing it | Strain (design below) |
  | --- | --- | --- | --- |
  | Sprint (14 m/s on foot) | 1.0 pts/s | 0.03 fp/s | yes |
  | Swim (4 m/s, `swim-pace-and-current`) | 0.8 pts/s | 0.025 fp/s | yes |
  | Swim with sprint held (7 m/s) | 1.6 pts/s | 0.05 fp/s | yes |
  | Tread water (in deep water, not moving) | 0.25 pts/s | none | yes |
  | Jump | 2 pts each | none | no |
  | Climb (a layer stepped up, a ledge climbed out of water, a ladder) | 1.5 pts per metre risen | none | no |
  | Throw a spear, hold a bow fully drawn | 4 pts a throw, 0.5 pts/s drawn | none | no |
  | Walk, stand, dig, chop, swing, fish, craft | none | none | - |

  "Slowly" is the owner's word: a full bar sprints for 100 s (1.4 km), and
  swims 125 s (500 m). Tool work spends none, so the loop the owner built
  first (digging, chopping, farming, fishing) never waits on the bar.
- **The refill draws on food.** After 1.0 s with nothing spending it, the
  bar refills at 12 pts/s, and each point refilled costs food at the bar's
  height when it is refilled:
  - 0.02 fp a point while the bar is at or below half ("half stamina
    consumes less food");
  - rising in a straight line from 0.02 at half to 0.05 at full.

  So refilling the bottom half costs 1.0 fp and the top half 1.75 fp: a
  player who lets the bar sit low pays less, and one who keeps it brim full
  pays most.
- **Pushing on costs more to refill** ("consume more food to regen back
  up"). Points spent by an action marked strain above are counted as strain;
  while there is strain, every point refilled costs 1.5 times as much, and
  each point refilled pays off a point of strain. Jumps and climbs are the
  owner's "slightly": they spend points but leave no strain.
- **Food sets the refill.** At or above 50 fp the refill runs at its full
  12 pts/s; below that it slows in a straight line to 40% at 0 fp. Starving
  (0 fp), the bar refills only to half, from the body's last reserve, and
  costs no food (there is none to take).
- **Empty is winded.** At 0 the player is winded until a quarter of the bar
  is back: no sprint, swimming at half speed, jumps at 60% height, no
  climbing out of water. A winded swimmer floats with the head out and does
  not sink, so running out of stamina in the water is slow and recoverable,
  never a drowning on its own (breath is design 5).
- **Cold** halves the refill rate (design 14).

**Worked examples** (from full stamina, food at 80 fp; worked out by
`tools/survival/stamina_model.py`, a 10 ms step of exactly this model, which
the mockup's Stamina tab runs in the page too):

| Case | Stamina after | Food spent | Of which the effort |
| --- | --- | --- | --- |
| A minute of sprinting (840 m), then 10 s to refill | 40, then 100 after 5 s | 6.28 fp | 4.72 fp (1.80 burned while sprinting, 2.92 refilling 60 points of strain) |
| The same 70 s walking (560 m) | 100 | 1.56 fp | 0 |
| A 200 m swim (50 s at 4 m/s), then 6 s to refill | 60, then 100 | 4.77 fp | 3.53 fp (1.25 while swimming, 2.28 refilling) |
| A world day of normal play (48 min): 18 sprints of 20 s, two 60 s swims, 40 jumps, 30 layers climbed, the rest walking and working | - | 112.9 fp | 48.9 fp |
| A world day standing about | - | 64.0 fp | 0 |

A sprint covers ground about four times dearer in food than a walk. A
normal day eats a little over one full bar (design 4's "about 45 minutes"),
which is three or four meals (design 4's `fills`, below).

**Meals fill** (`fills`, fp, at one star; two stars x1.15, three x1.3):
grilled fish 25, chili fish 30, roast meat 35, meat stew 45 (new, design 17),
flatbread 20, roast turnip 20, kale skewer 25, fried egg 15, steamed mussels
20, kelp crisps 10. Raw: raw meat 8, fruit 6, egg 4. Raw fish is not eaten.

### 13. Body temperature: where it comes from (owner, 2026-11-26)

The owner: "Temperature will cause stress, like too hot or too cold. When in
hot weather the body will need to consume more water to avoid heat strokes."

Nothing new is invented for the weather: the atmosphere already models it
for every place on the planet, on the CPU, in the core, saved with the
world, and the server already runs it.

- **The air.** `pbd_core::atmosphere::Atmosphere::sample(direction)` at the
  player gives `air_c` (the air's temperature at the ground's own height,
  deg C), which already carries the biome, the latitude, the season
  (`orbit-and-seasons`), the hour and the altitude; and `humidity` (0..1),
  `wind` (m/s), `sunlight` (W/m^2), `rain_rate` (mm/s) and `snow`.
  Airless bodies have no atmosphere: the suit keeps the body at 37 C there,
  and only its air counts (design 5).
- **The "feels like" temperature** (`feel`, deg C), summed from:

  | Term | Rule |
  | --- | --- |
  | Air | `air_c` |
  | Sun | + 6 C at 1000 W/m^2 of `sunlight`, in proportion, when the sun is not shaded |
  | Shade | the sun term is 0 under a solid cell within 8 layers above, or in a tree's or a building's shadow: a short CPU march toward the sun through the column tier, never a GPU read |
  | Humid heat | above 26 C, + (humidity - 0.5) x 8 C |
  | Wind chill | below 20 C, - 0.5 C per m/s of wind above 2 m/s |
  | Rain or snow on the player | - 2 C, and the player is wet |
  | Wet | for 120 s after leaving water or rain: - 5 C, drying twice as fast within a fire's reach |
  | In water | the water's own temperature, `air_c` clamped to 2..28 C, - 4 C |
  | A fire | campfire + 18 C at the fire, falling in a straight line to 0 at 5 m; brazier + 14 C to 4 m; a held lit torch + 3 C |
  | Shelter | under a roof with walls (no sky above, the cell light field's sky term below a tenth), `feel` moves 40% of the way to 18 C |
  | Worn | a wool cloak + 10 C while `feel` is below 18 C (and - 2 C above 26 C); a straw hat halves the sun term |

- **The body.** `body_c` starts at 37.0 and moves toward a target with a time
  constant of 240 s (about 4 minutes to most of the way):
  - **comfortable** where `feel` is 10 to 26 C: the target is 37.0;
  - **hot**: target = 37 + (`feel` - 26) x 0.12, or x 0.20 while water is
    below 15, plus 0.8 while sprinting or swimming hard. Water is what keeps
    the heat down (the owner's "consume more water to avoid heat strokes");
  - **cold**: target = 37 - (10 - `feel`) x 0.10.
- **Worked examples** (all recommendation taken):
  - desert noon, 38 C air, 900 W/m^2 sun, dry air: `feel` 43.4 C. Watered,
    the body settles at 39.1 C: hot, drinking fast, never a stroke. Out of
    water, it heads for 40.5 C and reaches heat stroke's 40.0 C in about
    7.8 minutes. In the shade of a rock the same air settles at 38.4 C.
  - tundra night, - 18 C air, 6 m/s wind: `feel` - 20 C, the body heads
    for 34.0 C (hypothermia); with a wool cloak, 35.0 C (cold, not hurting);
    with the cloak and 2 m from a campfire, 36.1 C.
- **Shown** as a thermometer beside the bars (Survival only), with the
  trend arrow the design doc's "telegraph cause, trend, threshold" asks for:
  the cause shows as a small icon (sun, wind, wet, fire, snow) beside it.
- **Worn slots.** Two, head and body, beside the pack's grid, in Survival
  only: a straw hat (by hand: 3 wheat; or 6 fibre) and a wool cloak (at the
  workbench: 4 wool, 1 rope). They are not armour and never wear out. They
  show on the player's model in multiplayer, textured like every model.

### 14. Too hot, too cold, and heat stroke (owner, 2026-11-26)

The owner: "Heat strokes will cause damage to health and cause players to
pass out."

| `body_c` | State | What it does |
| --- | --- | --- |
| 36.0 to 38.0 | fine | nothing |
| 38.0 to 39.5 | hot | water used faster (design 15) |
| 39.5 to 40.0 | heat exhaustion | as hot, and health - 0.2 a second; the screen's edge warms |
| 40.0 or more | heat stroke | 10 health at once, then the player passes out |
| 35.5 to 36.0 | cool | nothing |
| 34.5 to 35.5 | cold | stamina refills at half rate; food burns 1.5 times as fast (shivering) |
| below 34.5 | hypothermia | as cold, and health - 0.2 a second |

**Passing out** (heat stroke only; recommendation taken: cold hurts but does
not knock the player down, so a frozen night is a slow warning, not a
sudden loss):
- the view fades to dark over 1.5 s and the player drops where they stand;
  controls are off, and the world goes on (predators can find a body);
- out for 30 s of world time. Lying still, the body cools: its target is
  held at 39.0 C at most while out;
- it ends after the 30 s, or at once when the player is hit (the hit's
  damage still lands), or when another player revives them (hold use on
  them for 2 s with water in hand: + 20 water);
- they wake dazed for 60 s: no sprint, walking at 75%;
- a stroke in deep water floats the player face up, head out; the water
  cools them, so it does not drown them;
- if the health lost reaches 0, it is a death as any other (design 8).

### 15. Water and drinking (owner, 2026-11-26)

The owner: "When in hot weather the body will need to consume more water",
"You could craft straws to get juice out of cactus."

- **The bar.** Water, 0 to 100. At comfort it empties in 50 minutes of play
  (0.0333 a second); sprinting and swimming hard add 0.02 a second.
- **Heat raises it**: x (1 + (`feel` - 26) / 10) above 26 C, so x 2.7 at a
  desert noon (full to empty in 18 minutes); x 0.8 below 10 C.
- **Empty**: health - 1 every 6 s, and the body heats at the dry rate
  (design 13). The bar is shown the whole way down.
- **Drinking.**

  | Source | Water | How |
  | --- | --- | --- |
  | Fresh water (`planet_gen::fresh_water_at`: rivers, lakes, swamp pools) | + 10 a sip, 1 s | crouch at it with an empty hand and hold use |
  | A water bucket (`water-bucket`, exists) | + 15 a sip, 4 sips | in hand, hold use; it becomes a `bucket` |
  | Milk | + 15 | eat it |
  | Fruit | + 4 | eat it |
  | Meat stew | + 20 | eat it |
  | A cactus, with a straw in hand | + 8 a sip, 1.5 s | aim at a standing cactus within reach, hold use |
  | Sea water | 0 | "Salt water", and nothing taken |

- **The straw.** `straw`, by hand in the pack's Craft row: 2 fibre, 1 s.
  It is a tool with 30 sips of wear (design 11). A desert cactus
  (`tree_kit::grows_cactus`: desert sand and rock, 8 cells in 256) holds 3
  sips; each one taken is written to a `cactus` record keyed like a tree's
  (`FruitPicked`'s shape: the day and the sips taken), and the cactus gains
  a sip back each world day. A dry cactus says "Dry". Cacti stay scenery: a
  straw never fells or harms one, and its look does not change.
- **Explorer** has no water bar; drinking does nothing there.

### 16. Weapons (owner, 2026-11-26)

The owner: "Making spears, arrows, hunting axe, swords will all be a thing."

On the existing mechanism: recipes in `recipes.ron` at the pack's Craft row
(`hand`), the workbench and, through their ingots, the forge; each tier
behind the starter tree's unlocks (design 18). Every weapon is a tool with
uses (design 11, by tier) and a held model by the `animation` skill and
`held-swing`; every weapon's hit lands on the swing's own hit moment.

| Weapon | Recipe (station: inputs) | Damage | Reach or range | Time | Uses |
| --- | --- | --- | --- | --- | --- |
| Stone spear | hand: 1 log, 2 stone, 2 fibre | 14 thrust, 22 thrown | 2.6 m; thrown 18 m/s | 0.7 s | 60 |
| Bronze spear | workbench: 1 bronze ingot, 2 planks | 20, 30 | as stone | 0.7 s | 160 |
| Iron spear | workbench: 1 iron ingot, 2 planks | 26, 38 | as stone | 0.7 s | 320 |
| Bow | workbench: 2 planks, 2 rope | arrows' | 45 m/s at full draw (0.8 s) | 0.8 s draw | 120 |
| Stone arrows x4 | hand: 1 planks, 1 stone, 1 fibre | 18 at full draw | the body's own gravity | - | used up |
| Bone arrows x4 | workbench: 1 planks, 1 bone, 2 feather | 22 | - | - | used up |
| Bronze arrows x8 | workbench: 1 bronze ingot, 1 planks, 2 feather | 26 | - | - | used up |
| Bronze hunting axe | workbench: 2 bronze ingot, 1 planks, 1 leather | 24 | 2.0 m | 0.6 s | 160 |
| Iron hunting axe | workbench: 2 iron ingot, 1 planks, 1 leather | 30 | 2.0 m | 0.6 s | 320 |
| Bronze sword | workbench: 3 bronze ingot, 1 planks, 1 leather | 28 | 2.2 m | 0.5 s | 160 |
| Iron sword | workbench: 3 iron ingot, 1 planks, 1 leather | 36 | 2.2 m | 0.5 s | 320 |

- **The tools a player already has** hit too (design 6): bare hands 3, stone
  axe 10, bronze 14, iron 18; a pickaxe 0.7 of its tier's axe; a shovel 0.5.
- **What each is for.** The spear is the first weapon, made by hand, held
  or thrown (a thrown spear lands as a drop to pick up). The bow reaches
  game that runs. The hunting axe dresses a kill for its full drops (design
  17) and chops at its tier's axe speed with half the logs. The sword is
  only a weapon, the quickest and hardest-hitting, for predators and, in
  Survival, townsfolk who fight (design 7).
- **Arrows fly in the floating origin.** An arrow is a local `f32`
  projectile under the body's own gravity, moved by the origin's rebase
  like any simulated thing (`tests/origin_guard.rs`), hit-tested on the CPU
  against creature bodies and the column tier. The server flies it in
  multiplayer and the client draws its own prediction. An arrow that lands
  is kept 70% of the time (by a hash of its flight, so a replay agrees) and becomes a
  drop record where it stopped, kept until picked up; the rest break.
- **Explorer** has none of these recipes (recommendation taken: Explorer's
  animals never threaten, and nothing in it is hunted). A world's mode is
  fixed (decision 1), so no Explorer world ever holds a weapon.

### 17. Hunting (owner, 2026-11-26)

The owner: "Hunger is a thing, hunting is a thing too."

- **What is hunted**, Survival only. The home roster (`husbandry.ron`
  `land."home"`) today is the meadow hen, the woolback, the plodder and the
  horse. Wild members of the first three are game. Appended to the roster
  (append-only), with M7's species (`animal-husbandry` decision 1): the
  hare (wild only, skittish), the ridge goat and the rootling (tameable),
  and the predators of decision 6, the red fox and, in the desert, the dust
  hound.

  | Species | Health | Behaviour when hunted | Drops (per kill) |
  | --- | --- | --- | --- |
  | Meadow hen | 10 | runs at 3 m/s from 9 m | raw meat 1, feather 3 |
  | Hare | 12 | bolts at 9 m/s from 14 m, zigzags | raw meat 1, hide 1, bone 1 |
  | Woolback | 40 | the herd runs together | raw meat 3, hide 1, wool 2, bone 2 |
  | Ridge goat | 45 | runs uphill | raw meat 3, hide 1, bone 2 |
  | Rootling | 55 | a boar charges once if cornered (8 damage), then runs | raw meat 4, hide 1, bone 2 |
  | Plodder | 70 | lumbers off at 3.2 m/s | raw meat 5, hide 2, bone 3 |
  | Red fox | 30 | a predator (decision 6) | hide 1, bone 1 |
  | Dust hound | 50 | a predator | hide 1, bone 2 |

- **Never hunted:** protected species (`fauna.ron` and M25's list: otters,
  seals, beavers, reef turtles, wild dabblers) and every animal a player
  keeps or bought (tamed, penned, the horse). A strike at one does nothing,
  as in Explorer, and says "Protected" or nothing at all for a kept animal.
  Butchering a kept animal is not in this change (recommendation taken).
- **Dressing.** A kill drops its meat and bones as world drops at once. Its
  hide comes whole only when the carcass is dressed: use a hunting axe on it
  for the hide and one more meat; any other blade takes the hide one time in
  two (by a hash of the kill). The carcass lies 120 s, then its remaining
  drops lie as drops.
- **Herds come back.** A wild herd is seeded by its herd square
  (`husbandry.ron` `herds`). A kill writes a `hunted` record (the square's
  id, the member, the world day); the member is back after 3 world days of
  play. So the land is never hunted bare, and the record keeps a reload from
  bringing a killed animal straight back.
- **Where the kill goes.** Raw meat cooks into `roast-meat` at the campfire
  (in `cooking.ron`, waiting for `raw-meat` in `awaited` since `cooking`),
  and into a new `meat-stew` (raw meat 1, turnip 1, a water bucket; fills 45,
  water + 20). Hide becomes `leather` at a new tanning rack (workbench: 4
  planks, 2 rope; one hide, 60 s of world time), which the hunting axe, the
  sword and the bedroll want. Bone makes bone arrows and, by hand, 2
  `fertilizer` (an existing good) from 1 bone. Feathers fletch arrows.
- **The server decides** every hit, kill and drop (decision 10).

### 18. The Survival branch of the starter missions (owner, 2026-11-26)

The owner: "we're gonna work on survival mode which will add new branch to
starter missions to handle survival."

- **One tree, one more branch.** The branch is `8 · SURVIVAL` in the
  starter tree (`starter-missions`, "Missions are a tree that unlocks the
  tech"), not a third tree: the survival loop is the same loop with its
  pressures on, and the town tree stays the town's. Each of its missions
  carries `mode: survival` in `missions.ron`; the Missions screen of an
  Explorer world never shows them, and they are never counted there.
- **It opens with the pack** (`pack`, the trunk's second mission), so the
  first thing a new Survival player is told is to drink: water empties
  first. It keeps the tree's rules: one parent each, a hidden `after` gate
  for an extra need, unlocks as each mission's reward.
- Mocked up on M33's Survival tab (`docs/mockups/mission-tree.html#survival`).

| id | Mission | Parent (after) | Objectives | Unlocks |
| --- | --- | --- | --- | --- |
| `drink` | Drink water | `pack` | drink 3 sips of fresh water (**new** kind `drink`) | `straw` |
| `forage` | Something to eat | `drink` | eat 2 raw foods (fruit, an egg) | `stone-spear` |
| `spear` | Make a spear | `forage` | craft a stone spear | - |
| `hunt` | Hunt | `spear` | **kill** 1 wild hen, hare or goat (**new** kind `kill`) | - |
| `cook-meat` | Cook meat | `hunt` (`fire`) | cook 2 `roast-meat` | `meat-stew` |
| `first-night` | The first night | `cook-meat` | be out past 20:00 and alive at 06:00 (**new** kind `dawn`) | `bedroll` |
| `dress` | Dress a kill | `hunt` | take 1 hide from a kill | `tanning-rack` |
| `tan` | Tan a hide | `dress` | make 1 leather at the tanning rack | `bow`, `stone-arrows` |
| `bow` | Bow and arrows | `tan` | craft a bow and 8 arrows | `bone-arrows` |
| `long-shot` | A long shot | `bow` | kill 1 animal with an arrow from 20 m or more | `bronze-hunting-axe` |
| `hunting-axe` | A hunting axe | `long-shot` (`smelt`) | craft a bronze hunting axe; dress a kill with it | `bronze-sword`, `bronze-spear`, `bronze-arrows` |
| `sword` | A sword | `hunting-axe` | craft a bronze sword | `iron-sword`, `iron-spear`, `iron-hunting-axe` (with `iron`) |
| `fend-off` | Drive off a predator | `sword` | make a predator flee or kill it (**new** kind `drive-off`) | - |
| `straw` | A cactus straw | `drink` | craft a straw | - |
| `cactus` | Cactus juice | `straw` | drink 3 sips from a cactus | `straw-hat` |
| `cool-down` | Out of the heat | `cactus` | cool from hot to fine (**new** kind `cool`) | - |
| `warm-up` | Warm by the fire | `cool-down` (`fire`) | warm from cold to fine within a fire's reach (**new** kind `warm`) | `wool-cloak` |
| `dress-weather` | Dress for the weather | `warm-up` | wear a straw hat or a wool cloak (**new** kind `wear`) | - |
| `long-swim` | A long swim | `forage` | swim 100 m without being winded (kind `swim`, held) | - |
| `hold-breath` | Hold your breath | `long-swim` | stay under for 15 s and surface (**new** kind `dive`) | - |
| `soft-landing` | Into the water | `hold-breath` | fall 10 layers or more into deep water unhurt (**new** kind `fall`) | - |
| `relight` | Relight a torch | `cook-meat` (`torch`) | relight a torch the rain put out (`wildfire`; **new** kind `relight`) | - |
| `put-out` | Put out a fire | `relight` | put out 3 burning cells with water (`wildfire`; **new** kind `douse`) | - |

- **Rewards** are unlocks, as the rest of the starter tree's are: the branch
  opens the survival recipes in the order the player needs them. The
  `bedroll` (by hand: 2 hide, 1 wool) is the branch's own payoff: laid on the
  ground it sets the player's spawn (decision 8), so a Survival player has a
  place to wake before they can build a bed.
- **Help lines** are short and plain, in the locale file: "Crouch at a river
  or lake and hold use.", "Hunt with the spear. Thrown, it hits harder.",
  "Stand in the shade or get in the water.", "Fires start in the dry season."
- **The weapons' recipes are locked** in Survival until their mission
  (unlocks); in Explorer they do not exist (decision 16).
- **Saves.** The branch's ids join the world's `missions` record,
  append-only; an older world is Explorer (decision 1) and never sees them.
- **The bot** plays the branch as its own goal: "survive the first night in
  Survival" (task 5.1) is this branch's trunk, `drink` to `first-night`.

### 19. Explorer and Survival side by side

| | Explorer | Survival |
| --- | --- | --- |
| Health, falls, death, the dropped pack | none | decisions 2, 3, 8 |
| Hunger, meals' `fills` | buffs only (`cooking` Q1) | decision 4, buffs too |
| Stamina | none: sprint and swim as today | decision 12 |
| Water, body temperature, heat stroke | none | decisions 13 to 15 |
| Breath and suit air | unlimited (`swimming`) | decision 5 |
| Predators | never attack | decision 6 |
| Hunting, weapons, worn slots | none | decisions 13, 16, 17 |
| Tool wear | none | decision 11 |
| The Survival branch | hidden | decision 18 |
| Torches | as today: always lit, light only the world | they light things and the rain puts them out (`wildfire`) |
| Forest fires | none; lightning's short fires stay weather (`lightning-fire`) | they start in the dry season and spread (`wildfire`) |

### 20. `assets/config/survival.ron`, the proposed shape

One file, read at load, validated (every number finite and in its range,
every name a real good, species or recipe), the code's one source of
defaults. Units in the names or the comments. Not written until the build
(this is the plan).

```ron
// Survival (pbd_core::survival; openspec/changes/survival-mode). Points
// (pts) out of 100 unless named; seconds of play; world days are 48 minutes
// of play. Nothing here applies in an Explorer world.
(
    health: (max: 100.0, regen_s_per_pt: 4.0, regen_hungry_mult: 0.5,
             hungry_below_fp: 25.0, starving_drain_s_per_pt: 10.0),
    fall: (safe_height_m: 3.5, pts_per_mps: 4.35, water_depth_layers: 2),
    food: (max_fp: 100.0, idle_empty_min: 75.0, start_fp: 100.0,
           wake_fp: 50.0, cold_mult: 1.5),
    stamina: (
        max: 100.0, regen_pts_per_s: 12.0, regen_delay_s: 1.0,
        cost_low_fp_per_pt: 0.02,   // at or below half
        cost_high_fp_per_pt: 0.05,  // at full, linear from half
        strain_mult: 1.5,
        full_rate_above_fp: 50.0, rate_at_empty_food: 0.4,
        starving_cap: 50.0, winded_until: 25.0,
        winded: (swim_speed_mult: 0.5, jump_mult: 0.6),
        cold_regen_mult: 0.5,
        spend: (
            sprint: (pts_per_s: 1.0, fp_per_s: 0.03, strain: true),
            swim: (pts_per_s: 0.8, fp_per_s: 0.025, strain: true),
            swim_sprint: (pts_per_s: 1.6, fp_per_s: 0.05, strain: true),
            tread: (pts_per_s: 0.25, fp_per_s: 0.0, strain: true),
            jump_pts: 2.0, climb_pts_per_m: 1.5,
            spear_throw_pts: 4.0, bow_drawn_pts_per_s: 0.5,
        ),
    ),
    water: (max: 100.0, comfort_empty_min: 50.0, exertion_per_s: 0.02,
            heat_per_c_above: 0.1, cold_mult: 0.8, empty_drain_s_per_pt: 6.0,
            dry_below: 15.0),
    body: (
        start_c: 37.0, time_constant_s: 240.0,
        comfort_c: (10.0, 26.0),
        hot_c_per_c: 0.12, hot_dry_c_per_c: 0.20, exertion_c: 0.8,
        cold_c_per_c: 0.10,
        feel: (sun_c_at_1000_wm2: 6.0, humid_c_per_unit: 8.0,
               chill_c_per_mps: 0.5, chill_above_mps: 2.0, chill_below_c: 20.0,
               rain_c: -2.0, wet_c: -5.0, wet_s: 120.0,
               water_c: (2.0, 28.0), in_water_c: -4.0,
               shelter_pull: 0.4, shelter_c: 18.0, shade_march_layers: 8),
        fires: [("block:campfire", 18.0, 5.0), ("block:brazier", 14.0, 4.0),
                ("held:torch", 3.0, 0.0)],
        worn: [("wool-cloak", 10.0, -2.0), ("straw-hat", 0.0, 0.0)],
        hot_c: 38.0, exhaustion_c: 39.5, stroke_c: 40.0,
        cool_c: 36.0, cold_c: 35.5, hypothermia_c: 34.5,
        hurt_per_s: 0.2, stroke_pts: 10.0,
    ),
    pass_out: (fade_s: 1.5, out_s: 30.0, cool_to_c: 39.0, dazed_s: 60.0,
               dazed_walk_mult: 0.75, revive_hold_s: 2.0, revive_water: 20.0),
    breath: (under_s: 30.0, refill_s: 3.0, drown_pts_per_s: 5.0),
    suit_air: (reserve_s: 60.0),
    drinks: [(source: "fresh", water: 10.0, sip_s: 1.0),
             (source: "water-bucket", water: 15.0, sips: 4),
             (source: "cactus", water: 8.0, sip_s: 1.5, sips: 3, regrow_days: 1.0),
             (source: "milk", water: 15.0), (source: "fruit", water: 4.0),
             (source: "meat-stew", water: 20.0)],
    fills: {"grilled-fish": 25.0, "chili-fish": 30.0, "roast-meat": 35.0,
            "meat-stew": 45.0, "flatbread": 20.0, "roast-turnip": 20.0,
            "kale-skewer": 25.0, "fried-egg": 15.0, "steamed-mussels": 20.0,
            "kelp-crisps": 10.0, "raw-meat": 8.0, "fruit": 6.0, "egg": 4.0},
    star_fill_mult: [1.0, 1.15, 1.3],
    weapons: {
        "hand": (damage: 3.0, reach_m: 1.6, time_s: 0.5),
        "stone-spear": (damage: 14.0, thrown: 22.0, reach_m: 2.6, throw_mps: 18.0, time_s: 0.7),
        "bow": (draw_s: 0.8, arrow_mps: 45.0, kept_share: 0.7),
        "stone-arrow": (damage: 18.0), "bone-arrow": (damage: 22.0),
        "bronze-arrow": (damage: 26.0),
        "bronze-hunting-axe": (damage: 24.0, reach_m: 2.0, time_s: 0.6),
        "bronze-sword": (damage: 28.0, reach_m: 2.2, time_s: 0.5),
        // ... every tier as the design 16 table
    },
    tool_damage_share: {"pickaxe": 0.7, "shovel": 0.5},
    hunting: (
        carcass_s: 120.0, herd_back_days: 3.0, dress_bonus_meat: 1,
        other_blade_hide_share: 0.5,
        game: {
            "meadow-hen": (health: 10.0, flee_m: 9.0,
                           drops: [("raw-meat", 1), ("feather", 3)]),
            "hare": (health: 12.0, flee_m: 14.0,
                     drops: [("raw-meat", 1), ("hide", 1), ("bone", 1)]),
            "woolback": (health: 40.0, flee_m: 12.0,
                         drops: [("raw-meat", 3), ("hide", 1), ("wool", 2), ("bone", 2)]),
            "plodder": (health: 70.0, flee_m: 10.0,
                        drops: [("raw-meat", 5), ("hide", 2), ("bone", 3)]),
            // ridge goat, rootling, red fox, dust hound as design 17
        },
    ),
    tool_uses: (stone: 60, bronze: 160, iron: 320, steel: 640, straw: 30, bow: 120),
)
```

## Questions for the owner (with screenshots), 2026-11-26

Asked only where there is something to look at (CLAUDE.md, "Ask only with
screenshots"); each recommendation is built into the mockup while the
question is open. Screenshots in `docs/screenshots/survival-mode/mockup/`.
To go into the survey (`docs/design/survey.html`) when the pages are
published.

| Id | Question | Options | Recommendation | Screenshots |
| --- | --- | --- | --- | --- |
| G6 | The stamina and food curve: a normal day eats about 1.1 bars of food, a minute's sprint 6.3 | as drawn; gentler (half the effort's food); harsher (half again) | as drawn: sprinting costs about four times walking, and three or four meals a day | `stamina-sprint.png`, `stamina-swim.png`, `stamina-day.png` |
| G7 | The survival HUD: four short bars and a thermometer above the hotbar | as drawn; a vertical stack at the left edge; bars only when not full | as drawn: one glance beside the hotbar, Explorer unchanged | `hud-day-fine.png`, `hud-night-fine.png`, `hud-day-hot.png`, `hud-night-cold.png`, `hud-night-winded.png`, `hud-day-worn.png` |
| G8 | Heat stroke and passing out: the edge warms, then the view fades for 30 s | as drawn; no pass-out, health only | as drawn: the owner asked for passing out, and it is telegraphed from 38 C | `hud-day-stroke.png`, `hud-day-hot.png` |
| G10 | The weapons' looks: stone spear, bow and arrow, bronze hunting axe, bronze sword | approve for the animation step; change a look | approve, then the held models and swings go through the `animation` skill (M36) | `weapons-day-all.png`, `weapons-night-all.png`, `weapons-day-spear.png`, `weapons-day-bow.png`, `weapons-day-axe.png`, `weapons-day-sword.png` |
| G11 | Where the Survival branch hangs: off "Open the pack" | off the pack; off "Make a fire" | off the pack: water empties first, so drinking is taught first | `m33-survival-fit.png`, `m33-survival-hunt.png`, `m33-survival-hunting.png`, `m33-survival-heat.png` |

(G9 is `wildfire`'s.)

**Recommendations taken (ask only with screenshots)**, not the owner's own
words: every number in design 12 to 17 and 20; tool work spends no stamina;
cold hurts but never knocks the player down; a winded swimmer floats; two
worn slots (hat, cloak) that are not armour; drinking by hand at fresh water;
the straw from 2 fibre with 30 sips; no hunting, weapons or worn slots in
Explorer; kept and bought animals cannot be struck (butchering a follow-up);
a kill's hide needs dressing; herds back after 3 world days; the branch as
a branch of the starter tree, not a third tree; rewards as unlocks, the
bedroll as its payoff.

## Risks / Trade-offs

- **Wear can feel like a chore.** That is why missed swings and abandoned digs are free, why a broken tool is kept and can be repaired, and why the numbers live in one file.

- **It reverses a decision the owner made with screenshots** (`cooking` Q1),
  but only in Survival. Explorer keeps it. The owner sees both on the HUD
  mockup.
- **Predators that hunt can make the night a chore.** That is why every
  attack is warned first, why they flee when hurt, and why the tuning lives
  in one file the owner can play with.
- **Fall damage depends on the walker reporting impact speed correctly** in
  the floating origin, on every body. A test lands on the home planet and on a
  lower-gravity body.
- **Stamina could tax the calm loop.** That is why tool work spends none,
  why the bar is slow ("slowly" is the owner's word), why a winded swimmer
  floats, and why the whole curve is on the mockup's sliders for the owner
  to feel before a number is built.
- **Heat stroke can feel like a sudden loss.** It is telegraphed from 38 C
  (the thermometer, its trend and its cause), water holds it off entirely,
  and passing out lasts 30 s with the pack kept.
- **Body temperature reads the atmosphere at the player.** That is a CPU
  sample of a saved, deterministic model the server already runs, so it
  needs no new authority; the shade test is a short CPU march, never a GPU
  read (CLAUDE.md, "Architecture and authority").
- **Fighting townsfolk needs `townsfolk`** (phase 3) to have reputation and
  guards. Phase 4 comes after it, so it will.
