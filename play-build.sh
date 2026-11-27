#!/usr/bin/env bash
# Build and run Pale Blue Dot (macOS / Linux), like build-play.bat.
# Builds the "fast" profile by default: optimised, but without link-time
# optimisation, so a small change relinks in seconds.
#   ./play-build.sh --full    the real release build (slow link): use it for
#                             performance numbers, recordings and regression checks.
# Other arguments are passed to pbd-app, e.g. ./play-build.sh --tour.
set -u
cd "$(dirname "$0")" || { echo "Could not open the Pale Blue Dot project directory."; exit 1; }

if ! command -v cargo >/dev/null 2>&1; then
    echo "Cargo was not found. Install Rust from https://rustup.rs,"
    echo "then reopen your terminal and run this launcher again."
    exit 1
fi

profile=fast
app_args=()
for arg in "$@"; do
    if [ "$arg" = "--full" ]; then profile=release; else app_args+=("$arg"); fi
done

echo "Building and running Pale Blue Dot, $profile profile."
[ "$profile" = fast ] && echo "Use ./play-build.sh --full for the full release build."
echo "The first build may take several minutes."
echo

if pgrep -x pbd-app >/dev/null 2>&1; then
    echo "Note: pbd-app is already running."
fi

if [ "$profile" = release ]; then
    cargo build --release -p pbd-app || { code=$?; echo "Build failed with exit code $code."; exit $code; }
else
    cargo build --profile fast -p pbd-app || { code=$?; echo "Build failed with exit code $code."; exit $code; }
fi

"target/$profile/pbd-app" ${app_args[@]+"${app_args[@]}"}
code=$?
[ $code -ne 0 ] && echo "The game exited with code $code."
exit $code
