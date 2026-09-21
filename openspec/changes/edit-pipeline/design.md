# Design: one ordered log, one apply, and nothing copied

## What an edit does today, in order

`desktop::digging::apply_edit` is the whole path. A click that survives the
early returns (bedrock, no change, a second torch on one cell) does this:

| step | what it touches | measured |
| --- | --- | ---: |
| Move the hotbar | a speculative clone of nine slots | - |
| `save.accept(...)` | the edit log and the writer thread | - |
| `(*fine.set).clone()` | **every record in the fine set** | 6.3 - 21.1 ms |
| `set_layer` + `repack` + `reconcile` | the column, its six neighbours, one surface record | **0.01 ms** |
| `set.columns.relight()` | **every column in the tier** | 7.7 - 11.1 ms |
| `contact.set_fine(&set)` | **the finest level again, and a rebuilt grid** | 3.8 - 9.3 ms |
| `fine.version += 1` | the render world's gate | - |
| `upload_fine` next frame | **every record, every column word, every light word** | not measured |

Six samples, `--walk --dig 8 --dig-ahead`, release, this container. Medians:
clone 19.3 ms, relight 9.3 ms, contact 8.2 ms, total 36.9 ms; the range of
totals is 18.2 to 40.3 ms. A frame at 60 Hz is 16.7 ms.

**The startup log says why each of the three is the size it is.** The fine set
is 159,475 records at 192 bytes, so a clone is 30.6 MiB. The tier is 3,105
columns of 320 layers, so a full bake is about a million cells - the same 8.2 ms
the startup line reports for the bake it does once. And `FineTier::new` clones
`levels[3]` and `finest_neighbors` and rebuilds a `HashMap` grid, which is the
third copy of the same edit.

So the work an edit actually requires is 0.01 ms, and it is 0.03% of what an
edit costs. Everything else is a copy of, or a re-derivation of, state the edit
did not reach.

## The three costs, and why each one exists

**The clone exists because `PlanetFine.set` is an `Arc<FineSet>` with two
holders.** `Arc::make_mut` would not save it: `PlanetContact` keeps its own
strong reference through `FineTier`, so the count is never one and `make_mut`
clones exactly as the explicit clone does. The fix is therefore not a smarter
`Arc` call - it is to stop treating a write to one cell as a new version of the
world. `FineSet` becomes owned by the resource and mutated in place, and what
the contact holds becomes a borrow or its own index rather than a second copy.

**The relight exists because `relight()` is the only bake there is.** It is the
right function for building a tier and the wrong one for changing a cell.
`pbd_core::light::MAX` is 15 and every step costs at least 1, so nothing more
than 15 cells from a changed cell can change value: the region an edit can
reach is a ball of radius 15 around it, which on a 2.833 m tile is the edited
column and its first ring or two of neighbours, over 31 layers. Re-baking that
region from its own seeds keeps the property this project chose the full bake
for - a bake has nothing to remove, where an incremental propagation must undo
light that no longer has a source. The reference needs a removal pass because it
relights in place; a region re-baked from scratch does not.

The region has to be the CLOSURE of the edit, not the edit's own column: light
crosses column boundaries, so the ball is measured in metres and turned into a
set of columns, and the bake runs over those columns with everything outside
them read as it stands. `light::bake` already takes a region and treats
`OFF_REGION` as solid, which is the wrong boundary here - a region boundary that
reads solid would put a dark wall 15 cells from every hole. The bake needs a
third answer for "outside the region": read the light that is already there and
do not write it.

**The contact rebuild exists because `FineTier` is derived from the set and
nothing tells it what changed.** It is the same shape of problem one level
along: the grid, the neighbours and the records are all still correct; one
column's runs are not. What it needs is the same `reconcile`-shaped call the GPU
records get.

## And the upload is a fourth cost nobody has measured

`upload_fine` is gated on `fine.version`, and a version bump means it writes
every level's records, every column word and every light word: 30.6 MiB of cells
plus 0.14 MiB of column records plus 0.95 MiB of light, per block. It is on the
render thread and it does not show up in the numbers above, which is exactly why
it is worth naming before the apply is made cheap: making the CPU side 0.01 ms
while the GPU side still writes 31 MiB a block moves the cost rather than
removing it.

