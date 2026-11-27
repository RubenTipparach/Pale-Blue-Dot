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
# A fields meadow away from water for the fire (the survival meadow at
# -37.5 -84 lies along a river, too green to carry one), the survival
# meadow for the torch.
AT="--at 32.9505 4.01833"
MEADOW="--at -37.5 -84"
shot() {
  local name=$1 scene=$2; shift 2
  PBD_FIRE=$scene timeout 1500 xvfb-run -a -s "-screen 0 1440x900x24" $EXE "$@" --weather-at 7200 --capture "$D/$name.png" > "$D/$name.log" 2>&1
  echo "$name exit $?"
  python3 -c "from PIL import Image; Image.open('$D/$name.png').convert('RGB').save('$D/$name.jpg', quality=88)" && rm -f "$D/$name.png"
}
# Each shot opens a world of its own, made afresh: a world kept from an
# earlier run would open where its player stood, not at --at.
rm -rf saves/wf-1 saves/wf-2 saves/wf-3 saves/wf-4 saves/wf-5
want() { [ -z "$WANT" ] || [[ " $WANT " == *" $1 "* ]]; }
WANT="$*"
want front-night && shot front-night front:25 --world wf-1 $AT --walk --rain 0 --time 2.5 --yaw 35 --pitch -20 --frames 160
want scar-next-day && shot scar-next-day scar:1 --world wf-2 $AT --walk --rain 0 --time 15 --yaw 35 --pitch -22 --frames 160
want scar-day-30 && shot scar-day-30 scar:30 --world wf-3 $AT --walk --rain 0 --time 15 --yaw 35 --pitch -22 --frames 160
# The torch stands under the camera (`--torch`), so the shot looks down
# at it; `lit` sets no scene but the Survival world, for the before.
want torch-lit-rain && shot torch-lit-rain lit --world wf-5 $MEADOW --walk --torch --rain 1 --time 2.5 --pitch -70 --frames 160
want torch-out-rain && shot torch-out-rain torch-out --world wf-4 $MEADOW --walk --torch --rain 1 --time 2.5 --pitch -70 --frames 160
