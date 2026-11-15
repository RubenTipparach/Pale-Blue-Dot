#!/usr/bin/env python3
"""Pale Blue Dot's synthesised sounds: the one place each synthesised cue and layer is made.

Ported from Fleet Command (tools/audio/gen_sfx.py at 08374c4, itself after Crab Breeze's at
0e569e3): its building blocks (oscillators, noise, filters, envelopes, the small reverb, the WAV
writer) unchanged in what they compute, and none of its sounds (a starship's weapons and console
do not belong on a planet). Changed here: 48 kHz (openspec/changes/audio-and-ambience, design
section 7), and a filter with a fixed cutoff runs through scipy's lfilter with the same
coefficients the per-sample loop uses, because a 40 s stereo bed through the loops took minutes.

Deterministic (each cue seeds its own generator from its name), mono unless a bed, peak-normalised
(the cue's peak_db; its volume in the game's data sets its level).

Two kinds of entry, one registry:
- a cue (group Menu, Craft, ...): written to the page and the game;
- a layer (group "_layer"): never written alone, only mixed into a recorded cue by
  tools/audio/cut_cc0.py ({"synth": "<name>"} in tools/audio/cc0_cues.json), where a recording
  lacks something (the sub-bass under a close thunder crack).

A recorded CC0 cut in tools/audio/cc0_cues.json replaces a synthesised cue of the same name.

Usage: python3 tools/audio/gen_sfx.py [--out DIR] [--only NAME ...]
Writes DIR/<cue>.wav and DIR/sfx.json (default output/sfx); tools/audio/render_sfx.sh encodes them.
"""
import argparse
import json
import os
import sys
import wave
import zlib

import numpy as np

try:  # the fast path for fixed-cutoff filters; the loops below are the reference and the fallback
    from scipy.signal import lfilter
except ImportError:  # pragma: no cover
    lfilter = None

SR = 48000

# ---------------------------------------------------------------- building blocks

def rng_for(name: str) -> np.random.Generator:
    """A generator seeded from the cue's name, so every run makes the same sound."""
    return np.random.default_rng(zlib.crc32(name.encode()))


def t_axis(seconds: float) -> np.ndarray:
    return np.arange(int(seconds * SR)) / SR


def phase(freq_hz) -> np.ndarray:
    """Running phase (radians) for a frequency curve, so sweeps stay continuous."""
    return 2 * np.pi * np.cumsum(np.asarray(freq_hz, dtype=np.float64)) / SR


def sine(freq_hz) -> np.ndarray:
    return np.sin(phase(freq_hz))


def saw(freq_hz) -> np.ndarray:
    p = phase(freq_hz) / (2 * np.pi)
    return 2 * (p - np.floor(p + 0.5))


def square(freq_hz, duty: float = 0.5) -> np.ndarray:
    p = (phase(freq_hz) / (2 * np.pi)) % 1.0
    return np.where(p < duty, 1.0, -1.0)


def noise(rng: np.random.Generator, n: int) -> np.ndarray:
    return rng.uniform(-1, 1, n)


def sweep(a: float, b: float, n: int, curve: float = 1.0) -> np.ndarray:
    """From a to b over n samples; curve > 1 lingers near a, < 1 rushes to b. Exponential in pitch."""
    x = np.linspace(0, 1, n) ** curve
    if a > 0 and b > 0:
        return a * (b / a) ** x
    return a + (b - a) * x


def exp_decay(n: int, seconds: float) -> np.ndarray:
    """An envelope falling by 60 dB over `seconds`."""
    return np.exp(-6.9 * np.arange(n) / (seconds * SR))


def adsr(n: int, a: float, d: float, s: float, r: float) -> np.ndarray:
    a_n, d_n, r_n = int(a * SR), int(d * SR), int(r * SR)
    s_n = max(0, n - a_n - d_n - r_n)
    env = np.concatenate([
        np.linspace(0, 1, a_n, endpoint=False),
        np.linspace(1, s, d_n, endpoint=False),
        np.full(s_n, s),
        np.linspace(s, 0, r_n),
    ])
    return np.pad(env, (0, max(0, n - len(env))))[:n]


def onepole_lp(x: np.ndarray, cutoff_hz) -> np.ndarray:
    """One-pole lowpass with a fixed or per-sample cutoff."""
    if lfilter is not None and np.ndim(cutoff_hz) == 0:
        a = 1 - np.exp(-2 * np.pi * float(cutoff_hz) / SR)
        return lfilter([a], [1, a - 1], x)
    c = np.broadcast_to(np.asarray(cutoff_hz, dtype=np.float64), x.shape)
    a = 1 - np.exp(-2 * np.pi * c / SR)
    y = np.empty_like(x)
    z = 0.0
    for i in range(len(x)):
        z += a[i] * (x[i] - z)
        y[i] = z
    return y


