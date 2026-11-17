# M35's waterfall look in the game (`water-flow` 6.6)

The owner approved mockup M35 (Waterfall mist, version 22) for the game on
2026-11-16: "alright, I'm happy with this, send it yo". These are the game's
stills of the port beside the mockup's New tab at its defaults.

- **Renderer:** the Mac (M4 Max, Metal). The game: `--profile fast`,
  `--capture` at frame 180 (fixed step: looks, not frame time), the lens off
  (`PBD_PASS_OFF=lens`), `--fly --time 13 --rain 0`, the eye held by
  `PBD_EYE_PATH` (one key: `0 lat lon altitude yaw pitch`). The mockup: the
  system Chrome on the GPU (ANGLE Metal, never SwiftShader), the canvas held
  at t = 12 s, by `tools/mockup_waterfall_mist_shots.js` (served over http:
  a `file://` page cannot upload its puff sheet to WebGL).
- **Falls:** the shipped seed's jungle cascade. Tall: 9 layers at
  `11.36520 4.78342` (`D` 8.64 m, pool 3 m); short: 2 layers at
  `12.35952 5.07612` (`D` 1.62 m, pool 2 m). Against the mockup's 9-layer
  (`D` 9.4) and 2-layer (`D` 2.4) falls. Each eye is placed on the fall's
  open side as the mockup frames its own (above: 6 + 0.85 D m out at -36°;
  beside: 5 + 0.55 D m at -9°; under: 1.6 m down in the pool at +16°).

| Shot | Eye (`lat lon alt yaw pitch`) |
| --- | --- |
| `tall-above` | `11.25099 4.71061 16.73 -122.0 -36` |
| `tall-side` | `11.26276 4.71811 10.28 -122.0 -9` |
| `tall-under` | `11.33992 4.76730 3.40 -122.0 16` |
| `short-above` | `12.35772 5.15450 44.07 -1.3 -36` |
| `short-side` | `12.35776 5.15269 40.87 -1.3 -9` |
| `short-under` | `12.35874 5.11005 37.40 -1.3 16` |

`*-sheet.jpg` is each shot, mockup left, game right; `game-*.jpg` and
`mockup-*.jpg` the two alone. `glassy-off-on.jpg` is the tall fall from the
side with the splash off, `falls.glassy` false then true: the strands at the
lip and the smoothed lip top are the Glassy lip.

## Differences between the stills and the mockup

1. **The places differ.** The mockup's falls drop off a sand plateau into one
   round pit; the game's are a jungle cascade of stepped pools, several falls
   in one view, under a canopy that shades them. Every look term below is
   read against that.
2. **Every riffle mists.** The proposed rule mists every fall, so in the game
   a river's one-layer steps (dozens near the cascade) each carry 8 puffs at
   0.45 x 2. Over a stepped river that is a line of puffs along it; the
   mockup has one riffle. This is the rule as approved, but it reads busier
   in the game than in the mockup.
3. **The mist is whiter and harder-edged up close in the short fall.** The
   game's puffs at a short fall stand out white against the dark jungle
   pool; the mockup's sit against bright sand and read softer. Same sheet,
   same alpha; the backdrop and the light differ.
4. **Under the water is the game's murk, not the mockup's teal.** The game's
   compose murks by its own downwelling (a dark navy two metres down in the
   jungle); the mockup's under view is a flat teal. Not touched here.
5. **Puffs from neighbouring pools show from under.** In the cascade a
   neighbour's mist is in front of the eye's own water from under (the
   6.1 shots showed the same with the old dot). A puff's part under its own
   pool's surface is no longer drawn.
6. **Bubbles are larger on screen up close.** The game's bubbles are the
   mockup's sizes, but its eye stands nearer them, and the 2-pixel floor is
   on the viewport's height in physical pixels.
7. **The glassy lip is a corner weight on the top, not metres along the
   flow.** The mesher gives a top's corners a lip weight (1 at an edge that
   goes over); the fragment reads it as metres before the lip over half a
   cell. The mockup's metres are along the flow ray from the source, so its
   glassy band follows the flow more sharply.
8. **Rain rings on still pools with `--rain 0`.** The game's pools show rings
   in these captures; they are the game's weather at that place and time,
   not part of the port (the 6.1 shots show them too).
9. **The fall's height measure.** The game's tall fall drops 8.64 m (the
   mockup's 9.4 m), its pool is 3 m (the mockup's 3.35 m), so it makes 13
   puffs to the mockup's 14 and slightly smaller ones.

Not ported, by the owner's word: the rounded lip ("nope go back") and the
droplet streaks ("I dont want them in game yet").

## Frame cost: owed

The owner's condition ("need to be a bit careful with perf") is not met
here: no `perf_suite.py` A/B was run in this sitting. The suite has no
scenario at a fall (`--stroll` needs a town, so it cannot stand in for one).
Until a scenario exists, the A/B is a held view at the tall fall with the
frame log, old exe and new interleaved, on the `--release` build with
nothing else running:

```text
pbd-app --fly --at 11.26276 4.71811 --height 10 --yaw -122 --pitch -9 --time 13 --rain 0 --frame-log <dir>/<variant>-<n>.csv
```

for 60 s each, three runs a variant, then `tools/frame_graph.py`; and
`PBD_PASS_OFF=splash` on the new exe prices the splash pass alone.
