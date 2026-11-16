# rain-matches-map: before and after

The owner, 2026-11-16: "there shouldnt be rain here, theres no rain on the
map". The change is `openspec/changes/rain-matches-map`.

Every shot is the same place and moment: a new world, on foot at frame
latitude 26.262, longitude -14.111 (map 26.262S 14.111W, between
Hollingmouth and Durnacrag, 0.3 degrees from the owner's place), at 22:00,
with the weather stepped 475 s (`--weather-at 475`). The instrument
`atmosphere::tests::rain_gap_near_owner` found that moment: the rain seen
there is 0.50 while the rain rate is 0 mm/h, between two bursts. The log
says `Weather here: cover 1.00, rain 0.50` in every run, before and after.

```
pbd-app --world <new> --walk --at 26.262 -14.111 --time 22 --weather-at 475 --capture world.png
  ... --menu map --map-clouds --map-mpp 60      (map-clouds)
  ... --menu map --map-overlay rain --map-mpp 60 (map-rain)
  ... --menu map --map-overlay rain --map-mpp 12 (map-rain-near)
```

| Shot | Before | After |
| --- | --- | --- |
| The world: rain falling, at night | `before-world.jpg` | `after-world.jpg` (the same: the player's rain did not change) |
| The map's RAIN overlay, close | `before-map-rain-near.jpg`: no rain under the player's arrow west of Durnacrag; blobs south-east of Durnacrag and by Holbrook, the owner's picture | `after-map-rain-near.jpg`: the arrow stands in the rain (light blue, 50%), the storm's trailing edge round the bursts |
| The RAIN overlay, wide | `before-map-rain.jpg` (-10..10 mm/h) | `after-map-rain.jpg` (-100..100 %) |
| The clouds-and-rain layer | `before-map-clouds.jpg`: almost no blue anywhere, the rate over 8 mm/h | `after-map-clouds.jpg`: blue where it rains |

Rendered on the Mac's GPU (M4 Max, Metal), fast profile. Each map run ends
in a wgpu "timed out waiting on the last submission" panic at exit, after
`CAPTURE_SAVED`, before and after alike; the captures in each run log the
same `LATE_PIPELINE` lines before and after (29 for the world, 32 for a map),
so neither is from this change.

## Half the rain, none on the desert

The owner, 2026-11-16: "50% less rain on the planet, and avoid rain cells
over desert". The same place and moment as above (`--at 26.262 -14.111
--time 22 --weather-at 475`), the RAIN overlay (`--map-mpp 150`, which the
map clamps to 20.9 m a pixel).

| | Before | After |
| --- | --- | --- |
| RAIN overlay | `half-before-map-rain-wide.jpg`: rain over most of the land, the deserts included | `half-after-map-rain-wide.jpg`: about half the rain; none on the deserts |
| Biomes, for where the deserts are (orange) | `half-before-map-biomes-wide.jpg` | (unchanged) |

The "before" is the build of the first part (the map reading the rain seen),
on the old settings and settled climate; the "after" is this commit's, on the
remade settled climate. The weather after 475 s is a different moment of a
different climate, so the two maps compare in amount and in where rain may
fall, not storm for storm. The numbers are in the change's design (30 days:
raining 19.8% -> 9.6% of the planet, 0.48x; mean rate 0.47x; desert 29.7% ->
0%).

## Snow in freezing air

The owner, 2026-11-16: "no rain in the cold regions, snow only please".

| Shot | What it shows |
| --- | --- |
| `snow-world.jpg`, `snow-map-rain.jpg` | `--at -56 159 --time 12`, air -2.0 C: snow falling (`Weather here: ... snow`), no rain streaks or lens drops; on the map the arrow stands in the violet snow, rain in blue to the south where the air is above 0 C |
| `snow-line-world.jpg`, `snow-line-map-rain.jpg` | `--at 77 -68 --time 12`, right on the line (air -0.0 C): flakes round the player, and a rain shaft or two from the cells just over the line. The map's texels (118 m) blend rain and snow at the line, so there the map leans blue where the player has snow: the line itself is the one place the two can differ, by under a texel |

Both from `atmosphere::tests::snow_falling` (an instrument).
