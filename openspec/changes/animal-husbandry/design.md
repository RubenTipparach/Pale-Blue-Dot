# Design: the animal kingdom and husbandry (animal-husbandry)

## Context

The proposal quotes the owner. This design fixes the numbers the mockup
(`docs/mockups/husbandry.html`, "Pens, Nests and Pedigrees") shows and how each
rule would enter the engine, so the page, the data and the core can never
disagree.

What exists, measured on `main` and in the other mockups:
- **The game's fauna is eight fish.** `pbd_core::fauna` holds the species,
  their water classes and temperature windows, and schools of boids;
  `assets/config/fauna.ron` lists them under `home` in an append-only order,
  because a caught fish is saved as `Item::Fish(index)`. A body with no species
  has no fish; no species is on two bodies (`openspec/specs/world/fauna`).
  There is no land animal, no egg and no breeding in code.
- **M7, animals** (`animals.html`, no OpenSpec change yet): 24 Tenebris land
  species by biome (fields, desert, jungle, swamp, mountains, tundra, beach),
  each a body plan and a hide drawn by `tools/art/animals_art.py`. It proposes:
  taming by feeding a trust meter (grain for woolbacks and hens, turnips for
  plodders, kale for ridge goats, dates for sand striders); a tamed animal as a
  stored record with id, species, seed and owner; wild animals derived and
  never saved; a fleece every 4 days, an egg a day, milk a day; breeding when
  two share a pen, the young's seed mixing its parents'; the sand strider as
  the one mount; hunting optional, so every drop but meat and hide comes from
  a tamed animal; rosters append-only.
- **M25, water life** (`ocean.html`, `ocean-environments`): 41 more species.
  Reed dabblers (ducks) lead ducklings, sleep on the water, and give eggs and
  feathers as drops; ducks, reef turtles, sea otters and seals cannot be caught.
- **M9, fish lab** (`fish.html`): an individual is rolled from a species and a
  seed, 85 to 125% of the species' length, with a rare colour morph 1 in 64;
  a caught fish keeps its seed in its slot line.
