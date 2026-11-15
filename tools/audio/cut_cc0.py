#!/usr/bin/env python3
"""Cut Pale Blue Dot's recorded sound effects and ambience beds from CC0 and public domain originals.

Ported from Pale Blue Dot (tools/audio/cut_cc0.py at 08374c4, itself after Crab Breeze's at
0e569e3). Changed here: the default source folder (output/sfx-src, git-ignored); `takes`, one cue
written as several files (name_1, name_2, ...), because a footstep or a dig is heard many times
running and the game picks a different take each time (openspec/changes/audio-and-ambience,
design section 4); and the sources tables name each take's cue once.

tools/audio/cc0_cues.json says, for each recorded cue, which original files to layer and how
(start, length, which shot of a multi-shot take, pitch as a tape-speed ratio, gain, steep lowpass
and highpass, fades, delay), and may add a synthesised layer from tools/audio/gen_sfx.py where a
recording lacks weight. This script applies it over the synthesised set gen_sfx.py wrote: the cue's
WAV in --out is replaced, its sfx.json entry updated, and assets/audio/sfx/SOURCES.md rewritten
with one row per cue naming every original it is cut from.

A cue with `takes` (a list of objects, each merged over the cue's first layer, or over the whole
cue when it has a `layers` key of its own) is cut once per take into <name>_<k>.wav.

The originals are fetched by tools/audio/fetch_cc0.py into --src and are not committed; the cuts
are. A cue whose originals are missing is left synthesised and reported.

Usage: python3 tools/audio/cut_cc0.py [--src DIR] [--out DIR] [cue ...]
"""
import argparse
import json
import os
import subprocess
import sys
import zlib

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
SRC_DEFAULT = os.environ.get("SFX_SRC", os.path.join(REPO, "output", "sfx-src"))
sys.path.insert(0, HERE)
import gen_sfx as g  # noqa: E402  (the shared filters, writer and synthesised cues)

SR = g.SR


def find(src_root: str, ref: str) -> str:
    """'pack:file' resolved to a path: the file is searched for anywhere under the pack's folder,
    so a pack's own zip layout does not leak into the spec."""
    pack, _, name = ref.partition(":")
    base = os.path.join(src_root, pack)
    for root, _, files in os.walk(base):
        if name in files:
            return os.path.join(root, name)
    raise FileNotFoundError(f"{ref} (looked under {base})")


def load(path: str, channels: int = 1) -> np.ndarray:
    """The file as float samples, shape (channels, n): a mono file is copied to both sides."""
    raw = subprocess.run(["ffmpeg", "-v", "error", "-i", path, "-ac", str(channels), "-ar", str(SR), "-f", "f32le", "-"],
                         capture_output=True, check=True).stdout
    return np.frombuffer(raw, np.float32).astype(np.float64).reshape(-1, channels).T


def shot_start(x: np.ndarray, k: int, gap_s: float = 0.25) -> float:
    """Where the k-th shot (from 1) of a multi-shot take begins: a rise above a fifth of the take's
    peak after at least gap_s below it."""
    env = np.convolve(np.abs(x), np.ones(64) / 64, "same")
    hot = env > env.max() * 0.2
    starts, last = [], -10 * SR
    for i in np.flatnonzero(hot):
        if i - last > gap_s * SR:
            starts.append(i)
        last = i
    if k > len(starts):
        raise ValueError(f"asked for shot {k}, the take has {len(starts)}")
    return max(0, starts[k - 1] - int(0.005 * SR)) / SR


def varispeed(x: np.ndarray, ratio: float) -> np.ndarray:
    """Tape-style pitch: ratio below 1 is lower and longer, as a bigger machine would sound."""
    if ratio == 1:
        return x
    n = int(x.shape[-1] / ratio)
    return np.stack([np.interp(np.arange(n) * ratio, np.arange(x.shape[-1]), c) for c in x])


