# Design: lens-weather

## 1. The drop scale

`rain_lens_scale` joins the lens knobs in `weather.ron` / `WeatherSettings`
(validated > 0; the `.ron` stays equal to the code defaults). In `lens()`,
`drop_uv` is multiplied by it after the aspect correction, and the refraction
offset is divided by it. The four Tenebris knobs keep their meaning; at the
default 2 the coverage is unchanged (four times the drops at half the size),
and drops slide half as fast on screen.

## 2. The mist

**Signal.** A pass `lens_mist_pass` draws one pixel: `cloud_density` at the
eye with the march's own arguments and a footprint of a quarter cell. The
mist amount lives in a 1x1 R32Float pair on `WaterViewGpu`, read one frame
and written the other, like the clouds' history:

```text
target = smoothstep(0.01, 0.12, density at the eye)   (0 outside cloud)
rate   = (1 + v * lens_mist_per_mps) / (target > F ? lens_mist_fog_s : lens_mist_clear_s)
F      = target + (F - target) * exp(-dt * rate)
```

Any cloud mists a lens, so the target saturates at a thin cloud's density:
taken as the density itself, the first build barely misted in the middle of a
storm deck (the density at the eye was 0.1-0.4; 7% of the frame changed, by
at most 16 of 255). F is zero under water, with the clouds pass off, or when the history resets
(a jump). The CPU sends `dt` and the airspeed `v` (the eye's step over `dt`).

**When the lens runs.** The lens pass already runs only when needed. The CPU
opens a window when the eye is within the cloud layer's shell and the map's
cover near it is over 0.01 (a gate that cannot be false where the density is
above zero), holding it for five clearing times after, so the mist can clear
on screen; it logs the window opening and closing.

**Look**, in `lens()` before the drops:

- a mask: F against a per-pixel threshold of patch noise plus the distance
  from the screen centre, so the mist clears from the edges inward, in patches;
- beads: a static-drop field at `lens_mist_beads` per screen height (about
  4 px), each refracting one tap by about a pixel;
- a film: the existing blur at `lens_mist_blur` of the screen height, lifted
  toward its own luminance by `lens_mist_whiten` (never toward a fixed white:
  a cloud at night mists dark);
- `mix(color, misted, mask * lens_mist_strength * above_water * (1 - wipe))`,
  the wipe being the drops and their trails times the rain: drops cut clear
  runs through the mist.

**Knobs** (`weather.ron`):

| Knob | Default | Unit |
| --- | ---: | --- |
| `lens_mist_strength` | 0.8 | 0..1 |
| `lens_mist_fog_s` | 2.0 | s (0 instant) |
| `lens_mist_clear_s` | 6.0 | s (0 instant) |
| `lens_mist_per_mps` | 0.02 | per m/s |
| `lens_mist_beads` | 220 | per screen height |
| `lens_mist_blur` | 0.008 | screen heights |
| `lens_mist_whiten` | 0.35 | 0..1 |

At 150 m/s both times run four times faster: the lens fogs in about half a
second in a cloud and clears in about a second and a half after.

## Verification

- Drops: stills at scale 1, 2 (`--tool shovel --pitch 14 --time 11 --rain
  0.25 --frames 150`), against `output/captures/ghost2/turn-after.png`.
- Mist: a cloud-hop and a scenic-in-rain recording: fogs within about half a
  second of entering a cloud, clears edges first after, clear between puffs,
  none on foot, drops wipe tracks through it.
- Tests: the `.ron` against the defaults, validation, the window; the GPU
  tests; the perf suite on `clouds`, `storm`, `walk`.
