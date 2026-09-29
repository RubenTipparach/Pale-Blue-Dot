# Proposal

## Why

The owner, 2026-09-29, on the map mockup's Jet overlay after
`tropical-upper-wind` was built: "why are jets still forming this band here?
seems un natural" (`docs/screenshots/tropical-weather-aloft/owner-mockup.webp`).

The calm band is gone, but a band is still there. It is flat blue between
about 10 degrees north and south, and a thin dark line runs along each edge.
Read back from the mockup's own frames, the lines sit at 11.9 degrees north and
11.8 degrees south. All the way round the planet, each stays within about a
degree of that latitude. Inside the band the speed is 8.5 m/s everywhere,
varying by 0.4 m/s round the planet (design, "Measured").

It looks drawn on because it is. `tropical-upper-wind` gave the tropics a
constant 8 m/s easterly aloft, faded out by latitude. The jet it meets is
held at its 45 m/s cap over most of the tropics, so after the latitude fade it
also depends on latitude alone. Two things that depend only on latitude
cancel on a line of latitude. Underneath both, the model's own tropics hardly
vary round the planet:
- the surface wind is under 1 m/s within 20 degrees;
- the warmest air lies at 11-16 degrees north in almost every sector.

On Earth, the tropics aloft are driven by their storms. Air that rises in
them spreads out at the top and circles the high it builds there. The line
where the easterlies meet the westerlies bends with the continents and the
seasons. The summer monsoon's high pushes the easterlies far poleward over
Asia, while over the eastern oceans westerlies reach close to the equator.

## What Changes

Proposed, not built. The owner chooses on survey W2; the candidates are drawn
side by side in `docs/screenshots/tropical-weather-aloft/options.jpg`.

- **D. The jet follows the air.** Fade the jet in before the cap, not after.
  Balance it against the spin as at 30 degrees everywhere equatorward of 30
  (the floor on `f` moves from 14.5 to 30 degrees).
  - The jet at 5-20 degrees stops being pinned at its cap: 0-14% of cells
    against 49-58% now.
  - The subtropical jet breaks into streaks, and the straight bright edge
    along 22 degrees north goes.
  - Nothing poleward of 30 degrees changes.
  - The band inside stays flat.
- **H. D, and the tropics aloft follow their storms (recommended).** Where
  the fade leaves the tropics their own wind, add two terms, both worked out
  from the model's own rising air (`lift`):
  - the outflow: air rising in the storms spreads out aloft, 5 m/s rms;
  - the wind round the high the storms build aloft, anticyclonic, 8 m/s rms.

  The steady easterly drops from 8 to 6 m/s. The band's speed now varies by
  6.3 m/s round the planet, against 0.4. The dark line runs 37% of the way
  round in the north (was 49%) and 54% in the south (was 85%). It still runs
  straight where the tropics are quiet.
- **Not in these candidates:** the tropics' surface weather. It is the reason
  the line survives over the quiet half. The owner, 2026-09-29, on survey W3
  (write it up next, after W2 is built): "recommended". It is written up as
  its own change; nothing is built from it until the owner chooses.

## Impact

- `pbd-core` `atmosphere::step::aloft`: the order of the cap and the fade, the
  floor on `f`, and for H the two storm terms. H adds a relaxation for the
  outflow's potential, warm-started each step.
- `assets/config/atmosphere.ron`: for H, the outflow and circling speeds, m/s,
  validated; the easterly's default moves to 6 m/s.
- The settled climates are made again: level 3 here, level 5 on the owner's
  desktop. That is the run `tropical-upper-wind` task 3.1b already waits on,
  so one desktop run covers both.
- `openspec/specs/world/weather`: the requirement on the wind at cloud height
  gains a scenario that the tropics aloft vary round the planet. The edge
  limit may move from 3.5 to 4 m/s a degree (design, "Risks").
- Saved worlds: nothing saved changes. The wind at cloud height and the
  outflow's potential are worked out afresh. A world opens as it was saved.
- Frame cost: not measured in a cloud session. The owner runs
  `tools/perf_suite.py`.
