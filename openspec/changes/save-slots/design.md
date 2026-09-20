# Design: a log, a snapshot, and one thread that owns the disk

## The shape

```text
saves/
  preview/
    world.ron     name, seed, made, last played, pose, selected slot
    edits.log     seq cell layer material  s0 s1 .. s9
  caves/
    ...
```

A write-ahead log and a snapshot, which is the oldest answer there is to
"durable on every change, without rewriting the world on every change". The
log is append-only and bounded by the edit; the snapshot is a whole file and
bounded by the timer.

**A torn write costs the last line and nothing else.** The reader already
tolerates a damaged line (`parse_line` returns `None` and the load counts it),
and that property is why the log is lines rather than a serialised structure:
a `ron` file with its last byte missing is a file with nothing in it.

## The writer is a thread, and the game never touches a file

```rust
pub struct SaveWriter {
    jobs: Sender<Job>,
    committed: Arc<AtomicU64>,   // the highest sequence the disk has taken
    failed: Arc<Mutex<Option<String>>>,
    queued: u64,
}
```

`WorldEdits::accept` becomes: apply the edit in memory, bump `queued`, send the
line, return. Nothing on the frame opens, writes, flushes or syncs.

The thread's loop is the interesting half:

```text
block on the first job
drain everything else already waiting
write them all
ONE fsync
publish the highest sequence as committed
```

Draining is what makes this faster than the synchronous version rather than
merely smoother: holding a mouse button down through a wall of dirt used to be
an fsync per cell, and is now an fsync per batch. It is also why the sequence
number is a high-water mark rather than a set - the jobs are written in the
order they were sent, so everything below the mark is down.

**Acknowledgement is the mark, never the send.** `SaveWriter::pending()` is
`queued - committed`, and the saves screen shows it. A write that failed puts
its reason in `failed` and the screen says that instead, because a save that is
silently not happening is the worst state this feature can be in.

**Shutdown drains.** `Drop` sends a final barrier and joins the thread. The
window closing, QUIT, and `AppExit` all run through it, so the last edit before
a quit is on disk before the process ends.

## Why a thread and not `AsyncComputeTaskPool`

The pool is for work that finishes. This work is a single long-lived owner of
two file handles, and the thing that makes it correct is that it is the ONLY
writer: one thread, one queue, one order. A task per write would have as many
writers as there are tasks, and the ordering of two appends to one file would
be whatever the pool decided. The log's order is the world's history.

## Loading

1. Read `world.ron`. Refuse if its seed is not the running world's, with the
   reason on screen: a save is of a world, and loading it into a different one
   would be a save that silently became somebody else's.
2. Replay `edits.log` into `Edits`, taking the hotbar from the LAST line that
   carried one.
3. Put the walker at the saved position with the saved heading and pitch, and
   select the saved slot.
4. Rebuild the column tier at the new anchor and the fine set around it, which
   is the same path walking there would have taken.

Step 4 is why loading a slot is cheap while a different SEED is not: the tier
is a sub-band around the player and is rebuilt whenever they move far enough
anyway, and the planet's own 160,000-column fine set is seed-shaped and takes
29 seconds. A per-world seed therefore needs the world built on entering a
state rather than in `Startup`, which is this project's next architectural
step and the reference project's own `AppState` with its `Boot` frame.

## The pose is written by a timer, and on the way out

`AUTOSAVE_S` is 5, which is Tenebris's figure for the same job. It also fires
when the menu opens, because the player who opens the menu is usually the
player about to quit.

What it holds is the position, the heading and pitch, and the selected slot -
not the hotbar, which rides the log, and not the world, which IS the log.

## Slots in the menu

`Screen::Saves`, off the pause menu. A row per slot, newest played first, with
LOAD and DELETE; NEW WORLD at the end.

**The list is rebuilt rather than updated.** A slot list changes shape when a
world is made or deleted, so the panel's children are despawned and respawned
whenever the index changes. It is a dozen rows and it happens on a press.

**DELETE asks twice.** The first press turns the row into DELETE <name>?
CONFIRM / CANCEL, and only the second removes the directory. A control that
destroys a world on one click is a control that will destroy a world.

