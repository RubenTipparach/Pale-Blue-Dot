## Why

The owner, 2026-11-18: "lets divise a plan for multiplayer. I think
multiplayer should be 3b", and then the model it should follow:

> server authority - server contains all data on item locations, map changes
> etc.
>
> a player interacting within the area of another player, the server must
> mediate those updates
>
> a player interacting on one side of the planet with another player on other
> side of the planet, server must mediate that. each player handles their own
> LOD local data
>
> sever periodically sends out checksum for the "chunk" client compares, and
> request new data if the "chunk data doest match

**"3b" is read as a new phase between phase 3 (the townsfolk) and phase 4
(town building and civil engineering).** It comes after the NPCs, missions,
carpentry and the first power, and before towns are built and grown. That
order is the right one: town building, roads and everything after it are
shared, permanent works, and they should be built once on a world that already
has more than one author, not retrofitted. (Recommendation taken, ask only
with screenshots: if the owner meant a step inside phase 3, the plan is the
same and only the tracker row moves.)

Most of the groundwork is already argued for in the tree, and this change is
where it is collected:
- `edit-pipeline` section 5 splits an edit into ACCEPT (the half that can
  refuse, which a server owns) and APPLY (the half that cannot, which any
  client runs), and puts a sequence on the log.
- `world-persistence` decision 8 cuts the planet into 20,480 regions (a
  level-5 icosahedron triangle, about 180 m, about 2,000 finest cells) with
  snapshots that carry their last sequence. A region is the "chunk" of the
  owner's model.
- `pbd_core::records::Author::Player(u32)` already names which player made a
  change; the single player is 0.
- Terrain, biomes, weather and the derived tier are pure functions of the
  seed, the generator version, the world clock and the edits. Two machines
  with the same edits build the same world without sending a single column.

What is missing is the network itself, the server process, the per-region
subscription, the checksum, and the rule for every non-terrain thing a player
can own or change (items, drops, chests, crops, animals, water).

## What Changes

- **A server owns the world.** It holds the one save: the edit log, records,
  every inventory, every dropped item, crop, animal and container. Clients
  send what the player tried to do; the server accepts or refuses it and
  sends out the result in order. A client never writes the world.
- **Single player runs the same server, in the same process.** There is one
  authoritative path, not a single-player path and a network path that drift
  apart. Hosting a game opens that in-process server to others; a dedicated
  server is the same code without a window.
- **Players near each other share a region stream.** Each client subscribes to
  the regions around it. An edit, a drop or an animal in a region goes, in the
  server's order, to every client subscribed to it. Two players digging one
  cell are resolved by that order, never by whichever client drew first.
- **Players far apart share nothing but the server.** Each client builds its
  own LOD, fine tier and light from the seed and the edits of the regions it
  holds. An interaction across the planet (a trade, a message, a shared
  container, a map marker) is a request to the server, which applies it to the
  authoritative store and tells each party.
- **The server checks the clients' copies.** Periodically, and when a client
  subscribes, the server sends each region's sequence and digest. The client
  digests its own copy at that sequence; on a mismatch it asks for the region
  and replaces its copy with the server's snapshot.
- **A join checks the world is the same world.** Protocol, topology,
  generator and content versions and the world identity must match, or the
  join is refused with a plain message.
- **Other players are seen.** Each client draws the others it can see,
  interpolated between the server's snapshots, in its own floating origin.
- **Saves are unchanged for single player.** An old save opens as before and
  can be hosted; player 0 is its owner. Every other player's state is a new
  record, added without changing what is already there.

Not in this change: PvP, server browsers or matchmaking, voice, more than 8
players, ships and vehicles with more than one person aboard (a follow-up once
`vehicles` and `steer-and-sail` exist), and anything after phase 4.

## Capabilities

### New Capabilities
- `world/multiplayer`: how a world is shared: one authority, joining, region
  subscription and ordered replication, the region checksum and resync,
  server-mediated interactions near and far, other players' presence, and what
  the world clock does with more than one player.

### Modified Capabilities
<!-- None: the requirements this rests on (the accept/apply seam, regions)
are themselves still in `edit-pipeline` and `world-persistence` changes, not in
openspec/specs. When those land, this change's dependency on them is a task,
not a spec delta. -->

## Impact

- **Depends on, and must land first:** `edit-pipeline` section 5 (accept,
  sequence, apply); `world-persistence` 4.1 (the world clock and processes),
  5.1 (regions and region snapshots) and 5.2 (the segmented journal).
- **New crate `pbd-net`**: the transport (sockets, connection, channels). No
  Bevy, no rendering. `pbd-core` gains the protocol's message types, their
  byte encoding and the region digest, which are pure functions with tests.
- **`pbd-app`**: a server plugin (authority, subscription, broadcast), a client
  plugin (prediction, reconcile, resync), the host and join screens, other
  players' bodies, and a `--server` headless launch.
- **Every gameplay rule that changes stored state** (digging, placing,
  crafting, inventory moves, drops and pickups, farming, herding, fishing's
  catch, cooking, missions, lamps) moves behind ACCEPT. Each is already a core
  rule; what changes is who calls it.
- **Saves:** additive only. A new per-player record kind; the journal's
  existing player token carries the player number. No migration of old saves.
- **Mockups first:** the host and join screens and how another player looks in
  the world are mockups for the owner before any game UI is built.
- **Frame cost** is not measured in a cloud session; the owner runs
  `perf_suite.py` with a second client connected.
