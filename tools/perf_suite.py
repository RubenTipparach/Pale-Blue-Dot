"""Performance suite: fly and walk fixed scenarios and compare builds or variants.

The rig of `openspec/changes/perf-rig`, in the style of godot-sandbox's battle
bench: fixed scenarios from a fresh world, windowed and uncapped, a warm-up
that is thrown away, a check that the stressful part of the run really
happened, one JSON file per run, and variants interleaved A, B, A, B so drift
over the sitting falls on every variant alike.

Scenarios (each ends itself):
    clouds    --route scenic      lift-off, into the thickest reachable cloud, low over the land
    storm     --route scenic --rain 1   the same through a forced storm's deck
    cloud-hop --route clouds      low in the layer through several separate clouds
    far-side  --route far-side    ground to 3 km and down on the far side
    walk      --walk-distance 1000
    town      a walk from Ashenstead's middle out through the walled town, rain forced
    sail      Holinghaven's cog cast off from its helm, --sail-for 60

Variants are `name|exe|ENV=1;ENV2=2|assets` (all but the name optional); an
item of the third field that starts with `--` is a launch flag instead
(`lat2||--frame-latency 2`). The
game reads its shaders and `assets/config` from this checkout at run time, so
an old exe would run the NEW shaders (and refuse a config field it does not
know): `assets` is a git revision whose `assets/` files are laid over the
checkout's for that variant's runs only, and put back after each run.
For example:
    python tools/perf_suite.py --out output/perf/today
    python tools/perf_suite.py --scenarios clouds --repeats 3 \\
        --variant "clouds-on|target/release/pbd-app.exe" \\
        --variant "clouds-off|target/release/pbd-app.exe|PBD_NO_CLOUDS=1"
    python tools/perf_suite.py --variant "before|output/perf/baseline-exe/pbd-app.exe||HEAD" \
        --variant "after"

The warm-up (`--skip-s`) is counted from the end of loading (`lighting-perf`
1.3): from the game's "ready" log line where it prints one (`--ready-pattern`),
else from the first frame. A fresh exe on a cold shader cache once drew its
first frame 50-55 s after launch, so a warm-up counted from launch let loading
frames into the numbers. The fresh world's planet rebuild (`towns.rs` `settle`,
a multi-second frame that every variant pays) is left out of the statistics and
reported beside them as `rebuild_ms`.

Budgets are the owner's: 120 fps (8.3 ms) wanted at p95, 60 fps (16.7 ms) the
floor at p99. Run from the repository root (the game's saves are relative).
"""
from __future__ import annotations

import argparse
import csv
import datetime as dt
import json
import math
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

SCENARIOS = {
    "clouds": ["--route", "scenic"],
    "storm": ["--route", "scenic", "--rain", "1"],
    "cloud-hop": ["--route", "clouds"],
    "far-side": ["--route", "far-side"],
    "walk": ["--walk-distance", "1000"],
    # From the middle of Ashenstead (walled, the shipped seed's tour town 4)
    # out through it under forced rain (`town-frame-time`).
    "town": ["--walk", "--at", "19.644", "59.178", "--walk-distance", "400", "--rain", "1"],
    # Up and down Holbrook's lane in a forced rain (`wet-near-detail`): wet
    # ground near and far of the eye, on a script that never digs.
    "rain-stroll": ["--walk", "--at", "29.69673", "1.19287", "--yaw", "179.4", "--rain", "1",
                    "--stroll", "45", "--stroll-leg", "7"],
    # Holinghaven's cog cast off with its yard braced, from its helm
    # (`sail-the-cog` task 6.1).
    "sail": ["--walk", "--at", "23.71891", "-115.27982", "--aboard", "cog", "--sail", "-40",
             "--seat", "--sail-for", "60"],
}
DONE = {"clouds": "ROUTE_COMPLETE", "storm": "ROUTE_COMPLETE", "cloud-hop": "ROUTE_COMPLETE", "far-side": "ROUTE_COMPLETE", "walk": "WALK_DONE", "rain-stroll": "STROLL_DONE",
        "town": "WALK_DONE",
        "sail": "SAIL_DONE"}
