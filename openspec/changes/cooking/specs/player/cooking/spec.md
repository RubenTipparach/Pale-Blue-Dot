# Player: cooking

## Purpose

Kitchen stations, recipes, heat, freshness, quality, eating and its effects,
and how food is saved. Proposed with the hearth and kitchen mockup; nothing
here is built.

## ADDED Requirements

### Requirement: Kitchen stations are placed stations

Each kitchen station SHALL be built from materials and placed on a cell like
the crafting stations, through the durable edit path, and a cooking job SHALL
be a stored record on its station that advances only while the world is
played. A heated station SHALL take its fuel as an ingredient, one per cook.

#### Scenario: A job does not run while the game is closed

- **WHEN** a 14-second loaf is put in an oven and the game is closed after
  5 seconds of play and opened again
- **THEN** the loaf still has 9 seconds to go

#### Scenario: Fuel is taken with the ingredients

- **WHEN** a heated recipe is cooked with one log in the pack
- **THEN** the log leaves the pack with the ingredients, and the recipe
  cannot be cooked with no log

### Requirement: Eating gives an effect, and not eating costs nothing

Eating a dish SHALL give its effect for its minutes of play times its star
multiplier (1, 1.5, 2). At most three effects SHALL work at once; a fourth
SHALL replace the one with least time left; the same effect SHALL NOT stack,
the longer one SHALL count. No status SHALL drain while the player does not
eat.

#### Scenario: A fourth dish

- **WHEN** three different effects are working and a dish with a fourth
  effect is eaten
- **THEN** the effect with least time left ends and the new one starts

#### Scenario: Not eating

- **WHEN** the player plays for a game week without eating
- **THEN** no speed, tool or movement value differs from a new player's

### Requirement: Effects only touch built systems

Stride SHALL scale sprint and swim speed, steady hands the line's tension
limit and the fish's stamina drain, keen the dig and break time, and glow
SHALL add a light on the player. Warmth SHALL have no effect while the game
has no cold.

#### Scenario: Stride

- **WHEN** a stride dish is working and the player sprints on flat ground
- **THEN** the sprint speed is 15% above the configured sprint speed

### Requirement: Freshness counts played time only

A food stack SHALL store the world time it was made, and its age SHALL be
world time now minus that. It SHALL be fresh to a third of its keep, good to
its keep, stale to twice its keep, and then SHALL become compost, one for one.
Merging two stacks of the same food and stars SHALL keep the older time.

#### Scenario: Spoiled food becomes compost

- **WHEN** three raw fillets (keep one day) are carried for two days of play
- **THEN** the stack is three compost and no fillet

#### Scenario: A closed game ages nothing

- **WHEN** a world is closed with fresh fish and opened again
- **THEN** the fish is as fresh as when it was closed

### Requirement: Quality is worked out, never rolled

A dish's stars SHALL be a function of the freshness band of its oldest
ingredient, whether the chosen heat is the recipe's, and whether the recipe
has been cooked five times; the same inputs SHALL always give the same stars.

#### Scenario: Same inputs, same stars

- **WHEN** the same recipe is cooked twice from ingredients in the same bands
  at the same heat with the same practice
- **THEN** both dishes have the same stars

### Requirement: Food is saved without migrating old saves

Food SHALL be a new item kind indexing an append-only food roster, with its
made-at time and stars on the stack. A caught fish SHALL remain a fish item
and be usable as an ingredient. Known recipes SHALL be saved as recipe ids.

#### Scenario: An old save opens unchanged

- **WHEN** a world saved before food existed, with fish in the pack, is
  opened by a build with cooking
- **THEN** the same fish are in the same slots, and nothing was rewritten