def onepole_hp(x: np.ndarray, cutoff_hz) -> np.ndarray:
    return x - onepole_lp(x, cutoff_hz)


def bandpass(x: np.ndarray, centre_hz, q: float) -> np.ndarray:
    """A state-variable bandpass with a fixed or swept centre (Chamberlin form, stable below SR/6)."""
    f = np.broadcast_to(np.asarray(centre_hz, dtype=np.float64), x.shape)
    k = 2 * np.sin(np.pi * np.minimum(f, SR / 6.5) / SR)
    damp = 1.0 / q
    low = band = 0.0
    y = np.empty_like(x)
    for i in range(len(x)):
        low += k[i] * band
        high = x[i] - low - damp * band
        band += k[i] * high
        y[i] = band
    return y


def shape(x: np.ndarray, drive: float) -> np.ndarray:
    """Soft saturation."""
    return np.tanh(x * drive) / np.tanh(drive)


def _svf(x: np.ndarray, cutoff_hz, q: float, mode: str) -> np.ndarray:
    """One zero-delay-feedback state-variable filter stage (12 dB/octave), stable while its
    cutoff sweeps per sample (Zavalishin's topology-preserving form). With a fixed cutoff it is the
    bilinear biquad below, the same filter, run by lfilter."""
    if lfilter is not None and np.ndim(cutoff_hz) == 0:
        g = np.tan(np.pi * np.clip(float(cutoff_hz), 5.0, SR * 0.45) / SR)
        k = 1.0 / q
        d = 1 + k * g + g * g
        den = [1, 2 * (g * g - 1) / d, (1 - k * g + g * g) / d]
        num = [g * g / d, 2 * g * g / d, g * g / d] if mode == "lp" else [1 / d, -2 / d, 1 / d]
        return lfilter(num, den, x)
    c = np.broadcast_to(np.asarray(cutoff_hz, dtype=np.float64), x.shape)
    g = np.tan(np.pi * np.clip(c, 5.0, SR * 0.45) / SR)
    k = 1.0 / q
    a1 = 1.0 / (1.0 + g * (g + k))
    a2 = g * a1
    a3 = g * a2
    ic1 = ic2 = 0.0
    y = np.empty_like(x)
    low = mode == "lp"
    for i in range(len(x)):
        v3 = x[i] - ic2
        v1 = a1[i] * ic1 + a2[i] * v3
        v2 = ic2 + a2[i] * ic1 + a3[i] * v3
        ic1 = 2 * v1 - ic1
        ic2 = 2 * v2 - ic2
        y[i] = v2 if low else x[i] - k * v1 - v2
    return y


