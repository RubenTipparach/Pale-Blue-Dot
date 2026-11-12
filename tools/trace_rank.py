"""Rank where a frame's main-thread time goes, from a Chrome trace written by
the `trace` build (`cargo build --profile fast -p pbd-app --features trace`).

    python tools/trace_rank.py trace-1234.json --frames 3000 --skip-s 20

Each span is keyed by its system or stage name (the span's `name` argument
where Bevy gives one), and its time is summed per thread. The table lists the
spans with the most total time, as ms per frame, on the thread that does the
most work (the main schedule's thread) and over all threads. Spans nest, so a
schedule's time includes its systems': read a system against its siblings.
The spans cost time themselves: this ranks, it does not measure
(`openspec/changes/town-frame-time/design.md`).

Building it (2026-10-08): the lock's `profiling` 1.0.18 wants a
`profiling-procmacros` 1.0.18 that crates.io no longer serves, so step it
back for the build and put the lock back after:
    cargo update -p profiling --precise 1.0.17
    cargo build --profile fast -p pbd-app --features trace
    git checkout Cargo.lock
The game writes `trace-<n>.json` in the working directory: a town walk of
100 m is about 2 GB.
"""
from __future__ import annotations

import argparse
import json
from collections import defaultdict
from pathlib import Path


def load(path: Path):
    """The trace's events one at a time: `tracing-chrome` writes one event a
    line, and a town walk's trace runs to gigabytes."""
    with path.open(encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.strip().rstrip(",")
            if not line.startswith("{"):
                continue
            try:
                yield json.loads(line)
            except json.JSONDecodeError:
                # The last line of a trace cut off when the game quit.
                continue


def key(event: dict) -> str:
    args = event.get("args") or {}
    name = args.get("name") or args.get("system") or ""
    return f"{event.get('name', '?')}: {name}" if name else event.get("name", "?")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("trace", type=Path)
    ap.add_argument("--frames", type=int, required=True, help="frames in the window (from the frame log)")
    ap.add_argument("--skip-s", type=float, default=0.0, help="seconds at the start left out")
    ap.add_argument("--until-s", type=float, default=1e9)
    ap.add_argument("--top", type=int, default=45)
    a = ap.parse_args()

    t0 = None
    count = 0
    per_thread: dict[int, dict[str, float]] = defaultdict(lambda: defaultdict(float))
    open_spans: dict[tuple, list] = defaultdict(list)
    for e in load(a.trace):
        count += 1
        ph, ts = e.get("ph"), e.get("ts")
        if ts is None:
            continue
        if t0 is None:
            t0 = ts
        lo, hi = t0 + a.skip_s * 1e6, t0 + a.until_s * 1e6
        tid = e.get("tid", 0)
        if ph == "X":
            if lo <= ts <= hi:
                per_thread[tid][key(e)] += e.get("dur", 0.0)
        elif ph == "B":
            open_spans[(tid,)].append(e)
        elif ph == "E":
            stack = open_spans[(tid,)]
            if stack:
                begin = stack.pop()
                if lo <= begin["ts"] <= hi:
                    per_thread[tid][key(begin)] += ts - begin["ts"]

    totals: dict[str, float] = defaultdict(float)
    for spans in per_thread.values():
        for k, us in spans.items():
            totals[k] += us
    busiest = max(per_thread, key=lambda t: sum(per_thread[t].values()))
    print(f"{count} events, {len(per_thread)} threads; busiest thread {busiest}; "
          f"{a.frames} frames in the window\n")
    print(f"{'ms/frame all':>12} {'ms/frame main':>13}  span")
    for k, us in sorted(totals.items(), key=lambda kv: -kv[1])[: a.top]:
        main_us = per_thread[busiest].get(k, 0.0)
        print(f"{us / 1000 / a.frames:12.3f} {main_us / 1000 / a.frames:13.3f}  {k}")


if __name__ == "__main__":
    main()
