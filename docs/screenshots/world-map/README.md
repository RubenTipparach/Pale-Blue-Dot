# World map mockup: screenshots

For `openspec/changes/world-map` task 1.3: one screenshot of each layer of
`docs/mockups/world-map.html`, taken in headless Chromium at 1440 x 900 (and
400 x 800 for the phone) from a local server that wraps the page as the
Artifact publish does. The published page is linked from the PR and the
roadmap.

The screenshots use the page's fallback fonts, because the sandbox that took
them cannot reach Google Fonts; the published page loads IBM Plex and
Spectral SC. That font request is the only error in the console.

| file | what it shows |
| --- | --- |
| `dusk.jpg` | The page as it opens: the spawn's continent at 19:30 on day 150, the night side coming in from the east, the settlements' lamps lighting the ground round them, and the live cloud layer (`examples/map_weather.rs`, twelve frames two game hours apart, faded one into the next). |
| `noon.jpg` | The same view at noon (`L`). |
| `close.jpg` | Before task 1.3c: `Home`, zoomed to the player, where each raster pixel (14.7 m) shows as a block of the top block's colour. |
| `biomes-750.jpg` | The biome overlay at four times the width (survey B1), greying the planet out beneath it (survey M3), with each biome's share of the land. |
| `biomes-today.jpg` | The same, as shipped: the temperate land is 91% fields. |
| `temperature.jpg` | The second year's mean surface temperature from the balanced atmosphere at the game's level 5 (`climate-balance` task 3.1): 15.2 °C over the whole surface, 17.3 °C over the water. An overlay hides the live clouds and fades the night side to a hint, since a year's mean has no hour. |
| `sea-ice.jpg` | Water frozen all year, and for part of it. |
| `fish.jpg` | How many species can live in each water, by the fish plan's rules. |
| `opens.jpg` | The page as it now opens: the spawn's noon, with the night side and the live clouds and rain off until asked for (the owner, 2026-09-27). |
| `weather-wind.jpg` | The surface wind in the game's speed ramp, with streaks running along it, and a cyclone east of the spawn. Every weather shot is the game's own overlay (`Overlay::texel`) at the clock's hour, from `examples/map_weather.rs` 30 days into a new world (decision 8). |
| `weather-jet.jpg` | The wind at cloud height, which carries the cloud. |
| `weather-currents.jpg` | The sea's surface current, the land left clear, the streaks sped up most since the current is slow. |
| `weather-cloud.jpg` | Cloud cover, fading to clear as the globe's does. |
| `weather-rain.jpg` | Rain, and snow in violet near the pole. |
| `weather-humidity.jpg` | Relative humidity at the surface. |
| `weather-sunlight.jpg` | Sunlight reaching the ground after the cloud. |
| `weather-temperature.jpg` | The surface temperature now, beside the year's mean in `temperature.jpg`. |
| `night-and-clouds-on.jpg` | Both toggles turned on, at midnight (`L`): the night side and the towns' lights, with the live clouds. |
| `closed.jpg` | `M` again: the map closed over the game. |
| `phone.jpg` | The page at phone width. |
| `close-smooth.jpg` | After task 1.3c (survey M5), at the 100 m scale: the map drawn with linear filtering, and the 7.4 m tiles loaded for the view in place of the 14.7 m base. |
| `closest-detail.jpg` | The finest zoom, at the 50 m scale, on the 3.7 m tiles. Filtered, not blocky. The pale stripes across the sand are in the raster itself, not the drawing. They are most likely moire: a 3.7 m pixel sampling cells 2.83 m across, where sand and grass alternate cell by cell, so the rows of pixels and the rows of cells beat. Not yet measured. |
