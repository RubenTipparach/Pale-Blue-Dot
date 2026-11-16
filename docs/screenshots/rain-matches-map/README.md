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
