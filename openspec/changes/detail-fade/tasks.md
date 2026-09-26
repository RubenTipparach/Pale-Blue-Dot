# Tasks

- [x] 1. Trees: `tree_fade_m` in `scatter.ron`, the dithered fade in the fragment, toward the edge of the trees' band
- [x] 2. Stage two coverage check at each landing (`fade_covered`, logged as `LOD_FADE skipped`)
- [x] 3. Stage two: the landing cross-fade (visibility, vertex, fragment), `lod_fade_s`
- [x] 4. Stills, perf suite old against new, a recording

## The owner's review: it still cuts (design section 3)

- [x] 5. Lay each level's ring to also cover the partition it replaces
      (`lay_band` takes the replaced partition), so `fade_covered` passes by
      construction unless capacity truncated the ring
- [x] 6. Test: a set laid while replacing one whose anchor is well past the
      margin, and one laid after the bands resized, both pass `fade_covered`
- [x] 6b. A tree both partitions draw eases its fade from the old
      partition's to the new one's by the cross-fade's progress
- [x] 7. Re-count `LOD_FADE skipped` on the flights (cloud-hop, scenic clouds,
      storm, far-side) against the counts in design section 3: cloud-hop 15 of 31 skipped before, 6 of 28 after; scenic clouds 10 of 30, 5 of 29. Every remaining skip but one is capacity (level 11 records end at 363-394 m, level 10 at 732-774 m, where the old band reaches up to 196 m further); one is a level no longer laid
- [ ] 8. Re-record for the owner, and judge whether 0.6 s reads as a fade

- [ ] 9. Fade every landing; the ring the records lack is drawn whole from the new partition (design section 4): `records_in`/`records_out`, the visibility rule, the log, the tests
- [ ] 10. The column tier and the water sheet still switch at a landing (design section 3): not in this change
