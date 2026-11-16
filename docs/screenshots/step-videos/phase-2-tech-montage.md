# Phase 2 (environment v2) tech montage

One technical montage of environment v2, cut on 2026-11-15 from scenery clips that had already been recorded. It follows the
owner's rule that "tech demos show technical things, in montages", and the owner okayed cutting it from the existing
clips. No game, OBS or browser was run to make it.

- **File:** `My Drive/dungeon-crawler-2026/pale-blue-dot-phase-1-2-demos/pale-blue-dot-phase-2-tech-montage-1080p60.mp4`
- **Length:** 2:10.3 (130.3 s). H.264, 1920x1080, 60 fps, yuv420p, CRF 20, `+faststart`, no audio, 147 MB.
- **Contact sheet:** [phase-2-tech-montage-sheet.jpg](phase-2-tech-montage-sheet.jpg) shows 12 frames spaced evenly across the video.
- **Left out:** all water-flow clips (pour, fall, arrows, floods, tides, river). The owner said "I'd hold off on the water
  stuff until the lerp work is done".

## Shot list

The times are where each segment starts in the montage. Each crossfade begins 0.3 s before that time. "In" is the in point
in the source clip, `output/clips/<clip>-cut.mp4`, which is the recording with the loading screen already removed.

| # | Montage start | Length | Source clip | In | Caption |
| -: | -: | -: | --- | -: | --- |
| 0 | 0:00.0 | 2.0 s | title card | - | Pale Blue Dot / Environment v2 · tech montage |
| 1 | 0:01.7 | 1.5 s | section card | - | Grass / Ground cover |
| 2 | 0:02.9 | 9.0 s | p2-grass-tufts-fields | 1.5 s | Pixel tuft grass, coloured from the ground |
| 3 | 0:11.6 | 9.0 s | p2-grass-jungle | 1.0 s | Jungle floor: ferns and tufts in the biome's greens |
| 4 | 0:20.3 | 1.5 s | section card | - | Trees / Species, seasons and fruit |
| 5 | 0:21.5 | 9.0 s | p2-trees-temperate | 2.0 s | Temperate: oak, birch and acacia |
| 6 | 0:30.2 | 9.0 s | p2-trees-autumn | 2.5 s | Autumn: crowns turn |
| 7 | 0:38.9 | 9.0 s | p2-trees-winter | 2.5 s | Winter: bare branches |
| 8 | 0:47.6 | 9.0 s | p2-trees-jungle | 4.0 s | Kapok and palm in the jungle |
| 9 | 0:56.3 | 6.2 s | p2-trees-taiga | 10.6 s | Taiga: pines on the snow |
| 10 | 1:02.2 | 9.0 s | p2-trees-desert | 3.0 s | Desert: cacti and palms |
| 11 | 1:10.9 | 7.0 s | p2-fruit-close | 2.0 s | Fruit in the crowns |
| 12 | 1:17.6 | 1.5 s | section card | - | Sea life / Under the water |
| 13 | 1:18.8 | 9.0 s | p2-reef | 1.5 s | Reef: corals, sponges and fish |
| 14 | 1:27.5 | 9.0 s | p2-kelp | 3.0 s | Kelp forest |
| 15 | 1:36.2 | 8.5 s | p2-vents | 0.3 s | Hydrothermal vents at 82 m |
| 16 | 1:44.4 | 8.5 s | p2-sea-ice-seals | 3.0 s | Sea ice, with seals hauled out |
| 17 | 1:52.6 | 9.0 s | p2-fish-lineup | 5.3 s | Every water species |
| 18 | 2:01.3 | 9.0 s | p2-underwater-light | 3.0 s | Sunlight shafts under the surface |

The 82 m depth in the vents caption comes from the clip's own debug readout ("-82.3 m elevation"). The tree species named in
the captions match the tree kit listed in the clip logs (oak, birch, kapok, palm, acacia, pine, willow, cactus).

## How it was cut

The montage was cut with ffmpeg 7.1 from `imageio_ffmpeg`, with a script in the session scratchpad. The script was not
checked in.

1. **Choosing in and out points.** I made a 1 fps contact sheet of every clip, plus 2 to 4 fps sheets of taiga and fruit,
   and picked each shot's in and out points from them.
   - The temperate, autumn and winter clips (and grass-tufts-fields) open with a large translucent dithered ghost of a near
     tree, which is still fading in. Every one of those shots starts at 1.5 s or later, and the trees start at 2 s or later.
   - The fish line-up only appears 5 s into its clip, so that shot starts at 5.3 s.
2. **Cropping the debug readout out.** The debug text reaches about y = 285 px of the 2880x1856 frame, below the macOS
   window bar, the compass and the town label. Every shot uses the same crop, `crop=2766:1556:57:300`: it removes the top
   300 px and keeps 16:9, centred. The crop is then scaled with Lanczos to 1920x1080 at 60 fps. The compass and the debug
   text are gone, and the hotbar stays at the bottom of every shot.
3. **Captions.** Each caption is burned in with `drawtext`, Arial 40 px, white on a black band at 55% opacity. It sits
   bottom-left (x = 56, y = h - 170), above the hotbar.
4. **Cards.** The title card (2 s) and the three section cards (1.5 s each) are Arial Bold on dark navy `#0B1320`, with a
   thin blue rule and a grey subtitle.
5. **Assembly.** Each segment was encoded at 1080p60 (CRF 14) and the segments were joined with 0.3 s `xfade` fades. The
   video fades in from black over 0.5 s and out over 0.6 s, and the final encode is CRF 20, preset slow, with `+faststart`.

## What could not be avoided

- **Fruit (shot 11):** thin, bright vertical streaks of a light blue-white, apparently rain streaks, cross the frame about
  once a second through the whole fruit clip. No clean 6-second stretch exists, so they show in this shot.
- **Taiga (shot 9):** a faint thin vertical line shows briefly mid-frame, about 3.4 s into the shot (source clip about
  14 s). The window chosen, 10.6 to 16.8 s, was the longest stretch with the fewest lines. Lines also show at about 5.5 s
  and 10 s in the source.
- **Tree ghosts:** none are visible. Frames checked at the start of the autumn, winter and fruit shots show no ghost tree.
- **Debug text:** none is visible in any shot.
