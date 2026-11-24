#!/bin/bash
# Talking to a real townsperson in the game (`townsfolk` design 35): the
# walker stood face to face with Linenleigh's merchant (its trader)
# and its reeve (its job giver), from the spots
# `cargo test --profile fast -p pbd-app --lib -- --ignored
# print_where_to_stand_for_the_townsfolk_shots --nocapture` prints. Nothing
# else is set: PBD_KEY_SCRIPT presses F to talk, and `press:<button>` steps
# press the talk panel's buttons as a click would.
# Needs the fast build: cargo build --profile fast -p pbd-app
# Usage: tools/capture_talk.sh [out dir] [exe]
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
STALL=${STALL:-"--at 29.208122 8.793697 --yaw 87.5"}
SMITH=${SMITH:-"--at 29.521185 8.849977 --yaw 87.4"}
ONLY=${ONLY:-all}
[ "$ONLY" = all -o "$ONLY" = talk ] && shot talk "120:KeyF" $STALL --frames 170
[ "$ONLY" = all -o "$ONLY" = trade ] && shot trade "120:KeyF,140:press:trade,160:press:sell" $STALL --frames 200
[ "$ONLY" = all -o "$ONLY" = board ] && shot board "120:KeyF,140:press:work" $SMITH --frames 190
# The trade columns scroll and filter (design 38b, 38c): the merchant's nine
# goods in a window of seven, at the top, one wheel notch down over BUY, and
# the BUY column filtered to food. `wheel:<list>:<notches>` turns the wheel
# over that list as the wheel would; `press:buy-food` presses its tab.
if [ "$ONLY" = all -o "$ONLY" = scroll ]; then
  shot trade-scroll-top "120:KeyF,140:press:trade" $STALL --frames 190
  shot trade-scroll-partway "120:KeyF,140:press:trade,170:wheel:buy:-1" $STALL --frames 200
  shot trade-filter-food "120:KeyF,140:press:trade,170:press:buy-food" $STALL --frames 200
fi
# Barter (design 37a, task 11.3): two of the merchant's first good in YOU
# TAKE and two of the pack's first good it buys in YOU GIVE, the line under
# them saying who pays; then the deal made, and the coin count beside the
# hotbar after it (task 5.2), the talk left with BACK and BYE.
if [ "$ONLY" = all -o "$ONLY" = barter ]; then
  # PBD_PACK (a capture instrument) puts goods the merchant buys in the pack.
  PACK="wheat:6,log:4,egg:3"
  PBD_PACK="$PACK" shot trade-basket "120:KeyF,140:press:trade,160:press:buy,165:press:buy,170:press:sell,175:press:sell" $STALL --no-readout --frames 210
  PBD_PACK="$PACK" shot coins-hotbar "120:KeyF,140:press:trade,160:press:sell,165:press:sell,170:press:sell,175:press:deal,185:press:back,195:press:bye" $STALL --no-readout --frames 230
fi
# Framing (design 37a, framing; owner, survey P1, 2026-11-24): the same
# barter, the panel on the right and the look turned on F so the merchant
# stands in the left third.
if [ "$ONLY" = all -o "$ONLY" = framed ]; then
  PBD_PACK="wheat:6,log:4,egg:3" shot trade-framed "120:KeyF,140:press:trade,160:press:buy,165:press:buy,170:press:sell,175:press:sell" $STALL --no-readout --frames 210
fi
# The lost villager (`town-missions-tree` task 6.3, `townsfolk` 12.4): the
# reeve's board on a day it posts one (DAY, from `cargo test --profile fast
# -p pbd-app --lib -- --ignored print_the_days_linenleighs_board_posts_a_lost_villager
# --nocapture`: day 13 is the first), its own icon in the row's well; then the job taken and the
# compass and the map marking where the villager stands.
DAY=${DAY:-13}
if [ "$ONLY" = all -o "$ONLY" = lost ]; then
  shot board-lost "120:KeyF,140:press:work" $SMITH --no-readout --day "$DAY" --frames 190
  shot lost-compass "120:KeyF,140:press:work,160:press:take-lost,170:press:back,180:press:bye" $SMITH --no-readout --day "$DAY" --frames 230
  shot lost-map "120:KeyF,140:press:work,160:press:take-lost,170:press:back,180:press:bye,200:KeyM" $SMITH --no-readout --day "$DAY" --frames 260
fi
true
