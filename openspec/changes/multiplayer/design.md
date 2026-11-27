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
| world clock | server; sent every 5 s (design 15.8) | runs it forward between |
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

### 15. What we take from Luanti

Luanti (formerly Minetest) is a shipped server-authoritative voxel game, and
its server answers several questions this design had left open. Read at
`luanti-org/luanti` `8bb14e8` (2026-11-16). Only the ideas are taken, not the
code: Luanti is LGPL-2.1-or-later, the same terms as the liquid rule
`water-flow` already ports with its provenance.

1. **Single player is a local server.** `client/game.cpp` builds a `Server` in
   process when `simple_singleplayer_mode` is set. A shipped game confirms
   decision 5.
2. **Near changes are sent; far changes mark the block stale.**
   `Server::sendNodeChangePkt` sends a single-node change only to players
   within 30 nodes who already hold the block. Anyone farther away, or still
   waiting for the block, gets `SetBlockNotSent`, and the whole block goes out
   again later at low priority. *Taken:* within a near radius (to be measured,
   about the dig reach plus a margin) a subscriber gets the change itself.
   Beyond it, the region is marked stale for that client and re-sent as a
   snapshot when it reaches the head of the send queue. This bounds traffic
   when many players stand at the edge of a busy region.
3. **A wrong prediction is fixed by resending the state, not by an undo
   message.** `RemoteClient::fixupNodePrediction` re-sends the block when a
   dig or place was refused, and also when the block was still on the wire
   when the client predicted, which is the race this design had not named.
   A refused inventory action re-sends the whole inventory (`SendInventory`).
   *Taken:* `Refused` carries no undo. The server re-sends the touched cells
   and the player's whole inventory. If a snapshot of that region was in
   flight at the time, it re-sends the region.
4. **Nearest first, a few at a time.** `RemoteClient::GetNextBlocks` walks
   outward from the player, remembers the nearest unsent distance, and keeps
   at most `max_simultaneous_block_sends_per_client` blocks in flight, each
   acknowledged by `TOSERVER_GOTBLOCKS`. *Taken:* snapshots go out nearest
   region first, with a per-client in-flight cap, acknowledged by the client.
5. **The client says what it dropped and how far it sees.**
   `TOSERVER_DELETEDBLOCKS` tells the server which blocks the client threw
   away, so they are re-sent if needed. `TOSERVER_PLAYERPOS` carries
   `wanted_range`, the client's view range, which sizes what the server sends.
   *Taken:* the client's unsubscribe message is that list. The client also
   sends its tier reach, from which the server sizes the subscription, rather
   than the server guessing it.
6. **Three separate radii.** `active_block_range` (what the server
   simulates: ABMs, liquids, entities), `active_object_send_range_blocks`
   (which entities are sent) and `max_block_send_distance` (which terrain is
   sent) are separate settings. *Taken:* the simulate radius (animals, water)
   is set apart from the subscribe radius (terrain and records) and the
   presence radius (other players drawn). Each is a validated config value.
7. **Cheat checks that switch off in single player.** `checkMovementCheat`
   (speed against `anticheat_movement_tolerance`), interaction distance
   (`checkInteractDistance`), and dig time against the tool (`m_dig_pool`)
   are each a flag in `anticheat_flags`. A failed check calls `on_cheat` and
   is logged. None of them runs in single player. *Taken:* decision 8's
   plausibility check becomes these three named checks, logged with the
   player and the reason, never run on the in-process server.
8. **The clock is sent every 5 s** (`time_send_interval`), and the client
   runs it forward in between. *Taken:* every 5 s rather than every second.
9. **A protocol range, not one version.** The server accepts any client
   protocol from `SERVER_PROTOCOL_VERSION_MIN` up to its own. *Taken:* the
   protocol version is a range, so an older client can join a newer server
   while the wire allows it. The generator, topology and content versions
   stay exact matches, because they decide the world itself.
10. **Content by hash.** Media is announced and cached by SHA-1
    (`TOCLIENT_ANNOUNCE_MEDIA`). Not needed while every player runs the same
    build. Kept as the model for mods and custom textures later.
11. **Rollback by author.** `server/rollback.cpp` records who changed which
    node, so an admin can undo one player's griefing. Our journal already
    names the author of every entry, so rollback is cheap. A named follow-up,
    since `docs/GAME_DESIGN.md` 16 asks for it on persistent servers.

**What Luanti does not have** is the owner's checksum. It relies on reliable
delivery plus resend on mispredict, and has no way to notice a copy that
drifted silently. The region digest (decision 2) is the safety net Luanti
lacks, so it stays. It can run at a slow rate, because resend-on-refuse
(item 3) catches the common case first.

### 16. Started early, and the order it is built in (2026-11-21)

The owner, 2026-11-21: "check out the multiplayer specs for phase 3, lets
get to work on that". So 3b starts now, ahead of phase 3's townsfolk, which
were to come first (decision 13). Nothing in this design rested on the
townsfolk being built; decision 13 only asks that they be written to run on
the server when they come, and they still will be.

**The region is `region-cell-keys`' region.** That change, written after
this one, defines a region as a triangle 64 cells on a side on every body.
On the home world (level 11) that is exactly the level-5 triangle of
decision 1, so nothing here changes; on other bodies it keeps the checksum
unit the same size in metres, which is the reason it was written. The
region's id is `region-cell-keys`' 32-bit region address, and a cell in it
is its 12-bit place. This change uses that change's address functions
(its task 2.1) and does not wait for its save migration (2.2 to 2.4): today's
`Edits` stay keyed by the 32-bit cell key, and a region's edits are found by
mapping each key to its region.

**The digest's sections are tagged.** Decision 2's three parts (edits,
liquids, records) are written as tagged sections, and an empty section is
left out. The edits section is built first; liquids join when the server
runs the water (task 8.4) and records when they are replicated (8.2). A
region with no liquids or records digests the same before and after those
sections exist, so the pinned test only moves when a section with content
is added to its fixture.

