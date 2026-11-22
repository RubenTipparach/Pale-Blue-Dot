#!/bin/bash
# The roads between towns in the game (`townsfolk` 8.2 and 8.4), from the
# spots a new world's plan logs ("road A to B ...: middle --at ... --yaw ...;
# first climb ...; first bridge ..."). Each run is a new world, so it plans
# the roads again in its loading thread, as an old save's first open does.
# Needs the fast build: cargo build -p pbd-app --profile fast
# Usage: tools/capture_roads.sh <out dir> [exe]   (minutes a shot on lavapipe)
set -u
D=${1:-docs/screenshots/roads}
cd "$(dirname "$0")/.."
EXE=${2:-target/fast/pbd-app}
shot() {
  name=$1; shift
  timeout 1500 xvfb-run -a -s "-screen 0 1440x900x24" "$EXE" "$@" --capture "$D/$name.png" > "$D/$name.log" 2>&1
  echo "$name exit $?"
}
# Set the spots from a first run's log, then run the rest.
[ -n "${ROAD:-}" ] && shot game-road-between-towns --walk $ROAD --time 11 --rain 0 --frames 150
[ -n "${CLIMB:-}" ] && shot game-road-climb --walk $CLIMB --pitch 8 --time 11 --rain 0 --frames 150
[ -n "${CLIMB:-}" ] && shot game-road-climb-night --walk $CLIMB --pitch 8 --time 22.5 --rain 0 --frames 150
[ -n "${BRIDGE:-}" ] && shot game-road-bridge --walk $BRIDGE --time 11 --rain 0 --frames 150
[ -n "${OVER:-}" ] && shot game-road-overview --view column $OVER --height 60 --pitch -42 --time 11 --rain 0 --frames 150
true
