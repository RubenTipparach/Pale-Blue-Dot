#!/bin/bash
# Holbrook in the game from the spots the towns mockup's village shots are
# taken from (docs/screenshots/cities-in-the-world/README.md). The spots come
# from `cargo test -p pbd-app --lib -- --ignored print_where_to_stand --nocapture`.
# Needs the fast build: cargo build -p pbd-app --profile fast
# `--weather-at 7200` runs the weather two hours past 11:00 without moving the
# clock: it rains over Holbrook at 11:00 on a new world's first days, and is
# dry two hours on (`--ignored print_the_rain_over_holbrook`). `--rain 0`
# alone forces no storm but leaves the weather's own rain.
# `--open-doors` opens the doors, as the mockup's shots do; doors start shut.
# Usage: tools/capture_holbrook.sh <out dir>   (about 6 minutes a shot on lavapipe)
set -u
D=${1:-docs/screenshots/cities-in-the-world}
cd "$(dirname "$0")/.."
EXE=target/fast/pbd-app
for spec in \
  "game-lane:--walk --at 29.69673 1.19287 --yaw 179.4" \
  "game-outside:--walk --at 29.71981 0.82332 --yaw 59.2" \
  "game-inside:--walk --at 29.66752 0.78743 --yaw -120.8" \
  "game-overview:--view column --at 29.70 2.18 --height 60 --pitch -42"; do
  name=${spec%%:*}; args=${spec#*:}
  timeout 1100 xvfb-run -a -s "-screen 0 1440x900x24" $EXE $args --time 11 --rain 0 --weather-at 7200 --open-doors --capture "$D/$name.png" --frames 150 > "$D/$name.log" 2>&1
  echo "$name exit $?"
done
