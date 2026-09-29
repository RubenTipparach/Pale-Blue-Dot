# Hand-off, 2026-09-29: towns slice 1 and 2a, bare hands

Where the work stopped when the day's tokens ran out, and what to do next.
Branch `claude/medieval-city-html-mockup-8692xs`, PR #19 (draft).

## Built and tested

- **Towns slice 1** (`openspec/changes/cities-in-the-world`, design
  decision 9, tasks 0.1-0.5):
  - `pbd_core::settlement` holds the chart, the ground (terrace plus
    margin) and the pieces;
  - `pbd_app::towns` lays Holbrook out at the home site and draws it;
  - `--at <lat> <lon>` places a capture's walker.
- **Owner fixes:**
  - Each building's frame follows the rows (the roofs had sat crossways).
  - Village houses are two empty columns apart, in the mockup and in the
    game.
  - `settlement::tests` hold every plan to the mockup's and keep roofs
    apart.
- **Slice 2a, walls and doorways** (task 0.7):
  - Walls, posts, chimneys and upper floors are solids
    (`walking::Structures`).
  - The walker stops at walls and goes in at doors. Slabs and lintels are
    ceilings.
  - Tests:
    - `settlement::tests::every_village_building_cuts_into_its_pieces`;
    - `walking::tests::a_town_wall_stops_the_walker_and_its_doorway_lets_it_in`.
- **Bare hands** (`openspec/changes/inventory-grid`, decision 8, task 4.5):
  - `Equipment.held` is an `Option<Tool>`, and a new world holds nothing.
  - The roll-out starts with "Bare hands" (icon `assets/items/tools/hand.png`,
    drawn by `tools/gen_item_icons.py`).
  - `Equipment::digging_tool` is the dig rule.
  - The `hand` line writes `-` for bare hands.

Checks run in the cloud session:
- `cargo test -p pbd-core`: 297 passed.
- pbd-app: the new walker, tool-slot, hand-line, town and picker tests
  pass. The full pbd-app suite was not rerun after the last edits.
- `cargo fmt --check` is clean. `openspec validate --all`: 92 passed.
- `cargo clippy -p pbd-core -p pbd-app --all-targets -- -D warnings` is
  clean.
- Frame cost was not measured (no GPU).

## Not finished

1. **The game's Holbrook captures.** Four shots sit beside the mockup's:
   lane, outside the Fieldstone house, inside it looking out, and the
   overview.
   - Run `tools/capture_holbrook.sh docs/screenshots/cities-in-the-world`
     after `cargo build -p pbd-app --profile fast`. It takes about 25
     minutes on lavapipe.
   - The mockup's shots are in the same folder. To retake them, run
     `tools/mockup_village_shots.js`.
   - Check that the overview (`--view column`) actually builds the town. If
     it does not, the site list needs a save in that view; use a `--walk`
     shot instead.
   - The README table in that folder already names the four game files.
2. **Tick tasks:**
   - `cities-in-the-world` 0.7 and `inventory-grid` 4.5, once the captures
     are looked at;
   - `cities-in-the-world` 0.6, once the owner looks at the shots.
3. **Update PR #19's body.** It still says groups 3 and 4 of `city-sites`
   are not built. Add:
   - the record store (`world-persistence` 3.1-3.3);
   - sites stored and on the map (`city-sites` 3.1, 4.1);
   - towns slices 1 and 2a;
   - bare hands.
4. **Do not merge PR #19 to main** until towns are stored records
   (`cities-in-the-world` slice 3, task 4.5; CLAUDE.md, "Saved games
   survive every change"). Holbrook is built from the template each time,
   which is safe only while no saved world has towns.

## The owner's notes on the towns mockup (2026-09-29)

The owner's screenshots are in `docs/handoff/2026-09-29-owner-notes/`,
one per note, named in each note below. Open them before starting: they are
the reference for what is wrong. Each
note is to be written up in `openspec/` before code (CLAUDE.md). It is
tracked as `cities-in-the-world` task 0.8.

1. **Water inside the boats** (`water-in-boats.png`). The harbour's water
   sheet shows through the hulls. Boats need to mask the water where a hull
   is, for example with a pass that cancels the water inside the hull's
   footprint (a stencil or depth mask).
2. **The stair block's outside has inverted faces** (`stair-block-faces.png`).
   Some faces of the enclosed stair, seen from outside, face the wrong way.
3. **Windows** (`windows.png`):
   - They should be double-sided and transparent.
   - They should not glow. The rooms behind them are lit and have their own
     light, which is enough.
4. **Chimney bottoms z-fight** (`chimney-zfight.png`). Where the chimney's
   base meets the roof or ceiling, two faces share a plane.
5. **Doors that open into furniture** (`mound-door-into-furniture.png`).
   Some doors, like the mound houses' big round ones, swing into the table
   placed behind them. Doors this big, with furniture behind them, should
   open outward. This matters for the game too: slice 2a draws every door
   leaf swung inward.
6. **Jungle bridge posts miss the platform's corners**
   (`jungle-bridge-posts.png`). In the jungle town, the rail posts at each
   end of a rope bridge should stand on the edge vertices of the hex
   platform the bridge meets, so bridge and platform rails join.
7. **The stairwells need more light.** The enclosed stairs, the newel
   stairs and the straight flights, are too dark. Give them a light of their
   own, such as a candle or a lamp on the landing.
8. **The igloo's geometry needs work** (`igloo.png`, the tundra town at
   dusk). What the shot shows at the tunnel:
   - The side walls are flat slabs whose cut ends and inner faces show
     bare and light-coloured from outside.
   - The barrel vault sits on the walls as a separate shell, with gaps at
     its ends, and does not meet the dome.
   - The dome's opening over the tunnel is a jagged cut.

   Make the tunnel one piece with the dome, built of the same snow blocks,
   with closed ends and no bare faces.

The game's `game-lane.png` was captured after the commit above and is added
beside the mockup's. It has not been looked at yet. The other three game
shots are still to be taken.

## Next steps (the owner's order)

- Slice 2b: sliding along faces, holding the walker to floors, stairs, and
  doors that open and shut through the save.
- Slice 3: lanterns and the night, settlements as stored records, the far
  form.
- Slice 4: the other templates, and villages at every site.

Differences from the mockup are listed in
`openspec/changes/cities-in-the-world/design.md` ("How Holbrook differs
from the mockup's village").

## Environment notes

- The container's disk filled once (a linker bus error). To free space,
  delete `target/debug/incremental`, `target/fast/incremental` and stale
  `target/debug/deps/pbd_app-*` test binaries.
- Never use `pgrep -f "<text>"` in a wait loop whose own command line holds
  that text: it matches itself and never exits.
