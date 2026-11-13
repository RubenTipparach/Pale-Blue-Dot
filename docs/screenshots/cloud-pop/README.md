# cloud-pop: a cloud tower grows without re-forming

The proof is the test `a_tower_grows_and_shrinks_without_re_forming`
(`pbd-core/src/cloud_puffs.rs`): it sweeps a site's cloud top from 0 to 1 in
2,000 steps and fails if any billboard moves more than 5 m, resizes more than
5 m or changes opacity by more than 2 % in one step. Before the fix it failed
at top 0.614, billboard 0 jumping 62 m: the whole tower re-formed in one
frame. After it, it passes.

`flight-before-after.jpg`: a look check that nothing else moved. Fixed-step
capture on the Mac (`--profile fast`), frame 700 of the scripted scenic
flight from a world made fresh for each shot, before (only the puff code
reverted) left, after right: at 27.709 5.334 (map 27.709S 5.334E), 455.1 m
elevation, 365.1 m above the ground, in cloud; `Weather here: cover 1.00,
rain 1.00`; 10:00 (`--time 10`). 0.18 % of the pixels differ by more than
8/255.

`pbd-app --world <new> --route scenic --time 10 --frames 700 --capture <png>`
