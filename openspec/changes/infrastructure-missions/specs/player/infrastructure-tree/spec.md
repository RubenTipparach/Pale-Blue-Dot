# Player: infrastructure tree

## Purpose

A third mission tree, for phase 4a: stoneworks, roads, civil works, a town
grown on the grid and early power. Proposed; nothing here is built.

## ADDED Requirements

### Requirement: The infrastructure tree is its own tree

The infrastructure tree SHALL be read from its own file, counted into its
own record (`infrastructure_missions`) and shown on its own tab of the
Missions screen. Every mission SHALL have exactly one parent but the root, no
mission of another tree SHALL be its parent, and an extra need across trees
SHALL be a hidden `after` gate.

#### Scenario: Three trees, three records

- **WHEN** an infrastructure mission is done
- **THEN** the `infrastructure_missions` record changes and neither the
  starter tree's `missions` record nor the `town_missions` record does

### Requirement: The tree opens with the town's first job

The infrastructure tree SHALL open when the town tree's "A job from the
board" is done, and its tab SHALL not be shown before then.

#### Scenario: The first job handed in

- **WHEN** the player hands in the job that finishes "A job from the board"
- **THEN** the infrastructure tree is open at its root, "Build a grinder"

### Requirement: The roads belong to the infrastructure tree

The missions Build a grinder, Crush stone, Build a mixer, Mix gravel, Mark a
road, Pave it and Join a town SHALL be missions of the infrastructure tree
under their existing ids, and not of the town tree, whose Carts branch SHALL
hang off "A job from the board".

#### Scenario: The town tree without roads

- **WHEN** the town tree is shown
- **THEN** it has no road mission, and "Make wheels" opens once "A job from
  the board" is done

### Requirement: A save keeps every road mission it has done

On opening a world whose town record holds any moved road mission, the game
SHALL write the infrastructure record with each moved mission done, or open
with its counts, as the town record had it, SHALL leave the town record as it
was, and SHALL never pay a moved mission's coins a second time.

#### Scenario: Paved before the move

- **WHEN** a world whose town record has "Pave it" done and "Join a town" at
  0 of 1 is opened with this tree for the first time
- **THEN** the infrastructure tree has "Pave it" done and "Join a town"
  open, the purse is unchanged, and the town record is unchanged

### Requirement: Power is a branch of the infrastructure tree

Early power's missions SHALL be the infrastructure tree's Power branch, its
first mission hanging off "Build a grinder" with a hidden gate on the starter
tree's "The iron age".

#### Scenario: A grinder but no iron

- **WHEN** "Build a grinder" is done and the starter tree's "The iron age" is not
- **THEN** "Cut gears" is locked, and opens when "The iron age" is done
