# Proposal: save slots, and a writer that never blocks the frame

## Why

**The owner's words: "make sure to implement game saves as well. save slots,
abillity to delete slot and create new slots. loading up each world puts me
back in the same position, and inventory items and I expect the world state to
be as it were when I logged off/autosave happened"**, and then: **"I expect
saving to be fully async"**.

What exists is one hard-coded path, `saves/preview/edits.txt`, holding one kind
of thing: the block edits. There is no slot, nothing to create or delete, and
the two parts of a save a player would notice first - where they were standing
and what they were carrying - are not written at all. Walk a kilometre, fill
the hotbar, quit, and you come back at the spawn with the starting kit and a
field of holes you did not dig from there.

## What a save is

Three things, and they are durable on different clocks BECAUSE they change on
different clocks.

| | written | why |
| --- | --- | --- |
| Block edits | per edit, appended | a dig is a decision; it cannot wait on a timer |
| The hotbar | per edit, on the same line | an edit is what CHANGES it; see below |
| Pose and selection | on a timer, and on quit | cheap to lose, expensive to write |

**The hotbar rides the edit log, and that is the one design decision here worth
arguing.** The obvious shape is a periodic snapshot of pose and inventory
together. It is wrong in a way that is invisible until it bites: a block edit
is durable the instant it happens and the inventory would be durable up to five
seconds later, so a crash in between leaves a world where the hole is dug and
the block it yielded was never picked up. The world and the player would
disagree about an event they both took part in.

What changes a stack is a dig or a place. So the log line carries the whole
hotbar after the edit, and replaying the log reconstructs both from one record.
Ten slots is about eighty bytes on an operation that already costs an fsync,
and the inventory needs no timer of its own.

Pose is the one thing that genuinely IS cheap to lose, which is the reference
project's own rule: "The 5 s pose autosave is acceptable because pose is cheap
to re-derive - block + torch state is not."

## Fully async, and how that squares with the durability rule

`CLAUDE.md` says: "Every accepted world mutation enters the durable transaction
path immediately. A queued write alone is not a durable save; acknowledge
commitment only after the storage backend succeeds."

The owner now asks for saving to be fully async. **These are the same
requirement once you separate the WRITE from the ACKNOWLEDGEMENT**, and the
rule's own wording is what says so: it forbids acknowledging on the enqueue,
not writing off the frame.

- A save thread owns the files. The game hands it a record and returns
  immediately: no `write`, no `flush`, no `fsync` on the frame, ever.
- The record is acknowledged only when the thread has fsynced it. A committed
  sequence number is what says so, and the saves screen reads it.
- A burst of digs costs ONE fsync, not one each: the thread drains everything
  waiting, writes it, syncs once, and publishes the highest sequence it got
  down. Batching is what async buys, and it only became available when the
  write left the frame.
- Quitting drains. The one place it is right to block is the place a player is
  already waiting, and losing the last two digs because the writer had a
  millisecond of work left would be the whole feature failing at the one moment
  it is most visible.

So the acknowledgement is later than the enqueue by whatever the disk takes,
and it is never assumed. That is a stronger position than the synchronous
version, which acknowledged by not having crashed yet.

## Slots

`saves/<slot>/` holds `world.ron` (name, seed, when it was made, when it was
last played, the pose and the selected slot) and `edits.log` (the transaction
log). The list is the directory, sorted by name, because a save list whose
order comes from a hash is a list that reorders itself between launches.

SAVES on the pause menu lists them with the newest played first: LOAD, DELETE,
and NEW WORLD. A delete asks twice, because it removes a directory and nothing
brings it back.

Starting the game with no `--world` opens the one played most recently, which
is what "logging back on" means; `--world <name>` picks one, and creates it if
it is not there.

## What this is NOT

**Every slot is a save of the SAME world.** The generator takes its seed from
the `TERRAIN` constant, so there is one world shape and the slots are saves of
it. The seed is WRITTEN into each slot's metadata and CHECKED on load, so the
day a seed is authored per world the old saves say which world they belong to
rather than silently loading into a different one. Loading a slot whose seed is
not the running world's is refused with the reason on screen.

Making that work means regenerating the planet on a load - 29 seconds at the
measured cost - which needs the world built on entering a state rather than at
startup. That is a real change and it is the next one, named in the design.

**No compaction.** The log grows by a line per edit for the life of a world. At
about a hundred bytes a line, a hundred thousand edits is ten megabytes, which
is not a problem yet and is a problem eventually. The snapshot is the place to
fold the log into, and the design says how.

**No cloud, no versioning, no backups.** One directory per slot, on disk.
