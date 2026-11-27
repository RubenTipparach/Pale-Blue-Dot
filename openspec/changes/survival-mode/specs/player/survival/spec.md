# Spec Delta

## Purpose

What it takes to stay alive in a Survival world: health, falls, hunger,
stamina and its food cost, water, body temperature and heat stroke, breath
and air, hunting and weapons, tool wear, predators, fights with townsfolk,
death and respawn, with
Explorer worlds left exactly as calm as before.

## ADDED Requirements

### Requirement: Falling hurts by impact speed
In Survival, landing SHALL cause damage when the speed at impact exceeds a
safe speed, growing with the excess. The safe speed SHALL be computed in the
body's own gravity, so the same fall height hurts less on a lower-gravity
body. Landing in water deep enough to swim in SHALL cause no fall damage.

#### Scenario: A short drop
- **WHEN** the player steps off a ledge two layers high
- **THEN** they take no damage

#### Scenario: A cliff
- **WHEN** the player falls from twenty layers onto rock
- **THEN** they take damage that a fall from ten layers does not exceed

#### Scenario: Into a lake
- **WHEN** the player falls from twenty layers into deep water
- **THEN** they take no damage

### Requirement: Hunger empties with play and meals refill it
In Survival, hunger SHALL empty slowly with play time only, never while the
game is closed. A meal SHALL refill it as well as giving its buff. While
hunger is empty, health SHALL NOT refill and SHALL drain slowly.

#### Scenario: Closing the game hungry
- **WHEN** a player quits with hunger nearly empty and returns a day later
- **THEN** hunger is where they left it

#### Scenario: Eating
- **WHEN** a hungry player eats a cooked dish
- **THEN** hunger rises by the dish's amount and its buff starts

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

### Requirement: Tools wear out in Survival
In Survival, each owned tool SHALL carry the uses it has left, starting at
the number its tier sets. A finished action with the tool (a block broken, a
tree felled, a cell tilled, a fish landed, a hit landed) SHALL spend one use,
or two for a block of a higher tier than the tool.  At zero uses the tool SHALL become a broken
tool in the same slot that works as bare hands, and repairing it at a
workbench or with a town's smith SHALL restore all its uses. In Explorer,
tools SHALL NOT wear.

#### Scenario: A stone pickaxe wears out
- **WHEN** a Survival player breaks stone with a stone pickaxe until its
  uses reach zero
- **THEN** the pickaxe stays in its slot as a broken pickaxe, digs as bare
  hands do, and is still there after the world is reopened

#### Scenario: Repair at the workbench
- **WHEN** the player repairs a broken stone pickaxe at the workbench with
  half its head materials
- **THEN** it has its full uses again and keeps its tier

#### Scenario: A missed swing is free
- **WHEN** the player swings at nothing, or stops a dig half way
- **THEN** the tool spends no use

#### Scenario: An old save
- **WHEN** a world saved before tool wear is opened in Survival
- **THEN** every tool it holds has its full uses

#### Scenario: Explorer never wears
- **WHEN** an Explorer player breaks a thousand blocks with one pickaxe
- **THEN** the pickaxe has not changed

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

### Requirement: Body temperature follows the weather at the player
In Survival, the player's body temperature SHALL move toward a target set by
how the place feels: the atmosphere's own air temperature at the player
(which carries the biome, season, hour and altitude), the sun unless shaded,
humid heat, wind chill, rain, being wet or in water, a nearby fire,
shelter, and what the player wears. Too hot SHALL use water faster; too cold
SHALL slow stamina's refill and burn food faster; past either end, health
SHALL drain. The shade test SHALL be made on the CPU from the terrain, never
read back from the GPU.

#### Scenario: Shade at noon
- **WHEN** a Survival player stands in a desert at noon, then in the shade
  of a rock
- **THEN** the body heats in the sun and settles cooler in the shade

#### Scenario: A cloak and a fire
- **WHEN** a Survival player stands in a tundra night, then puts on a wool
  cloak, then stands beside a campfire
- **THEN** each step leaves the body warmer than the one before

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

### Requirement: Wild game is hunted for meat, hide and bone
In Survival, wild game SHALL take damage and flee, and a kill SHALL drop its
species' meat and bones, and its hide whole when the carcass is dressed with
a hunting axe. Raw meat SHALL cook at the campfire, hide SHALL become
leather at a tanning rack, and bone SHALL make arrows and fertiliser. A
killed herd member SHALL come back after days of play, and the kill SHALL be
kept in the save. Protected species, and animals a player keeps or bought,
SHALL NOT be harmed.

#### Scenario: A woolback hunted and dressed
- **WHEN** a Survival player kills a wild woolback and dresses it with a
  hunting axe
- **THEN** raw meat, a hide, wool and bones lie at the kill, and the herd has
  its member back after three world days

#### Scenario: A protected otter
- **WHEN** a Survival player strikes a river otter
- **THEN** it takes no damage

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