The shape of the fix is the one `upload_fine` already has for levels - an offset
and a slice. What is missing is a record of WHICH slots changed. The edit knows:
it is the edited record, its six neighbours, and the surface record. A dirty
list on `PlanetFine` beside the version, cleared by the upload, is the whole
mechanism, and it is the same list the incremental relight needs.

## The worker thread, which was the question

The owner asked whether the data could be updated on a worker thread and joined
before submitting. It can, and the machinery is already in the tree:
`refresh_lod` spawns `generate_fine` on `AsyncComputeTaskPool` and polls it with
`block_on(poll_once(task))`, so a task that lands a frame or two later is a
pattern this app already keeps.

**But an edit splits into two halves with different deadlines, and only one of
them can wait.**

| half | deadline |
| --- | --- |
| The APPLY: the log, the runs, the surface record, the contact | **this frame** |
| The RE-BAKE: the light in the ball of 15 | **a frame or two** |

The apply cannot wait because the block the player broke must be gone when they
look at it, and because their feet must not stand on a floor that is no longer
there. The re-bake can, because a hole lit for two frames by the light that
stood there before it is a picture nobody notices.

The apply is 0.01 ms once it is not a copy, so it does not want a thread. The
re-bake is the half that does, and it wants one only if the bounded version is
still expensive: a ball of 15 over a few dozen columns is a small fraction of a
million cells, and the honest order is to measure it before spawning anything.

A thread also brings a rule with it: **the apply that ran on the frame and the
re-bake that lands later must agree about the order of edits.** A second edit
while a re-bake is in flight must not be overwritten by a result computed
against the world before it. The sequence number the log already assigns is what
resolves that - a re-bake carries the sequence it was computed at and is dropped
if the log has moved past it - which is the same mechanism multiplayer needs,
one process early.

## Multiplayer: the authority is already right, and that is the point

Nothing here is a network layer. What this change settles is where an edit comes
FROM, because the alternative is that a socket invents a second answer later.

**Three things a shared world needs, and the tree has all three.**

- **One authority.** `pbd_core::edits::Edits` plus the save's append-only log.
  Every other representation - the columns, the light, the GPU records, the
  contact - is documented as derived and is thrown away whenever the tier's
  anchor moves. `edits.rs` states the invariant this rests on: a column is a
  pure function of its direction, the worm field and these edits, so two
  processes with the same log build the same world.
- **A stable name for a place.** The cell ID in `metadata[3]`, which survives a
  tier rebuilt at a different anchor. Two clients agree about a cell without
  sharing an anchor, a camera, or a tier.
- **An order.** The log's own, already durable, already sequenced. `Edits::set`
  replaces a second edit to one layer in place rather than appending, so the
  order is a function of the edits and not of when they were replayed.

**What is missing is only WHOSE and WHENCE.** `apply_edit` is a Bevy system that
reads a mouse. An edit that arrived from a socket has nowhere to go.

So the change is to split `apply_edit` in two at the seam that already exists in
it:

1. **Accept**: validate against the world (bedrock, no change, a second torch),
   take the item out of the hands, write the log, get a sequence. This is the
   half that can refuse, and it is the half a server owns.
2. **Apply**: given a sequenced edit, mutate the columns, the surface record,
   the contact, and mark the light dirty. This half cannot refuse and does not
   know where the edit came from.

A local click runs both. A socket runs the second. The local click stays
optimistic - it applies at once with a provisional sequence - and a server
ordering that disagrees supersedes it by replaying the log from the last agreed
sequence, which is a thing the tier rebuild already does from cold every time
the anchor moves. Two players digging one cell is then last-writer-wins on one
ordered log, which is a rule, rather than two clients that have diverged, which
is a bug with no rule in it.

**The durability rule does not move.** An accepted edit reaches the log before
it is acknowledged, and `SaveWriter` already answers that with a committed mark
rather than a queued one. A remote edit is accepted by whoever owns the log; a
client applying one has nothing to acknowledge.

## What is deliberately not decided here

- **The protocol.** No sockets, no wire format, no server. The seam above is
  what a protocol would deliver to, and building the seam first is what keeps
  the ordering out of the socket.
- **Interest management.** Which edits a client is told about is a question
  about a world bigger than one tier, and it needs the band's own answer first.
- **The incremental removal pass.** Re-baking a bounded region has nothing to
  remove. If the ball of 15 measures too expensive, the removal pass is the next
  step, and it is a change to `pbd_core::light` with tests of its own.
