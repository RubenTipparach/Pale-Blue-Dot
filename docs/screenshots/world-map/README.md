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
| `closest-detail.jpg` | The finest zoom, at the 50 m scale, on the 3.7 m tiles. Filtered, not blocky. The stripes across the sand are in the ground itself (measured 2026-09-28, on the game's own map at 0.7 m a pixel): a desert's top is sand broken by rock, chosen by `(latitude * 977).fract()` in `planet_gen::top_material`, a latitude-only dither ported from Tenebris. So the rock runs in east-west bands about a metre wide every four to six metres, and a swamp's water and dirt the same way. First read here as moire; it is not. Whether it stays is asked in the survey (G1). |

## The map in the game (task groups 3 to 5)

Captured 2026-09-28 in a cloud container on lavapipe at 1440x900, with no GPU,
each a fresh run of the fast build (`--profile fast`) on a memory-only save:
`--walk --menu map --time 12` plus the flags named. They show what the map
looks like, not how smoothly it runs; frame cost was not measured
(CLAUDE.md). The atmosphere is spun up from rest, as the level-5 settled
climate was still being made.

| file | flags | what it shows |
| --- | --- | --- |
| `game-open.jpg` | none | M's map as it opens: centred on the player at 28.7 N, 5 m a screen pixel, drawn from the base and the 5.7 m tiles. The white arrow is the player, pointing along the walk; the amber diamonds are the ship, parked under the arrow, and two boats. The legend is on the left and the place under the pointer and the scale in the corner. |
| `game-planet.jpg` | `--map-mpp 40` | Zoomed all the way out: the whole planet, both poles on screen, 20.9 m a pixel. The same planet as the approved mockup's base, the snow line near 62 degrees included. |
| `game-night-terminator.jpg` | `--map-mpp 40 --map-night` | The night side at noon on day 0 (task 3.1). It crosses the equator at -60.6 and 116.2 degrees east, measured off this picture, against -59.5 and 116.1 from `Clock::daylight` (`examples/terminator.rs`); a pixel is a quarter of a degree here. Taken with the first night strength, which left the night at 55% of the day's brightness; it is darker now (`game-night.jpg`). |
| `game-closest.jpg` | `--map-mpp 0.5` | The closest zoom, 0.7 m a screen pixel on the 2.83 m tiles: every cell four pixels across, the terraces and the forest edges cell by cell. The striped sand is the desert's rock, a latitude dither in the generator (survey G1), not the drawing. |
| `game-night.jpg` | `--time 20 --map-night --map-mpp 12` | 20:00: the night coming in from the east, at about a third of the day's brightness. |
| `game-clouds.jpg` | `--map-clouds --map-mpp 12` | The live clouds from the atmosphere's own maps, with the rain falling round the spawn in blue. |
| `game-wind.jpg` | `--map-overlay wind --map-mpp 12` | The wind overlay in the game's speed ramp over the greyed base, from the same texels as the globe's, with its ramp in the key. |
| `game-rain.jpg` | `--map-overlay rain --map-clouds --map-mpp 12` | Rain, fading to clear where it is dry as the globe's does. The clouds switch is on, and under an overlay they are not drawn, as in the mockup. |
| `game-biomes.jpg` | `--map-layer biomes --map-mpp 12` | The biome layer, the first added through the layer registry, over the greyed base, with each biome's swatch and share in the key. |

### The layers as the mockup composes them (decision 11)

The owner, 2026-09-28: "rain and biome maps colors dont match with the
spec". The colours were the mockup's: its frames were coloured from the
game's own ramps. How the layers were laid down was not. The `-before`
shots are the build before the fix, with the same flags. Measured off the
shots (sRGB, 0 to 255, a patch's mean):

| | before | after | the mockup |
| --- | --- | --- | --- |
| sea under the biome layer | (98, 119, 140), blue-grey | (64, 67, 67), charcoal | charcoal, greyed and darkened |
| rain at its heaviest | (157, 172, 231), pale under cloud | (29, 54, 172) | (26, 53, 168), `weather-rain.jpg` |
| dry land under the rain | (95, 112, 136) | (54, 56, 58) | grey |

- The planet under an overlay is greyed by luminosity and darkened by 45%,
  as the mockup's canvas does, by the base's own material rather than a
  grey veil laid over it. A GPU test holds the shader to the mockup's
  arithmetic.
- The biome layer is drawn at 0.88 and a weather overlay at 0.9, the
  mockup's opacities.
- The live clouds are not drawn under an overlay, and the night only at 0.3.
- The legend has the mockup's key.

The key's shares are counted off the base the game draws, so a beach under
the sea's surface is sea, as it is drawn. The mockup's summary counted it as
beach, which is the difference between its 10% beach and the game's 5%.

| file | what it shows |
| --- | --- |
| `game-biomes-before.jpg`, `game-rain-before.jpg`, `game-wind-before.jpg` | The same three views before decision 11. |
| `game-cloud-beside-globe.jpg` | globe: `--view orbit --overlay cloud --time 12`; map: `--map-at 19.6 51 --map-mpp 16 --map-overlay cloud` | Task 4.2: the globe's cloud overlay from orbit (it looks down on 19.6 N 51 E) beside the map's at the same clock, drawn from the same weather maps: the same central mass, the clear gap to the south-east and the blobs to the south. The map holds its whole height on screen at this zoom, so it centres nearer the equator than the globe does. |
| `game-clouds-live-at-51e.jpg` | the map's, with `--map-clouds` in place of the overlay | The live clouds and rain at the same place and hour. |
