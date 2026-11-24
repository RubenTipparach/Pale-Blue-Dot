## Why

The owner, 2026-11-23: "Design missions specifically for coop like building
a house together". Multiplayer (`multiplayer`, phase 3b) makes the world
shared, but every mission the game has is one player's: the starter chain
(`starter-missions`) and the town jobs (`townsfolk` design 8) count one
player's own deeds into one player's record. Two players in one world today
each walk their own chain side by side. Co-op missions are goals that are
met by a crew, that count each hand's share, and that some steps of need two
people at once.

This change is a design only. It builds on pieces that are not finished:
the server deciding player-building's pieces (`multiplayer` design 17, the
`piece` writer), presence (task 8.1) and drops (task 8.2). It waits for
them and for a request.

## What Changes

- **A crew mission**: progress kept once per world by the server, not per
  player, counted from the changes the server decides with their authors,
  by the same pure counting function the starter chain uses.
- **Each hand's share**: every objective of a crew mission records who did
  how much, and an objective may need at least two hands, or a share from
  each player present.
- **A step for two**: an objective met only when two players act on the
  same thing within a few seconds of each other (the ridge beam held at
  both ends), decided by the server as one transaction (`multiplayer`
  design 12).
- **The first chain, "Raise a house together"**: clear the site, lay the
  floor, raise the walls, set the ridge beam together, roof it and light the
  hearth; the house is then the crew's, co-owned in its records.
- **Four more crew missions drafted** (log drive, bucket chain, fish and
  fry, the town's order), each named with what it needs that is not built.
- **The mission panel** shows the crew's progress and each hand's share;
  its look is a mockup first (`game-ui-frontend`), before any game code.

## Capabilities

### New Capabilities

- `coop-missions`: crew missions, shares, steps for two, and the first
  chain.

### Modified Capabilities

- None yet: the starter chain and the town jobs keep counting as they do.
  When this is built, `starter-missions`' counting function gains the
  author it already receives.

## Impact

- `pbd_core::missions`: a crew record beside the player record; objective
  fields `hands` and `each`; the `Together` kind.
- `pbd-server`: counts crew progress on every change it accepts, and
  decides a step for two as a transaction.
- `assets/config/missions.ron`: a `crew` section for the chains.
- `pbd-headless` (`multiplayer` design 21): its bots play the house chain
  as the protocol test and as the bot of the owner's gameplay demo
  (`multiplayer` task 9.2).
