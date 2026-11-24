# GPU validation of `claude/phase-2-1` on real hardware (2026-11-22)

- **Machine:** the owner's MacBook Pro, Apple M4 Max, Metal (a real GPU, not lavapipe).
- **Build:** `51c03209` (branch `claude/phase-2-1`, contains `0d46c480`),
  `cargo build --profile fast -p pbd-app`: the **fast** profile, not release.
- **Captures:** `--capture` fixed-step shots. Townsfolk and beards at 960 x 600, the rest at 1200 x 750
  (logical points; the Retina window renders at 2x). Downscaled to 1280 wide. The `*-close.png` files are
  crops of the full 2x frame.
- **Weather:** `PBD_WEATHER_SCRIPT=1:dry`. **Note:** `0:dry` (the frame the brief suggested) never fires,
  because the script's first sampled frame is 1. The first night shots taken with `0:dry` were raining
  (rain 0.54, ripples on the ground). They were retaken.
- **Worlds:** memory worlds, plus one throwaway world `gpuval-house` (deleted afterwards). A launch with no
  `--world` opens the most recent save, so the later runs opened `gpuval-house`. None of the owner's
  saves was touched (the worktree has its own `saves/`).

## Shots

| Shot | What it shows | What is wrong / notes |
| --- | --- | --- |
| `log-drives-log-drifting.png` | From the rower's seat, anchored on the desert river (`--aboard raft --at 23.644 -2.685 --load-hold`, keys `60:KeyT,90:KeyU`). Log 1 has gone over the side and is drifting behind the hold box. | Water is drawn inside the raft between the deck planks (dark swirls in the gaps). The hold is a plain box on the deck. |
| (no PNG) log at rest | **The drift finishes on the GPU.** `LOG 1 Floating -> Resting at 59.0 m` came about **34 s** (one run) and **50 s** (another) of wall time after `LOG 1 unloaded`, in a 3,600-frame capture of about 60-80 s. On lavapipe it had not finished after 25 minutes. | Not photographed: the seated and chase cameras ignore `--yaw`, and the log came to rest out of view. The anchored raft still moved about 8 m and turned about 180° over those frames (`23.625 -2.684` to `23.577 -2.604`). |
| `build-red-ghost-roof.png` | Roof slot (`PBD_BUILD_SHOT=mode:4,at:100`, pitch -15). The line under the crosshair reads WALLS MUST CLOSE ROUND THE ROOM, and the red roof ghost shows along the top edge. | **The red ghost cannot be got wholly in frame** on foot. The aim must land within `REACH_M` (7 m), and the roof ghost sits a storey above that cell. At pitch -13 and shallower the aim finds no cell (no ghost, no text). At -17 and steeper the ghost leaves the top of the frame and only the text shows. I tried yaws 0/90/180/330 on the terraces. A player aiming a roof sees the same thing. Wall and floor ghosts next to a placed house came out green or filled the screen. |
| `build-house-footing-terraces.png` | The Timber house (`choice:2,house:999,ahead:18`) on the desert terraces, standing on its banded fieldstone footing down to the lower ground. | Footing works (6.1). |
| `build-house-whole-1600.png` (16:00) / `build-house-whole-0800.png` (08:00) | The whole two-storey house with its gable roof, 28 m off. | — |
| `build-house-shadow-1630.png` | 16:30, house 12 m ahead. **The house casts the sun's shadow**: the sand in front is in its shadow, with the shadow's edge across the lit sand to either side (6.2 works). | **A cactus stands inside the house** and shows through the ground-floor window. Placing the blueprint does not clear the flora in its cells. |
| `build-house-night-2230.png` | The house at 22:30: its lanterns light the windows warm. | — |
| `townsfolk-smith-face.png` | Holingmouth's smith face to face, 11:00. | Nose: a small faceted solid stuck on the flat face. Hair: a slab wider than the head, set on top like a lid. Beard: stepped blocks proud of the face, see the verdict. |
| `townsfolk-smith-2230.png` (+ `-close`) | The smith at 22:30, lit by the town's lamps and the field. | **No forge.** No town template has a smithy or forge fire (`townsfolk` design 26), so there is no "smith at a forge at night" to shoot. |
| `townsfolk-market-day.png` / `townsfolk-market-2230.png` | Linenleigh's street by the stalls, day and night. The lamps light the street well at night. | A thin "crowd": one stallholder close, about four people further off. On the right two townsfolk overlap: the one in red stands half inside the other. |
| `townsfolk-talk.png` | F at Linenleigh's merchant: MERCHANT / WANT TO TRADE? with TRADE and BYE. | The panel covers the top of the merchant's head. The spot is dim, under a roof. |
| `townsfolk-trade.png` | TRADE pressed: nine goods with icons in slot wells, prices, BUY, PURSE 0, BACK. SELL reads NOTHING THEY WANT. | — |
| `townsfolk-job-board.png` | WORK at the reeve: three jobs with rewards and TAKE. | — |
| `roads-between-towns.png` | The dirt road from Holbrook to Linenleigh through the forest, trees cleared, no tufts on it. | — |
| `roads-climb-day.png` / `roads-lamps-2230.png` | The road leaving Holbrook up the hill. At 22:30 a lantern post stands beside the road and warms it. | The sloped cells read as raised slabs with vertical side faces where the road climbs a layer. |
| `roads-bridge-deck.png` / `roads-bridge-deck-2230.png` | From the bank end, on the deck of the timber bridge (`--at 14.34476 59.28880 --yaw 20`): plank deck and board railings across the river. | No lamps on it at night; by design 34 they are only within 90 m of a town. The deck is very dark at 22:30. |
| `beard-goatee.png`, `beard-goatee-talking.png` (+ `-close`) | Linenleigh's innkeeper (goatee) face to face, 11:00 and `PBD_TOWNSFOLK_EXPRESSION=talking`. | See the verdict. |
| `beard-full-front.png`, `beard-full-front-2230.png` (+ `-close`) | The stallholder's full front beard, 11:00 and 22:30. | See the verdict. |