def fades(x: np.ndarray, fin: float, fout: float) -> np.ndarray:
    x = x.copy()
    n = x.shape[-1]
    a, b = min(n, int(fin * SR)), min(n, int(fout * SR))
    if a:
        x[..., :a] *= np.linspace(0, 1, a)
    if b:
        x[..., -b:] *= np.linspace(1, 0, b) ** 2
    return x


def per_channel(fn, x: np.ndarray, *a, **k) -> np.ndarray:
    return np.stack([fn(c, *a, **k) for c in x])


def fit(x: np.ndarray, n: int) -> np.ndarray:
    return np.pad(x, ((0, 0), (0, max(0, n - x.shape[-1]))))[:, :n]


def layer(spec: dict, src_root: str, channels: int) -> np.ndarray:
    """One layer as (channels, n). pan (-1 left to 1 right, equal power) places it in a stereo cue."""
    if "synth" in spec:
        group, what, fn, _ = g.SOUNDS[spec["synth"]]
        x = np.repeat(fn(g.rng_for(spec["synth"]))[None, :], channels, axis=0)
    else:
        x = load(find(src_root, spec["src"]), channels)
        start = shot_start(x.mean(axis=0), spec["shot"]) + spec.get("start", 0.0) if "shot" in spec else spec.get("start", 0.0)
        x = x[:, int(start * SR):]
        x = varispeed(x, spec.get("pitch", 1.0))
        if spec.get("repeat", 1) > 1:
            x = np.tile(x, (1, spec["repeat"]))
    if "dur" in spec:
        x = x[:, : int(spec["dur"] * SR)]
    if "hp" in spec:
        x = per_channel(g.hp, x, spec["hp"])
    if "lp" in spec:
        x = per_channel(g.lp, x, spec["lp"], order=spec.get("lp_order", 4))
    x = fades(x, spec.get("fadein", 0.003), spec.get("fadeout", 0.01))
    x = x / (np.max(np.abs(x)) or 1.0) * 10 ** (spec.get("gain", 0.0) / 20)
    if channels == 2 and "pan" in spec:
        a = (spec["pan"] + 1) * np.pi / 4
        x = x * np.array([[np.cos(a)], [np.sin(a)]]) * np.sqrt(2)
    return np.concatenate([np.zeros((channels, int(spec.get("delay", 0.0) * SR))), x], axis=1)


def smooth(x: np.ndarray, seconds: float) -> np.ndarray:
    """A moving average over `seconds`, centred as np.convolve's "same" is. Through an FFT where scipy
    is there (changed here: a direct 0.3 s window over a 40 s bed took a minute a bed)."""
    w = max(1, int(seconds * SR))
    if len(x) >= w:
        try:
            from scipy.signal import fftconvolve
            return fftconvolve(x, np.ones(w) / w, "same")
        except ImportError:  # pragma: no cover
            pass
    return np.convolve(x, np.ones(w) / w, "same")


def compress(x: np.ndarray, threshold_db: float = -18.0, ratio: float = 3.0) -> np.ndarray:
    """Evens out a bed so no bird, car or shout jumps forward: a 300 ms RMS follower, gain above the
    threshold reduced by the ratio, the gain itself smoothed over 200 ms. x is (channels, n)."""
    peak = np.max(np.abs(x)) or 1.0
    env = np.sqrt(smooth((x / peak).mean(axis=0) ** 2, 0.3)) + 1e-9
    thr = 10 ** (threshold_db / 20)
    gain = np.minimum(1.0, (thr / env) ** (1 - 1 / ratio))
    return x * smooth(gain, 0.2)


def wrapped_reverb(x: np.ndarray, wet: float, room: float) -> np.ndarray:
    """gen_sfx's reverb with its tail folded back onto the start, so a loop stays seamless."""
    n = len(x)
    r = g.reverb(x, room=room, wet=wet, tail_s=1.5)
    y = r[:n].copy()
    y[: len(r) - n] += r[n:]
    return y


