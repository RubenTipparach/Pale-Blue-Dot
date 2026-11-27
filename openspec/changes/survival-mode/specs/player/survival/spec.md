# Spec Delta

## Purpose

What it takes to stay alive in a Survival world: health, falls, hunger,
stamina and its food cost, water, body temperature and heat stroke, breath
and air, hunting and weapons, tool wear, predators, fights with townsfolk,
death and respawn, with
Explorer worlds left exactly as calm as before.

## ADDED Requirements

### Requirement: Breath under water and air in thin air or vacuum
In Survival, while the player's head is under water, breath SHALL empty,
then health SHALL drain until they surface. On foot where the air is too thin
or absent, a suit air reserve SHALL do the same. Inside a sealed craft cabin,
the player SHALL breathe the craft's air.

#### Scenario: A long dive
- **WHEN** the player stays under water past their breath
- **THEN** health drains until they surface, and breath refills once they do

#### Scenario: Stepping out on an airless body
- **WHEN** the player leaves a sealed craft on a body with no atmosphere
- **THEN** the suit's air reserve starts to empty, and refills once they are
  back in the cabin

### Requirement: Water is drunk, and heat uses it faster
In Survival, the player SHALL have a water bar that empties with play time,
faster in heat and with effort. Fresh water, a water bucket, milk, fruit and
meat stew SHALL refill it; sea water SHALL NOT. A straw SHALL drink juice
from a standing desert cactus, a few sips a cactus, which come back over
world days and are kept in the save. An empty bar SHALL drain health slowly.

#### Scenario: A cactus with a straw
- **WHEN** a Survival player holding a straw drinks from a desert cactus
  until it is dry, closes the world and reopens it the same day
- **THEN** each sip raised water, the cactus is still dry, and it gives a sip
  again a world day later

#### Scenario: Sea water
- **WHEN** a Survival player tries to drink from the sea
- **THEN** water does not rise

### Requirement: Heat stroke hurts and makes the player pass out
In Survival, a body temperature of 40 C or more SHALL take health and make
the player pass out where they stand for a fixed time, during which the
world goes on. Being hit, or being revived by another player with water,
SHALL wake them at once. Waking, they SHALL be dazed for a while. A heat
stroke in deep water SHALL leave the player floating face up. Water drunk
SHALL hold the body below heat stroke in the hottest weather the planet has.

#### Scenario: Out of water at noon in the desert
- **WHEN** a Survival player with an empty water bar stays in the desert sun
- **THEN** within minutes the body passes 40 C, health drops, and the player
  passes out for 30 s, then wakes dazed

#### Scenario: Watered at noon in the desert
- **WHEN** the same player keeps drinking
- **THEN** the body stays below 40 C and they never pass out

### Requirement: The survival HUD shows a stat only when it matters
In Survival, health, stamina, hunger and water SHALL be shown as bars
stacked at the left edge of the screen. A full bar SHALL be hidden, and
hunger and water SHALL be shown only at half or lower. Every stat SHALL be
shown, full or not, in the pack. While the body is in heat exhaustion or
heat stroke, a short warning SHALL be shown above the hotbar. In Explorer
none of it SHALL be shown.

#### Scenario: Fed, watered and rested
- **WHEN** a Survival player is at full health and stamina with hunger and
  water above half
- **THEN** no survival bar is on the screen, and opening the pack shows all
  four with their values

#### Scenario: Getting hungry
- **WHEN** hunger falls from 60 to 40
- **THEN** the hunger bar appears on the left stack as it passes half

#### Scenario: Heading for heat stroke
- **WHEN** a Survival player's body passes 39.5 C
- **THEN** the heat stroke warning appears above the hotbar, and goes once
  the body is back under 39.5 C

### Requirement: The server decides survival
In a multiplayer world, health, hunger, breath, damage, death and the
dropped pack SHALL be decided by the server like any other stored change.
Tool wear, stamina, water, body temperature, passing out, every weapon hit
and every kill SHALL be decided by the server too. Players SHALL NOT damage each other.

#### Scenario: Two players and a fox
- **WHEN** a fox attacks one of two players standing together
- **THEN** both players' games show the same hit and the same health for the
  one attacked, and the other cannot hurt the first with their own swing
