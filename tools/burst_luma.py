"""Brightness frame by frame over a burst of captures (`baked-light-ease`).

burst_luma.py <dir> <out_prefix> [name=x0,y0,x1,y1 ...]

<dir> holds the `f<frame>.png` files `PBD_CAPTURE_RUN` saves. Each patch is
given in fractions of the frame. Writes <out_prefix>.csv (per frame: the
frame number, each patch's mean Rec. 709 luma 0..255 of the sRGB pixels, and
the whole frame's mean absolute change from the frame before) and a quick
look at it, <out_prefix>-plot.png, drawn with PIL. Prints each patch's
largest frame-to-frame steps: a swap that steps shows as one, far over the
frames round it.
"""

import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw


def luma(rgb):
    return rgb[..., 0] * 0.2126 + rgb[..., 1] * 0.7152 + rgb[..., 2] * 0.0722


def main():
    src = Path(sys.argv[1])
    out = Path(sys.argv[2])
    patches = []
    for arg in sys.argv[3:]:
        name, box = arg.split("=")
        patches.append((name, [float(v) for v in box.split(",")]))
    files = sorted(src.glob("f*.png"))
    rows = []
    prev = None
    for f in files:
        img = np.asarray(Image.open(f).convert("RGB"), dtype=np.float32)
        h, w = img.shape[:2]
        y = luma(img)
        row = [int(f.stem[1:])]
        for _, (x0, y0, x1, y1) in patches:
            row.append(float(y[int(y0 * h):int(y1 * h), int(x0 * w):int(x1 * w)].mean()))
        row.append(float(np.abs(y - prev).mean()) if prev is not None else 0.0)
        prev = y
        rows.append(row)
    names = [p[0] for p in patches]
    with open(f"{out}.csv", "w") as fh:
        fh.write(",".join(["frame"] + names + ["frame_change"]) + "\n")
        for r in rows:
            fh.write(",".join(f"{v:.3f}" if isinstance(v, float) else str(v) for v in r) + "\n")
    # Plot: one line per patch, plus the frame change, over the frames.
    W, H, pad = 1200, 520, 60
    plot = Image.new("RGB", (W, H), (250, 250, 248))
    d = ImageDraw.Draw(plot)
    data = np.array([r[1:] for r in rows])
    frames = np.array([r[0] for r in rows])
    lo, hi = float(data[:, :-1].min()), float(data[:, :-1].max())
    hi = max(hi, lo + 1.0)
    colours = [(31, 119, 180), (214, 39, 40), (44, 160, 44), (148, 103, 189), (255, 127, 14)]

    def xy(i, v, lo_, hi_):
        x = pad + (frames[i] - frames[0]) / max(frames[-1] - frames[0], 1) * (W - 2 * pad)
        yv = H - pad - (v - lo_) / (hi_ - lo_) * (H - 2 * pad)
        return x, yv

    d.rectangle([pad, pad, W - pad, H - pad], outline=(180, 180, 180))
    for k, name in enumerate(names):
        pts = [xy(i, data[i, k], lo, hi) for i in range(len(rows))]
        d.line(pts, fill=colours[k % len(colours)], width=2)
        d.text((pad + 10 + 160 * k, 12), f"{name} (luma)", fill=colours[k % len(colours)])
    ch = data[:, -1]
    chi = max(float(ch.max()), 1.0)
    pts = [xy(i, ch[i], 0.0, chi * 3.0) for i in range(len(rows))]
    d.line(pts, fill=(120, 120, 120), width=1)
    d.text((pad + 10 + 160 * len(names), 12), f"frame-to-frame change (max {chi:.1f})", fill=(120, 120, 120))
    d.text((pad, H - pad + 8), f"frame {frames[0]}", fill=(60, 60, 60))
    d.text((W - pad - 80, H - pad + 8), f"frame {frames[-1]}", fill=(60, 60, 60))
    d.text((8, pad), f"{hi:.0f}", fill=(60, 60, 60))
    d.text((8, H - pad - 10), f"{lo:.0f}", fill=(60, 60, 60))
    # Never `<out>.png`: that is the name a run's own `--capture` takes.
    plot.save(f"{out}-plot.png")
    # The biggest single-frame jumps per patch.
    for k, name in enumerate(names):
        steps = np.abs(np.diff(data[:, k]))
        top = np.argsort(steps)[::-1][:5]
        print(name, "largest frame-to-frame steps:",
              ", ".join(f"{frames[i]}->{frames[i + 1]}: {data[i + 1, k] - data[i, k]:+.2f}" for i in top))
    top = np.argsort(ch)[::-1][:6]
    print("whole frame change:", ", ".join(f"{frames[i]}: {ch[i]:.2f}" for i in top))


if __name__ == "__main__":
    main()
