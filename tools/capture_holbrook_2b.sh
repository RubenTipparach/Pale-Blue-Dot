#!/bin/bash
# Holbrook's stairs and doors in the game (cities-in-the-world slice 2b), from
# the spots `cargo test -p pbd-app --lib -- --ignored print_where_to_stand
# --nocapture` prints. Beside each is the mockup's same view
# (tools/mockup_village_2b_shots.js). `--up 3.2` stands the walker on the upper
# floor; `--open-doors` opens every door without saving it. Doors start shut.
# Needs the fast build: cargo build -p pbd-app --profile fast
# Usage: tools/capture_holbrook_2b.sh <out dir>   (about 6 minutes a shot on lavapipe)
set -u
D=${1:-docs/screenshots/cities-in-the-world}
cd "$(dirname "$0")/.."
EXE=target/fast/pbd-app
for spec in \
  "game-2b-newel-below:--walk --at 29.63663 0.80156 --yaw -0.4 --pitch 20 --open-doors" \
  "game-2b-newel-above:--walk --at 29.63663 0.80156 --yaw -0.4 --pitch -35 --up 3.2 --open-doors" \
  "game-2b-flight-foot:--walk --at 29.63542 0.98162 --yaw 179.5 --pitch 20 --open-doors" \
  "game-2b-flight-landing:--walk --at 29.63465 1.08097 --yaw -0.5 --pitch -28 --up 3.2 --open-doors" \
  "game-2b-door-shut:--walk --at 29.71981 0.82332 --yaw 59.2" \
  "game-2b-door-open:--walk --at 29.71981 0.82332 --yaw 59.2 --open-doors"; do
  name=${spec%%:*}; args=${spec#*:}
  timeout 1100 xvfb-run -a -s "-screen 0 1440x900x24" $EXE $args --time 11 --rain 0 --weather-at 7200 --capture "$D/$name.png" --frames 150 > "$D/$name.log" 2>&1
  echo "$name exit $?"
done
