# Handoff: test `claude/phase-2-1` on the desktop (2026-11-19)

For a Pale Blue Dot session on the owner's desktop (RTX 3070, Windows). The
cloud session that built this has no GPU: every shot it took ran on lavapipe,
and no frame cost was measured. This page says what to pull, what to play,
what to measure, and what is still in flight.

## 1. Get the build

```bat
git fetch origin
git checkout claude/phase-2-1
git pull
build-play.bat
```

`build-play.bat` builds `cargo build --profile fast -p pbd-app` (CLAUDE.md:
fast for play, `--release` only for performance). The PR is
https://github.com/RubenTipparach/Pale-Blue-Dot/pull/130 (one PR for all of
this; CLAUDE.md "One pull request, not many").

**Make a NEW world** for the terrain items (swamps, beach acacias): they are
generator version 12, which only new worlds use. Also open an OLD save to
check it is unchanged (section 3).

## 2. What to play and look at

Each item: what changed, how to see it, and the cloud's screenshot to compare.

| # | What | How to check | Cloud shots |
| --- | --- | --- | --- |
| 1 | **Vines read from the ground** (`flora-v2` 12.4) | Walk into a jungle; vines hang as green strands from kapoks and palms down to about 2 m. | `docs/screenshots/flora-2-1/jungle-compare.jpg` |
| 2 | **Fruit picked with F** (9.6) | Aim at a hanging fruit (not the trunk): "F PICK ..." shows; F takes them, the toast shows the fruit with its icon. Reload: the tree stays bare. | `fruit-pick-played-*.jpg` |
| 3 | **Ore drawn as ore** (12.12) | Dig into stone in the desert (ore lies just under its sand) or place a dug ore block: coal (black), copper (orange) and tin (steel grey) flecks. | `docs/screenshots/ore-veins/` |
| 4 | **Swamps, generator 12** (`swamps`) | New world: the map's biome layer (M, Biomes) shows about 7% swamp along coasts and rivers. Visit one: moss, pools that hold, mud, willows, reeds. | `docs/screenshots/swamps/` |
| 5 | **Beach acacias, generator 12** (12.13) | New world: acacias on the dry grass just behind warm beaches (none stand in the tide). | `flora-2-1/v12-beach-acacias.jpg` |
| 6 | **Punching trees for logs** (`tree-punch-shake` 1) | Bare hands on a tree: a log about every 2.4 s, up to 4, and the tree falls. A stone axe is about 4x faster. The first mission says "Punch a tree for logs". | none yet |
| 7 | **Dropped items lit like the ground** (`drop-lighting`, merged in #129) | Dig sand in a pit: the floating block is as dark as the pit, no glow. | `docs/screenshots/drop-lighting/` |
| 11 | **Trees shake when hit** (`tree-punch-shake` 2) | Chop or punch a tree: its crown sways about 12 cm and settles about 0.6 s after the last hit; neighbours stay still; its shadow should move with it (not yet seen in a capture). | `docs/screenshots/tree-punch-shake/` |
| 9 | **Tools that stopped digging** (`dig-fixes` 1) | Fell a tree with an axe or by hand, then stand on or beside where it stood and dig: the ground digs (before, the felled tree's invisible trunk took every swing). | none |
| 10 | **Bait by rivers and the sea** (`dig-fixes` 2) | Dig dirt, grass or sand within about 3 cells of a river or the sea: worms in about one dig in three; the "Dig for bait" mission counts them. | none |
| 12 | **Dry holes, and the sea flowing in** (`dry-holes`) | Dig a pit below sea level a few cells behind a beach: it stays dry (before, the sea's surface was drawn over it and you swam in it). Then cut a channel from the shore into it: the sea runs in along it, falls over the lip and fills the pit to sea level, not higher. Water poured from a bucket into a sealed pit stays ordinary water. Old saves: old dug holes open dry; old holes beside the sea flood at load as before. | `docs/screenshots/dry-holes/` |
| 8 | **Item toasts** (merged earlier) | Dig dirt or sand: "DIRT X1", "X2" stacking, icon, 3 s, then fade; up to 10, the rest paged. | `docs/screenshots/item-toasts/` |

Watch for anything that looks wrong at full speed on the GPU that a still on
lavapipe would not show: vine flicker, ore flecks shimmering at distance,
swamp pools popping, toasts stuttering.

## 3. Old saves must be unchanged

CLAUDE.md "Saved games survive every change". Open an existing world made
before this branch (version 10 or 11):
- the ground, biomes and trees are as they were (no new swamps, no moved
  acacias);
- dug holes and placed blocks are where they were;
- a tree you had taken a hand branch from now gives up to three more logs
  (expected: the punching rule).

## 4. Measurements only the desktop can take

None of these were taken in the cloud.

1. **Frame cost of the branch against main** (CLAUDE.md, `perf-measure`):
   `--release` build, `tools/perf_suite.py`, the old exe and the new
   interleaved in one sitting, nothing else running. The changes that can
   move frame time: the vines (near tree 708 -> 1068 vertices on a kapok),
   the ore flecks (shader), the swamp pools, the drop lighting. Report in
   `docs/benchmarks/2026-11-19-phase-2-1/`.
2. **Terrain build time with the swamp pool check** (`swamps` task 4.3): one
   river terrain test went from about 3 s to about 14 s on the busy cloud
   machine. Time a new version 12 world's terrain build near a river against
   version 11.
3. **The Mac's (and Windows') ground digests for generator versions 10 and
   11**: the pinned values were taken on Linux (`version_10_keeps_its_ground`,
   `version_11_keeps_its_ground` in `crates/pbd-core/src/planet_gen.rs`, gated
   off macOS). Run `cargo test -p pbd-core planet_gen` on the desktop and the
   Mac; if they differ by platform, add the platform's values as the version 9
   and 10 pins do.

## 5. Known failing tests (not from this branch's own work)

- `planet::water::water_junction_tests::where_waters_meet_the_surfaces_join_and_the_eye_follows_them`:
  fails the same way before the 2.1 work.
- `tests/new_world_start.rs`: the start's cloud cover is 0.528 against
  0.56 +/- 0.03; it came in with main's `river-downhill` merge (the same with
  the acacias switched off). Worth a look on the desktop.

## 6. Still in flight in the cloud

Nothing. Still open in `dry-holes`: the tide itself does not flow yet (task 4.1). At a tide high enough to reach over the rim of a beach pit, the sea's surface stops at the pit's edge. Also, each edit at the coast now does one more walk over the tier, about 1.2 ms on the cloud CPU; time it on the desktop (task 4.2).

## 7. Where things are

- Mockup tracker: https://claude.ai/artifact/VyvoqUcJ869F7Hi6C5dxuA (phase 2.1 shows built).
- Survey: https://claude.ai/artifact/HvNgkEFwnJkadM3Y3ELqi9 (F1 answered; all pictures restored).
- Write-ups: `openspec/changes/flora-v2`, `swamps`, `tree-punch-shake`,
  `drop-lighting`, `item-toasts`, `dry-holes`, `log-drives` (phase 3, planned only).
- Generator versions: 10 (flora-v2), 11 (`river-downhill`, main), 12
  (`swamps` + beach acacias, this branch). `mountain-ground` is planned as 13.
