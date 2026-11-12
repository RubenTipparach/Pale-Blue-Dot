"""Classify a sitting's slow frames: what the frames over the 60 fps floor were.

A measurement instrument (`lighting-perf`, `crab-breeze-lessons` rank 10).
For each run of a `perf_suite.py` sitting, every frame after the warm-up that
took longer than `--over-ms` (16.7 ms) is put in one class:

- `landing`: a new fine set was drawn this frame or the two before it (the
  frame log's `fine_version` moved);
- `log:<what>`: the game logged a known piece of main-thread work within 0.15
  s of the frame (a `SPENT` line, the towns' ground, a bake, an upload);
- `gpu`: the GPU's own time for the frame (the newest `gpu_total_ms` within
  the next four frames, which trail it) was itself over the floor, less 1 ms:
  the frame was the GPU's;
- `gpu-paced`: a long frame beside a short one whose mean is within 1.5 ms
  of the GPU's time: the GPU set the pace and the queue split it unevenly;
- `cpu`: none of those, with a GPU time under the floor: main-thread or
  render-thread time with nothing logged.

    python tools/tail_classes.py output/perf/<sitting> [--variant name] [--over-ms 16.7]

It prints, per scenario and variant, the share of frames over the floor, the
p99, and the classes by count and by summed excess over the floor.
"""
from __future__ import annotations

import argparse
import json
import math
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from perf_suite import ANSI, STAMP, load_frames, log_zero, stamp_s  # noqa: E402

KNOWN = [
    (re.compile(r"SPENT [\d.]+ ms (.+)"), lambda m: "spent:" + m[1].split(" ")[0][:24]),
    (re.compile(r"towns' ground installed"), lambda m: "towns-ground"),
    (re.compile(r"PUFFS built"), None),  # a periodic report, not work
    (re.compile(r"(bake|baked)", re.I), lambda m: "bake"),
    (re.compile(r"(upload|uploaded)", re.I), lambda m: "upload"),
    (re.compile(r"LATE_PIPELINE"), lambda m: "late-pipeline"),
    (re.compile(r"LOD_FADE"), lambda m: "lod-fade"),
]


def classify(rows, text, over, zero):
    events = []
    for line in text.splitlines():
        t = stamp_s(line, zero)
        if t is None:
            continue
        for rx, name in KNOWN:
            m = rx.search(line)
            if m:
                if name:
                    events.append((t, name(m)))
                break
    events.sort()
    out = []
    for i, r in enumerate(rows):
        if r["ms"] <= over:
            continue
        cls = None
        ver = [rows[j].get("ver") for j in range(max(0, i - 2), i + 1)]
        if len(set(ver)) > 1:
            cls = "landing"
        if cls is None:
            near = [e for e in events if abs(e[0] - r["t"]) <= 0.15]
            if near:
                cls = "log:" + near[0][1]
        if cls is None:
            gpu = [rows[j]["gpu_total"] for j in range(i, min(len(rows), i + 5))
                   if not math.isnan(rows[j]["gpu_total"])]
            g = max(gpu) if gpu else math.nan
            pair = min((r["ms"] + rows[j]["ms"]) / 2 for j in (i - 1, i + 1) if 0 <= j < len(rows))
            if g > over - 1.0:
                cls = "gpu"
            elif pair <= g + 1.5:
                # A long frame beside a short one that average to the GPU's
                # time: the queue paced a GPU-bound frame unevenly.
                cls = "gpu-paced"
            else:
                cls = "cpu"
        out.append((cls, r["ms"] - over))
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("run_dir", type=Path)
    ap.add_argument("--variant")
    ap.add_argument("--over-ms", type=float, default=1000 / 60)
    a = ap.parse_args()
    suite = json.loads((a.run_dir / "suite.json").read_text(encoding="utf-8"))
    table = defaultdict(lambda: {"frames": 0, "over": 0, "classes": Counter(), "excess": Counter(), "p99": []})
    for r in suite["results"]:
        if not r["valid"] or (a.variant and r["variant"] != a.variant):
            continue
        tag = f"{r['scenario']}_{r['variant']}_{r['index']}"
        text = ANSI.sub("", (a.run_dir / f"{tag}.log").read_text(encoding="utf-8", errors="replace"))
        every = load_frames(a.run_dir / f"{tag}.csv")
        # The fine set's version, which `load_frames` does not keep.
        import csv
        with open(a.run_dir / f"{tag}.csv", newline="") as f:
            versions = [row.get("fine_version") for row in csv.DictReader(f)]
        for row, v in zip(every, versions):
            row["ver"] = v
        warm = r.get("warm_up", {})
        start = warm.get("from_s", 0.0) + warm.get("skip_s", 10.0)
        rows = [x for x in every if x["t"] >= start and x["ms"] < 1000]
        key = (r["scenario"], r["variant"])
        t = table[key]
        t["frames"] += len(rows)
        cl = classify(rows, text, a.over_ms, log_zero(text))
        t["over"] += len(cl)
        for c, ex in cl:
            t["classes"][c] += 1
            t["excess"][c] += ex
        t["p99"].append(r["frame_ms"].get("p99", math.nan))
    for (scenario, variant), t in sorted(table.items()):
        share = 100 * t["over"] / max(t["frames"], 1)
        print(f"## {scenario} / {variant}: {t['over']} of {t['frames']} frames over "
              f"{a.over_ms:.1f} ms ({share:.2f}%), p99 {', '.join(f'{x:.2f}' for x in t['p99'])}")
        for c, n in t["classes"].most_common():
            print(f"   {c:<32} {n:5d} frames  {t['excess'][c]:8.1f} ms over the floor")


if __name__ == "__main__":
    main()
