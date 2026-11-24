#!/bin/bash
# The townsfolk in the game (`townsfolk` tasks 2.1, 3.2 and 10.4), from the
# spots `cargo test --profile fast -p pbd-app --lib -- --ignored
# print_where_to_stand_for_the_townsfolk_shots --nocapture` prints.
# Needs the fast build: cargo build --profile fast -p pbd-app
# `PBD_TOWNSFOLK_EXPRESSION=<id>` holds every face (a capture instrument).
# Usage: tools/capture_townsfolk.sh [out dir] [exe]
set -u
D=${1:-docs/screenshots/townsfolk}
cd "$(dirname "$0")/.."
EXE=${2:-target/fast/pbd-app}
mkdir -p "$D"
SMITH_FRONT="--walk --at 29.884943 1.485554 --yaw 17.1 --pitch -14"
SMITH_SIDE="--walk --at 29.860050 1.502882 --yaw -39.2 --pitch -8"
MARKET="--walk --at 29.702875 9.150705 --yaw 24.2 --pitch -6"
GUARD="--walk --at 29.112494 8.325739 --yaw -92.3 --pitch -8"
# The forge (design 26a): from 2 m past the anvil, looking at the forge.
FORGE="--walk --at 29.882999 1.454057 --yaw 179.2 --pitch -12"
# The idle look (task 2.2): the first idler whose spot 2.5 m in front is
# clear of the town, facing Linenleigh's stallholder (the printer checks it).
IDLE="--walk --at 29.654202 9.099745 --yaw -92.7 --pitch -6"
shot() {
  name=$1; expr=$2; shift 2
  PBD_TOWNSFOLK_EXPRESSION=$expr timeout 1500 xvfb-run -a -s "-screen 0 960x600x24" $EXE "$@" \
    --rain 0 --weather-at 7200 --capture "$D/$name.png" --frames 150 > "$D/$name.log" 2>&1
  echo "$name exit $?"
}
# Face to face with the first person at a post wearing each beard (design 36):
# Linenleigh's innkeeper (a goatee) and a stallholder (a full front beard),
# 2 m off so the head and shoulders show (design 36a).
GOATEE="--walk --at 29.726465 8.515085 --yaw -4.4 --pitch -6"
FULL_FRONT="--walk --at 29.707064 9.102589 --yaw 87.3 --pitch -6"
ONLY=${ONLY:-all}
# The motion (task 2.2) as capture sequences: every 4th frame of 48 from frame
# 150 (`PBD_CAPTURE_RUN`, a capture instrument), so a contact sheet shows
# the smith's hammer, the innkeeper's wipe and the market's walkers;
# tools/crop_sequence.py lays each run out as a sheet.
if [ "$ONLY" = motion ]; then
  run() {
    name=$1; shift
    PBD_CAPTURE_RUN="$D/motion-$name,150,48,4" timeout 2400 xvfb-run -a -s "-screen 0 960x600x24" \
      $EXE "$@" --rain 0 --weather-at 7200 --time 11 --capture "$D/motion-$name.png" --frames 200 \
      > "$D/motion-$name.log" 2>&1
    echo "motion-$name exit $?"
  }
  run hammer $SMITH_SIDE
  run wipe $GOATEE
  run walk $MARKET
  run idle $IDLE
  exit 0
fi
# The ears (design 38): the smith from the side, and the two bearded faces.
if [ "$ONLY" = ears ]; then
  shot ears-smith-side "" $SMITH_SIDE --time 11
  shot ears-smith-front "" $SMITH_FRONT --time 11
  shot ears-smith-side-2230 "" $SMITH_SIDE --time 22.5
  exit 0
fi
if [ "$ONLY" = beards ]; then
  shot beard-goatee "" $GOATEE --time 11
  shot beard-goatee-talking talking $GOATEE --time 11
  shot beard-full-front "" $FULL_FRONT --time 11
  shot beard-full-front-2230 "" $FULL_FRONT --time 22.5
  exit 0
fi
[ "$ONLY" = all ] && shot smith-face "" $SMITH_FRONT --time 11
[ "$ONLY" = all ] && shot smith-forge-2230 "" $FORGE --time 22.5
[ "$ONLY" = all ] && shot forge-day "" $FORGE --time 11
shot market-day "" $MARKET --time 11
shot gate-guard "" $GUARD --time 11
for e in surprised sleepy delighted worried cross thinking; do
  shot "face-$e" "$e" $SMITH_FRONT --time 11
done