**Order of work.** Pure core pieces first, each with its own tests and none
of them visible in the game:
1. a cell key's region and place (`region-cell-keys` 2.1);
2. the region's canonical encoding and digest (task 3.1);
3. the protocol's messages and their byte encoding (task 3.2);
4. the edit's ACCEPT as a core rule (`edit-pipeline` 5; task 2.2's first
   rule), with the table of every other writer (task 2.1);
5. the in-process server (task 4.1) and single player talking to it (4.2).

The network, the screens and other players' bodies come after those, and
the screens wait for their mockups (tasks 5.1, 5.2).

### 17. Every writer of stored state today (task 2.1, 2026-11-21)

Everything that reaches the save goes through `WorldSave`
(`crates/pbd-app/src/saves/mod.rs`): `accept` (an edit, its drop and the
slots), `record_pick`, `record_toss`, `record_pack`, `deal_kit`,
`record_catch`, `record_hand`, `store`, `store_with_hands`, `accept_water`,
`propose`, `note_record_kinds`, and the whole-file `snapshot`,
`snapshot_weather` and `snapshot_vehicles`. Paths are under
`crates/pbd-app/src/`. "Inline" means the app decides it, not a core rule.

| Stored | App writer | Trigger | Core rule (or inline) |
| --- | --- | --- | --- |
| Dig (Air, its drop and finds) | `desktop/digging.rs` `dig_and_place` → `apply_edit` | left mouse held | `aim::march`, `dig::Breaking`, `farming::sprouts_seed`, `GatherSettings::find`, `station::ore_yield`; refusals inline in `apply_edit` |
| Place a block, lamp or station | `dig_and_place` → `apply_edit(Spend)` | right mouse | `aim::march`; body overlap, head room, crop in the way, one lamp a column inline |
| Bucket take or pour | `dig_and_place` → `apply_edit(Swap)` | right mouse | `liquid::bucket` |
| Till, and a turned-up seed | `desktop/farm.rs` `use_key` → `farm_act` → `apply_edit(Till)` | F | `farming::tillable`, `seed_turned_up`; `farm_act` itself is inline |
| Log taken from a tree | `digging.rs` `take_log` (`store_with_hands`) | left mouse on a trunk | `GatherSettings::tree_gives`, `flora::grows_tree`; trunk ray inline |
| Crop sown, uprooted, harvested | `farm.rs` `plant`, `uproot`, `harvest` | right mouse, dig, F | `Crop::of_seed`, `grows_in`, `Plot`, `FarmConfig::stage` |
| Wild plant, clutter, fruit, glow mushroom picked | `farm.rs` `pick`, `gather`; `desktop/fruit.rs` `pick`; `desktop/glow_plants.rs` `pick` | F | `FarmConfig::wild_crop`, `scatter::Kind::gives`, `flora::hanging_fruit`, `flora::glow_plant_point`; each ray inline |
| Craft, forge and campfire jobs queued and collected | `desktop/craft.rs` `queue`, `collect_hands`; `desktop/forge.rs` `work`; `desktop/kitchen.rs` `work` | a click, a timer, the panel open | `StationState::queue_for`, `queue`, `collect_into`; `cooking::concrete` |
| Meal eaten | `kitchen.rs` `eat_held` | right mouse with a dish | `cooking` `meals(..).eat` |
| Pack move, split, trash | `desktop/pack.rs` `press` (`record_pack`) | clicks | `Slots::shift`, `send`, `take` |
| Toss and pickup of drops | `drops.rs` `toss`, `gather` | Q, the magnet each frame | `drops::scattered`, `drops::step` |
| Tool in hand | `desktop/equipment.rs` `pick`, `pack_hand` | G, a click | `Equipment::hold` |
| Starting kit | `hotbar.rs` `Hotbar::restore` | world open | `Slots::give` (frame-free already) |
| Fish caught | `fish.rs` `land` (`record_catch`) | reel | `fishing::Angler`, `Line` |
| Boat launched and taken up, hold contents, refund | `vehicles/launch.rs`, `vehicles/hold.rs`, `vehicles/refund.rs` | right mouse, left held, pack clicks, a timer | `vehicle::hold::send`, `shift`; `station::retired_inputs` |
| Animal fed, tamed, products, pens, fences, incubators, eggs, pairing | `husbandry.rs` `use_on`, `feed_wild`, `settle`, `place_piece`, `take_piece`; `husbandry/breed.rs` | F, right mouse, a click, a timer | `husbandry` `Trust`, `FarmState`, `FenceBody`, `breeding::*` |
| Births and conceptions | `breed.rs` `realise` (`propose`, a world author) | every 0.5 s near | `breeding::farm_events` |
| Map waypoints | `desktop/map_screen.rs` `store_marks` | a click | `map::Mark::set`, `add_mark` |
| Town doors | `towns.rs` `use_doors` | F | `settlement::record::door_record` |
| Missions progress | `missions.rs` `count` | the other systems' messages | `missions::Progress::record`, `settle` |
| Water flow | `water_flow.rs` `run_ticks` (`accept_water`) | the water tick | `liquid` `tick` over the resident tier's columns |
| Beaver colonies and dams | `beaver.rs` `found`, `run` | a timer, the world clock | `Colony::new`, `beaver::catch_up` |
| Sites and towns | `sites.rs`, `towns.rs` | world open | `sites::generate`, `settlement::lay_on` (frame-free already) |
| Pose and clock, weather, fleet | `desktop.rs` `autosave`, `save_weather`; `vehicles.rs` `save_vehicles` | a timer, a menu, exit | inline; whole-file replaces |

**What has to move for a server to run them** (task 2.2):
1. **Picking.** Nearly every player writer starts from a camera ray
   (`aim::march`, `tree_on_ray`, `plant_on_ray`, `fruit_on_ray`,
   `aimed_beast`, `edge_at`, `door_in_reach`). The client sends the target
   it picked (a cell, a record id); the server checks the target is in reach
   and valid, and does not repeat the ray.
