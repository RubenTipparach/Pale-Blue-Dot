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