def distance(x: np.ndarray, spec: dict) -> np.ndarray:
    """Pushes a bed into the background: compressed, lowpassed, then a diffuse reverb, slightly
    different on each side so the bed is wide and nothing in it sounds close."""
    x = compress(x, spec.get("threshold_db", -18.0), spec.get("ratio", 3.0))
    x = per_channel(g.lp, x, spec.get("lp", 3500), order=2)
    rooms = (0.8, 0.88)
    return np.stack([wrapped_reverb(c, spec.get("wet", 0.35), rooms[i % 2]) for i, c in enumerate(x)])


def scattered(cue: dict, name: str) -> list:
    """A cue's "scatter" groups expanded into layers: each group places `count` short sounds, picked from
    `pick` (each a layer spec, recorded or synthesised), at seeded times inside `within` seconds and no
    closer than `min_gap`, with a gain in dB, a pan and a pitch drawn from the group's ranges. Seeded from
    the cue's name and the group's place, so a rebuild places every sound where it was. Written in this
    repository (2026-11-06) for the bridge bed's bleeps and scans; Crab Breeze's cutter has no scatter."""
    out = []
    for k, grp in enumerate(cue.get("scatter", [])):
        rng = np.random.default_rng(zlib.crc32(f"{name}/{k}".encode()))
        times, tries = [], 0
        while len(times) < grp["count"] and tries < 10000:
            t = float(rng.uniform(grp.get("from", 0.0), grp["within"]))
            if all(abs(t - u) >= grp.get("min_gap", 0.5) for u in times):
                times.append(t)
            tries += 1
        for t in sorted(times):
            spec = dict(grp["pick"][int(rng.integers(len(grp["pick"])))])
            lo, hi = grp.get("gain", [-18.0, -12.0])
            spec["gain"] = spec.get("gain", 0.0) + float(rng.uniform(lo, hi))
            spec["pan"] = float(rng.uniform(-1, 1)) * grp.get("pan", 0.8)
            plo, phi = grp.get("pitch", [1.0, 1.0])
            if "src" in spec:
                spec["pitch"] = spec.get("pitch", 1.0) * float(rng.uniform(plo, phi))
            spec["delay"] = t
            out.append(spec)
    return out


def expand_takes(name: str, cue: dict) -> list:
    """(file name, cue) pairs: the cue itself, or one per take. A take is merged over the cue's first
    layer (the usual case: another shot of the same recording), or, when the take has its own
    `layers`, over the cue (a take built differently)."""
    takes = cue.get("takes")
    if not takes:
        return [(name, cue)]
    out = []
    for k, t in enumerate(takes, 1):
        c = {key: v for key, v in cue.items() if key != "takes"}
        if "layers" in t or any(key in t for key in ("len", "drive", "lp", "hp", "peak_db")):
            c.update(t)
        else:
            c["layers"] = [dict(cue["layers"][0], **t)] + list(cue["layers"][1:])
        out.append((f"{name}_{k}", c))
    return out


def all_layers(cue: dict) -> list:
    """Every layer spec a cue can draw on (its layers and every scatter pick), for the sources lists."""
    return (list(cue["layers"]) + [p for grp in cue.get("scatter", []) for p in grp["pick"]]
            + [l for t in cue.get("takes", []) for l in t.get("layers", [])])