2. **`apply_edit`.** It reads a column's material from the resident render
   tier. The server reads it from a column it generates from the seed and
   the edits, which is the same pure function.
3. **Rules inline in the app:** `farm_act`, `apply_edit`'s refusals, the
   husbandry `use_on` flow, and missions counted from Bevy messages (a core
   rule returns its happenings instead).
4. **Slots changed before the save took the change**, which a server must
   never do: `fish::land`, husbandry's collect, `take_piece`,
   `use_incubator`, `place_piece`, `place_incubator`, `set_eggs`,
   `feed_wild`, `farm::uproot`, and `equipment.rs`'s `record_hand`.
5. **Two lines that are not one change:** a craft's collect (hand, then the
   station), a boat's launch and its take-up (fleet, then the pack).
6. **State outside the save:** a wild animal's trust, a door's leaf, and
   the whole-file fleet, pose and weather, which need one owner each (the
   server; pose per player, design 10).

### 18. Single player through the server, edits first (task 4.2, 2026-11-21)

The server crate exists (task 4.1) and decides a dig or a place with the
same core rule a client predicts with. Task 4.2 puts single player behind
it. The order below keeps every save byte for byte what it is today at each
step, which is that task's own check.

**Who writes the journal.** Today `WorldSave` is both the world and its
journal: `accept` sets the edit, numbers it and queues the line. With a
server, the server owns the world and the journal; the client holds a copy.
So `WorldSave` splits along that line, in this order:
1. **`SaveJournal`**: `pbd_server::Journal` implemented over the save's
   writer, writing exactly the line `accept` writes today (`line_of`, with
   the drop and the slots). The server's journal is then the save's file.
2. **`LocalServer`**, a Bevy resource holding `Server<SaveJournal>`, made
   when a world opens, from the save's edits, clock and slots. Single player
   is player 0 (design 10), joined with the save's install key.
3. **The edit path moves**: `apply_edit` predicts as now (core ACCEPT on the
   client's copy, then `FineSet::apply`), and sends the request to the
   `LocalServer` instead of calling `save.accept`. The server decides,
   journals, and answers in the same frame, because it is in process; the
   client applies the server's `Change` to `save.edits` (its copy) and
   confirms. A refusal returns the region from the snapshot and the slots
   from the server's copy.
4. **Every other writer** (design 17) follows the same way, one at a time,
   as its ACCEPT reaches the core (task 2.2). Until a writer has moved it
   keeps writing through `WorldSave` as today, on the same writer thread, so
   lines from both paths interleave in one file in the order they happen.

**What a request carries that the server cannot see.** A dig's finds (seed,
fibre, bait) and a till's seed are rolled by the app from config the core
does not hold yet (`GatherSettings`, `FarmConfig`). Until they move, the
client sends the hands it settled (`Request::Edit` with the hands), and
the in-process server takes them as asked; only a remote client is held to
the server's own roll (task 8.7). This is the one place single player trusts
the client, and it is the client that is the server.

**The column a server reads.** The client's tier makes its rim and the
river band solid (`planet_column.rs`), the server's columns are always
carved. Where a cave meets a rim cell the two could read a different
material, the server refuses, and the client's prediction is undone by the
snapshot. It cannot lose an edit; it can refuse one that looked valid. Each
one is logged as `SERVER_DISAGREES` with the cell, so a playtest shows
whether it happens.

**Checks for 4.2:** the same inputs make byte-identical saves before and
after (a scripted dig run, both builds, the logs compared); the starter
missions finish under `--capture`; an old save opens with its player as 0.

### 19. Pack moves and pickups through the server (task 2.2, 4.2; 2026-11-22)

The next two writers of design 17, after edits: the pack and the drops.
Both change only the player's own slots and, for a pickup, a drop. The
journal lines stay what they are (`pack`, `pick`), so saves do not change.

