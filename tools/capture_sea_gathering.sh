#!/bin/bash
# Gathering the sea floor (`sea-gathering` task 4): `PBD_SEA_SHOT` (a capture
# instrument) turns the walker onto the nearest piece of a kind, swims to it
# and presses F on a frame; a frame past the capture never presses, which is
# the "before". With a third part it holds that good and right-clicks the
# ground ahead instead (an ornament). `--menu pack` with `PBD_PACK` shows the
# pack with the new icons. Rendered on lavapipe in a cloud session: these
# show looks, never frame time.
# Needs the fast build: cargo build --profile fast -p pbd-app
# Usage: tools/capture_sea_gathering.sh [out dir] [exe]
set -u
D=${1:-docs/screenshots/sea-gathering}
cd "$(dirname "$0")/.."
EXE=${2:-target/fast/pbd-app}
mkdir -p "$D"
shot() {
  name=$1; shift
  PBD_WINDOW=1200x750 timeout 2400 xvfb-run -a -s "-screen 0 1200x750x24" \
    "$EXE" --walk --hold-pointer --no-readout "$@" --rain 0 --weather-at 7200 \
    --capture "$D/$name.png" > "$D/$name.log" 2>&1
  echo "$name exit $?"
}
REEF=${REEF:-"--at 0 0"}
KELP=${KELP:-"--at 0 0"}
LAND=${LAND:-"--at 0 0"}
ONLY=${ONLY:-all}
on() { [ "$ONLY" = all ] || [[ ",$ONLY," == *",$1,"* ]]; }
WORLD="--world sea$$"
on reef && PBD_SEA_SHOT=any:99999 shot reef-before $REEF --time 12 --frames 420 $WORLD-a
on reef && PBD_SEA_SHOT=any:30 shot reef-after $REEF --time 12 --frames 420 $WORLD-b
on kelp && PBD_SEA_SHOT=kelp:99999 shot kelp-before $KELP --time 12 --frames 420 $WORLD-c
on kelp && PBD_SEA_SHOT=kelp:30 shot kelp-after $KELP --time 12 --frames 420 $WORLD-d
on pack && PBD_PACK="staghorn-coral:2,finger-coral:2,brain-coral:1,table-coral:1,soft-coral:3,tube-sponge:2,barrel-sponge:1,sea-fan:2,sea-anemone:2,sea-lettuce:5,coral-rubble:4,mussels:6,kelp:8,fertilizer:3,steamed-mussels:1,kelp-crisps:2" \
  shot pack $LAND --time 12 --menu pack --frames 200 $WORLD-e
on ornament && PBD_SEA_SHOT=any:150:brain-coral shot ornament-night $LAND --time 22.5 --frames 260 $WORLD-f
for f in "$D"/*.png; do
  [ -f "$f" ] && convert "$f" -quality 88 "${f%.png}.jpg" && rm "$f"
done
true
