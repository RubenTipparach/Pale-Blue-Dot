#!/bin/bash
# `survival-mode` 5.1: each survival state on the HUD, and the bedroll and
# the dropped bag, with no debug readout (`--no-readout`). Each state is set
# by the capture instrument `PBD_SURVIVAL_STATE=<state>` (the capture's
# world made a Survival one, its bars held at the mockup's state;
# `desktop::survival_play::instrument`). Needs the fast build. On a Claude
# Code cloud container this renders on lavapipe (software, no GPU): it shows
# what things look like, never how fast they draw, and a still is
# fixed-step.
# Usage: tools/capture_survival_hud.sh [out dir] [shot ...]
set -u
D=${1:-docs/screenshots/survival-mode}
shift || true
cd "$(dirname "$0")/.."
EXE=target/fast/pbd-app
SEA="--at 3.5 -18.15 --yaw -150"
shot() {
  local name=$1 state=$2; shift 2
  PBD_SURVIVAL_STATE=$state timeout 1500 xvfb-run -a -s "-screen 0 1440x900x24" $EXE --walk --rain 0 --no-readout "$@" --capture "$D/$name.png" > "$D/$name.log" 2>&1
  echo "$name exit $?"
  python3 -c "from PIL import Image; Image.open('$D/$name.png').convert('RGB').save('$D/$name.jpg', quality=88)" && rm -f "$D/$name.png"
}
want() { [ -z "$WANT" ] || [[ " $WANT " == *" $1 "* ]]; }
WANT="$*"
want hud-full-hidden-day && shot hud-full-hidden-day full --frames 300 --time 12
want hud-thirsty-50-day && shot hud-thirsty-50-day thirsty50 --frames 300 --time 12
want hud-thirsty-25-night && shot hud-thirsty-25-night thirsty25 --frames 300 --time 22
want hud-winded-day && shot hud-winded-day winded --frames 300 --time 12
want hud-heat-warning-day && shot hud-heat-warning-day hot --frames 300 --time 13
want hud-cold-night && shot hud-cold-night cold --frames 300 --time 3
want hud-under-water-breath && shot hud-under-water-breath under --frames 300 $SEA --pitch 10 --time 11
want hud-salt-water-day && shot hud-salt-water-day salt --frames 300 $SEA --pitch -30 --time 11
want hud-dead-pack-dropped-day && shot hud-dead-pack-dropped-day dead --frames 300 --time 12 --survival-kit --pitch -70
want hud-worn-pickaxe-day && shot hud-worn-pickaxe-day worn --frames 300 --time 12 --tool pickaxe
want hud-broken-axe-day && shot hud-broken-axe-day worn --frames 300 --time 12 --tool axe
want bedroll-day && shot bedroll-day bedroll --frames 300 --time 12 --pitch -40
want bedroll-night && shot bedroll-night bedroll --frames 300 --time 22 --lamps 5 2 --hold block:torch --pitch -40
want bag-day && shot bag-day bag --frames 300 --time 12 --pitch -30 --survival-kit
want hud-passed-out-day && shot hud-passed-out-day out --frames 300 --time 13
want hud-hit-day && shot hud-hit-day hit --frames 300 --time 12
