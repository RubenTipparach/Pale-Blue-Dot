# Tasks

## 1. Measure

- [x] 1.1 `atmosphere::tests::weather_steps`: cover moved per publish of 1, 3
      and 30 steps; at 3,000 places over ten minutes, rain starts and stops,
      what each jumps by, spell and gap lengths, and the candidate followers
      (numbers in the proposal and design).

## 2. The rain seen

- [ ] 2.1 `rain_rise_s`, `rain_fall_s` in `AtmosphereSettings` and
      `atmosphere.ron`, validated positive; the shipped settled `.ron` files
      still match.
- [ ] 2.2 `Atmosphere::rain_seen`: set to its target by `new` and `restore`,
      followed after each step, interpolated by `sample`; not in `to_bytes`.
- [ ] 2.3 `weather::cloud_cell`, `rain_at`, `raining_cells`,
      `precipitation_map` read it; `raining_cells` returns each cell's rain.
- [ ] 2.4 Tests: a step moves it no more than the rise allows; a 20 s gap
      does not reach nothing; `to_bytes` is unchanged by it; a restored state
      starts at its target.

## 3. The shown pair

- [ ] 3.1 `Air::shown` and `blend_air`: pending, carry-over, span cap, snaps
      (first, capture, clock jump).
- [ ] 3.2 Tests: a step plays over a second with no jump between frames; a
      thirty-step publish plays over the cap; a state published mid-blend
      waits; a clock jump snaps.
- [ ] 3.3 `sample_field`, `fill_rain_map` and `cell_shafts` mix the pair;
      shafts thin with the rain seen.

## 4. The GPU mix

- [ ] 4.1 `from`/`to` textures, uploaded when the pair moves; the cubes take
      storage; `weather_blend.wgsl` and its pipeline; the dispatch each frame;
      the fallback write before it compiles.
- [ ] 4.2 GPU test: the pass's output against the CPU's mix on an adapter.

## 5. Show it

- [ ] 5.1 A chart of the rain and cover over one place through a minute,
      before and after, from the instrument; captures under a shower before
      and after. In `docs/screenshots/smooth-weather/`.
- [ ] 5.2 Video for the batch (CLAUDE.md: screenshots stand in for now). A
      cloud capture steps in place and so snaps by design; smoothness is
      judged in a real-time run by the owner.
