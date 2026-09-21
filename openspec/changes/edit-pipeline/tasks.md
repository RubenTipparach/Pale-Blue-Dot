# Tasks

Nothing here is built. The measurement instrument in `apply_edit` is, and it is
the only code this change has produced so far.

## 0. Measure, which is done
- [x] Time the four phases of `apply_edit` per block and report them as one
      `debug!` line: clone, repack+reconcile, relight, contact.
- [x] Six samples off `--walk --dig 8 --dig-ahead`, medians and range in the
      proposal, against the 16.7 ms a 60 Hz frame has.
- [ ] Measure the upload too, which is the one cost still quoted from a size
      rather than a clock: `upload_fine` writes about 31 MiB per version bump.

## 1. Stop copying the world to change a cell
- [ ] `PlanetFine` owns its `FineSet` rather than sharing an `Arc` of it, and
      `apply_edit` mutates it in place.
- [ ] `PlanetContact` stops holding a second strong reference: either it borrows
      the set for the frame it answers on, or it keeps only the index it needs.
      Whichever it is, `FineTier::new` stops being on the edit path.
- [ ] A test that an edit does not allocate a copy of the set - the honest form
      is a measurement, so the shape is a bound on the phase, not a mock.

## 2. Tell the GPU only what changed
- [ ] `PlanetFine` carries a dirty list of finest-level slots beside its
      version: the edited record, its six neighbours, the surface record.
- [ ] `upload_fine` writes those slots at their own offsets, and the column and
      light words for the same slots, rather than every record in the set.
      Cleared by the upload that consumed it.
- [ ] The full-rewrite path stays for a new tier, which is what the version is
      still for. One function, two callers, not two paths.

## 3. Rebuild the contact tier incrementally
- [ ] The same `reconcile`-shaped call the GPU records get: one column's runs
      change, the grid and the neighbours do not.

## 4. Relight only what an edit can reach
- [ ] `pbd_core::light` gains a bounded re-bake: the region is the ball of
      `MAX` (15) cells around the changed cell, and cells outside it are READ
      and not written. `OFF_REGION` reading as solid is the right boundary for a
      tier and the wrong one here - a region edge that reads solid would put a
      dark wall 15 cells from every hole.
- [ ] Tests: a bounded re-bake agrees cell for cell with a full bake of the same
      tier, for a dig, a place, a torch placed and a torch taken away. That
      equality is the whole guarantee; without it the bound is a guess.
- [ ] Gate it on whether the edit changed opacity or emission at all, which the
      reference does with `covers_face_raw(old) != covers_face(new)`. Today's
      relight is unconditional.
- [ ] Measure the bounded pass against the 9.3 ms full one, same instrument.

## 5. Accept, sequence, apply - the seam multiplayer needs
- [ ] Split `apply_edit`: an ACCEPT that validates, spends the item and writes
      the log for a sequence, and an APPLY that takes a sequenced edit and
      mutates the derived state. The apply must not know where the edit came
      from.
- [ ] The local click runs both, optimistically, with its own sequence.
- [ ] A test that applying a sequence of edits in order, from cold, gives the
      same tier as having applied them one at a time - which is the property a
      shared world rests on and the one a replay after a supersede needs.
- [ ] Nothing about a socket. The seam is the deliverable.

## 6. Only then, a thread if it is still worth it
- [ ] If the bounded re-bake still costs more than a frame can spare, move that
      half to `AsyncComputeTaskPool` on the pattern `refresh_lod` already keeps,
      and carry the sequence it was computed at so a result that is behind the
      log is dropped rather than applied.
- [ ] Measure first. A thread that hides 0.2 ms is a thread that costs more than
      it saves.

## 7. Held, with the reasons in the design
- [ ] The incremental removal pass, if the bounded re-bake measures too dear.
- [ ] Interest management, which needs the band's answer before the tier's.
- [ ] The protocol.
