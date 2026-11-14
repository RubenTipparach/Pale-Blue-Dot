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
