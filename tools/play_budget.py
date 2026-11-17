"""Where a frame goes in real play (`openspec/changes/play-frame-budget`).

The showcase bot plays the phase-1 branches in order on one world, through
the player's own inputs (`--showcase`): the trunk (gather, an axe, felling,
a bench, stone tools, a fire), fishing at the sea, farming, herding and
cooking. Each run writes a `--frame-log`; this script then reports, per
branch and per shot (activity), the frame time against the GPU's total and
the main and render worlds' CPU, which of them bounds the frame, and the
GPU passes costliest first.

    python tools/play_budget.py run --out docs/benchmarks/2026-11-17-play-budget
    python tools/play_budget.py report docs/benchmarks/2026-11-17-play-budget

Frames before the game's READY line, and the first 10 s after it, are left
out, as the perf suite's warm-up is.
"""

import argparse
import csv
import json
import re
import statistics
import subprocess
import sys
import time
from pathlib import Path

BRANCHES = ["phase-1-trunk", "phase-1-fishing", "phase-1-farming",
            "phase-1-herding", "phase-1-cooking"]
SKIP_S = 10.0


def run(args):
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    world = args.world or f"play-budget-{int(time.time())}"
    for name in args.branches:
        log = out / f"{name}.log"
        frames = out / f"{name}.csv"
        cmd = [str(Path(args.exe).resolve()), "--showcase", name, "--world", world, "--frame-log", str(frames)]
        print("RUN", " ".join(cmd), flush=True)
        started = time.time()
        with open(log, "w", encoding="utf-8") as f:
            code = subprocess.call(cmd, stdout=f, stderr=subprocess.STDOUT)
        text = log.read_text(encoding="utf-8", errors="replace")
        status = "end" if "SHOWCASE_END" in text else ("stall" if "SHOWCASE_STALL" in text else f"exit {code}")
        print(f"DONE {name} {status} in {time.time() - started:.0f} s", flush=True)
    report(argparse.Namespace(dir=args.out))


def pct(values, p):
    if not values:
        return float("nan")
    s = sorted(values)
    return s[min(len(s) - 1, int((len(s) - 1) * p))]


def shots_of(log_text):
    """Shot number -> caption, from the run's log."""
    shots = {}
    for m in re.finditer(r"SHOT (\d+) (.*)", log_text):
        shots[int(m.group(1))] = re.sub(r"\x1b\[[0-9;]*m", "", m.group(2)).strip()
    return shots


def ready_s(log_text):
    m = re.search(r"READY playable ([0-9.]+) s after launch", log_text)
    return float(m.group(1)) if m else 0.0


def summarise(rows):
    num = lambda k: [float(r[k]) for r in rows if r.get(k) not in (None, "", "nan", "NaN")]
    wall = num("wall_ms")
    gpu = num("gpu_total_ms")
    main = num("cpu_main_ms")
    render = num("cpu_render_ms")
    acquire = num("acquire_ms")
    render_work = [r - a for r, a in zip(render, acquire)]
    passes = {}
    for k in rows[0].keys():
        if k.startswith("gpu_") and k.endswith("_ms") and k not in ("gpu_total_ms", "gpu_clouds_ms"):
            v = num(k)
            if v:
                passes[k[4:-3]] = statistics.fmean(v)
    # Which side bounds each frame: the largest of the GPU, the main world
    # and the render world's own work (its acquire is waiting, not work).
    bound = {"gpu": 0, "main cpu": 0, "render cpu": 0}
    for g, m, rw in zip(gpu, main, render_work):
        side = max((g, "gpu"), (m, "main cpu"), (rw, "render cpu"))[1]
        bound[side] += 1
    n = max(1, sum(bound.values()))
    return {
        "frames": len(wall),
        "seconds": sum(wall) / 1000.0,
        "fps": 1000.0 * len(wall) / max(1e-6, sum(wall)),
        "wall": {p: pct(wall, q) for p, q in (("p50", .5), ("p95", .95), ("p99", .99))},
        "mean_ms": {
            "wall": statistics.fmean(wall) if wall else float("nan"),
            "gpu": statistics.fmean(gpu) if gpu else float("nan"),
            "main_cpu": statistics.fmean(main) if main else float("nan"),
            "render_cpu": statistics.fmean(render_work) if render_work else float("nan"),
            "acquire_wait": statistics.fmean(acquire) if acquire else float("nan"),
        },
        "p95_ms": {
            "gpu": pct(gpu, .95), "main_cpu": pct(main, .95), "render_cpu": pct(render_work, .95),
        },
        "bound_share": {k: v / n for k, v in bound.items()},
        "gpu_passes": dict(sorted(passes.items(), key=lambda kv: -kv[1])),
    }


def report(args):
    d = Path(args.dir)
    result = {"branches": {}}
    all_rows = []
    for csv_path in sorted(d.glob("phase-1-*.csv"), key=lambda p: BRANCHES.index(p.stem) if p.stem in BRANCHES else 99):
        log_path = csv_path.with_suffix(".log")
        text = log_path.read_text(encoding="utf-8", errors="replace") if log_path.exists() else ""
        start = ready_s(text) + SKIP_S
        rows = [r for r in csv.DictReader(open(csv_path, encoding="utf-8"))
                if float(r["since_start_s"]) >= start]
        if not rows:
            continue
        all_rows += rows
        shots = shots_of(text)
        branch = {"status": "end" if "SHOWCASE_END" in text else "stall" if "SHOWCASE_STALL" in text else "?",
                  "all": summarise(rows), "shots": {}}
        for n in sorted({int(r["shot"]) for r in rows}):
            part = [r for r in rows if int(r["shot"]) == n]
            if len(part) > 60:
                branch["shots"][f"{n} {shots.get(n, '')}"] = summarise(part)
        result["branches"][csv_path.stem] = branch
    if all_rows:
        result["all"] = summarise(all_rows)
    (d / "summary.json").write_text(json.dumps(result, indent=1), encoding="utf-8")
    print_report(result)


def print_report(result):
    def line(name, s):
        m = s["mean_ms"]
        b = s["bound_share"]
        print(f"{name[:58]:58} {s['seconds']:6.0f}s {s['fps']:5.1f}fps p50 {s['wall']['p50']:5.1f} p99 {s['wall']['p99']:5.1f} | "
              f"gpu {m['gpu']:5.1f} main {m['main_cpu']:5.1f} render {m['render_cpu']:5.1f} wait {m['acquire_wait']:5.1f} | "
              f"bound gpu {b['gpu']:.0%} main {b['main cpu']:.0%} render {b['render cpu']:.0%}")
    for name, branch in result["branches"].items():
        line(f"{name} ({branch['status']})", branch["all"])
        for shot, s in branch["shots"].items():
            line("  " + shot, s)
    if "all" in result:
        line("ALL", result["all"])
        print("GPU passes, mean ms over all play:")
        for k, v in list(result["all"]["gpu_passes"].items())[:16]:
            print(f"  {v:6.2f}  {k}")


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run")
    r.add_argument("--out", required=True)
    r.add_argument("--exe", default="target/fast/pbd-app.exe")
    r.add_argument("--world")
    r.add_argument("--branches", nargs="+", default=BRANCHES)
    p = sub.add_parser("report")
    p.add_argument("dir")
    a = ap.parse_args()
    (run if a.cmd == "run" else report)(a)


if __name__ == "__main__":
    sys.exit(main())
