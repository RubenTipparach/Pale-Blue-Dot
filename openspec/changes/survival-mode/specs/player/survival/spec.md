# Spec Delta

## Purpose

What it takes to stay alive in a Survival world: health, falls, hunger,
breath and air, tool wear, predators, fights with townsfolk, death and respawn, with
Explorer worlds left exactly as calm as before.

## ADDED Requirements

### Requirement: A world is Explorer or Survival
A world SHALL have a mode, Explorer or Survival, chosen when it is made and
stored with it. In Explorer, no rule in this capability SHALL apply. A world
saved before modes existed SHALL open as Explorer.

#### Scenario: An old save
- **WHEN** a world saved before this change is opened
- **THEN** it opens as Explorer, with no health, hunger or breath shown, and
  plays as before

#### Scenario: A new Survival world
- **WHEN** a player makes a world and picks Survival
- **THEN** the world opens with full health, hunger and breath, and keeps
  Survival every time it is opened

### Requirement: Health and death
In Survival, the player SHALL have health that damage lowers and that
refills slowly while hunger is not empty. At zero health the player SHALL
die, their pack SHALL drop at that place as a bag that stays until picked up,
and they SHALL wake at their bed, or at the world's spawn if they have none.

#### Scenario: Dying with a full pack
- **WHEN** the player dies carrying items
- **THEN** a bag holding all of them lies where they died, it is still there
  after the world is closed and reopened, and walking into it returns the
  items

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

### Requirement: Predators hunt in Survival
In Survival, predators SHALL hunt the player. Each SHALL give a visible and
audible warning before its first attack, SHALL flee when badly hurt, and
SHALL be bolder at night. In Explorer, predators SHALL keep their earlier
behaviour and never attack the player.

#### Scenario: A fox at night
- **WHEN** a Survival player meets a red fox at night
- **THEN** it stalks, warns, and attacks, and flees once it is badly hurt

#### Scenario: The same fox in Explorer
- **WHEN** an Explorer player meets a red fox
- **THEN** it never attacks them

### Requirement: Townsfolk fight back
In Survival, a struck townsperson SHALL fight back or flee, nearby guards
SHALL come to fight, and the town SHALL remember the fight in the player's
standing with it. In Explorer, townsfolk SHALL NOT be harmed.

#### Scenario: Striking a villager
- **WHEN** a Survival player strikes a villager in a town
- **THEN** the guards come for the player, and the town's standing with them
  drops

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

### Requirement: The server decides survival
In a multiplayer world, health, hunger, breath, damage, death and the
dropped pack SHALL be decided by the server like any other stored change.
Tool wear SHALL be counted by the server too. Players SHALL NOT damage each other.

#### Scenario: Two players and a fox
- **WHEN** a fox attacks one of two players standing together
- **THEN** both players' games show the same hit and the same health for the
  one attacked, and the other cannot hurt the first with their own swing
