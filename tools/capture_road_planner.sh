#!/bin/bash
# The road planner in the game (`townsfolk` design 42 to 42l), by day and at
# 22:30. `--paving [metres] --road-plan <scene>` is the capture instrument:
# the roads go in through the same paths the planner's clicks take
# (`roads::paving::end_road`, `erase_road`), stakes in hand.
#   two     a road by two clicks, and an open start whose ghost follows the aim
#   stake   a second road started from the first road's middle stake
#   erased  that second road taken by the eraser (its stakes back)
#   wide    a three-wide road open from up the view to the aim (its ghost: lanes red where the ground is off its height)
#   lamps   a three-wide dirt road with lamps, paved, its street lanterns built both sides (design 42m)
#   slope   a gravel road up a slope, two wide where the planner takes it, else one, paved whole
#   bridge  a gravel road over a river on a plank bridge, paved whole
#   ghost-lamps   the lamps road open in the planner, nothing staked: its ghost's lamp posts (design 42o)
#   ghost-bridge  the bridge road open in the planner: its ghost's deck, posts and rails (design 42o)
# With stakes in hand the view is lifted (design 42n, the owner's survey P2).
# Each shot is a new world. Needs the fast build:
#   cargo build --profile fast -p pbd-app
# Usage: tools/capture_road_planner.sh <out dir> [exe]   (minutes a shot on lavapipe)
set -u
D=${1:-docs/screenshots/townsfolk}
cd "$(dirname "$0")/.."
EXE=${2:-target/fast/pbd-app}
mkdir -p "$D"
COMMON="--walk --spawn fields --weather-at 1800 --rain 0 --yaw 270 --hold-pointer --no-readout"
# The slope is found from the `fields` spawn, where the dunes climb and no
# tree stands in the way (the forest by the world's first climb near the
# spawn has no stretch clear of trees); the bridge
# 20 m back from the river under `docs/screenshots/roads`' bridge spot
# (14.32551 59.25625). Each stakes its own road from a fan of spots ahead,
# clear of the world's road.
SLOPE="--spawn fields --yaw 270"
BRIDGE="--at 14.44955 59.46562 --yaw -31.4"
AT="--walk --weather-at 1800 --rain 0 --hold-pointer --no-readout"
shot() {
  name=$1; shift
  base=$COMMON
  case "$*" in *--at*) base=$AT ;; esac
  case "$*" in *--view*) base="--weather-at 1800 --rain 0 --hold-pointer --no-readout" ;; esac
  timeout 2400 xvfb-run -a -s "-screen 0 1440x900x24" "$EXE" --world "planner-$name-$$" $base "$@" \
    --capture "$D/$name.png" > "$D/$name.log" 2>&1
  echo "$name exit $?"
  [ -f "$D/$name.png" ] && convert "$D/$name.png" -quality 88 "$D/$name.jpg" && rm "$D/$name.png"
}
ONLY=${ONLY:-all}
# Every scene by day; the two-click road and the marks at 22:30 too (a shot
# takes about a quarter of an hour on lavapipe).
want() { [ "$ONLY" = all -o "$ONLY" = "$1" ]; }
for scene in two stake erased wide lamps; do
  want "$scene" || continue
  shot "road-planner-$scene-1100" --frames 400 --paving 5 --road-plan "$scene" --pitch -24 --time 11
done
# The slope and the bridge in two runs: the first finds the road from the
# spot and logs where to stand ("scripted road view: --at ... --road-plan
# <scene>:<a>:<b>:<width>") and is stopped; the second stakes that road and
# is shot from its first stake's cell, facing along it.
# `how` is `view` (on foot on the first cell) or `overview` (from 10 m
# over the ground 14 m back, `--view column`).
road_shot() {
  name=$1; scene=$2; pitch=$3; how=$4; shift 4
  log="$D/$name-find.log"
  timeout 900 xvfb-run -a -s "-screen 0 640x400x24" "$EXE" --world "find-$name-$$" $AT "$@" \
    --frames 400 --paving 2 --road-plan "$scene" --time 11 --capture "$D/$name-find.png" > "$log" 2>&1 &
  until grep -aq "scripted road view\|road in the fan" "$log" 2>/dev/null || ! kill -0 $! 2>/dev/null; do sleep 5; done
  pkill -f "find-$name-$$"
  view=$(grep -ao "scripted road $how: [^[:cntrl:]]*" "$log" | head -1 | sed "s/.*$how: //")
  rm -f "$log" "$D/$name-find.png"
  [ -n "$view" ] || { echo "$name: no road found"; return; }
  # A ghost shot leaves the found road open in the planner instead.
  [ -n "${GHOST:-}" ] && view=${view/--road-plan /--road-plan ghost-}
  echo "$name: $view"
  if [ "$how" = overview ]; then
    shot "$name" --view column --height 10 --frames 400 --paving 2 --pitch "$pitch" --time 11 $view
  else
    shot "$name" --frames 400 --paving 2 --pitch "$pitch" --time 11 $view
  fi
}
want slope && road_shot road-slope slope -2 view $SLOPE
want bridge && road_shot road-bridge bridge -28 overview $BRIDGE
want ghost-bridge && GHOST=1 road_shot road-planner-ghost-bridge-1100 bridge -20 view $BRIDGE
want ghost-lamps && shot road-planner-ghost-lamps-1100 --frames 400 --paving 5 --road-plan ghost-lamps --pitch -24 --time 11
want lamps && shot road-planner-lamps-2230 --frames 400 --paving 5 --road-plan lamps --pitch -24 --time 22.5
# want two && shot road-planner-two-2230 --frames 400 --paving 5 --road-plan two --pitch -24 --time 22.5
# The mark's look close up (design 39a, task 13.4), by day and at 22:30.
want marks && {
  shot mark-close-1100 --frames 400 --paving 0.5 --pitch -32 --time 11
  shot mark-close-2230 --frames 400 --paving 0.5 --pitch -32 --time 22.5
}
true
