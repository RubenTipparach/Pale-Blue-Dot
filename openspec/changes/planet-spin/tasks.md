# Tasks

## 1. The spin, in the core
- [x] `Clock::spin()`, a quaternion about the pole, and `sky_from_system()`
      its inverse; `Clock::sun()` is that applied to `SUN_FIXED`, and
      `the_sky_turns_as_one_about_the_pole` holds the sun, a star and the
      moon's orbit to one rotation.
- [x] `DAY_S` 2880.

## 2. The sky, in the app
- [x] The star mesh rotates by the spin's inverse each frame (`turn_stars`).
- [x] The moon's orbit position is rotated by the spin's inverse (`move_moon`).
- [x] A sun disc in the sky shell along the sun direction, limb and glow,
      hidden by the planet's shadow and the clouds.
- [x] Captures from one spot, aimed off the launch log: the sun at 8 on the
      sunrise aim and at 16, 18 and 22 on the sunset aim
      (`docs/screenshots/sky-sun-walk.png`), the stars at 22, 0 and 2 on the
      sunset aim (`docs/screenshots/sky-stars-turn.png`: 52, 56 and 73
      stars detected, and only one or two of each frame's sit within four
      pixels of one from two hours before), the moon on the
      horizon at 23 from the coast (`docs/screenshots/sky-23-moon.png`).
      The listed hours 6, 9 and 12 stand behind or under the horizon from
      that aim; the log says where each is.

## 3. Held
- [ ] The rotating body frame for a ship in orbit.
- [ ] Moon phases; the sun's surface.
