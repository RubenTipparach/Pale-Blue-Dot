#!/bin/bash
# Marking and paving (`townsfolk` design 39 to 44): the grinder and the
# mixer either side of the view, and a plan staked along it, its nearer
# half paved, by day and at 22:30. `--paving [metres]` is the capture
# instrument: the stations and the stakes go in through the same paths a
# player's place button, stake and shovel take. Each shot is a new world.
# Needs the fast build: cargo build -p pbd-app --profile fast
# Usage: tools/capture_paving.sh <out dir> [exe]   (minutes a shot on lavapipe)
set -u
D=${1:-docs/screenshots/paving}
cd "$(dirname "$0")/.."
EXE=${2:-target/fast/pbd-app}
mkdir -p "$D"
COMMON="--walk --spawn fields --weather-at 1800 --rain 0 --yaw 270"
shot() {
  name=$1; shift
  timeout 2400 xvfb-run -a -s "-screen 0 1440x900x24" "$EXE" --world "paving-$name-$$" $COMMON "$@" --capture "$D/$name.png" > "$D/$name.log" 2>&1
  echo "$name exit $?"
}
# The road pieces and pegs are stood after the capture's first frames;
# 400 frames leaves their pipelines time to build (a capture does not warm).
shot paving-day --frames 400 --paving 5 --pitch -20 --time 11
shot paving-2230 --frames 400 --paving 5 --pitch -20 --time 22.5
shot marks-close --frames 400 --paving 0.5 --pitch -25 --time 11
shot stations-day --frames 400 --paving 7 --pitch -2 --time 11
shot stations-2230 --frames 400 --paving 7 --pitch -2 --time 22.5
true
