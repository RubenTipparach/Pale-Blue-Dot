# Design

## Context

The owner's model (proposal) is server authority, the server mediating every
interaction near or far, each client keeping its own LOD, and a per-chunk
checksum that makes a client re-fetch a chunk that does not match. This
design fits that model onto what the tree already has, and names what is
missing.

**What exists (observed in the tree, 2026-11-18):**
- `pbd_core::edits::Edits`: every terrain edit, by cell, with the layers of a
  cell kept in the order they were made, and a `revision`. A column is a pure
  function of its direction, the worm field and these edits, so two processes
  with the same edits build the same column.
- The save's append-only journal, written by `SaveWriter` on its own thread,
  which reports a committed mark rather than a queued one.
- `pbd_core::records` with `Author::{Player(u32), Creation, World{..}}` and
  the yield set: a world change never overwrites a player's.
- `sim_origin.rs`: one `SimOrigin` per process, rebase hooks, and the rule
  that every stored position is a body-relative `f64` `WorldPos`.
- Weather, biomes, terrain and the sites are functions of the seed, the
  generator version and the world clock.

**What is designed but not built, and this change needs first:**
- `edit-pipeline` section 5: ACCEPT (validate, spend, journal, sequence) and
  APPLY (mutate columns, record, contact, light) as two halves.
- `world-persistence` 4.1 (the world clock and processes), 5.1 (regions and
  region snapshots with their last sequence), 5.2 (the segmented journal).

**What does not exist at all:** a network, a server process, other players,
and any rule for which machine runs the fauna, the water flow or the crops.

**Where gameplay rules live today.** Most rules are in `pbd-core` (dig,
farming, husbandry, cooking, fishing, inventory, goods, missions). Several
are still applied inside Bevy systems in `pbd-app` that read the mouse and
write the world in one go (`apply_edit` is the measured example). A server
has to be able to run a rule without a mouse, a window or a frame.

## Goals / Non-Goals

**Goals:**
- One authority, the server, for everything stored: edits, records,
  inventories, drops, containers, crops, animals, the clock.
- One code path: single player is a server in the same process.
- Each client streams only the regions near its player, and builds its own
  LOD, tier and light from them.
- A periodic per-region digest, and a resync of any region that differs.
- Nearby players see each other's changes in one agreed order; far players
  interact only through server transactions.
- 2 to 8 players, co-op, on a LAN or a direct address.
- Every old save opens, plays and hosts unchanged.

**Non-Goals:**
- PvP, damage between players, and griefing tools beyond kick, ban and rate
  limits (claims wait for `player-building` and town building).
- Matchmaking, a server list, relays and NAT traversal. Direct address and LAN
  first; a relay is a follow-up.
- More than one player aboard a moving craft. `vehicles` and `steer-and-sail`
  are not built; their seats are a follow-up on this design.
- Lockstep or deterministic physics. Avian is not bitwise deterministic across
  machines, and this design never needs it to be.
- Voice. Text chat only.

## Decisions

### 1. The "chunk" is the region

The owner's checksum unit is the `world-persistence` region: a level-5
triangle of the icosahedron, 20,480 on the planet, about 2,000 finest cells,
about 180 m across. It is already the unit of the save's snapshots, of
loading, and of the yield set, and every cell key maps to its region by
shifts. Using it for the network too means a resync is "send the region's
snapshot", the same bytes the save writes, and not a second format.

The alternatives were the fine tier's own blocks (they move with the tier's
anchor, so two clients would not agree what a chunk is) and per-cell hashing
(far too many digests).

### 2. A region's state, and its digest

A region's authoritative state is, in this order:
1. its terrain edits, by cell key ascending, each cell's layers in the order
   `Edits` keeps them;
