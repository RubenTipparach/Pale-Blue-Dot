#!/usr/bin/env python3
"""MEASUREMENT (`movement-smoothness` design 9.1): read a `PBD_STEP_TRACE`
CSV and measure how each fixed step's move of the walker's stored position
differs from `velocity * dt`.

    tools/step_graph.py steps.csv [--speed 1.0] [--excerpt 8]

Only steps of a steady walk are counted: speed within 10% of `--speed` and
the velocity changed by under 1% since the step before, so a jump or a turn
is not read as rounding. The move is taken in the planet frame (`world_*`,
rebuilt in f64 from the origin and the stored local position), so a rebase
(the origin moving) is not a move. Prints the error per step (|move - v dt|)
and as a fraction of the step, and an excerpt of rows at full precision.
"""
import argparse
import csv
import math
import statistics


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("csv")
    ap.add_argument("--speed", type=float, default=1.0)
    ap.add_argument("--excerpt", type=int, default=8)
    ap.add_argument(
        "--from-tick",
        type=int,
        default=0,
        help="skip steps before this tick (the coarse ground before the column tier lands)",
    )
    a = ap.parse_args()
    rows = list(csv.DictReader(open(a.csv)))
    f = lambda r, k: float(r[k])
    errs, fracs, lateral, kept = [], [], [], []
    rebases = 0
    for prev, cur in zip(rows, rows[1:]):
        if int(cur["tick"]) != int(prev["tick"]) + 1 or int(cur["tick"]) < a.from_tick:
            continue
        if any(f(cur, f"origin_{c}") != f(prev, f"origin_{c}") for c in "xyz"):
            rebases += 1
        v = [f(cur, f"v{c}") for c in "xyz"]
        vp = [f(prev, f"v{c}") for c in "xyz"]
        speed = math.sqrt(sum(c * c for c in v))
        dv = math.sqrt(sum((x - y) ** 2 for x, y in zip(v, vp)))
        if abs(speed - a.speed) > 0.1 * a.speed or dv > 0.01 * speed:
            continue
        dt = f(cur, "dt_s")
        move = [f(cur, f"world_{c}") - f(prev, f"world_{c}") for c in "xyz"]
        ideal = [x * dt for x in v]
        e = [m - i for m, i in zip(move, ideal)]
        err = math.sqrt(sum(c * c for c in e))
        step = math.sqrt(sum(c * c for c in ideal))
        # Along the motion vs across it (across includes up/down).
        unit = [c / step for c in ideal]
        along = sum(x * u for x, u in zip(e, unit))
        across = math.sqrt(max(err * err - along * along, 0.0))
        errs.append(err)
        fracs.append(err / step)
        lateral.append(across)
        kept.append((prev, cur, move, ideal, err))
    if not errs:
        print("no steady steps")
        return
    q = lambda xs, p: sorted(xs)[min(len(xs) - 1, int(p * len(xs)))]
    print(f"steady steps: {len(errs)} at {a.speed} m/s; rebases in trace: {rebases}")
    print(
        f"|move - v dt| m: mean {statistics.mean(errs):.3e}  p50 {q(errs, .5):.3e}  "
        f"p99 {q(errs, .99):.3e}  max {max(errs):.3e}"
    )
    print(
        f"as a fraction of the step: mean {100 * statistics.mean(fracs):.2f}%  "
        f"p50 {100 * q(fracs, .5):.2f}%  max {100 * max(fracs):.2f}%"
    )
    print(f"across the motion (incl. vertical) m: p50 {q(lateral, .5):.3e}  max {max(lateral):.3e}")
    print()
    print("tick  dt          x (stored)       y (stored)       z (stored)       "
          "world dx       world dy       world dz       v*dt x         |err| m")
    for prev, cur, move, ideal, err in kept[: a.excerpt]:
        print(
            f"{cur['tick']:>5} {f(cur, 'dt_s'):.9f} {cur['x']:>16} {cur['y']:>16} {cur['z']:>16} "
            f"{move[0]:+.3e} {move[1]:+.3e} {move[2]:+.3e} {ideal[0]:+.3e} {err:.2e}"
        )


if __name__ == "__main__":
    main()