## Primitives verdict (CLAUDE.md "No obvious primitives")

Yes, some parts still read as primitives stuck on:

- **Hats and hair: yes.** The stallholder's cap and the smith's hair are flat slabs wider than the head,
  set on it like a lid. The stallholder's cap also has a small box knob on top. The market stallholder
  has a blond block on top of the head with a side tuft.
- **Noses: yes.** The smith's and the merchant's noses are small faceted solids on the flat face.
- **Ears: mildly.** They are small flat slabs on the sides of the head. They are not proud boxes, but they
  are not shaped into the head either.
- **Hands: yes.** Faceted ball fists. The innkeeper holds a grey faceted ball.
- **Full front beard: mostly one piece.** It is a soft-shaded frame round the face and jaw that follows
  the head. The mouth sits in a hard-edged darker rectangle cut into it, which reads as a window in a
  panel.
- **Goatee: mostly painted.** It is a stepped dark shape on the face, with small grey nubs at the jaw
  corners standing proud of the face edge.
- **Smith's red beard: yes, it reads as blocks.** It is stepped blocks round the mouth and chin.
- **Props: yes.** The raft's hold is a plain box on the deck.

## Playtest (real-time window, not `--capture`)

All runs were 1440 x 900 logical (2880 x 1800 Retina), fast profile, `--frame-graph`, with a frame log.
Stats count frames from `READY` + 120.

| Run | What | Wall p50 / p99 (max) | cpu_main p50 / p99 |
| --- | --- | --- | --- |
| `town-walk` | Linenleigh, 11:00, `--walk-distance 350` through the town and out (6 stall turns) | 16.9 / 38.6 ms (137.6 at exit) | 4.0 / 23.8 ms |
| town section only (first 17 s) | in the town | 16.0 / 35.6 ms | 4.8 / 20.9 ms |
| `road-walk` | Holbrook-Linenleigh road, middle, 11:00, `--walk-distance 700`. Straight 291 m, then it left the road and got stuck in terrain for about 3.5 min (33 stall turns, digging) | 20.3 / 43.4 ms overall | 4.4 / 24.8 ms |
| on the road (first 40 s) | on the road | 17.0 / 39.0 ms | 3.9 / 22.9 ms |
| `town-night-walk` | Linenleigh at 22:30, 300 m | 21.3 / 41.2 ms (129.5 at exit) | 3.7 / 23.5 ms |
| `market-stand` | Standing at the merchant, 150 s, key script F / trade / Esc / N / N / M / M | flat **30.0** ms | 3.3 ms |

That is about 10.5 minutes of real-time play in all.

- **`LATE_PIPELINE`: none** in any real-time run. `READY` came at 20.7-21.3 s with 107 pipelines. The
  capture runs log about 36 `LATE_PIPELINE` lines (the rain volume, the weather overlay and others), as
  expected, because captures skip the warm-up.
- **Hitches:** the walks have about 1-3% of frames over 33 ms, with cpu_main at 25-37 ms in them. These
  look like streaming and rebuild hitches as the walker moves, roughly every few seconds while walking.
- **Standing still in the market ran at a flat 30.0 ms** (33 fps) while cpu_main was 3.3 ms. It looks
  like a pacing cap, not load: maybe the window was unfocused, or the `--hold-pointer` run was
  throttled. It is worth checking.
- **The talk key did not open the talk panel in real time:** `PBD_KEY_SCRIPT: no trade button by frame 1760`.
  The same F worked in the capture.
- **Broken:** `--view column` with no `--walk` panics at once:
  `Parameter ... failed validation: Resource does not exist` (bevy_ecs, a system parameter). It happens
  with or without `--world`. `tools/capture_roads.sh`'s overview shot uses this view.
- At `READY` each run logs a dozen `Could not despawn entity ... invalid` warnings. They are harmless but
  noisy.
- Frame cost here is the fast profile on an M4 Max; it is not a release number. No `perf_suite` was run.
