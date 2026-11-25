## Context

- **Missions today** (`pbd_core::missions`): a chain in
  `assets/config/missions.ron`; an objective is a `Kind`, what it counts
  and how many; progress is the player record `missions`. Counting is a
  pure function of the chain, the record and one `Happened`, written so
  "a future multiplayer counts the same way".
- **Multiplayer** (`multiplayer`): the server decides every change and
  names its author (`Author::Player(n)`); single player runs the same
  server in process. Pieces of a player's building (`player-building` B1 to
  B3) are records the server will decide when their writer moves behind it
  (design 17). Presence is task 8.1, drops 8.2, far transactions 8.5.
- **Player building** (`player-building`, built in phase 3): walls, doors
  and windows on cell edges, floors, stairs, posts, lanterns and roofs, each
  a `piece` record checked by `building::rules`.
- **The headless bots** (`multiplayer` design 21) already build a hut of
  blocks together over the network; the house chain is the same play with
  real pieces.

## Goals / Non-Goals

**Goals:**
- Goals that need, or reward, more than one player in a world.
- Every hand's share visible, so no one carries the crew unseen.
- One counting rule for solo and crew missions, run by the server.
- A first chain built on systems the game already has (building, digging,
  cooking, fishing), so it adds rules, not content systems.

**Non-Goals:**
- Competitive or PvP goals, scoring between players, rewards that one
  player can take from another.
- Matchmaking or public crews: a crew is whoever is in the world
  (`multiplayer` design 14: invited co-op, up to 8).
- Voice or text chat beyond `multiplayer` 8.5.

## Decisions

### 1. A crew is the players of one world, and its record is the world's

Crew progress is a world record (kind `crew_missions`), kept and counted
by the server, journaled through the same durable path as any change.
Whoever is in the world is in the crew; a player who joins halfway sees the
crew's progress as it stands and adds to it. The player record `missions`
stays each player's own (the starter chain is a tutorial, and each player
walks it).

*Instead of* crews formed by invitation: one world, one crew is all the
game needs at up to 8 friends, and it needs no screen. *Instead of* copying
crew progress into each player's record: two copies are two copies that
will disagree.

### 2. Counting keeps one function, and gains the author

`missions::count(chain, record, happened)` already takes one `Happened`.
The server builds a `Happened` from each change it accepts, with its
author, and calls the same function for the author's own record (the solo
chains) and for the world's crew record. The crew record's counts are
per author: `{ objective: { player: n } }`, and the objective's total is
the sum.

### 3. Two fields make an objective a crew's

An objective of a crew mission may carry:
- `hands: n`: met only when at least `n` different players have counted
  toward it (`hands: 2` for "raise the walls" means not one player alone);
- `each: n`: every player in the world when it is met has counted at least
  `n` (a share from everyone; a player who joined in the last minute is
  exempt until they have been in for `grace_s`, 300 s by default).

Both are checked when the total reaches its count; until they hold, the
objective shows what is missing ("Needs another pair of hands").

### 4. A step for two is a transaction

A new kind, `Together`, counts one thing done by two players at once:
both send the same request on the same target (the ridge beam's two ends:
a `piece` of class `beam` whose two end cells each need a player in reach)
within `together_s`, 3 s. The server holds the first request open for that
long; the second within it settles both as one change with two authors; a
first with no second is refused as `Alone`, and nothing changes. It is the
far transaction of `multiplayer` design 12 used near.

In single player the step for two is skipped and counts as met, as a
pending objective is (`starter-missions` design 8), so a crew chain can be
finished alone, more slowly. A townsperson as the second pair of hands is
a follow-up with `townsfolk`.

### 5. The first chain: "Raise a house together"

On a site the crew marks on the map (a map marker, `multiplayer` 8.5),
within 40 m of each other:

| # | Mission | Objectives | Crew rule |
| --- | --- | --- | --- |
| H1 | Clear the site | dig the site level: 7 cells, top layer flat | `hands: 2` |
| H2 | Gather timber | fell 6 trees, bring 24 logs | `each: 4` logs |
| H3 | Lay the floor | 7 floor pieces | `hands: 2` |
| H4 | Raise the walls | 6 walls, 1 door, 1 window | `hands: 2`, `each: 1` |
| H5 | Set the ridge beam | 1 beam, `Together` | two at once |
| H6 | Roof it | 1 roof over the floor | none |
| H7 | Light the hearth | 1 campfire or brazier inside | none |

When H7 is met the house's pieces gain the crew as co-owners: each piece
record names every player who placed a piece of it, so no one of them can
be locked out by another, and removing a piece of a co-owned house needs no
one's leave (invited co-op; `multiplayer` design 14). A bed in it is a
spawn for any of them, when beds exist.

Pieces are the ones `player-building` already has; the beam is the one new
class (a roof's ridge between two posts), checked by `building::rules`.

### 6. Four more drafted, each with what it waits for

| Mission | The co-op | Waits for |
| --- | --- | --- |
| Log drive | two players walk a log down a river, one steering at each end; alone it snags | `log-drives`' log on the server, presence (8.1) |
| Bucket chain | fill a cistern 40 buckets from a well 30 m off, passed hand to hand; a pass between two players is a `Together` | buckets through the server, drops (8.2) |
| Fish and fry | one catches 6 fish, another cooks them at the house's hearth, a dish each | fishing's catch as a server request (8.3) |
| The town's order | a town asks for 60 planks and 20 bricks by a day's end; any hands deliver | `townsfolk` jobs counted per crew |

### 7. The panel shows the crew and each share

The missions panel gains a CREW tab: each crew mission's objectives, the
total, and a short bar per player in their colour with their name. Text is
plain (CLAUDE.md "Game text is plain"): "Raise the walls 5/8", "Needs
another pair of hands". The look is a mockup in `docs/mockups/` first,
listed on the tracker, with the owner's approval before the game's panel
changes.

### 8. The bots play it

`pbd-headless` bots (`multiplayer` design 21) gain the house chain as a
script once pieces go through the server, and it is the protocol test of
the step for two. Two of them, with the stated goal "build a house
together and light its hearth", are the gameplay demo of `multiplayer`
task 9.2, played through requests as a player's hands send them.

## Risks / Trade-offs

- **A crew of one carries another.** `each` and the shares make it seen;
  the rewards are the crew's, so it is not a loss to anyone.
- **The step for two on a slow link.** 3 s is generous on a LAN; over the
  internet it is measured before it is tuned, and it is validated config.
- **Solo players meet a wall.** The step for two counts as met alone, and
  crew chains are listed apart from the starter chain, never in its way.
- **It waits on unbuilt work** (pieces, presence, drops through the server).
  The order is: the piece writer behind the server, then H1 to H4 and H6,
  H7 with the crew record, then `Together` and H5.

## Owner answers (survey, 2026-11-24)

- N1, the site marker: a map marker shared by the crew, set when the
  chain starts (the recommendation, chosen in the survey).
- N2, rewards: the house itself, shared by the crew (the recommendation,
  chosen in the survey).

Multiplayer waits until the rest of phase 3 is done (owner, 2026-11-24:
"leave multiplayer out for now").
