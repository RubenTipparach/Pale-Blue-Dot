"""Capture repeat: run one `--capture` command twice and count the pixels
that differ (`loading-screen` decision 5).

A measurement instrument (`CLAUDE.md`: say what it measures). A capture steps
time by a fixed amount a frame, so the same exe, assets and arguments must draw
the same picture on every run; before the loading screen two runs matched to
0-14 pixels at 1/255. This runs each scene twice, compares the two PNGs channel
by channel, and reports the pixels with any channel more than `--tolerance`
apart. It exits 1 when a scene differs in more than `--max-pixels`.

It launches the game, so it holds the machine like any capture: it refuses
while `C:/Users/santi/repos/perf.lock` (or `PBD_PERF_LOCK`) exists and is not
its caller's.

Usage:
    python tools/capture_repeat.py <exe> [--out output/capture-repeat]
        [--frames 300] [--scenes lane,shore] [--tolerance 1] [--max-pixels 14]
"""
from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

import numpy as np
from PIL import Image

# The Holbrook lane (tools/capture_holbrook.sh) and a harbour seen from 30 m.
SCENES = {
    "lane": ["--walk", "--at", "29.69673", "1.19287", "--yaw", "179.4",
             "--time", "11", "--rain", "0", "--weather-at", "7200", "--open-doors"],
    "shore": ["--view", "column", "--at", "23.71891", "-115.27982", "--height", "30",
              "--pitch", "-20", "--time", "11", "--rain", "0", "--weather-at", "7200"],
}


def lock_held_by_other(lock: Path, owner: str | None) -> bool:
    if not lock.exists():
        return False
    return owner is None or owner not in lock.read_text(errors="replace")


def capture(exe: str, args: list[str], frames: int, png: Path) -> None:
    log = png.with_suffix(".log")
    with open(log, "w") as f:
        done = subprocess.run([exe, *args, "--capture", str(png), "--frames", str(frames)],
                              stdout=f, stderr=subprocess.STDOUT, timeout=900)
    if done.returncode != 0 or not png.exists():
        sys.exit(f"capture failed ({done.returncode}); see {log}")


def differing(a: Path, b: Path, tolerance: int) -> tuple[int, int, int]:
    x = np.asarray(Image.open(a).convert("RGB"), dtype=np.int16)
    y = np.asarray(Image.open(b).convert("RGB"), dtype=np.int16)
    if x.shape != y.shape:
        sys.exit(f"{a} and {b} differ in size: {x.shape} {y.shape}")
    delta = np.abs(x - y).max(axis=2)
    return int((delta > tolerance).sum()), int(delta.size), int(delta.max())


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("exe")
    ap.add_argument("--out", default="output/capture-repeat")
    ap.add_argument("--frames", type=int, default=300)
    ap.add_argument("--scenes", default="lane,shore")
    ap.add_argument("--tolerance", type=int, default=1, help="per channel, in 1/255")
    ap.add_argument("--max-pixels", type=int, default=14)
    ap.add_argument("--lock-owner", default=None,
                    help="text in perf.lock that marks it as the caller's own")
    a = ap.parse_args()
    lock = Path(os.environ.get("PBD_PERF_LOCK", "C:/Users/santi/repos/perf.lock"))
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    failed = False
    for name in a.scenes.split(","):
        pngs = []
        for run in (1, 2):
            if lock_held_by_other(lock, a.lock_owner):
                sys.exit(f"{lock} is held by another sitting; not launching")
            png = out / f"{name}-{run}.png"
            png.unlink(missing_ok=True)
            capture(a.exe, SCENES[name], a.frames, png)
            pngs.append(png)
        n, total, worst = differing(*pngs, a.tolerance)
        ok = n <= a.max_pixels
        failed |= not ok
        print(f"{name}: {n} of {total} pixels differ by more than {a.tolerance}/255 "
              f"({100 * n / total:.2f}%), worst {worst}/255: {'ok' if ok else 'DIFFERENT'}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