def lp(x: np.ndarray, cutoff_hz, order: int = 4, q: float = 0.707) -> np.ndarray:
    """A steep lowpass: `order` / 2 SVF stages (order 4 is 24 dB/octave). Combustion, blasts and
    rumble live under it: the onepole_lp above is too gentle and leaves them hissing."""
    for _ in range(order // 2):
        x = _svf(x, cutoff_hz, q, "lp")
    return x


def hp(x: np.ndarray, cutoff_hz, order: int = 2) -> np.ndarray:
    """A steep highpass (order 2 is 12 dB/octave); used at 20-30 Hz to keep sub energy audible."""
    for _ in range(order // 2):
        x = _svf(x, cutoff_hz, 0.707, "hp")
    return x


def brown(rng: np.random.Generator, n: int) -> np.ndarray:
    """Brown (red) noise: integrated white noise with a slight leak, the colour of a roar."""
    w = rng.uniform(-1, 1, n)
    if lfilter is not None:
        y = lfilter([0.05], [1, -0.998], w)
        return y / (np.max(np.abs(y)) or 1.0)
    y = np.empty(n)
    z = 0.0
    for i in range(n):
        z = 0.998 * z + w[i] * 0.05
        y[i] = z
    return y / (np.max(np.abs(y)) or 1.0)


def turbulence(rng: np.random.Generator, n: int, rate_hz: float = 7.0, depth: float = 0.45) -> np.ndarray:
    """A burning flame's unsteady swell: slow random amplitude between 1 - depth and 1 + depth."""
    m = lp(rng.uniform(-1, 1, n), rate_hz, order=2)
    m = m / (np.max(np.abs(m)) or 1.0)
    return 1.0 + depth * m


def crackle(rng: np.random.Generator, n: int, density: float = 0.004, cutoff_hz: float = 2500.0) -> np.ndarray:
    """Sparse pops of burning fuel, low passed so they sound like fire, not like static."""
    pops = rng.uniform(-1, 1, n) * (rng.uniform(0, 1, n) < density)
    return lp(pops, cutoff_hz, order=2)


def ring(freqs, decays_s, n: int, amps=None) -> np.ndarray:
    """Inharmonic struck-metal partials."""
    amps = amps or [1.0] * len(freqs)
    t = np.arange(n) / SR
    out = np.zeros(n)
    for f, d, a in zip(freqs, decays_s, amps):
        out += a * np.sin(2 * np.pi * f * t) * np.exp(-6.9 * t / d)
    return out


def _comb(x: np.ndarray, d: int, g: float) -> np.ndarray:
    """y[i] = x[i] + g * y[i - d], a block of d samples at a time: each block needs only the one
    before it, so this is the per-sample loop's result in len(x) / d numpy steps."""
    y = x.copy()
    for k in range(d, len(x), d):
        e = min(len(x), k + d)
        y[k:e] += g * y[k - d:e - d]
    return y


def _allpass(x: np.ndarray, d: int, g: float) -> np.ndarray:
    """y[i] = -g x[i] + x[i - d] + g y[i - d], block by block as _comb."""
    y = -g * x
    y[d:] += x[:-d]
    for k in range(d, len(x), d):
        e = min(len(x), k + d)
        y[k:e] += g * y[k - d:e - d]
    return y


def reverb(x: np.ndarray, room: float = 0.5, wet: float = 0.3, tail_s: float = 1.5) -> np.ndarray:
    """A small Schroeder reverb (four combs, two allpasses), with a tail appended. The combs and
    allpasses run a block of their delay at a time (_comb, _allpass): the same output as the
    per-sample loops Fleet Command and Crab Breeze run, in a fraction of the time."""
    x = np.concatenate([x, np.zeros(int(tail_s * SR))])
    out = np.zeros_like(x)
    for delay_ms, g in ((29.7, 0.805), (37.1, 0.827), (41.1, 0.783), (43.7, 0.764)):
        d = int(delay_ms * SR / 1000 * (0.6 + room))
        gg = g * (0.75 + 0.25 * room)
        out += _comb(x, d, gg)
    for delay_ms, g in ((5.0, 0.7), (1.7, 0.7)):
        d = int(delay_ms * SR / 1000)
        out = _allpass(out, d, g)
    return x * (1 - wet) + out * wet * 0.25


def fit(x: np.ndarray, n: int) -> np.ndarray:
    return np.pad(x, (0, max(0, n - len(x))))[:n]


def mix(*parts: np.ndarray) -> np.ndarray:
    n = max(len(p) for p in parts)
    return sum(fit(p, n) for p in parts)


def loopable(x: np.ndarray, fade_s: float = 0.15) -> np.ndarray:
    """Crossfades the end into the start so the sound loops with no click."""
    f = int(fade_s * SR)
    head, body, tail = x[:f], x[f:-f], x[-f:]
    w = np.linspace(0, 1, f)
    return np.concatenate([tail * (1 - w) + head * w, body])


def at(x: np.ndarray, seconds: float, n: int) -> np.ndarray:
    """x placed `seconds` into a buffer of n samples."""
    s = int(seconds * SR)
    out = np.zeros(n)
    if s >= n:
        return out
    end = min(n, s + len(x))
    out[s:end] = x[: end - s]
    return out


# ---------------------------------------------------------------- the sounds

SOUNDS = {}


def sound(name, group, what, peak_db=-1.0):
    """Register a cue or a layer. peak_db is the level it is written at: -1 dBFS for weapons and
    blasts, lower for console tones that should sit under the fight rather than over it."""
    def deco(fn):
        SOUNDS[name] = (group, what, fn, peak_db)
        return fn
    return deco


def is_layer(name: str) -> bool:
    """A layer is mixed into recorded cues and never written as a cue of its own."""
    return SOUNDS[name][0].startswith("_")


def chorus(f_hz, n: int, cents=(0.0, -9.0, 7.0), fn=None) -> np.ndarray:
    """Detuned copies of one oscillator, summed: the width of an energy weapon's tone."""
    fn = fn or saw
    f = np.broadcast_to(np.asarray(f_hz, dtype=np.float64), (n,))
    return sum(fn(f * 2 ** (c / 1200)) for c in cents) / len(cents)


def tone(freq_hz: float, seconds: float, a=0.004, d=0.03, s=0.6, r=0.05, fn=None) -> np.ndarray:
    """A short enveloped console tone, sine by default. The release is shortened to fit inside
    the note, so the tone always fades to zero and never ends on a click."""
    n = int(seconds * SR)
    r = max(0.005, min(r, seconds - a - d))
    return (fn or sine)(np.full(n, float(freq_hz))) * adsr(n, a, d, s, r)


# ---------------------------------------------------------------- the sounds
# Only what the design says to synthesise (openspec/changes/audio-and-ambience, design section 7):
# the menu tones, the stall warning, and layers a recording lacks. Everything else is recorded
# (tools/audio/cc0_cues.json).

def mallet(freq_hz: float, seconds: float, bright: float = 0.25, decay_s: float = 0.25) -> np.ndarray:
    """A soft wooden mallet note: a sine with a quieter octave and twelfth that die faster, a 2 ms
    attack and an exponential fall, so a menu tone sounds struck and warm rather than electronic."""
    n = int(seconds * SR)
    f = np.full(n, float(freq_hz))
    body = sine(f) + bright * sine(f * 2.0) * exp_decay(n, decay_s * 0.5) + 0.4 * bright * sine(f * 3.0) * exp_decay(n, decay_s * 0.3)
    env = exp_decay(n, decay_s) * np.minimum(1.0, np.arange(n) / (0.002 * SR))
    tail = np.linspace(1, 0, min(n, int(0.01 * SR)))
    env[-len(tail):] *= tail
    return body * env


# --- layers (mixed into recorded cues, never written alone)

@sound("sub_boom", "_layer", "The weight under a close thunder crack: a sine falling from 70 to 28 Hz over two seconds, as a recording made on a field microphone loses it.")
def sub_boom(r):
    n = int(2.6 * SR)
    body = sine(sweep(70, 28, n, 0.6)) * exp_decay(n, 2.2) * np.minimum(1.0, np.arange(n) / (0.01 * SR))
    return hp(body, 22.0, order=2)


@sound("sub_roll", "_layer", "A long low swell under a far roll of thunder: brown noise kept under 120 Hz, rising and falling over four seconds.")
def sub_roll(r):
    n = int(5.0 * SR)
    body = hp(lp(brown(r, n), 120.0, order=4), 25.0, order=2)
    env = np.sin(np.pi * np.linspace(0, 1, n)) ** 1.5
    return body / (np.max(np.abs(body)) or 1.0) * env


@sound("room_tone", "_layer", "A cave's still air under its drips: brown noise between 40 and 220 Hz, swelling a decibel either way over the loop. The drip recordings have none of it.")
def room_tone(r):
    n = int(40.0 * SR)
    body = hp(lp(brown(r, n), 220.0, order=4), 40.0, order=4)
    drift = 1.0 + 0.1 * np.interp(np.arange(n), np.linspace(0, n, 7), r.uniform(-1, 1, 7))
    return body / (np.max(np.abs(body)) or 1.0) * drift


# --- menus and HUD (not placed; the menus bus)

@sound("ui_hover", "Menu", "The pointer over a button: one soft, short mallet tick, high and quiet.", peak_db=-14.0)
def ui_hover(r):
    return mallet(1568.0, 0.09, bright=0.1, decay_s=0.05)


@sound("ui_click", "Menu", "A button pressed: a soft wooden mallet note with a quick upper grace note.", peak_db=-9.0)
def ui_click(r):
    n = int(0.22 * SR)
    return mix(at(mallet(784.0, 0.2, decay_s=0.12), 0.0, n), 0.5 * at(mallet(1175.0, 0.16, decay_s=0.08), 0.025, n))


@sound("ui_back", "Menu", "Back or close: the click's two notes falling, a fourth down.", peak_db=-9.0)
def ui_back(r):
    n = int(0.26 * SR)
    return mix(0.6 * at(mallet(880.0, 0.16, decay_s=0.08), 0.0, n), at(mallet(659.0, 0.22, decay_s=0.12), 0.04, n))


@sound("ui_toggle", "Menu", "A setting switched: two quick knocks, the second higher.", peak_db=-9.0)
def ui_toggle(r):
    n = int(0.2 * SR)
    return mix(at(mallet(523.0, 0.1, decay_s=0.05), 0.0, n), at(mallet(784.0, 0.12, decay_s=0.06), 0.06, n))


@sound("ui_slot", "Menu", "The hotbar moved one slot: a dry, very short wooden tick.", peak_db=-12.0)
def ui_slot(r):
    n = int(0.07 * SR)
    knock = mallet(1046.0, 0.07, bright=0.4, decay_s=0.03)
    click = onepole_hp(noise(r, n), 2500.0) * exp_decay(n, 0.006)
    return mix(knock, 0.3 * click)


@sound("pickup", "Menu", "A drop picked up: a soft rising pair of mallet notes, small and quick.", peak_db=-10.0)
def pickup(r):
    n = int(0.25 * SR)
    return mix(at(mallet(659.0, 0.14, decay_s=0.07), 0.0, n), at(mallet(988.0, 0.2, decay_s=0.12), 0.05, n))


@sound("catch_chime", "Menu", "A fish landed: three rising mallet notes with a little ring, a small reward.", peak_db=-8.0)
def catch_chime(r):
    n = int(0.9 * SR)
    notes = [(523.0, 0.0), (659.0, 0.09), (784.0, 0.18), (1046.0, 0.27)]
    return mix(*[at(mallet(f, 0.6, bright=0.3, decay_s=0.45), t, n) * (0.8 if i < 3 else 1.0) for i, (f, t) in enumerate(notes)])


# --- craft

@sound("stall_warn", "Craft", "The Kestrel's stall warning in the cockpit: a soft two-tone chime pulsing about three times a second (loops).", peak_db=-10.0)
def stall_warn(r):
    period = 0.32
    n = int(period * 4 * SR)
    beep = lambda f: tone(f, 0.13, a=0.006, d=0.02, s=0.7, r=0.05, fn=lambda ff: sine(ff) + 0.12 * sine(ff * 2))
    parts = [at(beep(880.0 if k % 2 == 0 else 740.0), k * period, n) for k in range(4)]
    return mix(*parts)




# ---------------------------------------------------------------- output
def write_wav(path: str, x: np.ndarray, peak_db: float = -1.0, loop: bool = False) -> None:
    """Peak-normalised 16-bit WAV, mono, or stereo when x is (2, n). A one-shot gets a 5 ms fade
    out; a loop must not, or the fade clicks at the seam on every repeat."""
    peak = float(np.max(np.abs(x))) or 1.0
    y = np.clip(x / peak * 10 ** (peak_db / 20), -1, 1)
    if not loop:
        fade = min(y.shape[-1], int(0.005 * SR))
        y[..., -fade:] *= np.linspace(1, 0, fade)
    channels = 1 if y.ndim == 1 else y.shape[0]
    if channels == 2:
        y = y.T.reshape(-1)   # interleave left and right
    with wave.open(path, "wb") as w:
        w.setnchannels(channels)
        w.setsampwidth(2)
        w.setframerate(SR)
        w.writeframes((y * 32767).astype("<i2").tobytes())


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--out", default="output/sfx")
    ap.add_argument("--only", nargs="*")
    args = ap.parse_args()
    os.makedirs(args.out, exist_ok=True)
    manifest = []
    path = os.path.join(args.out, "sfx.json")
    if args.only and os.path.exists(path):
        # a partial rebuild keeps every other cue's entry, or the Sounds page would list only the rebuilt ones
        # (changed here, 2026-11-14: Fleet Command's wrote the partial list over the whole one)
        manifest = [m for m in json.load(open(path)) if m["name"] not in args.only
                    and m.get("cue", m["name"]) not in args.only]
    for name, (group, what, fn, peak_db) in SOUNDS.items():
        if is_layer(name) or (args.only and name not in args.only):
            continue
        x = fn(rng_for(name))
        if not np.all(np.isfinite(x)):
            print(f"{name}: produced a non-finite sample", file=sys.stderr)
            return 1
        write_wav(os.path.join(args.out, f"{name}.wav"), x, peak_db, loop="(loops)" in what)
        # a 120-point peak outline for the page's waveform strip
        peaks = np.abs(x).reshape(-1)[: len(x) // 120 * 120].reshape(120, -1).max(axis=1)
        peaks = peaks / (peaks.max() or 1.0)
        manifest.append({"name": name, "group": group, "what": what, "seconds": round(len(x) / SR, 2),
                         "loop": "(loops)" in what, "peaks": [round(float(p), 3) for p in peaks]})
        print(f"{group:8} {name:18} {len(x) / SR:4.1f} s")
    with open(os.path.join(args.out, "sfx.json"), "w") as f:
        json.dump(manifest, f)
    return 0


if __name__ == "__main__":
    sys.exit(main())
