## 1. Instrument

- [x] 1.1 `system-times` cargo feature: each system's main-thread time and
  slowest call by span, entity, mesh, material and image counts, meshes
  modified a frame (`SYSTEM_TIMES`, `SYSTEM_WORST` every 30 s).
- [x] 1.2 `TICK_PARTS` at debug level: a water tick's copy, rule, save and
  contact.

## 2. Reproduce

- [x] 2.1 Standing in a copy of World 19 (6 min), a straight walk (5 min),
  the river walk (3 min), pours running by the cascade (4 x 3-4 min): none
  grows.
- [x] 2.2 The digging river walk (2 km from the cascade): two windows of
  main and render at 44-56 ms, every system slower, the background work
  stacked up.

## 3. Fix

- [x] 3.1 The river memos roll two generations instead of freezing at their
  cap; tests `a_full_memo_keeps_the_newest_values`,
  `a_key_in_both_generations_counts_once`.
- [x] 3.2 Background threads below normal priority: tried, no gain in the
  A/B, taken out.
- [x] 3.3 The A/B: the digging walk, release, interleaved: no slowdown in
  either build (89-91 fps before, 80-87 after, noise).
- [ ] 3.4 The owner's next session with the instrument at release, to
  name the system that grows on a run that slows.

## 4. Not done here

- [ ] 4.1 A tick that moves water copies the whole fine set and re-uploads
  the whole tier (`cascade-perf`, "Next"): write only what changed.
- [ ] 4.2 ~7 ms a frame in Bevy's `allocate_and_free_meshes`, standing:
  ~17 meshes modified every frame (fish schools, critters, the herd).
