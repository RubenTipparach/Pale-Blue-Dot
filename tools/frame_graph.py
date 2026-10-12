"""Frame graph: plot a `--frame-log` CSV and attribute its spikes.

A measurement instrument (`CLAUDE.md`: say what it measures). It reads the
per-frame wall time the game writes with `--frame-log`, and optionally the
game's own log for the same run, and produces:

- a PNG graph of frame time over the run, with the 60 and 120 Hz budgets, the
  frames over budget marked, and the fine-set rebuilds and landings shaded;
- a text report: percentiles, the budget misses, and for each spike what was
  happening on the frame clock at that moment (a rebuild in flight, a landing,
  the walk stalled) and which game-log lines fall near it.

Usage:
    python tools/frame_graph.py output/perf/walk1.csv [--log output/perf/walk1.log]
        [--out output/perf/walk1.png] [--skip-s 10] [--budget-ms 16.7]

The frame clock and the game log's clock both start with the process (the log's
first timestamped line stands in for time zero); they agree to about a tenth of
a second, which is the attribution's resolution.
"""
from __future__ import annotations

import argparse
import csv
import math
import re
from collections import Counter
from pathlib import Path


def load_frames(path: Path) -> list[dict]:
    rows = []
    with open(path, newline="") as f:
        for r in csv.DictReader(f):
            try:
                rows.append({
                    "t": float(r["since_start_s"]),
                    "ms": float(r["wall_ms"]),
                    "ver": int(r["fine_version"]),
                    "rebuild": r["rebuild_s"] != "",
                    "walked": float(r["walked_m"]) if r.get("walked_m") not in (None, "", "NaN", "nan") else math.nan,
                    "clearance": float(r["clearance_m"]) if r.get("clearance_m") not in (None, "", "NaN", "nan") else math.nan,
                })
            except (KeyError, ValueError, TypeError):
                continue  # the last row of a killed run can be cut short
    return rows


def load_log(path: Path) -> list[tuple[float, str]]:
    """Timestamped game-log lines as (seconds since the first line, text)."""
    events = []
    t0 = None
    for line in open(path, encoding="utf-8", errors="replace"):
        m = re.search(r"T(\d+):(\d+):([\d.]+)Z", line)
        if not m:
            continue
        t = int(m[1]) * 3600 + int(m[2]) * 60 + float(m[3])
        t0 = t if t0 is None else t0
        text = re.sub(r"\x1b\[[0-9;]*m", "", line).strip()
        text = re.sub(r"^\S+\s+", "", text)  # drop the timestamp
        events.append((t - t0, text))
    return events


