# Proposal: the sun holds the planet at 15 °C

## Why

**The owner (2026-09-27): "so the sun should maintain average temperature of
the planet to 15 c".** In the survey: fix the terms and add the thermostat
("do recommendation I guess", K1), averaged over "the whole surface, poles and
stuff are still cooler" (K2), and landed "before map mockup" (K3).

The planet freezes. `fishing-and-equipment`'s design (section 7) measured it
with `examples/fish_ranges.rs`, and the logs are committed in
`docs/wiki/fish-ranges/`:
- A new world starts from the latitude climatology `28 − 45·sin²(lat)`, which
  averages about +13 °C.
- The mean sea surface is 7.7 °C on day 10, −2.8 °C on day 30 and −19.0 °C on
  day 100. It settles near −23.5 °C from day 130.
- In the second game year every point of water is below −1.8 °C for the whole
  year, so the spawn rule puts no fish anywhere.
- A game day is 48 minutes, so the sea's mean is below freezing after about
  24 hours of play. Since world time passes only in play (the owner,
  2026-09-27), that is 24 hours of anyone's play.

The same design names the two terms out of balance, from
`assets/config/atmosphere.ron`:
1. **The sun is too weak for the heat-loss law.** `solar_wm2` is 1000, a
   clear-sky surface value. The outgoing longwave `203 + 2.09 T` is Budyko's
   law, calibrated against a top-of-atmosphere sun of about 1360 W/m².
2. **Clouds cool about ten times as hard as Earth's.**
   - In the tropics at 0.9 cover, the cloud albedo (0.6 of the sunlight)
     takes about 230 W/m².
   - The cloud greenhouse returns a flat 40 W/m².
   - Net, that is about −190 W/m², against about −20 W/m² for Earth's clouds.

Fixing the sun alone was measured, and it is not enough: with 1360 W/m² the
sea is still at −13.7 °C on day 140, and still falling.

It matters beyond fishing:
- the climate map (`climate-and-fish-maps`) would bake a frozen planet;
- the map mockup's climate layer would show one;
- weather, sea ice and every temperature window in the species roster
  assume a temperate world.

## What Changes

- **The two broken terms are fixed first.** The sun becomes a
  top-of-atmosphere value that matches the heat-loss law. The cloud terms are
  retuned until the planet's mean net cloud effect is in Earth's range, from
  −10 to −30 W/m². Both are measured with the instrument, not argued.
- **A thermostat on the sun holds the average at 15 °C.** Each simulation
  step, the area-weighted mean surface temperature is computed over the whole
  grid, land and sea. A slow controller scales the sun's strength to hold that
  mean at `target_mean_c` (15.0), which is the owner's "the sun should
  maintain".
  - It moves the planet's average only.
  - Latitude, seasons, day and night, and the weather stay the simulation's
    own.
  - Once the two terms are fixed it only trims. It is also the guard that
    keeps a later retune from freezing or cooking the planet.
- **The thermostat's state is saved with the weather**, so a reload continues
  exactly. An old save's weather has none, and it starts at 1.0. A world
  already frozen warms back over the following days.
- **Tests pin it.**
  - The instrument's 200-day run keeps the mean within 15 ± 1 °C from day 30
    on.
  - Every species has water it can live in during the second game year.
  - The existing day-one fish coverage test still passes.
- **The video gate.** Two time-lapses of the sea temperature over 200 game
  days, before and after, drawn frame by frame from the instrument's fields.
  A plot of the mean against the day, for both. In the game, the temperature
  overlay from orbit on day 150 of both runs, and a fish caught in the second
  year.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `world/weather`: one added requirement, that the planet's average surface
  temperature is held at a configured target by the sun's strength.

## Impact

- **`pbd-core/src/atmosphere`:**
  - `settings.rs` gains `target_mean_c`, `sun_trim_s` (the controller's time
    constant) and the trim's limits;
  - `step.rs` scales the sunlight by the trim;
  - `mod.rs` computes the area-weighted mean and steps the trim, and saves it
    (the weather format gains one field, versioned).
- **`assets/config/atmosphere.ron`:** `solar_wm2`, `cloud_albedo`,
  `cloud_greenhouse` and the new fields, at the measured values.
- **`examples/fish_ranges.rs` and `examples/climate.rs`:** print the mean and
  the trim per ten days, so the owner's numbers come from the instrument.
- **Docs:** `docs/wiki/fish-ranges/` is regenerated from the balanced
  atmosphere, and `fishing-and-equipment`'s owner question 8 is answered.
- **Performance:** one area-weighted sum over 10,242 cells per step. That is
  negligible next to the step itself, and not measured in a cloud session
  (CLAUDE.md).
- **Order:** it lands before the map mockup, whose climate layer shows it.
  That placement is the recommendation in the owner survey (K3), and it moves
  if the owner answers otherwise.
