#!/bin/bash
# Furniture in the game (`furniture` task 3.8), from Linenleigh's market stall
# (the spot `tools/capture_town_missions.sh` uses): `PBD_FURNITURE_SHOT` (a
# capture instrument) stands the walker at the chair nearest a town table
# (`room`, by day and at night), sits on it (`seated`), stands at the foot of
# a bed a townsperson lies in at night (`asleep`), or opens the nearest town
# chest (`chest`); `--craft carpenter` sets down a carpenter's bench and opens
# its Furniture branch; O on the town tree shows the Carpentry branch.
# Needs the fast build: cargo build --profile fast -p pbd-app
# Usage: tools/capture_furniture.sh [out dir] [exe]
set -u
D=${1:-docs/screenshots/furniture}
cd "$(dirname "$0")/.."
EXE=${2:-target/fast/pbd-app}
mkdir -p "$D"
shot() {
  name=$1; keys=$2; shift 2
  PBD_KEY_SCRIPT="$keys" PBD_WINDOW=1200x750 timeout 2400 xvfb-run -a -s "-screen 0 1200x750x24" \
    "$EXE" --walk --hold-pointer --no-readout "$@" --rain 0 --weather-at 7200 --pitch -6 \
    --capture "$D/$name.png" > "$D/$name.log" 2>&1
  echo "$name exit $?"
}
STALL=${STALL:-"--at 29.208122 8.793697 --yaw 87.5"}
ONLY=${ONLY:-all}
on() { [ "$ONLY" = all ] || [[ ",$ONLY," == *",$1,"* ]]; }
on room && PBD_FURNITURE_SHOT=room:150 shot game-room-day "" $STALL --time 11 --frames 230
on night && PBD_FURNITURE_SHOT=room:150 shot game-room-night "" $STALL --time 22.5 --frames 230
on seated && PBD_FURNITURE_SHOT=seated:150 shot game-seated-night "" $STALL --time 21 --frames 230
on asleep && PBD_FURNITURE_SHOT=asleep:150 shot game-asleep-night "" $STALL --time 23.5 --frames 230
on chest && PBD_FURNITURE_SHOT=chest:150 shot game-chest-panel "" $STALL --time 11 --frames 230
on bench && shot game-carpenter-panel "" $STALL --time 11 --craft carpenter 3 --frames 230
on tree && shot game-tree-carpentry "120:KeyF,150:KeyF,170:KeyO" $STALL --time 11 --tree-town --tree-select carpenter --frames 220
for f in "$D"/game-*.png; do
  [ -f "$f" ] && convert "$f" -quality 88 "${f%.png}.jpg" && rm "$f"
done
true
