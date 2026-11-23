#!/bin/bash
# The town tree in the game (`town-missions-tree` task 3.1): the walker at
# Linenleigh's market stall (the spot `tools/capture_talk.sh` uses), so the
# walk into town opens the tree; F talks to the merchant (one person met)
# and F again closes the talk; O opens the Missions tab on the town tree
# (`--tree-town`). `--town-focus meet` makes "Meet the townsfolk" the HUD's
# mission, as a pick on the tree would. Nothing else is set.
# Needs the fast build: cargo build --profile fast -p pbd-app
# Usage: tools/capture_town_missions.sh [out dir] [exe]
set -u
D=${1:-docs/screenshots/town-missions-tree}
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
SMITH=${SMITH:-"--at 29.521185 8.849977 --yaw 87.4"}
ONLY=${ONLY:-all}
[ "$ONLY" = all -o "$ONLY" = tree ] && shot game-tree "120:KeyF,150:KeyF,170:KeyO" $STALL --time 11 --tree-town --frames 220
[ "$ONLY" = all -o "$ONLY" = panel ] && shot game-panel "120:KeyF,150:KeyF,170:KeyO" $STALL --time 11 --tree-town --tree-select first-trade --frames 220
[ "$ONLY" = all -o "$ONLY" = hud ] && shot game-hud "120:KeyF,150:KeyF" $STALL --time 11 --town-focus meet --frames 200
[ "$ONLY" = all -o "$ONLY" = night ] && shot game-tree-night "120:KeyF,150:KeyF,170:KeyO" $STALL --time 22.5 --tree-town --frames 220
# The board at dusk (design 8 to 10): the lamps job posted from 17:00.
[ "$ONLY" = all -o "$ONLY" = board ] && shot game-board-dusk "120:KeyF,140:press:work" $SMITH --time 18 --frames 190
for f in "$D"/game-*.png; do
  [ -f "$f" ] && convert "$f" -quality 88 "${f%.png}.jpg" && rm "$f"
done
true
