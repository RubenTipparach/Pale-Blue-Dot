# Tasks

## 1. Measure

- [x] 1.1 Count landings and `LOD_FADE partial` on the scenic flight (twice)
      and the 1 km walk; pair each failing landing with its request (design
      section 1)
- [x] 1.2 The landing-frames instrument (`PBD_LANDING_SHOTS`, design
      section 5); frames round the far-side flight's landings before the
      change (the set that empties the fine levels at 2.5 km)

## 2. The old set stays for its dissolve

- [x] 2.1 Two halves of fine regions in the cell buffer; `half`,
      `counts_prev`; a moved landing writes the other half, an unmoved version
      the current one; `lod_offsets.z` and `lod_counts_prev` in the uniform
- [x] 2.2 The visibility pass: base / now / before slots, the twin that
      stands aside, trees by the same rows; `old_held`, `whole` and the
      records' rings retire
- [x] 2.3 The surface pass: the dissolve without `old_held`; an OLD
      instance drawn with no column
- [x] 2.4 The hold in `LodRefresh` (`LOD_FADE held`); `in_flight_s` counts a
      held set; an unmoved version keeps the dissolve (and its old centre:
      the view keys it to the landing, not the version); every landing
      dissolves but startup and a generator switch; the dissolve on the
      planet clock
- [x] 2.5 `records_cosines` and its test retire; `fade_covered` becomes the
      cover's test-only checker
- [x] 2.6 The sea sheet: listed by the ground's rows, two entries a slot;
      `water.wgsl` splits a marked sheet by its own partition and keeps its
      half of the pixels by the terrain's mask (its own commit)

## 3. Tests

- [x] 3.1 GPU, real fine sets: the dissolve's old half lists exactly the cells
      the old partition lists alone, and its new half exactly the new
      partition's, ground and trees, for a walk step, a fast descent past the
      margin, the climb that drops level 11, the descent that brings it back,
      and the climb and descent through the coarsest band
      (`a_dissolve_draws_both_partitions_whole_on_real_fine_sets`)
- [x] 3.2 GPU, the listing rows: a twin drawn alike is listed once, from the
      current half; drawn differently, once per half; the old half is idle
      outside a dissolve (`actual_gpu_cross_fade_draws_each_partition_from_its_own_set`)
- [x] 3.3 A cell laid in two sets round different anchors is the same floats
      in both (`a_cell_is_the_same_floats_in_every_set_that_lays_it`)
- [x] 3.4 The hold: a set landing inside a dissolve waits for it, and one
      landing after it does not (`a_set_landing_inside_a_dissolve_waits_for_it_to_end`);
      the upload rules and the half layout (`a_landing_dissolves_from_the_half_it_leaves`)
- [x] 3.5 The partition rule is the same text in both passes
      (`the_partition_rule_is_the_same_in_both_passes`, replacing the
      `old_held` copies test)

## 4. Evidence

- [x] 4.1 The same landing after the change, dissolving; crops and
      difference images in `docs/screenshots/two-set-dissolve/` with a README
- [x] 4.2 Landing, `LOD_FADE` and `LOD_FADE held` counts of the scenic
      flight, the walk and the far-side flight after the change (design,
      "What building it found")
- [ ] 4.3 The perf suite, old exe against new (owner, on the desktop)
- [ ] 4.4 A recording for the owner's batch of step videos

## 5. Follow-ups, not this change

- [ ] 5.1 The column tier dissolves at a landing (keep the old tier as the
      old records are kept): today a cell entering or leaving the tier has its
      record's surface rewritten from its column, and its cap, its voxel light
      and (rarely) its tree change in one frame
- [ ] 5.2 Retire `cover_ring` and the replaced partition's floors, measured