def cut(cue: dict, src_root: str, name: str = "") -> np.ndarray:
    """The cue as (channels, n): stereo for an ambience bed (cue "stereo": true), mono otherwise.
    Scattered sounds are mixed after the distance treatment, so they stay close and crisp over the bed."""
    ch = 2 if cue.get("stereo") else 1
    parts = [layer(s, src_root, ch) for s in cue["layers"]]
    n = cue.get("len") and int(cue["len"] * SR) or max(p.shape[-1] for p in parts)
    x = sum(fit(p, n) for p in parts)
    if "drive" in cue:
        x = g.shape(x / (np.max(np.abs(x)) or 1.0), cue["drive"])
    if "distance" in cue:
        x = distance(x, cue["distance"])
    extra = [layer(s, src_root, ch) for s in scattered(cue, name)]
    if extra:
        x = x / (np.max(np.abs(x)) or 1.0)
        x = x + sum(fit(p, n) for p in extra)
    x = per_channel(g.hp, x, cue.get("hp", 20), order=cue.get("hp_order", 2))
    if "lp" in cue:
        x = per_channel(g.lp, x, cue["lp"], order=cue.get("lp_order", 2))
    if cue.get("loop"):
        x = per_channel(g.loopable, x, cue.get("loop_fade", 0.3))
    else:
        x = fades(x, 0.0, cue.get("tail_fade", 0.05))
    return x


def credit(ref: str, licences: dict) -> str:
    pack, _, name = ref.partition(":")
    info = licences.get(pack, {})
    who = f" ({info['author']})" if info.get("author") else ""
    title = pack.split("/", 1)[1]
    lics = info.get("licences", [])
    lic = "public domain" if any("publicdomain/mark" in s for s in lics) else (
        "CC0" if "CC0" in lics or any("/zero/" in s for s in lics) else ", ".join(lics))
    title = info.get("title") or title
    attr = f" (credit: {info['attribution']})" if info.get("attribution") else ""
    return f"[{title}]({info.get('url', '')}){who} `{name}`, {lic}{attr}"


def short_sources(cue: dict, licences: dict) -> list:
    """For the Sounds page: each original a cue (or take) is made from, as 'pack (who), licence', once."""
    out = []
    for s in all_layers(cue):
        if "synth" in s:
            out.append("synthesised here (tools/audio/gen_sfx.py)")
            continue
        pack = s["src"].split(":")[0]
        info = licences.get(pack, {})
        lics = info.get("licences", [])
        lic = "public domain" if any("publicdomain" in x for x in lics) else ("CC0" if "CC0" in lics or any("/zero/" in x for x in lics) else ", ".join(lics))
        title = info.get("title") or pack.split("/", 1)[1].replace("-", " ")
        who = f" by {info['author']}" if info.get("author") else ""
        out.append(f"{title}{who} ({pack.split('/')[0]}, {lic})")
    return list(dict.fromkeys(out))


def rows_for(names, cues: dict, licences: dict) -> list:
    rows = []
    for name in names:
        if name in cues and cues[name].get("_cut"):
            srcs = []
            for s in all_layers(cues[name]):
                srcs.append(f"our `{s['synth']}` synthesis" if "synth" in s else credit(s["src"], licences))
            edits = cues[name].get("edits", "trimmed, layered, filtered, levelled")
            n = len(cues[name].get("takes", []))
            files = f"{name}_1 .. {name}_{n}" if n else name
            rows.append(f"| {files} | {'; '.join(dict.fromkeys(srcs))} | {edits} |")
        else:
            rows.append(f"| {name} | `tools/audio/gen_sfx.py` (synthesised, ours) | none |")
    return rows


HEADER = """Recorded cues are cut from CC0 and public domain originals by `tools/audio/cut_cc0.py` to the spec in
`tools/audio/cc0_cues.json` (originals fetched by `tools/audio/fetch_cc0.py`, which refuses a pack
whose page does not say CC0 or public domain, unless the owner asked for it under another licence:
`tools/audio/licence_exceptions.json`, with its credit); the rest are synthesised by `tools/audio/gen_sfx.py`.
Rebuild all of them with `tools/audio/render_sfx.sh`. This file is written by `cut_cc0.py`: edit the
spec, not this table. CC0 asks for no credit; the people who recorded the originals are named anyway. A CC-BY pack's credit is printed
beside it and must be in the game's credits.
The plan: `openspec/changes/audio-and-ambience` (design sections 4, 5 and 7).

| Cue | Made from | Edits |
| --- | --- | --- |
"""


