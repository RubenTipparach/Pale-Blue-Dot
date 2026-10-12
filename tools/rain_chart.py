#!/usr/bin/env python3
"""The `smooth-weather` chart: the rain shown at one place, before and after.

Reads the CSV `atmosphere::tests::rain_at_one_place` writes and draws an SVG:
the rain as it was shown (the cover where the simulation rains, nothing where
it does not, held for each one-second state) against the rain seen after the
change (the per-cell follower, mixed between states every frame, so drawn as
straight lines between the seconds).

Usage: tools/rain_chart.py <csv> <out.svg>
"""
import sys

SURFACE, INK, INK2, GRID = "#fcfcfb", "#0b0b0b", "#52514e", "#e4e3df"
BEFORE, AFTER = "#eb6834", "#2a78d6"


def main(src, out):
    rows = [l.strip().split(",") for l in open(src) if l[0].isdigit()]
    t = [float(r[0]) for r in rows]
    switched = [float(r[2]) for r in rows]
    seen = [float(r[3]) for r in rows]
    W, H = 960, 400
    L, R, T, B = 64, 150, 104, 52
    flips = sum((a > 0) != (b > 0) for a, b in zip(switched, switched[1:]))
    pw, ph = W - L - R, H - T - B
    tmax = t[-1]
    x = lambda s: L + s / tmax * pw
    y = lambda v: T + (1.0 - v) * ph
    svg = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" '
        f'font-family="system-ui, -apple-system, Segoe UI, sans-serif">',
        f'<rect width="{W}" height="{H}" fill="{SURFACE}"/>',
        f'<text x="{L}" y="28" font-size="17" font-weight="600" fill="{INK}">'
        "Rain shown at one place, three minutes</text>",
        f'<text x="{L}" y="50" font-size="12.5" fill="{INK2}">The place among 3,000 where the '
        "simulation's rain switched most often; cover overhead stays 1.0 throughout.</text>",
    ]
    for v in (0.0, 0.25, 0.5, 0.75, 1.0):
        svg.append(f'<line x1="{L}" x2="{L + pw}" y1="{y(v):.1f}" y2="{y(v):.1f}" '
                   f'stroke="{GRID}" stroke-width="1"/>')
        svg.append(f'<text x="{L - 8}" y="{y(v) + 4:.1f}" font-size="11.5" fill="{INK2}" '
                   f'text-anchor="end">{v:g}</text>')
    for s in range(0, int(tmax) + 1, 30):
        svg.append(f'<text x="{x(s):.1f}" y="{T + ph + 20}" font-size="11.5" fill="{INK2}" '
                   f'text-anchor="middle">{s} s</text>')
    svg.append(f'<text x="{L}" y="{T - 10}" font-size="11.5" fill="{INK2}">rain, 0 to 1</text>')
    # Before: held for each second, then switched.
    d = f"M{x(t[0]):.1f},{y(switched[0]):.1f}"
    for i in range(1, len(t)):
        d += f" H{x(t[i]):.1f} V{y(switched[i]):.1f}"
    svg.append(f'<path d="{d}" fill="none" stroke="{BEFORE}" stroke-width="2" '
               'stroke-linejoin="round"/>')
    # After: mixed between the seconds every frame.
    d = "M" + " L".join(f"{x(a):.1f},{y(b):.1f}" for a, b in zip(t, seen))
    svg.append(f'<path d="{d}" fill="none" stroke="{AFTER}" stroke-width="2.5" '
               'stroke-linejoin="round" stroke-linecap="round"/>')
    # Direct labels at the right end, in ink, with a swatch.
    for colour, label, sub, v in (
        (BEFORE, "Before", f"switched, {flips} times", switched[-1]),
        (AFTER, "After", "builds in, dies away", seen[-1]),
    ):
        yy = y(v) + 4
        if label == "Before":
            yy = y(1.0) + 4
        svg.append(f'<line x1="{L + pw + 8}" x2="{L + pw + 22}" y1="{yy - 4:.1f}" '
                   f'y2="{yy - 4:.1f}" stroke="{colour}" stroke-width="3"/>')
        svg.append(f'<text x="{L + pw + 28}" y="{yy:.1f}" font-size="12.5" font-weight="600" '
                   f'fill="{INK}">{label}</text>')
        svg.append(f'<text x="{L + pw + 28}" y="{yy + 15:.1f}" font-size="11" '
                   f'fill="{INK2}">{sub}</text>')
    # The legend, on its own row above the plot.
    lx = L
    for colour, label in ((BEFORE, "Before: the cover where it rains, held a second"),
                          (AFTER, "After: the rain seen, mixed every frame")):
        svg.append(f'<line x1="{lx}" x2="{lx + 16}" y1="68" y2="68" stroke="{colour}" '
                   'stroke-width="3"/>')
        svg.append(f'<text x="{lx + 22}" y="72" font-size="11.5" fill="{INK2}">{label}</text>')
        lx += 22 + 7 * len(label) + 28
    svg.append("</svg>")
    open(out, "w").write("\n".join(svg) + "\n")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