WANT_MS = 1000 / 120
FLOOR_MS = 1000 / 60
ANSI = re.compile(r"\x1b\[[0-9;]*m")


def pct(sorted_values: list[float], p: float) -> float:
    if not sorted_values:
        return math.nan
    return sorted_values[min(len(sorted_values) - 1, int((len(sorted_values) - 1) * p))]


def stats(values: list[float]) -> dict:
    v = sorted(x for x in values if not math.isnan(x))
    if not v:
        return {}
    return {
        "n": len(v),
        "mean": sum(v) / len(v),
        "p50": pct(v, 0.5),
        "p95": pct(v, 0.95),
        "p99": pct(v, 0.99),
        "p999": pct(v, 0.999),
        "max": v[-1],
        "over_want": sum(x > WANT_MS for x in v),
        "over_floor": sum(x > FLOOR_MS for x in v),
    }


def num(value: str | None) -> float:
    try:
        return float(value) if value not in (None, "") else math.nan
    except ValueError:
        return math.nan


def load_frames(path: Path) -> list[dict]:
    rows = []
    if not path.exists():
        return rows
    with open(path, newline="") as f:
        for r in csv.DictReader(f):
            t, ms = num(r.get("since_start_s")), num(r.get("wall_ms"))
            if math.isnan(t) or math.isnan(ms):
                continue  # a killed run's last row can be cut short
            rows.append({
                "frame": int(num(r.get("frame"))) if not math.isnan(num(r.get("frame"))) else None,
                "t": t,
                "ms": ms,
                "gpu_clouds": num(r.get("gpu_clouds_ms")),
                "gpu_total": num(r.get("gpu_total_ms")),
                "route_m": num(r.get("route_m")),
                "clearance": num(r.get("clearance_m")),
                # Per-pass GPU columns (`lighting-perf` 1.2): `gpu_<pass>_ms`.
                "passes": {k[4:-3]: num(v) for k, v in r.items()
                           if k and k.startswith("gpu_") and k.endswith("_ms")
                           and k not in ("gpu_clouds_ms", "gpu_total_ms")},
            })
    return rows


def parse_variant(spec: str, default_exe: str) -> dict:
    parts = spec.split("|")
    env, args = {}, []
    if len(parts) > 2 and parts[2]:
        for pair in parts[2].split(";"):
            if pair.strip().startswith("--"):
                # A launch flag rather than a variable: `--frame-latency 2`.
                args += pair.split()
                continue
            k, _, v = pair.partition("=")
            env[k.strip()] = v.strip()
    exe = parts[1] if len(parts) > 1 and parts[1] else default_exe
    assets = parts[3] if len(parts) > 3 and parts[3] else None
    return {"name": parts[0], "exe": str(Path(exe).resolve()), "env": env, "args": args, "assets": assets}


def changed_assets(revision: str) -> list[str]:
    """The files under `assets/` that differ between `revision` and the checkout."""
    out = subprocess.run(["git", "diff", "--name-only", revision, "--", "assets"],
                         capture_output=True, text=True, check=True).stdout
    return [line for line in out.splitlines() if line]


class AssetsAt:
    """Lay `revision`'s version of the changed asset files over the checkout,
    and put the checkout's back on exit, whatever happened in between."""

    def __init__(self, revision: str | None):
        self.revision = revision
        self.saved: dict[str, bytes | None] = {}

    def __enter__(self):
        if not self.revision:
            return self
        for path in changed_assets(self.revision):
            p = Path(path)
            self.saved[path] = p.read_bytes() if p.exists() else None
            old = subprocess.run(["git", "show", f"{self.revision}:{path}"], capture_output=True)
            if old.returncode == 0:
                p.write_bytes(old.stdout)
            elif p.exists():
                p.unlink()
        return self

    def __exit__(self, *exc):
        for path, content in self.saved.items():
            p = Path(path)
            if content is None:
                p.unlink(missing_ok=True)
            else:
                p.write_bytes(content)
        return False


WINDOWS = sys.platform == "win32"


