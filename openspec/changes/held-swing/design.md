# Design

## Measured (2026-11-17)

- `held::hold` turns the tool by `phase.sin()` while `Swinging` is true;
  `phase` grows at `swing_rate` 9 rad/s, so a cycle is 0.70 s. The strike
  reaches the block in 0.175 s, the back-lift is about 10 degrees. A sine is
  fastest when it leaves and slowest when it lands: that is the weak look.
- `Swinging` follows `mining.progress().is_some()`; there is no per-hit timer.
  The stone axe takes 0.6 s on wood, so the block breaks before one swing ends.
- Dig sounds play at the start and each third of progress
  (`desktop/sounds.rs`), unrelated to the swing.
- On stop the phase resets to 0: the tool snaps to rest.

## Decisions

1. A stroke curve in `held.ron` replacing the three `swing_*` values:
   `windup_s` 0.08 / `windup_rad` 0.35, `strike_s` 0.07 / `strike_rad` 1.15
   (eased in, t cubed), `impact_hold_s` 0.05, `recoil_rad` 0.12,
   `recover_s` 0.18, `stroke_target_s` 0.4, `drop_m` 0.09, and
   `camera_kick_rad` (0 unless the owner wants it).
2. One function, `stroke_at(progress, break_s) -> (stroke, phase)`, gives the
   stroke and its stage from the dig's progress. `hold` draws from it and the
   sound code plays on its impacts: one schedule, two readers.
3. The hold on impact holds the tool only. Game time and the dig run on.
4. The mockup (Tools in Hand, M36, `docs/mockups/tool-motion.html`) shows
   TODAY and STRIKE in each of its Dig and Chop tabs' one view (a toggle),
   with the curve's values as sliders.
5. One steady beat from block to block (owner, 2026-11-17, on the Tools in
   Hand mockup: "why is it in sets of 3?", "it should be like continous").
   Timing each block's N strokes at progress k/N left the 0.15 s
   `between_s` pause, plus a whole stroke, between a block's last impact and
   the next block's first: three hits, a gap, three hits. Instead a block and
   its pause, L = T + between_s, take N = max(1, round(L / stroke_target_s))
   strokes of one period P = L / N. The k-th impact of cycle c lands at
   c·L + T − (N − k)·P, so the last one still lands on the break, and the
   strokes run on at one beat while the button is held. Break times do not
   change.
6. The arm reaches (owner, 2026-11-17: "I think hand should extend outward
   when doing actions", pointing at a first-person chopping animation,
   https://sketchfab.com/3d-models/chopping-animation-8f4e1c3d4ec74e7580eae93d045e3581).
   Pitching the tool about a fixed fist reads as a wrist flick. The fist
   moves too: on the wind-up it draws back toward the eye and up
   (`windup_pull_m`, 0.06 back and 0.03 up). On the strike it drives forward
   along the view toward the target (`reach_m`, 0.16, eased in with the
   strike's t³) and in toward the crosshair (`reach_in_m`, 0.05). It holds
   through the impact and returns with the recover. The sleeve is long enough
   that the arm still runs off the screen at full reach. Shown in the Tools
   in Hand mockup's Proposed strokes before the game.

