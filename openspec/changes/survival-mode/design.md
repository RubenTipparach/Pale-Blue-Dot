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

The owner, 2026-11-26 (survey G7, picking "a vertical stack at the left
edge"): "Don't need to show food or water till it's 50% or lower. You can
check in pack these stats at any time..if stats are full hide them." And on
heat stroke (G8): "Just show stroke warning above hot bar. See G 7 for
data."

- Survival only:
  - health, stamina, hunger and water as short bars **stacked one above the
    other at the left edge of the screen**, each with its icon (2026-11-26
    adds stamina and water; G7 moves them from beside the hotbar);
  - **a bar that is full is hidden.** Health and stamina show once they are
    below full; **hunger and water show only at 50% or lower** (G7). A
    player who is fed, watered, rested and unhurt sees only the hotbar and
    the compass, as in Explorer;
  - body temperature as a small thermometer under the bars, with its trend
    and its cause (design 13), shown only while the body is outside fine
    (36.0 to 38.0 C): the same rule as a full bar (recommendation taken, ask
    only with screenshots);
  - breath or suit air as a row of bubbles, only while not full;
  - **every stat, full or not, in the pack** (G7): the pack screen gains a
    small stats panel beside its slots, with health, stamina, hunger and
    water as bars with their numbers, the body's temperature with its trend
    and cause, and the two worn slots (hat, cloak; design 13). It is the one
    place to check a stat that the HUD is hiding;
  - **one warning above the hotbar for heat stroke** (G8): an icon and the
    locale's "Heat stroke", shown from heat exhaustion (39.5 C, design 14)
    until the body is back under 39.5 C, and over the pass-out fade. It is
    the only word the survival HUD shows while alive. The bars and the
    thermometer stay on the left stack;
  - "Winded" and the pass-out fade as states of those bars and the screen,
    not as lines of text: the empty stamina bar flashes;
  - a red edge on the screen for a moment when hit;
  - a plain line on death, "You died", with one button, "Wake up".
- In the stone-and-carved-wood UI (M29), a mockup first, lit at night. No
  explaining text on screen (CLAUDE.md, "Game text is plain"). Every word
  from the locale file (`hud.heat_stroke`, `pack.stats`, `hud.you_died`,
  `hud.wake_up`).
- Mocked up in `docs/mockups/survival.html#hud` (version 2): every state
  by day and lit at night, Explorer beside Survival, including all full
  (nothing shown), hunger at 40%, the stroke warning, and the pack open
  with its stats panel.

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

The owner looked at the curve on 2026-11-26 (survey G6) and picked no
option: "Some sort of js animation would be nice to see this in action. Add
an expand button to lazy load". So the numbers stay as drawn (still a
recommendation taken, not the owner's own numbers), and the Stamina tab
gains the model in action: behind an **Expand** button, made only when it
is pressed, a side view of a player sprinting across a field, swimming a
lake, jumping up rocks, climbing a ledge and resting by a campfire, with the
stamina bar (its strain hatched) and the food bar moving live by this
model and the page's sliders. It stands still until **Play** is pressed.

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
- **Shown** as a thermometer under the left stack's bars (Survival only;
  design 9: only while outside fine, and always in the pack), with the
  trend arrow the design doc's "telegraph cause, trend, threshold" asks for:
  the cause shows as a small icon (sun, wind, wet, fire, snow) beside it.
- **Worn slots.** Two, head and body, beside the pack's grid, in Survival
  only: a straw hat (by hand: 3 wheat; or 6 fibre) and a wool cloak (at the
  workbench: 4 wool, 1 rope). They are not armour and never wear out. They
  show on the player's model in multiplayer, textured like every model.

### 14. Too hot, too cold, and heat stroke (owner, 2026-11-26)

The owner: "Heat strokes will cause damage to health and cause players to
pass out."

Approved as drawn on 2026-11-26 (survey G8: the warm edge from 38 C, then
passing out for 30 s), with: "Just show stroke warning above hot bar. See G
7 for data". The warning is design 9's: an icon and "Heat stroke" above the
hotbar from heat exhaustion (39.5 C) on; the numbers stay on the left
stack.

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
- **Empty**: health - 1 every 12 s (halved from 6 s so the stroke comes
  first: "As built"), and the body heats at the dry rate
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
  | A still (`still`, decision 18) | + 10 a sip, up to 6 | beside a lit campfire, filled at the sea or by rain; drink from it as from fresh water |

