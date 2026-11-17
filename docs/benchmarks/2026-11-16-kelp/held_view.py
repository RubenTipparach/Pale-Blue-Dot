"""Held view in the kelp forest at 71.308 -8.196, old exe against new, interleaved.
The walker stands on the bed (-9 m) inside the forest: `--stroll` waits there for a
town that never comes near, so the view holds. Frames from READY + 10 s to the end."""
import csv, json, os, re, subprocess, sys, time
from pathlib import Path

root = Path(sys.argv[1]).resolve()
out = Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
exe = root / "output/perf/kelp-exe"
variants = {
    "before": (exe / "pbd-app-before", {}),
    "after": (exe / "pbd-app-after", {}),
    "before-nokelp": (exe / "pbd-app-before", {"PBD_PASS_OFF": "kelp"}),
}
order = ["before", "after", "before-nokelp", "before-nokelp", "after", "before", "before", "after", "before-nokelp"]
args = ["--walk", "--at", "71.308", "-8.196", "--stroll", "600", "--no-vsync"]
ANSI = re.compile(r"\x1b\[[0-9;]*m")

def busy():
    r = subprocess.run(["pgrep", "-fl", "cargo|rustc|perf_suite|pbd-app"], capture_output=True, text=True)
    return [l for l in r.stdout.splitlines() if "zsh" not in l and "bench.py" not in l]

def pct(v, p):
    return v[min(len(v) - 1, int((len(v) - 1) * p))]

results = []
n = 0
queue = list(order)
while queue and n < 24:
    name = queue[0]
    while False:
        print("waiting, busy:", busy()[:2], flush=True); time.sleep(20)
    path, env_extra = variants[name]
    tag = f"{name}_{n}"
    csv_path, log_path = out / f"{tag}.csv", out / f"{tag}.log"
    env = dict(os.environ, **env_extra)
    with open(log_path, "w") as log:
        p = subprocess.Popen([str(path), *args, "--frame-log", str(csv_path)], cwd=root, stdout=log, stderr=subprocess.STDOUT, env=env)
        seen = set()
        t0 = time.time()
        while p.poll() is None and time.time() - t0 < 70:
            for l in busy():
                if str(p.pid) not in l and "pbd-app-" not in l:
                    seen.add(l[:120])
            time.sleep(1)
        if p.poll() is None:
            p.terminate(); p.wait()
    text = ANSI.sub("", log_path.read_text(errors="replace"))
    m = re.search(r"READY playable ([\d.]+) s after launch", text)
    ready = float(m[1]) if m else 30.0
    rows = [r for r in csv.DictReader(open(csv_path)) if r.get("wall_ms") and r.get("since_start_s")]
    ms = sorted(float(r["wall_ms"]) for r in rows if float(r["since_start_s"]) >= ready + 10)
    sea = re.findall(r"SEA_PLACES tiles=(\d+) tris=(\d+) kelp_tris=(\d+) swayed_verts=(\d+) tiles_ms=([\d.]+)", text)
    late = len(re.findall(r"LATE_PIPELINE", text))
    res = {"variant": name, "run": n, "frames": len(ms), "p50": pct(ms, .5), "p95": pct(ms, .95),
           "p99": pct(ms, .99), "max": ms[-1], "mean": sum(ms) / len(ms),
           "sea_places_last": sea[-1] if sea else None, "tiles_ms_med": sorted(float(x[4]) for x in sea[len(sea)//2:])[len(sea[len(sea)//2:])//2] if sea else None, "late_pipelines": late, "ready_s": ready, "contaminated": sorted(seen)[:3]}
    print(json.dumps(res), flush=True)
    n += 1
    if False:
        continue  # run again: something else ran beside it
    queue.pop(0)
    results.append(res)
(out / "results.json").write_text(json.dumps(results, indent=2))