def write_sources(cues: dict, licences: dict) -> None:
    named = {n for n in cues if not n.startswith("_")}
    amb = sorted(n for n in named if cues[n].get("group") == "Ambience")
    sfx = sorted(({n for n in g.SOUNDS if not g.is_layer(n)} | named) - set(amb))
    for title, names, folder in (("Sound effect sources", sfx, "sfx"), ("Ambience sources", amb, "ambience")):
        os.makedirs(os.path.join(REPO, "assets/audio", folder), exist_ok=True)
        with open(os.path.join(REPO, "assets/audio", folder, "SOURCES.md"), "w") as f:
            f.write(f"# {title}\n\n" + HEADER + "\n".join(rows_for(names, cues, licences)) + "\n")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--src", default=SRC_DEFAULT)
    ap.add_argument("--out", default=os.path.join(REPO, "output/sfx"))
    ap.add_argument("cues", nargs="*")
    args = ap.parse_args()
    with open(os.path.join(HERE, "cc0_cues.json")) as f:
        cues = json.load(f)
    lic_path = os.path.join(args.src, "licences.json")
    licences = json.load(open(lic_path)) if os.path.exists(lic_path) else {}
    man_path = os.path.join(args.out, "sfx.json")
    manifest = {m["name"]: m for m in json.load(open(man_path))}
    status = 0
    for name, cue in cues.items():
        if name.startswith("_") or (args.cues and name not in args.cues):
            continue
        if name not in g.SOUNDS and not ("group" in cue and "what" in cue):
            print(f"{name}: not a cue gen_sfx.py knows, and it names no group and what", file=sys.stderr)
            status = 1
            continue
        known = g.SOUNDS.get(name)
        takes = expand_takes(name, cue)
        for file_name, c in takes:
            try:
                x = cut(c, args.src, file_name)
            except (FileNotFoundError, ValueError, subprocess.CalledProcessError) as e:
                print(f"{file_name:22} left synthesised: {e}")
                status = 1
                continue
            if not np.all(np.isfinite(x)):
                print(f"{file_name}: produced a non-finite sample", file=sys.stderr)
                return 1
            cue["_cut"] = True
            peak_db = c.get("peak_db", known[3] if known else -1.0)
            g.write_wav(os.path.join(args.out, f"{file_name}.wav"), x if x.shape[0] == 2 else x[0], peak_db,
                        loop=bool(c.get("loop")))
            mono = np.abs(x).max(axis=0)
            peaks = mono[: len(mono) // 120 * 120].reshape(120, -1).max(axis=1)
            m = manifest.setdefault(file_name, {"name": file_name})
            m.update({"cue": name, "take": file_name[len(name) + 1:] if len(takes) > 1 else "",
                      "group": c.get("group", known[0] if known else "Other"),
                      "what": c.get("what", known[1] if known else ""), "seconds": round(x.shape[-1] / SR, 2),
                      "loop": bool(c.get("loop")), "peaks": [round(float(p), 3) for p in peaks / (peaks.max() or 1)],
                      "recorded": any("src" in l for l in all_layers(c)), "stereo": x.shape[0] == 2,
                      "sources": short_sources(c, licences)})
            print(f"{file_name:22} {x.shape[-1] / SR:4.1f} s  {'recorded' if m['recorded'] else 'synthesised'}"
                  f"{' stereo' if x.shape[0] == 2 else ''}")
    # cues listed but not cut this run keep a row only if cut earlier: rebuild every row from disk state
    for name, cue in cues.items():
        if not name.startswith("_") and any(m.get("cue", m["name"]) == name for m in manifest.values()):
            cue["_cut"] = True
    with open(man_path, "w") as f:
        json.dump(list(manifest.values()), f)
    write_sources(cues, licences)
    return status


if __name__ == "__main__":
    sys.exit(main())
