# Proposal: the animal kingdom and husbandry (animal-husbandry)

## Why

The owner, 2026-11-14: **"we need a whole animal kingdom and domestication of
animals, eggs, breeding".**

Today the game has fish and nothing else that lives:
- **Eight fish** swim in schools on Tenebris (the home planet, `home` in
  `assets/config/fauna.ron`; `pbd_core::fauna`, `openspec/specs/world/fauna`).
  A caught fish is an inventory slot, `Item::Fish(u16)`, the species' index in
  an append-only roster. Nothing walks, flies, lays or breeds.
- **Three mockups draw animals, and none of them is built.** M7 "Herds,
  Hunters and Glowmoths" (`docs/mockups/animals.html`) gives Tenebris 24 land
  species by biome, with herding, hunting, drops, and a first pass at taming
  (feed a trust meter), produce on a timer and breeding by mixing seeds. M25
  "Seas, Rivers and Marshes" (`ocean.html`, `ocean-environments`) adds 41
  water creatures, ducks among them, whose eggs and feathers are drops and
  which cannot be caught. M9 "Fish Lab and Aquariums" (`fish.html`) rolls each
  fish from a species and a seed, with rare morphs.
- **Other worlds have rosters on paper.** M1 (`planets.html`,
  `star-systems`) lists Sequoia's sparkit, shellkin, crane, raptor and serpent
  (built in Tenebris, the owner's earlier game) and four planned species; Wick's
  tallowback and dimwing; Fallow's drover, scythewing and burrowkit.
- **Farming (M8, `farming.html`)** already has pens beside its fields, and
  compost "from the pen's muck".

There is no one list of what lives where, no rule for what can be tamed and
what stays wild, no eggs that hatch, no young that grow, and no inheritance.

## What Changes

One mockup, `docs/mockups/husbandry.html`, "Pens, Nests and Pedigrees", and
this write-up. No game code: that waits for the owner's approval and a request
(CLAUDE.md, "Write it up before touching code").

- **One animal kingdom across every world.** Every species from the game, M7,
  M25, M9 and M1 in one taxonomy grouped by body form (mammals, birds,
  reptiles and amphibians, fish, arthropods, molluscs and soft-bodied), with
  three alien branches that fit none of them: Sequoia's twostriders and
  sporekin, and Wick's ringlings. Each species names its world, biome, size,
  diet, temperament (docile, skittish, territorial, predator), status (wild
  only, tameable, domestic, protected) and what it gives. A Kingdom tab
  browses them as a tree and by filters.
- **Domestication.** A wild animal is tamed one of five ways: by feeding its
  favourite food crouched until a trust meter fills; by raising young taken
  from a den or a nest; by hatching an egg; by moving a swarm into a hive; or
  it moves in on its own (the hedge cat). Predators, protected species and
  migrants stay wild.
- **A domestic roster for Tenebris**, named in the game's voice: yard hen
  (from M7's meadow hen) and cockerel, plodder, woolback, ridge goat,
  rootling (from the tusk boar), fellrunner (a new riding and draught animal
  of the fields), sand strider (M7's desert mount), yard dabbler (hatched from
  M25's reed dabbler eggs), clover bee, herd hound (from pale wolf pups) and
  hedge cat. **Alien domesticates**: Sequoia's shellkin (pack animal) and
  sparkit (companion), Wick's tallowback (tallow and milk), Fallow's drover
  (fleece) and burrowkit (a digger).
- **Husbandry.** Pens are fenced cells on the hex grid with a gate; coops,
  barns, stables, sties, hives, troughs, ponds and kennels are placed pieces.
  Each animal has needs (food, water, shelter, space) that make its mood and
  health, and gives products on a schedule in world time: eggs, milk, wool,
  honey and wax, down, manure (compost for M8's fields). Working animals
  plough, carry and are ridden. Meat comes from a butcher's block, off screen;
  the cuts are products, and every dish made from them is the cooking mockup's.
- **Eggs and breeding.** Eggs are laid in nests, fertile only with a cockerel
  in the coop; a broody hen or an incubator hatches them; young grow through
  life stages. Mammals pair in a pen and carry their young. Each animal has a
  small genome (size, coat, pattern, yield, temperament, hardiness, a
  recessive rare morph) inherited one allele from each parent, with a rare
  mutation. A breeding screen shows the possible offspring before you pair two,
  a pedigree, and the inbreeding cost of a close pairing.
- **Saved and authored.** A tamed animal is a stored record with an id, its
  genome and its parents' ids, written on the durable journal path the moment
  it is tamed (`world-persistence`). What the player does is `@p`; what the
  farm does on its own (an egg laid, a lamb born) is a world process,
  `@whusbandry`, that never overwrites the player. Wild animals stay derived
  and are never saved.

## Capabilities

### New Capabilities

- `world/husbandry`: taming, pens and buildings, needs and products, eggs,
  breeding and inheritance, and the stored animal record.

### Modified Capabilities

- `world/fauna`: land, air and water species share one taxonomy and one
  append-only roster per body, and each species says whether it can be tamed.

## Impact

- **Data:** a new `assets/config/husbandry.ron` (per species: how it is tamed,
  favourite food, needs, products and their periods, gestation or incubation,
  life stages, genome ranges); `fauna.ron` gains the land species (appended)
  and a `class` and `status` per species.
- **Core:** `pbd_core::husbandry` (genome, inheritance, needs, the product
  schedule, the steward's process), pure and deterministic, with no Bevy.
- **App:** pens read from placed pieces (`player-building`); animals near the
  player as ECS entities, far ones as a number per pen.
- **Persistence:** animal records and the `husbandry` world process through
  the `world-persistence` journal and steward; a new record kind with its own
  schema version in `identity.ron`.
- **Other mockups:** extends M7, M25, M9, M1 and M8, and contradicts none of
  them; the dishes belong to the cooking mockup being drafted beside this one.
- **Performance:** not measured. A mockup in a cloud session with no GPU; the
  screenshots ran on SwiftShader and say nothing about frame time.