def percentile(sorted_ms: list[float], p: float) -> float:
    return sorted_ms[min(len(sorted_ms) - 1, int((len(sorted_ms) - 1) * p))]


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("frames", type=Path)
    ap.add_argument("--log", type=Path, help="the game's own log for the same run")
    ap.add_argument("--out", type=Path, help="PNG path (default: beside the CSV)")
    ap.add_argument("--skip-s", type=float, default=10.0, help="ignore startup before this, seconds")
    ap.add_argument("--budget-ms", type=float, default=1000 / 60)
    ap.add_argument("--top", type=int, default=15, help="spikes to list in the report")
    a = ap.parse_args()

    rows = [r for r in load_frames(a.frames) if r["t"] >= a.skip_s]
    if not rows:
        raise SystemExit("no frames after the skip")
    ms = sorted(r["ms"] for r in rows)
    over = [r for r in rows if r["ms"] > a.budget_ms]
    duration = rows[-1]["t"] - rows[0]["t"]
    landings = [cur["t"] for prev, cur in zip(rows, rows[1:]) if cur["ver"] != prev["ver"]]

    report = []
    report.append(f"{a.frames.name}: {len(rows)} frames over {duration:.1f} s "
                  f"({len(rows) / max(duration, 1e-6):.0f} fps average)")
    report.append("frame ms: p50 {:.2f}  p90 {:.2f}  p99 {:.2f}  p99.9 {:.2f}  max {:.1f}".format(
        percentile(ms, .5), percentile(ms, .9), percentile(ms, .99), percentile(ms, .999), ms[-1]))
    report.append(f"over {a.budget_ms:.1f} ms: {len(over)} ({100 * len(over) / len(rows):.1f}%), "
                  f"over 33.3 ms: {sum(m > 33.3 for m in ms)}, over 100 ms: {sum(m > 100 for m in ms)}")
    in_rebuild = sum(r["rebuild"] for r in over)
    on_landing = sum(1 for r in over if any(0 <= r["t"] - lt < 0.05 for lt in landings))
    report.append(f"of those: during a fine-set rebuild {in_rebuild}, within 50 ms after a landing {on_landing}; "
                  f"{len(landings)} landings, rebuild in flight {100 * sum(r['rebuild'] for r in rows) / len(rows):.0f}% of frames")
    # Time lost: frames over budget, their excess summed.
    report.append(f"time over budget: {sum(r['ms'] - a.budget_ms for r in over) / 1000:.1f} s of {duration:.1f} s")

    events = load_log(a.log) if a.log else []
    report.append("")
    report.append(f"worst {a.top} frames:")
    for r in sorted(over, key=lambda r: -r["ms"])[: a.top]:
        near_land = min((abs(r["t"] - lt) for lt in landings), default=math.inf)
        report.append(f"  {r['t']:8.2f} s  {r['ms']:7.1f} ms  walked {r['walked']:6.0f} m  "
                      f"{'REBUILD ' if r['rebuild'] else ''}{'landing %+.2fs' % near_land if near_land < 1 else ''}")
        for t, text in events:
            if abs(t - r["t"]) < 0.25 and "Weather here" not in text:
                report.append(f"             log {t:8.2f}  {text[:150]}")

    # Which game-log lines crowd the spikes, against their share overall.
    if events:
        def kind(text: str) -> str:
            return re.sub(r"[\d.]+", "#", text)[:70]
        near = Counter()
        for r in over:
            for t, text in events:
                if abs(t - r["t"]) < 0.25:
                    near[kind(text)] += 1
        report.append("")
        report.append("log lines within 250 ms of an over-budget frame (count):")
        for k, n in near.most_common(10):
            report.append(f"  {n:5d}  {k}")

    text = "\n".join(report)
    print(text)
    out = a.out or a.frames.with_suffix(".png")
    out.with_suffix(".txt").write_text(text + "\n", encoding="utf-8")

    try:
        import matplotlib
        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
    except ImportError:
        print("matplotlib not installed: no graph (pip install matplotlib)")
        return
    t = [r["t"] for r in rows]
    y = [r["ms"] for r in rows]
    fig, (ax, ax2) = plt.subplots(2, 1, figsize=(16, 8), sharex=True,
                                  gridspec_kw={"height_ratios": [4, 1]})
    # Rebuilds in flight, shaded.
    start = None
    for r in rows:
        if r["rebuild"] and start is None:
            start = r["t"]
        elif not r["rebuild"] and start is not None:
            ax.axvspan(start, r["t"], color="#f2c14e", alpha=0.18, lw=0)
            start = None
    for lt in landings:
        ax.axvline(lt, color="#e07a1f", lw=0.6, alpha=0.6)
    ax.plot(t, y, lw=0.5, color="#3465a4")
    ax.scatter([r["t"] for r in over], [r["ms"] for r in over], s=8, color="#cc0000", zorder=3,
               label=f"over {a.budget_ms:.1f} ms ({len(over)})")
    ax.axhline(1000 / 60, color="#cc0000", lw=0.8, ls="--", label="60 Hz budget")
    ax.axhline(1000 / 120, color="#4e9a06", lw=0.8, ls="--", label="120 Hz budget")
    top = max(40.0, min(ms[-1] * 1.05, 250.0))
    ax.set_ylim(0, top)
    ax.set_ylabel("frame time, ms")
    ax.set_title(f"{a.frames.name}: p50 {percentile(ms, .5):.1f} ms, p99 {percentile(ms, .99):.1f}, "
                 f"p99.9 {percentile(ms, .999):.1f}, max {ms[-1]:.0f}; shaded = fine-set rebuild, "
                 f"orange lines = landings")
    ax.legend(loc="upper right")
    walked = [r["walked"] for r in rows]
    if any(not math.isnan(w) for w in walked):
        ax2.plot(t, walked, color="#555753")
        ax2.set_ylabel("walked, m")
    else:
        ax2.plot(t, [r["clearance"] for r in rows], color="#555753")
        ax2.set_ylabel("clearance, m")
    ax2.set_xlabel("seconds since start")
    fig.tight_layout()
    fig.savefig(out, dpi=110)
    print(f"graph: {out}")


if __name__ == "__main__":
    main()
