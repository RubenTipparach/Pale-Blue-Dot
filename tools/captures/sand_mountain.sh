#!/usr/bin/env bash
# The headless captures for `sand-tiles` and `mountain-ground` (2026-11-20):
# a cloud container on lavapipe (no GPU), the fast build, 1600 x 1000.
# Each shot is a fresh run of a new memory-only world. Run from the repo root:
#   bash tools/captures/sand_mountain.sh [shot-name ...]
set -u
EXE=${EXE:-target/fast/pbd-app}
SAND=docs/screenshots/sand-game
MOUNT=docs/screenshots/mountain-game
mkdir -p "$SAND" "$MOUNT"
shot() {
  local out=$1; shift
  echo "== $out: $*"
  PBD_WINDOW=${WIN:-1600x1000} timeout 1500 xvfb-run -a -s "-screen 0 ${WIN:-1600x1000}x24" \
    "$EXE" --frames 150 "$@" --capture "$out" > "${out%.png}.log" 2>&1
  grep -E "LATE_PIPELINE|panicked" "${out%.png}.log" | head -5
}
want() { [ $# -eq 0 ] && return 0; for w in "${ALL[@]}"; do [ "$w" = "$1" ] && return 0; done; return 1; }
ALL=("$@")
run() { local name=$1; shift; if [ ${#ALL[@]} -eq 0 ] || want "$name"; then shot "$@"; fi; }

# Sand: a beach and a desert by day, and a row of placed sand and desert sand.
run beach   "$SAND/beach-day.png"   --walk --at 36.8594 58.2286 --time 11 --pitch -30 --yaw 0
run desert  "$SAND/desert-day.png"  --walk --at 36.8675 0.3294 --time 11 --pitch -30 --yaw 0
run placed  "$SAND/placed-desert-sand.png" --walk --at 36.8675 0.3294 --time 11 --pitch -20 --yaw 0 \
  --place-row "sand,desert sand,sand,desert sand" 4

# Mountains: the same flank on version 14 (today's ground, with the draw-time
# looks every world gets) and version 15, from the air and on foot.
run air14   "$MOUNT/flank-air-v14.png" --generator 14 --fly --height 60 --at 31.30 -8.70 --time 11 --rain 0 --pitch -30 --yaw 60
run air15   "$MOUNT/flank-air-v15.png" --fly --height 60 --at 31.30 -8.70 --time 11 --rain 0 --pitch -30 --yaw 60
run foot14  "$MOUNT/flank-foot-v14.png" --generator 14 --walk --at 31.4450 -8.4220 --time 11 --rain 0 --pitch -8 --yaw 200
run foot15  "$MOUNT/flank-foot-v15.png" --walk --at 31.4450 -8.4220 --time 11 --rain 0 --pitch -8 --yaw 200
run meadow  "$MOUNT/meadow-close.png" --walk --at 31.2776 -8.4563 --time 10 --rain 0 --pitch -40 --yaw 30
run line    "$MOUNT/tree-line.png" --walk --at 31.2656 -8.8345 --time 15 --pitch -4 --yaw 0
run snow    "$MOUNT/snow-line.png" --walk --at 30.5872 -9.3497 --time 12 --pitch -20 --yaw 90
run scree   "$MOUNT/scree.png" --walk --at 28.5972 -11.7858 --time 12 --rain 0 --pitch -30 --yaw 0
run night   "$MOUNT/night-lamp.png" --walk --at 31.2776 -8.4563 --time 22 --pitch -30 --yaw 30 --lamps 4
