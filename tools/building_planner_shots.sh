#!/usr/bin/env bash
# The building planner's in-game captures (`building-planner` task 3.7):
# headless `--capture` shots through `PBD_PLAN_SHOT` (desktop/construction.rs),
# into docs/screenshots/building-planner/game-*.png. In a cloud container they
# run on lavapipe under xvfb: they show the look, never the frame time.
#
#   tools/building_planner_shots.sh [exe] [out dir]
set -euo pipefail
EXE=${1:-target/fast/pbd-app}
OUT=${2:-docs/screenshots/building-planner}
mkdir -p "$OUT"
shot() {
  local name=$1 plan=$2
  shift 2
  if [ -n "${ONLY:-}" ] && [[ $name != $ONLY ]]; then return; fi
  PBD_PLAN_SHOT="$plan" PBD_WINDOW=1200x750 xvfb-run -a -s "-screen 0 1200x750x24" \
    "$EXE" --capture "$OUT/$name.png" --walk --rain 0 --weather-at 7200 "$@" \
    > "$OUT/$name.log" 2>&1 || echo "$name failed"
  grep -E "PBD_PLAN_SHOT|PLANNER|SITE|panicked" "$OUT/$name.log" || true
  [ -n "${KEEP_LOGS:-}" ] || rm -f "$OUT/$name.log"
}
# The planner: the cottage top down; the two-storey house in isometric,
# its walls cut away; a storey over nothing, its check in red.
shot game-planner-top "planner:cottage,at:60" --time 11 --frames 240
shot game-planner-iso "planner:house,iso,storey:1,tool:2,at:60" --time 11 --frames 240
shot game-planner-checks "planner:cottage,upstairs,storey:1,at:60" --time 11 --frames 240
# The drafting table, set down ahead.
shot game-drafting-table "table,ahead:3,at:150" --time 11 --pitch -25 --frames 320
# The ground before and after a site is set out, from above.
shot game-site-before "view,dist:16,tilt:50,ahead:10,at:150" --time 11 --frames 340
shot game-site-set-out "give,site:cottage,view,dist:16,tilt:50,ahead:10,at:150" --time 11 --frames 340
# Half built, then the two-storey house with its scaffolding, then a
# change marking a built room for taking down; and the site at night.
shot game-site-half "give,site:cottage,build:14,view,dist:16,tilt:45,ahead:10,at:150" --time 11 --frames 340
shot game-site-scaffold "give,site:house,build:40,view,dist:20,tilt:35,ahead:10,at:150" --time 11 --frames 340
shot game-site-change "give,site:cottage,build:22,change,view,dist:16,tilt:45,ahead:10,at:150" --time 11 --frames 340
shot game-site-night "give,site:house,build:40,view,dist:20,tilt:35,ahead:10,at:150" --time 22 --frames 340
