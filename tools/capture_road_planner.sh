#!/bin/bash
# The road planner in the game (`townsfolk` design 42 to 42l), by day and at
# 22:30. `--paving [metres] --road-plan <scene>` is the capture instrument:
# the roads go in through the same paths the planner's clicks take
# (`roads::paving::end_road`, `erase_road`), stakes in hand.
#   two     a road by two clicks, and an open start whose ghost follows the aim
#   stake   a second road started from the first road's middle stake
#   erased  that second road taken by the eraser (its stakes back)
#   wide    a three-wide road along the view (lanes red where it climbs)
# Each shot is a new world. Needs the fast build:
#   cargo build --profile fast -p pbd-app
# Usage: tools/capture_road_planner.sh <out dir> [exe]   (minutes a shot on lavapipe)
set -u
D=${1:-docs/screenshots/townsfolk}
cd "$(dirname "$0")/.."
EXE=${2:-target/fast/pbd-app}
mkdir -p "$D"
COMMON="--walk --spawn fields --weather-at 1800 --rain 0 --yaw 270 --hold-pointer --no-readout"
shot() {
  name=$1; shift
  timeout 2400 xvfb-run -a -s "-screen 0 1440x900x24" "$EXE" --world "planner-$name-$$" $COMMON "$@" \
    --capture "$D/$name.png" > "$D/$name.log" 2>&1
  echo "$name exit $?"
  [ -f "$D/$name.png" ] && convert "$D/$name.png" -quality 88 "$D/$name.jpg" && rm "$D/$name.png"
}
ONLY=${ONLY:-all}
# Every scene by day; the two-click road and the marks at 22:30 too (a shot
# takes about a quarter of an hour on lavapipe).
want() { [ "$ONLY" = all -o "$ONLY" = "$1" ]; }
for scene in two stake erased wide; do
  want "$scene" || continue
  shot "road-planner-$scene-1100" --frames 400 --paving 5 --road-plan "$scene" --pitch -24 --time 11
done
want two && shot road-planner-two-2230 --frames 400 --paving 5 --road-plan two --pitch -24 --time 22.5
# The mark's look close up (design 39a, task 13.4), by day and at 22:30.
want marks && {
  shot mark-close-1100 --frames 400 --paving 0.5 --pitch -32 --time 11
  shot mark-close-2230 --frames 400 --paving 0.5 --pitch -32 --time 22.5
}
true
