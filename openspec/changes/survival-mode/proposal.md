## Why

The owner, 2026-11-18: "3c will add survival mode, falling from high places,
dangeroous animals (we'll introduce predators), fighting with villagers,
hunger, and oxygen - underwater can t breath, or in space".

Phase 3c comes after multiplayer (3b) and before town building (phase 4).
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
- **Death**: the player's pack drops where they died as a marked bag that
  stays until it is picked up, and they wake at their bed or the world's
  spawn.
- **The HUD**: health, hunger and breath show only in Survival, and breath
  only while it is not full. A mockup first.

Not in this change: PvP (still off in multiplayer), armour, ranged weapons,
cold and heat (Warmth stays a buff waiting for its mechanic), disease, sleep
needs, difficulty levels beyond one tuning file, and craft crash damage.

## Capabilities

### New Capabilities
- `player/survival`: the world's mode, the player's health, damage from
  falls, hunger, breath and suit air, death and the dropped pack, and
  respawn.

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
- **Saves**: the world records its mode, and the player record gains health,
  hunger, breath and spawn, with full values for every older save. A dropped
  pack is a record. Additive only.
- **Animation**: the swing becomes an attack too (`held-swing`), and
  predators gain stalk, lunge and flee motions, mockup first by the
  `animation` skill.
- **Mockups first**: the survival HUD, a predator hunt by day and lit at
  night, and death with its dropped pack.
