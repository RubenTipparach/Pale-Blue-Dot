# M35's waterfall look in the game (`water-flow` 6.6)

The owner approved mockup M35 (Waterfall mist, version 22) for the game on
2026-11-16: "alright, I'm happy with this, send it yo". These are the game's
stills of the port beside the mockup's New tab at its defaults (second round,
after the coordinator's review of the first).

- **Renderer:** the Mac (M4 Max, Metal). The game: `--profile fast`,
  `--capture` at frame 240 (fixed step: looks, not frame time), the lens off
  (`PBD_PASS_OFF=lens`), `--time 13 --rain 0`, the HUD and debug text
  cropped off (the game has no flag to hide them). The mockup: the system
  Chrome on the GPU (ANGLE Metal, never SwiftShader), held at t = 12 s, by
  `tools/mockup_waterfall_mist_shots.js` (served over http: a `file://` page
  cannot upload its puff sheet to WebGL).
- **The falls, above and beside:** the mockup's own scene made in the game.
  At the default spawn (`24.117 -2.964`, desert sand, the mockup's latitude)
  the capture rig digs a pit one ring wide and pours a source two cells
  short of it: `pbd-app --walk --yaw 90 --pitch -25 --pour 1 --pour-ahead 12
  --pit 9 --water-ticks 60` (tall: `D` 9.79 m, the mockup's 9.4) and
  `--pit 2` (short: `D` 2.79 m, the mockup's 2.4). The camera is moved after
  the pour by `PBD_EYE_PATH`: keys 0 and 150 at the spawn pose (so the pour
  lands where it always does), then key 151 at the shot.
- **From under:** the pits' pools are one layer deep, so the views from under
  are in the jungle cascade's deeper pools (`--fly`, 3 m and 2 m deep).

| Shot | Eye at key 151 (`lat lon alt yaw pitch`) |
| --- | --- |
| `tall-above` | `24.1644 -2.9703 75.0 90 -50` |
| `tall-side` | `24.07649 -3.00759 60.09 121.6 -9` |
| `tall-under` | `11.33992 4.76730 3.40 -122.0 16` (cascade, single key, `--fly --at 11.33992 4.76730`) |
| `short-above` | `24.1167 -2.9703 70.0 90 -50` |
| `short-side` | `24.07642 -3.00754 64.29 121.6 -9` |
| `short-under` | `12.35874 5.11005 37.40 -1.3 16` (cascade, single key) |
| lip (glassy) | `24.05137 -2.99064 64.19 121.6 8`, `--pit 9`, splash off |

`*-sheet.jpg`: mockup left, game right. `glassy-off-on.jpg`: the tall pit's
lip, `falls.glassy` false then true. `cascade-wall-blend.jpg`: finding 1.

## What the review found, and what was fixed

1. **Tall fall from the side showed almost no mist (first round, the jungle
   cascade).** Measured with a temporary log of `animate_splash` at that
   view: 61 feet, 542 puffs a frame, mean half-size 1.2 m, alpha up to 0.36
   before the sheet, every fade at 1. The puffs were made; the wall blend
   took them. In that cascade every fall stands among other falls' faces, so
   each puff has a falling sheet within a metre or so behind it, and the soft
   fade (1.2 m against the water's depth) fades it out; with the wall blend
   at 0 the same frame shows the puffs, cut hard where they pass behind the
   faces (`cascade-wall-blend.jpg`). In the mockup's scene, a pit, the same
   rule rolls two big clouds either side of the column: `tall-side-sheet`.
   Also fixed on the way: a foot's fades were keyed by its column alone, so
   two feet stacked in one column (a cascade) reset each other's fade every
   frame; they are keyed by column and layer now.
2. **Short fall puffs: hard white blobs.** At full size the sheet samples
   smooth (linear, edges soft over its three texels) and the two frames
   cross-fade. Placement is from the foot's edge (1.42 m plus the spread) of
   each fall; in the cascade a river's every one-layer step is its own fall,
   so its puffs line the whole river (the approved rule). Against the pale
   pit (`short-side-sheet`) they read as the mockup's do.
3. **Glassy on and off looked the same.** Fixed. A falling cell's top slopes
   down to its low corners, so on the run's top cell the metres under the
   lip were measured from the cell's top, not from the face's own top edge:
   most of the edge started at 0.98 m, already past the glassy band. On the
   run's top cell the metres now start at each corner's own level.
   `glassy-off-on.jpg`: off, white froth to the edge; on, clear green water
   over the edge, then strands, filling in down the face.
4. **The captures carry no HUD now** (cropped; no flag exists).

## Differences that remain

1. **The pit is one ring wide; the mockup's 9-layer pit is two.** The mist
   fills the game's narrow pit wall to wall from the side, where the
   mockup's rolls out into a wider pool.
2. **The spray drops near the eye read as white balls.** Same sizes (0.08 to
   0.18 m) and alpha as the mockup; the game's side eye stands inside the
   pit, a few metres from them, where the mockup's stands ten metres off.
3. **The poured water spreads over the sand round the pit.** The game's rule
   (Luanti's) spreads a source seven cells, so the pit is fed along a
   stretch of its rim and several falls go in, with a flooded sheet behind
   them; the mockup has one fall off a dry plateau.
4. **Wall blend among many falls.** In a cascade (several falls side by
   side) the mist fades against the neighbouring sheets and reads far
   thinner than in the mockup's single fall (finding 1). As the mockup's rule
   was approved; a narrower blend against falling faces would be a look
   change for the owner.
5. **Under the water is the game's murk** (a dark navy two metres down in the
   jungle), not the mockup's flat teal; and a neighbouring pool's mist shows
   through it in the cascade (the 6.1 shots show the same with the old dot).
6. **Bubbles look large on screen up close**: the mockup's sizes, nearer eye.
7. **The glassy band on the lip's top** is the mesher's corner lip weight read
   as metres over half a cell, not metres along the flow as the mockup's.

Not ported, by the owner's word: the rounded lip and droplet streaks.

## Frame cost: owed

Not measured here. The suite has no fall scenario (`--stroll` needs a town).
Until there is one, the A/B is a held view over the scripted pit, old exe and
new interleaved, `--release`, nothing else running, three runs a variant of
60 s, then `tools/frame_graph.py`:

```text
pbd-app --walk --yaw 90 --pitch -25 --pour 1 --pour-ahead 12 --pit 9 --water-ticks 60 --time 13 --rain 0 --frame-log <dir>/<variant>-<n>.csv
```

and `PBD_PASS_OFF=splash` on the new exe prices the splash pass alone.
