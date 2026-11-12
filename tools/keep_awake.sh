#!/bin/sh
# Keep this Mac's screen from sleeping or locking while benchmarks or
# recordings run (`frame-budget`): under a locked screen macOS throttles the
# game's window and every frame time measures the throttle, not the game.
#
#   tools/keep_awake.sh              hold until Ctrl-C
#   tools/keep_awake.sh 4            hold for 4 hours
#   tools/keep_awake.sh -- <cmd...>  hold while <cmd> runs
#
# caffeinate -d keeps the display awake, -u declares the user active (no
# screensaver, so no lock behind it), -i/-m/-s keep the system, disk and
# power-adapter sleep off. It cannot unlock a screen that is already locked.
set -eu
[ "$(uname)" = Darwin ] || { echo "keep_awake: macOS only" >&2; exit 1; }
if [ "${1:-}" = "--" ]; then
  shift
  exec caffeinate -dimsu "$@"
elif [ -n "${1:-}" ]; then
  secs=$(awk "BEGIN { print int($1 * 3600) }")
  echo "keeping the screen awake for $1 h"
  exec caffeinate -dimsu -t "$secs"
else
  echo "keeping the screen awake until Ctrl-C"
  exec caffeinate -dimsu
fi
