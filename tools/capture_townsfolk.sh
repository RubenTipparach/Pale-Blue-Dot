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
MARKET="--walk --at 41.152229 108.949181 --yaw 5.2 --pitch -6"
GUARD="--walk --at 40.839832 107.927841 --yaw -108.3 --pitch -8"
shot() {
  name=$1; expr=$2; shift 2
  PBD_TOWNSFOLK_EXPRESSION=$expr timeout 1500 xvfb-run -a -s "-screen 0 960x600x24" $EXE "$@" \
    --rain 0 --weather-at 7200 --capture "$D/$name.png" --frames 150 > "$D/$name.log" 2>&1
  echo "$name exit $?"
}
shot smith-face "" $SMITH_FRONT --time 11
shot smith-forge-2230 "" $SMITH_SIDE --time 22.5
shot market-day "" $MARKET --time 11
shot gate-guard "" $GUARD --time 11
for e in surprised sleepy delighted worried cross thinking; do
  shot "face-$e" "$e" $SMITH_FRONT --time 11
done
