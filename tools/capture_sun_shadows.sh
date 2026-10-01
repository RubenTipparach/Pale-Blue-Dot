#!/bin/bash
# The sun-shadows views (openspec/changes/sun-shadows, tasks 1.1 and 7.1):
# Holbrook's rooms, a house front and the lane, and the town from above, each
# at several hours, so the light is judged across the day and not at one sun.
# The spots are the slice 2b ones (tools/capture_holbrook_2b.sh). Every door
# stands open, so a room's light is the light through its door.
# Needs the fast build: cargo build -p pbd-app --profile fast
# Usage: tools/capture_sun_shadows.sh <out dir> <prefix> [hours] [views]
#   hours default "8 11 14 17.5"; views default "newel flight front lane above"
#   (about 6 minutes a shot on lavapipe; logs go beside the shots as .log)
set -u
D=${1:-docs/screenshots/sun-shadows}
P=${2:-before}
HOURS=${3:-"8 11 14 17.5"}
VIEWS=${4:-"newel flight front lane above"}
# More flags for every shot, e.g. EXTRA=--no-shadows; and the weather's
# offset (--weather-at 7200 is dry at 08:00 to 14:00 over Holbrook, 3600 at
# 11:00 and 22:30: towns::tests::print_the_rain_over_holbrook).
EXTRA=${EXTRA:-}
WEATHER=${WEATHER:-7200}
cd "$(dirname "$0")/.."
mkdir -p "$D"
EXE=target/fast/pbd-app
view_args() {
  case $1 in
    newel) echo "--walk --at 29.63663 0.80156 --yaw -0.4 --pitch 20" ;;
    upstairs) echo "--walk --at 29.63663 0.80156 --yaw -0.4 --pitch -35 --up 3.2" ;;
    flight) echo "--walk --at 29.63542 0.98162 --yaw 179.5 --pitch 20" ;;
    landing) echo "--walk --at 29.63465 1.08097 --yaw -0.5 --pitch -28 --up 3.2" ;;
    front) echo "--walk --at 29.71981 0.82332 --yaw 59.2" ;;
    inside) echo "--walk --at 29.66752 0.78743 --yaw -120.8" ;;
    lane) echo "--walk --at 29.69673 1.19287 --yaw 179.4" ;;
    above) echo "--view column --at 29.70 2.18 --height 60 --pitch -42" ;;
    *) echo "unknown view $1" >&2; return 1 ;;
  esac
}
for hour in $HOURS; do
  for view in $VIEWS; do
    args=$(view_args "$view") || continue
    name="$P-$view-$(awk -v h="$hour" 'BEGIN { printf "%02dh%02d", int(h), (h - int(h)) * 60 }')"
    timeout 1100 xvfb-run -a -s "-screen 0 1440x900x24" $EXE $args --open-doors $EXTRA \
      --time "$hour" --rain 0 --weather-at "$WEATHER" --capture "$D/$name.png" --frames 150 \
      > "$D/$name.log" 2>&1
    echo "$name exit $?"
  done
done
