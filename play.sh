#!/usr/bin/env bash
# PLAY Pale Blue Dot straight away (macOS / Linux), like play.bat: launches
# the last fast build (target/fast/pbd-app) without running cargo, so it
# starts in seconds. Use ./play-build.sh to rebuild after pulling or changing code.
# Arguments are passed to the game, e.g. ./play.sh --tour.
#
# Every run's output (and a panic's message and backtrace, if it crashes) is
# kept in logs/play-last.log; the run before it in logs/play-previous.log.
set -u
cd "$(dirname "$0")" || exit 1

if [ ! -x target/fast/pbd-app ]; then
    echo "No fast build yet. Building it once with play-build.sh..."
    exec ./play-build.sh "$@"
fi

mkdir -p logs
[ -f logs/play-last.log ] && mv -f logs/play-last.log logs/play-previous.log
export RUST_BACKTRACE=1
echo "Playing. Output goes to logs/play-last.log"
target/fast/pbd-app "$@" > logs/play-last.log 2>&1
code=$?
if [ $code -ne 0 ]; then
    echo
    echo "The game exited with code $code. The end of logs/play-last.log:"
    tail -n 30 logs/play-last.log
fi
exit $code
