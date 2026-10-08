#!/bin/bash
# The compass bar and the map the right way round (`compass-bar`), from the
# default spawn on a new memory-only world. Needs the fast build:
#   cargo build -p pbd-app --profile fast
# `--yaw 0` faces compass WEST (the walker's default heading is Y x up), so
# `--yaw 180` faces east and `--yaw 90` north. The map before the change is
# cities-in-the-world/game-map-built-spawn.png, taken with the same flags: an
# older build cannot run here, since it reads this tree's assets at run time.
# Usage: tools/capture_compass.sh <out dir>   (6 to 15 minutes a shot on lavapipe)
set -u
D=${1:-docs/screenshots/compass-bar}
cd "$(dirname "$0")/.."
EXE=target/fast/pbd-app
shot() {
  local exe=$1 name=$2; shift 2
  timeout 1100 xvfb-run -a -s "-screen 0 1440x900x24" $exe "$@" --rain 0 --weather-at 7200 --capture "$D/$name.png" --frames 150 > "$D/$name.log" 2>&1
  echo "$name exit $?"
}
shot $EXE compass-sunrise --walk --time 7.6 --yaw 180 --pitch 6
shot $EXE compass-noon-north --walk --time 12 --yaw 90 --pitch 4
shot $EXE map-after --walk --time 12 --menu map --map-mpp 12
shot $EXE orbit-north-up --view column --at 28.64 0 --height 5000 --pitch -84 --yaw 90 --time 12
