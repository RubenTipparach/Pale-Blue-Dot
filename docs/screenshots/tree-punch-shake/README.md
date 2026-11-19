# A tree shakes when it is hit (`tree-punch-shake` task 2)

Taken 2026-11-19 in a cloud container with no GPU, on lavapipe (software
Vulkan) under Xvfb, 1440 x 900, the `fast` build, a new memory-only world
per run. They show how the shake looks, never how smoothly it runs: frame
cost was not measured. Logs: no `ERROR` lines; 31 to 33 `LATE_PIPELINE`
lines a run (a capture does not warm).

The same oak in the fields forest, looked up at from about 3 m so its crown
is in view. Three runs, identical but for the hands, so the wind sways
every tree the same at the same frame and only the hit differs:

| Shot | File |
| --- | --- |
| At rest: the axe in hand, no button held (frame 159) | `rest-f159.jpg` |
| Chopped with a stone axe, two frames after a hit (frame 159; the hit at 2.617 s, `TREE_SHAKE` in the log) | `axe-mid-f159.jpg` |
| Their difference: magenta where the two frames differ | `axe-diff-f159.jpg` |
| The crown, rest against chopped, 2x nearest | `crown-rest-vs-axe-2x.jpg` |
| At rest, frame 153 | `rest-f153.jpg` |
| Punched with bare hands, two frames after a hit (frame 153; the hit at 2.517 s) | `hand-mid-f153.jpg` |
| Their difference | `hand-diff-f153.jpg` |

What they show:

- Only the worked tree moves. The trees round it, the ground and the far
  canopy are the same pixels in both frames; the magenta besides the tree is
  the held axe or fist, the hotbar and the log toast.
- The crown moves most and the trunk's foot not at all: the shake grows
  with the square of the height over the base (0.12 m at the top, the
  leaves 1.3 times that), away from the eye with a little sideways.
- At the moment of the hit (frame 157 in the axe run) the two runs' frames
  differ only in the hand: the sway starts from rest.
- Hits come every 0.5 s while the button is held, and on each log: the axe
  run logged hits at 1.517, 2.017, 2.117 (log 1), 2.617, 2.883 (log 2) s;
  the hand run every 0.5 s from 1.517 s.

The settling (still after 0.6 s from the last hit, and never on another
tree) is pinned by `tree_shake::tests`. The sun's cascades are drawn with a
copy of the camera's planet uniform, so the shadow moves with the tree;
these frames look up into the crown and do not show it.

Launch lines (common: `PBD_WINDOW=1440x900 PBD_NO_CLOUDS=1
RUST_LOG=info,pbd_app::desktop::digging=debug PBD_CAPTURE_RUN=burst,150,32,1`
and `--walk --weather-at 1800 --time 10 --rain 0 --at 23.637 -0.7085
--pitch 40 --frames 182`):

    pbd-app --world tps-axe  ... --tool axe --break 10
    pbd-app --world tps-rest ... --tool axe
    pbd-app --world tps-hand ... --tool hand --break 10

The difference images mark a pixel whose largest channel moved by more than
12 over a 0.35 grey of the first frame.
