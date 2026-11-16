# Handoff, 2026-11-16: water first

For the next session (the owner is moving to another account). Read
`CLAUDE.md` first; it is the only copy of the rules. This note says where
things stand and what to do next, in priority order.

## Where everything is

- **Playtest branch:** `claude/env-v2`. Every finished fix is merged here.
  PR **#49** takes it to `main`. Build with
  `cargo build --profile fast -p pbd-app`.
- **Tests at the last push (`24d623a`):**
  - pbd-app 495 + 76 pass;
  - pbd-core 557 pass, the v4/v5 ground digests included (now pinned per
    OS: PR #56);
  - `openspec validate --all` passes 163.
- **Bug list page** (the owner's bugs with screenshots):
  https://claude.ai/artifact/GGaWtEEguVRkZAsYQ7eCYR
  (`docs/design/bug-list.html`). Keep it current, and add the bugs below to it.
- **Tracker:** https://claude.ai/artifact/EwNcNzHQU3B7C3jyi629Wk
- **Cloud session "PDB environment improvements"** works on
  `claude/env-v2-finish`. Its PRs go into `claude/env-v2`: M34 River Craft,
  inner tubes. Keep it mergeable: merge `claude/env-v2` into it, never
  force-push, and delete no branches (owner).
- **Repro key:** Shift+Alt+F in game copies a repro record to the clipboard:
  the command line, place, view, weather, and the blocks 3 cells around and
  3 layers up and down. Ask the owner to press it on every new bug.

## Priority 1: water (the owner: "prioritize water fixes")

**The validation rule for water bugs** (owner; memory
`complex-bugs-need-video`): a water bug is not fixed on stills.
- Make before and after MP4s at 1080p60 that cross the waterline slowly,
  with the eye's height burned in. The capture-only `PBD_EYE_PATH` exists.
- Upload them to Drive: `My Drive/dungeon-crawler-2026/pale-blue-dot-validation/`.
- Step the frames within ±0.3 m of every crossing, list every issue, and
  fix it.
- The coordinator reviews the frames too before merging.
- Cover both kinds of river and every place waters meet: mouths,
  tributaries, falls into pools.

**Done so far:** PRs #55 and #64 (merged). The capture videos are on Drive
and the contact sheets are in `docs/screenshots/river-fixes/videos/`.
- **Under-water test:** one function, `eye_surface_radius`, drives it.
- **Surface overhead:** drawn when the eye is under.
- **Mouths:** no double surface at river mouths.
- **Beds:** drawn as beds at draw time.
- **Waterline sky:** no darkened sky above the waterline.

**Still wrong, from the owner's last screenshots in this session:**
1. **The waterline shades above the water.**
   - The owner stood at `52.053 -118.367`, elevation -0.4 m, -0.4 m above
     ground: standing in shallow water at the edge.
   - The bank and sand ABOVE the water's surface show a dark blue band
     starting at the waterline.
   - The bright bands are on the far side.
   - The left bank's base reads dark teal as if under water.
   - This is the "bright/dark strip at the waterline" family. Reproduce it
     standing in very shallow water (eye just above the surface, feet in it)
     at that spot.
   - The owner: "you must be submerged at just the right height, if not 1
     block try 2 blocks".
2. **A tree stands in the water.**
   - In the sea-shore validation video, an acacia stands in the sea off the
     beach (`docs/screenshots/river-fixes/videos/sea-shore-after-sheet.jpg`).
   - Trees must not stand where water covers the cell. Extend the draw-time
     "no plants where water stands" rule (flora-v2 12.10) to the sea and
     tide-covered cells, and to trees, not only clutter.
   - The axe and fruit read the same records, so they stay consistent.
3. **Water sounds play on land.** Standing on a dry block that peaks above
   the water (the owner's shot above), the water footstep or ambience still
   plays. The audio's water test (`crates/pbd-app/src/audio/hooks.rs`,
   `Family::Shallow`) must use the same surface test as the renderer
   (`eye_surface_radius` / `Sea::state`) at the feet.
4. **The underwater river still needs work** (owner: "getting better, but
   still lots of work to do").
   - The shot is at `56.322 -117.795`, under a river.
   - The surface from below reads as a dark navy ceiling with blotches.
   - There's a band of speckled noise at the far end of the channel
     (dithered fog or a fade?).
   - The sand rims look floating.
   - It is far darker than the sea's look, because the bed is dirt and the
     channel is shaded. Lightening river beds (sand or gravel) is the
     owner's call: ask with the side-by-side
     `docs/screenshots/river-fixes/videos/same-depth-sea-vs-river.jpg`.
5. **Can't swim up at a waterfall's foot** (owner, 2026-11-16: "in river, I
   can't swim to the surface if I start at a lower block on a waterfall").
   Starting in the lower reach under a step, holding swim-up never reaches
   the surface. Likely cause: the swimming code's surface is read from the
   upper cell's drawn top, or from the sea's level, not from this column's
   water. Use `eye_surface_radius` / `water_flow::drawn_top_m` for the column
   the body is in. Pin it with a test in `walking.rs` beside the pool tests
   (`pool_rows`), and record it with the owner's Shift+Alt+F record if they
   send one.
6. **Two open items from the last river pass:**
   - a tributary riffle frame (f512) seen through a cell's side face;
   - mouth f804-808: the eye reads "above" for 5 frames after a 40 m capture
     teleport while the terrain rebuilds.

## Priority 2: performance (the owner's condition on approvals)

Merged but never measured:
- the water smoothing (#52);
- the rain splashes;
- the four REFLECTIONS levels (#58);
- the height fog.

They need a quiet Mac: unlocked, Chrome closed, no builds, and `caffeinate`
running. Or the desktop. The commands are in the tasks: water-flow 5.3,
weather-rain 6.5, reflection-quality 3.2, weather-rain 8.4. Publish each
report and link it from CLAUDE.md.

## Priority 3: other open work

- **The bot** (`step-videos`, `starter-missions`):
  - trunk, fishing and farming pass;
  - herding stalls taming (hens far across terraces);
  - getting about stalls on the 200 m walk;
  - mining is now testable (ore in generator 10).
- **The Windows ground check:** run `cargo test -p pbd-core planet_gen` on
  the desktop (digests pinned on Mac and Linux only).
- **The terrain's far LOD band** is still a distance fade, against the
  owner's rule "a fade is a moment in time". It's written up in
  distance-lod-fade.
- **Bait** has no item, so the `bait` mission is pending. The starter kit
  still gives fences, gates and torches to new worlds.

## Session notes

- Cross-session messages from the desktop did not reach this session (a
  product bug, with feedback drafted). Ask the owner to paste them, or have
  sessions hand off through git files like this one.
- The disk filled twice. Clear finished agents' `target/` folders; the
  `cb-*` and crab-breeze targets were cleared on 2026-11-16 (495 GB free).
