# Tasks

- [x] 1.1 Hands take logs to `axe_logs`; tests in `gather` (`hands_and_an_axe_take_a_tree_s_logs_alike`); the mission help reads "Punch a tree for logs".
- [x] 2.1 The worked tree and its last hit in the planet uniform; the near tree's damped shake in `planet_surface.wgsl`; the Rust/WGSL layout held by its test. Verified by `tree_shake::tests` (zero for any other tree, before the hit and once settled; a hit each half second held; the shader's constants and lines the Rust mirror's) and `actual_pipeline_layouts_use_static_offsets_and_correct_storage_access` and `the_splash_shader_carries_the_terrains_own_terms` (the new `shake` lane in all three Params). The cascades draw with a copy of the camera's params, so the shadow moves with the tree.
- [x] 2.2 Captures: a tree mid-shake against at rest, by hand and by axe, in `docs/screenshots/tree-punch-shake/`. Lavapipe, `fast`, 1440 x 900; three runs alike but for the hands, so the wind matches frame for frame: the difference images show only the worked tree moving, its foot still, its crown most.

## Playtest notes (owner's playtest, 2026-11-22; `docs/plans/2026-11-22-playtest-todo.md`)

- [ ] A punched log takes 4.8 s (`dig.hand_log_speed` 0.125): coded; built, tested, seen.
- [x] Dust and falling leaves on a hit and on the fell: built in the game without the mockup, at the owner's ask ("please get all fixes that I asked TONIGHT"); `tree_fx.rs`, `tree_fx.ron`, tests in `tree_fx::tests` and the config agreement test. Particles lit as the terrain, capped at 64. Not yet seen in play, and its frame cost not yet measured with `--frame-log`.