def running(image: str) -> bool:
    if not WINDOWS:
        # macOS and Linux: the same names without `.exe`, matched exactly.
        name = image.removesuffix(".exe")
        return subprocess.run(["pgrep", "-x", name], capture_output=True).returncode == 0
    out = subprocess.run(["tasklist", "/FI", f"IMAGENAME eq {image}"], capture_output=True, text=True).stdout
    return image.lower() in out.lower()


def screen_locked() -> bool:
    """macOS: whether the console session's screen is locked."""
    if sys.platform != "darwin":
        return False
    out = subprocess.run(["ioreg", "-r", "-k", "IOConsoleUsers", "-d1"], capture_output=True, text=True).stdout
    return '"CGSSessionScreenIsLocked"=Yes' in out


# Anything that would share the machine with a run. A build in the background
# once made a walk read 37 ms at p50 that reads a fraction of that alone.
BUSY = ["pbd-app.exe", "cargo.exe", "rustc.exe"]
# Apps that draw on the GPU or burn CPU behind the game's back (owner,
# 2026-10-29: "close out any apps using the gpu before doing your run").
# Steam was found at 173% CPU through a whole sitting.
if sys.platform == "darwin":
    BUSY += ["steam_osx", "Google Chrome", "Spotify", "Electron", "Code"]
# Idle, these cost little; recording, they cost a lot. Warn rather than refuse.
WARN = ["obs64.exe"] if WINDOWS else ["OBS"]


STAMP = re.compile(r"^(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d+)Z", re.M)
# The line `towns.rs` `settle` logs after the multi-second planet rebuild that
# a fresh world pays once (not a cost of anything a variant changes).
REBUILD = re.compile(r"towns' ground installed and the planet rebuilt")


def log_zero(text: str) -> float | None:
    """The frame log's zero as a UTC time: the game's first log line. The
    frame log's clock starts as the app is built, within a fraction of a
    second of that line; the process launch comes several seconds earlier."""
    m = STAMP.search(text)
    return dt.datetime.fromisoformat(m[1]).replace(tzinfo=dt.timezone.utc).timestamp() if m else None


def stamp_s(line: str, zero: float | None) -> float | None:
    """A log line's time in the frame log's clock, from its UTC stamp."""
    if zero is None:
        return None
    m = STAMP.match(line)
    if not m:
        return None
    when = dt.datetime.fromisoformat(m[1]).replace(tzinfo=dt.timezone.utc).timestamp()
    return when - zero


def warm_up_start(text: str, rows: list[dict], zero: float | None, ready: re.Pattern) -> tuple[float, str]:
    """Where the warm-up starts, in the frame log's clock, and what marked it:
    the game's ready line where it prints one, else the first frame."""
    for line in text.splitlines():
        if ready.search(line):
            # `READY playable ... at frame N` (`loading-screen`): that frame's
            # own row, exactly; else the line's time.
            m = re.search(r"at frame (\d+)", line)
            if m:
                frame = int(m[1])
                at = next((r["t"] for r in rows if r.get("frame") == frame), None)
                if at is not None:
                    return at, "ready"
            t = stamp_s(line, zero)
            if t is not None:
                return t, "ready"
    return (rows[0]["t"], "first-frame") if rows else (0.0, "none")


def rebuild_frames(text: str, rows: list[dict], zero: float | None) -> set[int]:
    """The indices of the frames the planet rebuild froze: for each rebuild
    line, the longest frame over a second ending within 3 s of it."""
    out = set()
    for line in text.splitlines():
        if REBUILD.search(line):
            t = stamp_s(line, zero)
            if t is None:
                continue
            near = [(r["ms"], i) for i, r in enumerate(rows)
                    if abs(r["t"] - t) <= 3.0 and r["ms"] > 1000 and i not in out]
            if near:
                out.add(max(near)[1])
    return out