**Pack moves.** The pack screen's `click` today returns the slots after a
move. It becomes a resolver that returns the move itself, a `PackMove`:
`Shift { from, to, count }` (put a held stack down), `Send { index }`
(shift-click between hotbar and pack) or `Trash { index, count }` (a stack
dropped on the trash). Applying a move is a core rule,
`pbd_core::accept::pack`, which refuses a move from an empty slot or of
more than the slot holds, and otherwise applies `Slots::shift`, `send` or
`take`, the same calls the screen makes now. The server's
`Server::pack` decides it and the store journals the slots after it (the
`pack` line). A move that changes nothing is not sent. The protocol gains
`Request::Pack`. The hold (a craft's cargo, `hold::store`) stays as it is
until craft are shared (design 8's follow-up).

**Pickups need the drops.** The server can only decide a pickup if it
knows the drop: that it exists, what it holds and how many are left. Today
the live drops are the app's (`Drops`), and the save keeps them only as
they were at load. So the store keeps them: `WorldSave` holds the live
drops by id, kept by the lines that already make and change them (a dig's
drop, a toss, a pickup), and the store reads one with `Store::drop`.
Expiry stays the app's for now: an expired drop is never asked about, and a
load never restores one (`drops.rs`).

The magnet, the pull and when a drop is close enough stay the client's
(`drops::step`): they are motion, not stored state. When the client's step
says a drop is picked, it asks the server, `Server::pick`, which gives the
drop's item to the player's slots with the core's own give order and
journals the `pick` line with what is left. A drop the store does not hold
is refused (`Refusal::Gone`), which is what two players reaching one drop
at once will need: the first pickup empties it and the second is refused.
Reach is checked only for a remote player (task 8.7).

**Checks:** a pack move and a pickup through the server write the same
lines as straight through the save; a pickup of a drop that is gone is
refused and writes nothing; two pickups of one drop give it once.

### 20. Wiring two games together (tasks 6.1, 6.2; 2026-11-22)

The owner, 2026-11-22: "wire it up". The rules and the server are in place
(designs 16 to 19); this is the network between two processes, and a
second game joining a hosted one, with nothing new on screen. The host and
join screens and another player's body wait for their mockups (tasks 5.1,
5.2), so this step is driven from the command line and checked with logs,
tests and captures.

**`pbd-net`, the transport (task 6.1).** A crate on `quinn` (QUIC) and the
core's protocol, with no Bevy. It runs its own thread with its own async
runtime, and the game talks to it through plain channels, so no frame waits
on a socket:
- `Host::bind(port)` listens; each connection is numbered, and its
  messages come out as `(connection, ToServer)` and go in as
  `(To, ToClient)`, the server's own outbox type.
- `Client::connect(address)` gives one channel each way.
- One reliable, ordered stream each way, each message length-prefixed
  (`u32` little-endian, then the protocol's bytes). Presence moves to
  unreliable datagrams with task 8.1, when there is presence to send.
- The certificate is self-signed per run, and a client accepts any: this
  is play on a LAN or a direct address among friends (design 14.3), and
  the join check, not TLS, is what refuses another world. A trusted
  certificate is a follow-up with the relay.

**Joining a world you do not have (design 9, amended).** A joining player
does not hold the host's world, so the client's `Hello` names world 0,
"any", and the server's `Welcome` carries the world's seed, generator and
topology. The client opens a memory-only world from them: it writes no
save, because the world's save is the host's (design 4). A client that
names a world still has it checked exactly, as before.

**The host (task 6.2).** `--host <port>` starts single player as always,
with its in-process server, and also binds `pbd-net`: remote players join
the same `Server`, their requests are decided against the same store (the
host's save), and the server's outbox is sent each frame. The host's own
edits keep going through `decide_edit`; their `Change` now reaches the
remote players too.

**The joining client.** `--join <address:port>` connects, sends `Hello`,
and on `Welcome` opens the world it names. Then:
- its digs and places are predicted as now, and sent as requests instead
  of decided locally;
- every `Change` the server sends is applied to the client's copy of the
  edits and to its tier through `FineSet::apply`, the one APPLY;
- a `Refused` drops the prediction and the region's `Snapshot` that
  follows replaces the client's copy of it.

**First cut of the joined client (2026-11-22).** Three things are simpler
than decision 4 asks, each named so it is not mistaken for done:
- *No prediction yet.* A joined player's dig or place is sent and shows when
  the server's `Change` comes back, rather than at once and then rolled back
  on a refusal. A refusal then has nothing to undo. On a LAN the round trip
  is a frame or two; prediction with its rollback is a follow-up.
- *The pack starts as the kit on both sides.* The host joins a new remote
  player with the starting kit, which is what the joining game deals itself,
  so the two agree. Until the inventory message (task 8.2) a joined
  player's pack moves and pickups stay on their own game.
- *A remote dig floats nothing yet.* The server decides the dig, but the
  block it would float needs a drop record the client can see (task 8.2).

**Snapshots on join.** Until region subscription (task 6.3) is built, a
joining client is sent every region that has an edit, nearest first. That
is the whole world's edits, which is small today and is exactly what 6.3
replaces with the regions near the player.

**Not in this step:** subscription and its radii (6.3), the near and far
rule (6.4), the digest round-robin (7.1), other players' bodies and
presence (8.1, after mockup 5.2), and every writer of design 17 besides
edits, pack moves and pickups: on a joined client those still change only
the client's own slots and are not sent, which is said in the log.

**Checks:** two processes on loopback join and a mismatched generator is
refused with its plain message (6.1); a hosted game and a joined game on
loopback in a cloud run, a dig on each (done 2026-11-22: the joined game
took the host's three-layer hole and dug two more layers through it, and
the host applied them).

**The real test is on two machines (owner, 2026-11-23):** "Multiplayer
test should use Mac and laptop mining mike sessions". The two-machine check
of 6.2, and every multiplayer gate after it, runs on the owner's own
computers through their Claude sessions, "[Mac] pale-blue-dot" and
"[Laptop] mining mike continue", not on one cloud container's loopback:
1. both check out the PR's branch and build `fast`;
2. the Mac hosts: `pbd-app --world mp-test --host 47123`, and reports its
   LAN address;
3. the laptop joins: `pbd-app --join <mac address>:47123`;
4. each digs and places by hand or with `--dig`; each takes a screenshot
   showing the other's change, and the logs (`player 1 joined`, `edits from
   the network applied`, any `SERVER_DISAGREES` or refusal) go with them;
5. the shots and logs land in `docs/screenshots/multiplayer-two-machines/`.
The GPUs make it real time, which the cloud's software renderer is not.
The laptop cannot run the game for long (owner, 2026-11-23: "the laptop's
gpu cant handle the extended load"): it joins, does its checks and its
screenshot, and closes. A long run (a soak, a recording) puts the Mac and
the desktop on it instead. First run, 2026-11-23: passed
(`docs/screenshots/multiplayer-two-machines/`), then paused on the owner's
word.

### 21. A headless server and protocol bots on one Mac (2026-11-23)

The owner, 2026-11-23, with the desktop busy and the laptop unable to take
a long run: "do you think you can playtest networking on your own? ... or
make a headless version?", then "3 headless", and "have a live mac session
connect to a server on the mac, and spin up a headless version on the mac
using loop back to playtest the protocol". This adds the pieces for that
and nothing to the game.

**`pbd-headless`, a crate with one binary.** It depends on `pbd-core`,
`pbd-server` and `pbd-net` and has no Bevy, so it builds in seconds and
runs anywhere. Three subcommands:
- `serve --port <p> [--seed <s>] [--clock <s>] [--kit <n>]` is a
  dedicated server: the same `Server` the host's game runs, over a
  `MemoryStore`, ticked at `TICK_S`, with `pbd-net`'s `Host` in front. It
  is the first cut of task 6.2's `--server`: its world lives in memory
  and is gone when it stops; a dedicated server's save is the follow-up.
  It logs the lines the host's game logs (`player N joined`, `left`, each
  refusal) and, every 10 s, `SERVED` with players, requests, accepted,
  refused and changes sent.
- `bot --join <addr> --role <i>/<n> [--site <lat,lon>] [--stress <s>]`
  is a joined player with no window: it sends `Hello` naming world 0, as
  the game does, keeps its own copy of the edits from the snapshots and
  changes, and plays a script (below) through the same `Request`s a
  player's hands send. It checks as it goes that every request is answered
  once and that each region's changes arrive in `seq` order with no gap,
  and at the end that its copy of every region it holds digests the same
  as the server's (it asks for each with `Resync` and compares `fnv1a64`
  of the bytes with its own `Regions` digest). It prints `BOT_OK` or
  `BOT_FAIL <why>` and exits 0 or 1, with the round-trip times (p50, p99,
  max).
- `soak --bots <n> [--stress <s>]` runs a server and `n` bots in one
  process over real QUIC on loopback, and checks every bot's copy against
  the store's. A cargo test runs a short one.

**The script: a hut built by several hands.** Every bot reads the same
site, the finest cell nearest `--site` (by default about 6 m from the
game's default spawn, so a joined game sees it from where it stands), and
its ring of neighbours. The hut is walls two high on the ring with one
door left open, and a roof one layer above them over the ring and the
centre. Each piece goes to the bot whose role is its index modulo `n`;
each bot also tries the centre's roof block, so one is accepted and the
rest are refused as `Unchanged`: the contention the server must order.
Before building, each bot digs its own small pit beside the site (and
digs the bottom twice, so a refusal on air is seen), and `--stress` then
digs and places at random cells near the site at a player's swing rate
for that long.

**Pickups do not cross the network yet (task 8.2)**, so a remote player's
dug block floats on no one's ground and a remote player can place only
what it joined with. The test server therefore deals each joining player
`--kit` stone blocks (64 by default); the host's game deals the starting
kit, which holds none. This is a test affordance and is said in the log.

**The run on the Mac.** `pbd-headless serve` on `127.0.0.1:47124`, the
live game `pbd-app --join 127.0.0.1:47124` in its window on the GPU, then
three bots. The game's player watches the hut go up and digs beside it;
its log must show `edits from the network applied` and no refusal it did
not cause, the bots must all print `BOT_OK`, and the shots and logs go in
`docs/screenshots/multiplayer-headless/`. What one machine cannot show is
left to the two-machine runs: a real network's loss and delay, firewalls,
and Windows against macOS.

**Found on the first run:** `--join` opened on the title screen, because
the opening screen counted only `--world` as naming a world; CONTINUE
there would have opened the last saved world in place of the host's. A
joined game now opens in play, as a named world does.

**And the second:** a game killed and started again rejoins as the same
player (its install key) before the server has seen the old connection
close. Both the host's game and the headless server sent the player's
messages to the first connection found for them, the dead one, so the
restarted game never had its welcome; and the old connection's close,
some 30 s later, would have taken the player out. A rejoin now forgets
and closes the old connection (`a_player_who_rejoins_is_served_on_the_new_connection`).

**And the third:** a joining game read its welcome and dropped every
message after it in the same batch, which is where the server's snapshots
of the world's edits come. A game that joined after others had built saw
none of it. The whole batch is now read before the welcome is acted on.

**First run, 2026-11-23: passed** (`docs/screenshots/multiplayer-headless/`).
Three bots built the hut in front of the live game and stressed it for
30 s: 387 requests, all answered once, every change in order, every bot's
copy equal to the server's; round trip p50 5-6 ms, p99 9-10 ms. The live
game applied every change, its own dig was accepted, and restarted it
rejoined and was sent the hut. A seven-bot soak for 60 s passed the same
checks (1715 requests).

### 22. A joined player's own pack (2026-11-27)

**Found.** The two-machine run (2026-11-23) noted the joining player's
hotbar empty. The starting kit has been empty since 2026-11-17, so an empty
hotbar on a first join is right. What is wrong is behind it: nothing sends
a joined player their own slots. The server keeps every player's slots
(`Player::slots`) and decides digs, places, pack moves and pickups against
them, but no message carries them back. A joined game makes no prediction
(design 20, first cut), so:

- a block the player digs goes into the server's copy and never shows up
  in their hotbar, so they have nothing to place;
- a returning player gets the starting kit again, because the host passes
  the kit to every `join`, even for a key it already knows;
- a refusal cannot put the pack back (design 15.3 and `refuse_hands` say
  "until the inventory message").

**The message.** `ToClient::Slots { slots }`: the player's whole carried
slots, the hotbar's ten and then the pack's thirty, sent only to that
player. A slot goes on the wire as a count (u16, 0 for empty) and then its
item: a kind byte, then the material's save code (u8) for a block, the tool
in `Tool::ALL` order (u8), the fish's roster index (u16), or the good's
code as text. A good goes by its code, never its table index, as the save
does (`farming-and-smelting` decision 1). An unknown good or material
fails to decode (`Decode::Invalid`). Which slot is selected is not sent: it
is the player's hand on their own game. The protocol goes to version 2,
and so does its minimum, so a version-1 game is refused with "Game version
differs" rather than missing messages it cannot read.

**When it is sent:**
- right after `Welcome`;
- after every accepted request that changed that player's slots (a dig, a
  place, a pack move or a pickup), after the request's `Accepted`;
- after every refused request about the hands or a cell, so a prediction
  is put back (design 15.3).

The host's own player, player 0, is never sent it: its slots are the app's
own (design 18).

**The joined game** sets its `Hotbar` to the slots it is sent and keeps its
own selected index. Its pack screen, the held item and the build preview
read the `Hotbar`, so they follow.

**Away is not gone.** `Server::leave` keeps a departed player's slots by
their number, and `join` gives a known key the slots it left with, not the
new player's kit. This lasts only while the host is running: keeping them
in the host's save is design 10's `player` record, which comes next.

**Found with it: a guest's slots were written as the host's.** On a
hosted world, the host's save is the store (design 18), and its lines carry
no author: every player line is player 0's (`world-persistence` decision
11). But `SaveStore` wrote a guest's dig, place, pack move or pickup with
the guest's slots. Only a `debug_assert` stood in the way, and it is off in
the `fast` and release builds. So the host's in-memory carried slots, and
after a reload the host's pack, became the guest's last slots. The fix is
in `SaveStore`:
- a guest's edit is written with the host's own slots as they stand;
- a guest's pickup writes what is left on the drop, also with the host's
  own slots;
- a guest's pack move writes no line, because the save has nothing of the
  guest's to keep until design 10's player record.

The guest's slots stay in the server's memory. A save written during the
2026-11-23 run may already have the laptop player's slots as the Mac
player's; the owner can check the Mac's pack.
Check: `a_guests_hands_never_become_the_hosts_slots`.

**Not covered here:** the aim fault from the same run, where bare hands
dug only straight down for the laptop's player. The request carries the
cell the client picked, so the fault is on the joined game's side. It
needs a run on the owner's machines to find, and it stays open in the
two-machine notes.

**Checks:**
- the round trip of `Slots` with one of each kind;
- a good whose code this build lacks fails to decode;
- a remote player's dig is answered with `Accepted` and then `Slots`
  holding the block;
- a refused place is answered with `Slots` as they were;
- a player who leaves and rejoins gets the slots they left with, not the
  kit;
- the joined game's `Hotbar` takes the slots and keeps its selected slot.

### 23. Drops through the server (task 8.2, the drops; 2026-11-27)

**Found.** Writing design 22's tests showed why a joined player cannot
build: a dig does not put its block in the pack, it floats out as a drop
for the magnet to pull in (`inventory-grid` decision 4). The server works
out what floats (`Decided::floats`), and for a remote player throws it away
(`handle`: "What floats out becomes a drop record with task 8.2"). Nobody
else sees a drop either: the host's own digs make drops only in the host's
app. So a joined player's dig yields nothing, and the first thing they try
in the world does nothing for them.

**The drop is made where the dig is decided.**
- Single player and the host's own player: the app makes the drop before
  asking, as today (`Drops::make`, at the dug cell plus the lift), and the
  store writes it on the dig's line.
- A remote player's dig: the app is not there, so the store makes the drop,
  `Store::make_drop(item, cell, layer, made_s)`. It is the same drop
  `Drops::make` would make: the save's next id, at the cell's centre
  half a metre up, scattered by its id, in the body's frame (f64), made at
  the server's world clock. It is written on the dig's line as the app's
  drop is. The save's lines do not change.

**Everyone is told.** Two messages, both broadcast to every player:
- `ToClient::Drop { drop }` when a drop is made by a dig, whoever dug. It
  carries the id, the item and count (design 22's slot encoding), the
  position (f64, the home body's frame) and the time it was made.
- `ToClient::DropLeft { id, left }` when a pickup is decided, whoever
  picked: how many are left, 0 for gone.

The host's own game is player 0. It gets both through the outbox, adds a
drop it does not already hold, and sets what is left on one it does. Its
own digs and pickups are already in its `Drops`, so applying them again
changes nothing.

**A joined game's magnet asks.** Its drops are only those it was told
about. When the magnet brings one in (`drops::step` says picked), it sends
`Request::Pick { drop }` once, through the same queue as a dig, and changes
nothing itself. What it takes comes back as `DropLeft` and `Slots`. Two
players reaching one drop are given it once, by the server (design 19).

**Not yet:** a toss (Q, or a stack dragged off the pack) by a joined
player, and a host's toss being seen by its guests; a chest broken by a
guest, whose contents spill; and expiry, which each game still runs on its
own clock from the drop's `made_s`. Drops are not yet sent to a player who
joins after they were made. All of these are left for region records (the
rest of 8.2, with 6.3's snapshots).

**Checks:**
- the two messages round-trip;
- a remote player's dig makes a drop in the store, writes it on the dig's
  line, and broadcasts `Drop` to everyone;
- the host's own dig with the app's drop broadcasts that drop;
- a remote player's pickup broadcasts `DropLeft` and sends `Slots` holding
  the item;
- the host's game adds a guest's drop to its `Drops` once.

### 24. Other players in the world (task 8.1; 2026-11-27)

The owner, 2026-11-27, on the handoff's list: "I asked to get it all
done". Seeing another player is the gap the handoff names first. The
protocol has carried a pose both ways since 3.2 (`ToServer::Presence`,
`ToClient::Presence`), but nothing sends it, the server drops it, and no
game draws it.

**What is sent.** Each game sends its own player's pose at 20 Hz while it
is joined or hosting:
- the feet, as a `WorldPos` in the home body's frame (f64): the walker's
  centre brought down by its half height, from the origin's own methods,
  never from a planet-centred f32;
- yaw, the way the camera faces, measured in the tangent plane at the feet
  from a fixed tangent frame (`protocol::tangent_frame`: east from the
  body's Z axis crossed with up, Y when up is along Z; north completes
  it). `yaw_of` and `forward_of` are one core pair, so the sender and the
  drawer cannot disagree.
- pitch, the camera's look up or down.

**The server** keeps each player's last pose and passes each one it
receives to every other player as `ToClient::Presence`, never back to
its sender. The host's own pose goes through the same call
(`Server::presence`), so the host is seen as any player is. When a
player leaves, everyone is told with a new `ToClient::Gone { player }`.
It rides the one reliable stream, as everything does today; unreliable
datagrams for presence (design 1) wait for a measured need.

**Each game** keeps, per other player, the last two poses it received with
the time each arrived (`Others`). It draws them 100 ms behind (the gap at
20 Hz, and a little more), interpolating the feet in f64 along the
chord and putting them back on the sphere's radius. Yaw turns the
short way. The stored state is f64 in the body's frame, so a rebase of
one game's origin moves nothing it stores; the figure's local position
is worked out from the origin each frame. Once the last pose is 2 s
old, the player stays where they were last seen.

**The figure is the townsfolk's**, approved in M5 and drawn by
`townsfolk`: one shaped, textured body with a face, its walk animation,
the field's light, and its fade in and out. Other players are the
people of a reserved site, `PLAYERS_SITE`, with one person each,
numbered by the player, a look generated from the player's number with a
village tunic. They are driven each frame (`Townsfolk::drive`) and walk
when they move faster than 0.3 m/s. They are not townsfolk to anything
else:
- `everyone()` leaves them out, so combat, guards and town missions never
  see them; a swing at another player does nothing
  (`survival-mode` 4.7's single-player half stands);
- they get no `Talker`.
- The walker's solids include them, as they do townsfolk, so players
  cannot walk through each other.

Their name over them waits for mockup 5.2 (names are empty today: the
join sends none).

**Checks:**
- `yaw_of` and `forward_of` round trip, at the poles too;
- the server passes a pose to the others and not back to the sender, and
  tells everyone `Gone` when a player leaves;
- interpolation is 100 ms behind and on the sphere;
- two games with different origins put the same other player at the same
  world point;
- a visitor is not in `everyone()`, and has no `Talker`.

### 25. A toss through the server (task 2.2, the toss; 2026-11-27)

Design 23 left the toss out: Q (or a stack dragged off the pack) took the
stack from the slots and wrote the toss line in the app alone, so a joined
player's toss did nothing on the host and a host's toss was never seen.

- **The rule** is `pbd_core::accept::toss(slots, slot, count)`: refused
  for an empty slot or more than it holds, otherwise the slots with
  `count` taken and the stack that was taken.
- **The request** is `Request::Toss { slot, count, at_mm }`. `at_mm` is
  where it lands, the app's point ahead of the chest, in whole millimetres
  in the body's frame. Millimetres keep `Request` `Eq`, which the joined
  game's queue needs, and are finer than anything a drop draws. Reach is
  checked with the rest of 8.7.
- **The server** decides it (`Server::toss`), and the store writes the
  toss line it always wrote (`Store::commit_toss`), naming the drop by its
  next id and pushed aside by it (`drops::scattered`), as `Drops::make`
  always placed a toss. A guest's toss carries the host's own slots, as in
  design 22.
  Everyone is told with `Drop`, and the player gets `Slots`.
- **The made time:** the host's own toss uses the app's clock, so single
  player's lines stay byte for byte what they were. A remote toss uses the
  server's clock.
- **A log thrown over water** stays the app's (`logs::throw_in`) until
  floating logs are shared.

**Checks:**
- the rule refuses an empty slot and too many;
- a guest's toss makes a drop everyone is told of and the guest's slots
  lose the stack;
- the host's toss through the server writes the line `record_toss`
  always wrote.

### 26. Crops through the server, and world records sent (task 2.2 and 8.2, crops; 2026-11-27)

Sowing and harvesting were decided in the app (`farm.rs` `plant`,
`harvest`), wrote a `plot` record with the slots, and nobody else ever saw
the record. This moves both into the core and the server, and sends every
record the server's store takes to every player.

**The rules** (`pbd_core::farming::accept`):
- `plant(ground, over, growing, climate, slots, layer, now_s)` takes one
  seed from the selected slot and returns the sown plot and the slots
  after. It is refused when:
  - the ground is not tilled soil with air over it (`NotTilled`);
  - a crop already grows there (`Growing`);
  - the climate does not suit the crop (`Climate`);
  - the slot holds no seed (`EmptySlot`).
- `harvest(plot, farm, slots, now_s)` returns the bare plot, the slots
  with the produce and seeds given, and how many of each. It is refused
  when the crop is not ripe (`NotRipe`) or the pack has no room (`NoRoom`).
  Nothing is given unless all of it fits, as today.

The app keeps its notes ("Seeds need tilled soil", "Ripe in 6 h") and
reads them from the refusal.

**The server** decides `Request::Plant { cell, layer, slot }` and
`Request::Harvest { cell }`:
- the ground comes from its column (seed and edits), the climate from
  `planet_gen::biome_at` at the cell's direction, as `climate_at` reads
  it;
- the plot comes from the store (`Store::record`);
- the farm's tuning is set on the server like the crew chain
  (`Server::set_farm`), and a harvest asked of a server with none is
  refused.

The store writes the record with the player's slots
(`Store::commit_record_with_hands`; a guest's with the host's own, design
22).

**Records are sent.** Every record the server's store takes, whether a
plot, a crew record or anything that comes this way later, is broadcast
as `ToClient::Record { record }` (kind, id, schema and its RON body). A
joined game puts it in its own records (`WorldSave::take_remote_record`),
so its crops draw from it as the host's do. The refusals grow by the
farming ones above, in the protocol's code table (protocol 2 is not yet
released, so its number stays).

**Not yet:** tilling stays an edit (it already goes through the server),
fertiliser, wild plants and clutter picking, and the missions' `Planted`
and `Gained` counts for a joined player (missions are a later writer).

**Checks:**
- each refusal of the rules, and a harvest that gives exactly what
  `harvest` gave;
- a guest's sowing through the server is broadcast as a `Record`, and a
  harvest of it gives the guest the produce;
- the host's sowing through the server writes the line `store_with_hands`
  always wrote.

### 27. A persistent server on Fly.io, the Tenebris way (2026-11-27)

The owner, 2026-11-27: "Use tenebris hosting pattern for now where we can
deploy a persistent server to fly.io, but yea the option is there if
someone wants to host server themselves". Hosting from the pause menu
(design 20, mockup 5.1) stays. Alongside it, a dedicated server runs on
Fly.io and is the game's default server, as `tenebris.fly.dev` is
Tenebris's.

**What Tenebris does** (`RubenTipparach/tenebris`: `tenebris-rs/Dockerfile`,
`tenebris-rs/fly.toml`, `server-js/fly.toml`,
`.github/workflows/server.yml`):
- a two-stage Dockerfile: `rust:bookworm` builds the server crate with
  `--release --locked`, and `debian:bookworm-slim` runs only the binary and
  its config tree;
- `fly.toml`: one app in `ord`, one shared-CPU machine, and the volume
  `tenebris_saves` mounted at `/data`, which holds the worlds and survives
  every deploy. Its comment warns that `fly launch --copy-config` strips the
  mount;
- a `/health` check and an `/admin` page (open for now, or behind an
  `ADMIN_TOKEN` secret);
- a GitHub workflow, run by hand or called by the release workflow. It
  needs a `FLY_API_TOKEN` deploy token, makes the volume the first time,
  and deploys with `fly deploy --ha=false`, one deploy at a time.

**What differs here, and the choice made** (recommendation taken, ask
only with screenshots):
- Tenebris speaks WebSocket over Fly's HTTP proxy. This game speaks QUIC
  over UDP (`pbd-net`, design 20).
- Fly serves UDP through a `[[services]]` block with `protocol = "udp"`.
  The server binds `fly-global-services`, and the app needs a dedicated
  IPv4 address.
- Fly's proxy does not start a stopped machine for UDP, so the machine is
  kept running (`min_machines_running = 1`). Tenebris's machines stop when
  idle.
- **Taken:** keep QUIC and pay for the dedicated address. A second
  transport (QUIC for self-hosting, WebSocket for Fly) would split one
  protocol into two to keep working, the thing the one-rule principle
  forbids. If Fly's UDP proves troublesome, a WebSocket transport is the
  fallback: `pbd-net` keeps its channels and swaps the socket.
- `/health` and `/admin` are served over HTTP on a separate TCP port by
  the same binary.

**The server binary** is `pbd-headless`'s `serve` (design 21) with two
additions:
- a store on disk, not `MemoryStore`: the same journal format as a
  `WorldSave` (the save's format module moves to a crate both can use,
  without Bevy), under `/data/worlds/<id>/`;
- `--server <port> --world <id>`, which opens or makes a world with a
  seed and serves it. The world clock runs only while someone is on
  (design 11).

**What goes in the repository:** `deploy/fly/Dockerfile`,
`deploy/fly/fly.toml` (app `pale-blue-dot`, volume `pbd_worlds` at
`/data`), and `.github/workflows/server-deploy.yml`. Deploying needs the
owner's Fly account, a `FLY_API_TOKEN` repository secret and the
dedicated IPv4 address: the owner's to set up, once.

**In the game:** the Multiplayer menu (design 28) lists the hosted server
first, then LAN games, then joining by address. Hosting stays in the
pause menu.

**Checks:**
- the on-disk store writes and reads back the same journal a `WorldSave`
  does (a byte-for-byte test);
- `pbd-headless --server` restarted keeps its world;
- the image builds in CI.

The first real deploy is the owner's, with a bot soak against it from the
Mac.

### 28. The title menu: Continue, New world, Multiplayer, Options, Quit (2026-11-27)

The owner, 2026-11-27: "Change menu to continue, new world, multiplayer,
options and quit. Continue will have worlds you can load. Or load latest
save".

- **Continue** opens the saved worlds list (today's saves page), with
  **Load latest** at its top. A plain launch's front door is still that
  list (CLAUDE.md, the saves page).
- **New world**: today's new world flow.
- **Multiplayer**:
  - the hosted server (design 27), then games found on the LAN, then
    join by address, with the refusal messages of mockup 5.1;
  - hosting your own open world stays in the pause menu.
- **Options**: today's settings, with a Character page beside them
  (`player-character`).
- **Quit.**

Every word comes from the locale file. The mockup is M40's Join tab,
updated with the Continue list. The game changes after the owner approves
it.

### 29. The digest round robin and the resync (task 7.1; 2026-11-27)

Design 2 says what a region's digest is. This is how a joined game finds a
region it has wrong and gets it right.

- **The server** sends one region's digest every `DIGEST_EVERY_S` (2 s) to
  every player: `Digest { region, seq, digest }`, the next region with an
  edit in a fixed rotation (`Server::send_digest`). The host's game and the
  dedicated server call it on their loops. It carries the server's
  sequence, so a game still waiting for changes in flight is not alarmed
  by them.
- **A joined game** keeps the server's last sequence per region, from every
  `Change` and `Snapshot` it applies. It does not use its own ledger's
  count, which counts layers set and not the server's accepts. On a
  `Digest` whose sequence it has reached, it compares its own digest of
  the region. If they differ it sends `Resync { region }` once (it is
  queued like a request, and not sent again while one is waiting).
- **A snapshot replaces the region whole.** Edits are only ever added or
  replaced, so a wrong extra layer would survive a merge. A snapshot first
  clears every cell of that region in the game's copy (`Edits::clear_cell`,
  new), then sets the snapshot's edits and recomputes those cells' digests
  (`Ledger::resync`, new). The join's first snapshots go the same way.
- The tier is brought round to the replaced cells through the same APPLY.

**Checks:**
- `Edits::clear_cell`;
- the server's rotation visits every region with an edit;
- a game whose copy of a region is wrong asks for it once, and the
  snapshot makes its digest equal the server's;
- a game that is behind in sequence does not ask.

### 30. The cheat checks (task 8.7; 2026-11-27)

Design 15.7's three named checks, each a flag in the server's `Cheats`,
all off unless set. A dedicated server sets them from its flags. The host's
in-process server never sets them, for its own player or anyone else, and
single player never runs them.
- **Movement** (`--max-speed <m/s>`): a pose whose distance from the last,
  over the time between them, is faster than the limit is logged as
  `CHEAT player N: moved X m/s` and not passed on to the others. The next
  pose within the limit is.
- **Reach** (`--reach <m>`): a request whose target is farther than the
  limit from the player's last pose is refused with `Refusal::Cheat` and
  logged. The target is the dug or placed cell's layer, the sown or
  harvested cell, the drop picked up, or where a toss lands. A request
  before any pose has arrived is not checked.
- **Dig time** against the tool is not built. It needs the swing's start
  in the request, and no request carries that yet. It stays listed here.

Every failed check is kept as `(player, reason)` in `Server::cheats`, for
the dedicated server's log and a later admin page.

**Checks:** a teleporting pose is not relayed and is logged; a dig past
reach is refused as `Cheat`, one within it is taken; with no flags set,
nothing is checked.

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
