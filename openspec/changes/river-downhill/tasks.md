# Tasks

## 1. Measure
- [x] The water's top down the owner's river: 49 rises, 54 m, worst 6 m,
      all inside reaches (the bank clamp).

## 2. Version 11
- [x] `RiverSettings::downhill`, `RiverSettings::V11`; `TENEBRIS_V11`;
      `GENERATOR_VERSION` 11; `for_version(11)`.
- [x] The ceiling table, made with the map and on reading it.
- [x] `top_at`, `carve` and `far_surface` take the ceiling under `downhill`.
- [x] Settled climate for version 11 shipped (`settle_climate` levels 3 and 5, 2 h 8 min on the desktop).

## 3. Tests
- [x] `version_11_rivers_never_rise_downstream`.
- [x] `version_11_is_version_10_off_the_rivers`.
- [x] Versions 8 to 10's ground unchanged (their digests as `origin/main`'s; the pinned values fail on main too).

## 4. Show it
- [x] The instrument down the owner's river reads no rises; screenshots of
      one of the old rises, version 10 against 11
      (`docs/screenshots/river-downhill/`, 610 m down the owner's river from a
      tube looking downstream: version 10's water climbs in steps toward the
      camera, its falls' faces turned uphill; version 11's runs level, 2 m
      lower. Memory-only worlds: `--generator N --walk --at ... --aboard tube
      --capture`).
