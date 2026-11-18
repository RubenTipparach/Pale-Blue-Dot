"""Motion graph: read a `PBD_MOTION_TRACE` CSV and measure how smoothly things move.

A measurement instrument (`movement-smoothness`). The game writes one row per
rendered frame: the camera, the walker's physics body, the fishing float, one
fish, and the held tool, the last three also in camera space as the GPU gets
them (an f32 view matrix times an f32 world position). This reads it and
reports, per series:

- the camera's world speed per frame (|delta| / frame dt): walking at a steady
  pace this is flat; a camera moved only on fixed physics steps shows frames
  that move 0 or 2 steps' worth;
- each object's position on screen, in physical pixels, and its jitter: the
  distance of each frame's point from the mean of its four neighbours (two
  each side). Smooth motion, however fast, leaves that near 0; a thing that
  steps while the camera glides leaves a sawtooth;
- the held tool's drawn error: its camera-space point as the GPU computes it
  against where its pose puts it, in pixels.

Usage:
    python tools/motion_graph.py trace.csv [--out plot.png] [--label TEXT]
        [--height-px 1800] [--fov-deg 45]
"""
from __future__ import annotations

import argparse
import csv
import math
from pathlib import Path


def load(path: Path) -> list[dict]:
    rows = []
    with path.open(newline="") as f:
        for r in csv.DictReader(f):
            row = {}
            for k, v in r.items():
                if k in ("phase", "fish_key"):
                    row[k] = v
                    continue
                try:
                    row[k] = float(v)
                except (TypeError, ValueError):
                    row[k] = math.nan
            rows.append(row)
    return rows


def screen(x: float, y: float, z: float, focal: float) -> tuple[float, float]:
    """Camera space (-z ahead) to pixels from the centre."""
    if not (z < -0.05):
        return (math.nan, math.nan)
    return (focal * x / -z, focal * y / -z)


def residual(points: list[tuple[float, float]]) -> list[float]:
    """Each point's distance from the mean of its two neighbours each side."""
    out = [math.nan] * len(points)
    for i in range(2, len(points) - 2):
        ring = [points[i + d] for d in (-2, -1, 1, 2)] + [points[i]]
        if any(math.isnan(p[0]) for p in ring):
            continue
        mx = sum(points[i + d][0] for d in (-2, -1, 1, 2)) / 4
        my = sum(points[i + d][1] for d in (-2, -1, 1, 2)) / 4
        out[i] = math.hypot(points[i][0] - mx, points[i][1] - my)
    return out


