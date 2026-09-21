# Tasks

## 1. The spin, in the core
- [ ] `daylight::Spin::at(clock)`, a quaternion about the tilted pole; a test
      that `Clock::sun()` equals the spin's inverse applied to the fixed sun.
- [ ] `DAY_S` 2880.

## 2. The sky, in the app
- [ ] The star mesh rotates by the spin's inverse each frame.
- [ ] The moon's orbit position is rotated by the spin's inverse.
- [ ] A sun disc in the sky shell along the sun direction, limb and glow,
      hidden by the planet's shadow and the clouds.
- [ ] Captures at 6, 9, 12, 18 and 22 hours from one spot.

## 3. Held
- [ ] The rotating body frame for a ship in orbit.
- [ ] Moon phases; the sun's surface.
