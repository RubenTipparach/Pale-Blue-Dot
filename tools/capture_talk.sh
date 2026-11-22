#!/bin/bash
# Talking to a real townsperson in the game (`townsfolk` design 35): the
# walker stood face to face with Linenleigh's first stallholder (its trader)
# and its smith (its job giver), from the spots
# `cargo test --profile fast -p pbd-app --lib -- --ignored
# print_where_to_stand_for_the_townsfolk_shots --nocapture` prints. Nothing
# else is set: PBD_KEY_SCRIPT presses F to talk, and `press:<button>` steps
# press the talk panel's buttons as a click would.
# Needs the fast build: cargo build --profile fast -p pbd-app
# Usage: STALL="--at <lat> <lon> --yaw <deg>" SMITH="..." tools/capture_talk.sh [out dir] [exe]
set -u
D=${1:-docs/screenshots/townsfolk-economy}
cd "$(dirname "$0")/.."
EXE=${2:-target/fast/pbd-app}
mkdir -p "$D"
shot() {
  name=$1; keys=$2; shift 2
  PBD_KEY_SCRIPT="$keys" PBD_WINDOW=1200x750 timeout 2400 xvfb-run -a -s "-screen 0 1200x750x24" \
    "$EXE" --walk --hold-pointer "$@" --time 11 --rain 0 --weather-at 7200 --pitch -6 \
    --capture "$D/$name.png" > "$D/$name.log" 2>&1
  echo "$name exit $?"
}
ONLY=${ONLY:-all}
[ "$ONLY" = all -o "$ONLY" = talk ] && shot talk "120:KeyF" $STALL --frames 170
[ "$ONLY" = all -o "$ONLY" = trade ] && shot trade "120:KeyF,140:press:trade,160:press:sell" $STALL --frames 200
[ "$ONLY" = all -o "$ONLY" = board ] && shot board "120:KeyF,140:press:work" $SMITH --frames 190
true
