# Proposal: an edit is a message, not a rebuild

## Why: the number I gave was wrong, and the shape is the reason

Asked whether the relight could move to a worker thread, I measured what an edit
actually costs instead of estimating it again. Six samples, `--walk --dig 8
--dig-ahead`, release, this container:

| | median | range |
| --- | ---: | ---: |
| Clone the fine set | **19.3 ms** | 6.3 - 21.1 |
| Repack the runs and reconcile the surface | **0.01 ms** | 0.01 |
| Re-bake the tier's light | 9.3 ms | 7.7 - 11.1 |
| Rebuild the walker's contact tier | 8.2 ms | 3.8 - 9.3 |
| **Total** | **36.9 ms** | 18.2 - 40.3 |

Three things follow, and the first is a correction.

**The relight was not the problem and I said it was.** I reported 6 ms and named
it the frame-budget risk. The startup bake is 8.2 ms on its own and in the edit
path it is 9.3, and it is the SMALLEST of the three costs around it - one of
which, the contact rebuild, I had not measured at all. An edit is 36.9 ms against
a 16.7 ms frame, so holding the dig button drops frames today rather than one
day.

**The work that is actually needed is 0.01 ms.** Repacking the edited column,
its neighbours' runs and the surface record - everything the GPU must be told
about - is 0.03% of the cost. The other 36.9 ms is two deep copies and a re-bake
of a region the edit could not have reached.

**And the copies are per EDIT of structures sized by the WORLD.**
`PlanetFine.set` is an `Arc<FineSet>`, so mutating it means cloning it: 159,475
cell records at 192 bytes, 30.6 MiB, to change one. `PlanetContact` then holds a
second strong reference, which is why `Arc::make_mut` would not help, and
`FineTier::new` clones the finest level again and rebuilds a `HashMap` grid. The
tier's light is re-baked over 3,105 columns of 320 layers to answer a question
about a few hundred cells. That is copy-on-write at the granularity of the whole
world for a write that touches one column, three times over, and it does not get
better as the tier grows.

There is a fourth cost nobody has measured: a version bump makes `upload_fine`
write every record, every column word and every light word - about 31 MiB to the
GPU per block. The design says what to do about it.

## What

**1. Stop copying the world to change a cell.** The edit path mutates in place
and tells the GPU about the records that changed, with `write_buffer` at their
own offsets. The render world reads through the same `Arc`; what it needs is a
version, which it already has. This is the whole of the 19.3 ms, and it is also
what makes an upload of only the changed slots possible.

**2. Rebuild the contact tier incrementally too.** It is the same fault one
level along: the grid, the neighbours and the records are all still right, and
one column's runs are not. It gets the same `reconcile`-shaped call the GPU
records get, rather than a fresh `FineTier` per block.

**3. Relight what an edit can reach, not the tier.** Light travels at most
`MAX` cells from what changed, so the region is a ball of 15 around the edit -
the bound the reference states as `sky_max^3`. Re-baking 3,105 columns to
answer a question about a few hundred cells is the same mistake as cloning the
set to change one record, one level down.

**4. Then, and only then, a worker thread if it is still worth it.** The
owner's question was whether the work can go off-frame and join. It can, and
the machinery is already here: `refresh_lod` spawns the tier rebuild on
`AsyncComputeTaskPool` and polls it with `poll_once`. But an edit is not a
tier rebuild - **it has to be visible on the frame the player clicked**, or
the block they broke is still there when they look at it, which is the one
thing a voxel game cannot get away with.

So the ordering matters: make the edit cheap first, and use a thread for what
is left over rather than to hide a cost that should not exist. The design says
which half goes where - the APPLY stays on the frame and the RE-BAKE of the
affected region can land a frame or two later, because a hole that is briefly
lit by yesterday's light is a picture nobody notices and a hole that is not
there is not.

## And the reason to do it now is multiplayer

**The authority is already the right thing, and that is worth saying before
anything is built on it.** A column is documented as "a pure function of its
direction, the worms and these edits", and `pbd_core::edits::Edits` is a
sparse override keyed by a STABLE cell ID that survives the tier being rebuilt
at a different anchor. The save is an append-only log of those edits with a
durable sequence.

That is, already, the shape a multiplayer world needs:

- **One authority**: the ordered edit log. Everything else - the columns, the
  light, the GPU records, the walker's contact - is derived and can be thrown
  away and rebuilt.
- **A stable name for a place**: the cell ID, which two clients agree about
  without sharing an anchor, a tier, or a camera.
- **An order**: the log's own, which is what makes two players editing the
  same cell resolvable rather than a race.

What it does NOT have is any notion of WHOSE edit, or of an edit that arrives
from somewhere else. The change that makes an edit cheap is the same change
that has to decide where an edit comes from, so this proposal settles that
question rather than leaving a second answer to be invented later:

**An edit is accepted, sequenced, and then applied** - and applying it is the
same code whether it came from this player's mouse or off a socket. The local
click is optimistic: it applies immediately with a provisional sequence, and
a server's ordering can supersede it. Two players digging the same cell is
then a last-writer-wins on one ordered log rather than two clients that have
diverged.

## What this is NOT

- **Not a network layer.** No sockets, no protocol, no server. What this does
  is make the local path a function of an ordered log, so a network layer has
  somewhere to deliver to. Building the socket first and the ordering later is
  how the ordering ends up in the socket.
- **Not a change to what an edit MEANS.** The durability rule stands: an
  accepted edit reaches the log before it is acknowledged.
- **Not the incremental light removal pass**, unless the measurement after
  step 3 says the ball of 15 is still too much. The reference needs a removal
  pass because it relights in place; re-baking a REGION from its seeds has
  nothing to remove, which is the property worth keeping for as long as it is
  affordable.
