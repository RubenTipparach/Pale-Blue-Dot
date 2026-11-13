# Tasks

## 1. Measure

- [x] 1.1 Count landings and `LOD_FADE partial` on the scenic flight (twice)
      and the 1 km walk; pair each failing landing with its request (design
      section 1)
- [ ] 1.2 The landing-frames instrument (`PBD_LANDING_SHOTS`, design
      section 5); frames round a failing landing of the scenic flight, before

## 2. The old set stays for its dissolve

- [ ] 2.1 Two halves of fine regions in the cell buffer; `half`,
      `counts_prev`; a moved landing writes the other half, an unmoved version
      the current one; `lod_offsets.z` and `lod_counts_prev` in the uniform
- [ ] 2.2 The visibility pass: base / now / before slots, the twin that
      stands aside, trees by the same rows; `old_held`, `whole` and the
      records' rings retire
- [ ] 2.3 The surface pass: the dissolve without `old_held`; an OLD
      instance drawn with no column
- [ ] 2.4 The hold in `LodRefresh` (`LOD_FADE held`); `in_flight_s` counts a
      held set; an unmoved version keeps the dissolve; every landing dissolves
      but startup and a generator switch; the dissolve on the planet clock
- [ ] 2.5 `fade_covered`, `records_cosines` and their tests retire

## 3. Tests

- [ ] 3.1 GPU, real fine sets: the dissolve's old half lists exactly the cells
      the old partition lists alone, and its new half exactly the new
      partition's, for a walk step, a fast descent past the margin, the climb
      that drops level 11 and the descent that brings it back
- [ ] 3.2 GPU, the listing rows: a twin drawn alike is listed once, from the
      current half; drawn differently, once per half; the old half is idle
      outside a dissolve
- [ ] 3.3 A cell laid in two sets round different anchors is the same floats
      in both
- [ ] 3.4 The hold: a set landing inside a dissolve waits for it, and one
      landing after it does not

## 4. Evidence

- [ ] 4.1 The same landing after the change, dissolving; crops and
      difference images in `docs/screenshots/two-set-dissolve/` with a README
- [ ] 4.2 Landing, `LOD_FADE` and `LOD_FADE held` counts of the scenic flight
      and the walk after the change
- [ ] 4.3 The perf suite, old exe against new (owner, on the desktop)
- [ ] 4.4 A recording for the owner's batch of step videos

## 5. Follow-ups, not this change

- [ ] 5.1 The column tier dissolves at a landing (keep the old tier as the
      old records are kept)
- [ ] 5.2 The sea sheet dissolves at a landing (the old half's water cells,
      OLD, in `water.wgsl`)
- [ ] 5.3 Retire `cover_ring` and the replaced partition's floors, measured