- **M1, planets** (`planets.html`, `star-systems`): Sequoia's sparkit,
  shellkin, crane, raptor and sea serpent (built in Tenebris's `fauna.rs`) and
  gloomhopper, sporeling, barkstrider, mossback (planned there, drawn on M1);
  six more planned there with no design yet (fernwhistle, canopyclaw, mirefoot,
  duskstalker, silthog, slatehorn). Wick's tallowback and dimwing; Fallow's
  drover, scythewing and burrowkit. Every other body is airless and lifeless.
- **M8, farming** (`farming.html`): wheat 3 a cell in 6 days, turnips 2 in 4,
  frost kale 2 in 5; compost from the pen's muck; a pen of three woolbacks and
  three hens beside the plot; world time passes only while the world is played,
  so a farm's growth is a function of world time.
- **M5, townsfolk** (`npcs.html`): a farmer who sells grain and eggs.
- **world-persistence**: a journal whose lines name their author (`@p`,
  `@w<process>:<record>`, `@c`), `rec <id> <body>` lines carrying a record
  whole, world processes run by a steward each in-game hour, and a check that
  refuses any world proposal touching what the player touched.

## Goals / Non-Goals

**Goals:**
- One list of every animal on every world, and one rule for which can be kept.
- A farm that reads at a glance, by day and lit by its own lamps at night,
  with animals that act like themselves.
- Eggs that hatch, young that grow, and offspring that visibly take after both
  parents, so breeding toward a goal is a game.
- Nothing a player tamed, bred or named is ever lost to a later change.

**Non-Goals:**
- Game code. This change is the write-up and the mockup.
- Recipes and dishes. Meat cuts, eggs, milk and honey are products here; what
  is cooked from them is the cooking mockup's, drafted beside this one.
- Fish keeping. Tanks are M9's; fish appear in the Kingdom only.
- Hostile animals. M7's recommendation stands: nothing attacks the player.
- Frame cost. Not measured (a cloud container, SwiftShader).

## Decisions

### 1. One taxonomy, by body form, across worlds

Life arose separately on each world, so a family tree across worlds would be
invented kinship. The Kingdom groups species by **form**: what the body is
like, which is what a player sees and what decides how it is kept.
- Six classes from Tenebris: **mammals** (fur, milk), **birds** (feathers,
  eggs), **reptiles and amphibians**, **fish**, **arthropods** (insects,
  crabs, krill) and **molluscs and soft-bodied** (clams, octopus, jellies,
  stars, urchins, worms).
- A species from another world takes the class its body fits (Sequoia's crane
  is bird-form; Fallow's drover is mammal-form), and its card says it is
  unrelated to Tenebris's.
- Three **alien branches** fit none of them: Sequoia's **twostriders**
  (two-legged, tailed walkers: sparkit, shellkin, barkstrider; Fallow's
  burrowkit has the same form, evolved apart), Sequoia's **sporekin**
  (fungal animals: gloomhopper, sporeling), and Wick's **ringlings**
  (black-skinned twilight-ring life that never sees a sunrise: tallowback,
  dimwing).
- Each species is one record: id, name, world, biomes, class, size (m),
  diet (grazer, browser, omnivore, predator, filter, forager), temperament
  (docile, skittish, territorial, predator), **status** (wild only, tameable,
  domestic, protected), and what it gives.
- The Kingdom on the page has 98 entries: the game's 8 fish, M7's 24, M25's
  39 (its 41 sprites, with the reed dabbler's drake, hen and duckling as one
  species), M9's 5, M1's 14, 4 new wild species here (fellrunner, clover bee,
  hedge cat, field mouse), and 4 domestic breeds (yard hen, rootling, yard
  dabbler, herd hound), each the same species as its wild kin (a yard hen is a
  meadow hen) and listed so the domestic roster can be browsed.
- *Alternative:* group by world. Rejected as the main view (it hides that a
  drover and a woolback are kept the same way), kept as a filter.
- This is put to the owner with the Kingdom tab's screenshot (Q1).

### 2. What can be kept

- **Domestic**: born on a farm or tamed; lives in a pen; gives products.
- **Tameable**: can become domestic by one of the five ways below.
- **Wild only**, refused with a reason: predators (red fox, dust hound,
  stripe cat, adult pale wolves, mire lurker, crag eagle, Sequoia's raptor,
  Fallow's scythewing), migrants (shagback, lark), the too-skittish (hares),
  swarms of light (firefly, glowmoth: netted for a glow jar, M7), and every
  fish (kept in M9's tanks, not pens).
- **Protected** (M25): reef turtle, sea otter, seal, river otter, wild reed
  dabblers. Catching one is refused; a dabbler's egg from a wild nest may be
  taken and hatched, which is how ducks are kept.
- Rosters say what spawns wild on a body, not what may be kept there: a
  shellkin carried to Tenebris in a hold stays Sequoia's species.

### 3. Five ways to tame

| Way | Species | Rule |
| --- | --- | --- |
| Feed | woolback, plodder, ridge goat, meadow hen, sand strider, fellrunner, drover, tallowback, shellkin, sparkit | Crouched with its favourite food in hand, the animal lets you to arm's reach (1.5 m). Each feed fills trust by its species' gain times its temperament factor; standing or running inside its fear radius (9 m, 4.5 m crouched, M7) spooks it and costs 10 trust. Full trust: it follows you for 60 s, or while a lead is on it; through a pen's gate it is yours. Trust left unfinished decays 5 an in-game hour. |
| Break in | fellrunner | After full trust, three gentling sessions on a lead (20 s each, walking it in a pen) before it takes a saddle. |
| Raise young | rootling (boarlets from a sounder), herd hound (pale wolf pups from a den), burrowkit (kits) | Young follow the first person who feeds them. A den is visited while its adults are out (pale wolves hunt at dusk). Adults of these species stay wild. |
| Hatch | yard hen, yard dabbler | An egg from a wild nest, hatched under a broody hen or in an incubator, hatches tame. A wild dabbler is protected; its egg is not. |
| Swarm | clover bee | In spring a swarm hangs from a branch. At dusk, with a smoker and an empty hive within 3 cells, the colony moves in. One hive is one record (the queen's genome). |
| Moves in | hedge cat | Within 3 days of a barn holding feed (which draws field mice) and a dish of milk, a hedge cat comes. It is never penned; it stays while the dish is filled. |

Trust gain per feed (full is 100): meadow hen 34, woolback 25, plodder 20,
sand strider 20, drover 20, tallowback 25, shellkin 25, sparkit 34, ridge goat
15, fellrunner 12. Temperament factor 1.3 (calm) to 0.6 (wild), from the
genome (decision 7).

### 4. The domestic roster

Tenebris (names in the game's voice; M7's names kept where M7 has the animal):

| Animal | From | Favourite food | Space | Gives (base) | Work |
| --- | --- | --- | --- | --- | --- |
| Yard hen | meadow hen (M7) | grain | 5 to a cell; coop at night | 1 egg a day; a feather each 10-day moult; drumstick | eats pests in crop cells |
| Cockerel | meadow hen (M7) | grain | as the hen | makes the coop's eggs fertile; crows at dawn | |
| Plodder | M7 | turnip | 2 cells; barn | 1 pail of milk a day for 40 days after a calf; 2 manure a day; steak, hide, horn | ploughs (tills a cell in 10 s), pulls a cart |
| Woolback | M7 | grain | 1 cell; shelter in winter | 3 wool every 4 days by shearing; 1 manure a day; chop, hide | |
| Ridge goat | M7 | frost kale | 1 cell | 1 jug of milk a day for 30 days after a kid; horn; 1 manure; chop, hide | clears scrub |
| Rootling | tusk boar (M7) | turnip, scraps | 1 cell; sty with mud | 2 manure a day; litters of 4 to 8; ham, hide | roots a pen cell to tilled soil |
| Fellrunner | new, wild herds on fields and mountain meadows | grain | 2 cells; stable | 2 manure a day | ridden at 9 m/s; plough; cart |
| Sand strider | M7 | dates | 2 cells | milk; hide | pack (8 slots, M7), ridden at 6 m/s |
| Yard dabbler | reed dabbler (M25) | grain | 4 to a pond cell | 1 duck egg every 2 days; down every 8 days | eats slugs in crop cells |
| Clover bee | new, swarms in hollow oaks on fields | flowers within 3 cells | a hive | 1 honey every 3 days in spring and summer; 1 wax every 6 days | crops within 3 cells of a hive yield 1 more |
| Herd hound | pale wolf pups (M7) | meat, scraps | kennel | | herds a flock through a gate on a whistle; guards pens at night |
| Hedge cat | new, fields and hedges | milk | roams | | catches field mice: stored feed loss 10% to 0 |

Other worlds:

| Animal | World | Gives | Work |
| --- | --- | --- | --- |
| Shellkin | Sequoia (built in Tenebris) | | pack animal, 6 slots, walks at your side |
| Sparkit | Sequoia (built in Tenebris) | | companion |
| Tallowback | Wick | tallow every 4 days (candles, lamp fuel), milk | |
| Drover | Fallow | drover fleece every 5 days (M1's Fallow export), milk | |
| Burrowkit | Fallow | | digs: loosens a cell, turns up what is buried |

New wild species appended to Tenebris's roster: fellrunner (fields,
mountains), clover bee (fields), hedge cat (fields), field mouse (fields;
a pest of stored feed).

### 5. Pens and buildings

- **A pen is a closed loop of fence edges with at least one gate.** Fences
  and gates are pieces on hex edges (`player-building`), so a pen is derived
  from placed pieces, never stored. Its id is the lowest cell key inside it.
  An animal names its pen; if the loop is broken the animals in it are loose
  in the yard (they wander, but stay records).
- **Buildings** are placed pieces with a footprint: coop (1 cell, 6 nest
  boxes, a door that shuts at dusk), barn (3 cells, stalls and a hay loft),
  stable (2 cells, 2 stalls, tack), sty (1 cell, a lean-to and a mud cell),
  hive (on an edge, 3 to a cell), feed trough, water trough, kennel, duck
  house, compost bin.
- **Water**: a pen cell within one cell of water, or a water trough filled by
  bucket (one bucket a day serves four animals); M8's irrigation rule, applied
  to animals.
- **Grazing**: a grass cell in a pen feeds one woolback or goat a day, or half
  a plodder or fellrunner, and regrows in 4 days; in winter it does not, and
  grazers eat hay (a grass cell mown with the scythe gives 1 hay).
- **Night predators**: a red fox comes to a farm after dark (M7: "keep the pen
  shut"). A shut coop is safe; a lit gate or a herd hound turns it away. What
  an open run costs is Q2 for the owner, with a screenshot.

### 6. Needs, mood and health

- Four needs, each met or not per day: **food** (grazing or feed in a
  trough), **water**, **shelter** (under a roof at night in winter or rain;
  hens every night), **space** (the pen's cells against the animals' space).
- **Mood** is 25 for each need met, plus 10 for a brushing today, minus 15 for
  a herd animal kept alone. **Health** falls 10 a day while two or more needs
  are unmet and recovers 10 a day otherwise; below 30 the animal is sick and
  gives nothing.
- **Neglect never kills.** After 5 days without food an animal strays: it
  leaves by the next open gate and waits nearby, still a record, and food
  brings it back. Recommendation taken (ask only with screenshots), as M8's
  crops never die from absence.
- **Age:** no animal dies of age in the first version. Past its prime (hen 120
  days, others 300) its yield halves. Recommendation taken (ask only with
  screenshots).

### 7. Products and the economy

- Each product has a base amount and a period in world days (decision 4);
  the amount is scaled by the animal's yield, `0.6 + 0.8 × yield`
  (0.6 to 1.4), and given only on a day its mood is 50 or more and it is not
  sick.
- What is waiting is derived: the record keeps the world day each product was
  last taken, so a farm costs nothing while the player is elsewhere and has
  simply produced when they return. Eggs are the exception: an egg is laid
  into a nest as an item (it can be hatched or broken), so it is a journal
  entry.
- **Manure** goes to a compost bin: 4 manure and the bin's 3 days make 1
  compost, M8's fertiliser (half again as fast, one more at harvest).
- **Slaughter** is a butcher's block in a barn. The player chooses an animal
  and the screen fades: no animation of it. The cuts (steak from plodders,
  chop from woolbacks and goats, ham from rootlings, drumstick from hens),
  hide and horn go to the inventory. Yard dabblers are not slaughtered (M25's
  protection follows the line). The animal's record is kept, ended, so
  pedigrees never break. Dishes are the cooking mockup's.
- **Economy tab**: per animal, inputs and outputs a day; a herd composer sums
  them by season and says how many wheat, turnip and kale cells (M8's yields)
  feed the herd.

### 8. Eggs, hatching and life stages

- A hen lays in a nest box in the coop, or on the floor without one. With a
  cockerel in the same coop her egg is **fertile** (an item `fertile egg`
  carrying its seed and both parents' ids, as M9 keeps a fish's seed in its
  slot); without, it is an egg to eat.
- **Hatching**: under a **broody hen** (a hen with the broody trait sits on
  up to 8 eggs and stops laying) or in an **incubator** (6 eggs, needs a lit
  lamp, lamp oil or tallow). Hen eggs take 3 days, duck eggs 4. An egg taken
  out for more than a day goes cold and is an egg to eat.
- **Life stages**, derived from the birth day, never stored:

| Animal | Stage (days, size of adult) |
| --- | --- |
| Yard hen | chick 0-5 (0.12 m), pullet 5-10 (0.3 m), hen 10+ (0.42 m), lays from 10 |
| Yard dabbler | duckling 0-6, juvenile 6-12, adult 12+ |
| Woolback | gestation 5, 1-2 lambs; lamb 0-6 (45%), hogg 6-12 (75%), adult |
| Plodder | gestation 8, 1 calf; calf 0-8 (40%), young 8-16 (75%), adult |
| Ridge goat | gestation 5, 1-2 kids; kid 0-5 (45%), young 5-10, adult |
| Rootling | gestation 4, 4-8 piglets; piglet 0-4 (30%), weaner 4-10 (60%), adult |
| Fellrunner | gestation 10, 1 foal; foal 0-10 (50%), yearling 10-20 (80%), ridden from 20 |
| Herd hound | gestation 4, 3-6 pups; pup 0-5 (35%), young 5-10, adult |

A day is 48 minutes of play (M8), so a chick is a laying hen in eight hours.

### 9. Breeding and inheritance

- **A pair** is a male and a female of one species, both adult, in one pen,
  mood 60 or more, health 50 or more, the female not carrying and past her
  cooldown (a gestation's length after a birth). Mating needs no action: a
  pair that qualifies at the steward's morning check conceives.
- **The genome** is seven genes, two alleles each (14 bytes):
  - four quantitative genes, each allele 0 to 1, expressed as the mean of
    the two: **size** (`0.85 + 0.40 × g` of the species' length, M9's 85 to
    125%), **yield** (`0.6 + 0.8 × g`), **temperament** (trust factor
    `1.3 - 0.7 × g`), **hardiness** (health loss `× (1.5 - g)`, cold and heat
    tolerance);
  - two discrete genes from the species' lists: **coat** and **pattern**,
    the earlier in the list dominant;
  - one **morph** gene, recessive: two copies show the species' rare morph
    (albino, golden, melanic). One wild allele in eight is the morph, so one
    wild animal in 64 shows it (M9's rate) and about one in four carries it.
- **Inheritance**: the young take one allele of each gene from each parent,
  chosen by `hash(sire id, dam id, birth count, gene)`. Each allele mutates
  with probability 1/32: a quantitative one moves by up to ±0.15, a discrete
  one becomes another of the list. Deterministic, so a replay or a peer agrees.
- **The breeding screen shows the truth.** For a chosen pair it draws every
  possible offspring value of each quantitative gene (four combinations,
  equally likely), the range they span, the coat and pattern odds, the morph
  odds (both carriers: 1 in 4), and the inbreeding cost.
- **Inbreeding**: Wright's coefficient F from the pedigree (four
  generations). The young's hardiness is lowered by `0.5 × F` (siblings,
  F = 0.25: minus 0.125) and a litter's size by `× (1 - F)`. The screen warns
  above F = 0.0625 (first cousins).
- **Selective breeding** is the loop: pair the best for a goal (more wool, a
  calmer temper), keep their best young, rotate in unrelated stock to hold F
  down. The page charts ten generations both ways.
- *Alternative:* M7's "the young's seed mixes its parents'". Rejected: a
  mixed seed gives a look between the parents but no rule a player can learn
  or breed toward.

### 10. What is saved, and who wrote it

- **Wild animals are derived** from the world seed, the herd's seed and the
  clock, and are never saved (M7, M9), so retuning wild behaviour never
  touches a save.
- **A tamed or farm-born animal is a stored record** (`world-persistence`
  decision 1: it must survive a change to the rules that made it). Its id is
  `a<n>`, from a counter record, never reused. The body:

  ```text
  rec animal:a1042 {v:1, sp:home/27, sex:f, born:1834.25, gen:[...14 bytes],
    sire:a1007, dam:a0991, name:"Nettle", pen:p88213, owner:p0,
    state:alive, via:born, took:{wool:1830.0}, needs:[day summaries x5]}
  ```

  `sp` is the body and the species' index, as a fish is saved; the roster is
  append-only, so the index never moves.
- **Taming writes the record at once** through the journal, `@p`, with the
  genome the wild animal had (rolled from its seed). The write is acknowledged
  only after the store commits (CLAUDE.md, "Every accepted world mutation
  enters the durable transaction path immediately").
- **The farm's own changes are the `husbandry` world process**, run by the
  steward each in-game hour: it uses feed from troughs, records the day's
  needs, lays eggs into nests, starts and ends pregnancies, and writes each
  birth as a new record, authored `@whusbandry:<pen id>`. Like every world
  process it yields: a proposal that would change a field the player wrote
  (name, pen, state) or a piece the player placed is refused, and an egg whose
  nest the player removed is not laid.
- **Records are kept when an animal ends** (butchered, taken by a fox if Q2
  says so) with a state, so pedigrees never break. A record is about 80 bytes;
  a farm of 200 animals over a long game is 16 KB.
- **Saved games survive**: the record has its own schema version in
  `identity.ron`; a later gene is added with a default for old records on
  open, and the old version stays readable. Nothing about husbandry touches
  terrain, so no generator version changes.
- **Towns' livestock** is the town's: records authored by the town's process
  (`@wtown:<settlement>`). Taking one is refused, as M25 says; buying one from
  the farmer (M5) moves its owner to the player in one `@p` line.

### 11. In the engine

- `pbd_core::husbandry`: species husbandry data, the genome and inheritance,
  needs and mood, the product schedule, the `husbandry` process's `evaluate`.
  Pure and deterministic, no Bevy; tested against the stepwise answer for
  catch-up, as `world-persistence` requires of every process.
- `assets/config/husbandry.ron` holds every number in decisions 3 to 9, with
  units, validated at load like `fauna.ron`.
- `pbd_app`: animals within 150 m of the player are ECS entities (M7's cap of
  40 wild, plus up to 60 penned), stepped at a fixed rate; farther pens are
  numbers. Bodies are box-built with pixel hides, as M7 and the mockup build
  them, until the owner picks the art route (M7's question).
- Multiplayer: the same core rules for every peer; the record names its owner.

## 12. The first playable slice (built 2026-11-14)

The owner, 2026-11-14: "start building the farming systems into the game, as
well as the basic smelting stuff", then "oh and cooking and domestication of
animals too". That is the request for tasks 4 to 6 (proposal, "Go-ahead").
Farming and smelting are built beside this on `claude/farming-smelting`; this
slice is `claude/animal-husbandry`. What it builds, and where it departs from
decisions 1 to 11 because something they lean on is not in the game yet:

- **S1. Three animals first: yard hen, woolback, plodder** (decision 4's
  Tenebris roster, M26's farmstead). Each is tamed by feeding (decision 3).
  The others in decision 4 wait for their ways (hatching, raising young,
  swarms) and for the buildings they need.
- **S2. Land species are their own roster beside the fish.** A caught fish is
  saved as `Item::Fish(index)` into `fauna.ron`'s roster, and every fish rule
  reads that list, so land species go in `assets/config/husbandry.ron` under
  `land: { "home": [...] }`, append-only, with ids unique across both lists.
  A record names its species as `sp: <land index>` on its body (`body` field).
  Every fish gains `class: Fish` and `status: WildOnly` (decision 2: fish are
  kept in tanks, not pens). Only `home` has life; no other body is in the game.
- **S3. Feed is farming's crops, and turf for the grazers.** After
  `farming-and-smelting` landed (merged from `claude/phase-1`), a species eats
  a list of foods, its favourite first: the yard hen wheat seed or wheat, the
  woolback wheat or turf, the plodder turnip or turf (decision 3's grain and
  turnips; turf is a grass block, which a grazer eats in its pen anyway). Frost
  kale waits for the ridge goat.
- **S4. No crouch exists; standing still is the crouch.** The fear radius is
  9 m while the player moves and 4.5 m while they stand still (decision 3's
  numbers). Recommendation taken (ask only with screenshots).
- **S5. Fences are edge pieces stored as records.** `player-building` is not
  built, so a fence or a gate is a `fence` record whose id is its edge (the
  two cells' keys, low key in the high word), authored `@p`, with a `gate`
  flag and a `state` (`placed`, `removed`: records are never deleted). They are
  drawn as posts and rails by the app; they do not block the walker yet. A pen
  is derived from them (decision 5): a flood over the cells from the one the
  player stands in, crossing only edges with no piece, of at most 64 cells; it
  is a pen if it never escapes and its border holds a gate. Its id is its
  lowest cell key.
- **S6. Fences, gates and products are goods.** Rows of the one goods
  table (`pbd_core::goods`, `farming-and-smelting` decision 1) under an
  `animal-husbandry` comment, saved by code: `fence`, `gate`, `egg`, `wool`,
  `milk` (`gfence,3` in a slot field). A fence is NOT a block: it stands on a
  hex edge as a `fence` record, so it takes no material code. Every action
  that changes a record and the hands together is one `with ... rec @p` line
  (`store_with_hands`): taming (the last feed and the new animal), a hand
  feed, a collection, a piece set or taken up. Kit grant version 4 deals 24
  fences and 2 gates to every world once (version 3 is the forge's); an old
  world gains these and nothing else: no animals.
- **S7. The farm is derived, not stepped: no world process yet.** The steward
  (`world-persistence` group 4) is not built. Everything the farm does on its
  own in this slice is a closed form of world time from the record's last
  player-written state (`as_of`, `health`, `hungry_s`, `made`), evaluated
  when the player looks: health, mood, straying and what is waiting. Nothing
  is written until the player acts (feeds, leads, collects), and each such
  write re-bases the record at that moment. So no record is written by the
  world, the yield set is never in question, and a farm costs nothing while
  the player is elsewhere. Eggs are products like milk and wool here (decision
  7's exception, an egg laid into a nest as a journal entry, comes with
  fertile eggs in task 7, which needs the steward).
- **S8. Needs in this slice.** Food: the pen's grass cells graze one woolback
  each (a plodder two cells, a hen a fifth), or a hand feed of turf, which
  lasts two world days. Water: a pen cell next to water. Shelter: never met
  (there are no buildings yet), so a content animal is food plus space (50),
  with water 75. Space: the pen's cells against the herd's space. A loose
  animal has space and no grazing. Health recovers while the animal is content and falls
  while it is not (decision 6 says "while two or more needs are unmet", which
  with shelter unmeetable would sicken every animal; it returns with
  buildings). Sickness and straying as decision 6; neglect never kills.
- **S9. Products accrue continuously.** Each content, healthy day adds
  `amount x (0.6 + 0.8 x yield) / period` to the product's tally; whole units
  are waiting, at most four days' worth (or one period, if longer). Collecting takes the whole units
  and keeps the fraction. Milk is a plodder cow's, daily, without a calf in
  this slice (calves are task 8).
- **S10. Wild herds are derived from the world seed.** The surface is cut
  into 160 m squares by latitude and longitude; a square holds a herd if its
  hash says so and its centre is dry grass-topped land. The herd's species,
  size and each member's seed come from the same hash. A tamed animal's record
  keeps the wild seed it came from, and the herd leaves that member out from
  then on, so it is never in two places.
- **S11. Animals move like animals** (the owner on M7, 2026-11-14: "sad,
  these chickens arent animated at all"). Each animal is one mesh rebuilt per
  frame from boxes posed by a small rig: legs swing on a phase advanced by
  ground distance over stride, so feet never slide; a walking bird bobs its
  head with its steps; a standing bird pecks (quick dips at random 0.4 to
  2 s), glances about and flicks its tail; it potters a short way every few
  seconds; a quadruped grazes head down and shifts its weight; all lie or
  roost at night (sun below the horizon). Drawn lit by the field, and faded in
  and out through the terrain's screen-door mask (`Faded`) over 1 s, never
  popped (priority 1).
- **S12. Controls.** `E` (the door key, now "use") on an animal: with turf in
  hand, feed it (a wild one gains trust; a tame one is fed); on a tame one
  with something waiting, collect it; otherwise lead it or let it stay.
  Fences and gates are placed with right click like a block, on the edge of
  the aimed cell nearest the aim, and taken back with `E`. The animals' `E`
  runs before farming's and consumes a press it used, so one press never also
  harvests the crop behind an animal. A tamed animal follows for 60 s or until let go; let go inside a pen,
  that pen becomes its home (`@p`).

- **S13. Messages for the starter missions.** `pbd_app::husbandry::Deed`:
  `Tamed { species }`, `Penned { species }` and `PenBuilt { pen, cells }`, and
  every collection a `hotbar::Gained { how: Collected }`, each on the frame
  its durable write was accepted.

Recommendations taken (ask only with screenshots) for M26's questions, as no
survey answer exists for them: **Q1** the Kingdom by body form, world as a
filter (S2 keeps `class` on every species); **Q2** a fox may take a hen from
an open unlit run, never from a shut coop or past a hound (no fox in this
slice); **Q3** the breeding screen draws every possible offspring as ticks
with a better/same/worse arrow beside each (breeding is task 8).

Frame cost: not measured. `perf_suite` waits for the owner's hardware.

## 13. Eggs, young and breeding (tasks 7 and 8, planned 2026-11-15)

The owner, 2026-11-15: "finish up fishing wiki, herding, and flora". This
section is the plan for groups 7 and 8, written before their code. It keeps
S7's premise: the farm is a closed form of world time, nothing steps a
clock, and a farm costs nothing while the player is away. What is new is
that some of what the farm does on its own now has to be WRITTEN, because a
birth makes an animal that must outlive every rule change.

- **E1. Life stages in data, derived from the birth day** (decision 8).
  Each kept species gains a `young` block in `husbandry.ron`: its stages
  (name, until what age in days, size against the adult), how it is born
  (`Eggs { incubation_days, clutch_max }` for the hen, `Live
  { gestation_days, litter: (min, max) }` for the woolback and plodder) and
  its young's names. `husbandry::young::stage(species, born, day)` gives the
  stage and the size; the app draws a kept animal scaled by it. Nothing about
  a stage is stored. A female gives products only from `born + adult_days`
  (a hen "lays from 10"): `FarmState::advance` takes the day it may start.
  Numbers are decision 8's: hen chick 0-5 (0.29 of the adult), pullet 5-10
  (0.71); woolback gestation 5, 1-2 lambs, lamb 0-6 (0.45), hogg 6-12 (0.75);
  plodder gestation 8, one calf, calf 0-8 (0.40), young 8-16 (0.75). The
  adult ages stay `adult_days` (10, 12, 16).
- **E2. A brood is a stored record** (kind `brood`, schema 1, id from a
  counter like an animal's): a clutch of eggs or a pregnancy, with its
  species, dam, sire, how many, the day it started and where it is
  (`Hen(id)`, `Incubator(cell)` or `Womb`). It is never deleted.
- **E3. Fertile eggs.** A hen in a pen with an adult cockerel lays fertile
  eggs. Collecting them gives the good `fertile-egg` (appended to the goods
  table, with its icon) and, in the same `with ... rec @p` line, a `clutch`
  of state `Carried` naming the hen and the cockerel and the day they were
  taken: the parentage an egg item cannot carry rides in the record (M9 keeps
  a fish's seed in its slot; a good is a count, so the record is the slot).
  A carried egg goes cold a day after it was laid (decision 8): setting
  takes parentage from the oldest warm carried clutch first, and eggs with
  no warm parentage left are refused ("these eggs have gone cold"). A fertile
  egg cooks like an egg.
- **E4. Setting eggs.** `E` with fertile eggs in hand:
  - on a hen of yours that is adult and not sitting: she goes **broody** and
    sits on up to 8 of them (decision 8). A sitting hen lays nothing; her
    product tally is re-based when she is set and when the clutch hatches.
  - on an **incubator**: up to 6 eggs. The incubator is a husbandry piece
    like a fence (S5): the good `incubator` (made at the workbench from 4
    planks and 2 rope), set with right click on a cell as an `incubator`
    record whose id is the cell's key, drawn as a small crate with a glass
    lid; `E` with empty hands takes it up while it holds no eggs. It needs a
    **lit lamp** within 3 m (a lamp block of any kind, `Material::is_lamp`);
    without one it refuses eggs ("it needs a lamp beside it"), and a clutch
    whose lamp is gone when the farm is next evaluated goes cold.
  Either way the set brood is a player line, `rec @p brood`, with the hands.
- **E5. Hatching and births are the world's lines, never the player's.** When
  the farm is evaluated (the 0.5 s sync while the player is within 150 m,
  and on opening the world) every brood due at or before now is realised in
  due-day order: an egg clutch on `set + incubation_days`, a pregnancy on
  `conceived + gestation_days`. Each young is a new `animal` record with
  `via: Born`, `born` the due day (so its stage is right however late it is
  realised), `sire`, `dam`, `brood` (the brood's id), its genome inherited
  (E7) and its sex by hash, homed in its mother's pen. They are proposed
  under `@whusbandry:brood/<id>` through `WorldSave::propose`, so the yield
  set guards them; they only ever add records. A brood is *done* when some
  animal names it, so the world never edits the player's brood line. The
  premise is S7's: conditions are taken as they are now for the whole span.
- **E6. Pairing** (decision 9). A dam of a live-bearing species conceives at
  the first **morning** (world day + 0.25) at or after the day she and a sire
  both qualify: same species, both alive and adult, both in the same pen,
  her mood at least 60 and both healths at least 50 as the farm stands now,
  she is not carrying, and her **cooldown** (one gestation after her last
  birth) is over. The sire is the one the player chose for her on the
  breeding screen (`mate`, a field she carries, player-written) if he is in
  her pen, else the lowest-id adult male there. Conception is a `brood` of
  place `Womb`, written by the world (`@whusbandry:pen/<pen>`) as in E5.
  The event loop repeats until nothing more falls due by now, so a hundred
  days away catch up in one evaluation, and the same events come out
  whether it runs once or every half second (a test). Hens pair the same
  way only to make their eggs fertile (E3).
- **E7. Inheritance** (decision 9): one allele of each gene from each
  parent, chosen by `hash(sire id, dam id, birth count, gene, side)`; each
  allele mutates with probability 1/32, a quantitative one by up to ±0.15
  (38 of 255), a discrete one to another of its list, the morph flips. The
  birth count is the dam's young by this sire so far, so every young of a
  pair differs and a replay agrees. `husbandry::breeding::offspring_values`
  gives the four equally likely values of a quantitative gene before
  mutation; a test holds births to exactly those four.
- **E8. Inbreeding** (decision 9): Wright's F of the young is the kinship
  of its parents, by the recursive method over the stored parents' ids
  (the mockup's `makeKin`), looking back four generations; an animal with no
  recorded parents is an unrelated founder. Full siblings give 0.25, half
  siblings 0.125, first cousins 0.0625. The young's expressed hardiness is
  lowered by `0.5 × F` and a litter's size is multiplied by `1 - F` (at
  least one).
- **E9. The breeding screen** (8.2), after M7's Breeding tab: a menu screen
  (`menu::Screen::Breeding`), opened with `B` on foot, holding the pointer as
  the field guide does. Left: the species tabs and the herd (kept, alive,
  adult or not), each a pressable row with sex, id, generation and coat; a
  press picks the sire (a male) or the dam (a female). Right: the pair, F
  with the warning above 0.0625, each quantitative gene's bar with the four
  ticks, the band, the parents' marks and the arrow (Q3's recommendation:
  ticks and the arrow beside them), the coat, pattern and morph odds, and a
  **PAIR** button that writes `mate` on the dam (`@p`). Below: the pedigree
  of the last animal pressed, three columns (it, its parents, its
  grandparents), "wild-caught founder" where a parent is unknown. Labels
  only on screen (CLAUDE.md, "Notes stay out of the game").
- **E10. Saves.** `brood` and `incubator` are new record kinds at schema 1,
  noted in `identity.ron` the first time one is stored. The `animal` body
  gains `brood` and `mate`, both `Option` with `None` defaults, so a body
  written before them reads unchanged (S's own rule) and the schema stays 1.
  A world with no broods opens with no births, no conceptions and nothing
  written (a test), and an animal record written before this change opens
  with the same fields it had.
- **E11. What this pass leaves.** No duck eggs (the yard dabbler is not in
  the roster). No egg item carries a seed: the record does. The steward
  (`world-persistence` group 4) is still not built, so births are realised
  only while the player is within 150 m or opens the world, which E5 makes
  invisible: a birth's day is its due day whenever it is written.

## 14. The pen in the game's own wood (owner, 2026-11-15)

The owner, on the eggs capture: "some of the fence shot is missing
textures".

**Finding.** Every pen piece the game draws is a box of flat vertex colour
on a plain white material: `husbandry::rig::Builder` writes positions,
normals and one colour per box, and no UVs. The posts only seem to have a
grain because the ground's light field and the shadow cascades fall across
them; a rail, and above all the gate's pale boards, show the flat fill
plainly. The approved mockup (M26, `docs/mockups/husbandry.html`, `pen()`)
dresses the pen in the game's settlement textures: posts in `timber`, rails
in `poles`, the gate in `boards`, a trough in `plank`. The first playable
slice (S11, decision 12) drew them flat, and this pass's incubator followed
it. The animals are the same: flat boxes, where M26 gives each box a tile of
its hide.

**Change** (drawing only; no record, rule or save changes):

- `rig::Builder` gains UVs: a textured box lays its texture along each face
  at the texture's `repeat_m` from `textures/settlement/manifest.ron` (the
  towns' own scale), so a 2.8 m rail shows its grain at the same size a
  town's beam does.
- The fences are drawn as three meshes, one per texture: posts `timber`,
  rails `poles`, gates `boards`, each on a `StandardMaterial` with that
  texture sampled nearest and repeated (the towns' `texture()` settings),
  lit by the field as the boxes were. The incubator's body is `plank`, its
  legs `timber`, and its glass lid stays a flat pale blue (M26: "water,
  glass and the sky are flat colour").
- The textured fence pipeline is built behind the loading screen: a hidden
  textured piece joins the yard's prewarm template.
- **Not in this fix**: the animals' hides. Each species' hide strip has to
  be drawn first (M26's `tools/art/husbandry_art.py` holds them, outlined;
  the world's copy loses the outline, CLAUDE.md "No outlines in the world"),
  and that is its own step, listed as task 9.2.

## 15. The herd is lit as the terrain is (2026-11-23)

The carts' captures showed a loose horse in the farmyard far paler than the
same horse hitched to its cart: the herd, its fences and its pen took only
`LitByField`, so Bevy's own picture lit them and they glowed against the
ground. CLAUDE.md ("Everything in the world is lit as the terrain is") wants
`LitLikeTerrain` beside it on every creature and prop, so the herd's roots,
the fence root and their pipeline prewarm templates carry both. Nothing a
rule reads changes; only the look, back to what the rule already demands.

## Risks / Trade-offs

- **Scope.** This is a large system. The tasks put it in slices: records and
  taming first (an animal that follows you and is saved), then pens and
  products, then eggs, then breeding.
- **Frame cost.** Up to 100 animals near the player. Not measured; a
  `perf_suite` run on the owner's desktop before any of it lands.
- **Balance.** Product rates and growth times are first guesses, tied to M8's
  so a farm's crops and animals keep pace. They live in data.
- **Inbreeding as a mechanic** can feel punishing; the cost is small and
  shown before pairing, never hidden.

## Migration Plan

Nothing to migrate: no save holds an animal today. The record kind starts at
schema version 1; the roster additions are appends.

## Recommendations taken (ask only with screenshots)

Recorded so they are never mistaken for the owner's words:
- Neglect never kills; animals stray. No death by age in the first version.
- Wild-only and protected lists as in decision 2.
- Ducks are kept by hatching wild eggs and are not slaughtered.
- Slaughter off screen, at a butcher's block; cuts are products.
- Mating needs no action; a qualifying pair conceives at the morning check.
- Genome of seven genes with explicit alleles, stored in the record.

## Questions for the owner (with screenshots, on the page and in the survey)

- **Q1. The Kingdom by body form, with three alien branches,** or by world?
  Screenshot: `docs/screenshots/mockup-husbandry/kingdom.jpg`.
  Recommendation: by form, world as a filter.
- **Q2. Can a fox take a hen** from an open, unlit run at night (the hen's
  record ends "taken by a fox"), or does it only frighten them?
  Screenshots: `farm-night.jpg`, `farm-night-fox.jpg`. Recommendation: it can
  take one from an open run, never from a shut coop or past a hound; a loss
  the player can always prevent.
- **Q3. The breeding screen**: every possible offspring as ticks on a range
  (as drawn), or a simpler "better, same, worse" arrow per trait?
  Screenshot: `breeding.jpg`. Recommendation: the ticks, with the arrow as a
  summary beside each.