def attention_problems(text: str, skip_s: float) -> list[str]:
    """A window that lost focus or was hidden after the warm-up: paced by the
    OS, not measured. A new window reports itself unfocused for a moment
    before it takes focus, inside the warm-up, which is not a loss."""
    stamps = STAMP.findall(text)
    if not stamps:
        return []
    start = dt.datetime.fromisoformat(stamps[0])
    problems = []
    for line in text.splitlines():
        for marker, problem in (("WINDOW_FOCUS false", "the window lost focus"),
                                ("WINDOW_OCCLUDED true", "the window was hidden")):
            m = STAMP.match(line)
            if marker in line and m and (dt.datetime.fromisoformat(m[1]) - start).total_seconds() > skip_s:
                if problem not in problems:
                    problems.append(problem)
    return problems


def run_once(scenario: str, variant: dict, index: int, a) -> dict:
    tag = f"{scenario}_{variant['name']}_{index}"
    csv_path = a.out / f"{tag}.csv"
    log_path = a.out / f"{tag}.log"
    world = f"perf-{scenario}"
    saves = Path("saves") / world
    shutil.rmtree(saves, ignore_errors=True)
    cmd = [variant["exe"], *SCENARIOS[scenario], "--world", world, "--time", str(a.time),
           *([] if a.vsync else ["--no-vsync"]), "--frame-log", str(csv_path), *variant.get("args", [])]
    env = {**os.environ, **variant["env"]}
    print(f"[{dt.datetime.now():%H:%M:%S}] {tag}: {' '.join(cmd)} {variant['env'] or ''}"
          f"{' assets@' + variant['assets'] if variant.get('assets') else ''}", flush=True)
    started = time.time()
    with AssetsAt(variant.get("assets")), open(log_path, "w", encoding="utf-8", errors="replace") as log:
        proc = subprocess.Popen(cmd, stdout=log, stderr=subprocess.STDOUT, env=env)
        try:
            code = proc.wait(timeout=a.timeout)
            timed_out = False
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
            code, timed_out = None, True
    shutil.rmtree(saves, ignore_errors=True)
    return analyse(scenario, variant, index, a, csv_path, log_path, code, timed_out,
                   round(time.time() - started, 1))


