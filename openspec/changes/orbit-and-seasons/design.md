# Design: orbit and seasons

## The clock

`daylight::Clock { seconds: f64 }` is world time since midnight of day 0.
- `fraction()`, the solar time of day in 0..1, is `(seconds / DAY_S) mod 1`.
- `day()` is its floor.
- `year_fraction()` is `(seconds / DAY_S / YEAR_DAYS) mod 1`.
- `at_hour(h)` is `h / 24 * DAY_S`.
- `advance(s)` adds `s` and ignores anything non-finite.

Every reader of `clock.fraction` becomes `clock.fraction()`. At `f64`, a
microsecond stays resolvable after a thousand years of play.

## The sun

The ecliptic longitude is `lambda = TAU * year_fraction + lambda0`, where
`lambda0 = pi/2`, so day 0 is the northern solstice. With the pole on +Y and the
ecliptic tilted about X by `TILT`, the sun's direction in the system frame is:

```text
s = (cos lambda, sin TILT * sin lambda, cos TILT * sin lambda)
```

Its declination is `asin(s.y)`, and its right ascension (azimuth about Y) is
`alpha = atan2(s.z, s.x)`.

## One rotation

`sky_from_system` is the rotation about Y that carries the sun's azimuth
`alpha` to the noon azimuth `phi0` (that of `SUN_FIXED`, 28.3 deg), minus the
hour angle `(fraction - 0.5) * TAU`. Then `sun() = sky_from_system() * s`, as
today, and the stars and the moon go through the same rotation. At day 0:
- `lambda = pi/2`, so `s = (0, 0.398, 0.917)` and `alpha = 90 deg`;
- noon puts the sun at `(0.807, 0.398, 0.435)`, which is `SUN_FIXED`.

So the present sky is day 0. Over a year `alpha` advances one turn, which is
the 101st rotation against the stars.

## Tests

- Day 0 at every hour gives today's sun (the rotation is unchanged).
- Declination is +23.45 deg at day 0, -23.45 deg at day 50, and 0 at days 25
  and 75, to 1e-3.
- Noon is the sun's highest point on the noon meridian, every day of the year.
- A solar day brings the sun back to within 1e-3 of where it was. A year of
  solar days turns the stars 101 times: one year from day 0, the star rotation
  equals day 0's.
- The world file round-trips `world_seconds`. A file without it opens at day 0.
