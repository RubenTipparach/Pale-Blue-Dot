#!/bin/bash
# `survival-mode` tasks 4.10 and 4.11 in the game: the weapons held and lying
# in the world, the game species close up by day and at night (lit by a row
# of the game's own lamps), and the pack with the new icons. Needs the fast
# build (cargo build -p pbd-app --profile fast). `--hold`, `--lay-weapons`,
# `--survival-kit` and `--husbandry game` are capture instruments: nothing a
# player meets. Each run is a memory-only world. On a Claude Code cloud
# container this renders on lavapipe (software, no GPU): it shows what
# things look like, never how fast they draw.
# Usage: tools/capture_survival_goods.sh [out dir] [shot ...]
set -u
D=${1:-docs/screenshots/survival-mode}
shift || true
cd "$(dirname "$0")/.."
EXE=target/fast/pbd-app
AT="--at -37.5 -84"
shot() {
  local name=$1; shift
  timeout 1500 xvfb-run -a -s "-screen 0 1440x900x24" $EXE "$@" --rain 0 --weather-at 7200 --capture "$D/$name.png" > "$D/$name.log" 2>&1
  echo "$name exit $?"
  python3 -c "from PIL import Image; Image.open('$D/$name.png').convert('RGB').save('$D/$name.jpg', quality=88)" && rm -f "$D/$name.png"
}
want() { [ -z "$WANT" ] || [[ " $WANT " == *" $1 "* ]]; }
WANT="$*"
want held-spear-day && shot held-spear-day --world sv-held-1 $AT --walk --hold stone-spear --time 16 --pitch 8 --frames 120
want held-sword-day && shot held-sword-day --world sv-held-2 $AT --walk --hold bronze-sword --time 16 --pitch -6 --frames 120
want held-bow-night && shot held-bow-night --world sv-held-3 $AT --walk --hold bow --time 3.8 --lamps 3 -2 --pitch -6 --frames 160
want held-axe-day && shot held-axe-day --world sv-held-4 $AT --walk --hold iron-hunting-axe --time 16 --pitch -6 --frames 120
want weapons-lying-day && shot weapons-lying-day --world sv-lay-1 $AT --lay-weapons 3 --time 16 --pitch -14 --frames 150
want weapons-lying-night && shot weapons-lying-night --world sv-lay-2 $AT --lay-weapons 3 --lamps 3 -3 --time 3.8 --pitch -14 --frames 200
want game-day && shot game-day --world sv-game-1 $AT --husbandry game-stand --time 16 --pitch -14 --frames 200
want game-night && shot game-night --world sv-game-2 $AT --husbandry game-stand --lamps 3 -5 --time 3.8 --pitch -14 --frames 260
want pack-icons && shot pack-icons --world sv-pack-1 $AT --walk --survival-kit --menu pack --time 16 --frames 90