## Held

- **Compaction.** When a log passes some size, the load could write a snapshot
  of the whole `Edits` and truncate. The snapshot format for that is the log
  itself with one line per edited layer, so nothing new is needed but the
  trigger and the rename.
- **A seed per world**, and with it the state machine that lets a world be
  built on entering `Playing` rather than in `Startup`.
- **Flight pose.** The snapshot holds the walker's. A player who quits in a
  ship comes back on foot where the ship was.

## Measured against the reference, after the fact

The owner asked for `tenebris-rs`'s own save system to be read. It was, after
this was built, and the shapes agree closely enough that the differences are
worth writing down rather than the similarities.

**The async writer is the same design, arrived at twice.**
`client_kv.rs:329` is a `OnceLock<mpsc::Sender<()>>`, a named `kv-writer`
thread, `try_recv` draining to coalesce every signal queued while the last
write ran, and two `AtomicU64` generation counters (`WRITE_REQ` / `WRITE_DONE`)
where `DONE >= REQ` means the disk has everything. That is this module's
queue, batch and high-water mark under other names. Its own comment
(`client_kv.rs:310`) gives the same reason: the synchronous rewrite "became a
main-thread lag spike (every block edit, plus a multi-write burst on each 5 s
autosave)".

**One thing was missing here and is ported: a CAP on the drain.** Its
`flush_blocking` (`client_kv.rs:375`) waits two seconds and then prints
"flush timed out - last write may be stale" rather than waiting for ever,
"so a wedged filesystem can't hang shutdown". `DRAIN_LIMIT` is that, for that
reason. A quit that will not finish is a worse failure than a save that is a
second stale, and the line is what turns a hang into a report.

**The 5 s pose autosave is its figure, checked.** `maybe_autosave_sp`
(`app.rs:5164`) fires on a literal `5.0`, and the same function refuses to run
at all while any `TENEBRIS_DEV_*CAM` variable is set, because a capture hook
that pins a synthetic pose every frame would write "a poisoned save". This
change has that rule as: a capture with no `--world` writes to no world.

**Where this deliberately differs, and why.**

- **It has no save slots at all.** A world there is a NAME, and every world's
  data is interleaved in one flat `kv.txt` under keys like
  `tenebris-c:tile:<name>:<body>:<tile>`; `MAX_WORLDS = 32` caps the picker
  rather than the storage. Slots were asked for here, so a world is a
  directory, and that also makes deleting one a `remove_dir_all` rather than a
  prefix scan over every key in the store (`persistence.rs:1063`, which has to
  enumerate six key families and still misses the `craft:` one).
- **The hotbar rides the edit log here and the 5 s timer there.** Theirs is
  saved beside the pose, and the cost of that is visible in the code: there is
  a `verify_inventory_saved` (`app.rs:5246`) that re-reads the hotbar
  immediately after writing it and prints every slot that failed to round
  trip, with the comment that items "would be LOST on reload". A record that
  carries the hotbar with the edit that changed it needs no verifier, because
  there is no second write to disagree with.
- **This fsyncs and the reference client does not.** Its `write_out`
  (`client_kv.rs:296`) is tmp + rename with no `sync_data`; its SERVER
  (`saves.rs:327`) does tmp + fsync + rename + parent-dir fsync inline on the
  tick thread, and its comment accepts the cost because "blocks are placed at
  human speed". Off the frame, the sync is free to the player, so it is kept.
- **Delete is its dialog, ported.** `menu.rs:565` replaces the world list with
  "Delete world 'X'?" over "All its blocks, bag and progress are erased
  forever", and two buttons. A row that says "delete?" beside nine other rows
  is a question a player answers without reading it; the dialog is the one
  that states what is lost. `NAME_MAX = 31` is its cap too
  (`persistence.rs:82`).

**One thing it does that this does not, recorded as a gap.** Its world
metadata has no timestamps at all - the picker shows names only - while this
shows when each was last played. That is not a difference to reconcile; it is
this project having the field because a directory made it free.
