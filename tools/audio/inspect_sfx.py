#!/usr/bin/env python3
"""Look at sounds without ears: the one place Pale Blue Dot's sound effects are measured.

A cloud session cannot listen, so choosing a recording and judging a cut are done by measurement
(the sound-effects skill). Three jobs:

- inventory: every recording under a fetched pack folder with its length and spectral centroid,
  to shortlist files by name, length and brightness;
- envelope REF ...: a 0.1 s loudness strip per 'pack:file' (the cc0_cues.json form), with the share
  of energy under 250 Hz and over 4 kHz, to find a take's start, its shots and its weight;
- check: every built cue in output/sfx: no clipping, no non-finite samples, loop seams no bigger than
  a few ordinary sample steps, how much it rings (a shield, an energy field, may not ring: NO_RING), and a spectrogram contact sheet (output/sfx/sheet.png) to look at
  before a set is called done.

Ported from Fleet Command (tools/audio/inspect_sfx.py at 08374c4, written there 2026-11-04). Changed
here: the sample rate is read from each WAV (48 kHz here) rather than assumed, no cue has a NO_RING cap
yet, and the default source folder is output/sfx-src.

Usage: python3 tools/audio/inspect_sfx.py inventory [--src DIR]
       python3 tools/audio/inspect_sfx.py envelope [--src DIR] REF ...
       python3 tools/audio/inspect_sfx.py check [--out DIR]
Needs: python3 with numpy, ffmpeg.
"""
import argparse
import glob
import json
import os
import subprocess
import sys
import wave

import numpy as np

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
AUDIO = (".wav", ".ogg", ".mp3", ".flac")
SR = 22050  # analysis rate: enough for the centroid and the bands


def read(path: str) -> np.ndarray:
    raw = subprocess.run(["ffmpeg", "-v", "error", "-i", path, "-ac", "1", "-ar", str(SR), "-f", "f32le", "-"],
                         capture_output=True, check=True).stdout
    return np.frombuffer(raw, np.float32).astype(np.float64)


def find(src: str, ref: str) -> str:
    pack, _, name = ref.partition(":")
    for root, _, files in os.walk(os.path.join(src, pack)):
        if name in files:
            return os.path.join(root, name)
    raise FileNotFoundError(ref)


def inventory(src: str) -> int:
    for site in sorted(os.listdir(src)):
        if not os.path.isdir(os.path.join(src, site)):
            continue
        for pack in sorted(os.listdir(os.path.join(src, site))):
            rows = []
            for root, _, files in os.walk(os.path.join(src, site, pack)):
                for f in sorted(files):
                    if not f.lower().endswith(AUDIO) or f.startswith("._"):
                        continue
                    x = read(os.path.join(root, f))
                    if len(x) < 100:
                        continue
                    seg = x[: SR * 4]
                    sp = np.abs(np.fft.rfft(seg))
                    fr = np.fft.rfftfreq(len(seg), 1 / SR)
                    rows.append(f"{f}({len(x) / SR:.1f}s,{(sp * fr).sum() / (sp.sum() or 1) / 1000:.1f}k)")
            print(f"## {site}/{pack} [{len(rows)}]: " + " ".join(rows))
    return 0


def envelope(src: str, refs) -> int:
    for ref in refs:
        x = read(find(src, ref))
        step = SR // 10
        rms = [np.sqrt(np.mean(x[i:i + step] ** 2)) for i in range(0, len(x), step)]
        top = max(rms) or 1.0
        strip = "".join(" .:-=+*#%@"[min(9, int(r / top * 9.99))] for r in rms)
        sp = np.abs(np.fft.rfft(x))
        fr = np.fft.rfftfreq(len(x), 1 / SR)
        lo, hi = sp[fr < 250].sum() / sp.sum(), sp[fr > 4000].sum() / sp.sum()
        print(f"{ref} {len(x) / SR:.1f}s low {lo:.2f} high {hi:.2f} |{strip}|")
    return 0