def stats(values: list[float]) -> tuple[int, float, float, float]:
    v = sorted(x for x in values if not math.isnan(x))
    if not v:
        return (0, math.nan, math.nan, math.nan)
    rms = math.sqrt(sum(x * x for x in v) / len(v))
    return (len(v), rms, v[len(v) // 2], v[min(len(v) - 1, int(len(v) * 0.99))])


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("trace", type=Path)
    ap.add_argument("--out", type=Path)
    ap.add_argument("--label", default="")
    ap.add_argument("--height-px", type=float, default=1800.0,
                    help="physical pixels the view is tall (1440x900 at 2x: 1800)")
    ap.add_argument("--fov-deg", type=float, default=45.0, help="vertical field of view")
    ap.add_argument("--skip", type=int, default=30, help="frames dropped from the start")
    a = ap.parse_args()

    rows = load(a.trace)[a.skip:]
    focal = (a.height_px / 2) / math.tan(math.radians(a.fov_deg) / 2)
    report = [f"{a.trace.name}: {len(rows)} frames {a.label}".rstrip()]
    dts = [r["dt_s"] for r in rows]
    report.append(f"frame dt: mean {1000 * sum(dts) / len(dts):.2f} ms, "
                  f"min {1000 * min(dts):.2f}, max {1000 * max(dts):.2f}")
    steps = [int(r["steps"]) for r in rows if not math.isnan(r["steps"])]
    hist = {n: steps.count(n) for n in sorted(set(steps))}
    report.append(f"fixed steps per frame: {hist}")

    # The camera's world speed, and the body's.
    def speed(key: str) -> list[float]:
        out = [math.nan]
        for p, r in zip(rows, rows[1:]):
            d = math.dist((p[key + "_x"], p[key + "_y"], p[key + "_z"]),
                          (r[key + "_x"], r[key + "_y"], r[key + "_z"]))
            out.append(d / r["dt_s"] if r["dt_s"] > 0 else math.nan)
        return out

    cam_speed = speed("cam")
    moving = [s for s in cam_speed if not math.isnan(s) and s > 0.05]
    if moving:
        mean = sum(moving) / len(moving)
        sd = math.sqrt(sum((s - mean) ** 2 for s in moving) / len(moving))
        still = sum(1 for s in cam_speed if not math.isnan(s) and s <= 0.05)
        report.append(f"camera world speed while moving: mean {mean:.3f} m/s, sd {sd:.3f} "
                      f"({100 * sd / mean:.1f}%), frames with no camera move {still}")
    # The beat of physics steps (`movement-smoothness` design 6.5): while the
    # body walks (over 1 m/s), the camera's speed on frames that ran one
    # physics step against those that ran two. Drawn at the raw step, a
    # two-step frame moves the camera twice as far in the same time.
    body_speed = speed("body")
    by_steps: dict[int, list[float]] = {}
    for r, c, b_ in zip(rows, cam_speed, body_speed):
        if not (math.isnan(c) or math.isnan(b_)) and b_ > 1.0 and c < 100.0:
            by_steps.setdefault(int(r["steps"]), []).append(c)
    for k in sorted(by_steps):
        v = by_steps[k]
        if len(v) > 3:
            report.append(f"camera speed while walking, frames with {k} physics step(s): n {len(v)}, "
                          f"mean {sum(v) / len(v):.2f} m/s")

    series = {}
    for name in ("float", "fish"):
        pts = [screen(r[f"{name}_vx"], r[f"{name}_vy"], r[f"{name}_vz"], focal) for r in rows]
        series[name] = (pts, residual(pts))
    def hand_offsets(prefix: str) -> list[tuple[float, float]]:
        """The tool's drawn point against its pose, on screen, px (x, y)."""
        out = []
        for r in rows:
            v = (r[prefix + "x"], r[prefix + "y"], r[prefix + "z"])
            l = (r["hand_lx"], r["hand_ly"], r["hand_lz"])
            if any(math.isnan(c) for c in v + l) or l[2] > -0.05:
                out.append((math.nan, math.nan))
                continue
            a_ = screen(*v, focal)
            b_ = screen(*l, focal)
            out.append((a_[0] - b_[0], a_[1] - b_[1]))
        return out

    def shake(offsets: list[tuple[float, float]]) -> list[float]:
        """How far the tool's error moves from one frame to the next, px:
        a constant error is invisible, a changing one is the shimmer."""
        out = [math.nan]
        for p, q in zip(offsets, offsets[1:]):
            out.append(math.hypot(q[0] - p[0], q[1] - p[1]))
        return out

    hand_now = shake(hand_offsets("hand_v"))
    has_rebased = "hand_rx" in rows[0]
    hand_rebased = shake(hand_offsets("hand_r")) if has_rebased else []
    for name, (_, res) in series.items():
        n, rms, p50, p99 = stats(res)
        report.append(f"{name} on screen, jitter (px from its neighbours' mean): "
                      f"n {n}, rms {rms:.3f}, p50 {p50:.3f}, p99 {p99:.3f}")
    floating = "origin_x" in rows[0]
    drawn_label = ("floating render origin (as drawn)" if floating
                   else "origin at the planet's centre (as drawn)")
    n, rms, p50, p99 = stats(hand_now)
    report.append(f"held tool shimmer, {drawn_label}: n {n}, "
                  f"rms {rms:.4f} px, p50 {p50:.4f}, p99 {p99:.4f} px")
    if has_rebased:
        n, rms, p50, p99 = stats(hand_rebased)
        report.append(f"held tool shimmer, origin rebased to the camera (16 m grid): n {n}, "
                      f"rms {rms:.4f} px, p50 {p50:.4f}, p99 {p99:.4f} px")
    # The drawn fish's own world motion: a fish drawn at a slower clock's
    # raw step stands still for whole frames and then jumps, so its speed
    # frame by frame spreads wide; drawn between steps, it glides.
    still, fish_speed = 0, []
    fish_speed_series = [math.nan]
    for p, r in zip(rows, rows[1:]):
        a_ = (p["fish_x"], p["fish_y"], p["fish_z"])
        b_ = (r["fish_x"], r["fish_y"], r["fish_z"])
        if any(math.isnan(c) for c in a_ + b_) or r["dt_s"] <= 0:
            fish_speed_series.append(math.nan)
            continue
        d = math.dist(a_, b_)
        if d > 1.0:  # another fish picked, or one caught: not motion
            fish_speed_series.append(math.nan)
            continue
        fish_speed_series.append(d / r["dt_s"])
        if d < 1e-4:
            still += 1
        else:
            fish_speed.append(d / r["dt_s"])
    if fish_speed:
        v = sorted(fish_speed)
        q = lambda f: v[min(len(v) - 1, int(len(v) * f))]
        report.append(f"fish drawn: frames it did not move {still} of {still + len(v)}; "
                      f"speed per frame p10 {q(0.1):.3f}, p50 {q(0.5):.3f}, p90 {q(0.9):.3f} m/s "
                      f"(p90/p10 {q(0.9) / max(q(0.1), 1e-6):.1f})")
    # The hooked float and the line's start (`movement-smoothness` 7): the
    # float's jitter while a fish is on, the frames it jumps (over 20 px from
    # its neighbours) and its largest move in one frame; and how far the
    # line's start is drawn from the rod's drawn tip.
    float_res = series["float"][1] if "float" in series else []
    hooked = [i for i, r in enumerate(rows) if r["phase"] == "Hooked"]
    if hooked and float_res:
        n, rms, p50, p99 = stats([float_res[i] for i in hooked])
        jumps = sum(1 for i in hooked if not math.isnan(float_res[i]) and float_res[i] > 20)
        moves = [math.dist((rows[i]["float_x"], rows[i]["float_y"], rows[i]["float_z"]),
                           (rows[i - 1]["float_x"], rows[i - 1]["float_y"], rows[i - 1]["float_z"]))
                 for i in hooked if i > 0 and rows[i - 1]["phase"] == "Hooked"]
        report.append(f"hooked float on screen, jitter: n {n}, rms {rms:.3f}, p50 {p50:.3f}, "
                      f"p99 {p99:.3f} px; frames over 20 px {jumps}; "
                      f"largest move in a frame {100 * max(moves, default=0):.1f} cm")
    # The fish's drawn pose (`movement-smoothness` 8): how far its forward
    # and its up turn from one frame to the next, the same fish both frames,
    # split into the fish at the lure and fish swimming free.
    if "fish_fx" in rows[0]:
        def angle(a: tuple, b: tuple) -> float:
            na, nb = math.hypot(*a), math.hypot(*b)
            if not (na > 0 and nb > 0) or math.isnan(na) or math.isnan(nb):
                return math.nan
            d = sum(x * y for x, y in zip(a, b)) / (na * nb)
            return math.degrees(math.acos(max(-1.0, min(1.0, d))))
        turns = {"at the lure": [], "swimming": []}
        for i in range(1, len(rows)):
            r, p = rows[i], rows[i - 1]
            if not r["fish_key"] or r["fish_key"] != p["fish_key"]:
                continue
            f = angle((r["fish_fx"], r["fish_fy"], r["fish_fz"]),
                      (p["fish_fx"], p["fish_fy"], p["fish_fz"]))
            u = angle((r["fish_ux"], r["fish_uy"], r["fish_uz"]),
                      (p["fish_ux"], p["fish_uy"], p["fish_uz"]))
            if math.isnan(f):
                continue
            group = "at the lure" if r["fish_held"] > 0.5 and p["fish_held"] > 0.5 else "swimming"
            turns[group].append(max(f, 0.0 if math.isnan(u) else u))
        for group, t in turns.items():
            if not t:
                continue
            t = sorted(t)
            q = lambda f: t[min(len(t) - 1, int(f * len(t)))]
            # Pitch: how far the forward stands out of level, at the lure.
            report.append(f"fish turn per frame, {group}: n {len(t)}, p50 {q(0.5):.2f}, "
                          f"p99 {q(0.99):.2f}, max {t[-1]:.1f} deg; "
                          f"frames over 30 deg {sum(1 for x in t if x > 30)}, "
                          f"over 10 deg {sum(1 for x in t if x > 10)}")
        held = [r for r in rows if r["fish_held"] > 0.5 and not math.isnan(r["fish_fx"])]
        if held:
            def pitch(r: dict) -> float:
                up = (r["fish_x"], r["fish_y"], r["fish_z"])
                return 90.0 - angle((r["fish_fx"], r["fish_fy"], r["fish_fz"]), up)
            ps = [abs(pitch(r)) for r in held]
            report.append(f"fish at the lure, nose out of level: max {max(ps):.1f} deg; "
                          f"frames over 30 deg {sum(1 for x in ps if x > 30)} of {len(ps)}")
    if "mouth_lure_m" in rows[0]:
        report.extend(behaviour(rows))
    if "line_vx" in rows[0]:
        gaps = []
        for r in rows:
            line = screen(r["line_vx"], r["line_vy"], r["line_vz"], focal)
            tip = screen(r["tip_vx"], r["tip_vy"], r["tip_vz"], focal)
            if not (math.isnan(line[0]) or math.isnan(tip[0])):
                gaps.append(math.hypot(line[0] - tip[0], line[1] - tip[1]))
        n, rms, p50, p99 = stats(gaps)
        report.append(f"line start from the drawn rod tip: n {n}, rms {rms:.2f}, "
                      f"p50 {p50:.2f}, p99 {p99:.2f}, max {max(gaps, default=math.nan):.2f} px")
    # With a floating origin the trace writes it (physics space, whose
    # centre is the planet's in play); before, the origin was the centre.
    o = ("origin_x", "origin_y", "origin_z") if "origin_x" in rows[0] else None
    radius = [math.dist((r[o[0]], r[o[1]], r[o[2]]) if o else (0, 0, 0),
                        (r["cam_x"], r["cam_y"], r["cam_z"])) for r in rows]
    rmax = max(radius)
    ulp = 2.0 ** (math.floor(math.log2(rmax)) - 23)
    report.append(f"camera distance from the render origin: up to {rmax:.1f} m, "
                  f"f32 step there {1000 * ulp:.3f} mm")
    text = "\n".join(report)
    print(text)
    out = a.out or a.trace.with_suffix(".png")
    out.with_suffix(".txt").write_text(text + "\n", encoding="utf-8")

    try:
        import matplotlib
        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
    except ImportError:
        print("matplotlib missing: no plot")
        return
    t = [r["wall_s"] - rows[0]["wall_s"] for r in rows]
    fig, ax = plt.subplots(4, 1, figsize=(12, 11), sharex=True)
    ax[0].plot(t, [1000 * d for d in dts], lw=0.7, color="#555")
    ax[0].set_ylabel("frame dt, ms")
    ax0b = ax[0].twinx()
    ax0b.step(t, [r["steps"] for r in rows], where="mid", lw=0.6, color="#c55", alpha=0.6)
    ax0b.set_ylabel("physics steps", color="#c55")
    ax[1].plot(t, cam_speed, lw=0.7, color="#2a6", label="camera")
    ax[1].plot(t, fish_speed_series, lw=0.7, color="#c63", label="fish (drawn)")
    ax[1].set_ylabel("speed per frame, m/s")
    ax[1].legend(loc="upper right")
    for name, colour in (("float", "#36c"), ("fish", "#c63")):
        ax[2].plot(t, series[name][1], lw=0.7, color=colour, label=name)
    ax[2].set_ylabel("jitter on screen, px")
    # A new fish picked (or one caught) is a jump of thousands of pixels,
    # not jitter: keep the scale on the jitter.
    ax[2].set_ylim(0, 40)
    ax[2].legend(loc="upper right")
    ax[3].plot(t, hand_now, lw=0.7, color="#839",
               label="floating origin, as drawn" if "origin_x" in rows[0] else "origin at planet centre")
    if has_rebased:
        ax[3].plot(t, hand_rebased, lw=0.9, color="#2a6", label="origin rebased to camera")
    ax[3].legend(loc="upper right")
    ax[3].set_ylabel("held tool shimmer, px/frame")
    ax[3].set_xlabel("seconds")
    fig.suptitle(f"{a.trace.stem} {a.label}".strip())
    fig.tight_layout()
    fig.savefig(out, dpi=90)
    print(f"wrote {out}")


def behaviour(rows: list[dict]) -> list[str]:
    """`fish-behaviour`: the fish the line works, frame by frame, by the
    line's phase. Flips: frames its forward turns over 20 deg, or its up
    (its roll, with the turn) over 20 deg. Pops: frames it moves over 5 cm
    (local, the same fish both frames), and the move on the frame a nibble
    starts. At the dips: the nearest its drawn mouth comes to the drawn lure
    in each nibble's dip (the last 0.12 s of a nibble's timer)."""
    def angle(a: tuple, b: tuple) -> float:
        na, nb = math.hypot(*a), math.hypot(*b)
        if not (na > 0 and nb > 0) or math.isnan(na) or math.isnan(nb):
            return math.nan
        d = sum(x * y for x, y in zip(a, b)) / (na * nb)
        return math.degrees(math.acos(max(-1.0, min(1.0, d))))
    out = []
    phases = ["Floating", "Nibble", "Bite", "Hooked"]
    turn = {p: [] for p in phases}
    roll = {p: [] for p in phases}
    step = {p: [] for p in phases}
    nibble_start = []
    for i in range(1, len(rows)):
        r, p = rows[i], rows[i - 1]
        if not r["fish_key"] or r["fish_key"] != p["fish_key"]:
            continue
        ph = r["phase"]
        if ph not in turn:
            continue
        # Only the fish the line works while it does (its key stays the
        # line's target from interest to landing).
        if r.get("targeted", 1) < 0.5 or p.get("targeted", 1) < 0.5:
            continue
        f = angle((r["fish_fx"], r["fish_fy"], r["fish_fz"]), (p["fish_fx"], p["fish_fy"], p["fish_fz"]))
        u = angle((r["fish_ux"], r["fish_uy"], r["fish_uz"]), (p["fish_ux"], p["fish_uy"], p["fish_uz"]))
        d = math.dist((r["fish_lx"], r["fish_ly"], r["fish_lz"]), (p["fish_lx"], p["fish_ly"], p["fish_lz"]))
        if not math.isnan(f):
            turn[ph].append(f)
        if not math.isnan(u):
            roll[ph].append(u)
        if not math.isnan(d) and d < 2.0:
            step[ph].append(d)
            if ph == "Nibble" and p["phase"] == "Floating":
                nibble_start.append(d)
    for ph in phases:
        t, u, s = turn[ph], roll[ph], step[ph]
        if not t:
            continue
        out.append(f"line fish, {ph}: frames {len(t)}; forward turn max {max(t):.1f} deg, "
                   f"over 20 deg {sum(1 for x in t if x > 20)}; up turn max {max(u, default=0):.1f} deg, "
                   f"over 20 deg {sum(1 for x in u if x > 20)}; move max {100 * max(s, default=0):.1f} cm, "
                   f"over 5 cm {sum(1 for x in s if x > 0.05)}")
    if nibble_start:
        out.append("line fish, move on the frame the nibble starts: "
                   + ", ".join(f"{100 * d:.1f} cm" for d in nibble_start))
    # Each nibble's dip: the frames of a nibble whose timer is under 0.12 s,
    # cut where the timer jumps up (a new nibble).
    dips, cur = [], []
    for i, r in enumerate(rows):
        dip = r["phase"] == "Nibble" and r["timer"] < 0.12
        if dip and cur and r["timer"] > rows[i - 1]["timer"] + 1e-4:
            dips.append(cur)
            cur = []
        if dip:
            cur.append(r["mouth_lure_m"])
        elif cur:
            dips.append(cur)
            cur = []
    if cur:
        dips.append(cur)
    if dips:
        near = [min((x for x in d if not math.isnan(x)), default=math.nan) for d in dips]
        out.append("line fish, nearest mouth to lure in each nibble's dip: "
                   + ", ".join(f"{100 * x:.1f} cm" for x in near))
    return out


if __name__ == "__main__":
    main()
