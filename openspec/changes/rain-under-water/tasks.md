# Tasks

- [x] 1.1 Before shots under water at `--at 12.748 9.455` (5.4 m under), night and day, looking up 20, 45 and 70 degrees, the rain volume on and off.
- [x] 1.2 Find where the rain is drawn over the underwater frame (the pass order and the ray's start).
- [x] 2.1 `water.wgsl`: `rain_under_water()`; `rain_far` and `rain` draw nothing under water. `planet_water.rs`: `rain_needed` false under water.
- [x] 2.2 Tests: the rain volume is not drawn under the sea (shader and CPU gate); the whole of `water.wgsl` validates with naga, its imports spliced, with every entry point its pipelines name (closes `water-lit` 3.4).
- [x] 2.3 Tried and dropped: the rain seen through Snell's window (design 3).
- [x] 3.1 After shots, the same set, and the owner's beach above water in rain before and after; the README.
- [ ] 3.2 The owner judges the shots (the gate is the owner's).
- [ ] 3.3 Open, not this change: at the window's edge the sheet's rain rings tilt the normal across the window's smoothstep and print as dark rings over what is seen through the surface (a hull above it in the shots), with the rain volume off too; the moon and stars through the window are sharp. `--rain 1` by day at this place panics (`min > max, or either was NaN` in an f64 clamp). Every launch logs about 20 `LATE_PIPELINE`s, before this change as after.

## 4. Splashes under a lake or river (owner, 2026-11-26: "rain fx underwater need fixin")

- [x] 4.1 Design 6: the splashes' gate knows only the sea; the shower's knows inland water too.
- [x] 4.2 The splashes and the shower ask `audio::hooks::eye_under_water` (one rule).
- [ ] 4.3 In the game: under a lake and under the sea in rain, before and after, one combined image; whether the far streaks are task 3.3's rings.
