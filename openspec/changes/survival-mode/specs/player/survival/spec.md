# Spec Delta

## Purpose

What it takes to stay alive in a Survival world: health, falls, hunger,
stamina and its food cost, water, body temperature and heat stroke, breath
and air, hunting and weapons, tool wear, predators, fights with townsfolk,
death and respawn, with
Explorer worlds left exactly as calm as before.

## ADDED Requirements

### Requirement: Suit air on an airless body
On foot on a body with no atmosphere, the suit's air reserve SHALL empty,
then health SHALL drain, and the reserve SHALL refill once the player is
back in a sealed craft cabin.

#### Scenario: Stepping out on an airless body
- **WHEN** the player leaves a sealed craft on a body with no atmosphere
- **THEN** the suit's air reserve starts to empty, and refills once they are
  back in the cabin

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

### Requirement: The server decides survival
In a multiplayer world, health, hunger, breath, damage, death and the
dropped pack SHALL be decided by the server like any other stored change.
Tool wear, stamina, water, body temperature, passing out, every weapon hit
and every kill SHALL be decided by the server too. Players SHALL NOT damage each other.

#### Scenario: Two players and a fox
- **WHEN** a fox attacks one of two players standing together
- **THEN** both players' games show the same hit and the same health for the
  one attacked, and the other cannot hurt the first with their own swing