def analyse(scenario: str, variant: dict, index: int, a, csv_path: Path, log_path: Path,
            code: int | None, timed_out: bool, seconds: float) -> dict:
    """One run's result from its frame log and its game log (also what
    `--reanalyse` redoes for a sitting already run)."""
    tag = f"{scenario}_{variant['name']}_{index}"
    text = ANSI.sub("", log_path.read_text(encoding="utf-8", errors="replace"))
    every = load_frames(csv_path)
    zero = log_zero(text)
    warm_from, warm_mark = warm_up_start(text, every, zero, a.ready)
    stalls = rebuild_frames(text, every, zero)
    rebuild_ms = [every[i]["ms"] for i in sorted(stalls)]
    rows = [r for i, r in enumerate(every) if r["t"] >= warm_from + a.skip_s and i not in stalls]

    problems = []
    if timed_out:
        problems.append(f"timed out after {a.timeout} s")
    elif code != 0:
        problems.append(f"exit code {code}")
    # A window out of focus or hidden is paced by the OS, not the game
    # (`frame-budget`): 60 Hz unfocused in Bevy, throttled under a locked
    # screen on macOS. Its frames say nothing about the frame's cost.
    problems += attention_problems(text, warm_from + a.skip_s)
    if DONE[scenario] not in text:
        problems.append(f"no {DONE[scenario]} line")
    if len(rows) < 300:
        problems.append(f"only {len(rows)} frames after the warm-up")
    result = {
        "scenario": scenario, "variant": variant["name"], "exe": variant["exe"], "env": variant["env"],
        "index": index, "seconds": seconds, "csv": str(csv_path), "log": str(log_path),
        "frame_ms": stats([r["ms"] for r in rows]),
        # With `--vsync`: frames that missed a refresh, i.e. took more than
        # one and a half 60 Hz intervals (a display at another rate reads
        # its own interval from the median frame).
        "missed": missed_refreshes([r["ms"] for r in rows]) if a.vsync else None,
        "gpu_total_ms": stats([r["gpu_total"] for r in rows]),
        "gpu_clouds_ms": stats([r["gpu_clouds"] for r in rows]),
        "gpu_pass_ms": {name: stats([r["passes"].get(name, math.nan) for r in rows])
                        for name in sorted({k for r in rows for k in r["passes"]})},
        "warm_up": {"from_s": round(warm_from, 2), "marked_by": warm_mark, "skip_s": a.skip_s},
        "rebuild_ms": rebuild_ms,
    }
    adapter = re.search(r'AdapterInfo \{ name: "([^"]+)".*?backend: (\w+)', text)
    if adapter:
        result["adapter"] = f"{adapter[1]} ({adapter[2]})"
    if scenario in ("clouds", "storm", "cloud-hop"):
        m = re.search(r"scenic route: into (\d+) m of cloud .*? from (\d+) m", text)
        if not m:
            problems.append("the scenic route found no cloud")
        else:
            length, start = float(m[1]), float(m[2])
            inside = [r for r in rows if start <= r["route_m"] <= start + length]
            result["cloud_leg_m"] = [start, start + length]
            result["in_cloud"] = {
                "frame_ms": stats([r["ms"] for r in inside]),
                "gpu_total_ms": stats([r["gpu_total"] for r in inside]),
                "gpu_clouds_ms": stats([r["gpu_clouds"] for r in inside]),
            }
            if len(inside) < 60:
                problems.append(f"only {len(inside)} frames on the cloud leg")
    result["valid"] = not problems
    result["problems"] = problems
    (a.out / f"{tag}.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    fm = result["frame_ms"]
    print(f"    {'VALID' if result['valid'] else 'INVALID ' + '; '.join(problems)}: "
          f"p50 {fm.get('p50', math.nan):.2f} p95 {fm.get('p95', math.nan):.2f} "
          f"p99 {fm.get('p99', math.nan):.2f} max {fm.get('max', math.nan):.1f} ms, "
          f"GPU clouds p50 {result['gpu_clouds_ms'].get('p50', math.nan):.2f} ms", flush=True)
    return result


def missed_refreshes(values: list[float]) -> dict:
    """With vsync on, the frames that held the screen for more than one
    refresh: over 1.5 refresh intervals, the interval read off the median
    frame (16.7 ms on a 60 Hz display)."""
    v = sorted(x for x in values if not math.isnan(x))
    if not v:
        return {}
    interval = pct(v, 0.5)
    late = [x for x in v if x > 1.5 * interval]
    return {"interval_ms": interval, "frames": len(v), "missed": len(late),
            "share": len(late) / len(v), "refreshes_lost": sum(round(x / interval) - 1 for x in late)}


def median(values: list[float]) -> float:
    v = sorted(x for x in values if not math.isnan(x))
    return v[len(v) // 2] if v else math.nan


def fmt(x: float, want: float | None = None) -> str:
    if math.isnan(x):
        return "-"
    mark = " **over**" if want is not None and x > want else ""
    return f"{x:.2f}{mark}"


def report(results: list[dict], a, meta: dict) -> str:
    lines = [f"# Performance suite, {meta['when']}", ""]
    lines.append(f"- revision: `{meta['revision']}`{' (uncommitted changes)' if meta['dirty'] else ''}")
    paced = "vsync on (`--vsync`)" if getattr(a, "vsync", False) else "uncapped (`--no-vsync`)"
    lines.append(f"- adapter: {meta.get('adapter', 'unknown')}; window 1440 x 900, {paced}")
    lines.append(f"- clock pinned at {a.time} h; first {a.skip_s} s after loading (the ready line, else the first frame) thrown away; "
                 f"{a.repeats} run(s) per variant, interleaved")
    lines.append(f"- budgets: p95 within {WANT_MS:.1f} ms (120 fps), p99 within {FLOOR_MS:.1f} ms (60 fps); "
                 "**over** marks a miss. Each cell is the median over the valid runs.")
    lines.append("")
    for scenario in a.scenarios:
        lines.append(f"## {scenario}")
        lines.append("")
        lines.append("| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |")
        lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
        for variant in a.variant_names:
            rs = [r for r in results if r["scenario"] == scenario and r["variant"] == variant]
            ok = [r for r in rs if r["valid"]]
            m = lambda key, sub="frame_ms", src=ok: median([r[sub].get(key, math.nan) for r in src])
            fps = 1000 / m("mean") if ok else math.nan
            lines.append(
                f"| {variant} | {len(ok)}/{len(rs)} | {fmt(fps)} | {fmt(m('p50'))} | {fmt(m('p95'), WANT_MS)} | "
                f"{fmt(m('p99'), FLOOR_MS)} | {fmt(m('p999'))} | {fmt(m('max'))} | {fmt(m('p50', 'gpu_total_ms'))} | "
                f"{fmt(m('p50', 'gpu_clouds_ms'))} / {fmt(m('p95', 'gpu_clouds_ms'))} |")
        if scenario in ("clouds", "storm", "cloud-hop"):
            lines.append("")
            lines.append("On the cloud leg only (the stretch the plan flies through cloud):")
            lines.append("")
            lines.append("| variant | fps | p50 | p95 | p99 | max | GPU total p50 / p95 | GPU clouds p50 / p95 |")
            lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
            for variant in a.variant_names:
                ok = [r["in_cloud"] for r in results
                      if r["scenario"] == scenario and r["variant"] == variant and r["valid"] and "in_cloud" in r]
                m = lambda key, sub="frame_ms": median([r[sub].get(key, math.nan) for r in ok])
                fps = 1000 / m("mean") if ok else math.nan
                lines.append(
                    f"| {variant} | {fmt(fps)} | {fmt(m('p50'))} | {fmt(m('p95'), WANT_MS)} | {fmt(m('p99'), FLOOR_MS)} | "
                    f"{fmt(m('max'))} | {fmt(m('p50', 'gpu_total_ms'))} / {fmt(m('p95', 'gpu_total_ms'))} | "
                    f"{fmt(m('p50', 'gpu_clouds_ms'))} / {fmt(m('p95', 'gpu_clouds_ms'))} |")
        if getattr(a, "vsync", False):
            lines.append("")
            lines.append("With vsync on: frames that missed a refresh (held the screen for more than one):")
            lines.append("")
            lines.append("| variant | refresh, ms | missed | share | refreshes lost |")
            lines.append("| --- | ---: | ---: | ---: | ---: |")
            for variant in a.variant_names:
                ok = [r["missed"] for r in results
                      if r["scenario"] == scenario and r["variant"] == variant and r["valid"] and r.get("missed")]
                if ok:
                    lines.append(f"| {variant} | {median([m['interval_ms'] for m in ok]):.2f} | "
                                 f"{sum(m['missed'] for m in ok)} of {sum(m['frames'] for m in ok)} | "
                                 f"{100 * sum(m['missed'] for m in ok) / sum(m['frames'] for m in ok):.2f}% | "
                                 f"{sum(m['refreshes_lost'] for m in ok)} |")
        passes = sorted({k for r in results if r["scenario"] == scenario and r["valid"]
                         for k in r.get("gpu_pass_ms", {})})
        if passes:
            lines.append("")
            lines.append("GPU time per pass, p50 (ms; median over the valid runs):")
            lines.append("")
            lines.append("| variant | " + " | ".join(passes) + " |")
            lines.append("| --- |" + " ---: |" * len(passes))
            for variant in a.variant_names:
                ok = [r for r in results if r["scenario"] == scenario and r["variant"] == variant and r["valid"]]
                cells = [fmt(median([r.get("gpu_pass_ms", {}).get(k, {}).get("p50", math.nan) for r in ok]))
                         for k in passes]
                lines.append(f"| {variant} | " + " | ".join(cells) + " |")
        stalls = [ms for r in results if r["scenario"] == scenario for ms in r.get("rebuild_ms", [])]
        if stalls:
            lines.append("")
            lines.append(f"Left out: the fresh world's planet rebuild, {len(stalls)} frame(s) of "
                         f"{min(stalls):.0f}-{max(stalls):.0f} ms.")
        bad = [r for r in results if r["scenario"] == scenario and not r["valid"]]
        for r in bad:
            lines.append(f"- invalid: {r['variant']} run {r['index']}: {'; '.join(r['problems'])}")
        lines.append(f"\n![{scenario}]({scenario}.png)\n")
    return "\n".join(lines)


def plot(results: list[dict], a) -> None:
    try:
        import matplotlib
        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
    except ImportError:
        print("matplotlib not installed: no graphs")
        return
    colours = ["#3465a4", "#cc0000", "#4e9a06", "#75507b", "#c4a000", "#555753"]
    for scenario in a.scenarios:
        rs = [r for r in results if r["scenario"] == scenario]
        if not rs:
            continue
        fig, (ax, ax2) = plt.subplots(2, 1, figsize=(16, 8), sharex=True, gridspec_kw={"height_ratios": [3, 2]})
        for r in rs:
            c = colours[a.variant_names.index(r["variant"]) % len(colours)]
            warm = r.get("warm_up", {}).get("from_s", 0.0) + a.skip_s
            rows = [x for x in load_frames(Path(r["csv"])) if x["t"] >= warm]
            t0 = rows[0]["t"] if rows else 0
            label = f"{r['variant']} #{r['index']}" + ("" if r["valid"] else " (invalid)")
            ax.plot([x["t"] - t0 for x in rows], [x["ms"] for x in rows], lw=0.5, color=c, alpha=0.7, label=label)
            ax2.plot([x["t"] - t0 for x in rows], [x["gpu_clouds"] for x in rows], lw=0.6, color=c, alpha=0.7)
            if "cloud_leg_m" in r:
                inside = [x["t"] - t0 for x in rows if r["cloud_leg_m"][0] <= x["route_m"] <= r["cloud_leg_m"][1]]
                if inside:
                    ax.axvspan(min(inside), max(inside), color=c, alpha=0.06, lw=0)
        ax.axhline(WANT_MS, color="#4e9a06", ls="--", lw=0.8, label="120 fps")
        ax.axhline(FLOOR_MS, color="#cc0000", ls="--", lw=0.8, label="60 fps")
        ax.set_ylim(0, 40)
        ax.set_ylabel("frame wall time, ms")
        ax.set_title(f"{scenario}: frame time per run (shaded: the cloud leg)")
        ax.legend(loc="upper right", fontsize=8)
        ax2.set_ylabel("GPU clouds, ms")
        ax2.set_xlabel("seconds after the warm-up")
        fig.tight_layout()
        fig.savefig(a.out / f"{scenario}.png", dpi=100)
        plt.close(fig)


def reanalyse(a) -> None:
    """Recompute every run of the sitting in `a.reanalyse` with this file's
    analysis (the warm-up, the rebuild, the pass columns), keeping how each
    run ended, and rewrite its JSON, `suite.json`, graphs and report."""
    a.out = a.reanalyse
    suite = json.loads((a.out / "suite.json").read_text(encoding="utf-8"))
    meta, old = suite["meta"], suite["results"]
    a.ready = re.compile(a.ready_pattern)
    a.scenarios = list(dict.fromkeys(r["scenario"] for r in old))
    a.variant_names = list(dict.fromkeys(r["variant"] for r in old))
    a.repeats = max(r["index"] for r in old) + 1
    a.vsync = "--vsync" in meta.get("args", [])
    variants = {v["name"]: v for v in meta["variants"]}
    results = []
    for r in old:
        problems = r.get("problems", [])
        timed_out = any(p.startswith("timed out") for p in problems)
        code = next((int(p.split()[-1]) for p in problems if p.startswith("exit code")), 0)
        tag = f"{r['scenario']}_{r['variant']}_{r['index']}"
        new = analyse(r["scenario"], variants[r["variant"]], r["index"], a,
                      a.out / f"{tag}.csv", a.out / f"{tag}.log", code, timed_out, r["seconds"])
        results.append(new)
    meta["reanalysed"] = dt.datetime.now().strftime("%Y-%m-%d %H:%M")
    (a.out / "suite.json").write_text(json.dumps({"meta": meta, "results": results}, indent=2), encoding="utf-8")
    plot(results, a)
    text = report(results, a, meta)
    (a.out / "report.md").write_text(text + "\n", encoding="utf-8")
    print(text)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--scenarios", default="clouds,storm,cloud-hop,far-side,walk")
    ap.add_argument("--variant", action="append", help="name|exe|ENV=1;ENV2=2 (repeatable)")
    ap.add_argument("--exe", default="target/release/pbd-app" + (".exe" if WINDOWS else ""), help="the exe of a variant that names none")
    ap.add_argument("--repeats", type=int, default=2)
    ap.add_argument("--time", type=float, default=10.0, help="the pinned hour: daylight, for the scenic clouds")
    ap.add_argument("--vsync", action="store_true",
                    help="present with vsync (no --no-vsync) and count the frames that missed a refresh")
    ap.add_argument("--skip-s", type=float, default=10.0, help="the warm-up, from the ready line or the first frame")
    ap.add_argument("--ready-pattern", default=r"\bLOADING_DONE\b|\bWORLD_READY\b|\bREADY\b|loading (?:is )?(?:done|complete|finished)",
                    help="a log line that marks the end of loading; the warm-up counts from it")
    ap.add_argument("--timeout", type=float, default=420.0)
    ap.add_argument("--out", type=Path, default=Path("output/perf") / dt.datetime.now().strftime("%Y%m%d-%H%M"))
    ap.add_argument("--reanalyse", type=Path, help="redo a sitting's results and report from its CSVs and logs")
    a = ap.parse_args()
    if a.reanalyse:
        reanalyse(a)
        return
    a.scenarios = [s.strip() for s in a.scenarios.split(",") if s.strip()]
    a.ready = re.compile(a.ready_pattern)
    for s in a.scenarios:
        if s not in SCENARIOS:
            raise SystemExit(f"unknown scenario {s}; known: {', '.join(SCENARIOS)}")
    variants = [parse_variant(v, a.exe) for v in (a.variant or ["current"])]
    a.variant_names = [v["name"] for v in variants]
    for v in variants:
        if not Path(v["exe"]).exists():
            raise SystemExit(f"{v['name']}: no exe at {v['exe']}")
    if screen_locked():
        raise SystemExit("the screen is locked: macOS throttles the game's window, so every run would be paced, not measured")
    if sys.platform == "darwin":
        # Hold the display awake and the user active for the whole sitting,
        # so the screen cannot sleep and lock part-way (`tools/keep_awake.sh`);
        # caffeinate ends with this process.
        subprocess.Popen(["caffeinate", "-dimsu", "-w", str(os.getpid())])
    busy = [image for image in BUSY if running(image)]
    if busy:
        raise SystemExit(f"{', '.join(busy)} running: close it, it would share the machine with the runs")
    for image in WARN:
        if running(image):
            print(f"warning: {image} is running; make sure it is idle (not recording)", flush=True)
    a.out.mkdir(parents=True, exist_ok=True)

    meta = {
        "when": dt.datetime.now().strftime("%Y-%m-%d %H:%M"),
        "revision": subprocess.run(["git", "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip(),
        "dirty": bool(subprocess.run(["git", "status", "--porcelain", "--untracked-files=no"],
                                     capture_output=True, text=True).stdout.strip()),
        "variants": variants,
        "args": sys.argv[1:],
    }
    results = []
    for scenario in a.scenarios:
        for index in range(a.repeats):
            # A, B, then B, A: each variant goes first as often as it goes last.
            order = variants if index % 2 == 0 else variants[::-1]
            for variant in order:
                results.append(run_once(scenario, variant, index, a))
    adapters = {r["adapter"] for r in results if "adapter" in r}
    if adapters:
        meta["adapter"] = ", ".join(sorted(adapters))
    (a.out / "suite.json").write_text(json.dumps({"meta": meta, "results": results}, indent=2), encoding="utf-8")
    plot(results, a)
    text = report(results, a, meta)
    (a.out / "report.md").write_text(text + "\n", encoding="utf-8")
    print()
    print(text)
    print(f"\nreport: {a.out / 'report.md'}")


if __name__ == "__main__":
    main()
