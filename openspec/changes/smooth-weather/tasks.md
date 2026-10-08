# Tasks

## 1. Measure

- [x] 1.1 `atmosphere::tests::weather_steps`: cover moved per publish of 1, 3
      and 30 steps; at 3,000 places over ten minutes, rain starts and stops,
      what each jumps by, spell and gap lengths, and the candidate followers
      (numbers in the proposal and design).

## 2. The rain seen

- [x] 2.1 `rain_rise_s`, `rain_fall_s` in `AtmosphereSettings` and
      `atmosphere.ron`, validated positive; the shipped settled `.ron` files
      still match.
- [x] 2.2 `Atmosphere::rain_seen`: set to its target by `new` and `restore`,
      followed after each step, interpolated by `sample`; not in `to_bytes`.
- [x] 2.3 `weather::cloud_cell`, `rain_at`, `raining_cells`,
      `precipitation_map` read it; `raining_cells` returns each cell's rain.
- [x] 2.4 Tests: a step moves it no more than the rise allows; a 20 s gap
      does not reach nothing; `to_bytes` is unchanged by it; a restored state
      starts at its target.

## 3. The shown pair

- [x] 3.1 `Air::shown` and `blend_air`: pending, carry-over, span cap, snaps
      (first, capture, clock jump).
- [x] 3.2 Tests: a step plays over a second with no jump between frames; a
      thirty-step publish plays over the cap; a state published mid-blend
      waits; a clock jump snaps.
- [x] 3.3 `sample_field`, `fill_rain_map` and `cell_shafts` mix the pair;
      shafts thin with the rain seen.

## 4. The GPU mix

- [x] 4.1 `from`/`to` textures, uploaded when the pair moves; the cubes take
      storage; `weather_blend.wgsl` and its pipeline; the dispatch each frame;
      the fallback write before it compiles.
- [x] 4.2 GPU test: the pass's output against the CPU's mix on an adapter.

## 5. Show it

- [x] 5.1 A chart of the rain shown at one place through three minutes,
      before and after, from `atmosphere::tests::rain_at_one_place` and
      `tools/rain_chart.py`, in `docs/screenshots/smooth-weather/`. No still
      capture under a shower: a capture steps in place and snaps by design
      (decision 2), and a still cannot show a change over time.
- [ ] 5.2 Video for the batch (CLAUDE.md: screenshots stand in for now). A
      cloud capture steps in place and so snaps by design; smoothness is
      judged in a real-time run by the owner.

The requirements stay in this change's delta until the owner's gate, as
`detail-fade`'s and `cloud-reach`'s do; each is pinned by a passing test:
`atmosphere::tests::the_rain_seen_builds_in_and_dies_away`,
`the_rain_seen_is_not_saved_and_starts_at_the_rain_as_it_stands`,
`pbd-app` `atmosphere::tests::a_step_plays_over_its_second`,
`a_held_step_plays_over_the_cap_and_a_new_one_waits`,
`a_clock_that_jumps_snaps_to_the_newest_state` and
`weather_gpu_tests::the_blend_pass_writes_the_mix_of_the_pair`.