- **Salt water is undrinkable** (owner, 2026-11-27: "Keep in mind salt
  water is undrinkable"). Every drink in the table above is fresh: the sea,
  and a bucket filled at the sea, give nothing, and no mission counts them.
  Drinking from a bucket of sea water reads "Salt water" as the sea does.
- **Trying to drink is one way to tell** (owner, 2026-11-27: "So that should
  be one way to test water salinity"). A sip from salt water costs nothing:
  no time beyond the second the sip takes, no water, no health, and the note
  "Salt water" above the hotbar. The owner's "one way" leaves room for
  others, proposed here (recommendation taken, ask only with screenshots):
  - **the look**: the sea's own colour and its surf, against a river's or a
    lake's clearer, greener water, as the water shader already draws them
    (no new rule, only kept distinct);
  - **a salt crust**: a pale rime on the shore's stones and sand just above
    the sea's waterline, drawn on the cells `planet_gen` already marks as
    sea shore, never on a lake's;
  - **the map**: sea and lakes drawn apart on the map (the map already names
    the sea places), with fresh water in its own colour;
  - **estuaries**: a river's last cells where it meets the sea are brackish
    and count as salt until a little way upstream (the cells within 6 of
    the sea along the river's own flow), so the sip test and the look agree
    at a river mouth.
- **Drinkable water on a coast or an island** (recommendation taken, ask only
  with screenshots). A Survival world can start where fresh water is far:
  - a new Survival world's spawn search prefers dry land within 150 m of
    fresh water (a river, a lake or a swamp pool), and falls back to today's
    rule when none is in reach; this is the spawn point only, not terrain, so
    no generator version moves and no saved world changes;
  - rain is fresh: standing in rain with an empty hand held up gives + 2 a
    second, an empty bucket left out fills in a minute of rain, and a still
    collects it too;
  - the still at the fire (decision 18) makes fresh water from the sea for
    an island with no river;
  - a cactus with a straw, fruit, milk and stew still count (the table).

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

The looks drawn on `survival.html#weapons` (the stone spear, the bow and
arrows, the bronze hunting axe and the bronze sword) were approved as drawn
on 2026-11-26 (survey G10), for the animation step: the held models and
their swings go through the `animation` skill (M36) next, mockup first.

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

### 18. Survival's missions: the starter trunk, then paths after the fire (owner, 2026-11-26 and 2026-11-27)

The owner, 2026-11-26: "we're gonna work on survival mode which will add
new branch to starter missions to handle survival." Then, 2026-11-27:

> "Survival: consist of the starter tree today then (in the game) then
> branch off to all survival stuff like eating and drinking, hunting etc."

> "Important note: after making a fire present players with the tree and
> show them options on what branch to take. Then when they click on that
> topic show them the tree branch, then the tree continues along their
> chosen path"

**This supersedes the owner's G11 answer of 2026-11-26** ("Off Open the
pack", "Go right into survival after pack is opened (if survival mode is
active)"), which hung a Survival branch off "Open the pack" and made "Drink
water" the goal right after it. Nothing survival hangs off the pack now:
the first survival missions open with "Make a fire", and the player picks
which to follow.

#### The shape

- **One tree.** A Survival world's tree is the starter tree (`starter-missions`,
  "Missions are a tree that unlocks the tech") exactly as `missions.ron`
  has it today, up to and including "Make a fire": `gather`, `pack`, `axe`,
  `wood`, `planks-rope`, `crafting-table`, `stone-tools`, `fire`, and the
  trunk's `storage` after it. By the fire the stone axe, pickaxe and shovel
  are all made ("so the player has all basic tools first", 2026-11-26), and
  a test pins that `fire`'s ancestors hold every basic tool mission.
- **After "Make a fire" the tree branches into paths.** Each child branch of
  `fire` is a path: four new survival paths, and the starter tree's own
  Fishing, Farming, Herding, Mining and smelting, and Light and building,
  which already hang off `fire`. Cooking and Getting about open after "Go
  fishing", as they do today, and join the paths when they open. The
  survival missions carry `mode: survival` and a `branch` of their path;
  an Explorer world never shows or counts them, and its tree is unchanged.
- **The tree's rules hold**: one parent each, a hidden `after` gate for an
  extra need, unlocks as each mission's reward, ids append-only.
- Mocked up on M33's Survival tab, version 11
  (`docs/mockups/mission-tree.html#survival`).

| Path (branch) | id | Mission | Parent (after) | Objectives | Unlocks |
| --- | --- | --- | --- | --- | --- |
| Eat and drink (`eat-drink`) | `drink` | Drink water | `fire` | drink 3 sips of fresh water (**new** kind `drink`; fresh only, decision 15) | `straw`, `still` |
| | `forage` | Something to eat | `drink` | eat 2 raw foods (fruit, an egg) | - |
| | `cook-meal` | A hot meal | `forage` | cook 1 dish at the campfire; eat it | - |
| | `straw` | A cactus straw | `drink` | craft a straw | - |
| | `cactus` | Cactus juice | `straw` | drink 3 sips from a cactus | `straw-hat` |
| | `still` | Water from the sea | `drink` | set a still beside a lit campfire; drink 3 sips from it (`drink`, what `still`) | - |
| Hunting (`hunting`) | `spear` | Make a spear | `fire` | craft a stone spear | - |
| | `hunt` | Hunt | `spear` | **kill** 1 wild hen, hare or goat (**new** kind `kill`) | - |
| | `cook-meat` | Cook meat | `hunt` | cook 2 `roast-meat` | `meat-stew` |
| | `dress` | Dress a kill | `hunt` | take 1 hide from a kill | `tanning-rack` |
| | `tan` | Tan a hide | `dress` | make 1 leather at the tanning rack | `bow`, `stone-arrows` |
| | `bow` | Bow and arrows | `tan` | craft a bow and 8 arrows | `bone-arrows` |
| | `long-shot` | A long shot | `bow` | kill 1 animal with an arrow from 20 m or more | `bronze-hunting-axe` |
| | `hunting-axe` | A hunting axe | `long-shot` (`smelt`) | craft a bronze hunting axe; dress a kill with it | `bronze-sword`, `bronze-spear`, `bronze-arrows` |
| | `sword` | A sword | `hunting-axe` | craft a bronze sword | `iron-sword`, `iron-spear`, `iron-hunting-axe` (with `iron`) |
| | `fend-off` | Drive off a predator | `sword` | make a predator flee or kill it (**new** kind `drive-off`) | - |
| Fishing (`fishing`) | `rod` ... `sea-fishing` | the starter tree's own | `fire` | as `missions.ron` | as today |
| Farming (`farming`) | `find-seeds` ... `feast` | the starter tree's own | `fire` | as `missions.ron` | as today |
| Shelter and weather (`shelter`) | `first-night` | The first night | `fire` | be out past 20:00 and alive at 06:00 (**new** kind `dawn`) | `bedroll` |
| | `warm-up` | Warm by the fire | `first-night` | warm from cold to fine within a fire's reach (**new** kind `warm`) | `wool-cloak` |
| | `dress-weather` | Dress for the weather | `warm-up` | wear a straw hat or a wool cloak (**new** kind `wear`) | - |
| | `cool-down` | Out of the heat | `first-night` | cool from hot to fine (**new** kind `cool`) | - |
| | `relight` | Relight a torch | `first-night` (`torch`) | relight a torch the rain put out (`wildfire`; **new** kind `relight`) | - |
| | `put-out` | Put out a fire | `relight` | put out 3 burning cells with water (`wildfire`; **new** kind `douse`) | - |
| In the water (`water`) | `long-swim` | A long swim | `fire` | swim 100 m without being winded (kind `swim`, held) | - |
| | `hold-breath` | Hold your breath | `long-swim` | stay under for 15 s and surface (**new** kind `dive`) | - |
| | `soft-landing` | Into the water | `hold-breath` | fall 10 layers or more into deep water unhurt (**new** kind `fall`) | - |
| Herding, Mining and smelting, Light and building | `fence`, `dig-deep`, `build` ... | the starter tree's own | `fire` | as `missions.ron` | as today |
| Cooking, Getting about | `cook-fish`, `map` ... | the starter tree's own | `go-fishing` | as `missions.ron` | as today |

25 survival missions, the 23 of 2026-11-26 and two new ones (`cook-meal`
and `still`), each marked `mode: survival`. What moved from the 2026-11-26 table: `drink`
now hangs off `fire`, not `pack`; `spear` heads Hunting off `fire`, not
`forage`; `cook-meat` loses its `after: fire` (the fire is its ancestor
now); `first-night` heads Shelter and weather off `fire`; `cool-down`,
`warm-up` and `relight` hang off `first-night` (and `warm-up` loses its
`after: fire`); `long-swim` heads In the water off `fire`. Fishing and
Farming keep `fire` as their parent, as decided on 2026-11-26; in a Survival
world they are paths beside the survival ones rather than drawn under one
Survival lane.

- **The spear's recipe** is named by no mission, so by the tree's own rule
  it is open from the start of a Survival world (a spear needs a log, so in
  practice the axe first). Every other weapon's recipe stays locked until
  its mission; in Explorer none of them exist (decision 16). Recommendation
  taken (ask only with screenshots).
- **The still** (`still`, a station, new; recommendation taken, ask only with
  screenshots): made at the workbench from 6 stone, 2 planks and 1 rope, set
  down within 2 cells of a campfire. Use it with an empty hand while it
  stands in reach of sea water (or pour a bucket of sea water in) to fill
  it; while the fire beside it burns it turns 1 sip of salt water into 1 sip
  of fresh water a minute, up to 6, drunk from it like fresh water (+ 10 a
  sip). Rain falling on it adds a sip a minute with the fire out too. See
  decision 15, "Drinkable water on a coast or an island".

#### The path picker

The owner's "present players with the tree and show them options on what
branch to take", as M33 version 11 draws it:

- **When "Make a fire" completes** in a Survival world, the Missions screen
  opens on its own with the picker over the tree. Its heading is "Choose a
  path" over the line "The fire is lit.". (Decision; alternative: a HUD
  prompt "Choose a path (O)" that waits for the player to open the screen.
  The owner's words are "present players with the tree", so it opens; it
  opens only on that moment and on a path's end, never mid-swing.)
- **A card a path** in the stone-and-carved-wood kit (the side panel's stone
  frame, a stone card, carved oak on hover): the path's icon in a slot, its
  name, one plain line ("Fresh water, wild food and a hot meal.", "A spear
  and a hunt, then a bow and blades.", "Seeds, a field and bread."), its
  first mission ("First: Drink water") and its count (0/6). The lines are
  game text, from the locale file (`mission_path.<branch>.line`).
- **A click picks it.** The picker closes, the tree pans to that path with
  its lane lit and its first open mission is **Now**; the HUD's mission
  block follows it, with the path's name over the mission's title.
- **The other paths stay open** (decision). Their missions open by the
  tree's rule and count whenever their work is done, as every open mission
  does today (`starter-missions` decision 9); the choice only says which is
  Now. A player can change path at any time from the picker (the Missions
  screen's **Paths** button). The alternative, locking the paths not taken
  until the chosen one is done, was weighed and not taken: it would take
  away work a player has already done in the world (a fish caught before
  the fishing path was picked would not count), it fights the tree's "every
  open mission counts", and the owner's words ask that the tree "continues
  along their chosen path", which Now does, not that the others close.
- **Now on a path** is the first open mission of the path in file order;
  when it is done, Now moves to the next open one of the same path.
- **When the path has nothing open**, because it is done or because its next
  mission waits on another path's (Hunting's `hunting-axe` waits on `smelt`,
  Shelter's `relight` on `torch`), the picker opens again, headed "Choose a
  path" over "Eat and drink is done." (or, for a path that waits, "Hunting
  waits on Smelt bronze."). It shows every path
  with something open, and a started path that waits keeps its card,
  greyed, with "Waits on Smelt bronze". A finished path is left off. When no
  path has anything open, Now falls back to the first open mission in file
  order, as today (the trunk's "A chest").
- **Later** closes the picker without a choice: the HUD then reads "Choose a
  path" and the picker shows when the Missions screen next opens.
- **Saved.** The chosen path is one more field of the player's `missions`
  record (`path`, a branch id, optional), schema 2, read as none from schema
  1; an older Survival world (there are none yet: decision 1) and every
  Explorer world read none and keep today's Now. In multiplayer each player
  picks their own path; the missions record is already the player's.
- **Explorer** has no picker: its Now is today's rule. (Recommendation taken,
  ask only with screenshots; the picker could come to Explorer later.)

#### Water before the fire

Water and food drain from the start in a Survival world, and the trunk to
the fire takes the first ten to twenty minutes, so the player may need a
drink before any mission says so. They are told by the HUD, not by a
mission (recommendation taken, ask only with screenshots):

- the water bar appears at 50% (decision 9, G7);
- at 50% the line "Thirsty" shows once above the hotbar for four seconds,
  where heat stroke's warning goes (G8), and again at 25% as "Very thirsty";
- the first time it shows in a world, the help line under it reads
  "Drink at a river or lake." (the same words as the drink mission's help);
- a sip of sea water gives nothing and the note "Salt water" (decision 15),
  so a player on a coast learns it the first time they try.

The mockup's bot plays this on a coastal start: "Thirsty" after "Gather
wood", a sip from the sea ("Salt water", nothing taken), then a river.

#### Rewards, help lines, saves, the bot

- **Rewards** are unlocks, as the rest of the starter tree's are: the paths
  open the survival recipes in the order the player needs them. The
  `bedroll` (by hand: 2 hide, 1 wool) is Shelter and weather's payoff: laid
  on the ground it sets the player's spawn (decision 8), so a Survival player
  has a place to wake before they can build a bed.
- **Help lines** are short and plain, in the locale file: "Crouch at a river
  or lake and hold use. Sea water is salt.", "Hunt with the spear. Thrown, it
  hits harder.", "Stand in the shade or get in the water.", "Fill the still at
  the sea. The fire boils the salt out.", "Fires start in the dry season."
- **The weapons' recipes are locked** in Survival until their mission
  (unlocks), the spear excepted (above); in Explorer they do not exist
  (decision 16).
- **Saves.** The paths' ids join the world's `missions` record,
  append-only; an older world is Explorer (decision 1) and never sees them.
- **The bot** picks through the same picker by the player's input. Its order:
  Eat and drink, Hunting (to the long shot), Shelter and weather, In the
  water, Fishing, Cooking, Farming, Herding, Mining and smelting (which opens
  the hunting axe), Hunting again, Light and building (which opens the
  torch), Shelter again, Getting about, then the chest. "Survive the first
  night in Survival" (task 5.1) is the trunk, Eat and drink, and Shelter and
  weather's "The first night".

#### As built (tasks 6.1, 6.2, 6.5; 2026-11-27)

- **The file.** The 25 missions are appended to `missions.ron`'s list with
  `mode: survival` (`pbd_core::missions::Mode`). `Chain::for_mode` cuts a
  world's tree; `pbd_app::missions` keeps every mode's as `AllMissions` and
  sets `MissionChain` to the open world's when it opens, so every reader of
  the starter tree (the stations' locks, the tab, the HUD) sees only its
  mode's missions. A mission for every world may not wait on one of a single
  mode (`resolve` refuses it).
- **Where the file differs from the table above**, to name what the game
  counts: "Something to eat" eats any fruit or an egg (`eat`); "A hot meal"
  is `cook` any dish and `eat` any dish; "Bow and arrows" is a bow and two
  arrow crafts (`recipe:stone-arrows|recipe:bone-arrows`, four arrows a
  craft, so eight); "Dress a kill" and the hunting axe's second objective
  are a hide taken (`gather hide`); "A long shot" is `kill long-shot`, which
  hunting raises beside the animal's own kill; "Drink water" counts
  `fresh|rain|bucket|still`; "A long swim" is `swim water 100` (winded is
  not checked until stamina is in the walker).
- **Pending until built.** The recipes, goods, station and game the survival
  build adds (`recipe:straw`, the weapons, `block:still`, `hide`, the hare
  and the ridge goat, ...) are in the file's `awaited`: a mission naming only
  these is pending (skipped, and met for the ones after it) until they land,
  then counts with no change to the file.
- **The mode.** `pbd_app::missions::mode_of` is the one place the missions
  ask a world's mode: the identity record's `WorldMode` (task 2.1), Explorer
  for every older world. `--mission-mode survival` is a capture instrument
  for a memory-only world.
- **The kinds.** All eleven are live. `dawn`, `dive` and `fall` are raised
  by the game now (`desktop::missions::dawn_watch` and `water_watch`); the
  rest are raised as `pbd_app::missions::Did` by the survival mechanics,
  `drink` named by `missions::drink_name(WaterSource)` (salt names nothing).
  Until then those missions are open and uncounted.
- **The picker.** `Progress.path` (record schema 2); `path_cards`,
  `path_now`, `choose_path`, `choosing` and `offer_after` are the rules;
  `desktop/path_picker.rs` draws it in the side panel's kit (a stone frame,
  an oak lintel, stone cards turning to carved oak under the pointer, a slot
  well for each path's icon). It opens when the player is back in play
  after the offer (never over a station's panel), with the Missions screen
  while a path is to be picked, and from the Paths plaque at the screen's
  top left. A pick re-centres the tree on the path's Now and rings its
  branch plaque. `--pick-path <branch>` is its capture instrument.

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
| The Survival paths and the path picker | hidden; today's tree | decision 18 |
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

## As built: the core rules (2026-11-27)

Tasks 3.1 to 3.8 and 2.1's core half are built in `pbd_core::survival`,
with `assets/config/survival.ron` (design 20's shape, adjusted below). They
are rules only: nothing in the game calls them yet (tasks 4.x wire them), so
no requirement has moved to `openspec/specs/` yet; each moves in the commit
that makes the game do it.

### The API the game calls

- **Config.** `survival::SurvivalConfig::shipped()` parses and validates the
  built-in file; `SurvivalConfig::parse(text)` and `.validate()` for the
  app's own loader (`config.rs`'s `Validated`). `named_goods()` lists every
  good the file names, to check against the goods at load.
- **Mode.** `survival::WorldMode { Explorer, Survival }`,
  `WorldMode::of_saved(Option<WorldMode>)` (absent reads Explorer),
  `.is_survival()`, `.name()` (locale `survival.mode.*`). The app's
  `saves::format::Identity` gains `mode: Option<WorldMode>` (serde default,
  skipped when absent), `Identity::new_in(seed, mode)` and `.mode()`.
- **A player's state.** `survival::Vitals` (serde; the player record keeps
  it): `health`, `food_fp` and `water` (f64: a tick's few ten-thousandths of
  a point would round in an f32), `stamina`, `breath`, `body_c`, `wet_s`,
  `consciousness`.
  - `Vitals::for_mode(cfg, mode)` gives `None` in Explorer;
    `Vitals::new(cfg)`; `wake_up(cfg)` after a death.
  - `tick(cfg, &Conditions, dt) -> Tick`: one step of PLAY seconds (the
    only clock; a closed game changes nothing). `Conditions { effort,
    exposure, head_in_water, in_deep_water, thin_air, in_cabin }`.
    `Tick { damage: Vec<(Cause, f32)>, healed, events, feel_c }`; `Event`
    is `Winded`, `Recovered`, `PassedOut`, `Woke(Wake)`, `Died(Cause)`,
    `Note(Note)`, `HeatWarning(bool)`.
  - `hit(cfg, cause, pts)` (a fall's or a creature's damage; wakes a
    passed-out player), `revive(cfg)`, `eat(cfg, good, stars) -> Eaten`,
    `drink(cfg, &Sip)`, `burst(cfg, Burst::{Jump, Climb(m), SpearThrow})`,
    `limits(cfg, in_deep_water) -> (stamina::Limits, passout::Control)` for
    the walker, `shown(cfg) -> Shown` for the HUD's left stack (design 9),
    `state(cfg) -> BodyState`.
- **Pieces, each usable alone:** `stamina::{Stamina, Effort, Burst}`;
  `fall::{impact_speed(vel, up), fall_damage(cfg, v, g, Landing)}`;
  `hunger::{fills(cfg, good, stars), eaten_water, idle_burn_fp_per_s}`;
  `water::{sip(cfg, WaterSource, water_now) -> Sip, drain_per_s,
  rain_catch, thirst_note, CactusDrunk, Still}`, with `WaterSource::{Fresh,
  Sea, Brackish, Bucket { salt }, Cactus, Still}` and `Sip { water, sip_s,
  note, counts }` (salt gives 0, `Note::SaltWater`, and does not count for a
  `drink` objective); `Note::text()` from the locale file
  (`survival.note.*`); `temperature::{Exposure::from_sample(&Sample), feel,
  target_c, approach, BodyState, Wet, shaded(cfg, from, sun, up, solid)}`
  (the shade march takes a `solid(point)` closure for the column tier);
  `breath::{Breath, Air, head_air}`; `passout::Consciousness`;
  `wear::{Wear, Wearer, Action, Spend, uses_spent, repair_inputs,
  repair_coins}` (an absent `Wear` reads full: `Wear::of_saved`);
  `weapons::{Held, melee_damage, melee_reach_m, Projectile, Missile,
  Struck, Flight}` (a projectile flies in the body's frame as a `WorldPos`
  in f64; `step(cfg, g, dt, hit)` takes a segment test; `kept()` is the
  70% keep by a hash of the launch; on `tests/origin_guard.rs`);
  `hunting::{Quarry, strike, dress, Blade, carcass_lies, Hunted}`
  (`Quarry::Protected` from `fauna.ron`'s status, `Quarry::Kept` for a kept
  or bought animal).

### Where it differs from the plan, and why (recommendations taken)

- **An empty water bar's damage is halved, so the stroke comes first.**
  At design 15's 1 pt every 6 s and design 14's 0.2 a second, a body whose
  water was EMPTY in the desert sun died at about 7.3 minutes, before it
  reached heat stroke's 40 C at 7.8, so the stroke design 14 draws could
  never happen to a player out of water in the sun. Recommendation taken
  (ask only with screenshots): an empty water bar takes 1 pt every 12 s
  (`water.empty_drain_s_per_pt`, 6 to 12), and nothing else in the tuning
  moves. The empty bar is still shown the whole way down and still kills,
  in about 20 minutes on its own; in the desert sun the player passes out
  at about 7.8 minutes, still alive. Tests
  `health::tests::an_empty_bar_drains_a_point_every_ten_and_twelve_seconds`
  and `body::tests::an_empty_water_bar_in_the_desert_sun_has_the_stroke_before_it_kills`.
- **The day example runs from a full food bar**, as the mockup's day case
  does: starting at 80 fp, food reaches 0 while the bar still refills, and
  design 12's starving rule (free refill to half), which the planning model
  leaves out, makes it 112.7, not 112.9.
- **A hide is not a drop.** `hunting.game.<species>.hide` is apart from
  `drops`: the kill drops the rest at once, dressing gives the hide. An
  undressed carcass gone after `carcass_s` takes its hide with it.
- **New numbers:** suit air refills in 5 s (`suit_air.refill_s`); water
  wakes at 50 (`water.wake`); a steel axe hits 22 (iron's step on); a rod
  hits as bare hands; a winded swimmer cannot hard-swim (it swims plain).
  "Thirsty" and "Very thirsty" are notes the core raises
  (`water.thirsty_at`, `very_thirsty_at`).
- **Meals' `fills` live in `survival.ron`**, not in `cooking.ron`, so the
  kitchen's file is unchanged in Explorer.

## Combat: the swings, the hunt, the throw and the town (2026-11-27)

The owner, 2026-11-27: "continue building all of survival phase 4". That is
the request to build tasks 1.2, 1.6, 4.3, 4.4 and 4.5 after their mockups,
so the game's motion follows the drafted mockups (`survival.html#weapons`'s
Swing view and `survival.html#hunt`) and is retuned once the owner reviews
them: every number below is a tunable in a validated file, and the mockups
print the RON their sliders make. Everything here is a recommendation taken
(ask only with screenshots) unless it quotes the owner.

### C1. The swing is the rule's; the drawing follows it

- **The rule.** `pbd_core::survival::melee::Swing` is one swing's clock: a
  press starts it, its **hit lands `hit_share` of the weapon's `time_s`
  after the press** (`weapons.hit_share`, 0.45), and holding the button
  starts the next swing one `time_s` after the last (one steady beat, the
  animation skill's rule). A tool or bare hands swing on `weapons.tool_swing_s`
  (0.55 s), today's stroke period. The hit is tested **at the hit moment**,
  along the aim then (staging: the blow lands where the player looks when
  it lands), against what is within the weapon's reach, measured level
  along the ground from the eye (so an animal at the feet is in reach), and
  a swing sweeps an arc, so it meets a body the aim passes within 0.35 m
  of (`SWING_SLACK_M`).
- **The drawing.** `held.ron`'s `weapon_swing` keys each swing's poses
  (rest, top, coil, hit, recoil) on the fist's arc about the shoulder, with
  the weapon's own pitch and roll on top, blended by the skill's curves:
  windup eased out, anticipation a slow ease into a held pose, the hit eased
  in (t cubed, fastest at contact), a held frame, a quick recoil with an
  overshoot, then the ease home. The stage times are fitted to the rule:
  windup and anticipation scale so the hit's end lands on the rule's
  `hit_s`; the hold, recoil and recover scale into the rest of the period.
  The hit stage is never shortened. Drawn only: nothing reads the pose.
- **Per weapon** (the mockup's Proposed): the spear **thrusts** (drawn back
  along the shaft, then driven forward 0.4 m along the aim); the hunting axe
  **chops** overhand, a bigger raise than the tool stroke and faster; the
  sword **slashes** from the right shoulder down across to the left; a
  **thrown spear** is raised over the shoulder while the right button is
  held and leaves the hand on release; the **bow** is raised and drawn while
  the left button is held, and looses on release. Today's stroke (the tool's,
  `held.stroke`) stays as it is for the tools and bare hands.
- **Throwing and loosing** (design 16). A thrown spear leaves on release
  after at least `throw_min_s` (0.25 s) raised: it is taken from the hotbar
  slot and flies as a `weapons::Projectile` from the eye along the aim; it
  lands as a drop. A bow looses the first arrow in pack order (stone, bone
  or bronze) on release, at the share of `bow.draw_s` drawn; an arrow that
  lands is a drop 70% of the time (`Projectile::kept`). Both fly in the
  body's frame in f64 and are tested each step against the creatures and the
  column tier; the game draws them at the local point the origin gives.

### C2. The hunt (design 6)

`pbd_core::survival::predator::Hunt` is a pure state machine stepped with
what the predator senses (the player's distance, night, whether the player
is downed, its own health share, whether it was just struck). **In an
Explorer world it never leaves `Wander`**: the step returns at once.

| State | Enters when | Does | Leaves when |
| --- | --- | --- | --- |
| Wander | start; calm again | the herd's own pottering | the player within `notice_m` (x `night_mult` at night) |
| Stalk | noticed | walks at `stalk_mps`, low, toward the player | within `warn_m`: Warn; past `notice_m` x 1.5 for `give_up_s`: Wander |
| Warn | close | stops, crouches, growls, for at least `warn_s` (1.5 s or more) | the hold done and the player within `lunge_m`: Attack; the player backs past `notice_m`: Wander |
| Attack | warned, or struck | lunges at `lunge_mps`; a bite within `bite_m` at the lunge's end deals `damage`; backs off to `warn_m` and re-warns for `rewarn_s` | below a third of health: Flee |
| Flee | below `flee_below` of health, or the player out of reach for `give_up_s` | runs away at the species' run speed for `flee_s` | then Wander, and will not notice for `calm_s` |

- A predator never attacks a player who is downed (passed out or dead): it
  leaves (Flee).
- A struck predator with more than a third of its health goes straight to
  Attack (it fights back); the warning before the *first* attack is never
  skipped when it was not struck.
- Numbers (`survival.ron` `predators`): red fox notice 14 m, warn 5 m, lunge
  3.5 m at 8 m/s, bite 1.3 m for 6, warn 1.8 s; dust hound notice 18 m,
  warn 6 m, lunge 4 m at 9 m/s, bite 1.4 m for 10, warn 1.5 s; night x1.5;
  re-warn 0.8 s; flee below 1/3 for 10 s; calm 30 s; give up 8 s.
- **Predators hunt the player only**, never the herd, in this change.
- **Health and kills** (design 17): a wild game animal's health is
  `hunting.game`'s, kept on the yard's animal while it is near; a strike goes
  through `hunting::strike`. A kill drops its meat and bones at once as world
  drops where it lies; the carcass lies `carcass_s`; a blade swung at the
  carcass dresses it (`hunting::dress`, a hunting axe for the hide and one
  more meat, another blade the hide half the time by the kill's hash), and
  the hide drops. The kill writes the `hunted` record (its herd square, its
  member, the world day), and the yard leaves that member out until
  `herd_back_days` have passed. A kept or tamed animal is `Quarry::Kept`:
  a strike does nothing.
- The animation (the skill's creature rules): the stalk is the walk cycle
  low and slow, the head down level with the back; the warn is a crouch
  (the body down 30%, the head low and forward) with the tail held; the
  attack is the four stages on the body (gather back, hold, spring at
  full speed, land with a squash, recoil back); the flee is the run with the
  tail down.

### C3. Townsfolk fight back (design 7)

- **In Survival**, a swing that lands on a townsperson does them no harm (the
  townsfolk are derived from the settlement record, never killed in this
  phase) and:
  - drops the town's **standing** by `townsfolk.struck_standing` (10), on a
    `standing` record per town (`pbd_core::survival::standing`, from 0,
    clamped to -100..100, absent reads 0);
  - a guard (a person with a spear) **fights**: it and every guard within
    `call_m` (40 m) come at `guard_mps` (3.2 m/s) and strike within
    `guard_reach_m` (1.8 m), a `guard_windup_s` (0.7 s) telegraph, then
    `guard_damage` (8, `Cause::Townsfolk`), one blow every `guard_every_s`
    (1.4 s);
  - anyone else **flees** from the player for `flee_s` (6 s), then goes back
    to their post.
- The fight ends when the player is past the town's walls (`walls_m` from
  its centre plus `leave_m`, 12 m) or is downed; the guards walk back.
- Standing is stored and shown nowhere yet; trade by standing is a
  follow-up (`townsfolk`'s consequences).
- **In Explorer** a swing at a townsperson does nothing at all.

### C4. No player strikes another

The aim finds creatures and townsfolk only; `melee::Target::Player` is
refused by the rule (`melee::lands`) in every mode, so no client can hurt
another. The server (design 10) decides every strike, kill and drop; in a
joined world the client sends nothing new yet (task 4.7 runs the
two-machine check).

### Combat as built (2026-11-27)

- **The rules** are `pbd_core::survival::{melee, predator, standing}` with
  `survival.ron`'s new `weapons.hit_share`, `tool_swing_s`, `throw_min_s`,
  `predators`, `hunt` and `townsfolk` blocks (validated: bite < lunge <=
  warn < notice, every warning at least `hunt.warn_min_s`, a `hunting.game`
  row for every predator).
- **The game** is `pbd_app::combat` (the swing's rule each frame, the claim
  on the click, the hit at the rule's moment, the flights, the town fight,
  the bites), `husbandry::hunt` (health, the hunt's drive, the carcass and
  its dressing, the `hunted` record and the herd's return) and
  `held::weapon_swing` (the drawn swings, `held.ron`'s `weapon_swing`, the
  mockup's printout pasted in; a test holds the file to the code's
  defaults). The digging reads `CombatClaim` and leaves a claimed click
  alone.
- **Where it differs from the plan** (recommendations taken, ask only with
  screenshots):
  - A tool's or bare hands' strike is drawn with today's tool stroke on the
    rule's own beat: the dig's beat with the rule's swing as its block (a
    block of `hit_s`, a pause of the rest of the period), so the drawn
    impact lands on the rule's hit frame (`combat::tool_beat`; test
    `a_tools_drawn_impact_lands_on_the_rules_hit_frame`). (Round 2; it was a
    frame or two off on the free beat before.)
  - The warning growls (`predator_growl`, a CC0 recording through the
    `sound-effects` pipeline) once, as a stalk turns to a warning; a re-warn
    between lunges is silent.
  - The lunge is drawn as the Hunt tab draws it: the gather (it stops, low,
    for the first quarter of the lunge), the spring (an arc 0.12 m high,
    nose up then down) and the landing squash, from `survival.ron`'s
    `hunt.lunge_draw` (drawn only). The body's own squash and stretch are
    the mockup's; the game lowers and lifts the whole rig. A carcass lies as
    a resting animal does.
  - The bow's string is drawn back 0.42 m at full draw and snaps home on the
    loose, the bow turned to face its belly ahead while raised; no arrow is
    drawn on the string in hand yet.
  - Weapons wear as tools do (design 11): a landed hit spends a use of the
    weapon (`weapon_wear` record, one per weapon kind, an old save full),
    an arrow loosed a use of the bow; at zero it breaks with the
    `tool_break` crack and "{tool} broke", a broken weapon hits as bare
    hands and a broken bow does not loose. A tool's landed hit spends a use
    of the tool. A weapon is repaired as a tool is ("Weapon repair" below).
  - A thrown spear or a loosed arrow still in the air when the game closes
    is lost: the slot it left is saved at the throw, its drop at the
    landing.
  - A projectile falls by the home planet's surface gravity.
  - The bow looses the first arrow it finds in the order stone, bone,
    bronze.

## As built: survival in the game (2026-11-27)

Tasks 2.1 (its UI half), 4.1, 4.2, 4.6, 4.9 and parts of 4.8, 6.3 and 6.6.

- **The mode on the new-world screen.** Under the name, two oak plaques,
  Explorer and Survival, the picked one lit; NEW WORLD makes the world in it
  (`saves::create_in`, `Identity::new_in`). An Explorer world's identity is
  written as before modes, so nothing in an Explorer save changes. Each row
  of the world list says its mode under the name, beside when it was played.
- **The server ticks the bars** (design 10). `pbd_server::Player` gains
  `vitals: Option<Vitals>`; `Server::survive` ticks them by the core's
  `Vitals::tick` and `Server::survival_act` lands a hit, a meal, a sip, a
  jump or a climb, a wake. They are a world record, kind `vitals`, one per
  player, journaled through `Store::commit_record`: at once for anything
  that happens to the player, and every `VITALS_EVERY_S` (5 s) of play
  while they only tick (recommendation taken, ask only with screenshots: a
  line every frame would be sixty a second; five seconds is the most a crash
  can take back). Single player's in-process server reads them from the
  world's records when it is made (`WorldSave::survive`, `survival_act`,
  `vitals`); an old save has no record and reads Explorer, so it has none.
- **The walker** (`pbd_app::survival`, `walking.rs`). The walker reports its
  effort (sprint, swim, swim with sprint, tread), its jumps and the water at
  its head in `Activity`; the tick reads climbs (the feet rising a layer
  while they stand) and falls (the fastest downward speed along up while in
  the air, spent by `fall::fall_damage` on landing, on the ground or in
  water with its depth) off its motion, with the body's own gravity. It
  obeys `WalkerLimits`: winded (no sprint, `swim_speed_mult`, jumps at
  `jump_mult` of the height, no climbing out of water, a swimmer that floats
  up), passed out (no controls, floating face up in deep water), dazed (no
  sprint, `walk_mult`) and dead (no controls).
- **The conditions** at the player: the atmosphere's sample at the player
  (`Exposure::from_sample`), the shade by `temperature::shaded` with the
  column tier as its `solid` closure (`PlanetContact::column_material`),
  shelter as no open sky over the head, the placed campfires within 6 m
  (looked for twice a second), what is worn, and wet while in water. A
  craft's seat is a cabin.
- **Worn slots.** The right button with a straw hat or a wool cloak in the
  selected slot puts it on (a `worn` record, saved with the hands); what was
  worn there comes back. Shown in the pack's stats panel when that lands.
- **The HUD** (design 9, the approved mockup): the left stack of short stone
  bars with the mockup's icons (`assets/ui/survival/`, copied unchanged),
  hidden while full, hunger and water only at half or lower, a bar flashing
  when it is low (health under 30%, food under 15%, water dry, stamina
  winded), stamina's strain shown as a paler band past its fill; the
  thermometer only outside fine, with the reading, the trend arrow and the
  cause; breath as ten bubbles over the hotbar while not full; "Heat stroke"
  above the hotbar from heat exhaustion on, with a warm wash on the screen;
  the line above the hotbar for the core's notes ("Salt water", "Thirsty",
  "Very thirsty") and "Pickaxe broke"; a red edge for a moment when hit; the
  pass-out's fade to dark. "You died" with Wake up is a screen of its own
  (`Screen::Dead`), so it holds the pointer and Escape does not leave it.
  The pack's stats panel (G7) is not built yet: the pack screen is another
  change's in this round, and it follows.
- **Death and the bag** (design 8). At zero health the whole pack, the
  hotbar included, goes into one `pack-bag` record at the feet, body-relative
  f64 (`WorldPos`), written in one line with the emptied slots
  (`store_with_hands`); it is drawn turning slowly over the ground as its
  first stack's card, lit by the field. Walking within 1.6 m of it takes
  everything that fits, and what does not stays in it. The tool slot is not
  the pack and stays with the player (recommendation taken, ask only with
  screenshots). Wake up wakes with full health, half food and half water, at
  the spawn record a laid bedroll writes (`survival::spawn_point`, the hook
  for task 6.3: the bedroll is a good in this build, and laying it waits for
  its laid model) or else at the world's spawn.
- **Drinking** (design 15). F with an empty hand at water within 3 m, or
  standing in it, takes one sip (the interact table's last row, "Drink
  water"). Where the water is: the sea (the sea's level and not
  `fresh_water_at`), fresh (a river, a lake, a swamp pool, or water gathered
  off the sea's level), brackish (a river's water standing level with the sea,
  at its mouth; the design's "6 cells along the flow" is approximated by that
  level). Salt gives nothing and "Salt water"; a fresh sip raises the bar and
  counts the missions' `drink`. Holding the right button with an empty hand
  in rain under open sky catches it (`rain_catch`), counted as `drink rain`.
  Not built yet: the still, a bucket's sip, the cactus and the straw, and the
  spawn near fresh water.
- **Wear** (design 11). A `wear` record holds each tool's uses left (no
  entry reads full, so an old save is full). A finished dig, fell, till or
  catch, read off the missions' own `Did` messages, spends the tool in hand;
  the record is written in the same frame as the action's own line, the next
  line after it, not in it (the action's line is the edit's, and widening it
  would change the save format for every edit). The tool slot gains its bar
  (red below a tenth); a broken tool's icon is greyed, it digs as bare hands,
  and "Pickaxe broke" shows once. The workbench and the smith's repair are
  `survival::repair_tool` and the core's `repair_inputs` / `repair_coins`;
  their buttons are not on a panel yet.
- **What the missions count, raised here.** A sip of fresh water raises
  `drink` by `missions::drink_name` (salt none), rain `drink rain`; a body
  back to fine from hot `cool heat`, from cold `warm fire` within a fire's
  reach or `warm other`; a hat or a cloak put on `wear <good>`. A Survival
  death clears the dawn watch, so a night counts only if lived through.
  `kill`, `drive-off`, `relight` and `douse` wait for hunting, predators
  and the wildfire's torches.
- **The recipe book follows the world.** The open world's mode sets
  `RecipeBook.survival` each frame it differs, so a Survival world offers
  the spears, the bow, the tanning rack, the still, the bedroll and the stew,
  and an Explorer world does not.

## The rest of the non-combat pieces: the plan (2026-11-27)

Tasks 4.2 (the bag's marker and look), 4.6 (the pack's stats), 4.8 (the
repair buttons and the break sound), 6.3 (the bedroll) and 6.6 (the still,
the bucket, the cactus, the river mouth and the spawn), and the `drink`
mission's help line. Written before the code, on the owner's "continue
building all of survival phase 4". Every choice not quoted from the owner is
a recommendation taken (ask only with screenshots); the captures go with the
commits.

- **The pack's stats (G7).** A third panel in the pack's row, left of the
  pack, on the kit's stone frame, Survival only: the four bars with their
  numbers (heart, stamina, food, water; the HUD's icons and colours, the
  mockup's `.prow`), the thermometer with its reading, trend and cause, and
  the two worn slots (hat, cloak) as slot wells with the worn good's icon.
  Every bar shows whatever its height: this is where a hidden stat is read.
  Explorer's pack has no panel. Its words are the locale's `pack.stats`.
- **Repair at the workbench.** While the workbench is open in a Survival
  world and the tool in hand is worn, a small REPAIR panel beside the
  workbench's, in the station panels' own row as the pack's craft panel
  stands beside the pack (round 2; it stood over the screen's top first): the tool in hand's icon at its tier,
  its uses left as the slot's bar, and the price, `repair_inputs` of the
  recipe that made that tool at that tier (half its head, rounded up), drawn
  as the recipe rows' icons with their counts. The plaque is greyed while
  the tool is full or the pack lacks the price; pressing it takes the price
  and restores every use in one saved line (`survival::repair_tool`). A
  tool no recipe makes (none today) shows no price and cannot be repaired.
- **Repair in a town.** On the talk panel of a town's trader or smith, in a
  Survival world, while the tool in hand is worn: a REPAIR plaque with its
  price in coins (`repair_coins` of `values.ron`'s `tool:<code>` and the
  share worn). Pressing it spends the coins (the purse's `Bought`: a repair
  is bought) and restores the tool; the wear and the purse go in one line
  (`survival::places::repair_for_coins`; the two records are stored in one
  call, two journal lines back to back). Short of coins, the talk says the
  trade refusal's own "Not enough coins". REPAIR is one of the talk panel's
  own buttons, between TRADE and BYE, its price in coins after it (round 2;
  it stood over the screen's top first).
- **The break sound.** A short dry crack, cut from a CC0 recording by the
  `sound-effects` skill (`tools/audio/cc0_cues.json`, its source named in
  `assets/audio/sfx/SOURCES.md` and `AUDIO_SOURCES.md`), played once at the
  player when a tool breaks, as the hotbar's other one-shot sounds are.
- **The still** (decision 18). Its record is `still`, keyed by the cell and
  layer as a station's (`station::record_id`), holding the core's `Still`. F
  on it (the interact table's new `UseStill` row, the survival reader's): a
  bucket of sea water in hand is poured in (4 sips of salt), and with an
  empty hand a fresh sip is drunk as from a river (counted `drink still`).
  Nothing to drink says "Empty"; salt in it with no fire says "Needs a fire".
  Every still on the fine tier is stepped once a second of play by the
  core's `Still::step`: lit when a campfire stands within
  `still.fire_reach_cells` cells (a placed campfire burns until the
  wildfire change's douse, which this reads when it lands), raining when the
  weather rains on it under open sky. A step that changed what it holds is
  written at once; the boil's seconds are kept in memory between, so a
  reload loses at most part of one sip's boil. A still off the fine tier
  does not boil: world time passes only where the world is played. Its fresh
  water is drawn in its cup (round 2): a disc of water textured by
  `tools/art/still_water_texture.py`, rising up the cup's inside wall from
  its floor to under its rim as the sips come, gone when it is drunk dry.
- **A bucket's sip.** A bucket filled at the sea is a good of its own,
  `sea-bucket` (appended: a bucket of sea water; every `water-bucket` an old
  save holds stays fresh). The empty bucket takes the sea's water as a
  `sea-bucket` and any other water as a `water-bucket`; either pours as
  water. F with a full bucket in hand and nothing else to do drinks a sip:
  four sips empty a fresh bucket (`drinks.bucket.sips`), counted in a
  `bucket` record of sips taken, and the fourth hands back the empty bucket
  in the same line; a sea bucket gives nothing and "Salt water". Its icon is
  the water bucket's drawn by `tools/gen_item_icons.py` with the sea's
  grey-green water. Sea water poured out of a sea bucket stays salt (round 2): each pour
  is a `salt-water` record (where it was poured, body-relative f64), and
  water within four cells of one reads as the sea's wherever it stands
  (`places::water_source_in`); the empty bucket filled there is a sea bucket
  again.
- **The cactus and the straw.** With a straw in hand, F on a standing
  desert cactus within 3 m (the cell whose record `tree_kit::grows_cactus`
  says, its trunk the eye ray passes within 0.6 m of) drinks a cactus sip
  through `CactusDrunk::drink`; the `cactus` record, keyed by the cell, is
  written with it. A dry cactus says "Dry". The straw wears one use a sip
  (`wear.straw_uses`, under `straw` in the `wear` record); when it breaks it
  is used up (one straw leaves the stack, the next is new), since a straw is
  a stack of goods and not a slot's single tool.
- **The river mouth, measured along the flow.** Brackish water is river
  water within 6 cells (6 x 2.833 m) of the sea along the river's own flow,
  as the design says: from the sip's point, step one cell downstream
  (`rivers::at`'s `upstream`, reversed) at a time; if the sea (water the
  generator did not make fresh, under the sea's level) is met within six
  steps, the water is brackish. This replaces the stand-in "a river level
  with the sea". It reads the generator and writes nothing, so no
  generator version moves.
- **The spawn near fresh water.** Only a NEW Survival world's first
  arrival (`walking::arrive_new`, a world nobody has played): after the
  usual start, if no fresh water (`fresh_water_at`) lies within 150 m, the
  search walks rings out to 3 km for the nearest fresh water, then takes
  the usual level start nearest that water, or the nearest dry land to it,
  if that is within 150 m of it; otherwise the usual start stands. An
  Explorer world, a world already played (its saved pose) and a world
  reopened never call it, so no spawn a world has ever had moves; a test
  holds both.
- **The bedroll** (decision 8, task 6.3). The right button with a bedroll in
  hand, on foot, lays it on the ground the eye is on within reach: the
  bedroll leaves the hand, and a `bedroll` record (where it lies,
  body-relative f64, and which way it faces) and the `spawn` record are
  written in one line with the hands. One bedroll is laid at a time: laying
  another takes the old one back into the pack in the same line. F on a laid
  bedroll takes it up (and clears the spawn). It is drawn as one shaped
  piece: a rolled-out wool and hide pad with a rolled head end, its normals
  smooth across the roll and the pad's long edges, textured by a 32 px hide
  and wool texture drawn by `tools/art/bedroll_texture.py` (manifest row),
  sampled nearest, lit by the field as the ground. Death wakes the player
  at it.
- **The dropped bag** (task 4.2). The card at the feet becomes a standing
  icon: a dropped item lying in the world is an icon (CLAUDE.md), so the bag
  is the pack's own icon, a pack sack drawn at 16 px by
  `tools/gen_item_icons.py`, on a card twice a drop's size that stands
  upright on the ground and turns to the eye, lit by the field. On the map
  every bag lying in the world has a marker: a pack icon in the player's
  colour beside the waypoints (`MapMarker::Bag`), the newest two shown,
  drawn over the player's arrow and standing on its point as a pin does, so
  a bag at the feet is not hidden under the arrow (round 2).
- **Help text.** The `drink` mission's help: "Press F at a river or lake.
  Sea water is salt."

## Weapon repair (2026-11-27)

Weapons wear as tools do (design 11, "Combat as built"), and design 11's
repair covers them the same way; this is the plan, written before the code.
Recommendations taken (ask only with screenshots).

- **What is repaired.** The thing in hand: a weapon in the hotbar's
  selected slot (`held::weapon::weapon_of`) before the tool slot's tool, as
  the swing reads them. Both repair panels (the workbench's REPAIR beside
  the workbench, the talk panel's REPAIR at a town's trader or smith) show
  a worn weapon exactly as they show a worn tool: its icon, its wear bar,
  its price.
- **At the workbench**: half the head of the recipe that makes the weapon
  (its `outputs` name the good), rounded up, as for a tool. A recipe whose
  every input is a handle good (the bow: planks and rope) has no head; its
  price is then half of every input, rounded up, so no repair is free
  (`wear::repair_inputs`, which applies the same fallback to any tool, none
  of which has such a recipe today).
- **In a town**: the weapon's row in `values.ron` (its good code) times the
  share worn, rounded up, paid from the purse; the `weapon_wear` record and
  the purse stored together.
- **Saving.** The `weapon_wear` record (schema 1, unchanged) takes the
  weapon's full count, in the same line as the hands at the workbench and
  as the purse in town. A save without the record reads full, as before.
- **Words.** "Repaired {tool}" names the weapon by its good's name; no new
  locale key.

## The bot run (task 6.4, 2026-11-27)

The bot is the showcase bot (`step-videos`, `desktop/showcase.rs`): it plays
`assets/showcase/phase-4-survival.ron` through the player's own inputs and
logs `mission done: <id>` as each mission completes. What it needed, written
before the code (recommendations taken, ask only with screenshots):

- **A Survival world.** The shot list's `survival: true` makes the
  showcase's new world a Survival one (`saves::create_in` with
  `WorldMode::Survival`, or a memory-only world with that mode), so the
  missions, the recipe book and the bars are Survival's from the start.
- **The trunk** is the phase-1 trunk's shots to "Make a fire", at the same
  start (a pasture with the sea near).
- **`Path(branch)`**: the Missions screen (opened by the fire, or with O),
  the picker up (from its Paths plaque if not), the path's card pressed as
  the pointer presses a control, the screen shut. A player's clicks, no
  state set.
- **`Empty`** and **`Drink(n)`**: bare hands and an empty hotbar slot by its
  number key, then F at the water ahead, a sip at a time.
- **`Hunt(species, until)`**: the weapon in hand, the nearest wild animal of
  the kinds named walked up to and looked at, the left button held so the
  swing lands on it; a kill is struck again where it lies to dress it, and
  its drops are walked over. The bot reads where animals are, and whether a
  kill lies dressed, from the yard as a player sees them (`husbandry::Seen`
  gains `dead` and `dressed`).
- **The order** is design 18's: the trunk, Eat and drink (the drink, the
  straw), Hunting (the spear, the hunt, cooked meat), Shelter and weather
  (the first night, the clock sped and captioned), In the water (the long
  swim). The rest of the order (the still, the cactus, the bow, the long
  shot, fishing, farming, the smelt, the torch's relight) is left for later
  runs.
- **In the cloud** the run renders on lavapipe (no GPU) at a reduced window
  (`PBD_WINDOW=960x600`, `--render-scale 50`) about one frame a second; the
  bot's clock steps at most 0.1 s a frame, so a run takes many times its
  game time. Its stills say so.

## Questions for the owner (with screenshots), 2026-11-26: answered

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
| G11 | Where the Survival branch hangs: off "Open the pack" (answered, then superseded 2026-11-27: off "Make a fire", as paths) | off the pack; off "Make a fire" | off the pack: water empties first, so drinking is taught first | `m33-survival-fit.png`, `m33-survival-hunt.png`, `m33-survival-hunting.png`, `m33-survival-heat.png` |

(G9 is `wildfire`'s.)

**Answered by the owner, 2026-11-26** (the survey's "Already decided"):

| Id | The owner's answer | Where it went |
| --- | --- | --- |
| G6 | no option picked; "Some sort of js animation would be nice to see this in action. Add an expand button to lazy load" | the numbers stay as drawn; the Stamina tab's animation behind Expand (design 12) |
| G7 | "a vertical stack at the left edge"; "Don't need to show food or water till it's 50% or lower. You can check in pack these stats at any time..if stats are full hide them." | design 9: the left stack, full bars hidden, hunger and water from 50%, the pack's stats panel |
| G8 | as drawn; "Just show stroke warning above hot bar. See G 7 for data" | design 9 and 14: the warning above the hotbar |
| G10 | approved as drawn, for the animation step | design 16 |
| G11 | "Off Open the pack"; "Go right into survival after pack is opened (if survival mode is active)" | **superseded 2026-11-27** by the owner's "Survival: consist of the starter tree today then (in the game) then branch off to all survival stuff like eating and drinking, hunting etc." and "after making a fire present players with the tree and show them options on what branch to take": design 18 now opens the survival paths at "Make a fire", with the path picker |

**Recommendations taken (ask only with screenshots)**, not the owner's own
words: every number in design 12 to 17 and 20; tool work spends no stamina;
cold hurts but never knocks the player down; a winded swimmer floats; two
worn slots (hat, cloak) that are not armour; drinking by hand at fresh water;
the straw from 2 fibre with 30 sips; no hunting, weapons or worn slots in
Explorer; kept and bought animals cannot be struck (butchering a follow-up);
a kill's hide needs dressing; herds back after 3 world days; the survival
paths as branches of the starter tree, not a third tree; rewards as unlocks,
the bedroll as its payoff; (2026-11-27) the picker opening on its own at the
fire, the paths not taken staying open, the waiting path's card, Explorer
without a picker, the spear's recipe open from the start, the still, the
"Thirsty" warning before the fire, the other ways to tell salt water, and
the spawn near fresh water; and, after G7 and G8, health and stamina hidden only
when full, the thermometer shown only outside fine (36 to 38 C), the stroke
warning from heat exhaustion (39.5 C), and the pack's stats panel holding
the worn slots.

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
