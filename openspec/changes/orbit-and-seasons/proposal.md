# Proposal: the planet orbits its sun, and has seasons

## Why

`daylight::SUN_FIXED` holds the sun at 23.45 deg N for ever: every day is the
northern summer solstice. With a simulated atmosphere
(`atmospheric-circulation`) that would park the tropical rain belt in the
north and keep the south in permanent winter. The owner: *"model the planet
orbiting the sun ... if we have 100 days in orbit, one rotation should be one
day right?"*

## The answer to "one rotation, one day"

Almost, and the difference is worth one line. The day the clock shows (noon
to noon, 48 minutes) is the **solar** day. Because the planet moves a little
way round its orbit each day, it has to turn slightly more than once to bring
the sun back overhead. Over a 100-day year it turns **101 times against the
stars** and 100 times against the sun. This change keeps the clock's day as the
solar day, so noon is always noon, and derives the extra turn. It shows as the
star field shifting by 3.6 deg a night over the year.

## What

- **A year is 100 days**, which is 80 hours of play. `YEAR_DAYS` sits beside
  `DAY_S` in `daylight`.
- **Obliquity 23.45 deg** (the existing `TILT`). The sun's latitude swings
  `asin(sin 23.45 deg * sin lambda)` as the orbital longitude `lambda` goes round:
  +23.45 deg at the northern solstice, 0 at the equinoxes, -23.45 deg at the
  southern solstice.
- **The world opens at the northern summer solstice at 09:00**, where the sun
  sits today. Day 0 is exactly the present sky, so every existing capture and
  test is unchanged.
- **The clock carries world time in `f64` seconds**, not a fraction of a day,
  as the frames rule asks of orbital time. The hour, the day and the day of the
  year are derived from it. It is saved in `world.ron`, so a world resumes in
  its season.
- **Still one rotation.** The sun, stars and moon still reach the planet through
  one sky rotation, now the sidereal one: the sun's hour angle for the solar time,
  plus its right ascension for the season.
- **`--day N`** for captures, beside `--time H`.

## Out of scope

- An eccentric orbit (the distance to the sun is constant).
- Other bodies' orbits.
- A calendar in the HUD. The overlay legend (`weather-overlays`) shows the day
  and season.
