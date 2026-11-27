#!/bin/bash
# `survival-mode` combat in the game (tasks 1.6, 4.3, 4.4, 4.5): a red fox
# stalking and warning, a dust hound warning, a sword swing landing on a fox,
# a thrown spear in flight, and a town's guard coming, by day and lit at
# night by the game's own lamps. `--combat <scene>` and `--hold <good>` are
# capture instruments (nothing a player meets); each run is a memory-only
# world made a Survival one. Needs the fast build (cargo build -p pbd-app
# --profile fast). On a Claude Code cloud container this renders on lavapipe
# (software, no GPU): it shows what things look like, never how fast they
# draw, and a still is fixed-step.
# Usage: tools/capture_survival_combat.sh [out dir] [shot ...]
set -u
D=${1:-docs/screenshots/survival-mode}
shift || true
cd "$(dirname "$0")/.."
EXE=target/fast/pbd-app
AT="--at -37.5 -84"
TOWN="--at 29.221240 8.794353 --yaw 87.5"
shot() {
  local name=$1; shift
  timeout 1500 xvfb-run -a -s "-screen 0 1440x900x24" $EXE "$@" --rain 0 --weather-at 7200 --capture "$D/$name.png" > "$D/$name.log" 2>&1
  echo "$name exit $?"
  python3 -c "from PIL import Image; Image.open('$D/$name.png').convert('RGB').save('$D/$name.jpg', quality=88)" && rm -f "$D/$name.png"
}
want() { [ -z "$WANT" ] || [[ " $WANT " == *" $1 "* ]]; }
WANT="$*"
DAY="--time 16"
NIGHT="--time 3.8 --lamps 6 2"
want combat-fox-stalk-day && shot combat-fox-stalk-day --world cb-1 $AT --combat fox-stalk $DAY --pitch -10 --frames ${STALK_F:-100}
want combat-fox-stalk-night && shot combat-fox-stalk-night --world cb-2 $AT --combat fox-stalk $NIGHT --pitch -10 --frames ${STALK_F:-100}
want combat-fox-warn-day && shot combat-fox-warn-day --world cb-3 $AT --combat fox-warn $DAY --pitch -14 --frames ${WARN_F:-130}
want combat-fox-warn-night && shot combat-fox-warn-night --world cb-4 $AT --combat fox-warn $NIGHT --pitch -14 --frames ${WARN_F:-130}
want combat-hound-warn-day && shot combat-hound-warn-day --world cb-5 $AT --combat hound-warn $DAY --pitch -14 --frames ${WARN_F:-130}
want combat-swing-hit-day && shot combat-swing-hit-day --world cb-6 $AT --combat swing-hit --hold bronze-sword $DAY --pitch -35 --frames ${HIT_F:-75}
want combat-swing-hit-night && shot combat-swing-hit-night --world cb-7 $AT --combat swing-hit --hold bronze-sword $NIGHT --pitch -35 --frames ${HIT_F:-75}
want combat-axe-hit-day && shot combat-axe-hit-day --world cb-8 $AT --combat swing-hit --hold iron-hunting-axe $DAY --pitch -35 --frames ${AXE_F:-76}
want combat-spear-flight-day && shot combat-spear-flight-day --world cb-9 $AT --combat spear-throw --hold stone-spear $DAY --pitch -4 --frames ${FLY_F:-104}
want combat-spear-flight-night && shot combat-spear-flight-night --world cb-10 $AT --combat spear-throw --hold stone-spear $NIGHT --pitch -4 --frames ${FLY_F:-104}
want combat-guard-day && shot combat-guard-day --world cb-g1 $TOWN --combat guard --time 11 --pitch -6 --frames ${GUARD_F:-220}
want combat-guard-night && shot combat-guard-night --world cb-g2 $TOWN --combat guard --time 20.6 --pitch -6 --frames ${GUARD_F:-220}