def ringing(x: np.ndarray, rate: int) -> float:
    """How much a sound rings after its attack, 0 to 1: in the window 60-400 ms after its peak, the share of energy (over
    150 Hz) in its eight strongest narrow spectral peaks. A thud, a whump or noise scores low; a struck plate, glass or a
    bell scores high (owner, 2026-11-11, of the shield hit: "IT IS NOT GLASS< NOT CERAMIC< NOT METAL< IT CANNOT DING")."""
    i = int(np.argmax(np.abs(x)))
    seg = x[i + int(0.06 * rate): i + int(0.4 * rate)]
    if len(seg) < 1000:
        return 0.0
    sp = np.abs(np.fft.rfft(seg * np.hanning(len(seg)))) ** 2
    f = np.fft.rfftfreq(len(seg), 1 / rate)
    m = f > 150
    s, ff = sp[m], f[m]
    taken, e = [], 0.0
    for j in np.argsort(s)[::-1]:
        if all(abs(ff[j] - t) > 40 for t in taken):
            taken.append(ff[j])
            e += s[max(0, j - 2): j + 3].sum()
        if len(taken) == 8:
            break
    return float(e / (s.sum() or 1.0))


# cues that must not ring (more than this share is refused); Fleet Command's were its shield sounds
NO_RING = {}


def check(out: str) -> int:
    manifest = {m["name"]: m for m in json.load(open(os.path.join(out, "sfx.json")))}
    status = 0
    for path in sorted(glob.glob(os.path.join(out, "*.wav"))):
        name = os.path.basename(path)[:-4]
        with wave.open(path) as w:
            rate = w.getframerate()
            x = np.frombuffer(w.readframes(w.getnframes()), "<i2").astype(np.float64).reshape(-1, w.getnchannels())
        clipped = int((np.abs(x) >= 32767).sum())
        problems = []
        if clipped > 4:
            problems.append(f"{clipped} clipped samples")
        if manifest.get(name, {}).get("loop"):
            seam = np.abs(x[0] - x[-1]).max()
            step = np.median(np.abs(np.diff(x, axis=0))) or 1.0
            if seam > 6 * step:
                problems.append(f"loop seam {seam:.0f} against a typical step of {step:.0f}")
        ring = ringing(x.mean(axis=1) / 32768.0, rate)
        if name in NO_RING and ring > NO_RING[name]:
            problems.append(f"rings ({100 * ring:.0f}%, at most {100 * NO_RING[name]:.0f}%): it may not ring")
        print(f"{name:20} {len(x) / rate:4.1f} s  ringing {100 * ring:4.1f}%  {'ok' if not problems else '; '.join(problems)}")
        status |= bool(problems)
    cells = os.path.join(out, "sheet")
    os.makedirs(cells, exist_ok=True)
    for path in sorted(glob.glob(os.path.join(out, "*.wav"))):
        name = os.path.basename(path)[:-4]
        subprocess.run(["ffmpeg", "-v", "error", "-y", "-i", path, "-lavfi",
                        "showspectrumpic=s=400x160:legend=0:scale=log:fscale=log:color=intensity,"
                        f"drawtext=text='{name}':x=4:y=4:fontcolor=white:fontsize=16",
                        os.path.join(cells, name + ".png")], check=True)
    n = len(glob.glob(os.path.join(cells, "*.png")))
    rows = -(-n // 5)
    subprocess.run(["ffmpeg", "-v", "error", "-y", "-pattern_type", "glob", "-i", os.path.join(cells, "*.png"),
                    "-vf", f"scale=400:200,tile=5x{rows}", "-frames:v", "1", os.path.join(out, "sheet.png")], check=True)
    print(f"contact sheet: {os.path.relpath(os.path.join(out, 'sheet.png'), REPO)} ({n} cues); look at it")
    return status


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("job", choices=["inventory", "envelope", "check"])
    ap.add_argument("refs", nargs="*")
    ap.add_argument("--src", default=os.environ.get("SFX_SRC", os.path.join(REPO, "output", "sfx-src")))
    ap.add_argument("--out", default=os.path.join(REPO, "output/sfx"))
    a = ap.parse_args()
    if a.job == "inventory":
        return inventory(a.src)
    if a.job == "envelope":
        return envelope(a.src, a.refs)
    return check(a.out)


if __name__ == "__main__":
    sys.exit(main())
