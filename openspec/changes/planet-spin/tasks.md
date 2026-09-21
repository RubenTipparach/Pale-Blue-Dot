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
- [ ] Captures at 6, 9, 12, 18 and 22 hours from one spot.

## 3. Held
- [ ] The rotating body frame for a ship in orbit.
- [ ] Moon phases; the sun's surface.
