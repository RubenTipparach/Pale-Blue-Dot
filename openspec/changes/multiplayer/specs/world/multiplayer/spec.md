# Spec Delta

## Purpose

How one world is shared by several players: a single server holds and orders
every change, each client builds its own view of the planet from that, and the
server keeps every client's copy honest with per-region checksums.

## ADDED Requirements

### Requirement: The server is the only authority
The server SHALL hold the one authoritative copy of the world: terrain edits,
records, every inventory, every dropped item, container, crop, animal and the
world clock. A client SHALL NOT write the world or the save; it SHALL send
what its player tried to do, and change its copy only from what the server
sends back.

#### Scenario: A client's edit is a request
- **WHEN** a client digs a cell
- **THEN** the server validates the dig against its own world, spends from its
  own copy of that player's inventory, journals it with a sequence and only
  then sends it to the clients

#### Scenario: A refused request is undone on the client
- **WHEN** a client predicted an edit and the server refuses it (the cell was
  bedrock on the server, or the item was already spent)
- **THEN** the client's copy returns to the server's state for that cell and
  inventory, and nothing about the refused edit is saved anywhere

### Requirement: Single player runs the same authority
A single-player game SHALL run the same server in the same process, through
the same accept and apply path a networked game uses. Hosting SHALL open that
running server to other players without restarting or reloading the world.

#### Scenario: Opening a single-player world to others
- **WHEN** a player in a single-player world chooses to host
- **THEN** the world keeps running without a reload and other players can join
  it

#### Scenario: One rule, not two
- **WHEN** the same sequence of player inputs is played once in single player
  and once as a client of a separate server
- **THEN** the two resulting saves hold the same edits, records and inventories

### Requirement: A join checks it is the same world
On joining, the client and the server SHALL compare protocol version, topology
version, generator version, content version and world identity, and SHALL
refuse the join with a short plain message naming what differs if any of them
does not match.

#### Scenario: Different generator version
- **WHEN** a client whose build lacks the world's generator version tries to
  join
- **THEN** the join is refused with a message saying the game version differs,
  and the client's own saves are untouched

### Requirement: Clients subscribe to the regions near them
Each client SHALL hold a subscription to the regions around its player. The
server SHALL send a region's changes only to clients subscribed to it, and
SHALL send a newly subscribed client the region's state at a sequence before
any later change to it.

#### Scenario: Walking into a region another player edited
- **WHEN** a player walks toward a region another player dug a tunnel in
  earlier, while the first player was far away
- **THEN** the region is subscribed before its fine cells are built, and the
  tunnel is there when the player arrives

#### Scenario: Far regions are not streamed
- **WHEN** two players are on opposite sides of the planet
- **THEN** neither client receives the other's region changes

### Requirement: Changes in a region arrive in the server's order
The server SHALL give every accepted change a sequence and send each region's
changes to its subscribers in that order. Every client SHALL apply them in
that order, so two clients holding a region at one sequence hold the same
region.

#### Scenario: Two players dig the same cell
- **WHEN** two nearby players dig and place in the same cell at nearly the same
  moment
- **THEN** the server orders the two changes, and both clients end with the
  cell the later one leaves

#### Scenario: Placing a block where another player stands
- **WHEN** a player places a block in a cell another player's body occupies on
  the server
- **THEN** the server refuses the placement, as it would for the placing
  player's own body

### Requirement: Each client builds its own level of detail
A client SHALL build its own LOD, fine tier, contact, light and drawn world
from the seed, the generator version, the world clock and the edits of the
regions it holds. The server SHALL NOT send derived geometry, light or GPU
data, and no decision SHALL rest on a client's derived data.

#### Scenario: Two clients, two views, one world
- **WHEN** one player is on the ground and another flies high over the same
  region
- **THEN** each draws the region at its own detail, and both hold the same
  edits for it

### Requirement: The server checks each client's regions
The server SHALL periodically send, for every region a client is subscribed
to, the region's sequence and a digest of its authoritative state, and SHALL
send one when a client subscribes. The client SHALL digest its own copy at
that sequence and, on a mismatch, SHALL request the region and replace its
copy with the server's.

#### Scenario: A region that drifted is repaired
- **WHEN** a client's copy of a region differs from the server's at the same
  sequence (a lost message, a bug)
- **THEN** the digests differ, the client requests the region, and after the
  reply its copy and digest match the server's

#### Scenario: The digest is the same on every machine
- **WHEN** the same region state is digested on two machines with different
  operating systems or processor types
- **THEN** the two digests are equal

### Requirement: Interactions far apart go through the server
An interaction between two players who share no region (a trade, a message, a
shared container, a map marker) SHALL be a request to the server, which SHALL
apply it to the authoritative store as one transaction and tell each party the
result.

#### Scenario: Sending an item to a player across the planet
- **WHEN** a player gives an item to a player on the far side of the planet
- **THEN** the item leaves the giver's inventory and enters the receiver's in
  one server transaction, and if either step cannot happen neither does

### Requirement: Other players are seen in each client's own frame
A client SHALL draw every other player whose position it receives, placed from
a body-relative position converted into its own floating origin and smoothed
between the server's snapshots. A client's origin rebase SHALL NOT move any
other client or the server.

#### Scenario: Another player walking past
- **WHEN** another player walks past at the server's snapshot rate
- **THEN** their body moves smoothly, without snapping between snapshots, at
  the place the server reports

### Requirement: Durability belongs to the server
A change SHALL be acknowledged to the player who made it only after the
server's save has committed it. A client's disconnect, crash or quit SHALL NOT
lose a change the server committed.

#### Scenario: A client crashes after digging
- **WHEN** a client crashes just after the server acknowledged its dig
- **THEN** the dug cell is in the world when that player rejoins

### Requirement: Old saves stay playable
An existing single-player save SHALL open, play and host unchanged, with its
player as player 0. Joining players SHALL be stored as new records added to
the save, and a save that has been hosted SHALL still open in single player.

#### Scenario: Hosting a save from before multiplayer
- **WHEN** a save made before this change is opened and hosted, a second player
  joins and builds, and the world is later opened alone
- **THEN** the owner's inventory and every edit, theirs and the guest's, are
  there
