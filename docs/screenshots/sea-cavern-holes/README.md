# Holes in the sea at caverns beside it (`ocean-environments` B22)

The owner, 2026-11-16, on a beach at `41.636 11.664`: "theres some holes here
in caverns next to the ocean".

## The videos

On Drive, `My Drive/dungeon-crawler-2026/pale-blue-dot-validation/`, H.264
1080p60, before and after at each place:

| Place | File stem | Where | Tide | What the path crosses |
| --- | --- | --- | --- | --- |
| The owner's spot | `sea-cavern-owner-spot-<before\|after>-1080p60.mp4` | `41.636 11.664`, the pit and cavern 6 to 9 m west | high (`--tide high`, surface +1.5 m) | over the bank, down into the pit, through the waterline, under the bank's overhang into the cavern, and back up |
| A sea cave | `sea-cavern-sea-cave-<before\|after>-1080p60.mp4` | `35.952 18.251` | the day's (about -1.3 m) | from the shore into the cave's mouth, under the waterline at the mouth, into the cave under water, up into the cave's air, and a look back out |
| A cave off a bay | `sea-cavern-bay-cave-<before\|after>-1080p60.mp4` | `39.67 9.54` | the day's | into the mouth, under at the mouth, up and into the cave, a look back out |

**How they were made:** the Mac (M4 Max, Metal), `--profile fast`, a fixed
60 fps step (looks, not frame time), a 960x540 window (1920x1080 on the
retina screen), `--time 13 --rain 0`, lens drops off. The camera follows
`PBD_EYE_PATH`; the label is the eye's height against the renderer's own
surface (`eye_surface_radius`), its state and the frame. Before is
`origin/claude/env-v2` at `a256f35`; after is this branch. The paths were
laid through open cells with the measurement `sea_cave_report`
(`PBD_ROW`), so the camera is never inside a block.

`*-sheet.jpg` is 12 frames across each video; `*-crossing1.jpg` and
`owner-spot-after-crossing2.jpg` step the frames through each crossing of
the waterline. `stills-before-after.jpg` is four still pairs (the owner's
spot from above at high tide, twice; the sea cave at the day's tide and at
high tide).

## What each before shows, and the after

- **Owner's spot, before:** the pit behind the beach is dry cobble at high
  tide, the sea's sheet stops at its edge (the hole the owner saw); under
  water the eye reads "under" in the dry pit (the eye's rule said sea, the
  sheet said dry); under the bank's overhang the cavern is dry and the eye
  "above" 0.7 to 1 m under the tide's surface.
  **After:** the pit holds the sea's surface at the tide's height; the eye
  crosses it once each way (f1448/f1472 down, f3140/f3146 up); the cavern
  under the overhang is under water and read so.
- **Sea cave, before:** the cave's water is drawn by the mesher at 0 m while
  the sea outside stands at -1.3 m: a slab of water floating in the mouth
  with no face under it, and a second surface inside the cave over the eye.
  **After:** one surface, the sea's, continuous from the shore into the
  cave at the tide's height, shaded by the cave.
- **Bay cave, before:** the same floating slab at the mouth and a raised
  sheet over the cave's floor. **After:** gone; the cave's shallow floor is
  above the low tide's surface and shows dry, as the shore outside does.

## Frame by frame: every issue seen, and what became of it

1. **The hole at the pit and the cavern** (before, all of the owner's spot):
   fixed, the sea's reach (design B22).
2. **The sea cave's floating slab and double surface** (before, sea cave
   f767-f990, f2109-f2332; bay cave f320-f712, f1890-f2283): fixed, the
   sheet draws the sea's top wherever it is listed.
3. **The eye "under" in a dry pit** (before, owner's spot f1638-f2165):
   fixed, the eye reads the same mark.
4. **The camera inside blocks** (the first takes of all three paths: white
   sky and dry cobble seen from inside a sand block or a roof slab): the
   paths, not the game. The takes were thrown away and the paths re-laid
   through open cells.
5. **The label said "no water" over the flooded pit** (the first after take
   of the owner's spot): the capture label's fallback read only the height
   field. Fixed (`eye_path.rs` reads the reach) and the take re-recorded.
6. **A dark band in the lower half for the last 2 to 6 cm over the surface**
   (owner's spot after, f1424, f1448): the composite's near-sheet stand-in
   (B19's "hairline at the waterline"). Not changed here.
7. **The bay cave's shallow floor shows dry at low tide** (after, f1890 on):
   the tide is drawn over the column, as at the shore outside. Not a hole:
   the floor is over the tide's surface there. Left as is.
8. **The cave's interior is dark under its own water** (after): the sheet
   is shaded by the cave's sky light, as the mesher's water was. Intended.
