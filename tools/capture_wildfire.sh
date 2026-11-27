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
# A dry mountain meadow for the fire, open grass with a few pines (found
# by the ignored test `wildfire::tests::find_dry_grassland`: 291 of its
# 421 cells near read plant_dry 0.7 or more in the dry season), and the
# survival meadow for the torch. No debug readout (`--no-readout`).
AT="--at -37.5 -107.5 --no-readout"
MEADOW="--at -37.5 -84 --no-readout"
shot() {
  local name=$1 scene=$2; shift 2
  PBD_FIRE=$scene timeout 1500 xvfb-run -a -s "-screen 0 1440x900x24" $EXE "$@" --weather-at 7200 --capture "$D/$name.png" > "$D/$name.log" 2>&1
  echo "$name exit $?"
  python3 -c "from PIL import Image; Image.open('$D/$name.png').convert('RGB').save('$D/$name.jpg', quality=88)" && rm -f "$D/$name.png"
}
# Each shot opens a world of its own, made afresh: a world kept from an
# earlier run would open where its player stood, not at --at.
rm -rf saves/wf-1 saves/wf-2 saves/wf-3 saves/wf-4 saves/wf-5 saves/wf-6 saves/wf-7
want() { [ -z "$WANT" ] || [[ " $WANT " == *" $1 "* ]]; }
WANT="$*"
want front-night && shot front-night front:90 --world wf-1 $AT --walk --rain 0 --time 2.5 --pitch -12 --frames 160
want tree-burning-night && shot tree-burning-night tree:25 --world wf-6 $AT --walk --rain 0 --time 2.5 --pitch -6 --frames 160
want tree-snag-next-day && shot tree-snag-next-day tree-snag:1 --world wf-7 $AT --walk --rain 0 --time 19.2 --pitch -6 --frames 160
want tree-snag-day-30 && shot tree-snag-day-30 tree-snag:30 --world wf-2 $AT --walk --rain 0 --time 19.2 --pitch -6 --frames 160
# A row of every lamp 3 m ahead (`--lamps`), the torch first: lit in the
# rain, then put out, its body shown by the lanterns beside it. `lit` sets
# no scene but the Survival world, for the before.
want torch-lit-rain && shot torch-lit-rain lit --world wf-5 $MEADOW --walk --lamps 3 0 --rain 1 --time 2.5 --yaw -25 --pitch -18 --frames 160
want torch-out-rain && shot torch-out-rain torch-out --world wf-4 $MEADOW --walk --lamps 3 0 --rain 1 --time 2.5 --yaw -25 --pitch -18 --frames 160
