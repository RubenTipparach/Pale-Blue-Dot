# Tasks

## 1. The core can be restored into
- [x] `Slots::set(index, Option<Stack>)`, which is what a load needs and what
      `give` cannot express: a stack of exactly this, here.
- [x] Test: set, read back, and a count over the item's limit refused.

## 2. The writer
- [x] `saves::SaveWriter`: a thread owning the files, a queue, a committed
      sequence and a last error.
- [x] Batch: drain everything waiting, write, ONE fsync, publish the mark.
- [x] `Drop` drains and joins, so a quit does not lose the last edit.
- [x] Tests: a record read back after its sequence is committed, a batch of
      many committed together, and a failed write reported rather than
      swallowed.

## 3. The format
- [x] The log line carries the edit AND the hotbar after it, and the reader
      still accepts the three-field line that came before.
- [x] `world.ron`: name, seed, made, last played, pose, selected slot.
- [x] Tests: a round trip through both, and a damaged line costing that line
      and no other.

## 4. Slots
- [x] `saves::Index`: list (sorted, newest played first), create, delete, and
      the one played most recently.
- [x] `--world <name>` picks one at startup and creates it if it is absent;
      absent, the most recently played opens.
- [x] Tests: create then list, delete then list, and a name that cannot become
      a directory refused.

## 5. Loading
- [x] Refuse a slot whose seed is not the running world's, with the reason.
- [x] Restore the walker's position, heading and pitch, the hotbar and the
      selected slot, and rebuild the tier at the new anchor.
- [x] Autosave on a 5 s timer and when the menu opens.

## 6. The screen
- [x] `Screen::Saves` off the pause menu: a row per slot with LOAD and DELETE,
      NEW WORLD at the end, and what the writer is doing.
- [x] DELETE asks twice.
- [x] The list is rebuilt when the index changes.

## 7. Prove it
- [x] `--menu saves` for the capture, and a headless run that digs, autosaves,
      relaunches and comes back to the same place with the same hotbar and the
      same holes.
- [x] Measure: the frame cost of an edit before and after the writer moved off
      it, and the fsyncs a burst of digs costs.
- [x] The suites, and `openspec validate --all`.
- [ ] The owner's in-game confirmation.

## 8. Held, with the reasons in the design
- [ ] Compaction.
- [ ] A seed per world, and the state machine that lets one be built on a load.
- [ ] The flight pose.
