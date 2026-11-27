#!/bin/bash
# `wildfire` task 4.2 in the game: a fire's front at night, its scar the next
# day and at day 30, and a placed torch the rain put out. Needs the fast
# build (cargo build -p pbd-app --profile fast). `PBD_FIRE=<scene>` is a
# capture instrument (nothing a player meets): it makes the memory-only world
# a Survival one in a dry late summer and lights, runs or puts out what the
# shot shows (`pbd_app::wildfire::capture_scene`). On a Claude Code cloud
# container this renders on lavapipe (software, no GPU): it shows what
# things look like, never how fast they draw.
# Usage: tools/capture_wildfire.sh [out dir] [shot ...]
set -u
D=${1:-docs/screenshots/wildfire}
shift || true
cd "$(dirname "$0")/.."
mkdir -p "$D"
EXE=target/fast/pbd-app
AT="--at -37.5 -84"
shot() {
  local name=$1 scene=$2; shift 2
  PBD_FIRE=$scene timeout 1500 xvfb-run -a -s "-screen 0 1440x900x24" $EXE "$@" --weather-at 7200 --capture "$D/$name.png" > "$D/$name.log" 2>&1
  echo "$name exit $?"
  python3 -c "from PIL import Image; Image.open('$D/$name.png').convert('RGB').save('$D/$name.jpg', quality=88)" && rm -f "$D/$name.png"
}
want() { [ -z "$WANT" ] || [[ " $WANT " == *" $1 "* ]]; }
WANT="$*"
want front-night && shot front-night front:70 --world wf-1 $AT --walk --rain 0 --time 22.5 --pitch -14 --frames 160
want scar-next-day && shot scar-next-day scar:1 --world wf-2 $AT --walk --rain 0 --time 15 --pitch -16 --frames 160
want scar-day-30 && shot scar-day-30 scar:30 --world wf-3 $AT --walk --rain 0 --time 15 --pitch -16 --frames 160
want torch-out-rain && shot torch-out-rain torch-out --world wf-4 $AT --walk --torch --rain 1 --time 20.5 --pitch -40 --frames 160