2. its liquid cells (`water-flow`'s level and flags), by cell key and layer;
3. the records located in it (drops, containers, crops, animals, lamps,
   placed stations), by record id ascending, in their saved encoding.

The digest is a fixed 64-bit hash (FNV-1a 64 over the region's canonical
byte encoding, little-endian, no padding) written in `pbd-core` with a test
that pins its value for a fixed region. It is never `std`'s hasher, whose
seed is random per process. 64 bits is enough to find drift; it is not a
security boundary, and the server never trusts a client's digest for
anything but "send me the region".

The digest is kept per region and updated as each change is applied, so a
check costs a lookup, not a pass over 2,000 cells. A full recompute runs in
tests and in a debug check.

Derived data (columns, light, GPU records, the coarse LOD) is never
digested. It is a pure function of the state above; if two clients with
equal digests draw different things, that is a generator bug, which the join
check (decision 9) and the generator's own digest tests exist to catch.

### 3. Subscription: the regions near each player

Each client subscribes to the regions that the fine tier around its player
can reach, plus one ring of margin, so a region arrives before a fine cell in
it is built (`world-persistence` 5.3's rule, with the network as the source
instead of the disk). The exact radius is measured off the real tier when it
is built, not chosen here; at the current tier it is a few dozen regions.

On subscribe, the server sends the region's snapshot with its sequence and
digest, then every later change to it in order. On unsubscribe (hysteresis:
a second ring further out), the client drops the region's state; its own
coarse LOD there falls back to the seed.

**Far-away edits and the coarse LOD.** A client does not hold the edits of
regions it is not subscribed to, so a large build on the far side does not
show in its far terrain. That is accepted for this phase: the far LOD is
already coarse enough that single cells do not show. A small per-region
summary for the coarse LOD and the far night lights (a lamp count, a height
change) is a follow-up, named so it is not forgotten.

### 4. Accept on the server, apply everywhere, predict locally

Every request that changes stored state goes:
1. the client runs ACCEPT locally as a prediction, applies the result to its
   copy at once, and marks it provisional with its own request number;
2. it sends the request (what the player tried, not the result);
3. the server runs ACCEPT on its world, journals it, gives it the next
   sequence in its region, and replies `Accepted(seq)` or `Refused(reason)`;
4. the server sends the change to every subscriber of the region, the
   requester included;
5. a client applies changes in sequence order. Its own provisional change is
   confirmed when the server's version of it arrives. If the server refused
   it, or ordered another change to the same cell before it, the client
   rebuilds the touched columns from the server's edits, which is what the
   tier rebuild already does from cold every time the anchor moves.

The local click stays instant. Two players digging one cell is
last-writer-wins on the server's order, a rule, not two clients that have
quietly diverged.

The durability rule does not move: the server acknowledges a change as saved
only after `SaveWriter` reports it committed. A client has nothing to save.

### 5. The server is a core-only process, not a Bevy app

The server is a new crate, `pbd-server`, built on `pbd-core` and `pbd-net`
only. It runs at a fixed 20 Hz tick on its own thread. A dedicated server is
that crate's binary; a single-player or hosted game starts the same server
inside the game process and talks to it through an in-memory channel with the
same message types as the socket.

This is the reason the phase is large: every rule that today is applied
inside a Bevy system has to run in the core, called by the server. The
project already requires that (`pbd-core` owns rules; `pbd-app` adapts them),
so this is finishing that split, not inventing a new one. A rule moves when
its ACCEPT half can run from a request with no frame, mouse or camera.

The alternative, a Bevy `SubApp` for the server, keeps the rules where they
are, but ties a dedicated server to the engine and lets presentation leak
into authority. It was rejected for that reason.

### 6. Transport: QUIC

`pbd-net` uses QUIC (the `quinn` crate): reliable ordered streams for region
changes, snapshots and transactions (one stream per kind, so a large
snapshot does not hold up an edit), and unreliable datagrams for presence.
It comes with encryption and congestion control, needs one UDP port, and has
no Bevy version to pin against `=0.18.1`.

Rejected: entity-replication libraries (`bevy_replicon`, `lightyear`), whose
model is "replicate components of entities", while this world's state is
packed arrays and records, and which pin Bevy versions; a hand-rolled UDP
protocol, which would be months of reliability work.

Messages are encoded by hand in `pbd-core`, with the save format's
discipline: versioned, little-endian, length-prefixed, a test per message
that round-trips it. The connection's first message is the version check of
decision 9.

### 7. Positions on the wire are body-relative `f64`

A position or velocity sent between machines is a `WorldPos`: the body it is
relative to and an `f64` vector, the same thing a save stores. Each client
converts it into its own `SimOrigin` at the boundary. The server has no
render origin; where it simulates (fauna, decision 8) it uses one local frame
per group of nearby players, so one player's rebase never moves another
player's simulation (`docs/GAME_DESIGN.md` 15).

### 8. Who simulates what

| thing | runs on | the client |
| --- | --- | --- |
| terrain edits, records, inventories, drops, containers | server | predicts its own, applies the rest |
| crops | server (world clock) | draws growth from the record and the clock |
| animals (herd, wild fauna that can be tamed or killed) | server, for subscribed regions, at 10 Hz | interpolates |
| fish | each client draws its own schools; the catch is a server request | draws |
| water flow | server, for subscribed regions | applies changed liquid cells as region changes; the existing smoothing between ticks draws them |
| weather | each client, from the seed and the clock | computes |
| world clock | server; sent once a second | follows it |
| walking and flying the player | the client | sends position at 20 Hz |
| missions | server (they read the stored state) | shows them |

The player's own movement is the client's, as in most co-op games: it feels
right only if it is local. The server checks it is plausible (speed within
the walker's and craft's limits, not inside a solid cell by a column lookup)
and corrects a client that is not. Cheating in co-op among invited players is
not worth a server-run physics world for every player in this phase.

### 9. Joining

The first exchange carries the protocol version, the topology version, the
generator version, the content version (the item, recipe and block tables'
digest) and the world identity. Any mismatch refuses the join with a plain
message ("Game version differs", "Different world") and nothing else is sent.
After that: the player's record, the clock, and the snapshots of the regions
around their saved position. The loading screen stays up until those regions
and every render pipeline are ready, as the boot does now.

### 10. Players in the save

Each player is a record of a new kind, `player`, keyed by a player number,
holding their inventory, equipment, position (`WorldPos`), mission progress
and spawn. A player is known to a server by a key generated on their install;
the server maps it to a number the first time they join. The host of an old
save is player 0, which is what `Author::Player(0)` already means, so nothing
already in a save changes meaning. A save that has been hosted opens in single
player as player 0 and keeps the guests' records untouched.

### 11. The world clock with more than one player

The owner's rule is that world time passes only while the world is played.
With a server, the world is played while at least one player is connected:
the clock runs then, and stops when the last player leaves. A dedicated server
with nobody on it does not age its world.

### 12. Far interactions are transactions

A trade, a gift, a shared container, a message or a map marker between players
who share no region is a request naming both parties. The server applies it to
both players' records in one journal entry, so it happens wholly or not at all,
and tells each party. Neither client has to hold the other's regions.

### 13. Townsfolk come first, and are built for this

Phase 3 (the townsfolk) is built before this phase. Its NPCs, missions and
building should take ACCEPT/APPLY and records from the start, so this phase
does not have to move them. That is a note for those changes, recorded here.

### 14. Recommendations taken (ask only with screenshots)

None of these has anything to look at yet, so each is a recommendation taken,
not the owner's word. The ones with a look (14.4, 14.5) go to the survey with
the mockups.
1. Phase 3b is a new phase after the townsfolk, before town building.
2. Up to 8 players; co-op only; no PvP in this phase.
3. Host from the pause menu, join by address or on the LAN; optional password;
   the host can kick and ban.
4. The host and join screens: a mockup first (stone-and-carved-wood, no notes
   on screen, lit at night where it shows the world).
5. How another player looks: a simple body in the world's art style, no
   outline, lit like the ground, with their name over it only when near. A
   mockup first.
6. Text chat on a key; no voice.

## Risks / Trade-offs

- **Every stateful rule moves to the server path.** This is most of the work,
  and it touches every phase-1 system. Mitigation: move one rule at a time
  behind ACCEPT, with single player running through the in-process server from
  the first step, so a regression shows in single player at once.
- **A dependency chain.** `edit-pipeline` 5 and `world-persistence` 4 and 5
  are not built. This phase cannot start its network work until they are.
- **Server cost of water and fauna** grows with the number of regions players
  stand in. Bounded by 8 players and the subscription radius; measured on the
  owner's machine with a second client, never in the cloud container.
- **Prediction mismatches look like rubber-banding** of a dug cell or an item.
  They are rare by design (the server and the client run the same ACCEPT), and
  the digest catches anything that slips past.
- **Trusting the client's movement** allows a modified client to teleport.
  Accepted for invited co-op; plausibility checks catch the honest cases.
- **Far builds do not show** in other players' coarse LOD until the region
  summary follow-up (decision 3).
- **NAT.** Hosting over the internet needs a forwarded port until a relay
  exists. LAN and direct address work without one.
- **Saves.** The only change is a new record kind. Old saves open without
  migration; the `player` record is ignored by nothing because nothing old
  reads it.
