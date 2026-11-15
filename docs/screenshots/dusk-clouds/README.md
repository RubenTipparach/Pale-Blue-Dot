# dusk-clouds: clouds lit by the sky's twilight, with no stars through them

The owner, 2026-11-14: "clouds seem a bit white for dusk", and stars showing
through a thick cloud. The write-up is `openspec/changes/dusk-clouds/`.

Fixed-step captures (`--capture`, `--frames 200`) on the M4 Max Mac (Apple M4
Max, Metal, GPU), 2880 x 1800, downscaled to 1200 x 750, each in a fresh
world. The before is this branch with `space-clouds` only (7200edf), so the
pair shows this change alone; the after adds `dusk-clouds`. Same arguments.

## Launch lines

From the repository root, `<w>` a fresh world:

```text
ground-dusk:  target/fast/pbd-app --world <w> --walk --at 24.117 -2.964 --time 21.3 --pitch 25 --yaw -70 --weather-at 7200 --frames 200 --capture ground-dusk.png
ground-night: ... --time 22.3 (otherwise the same)
ground-n:     target/fast/pbd-app --world <w> --walk --at 24.117 -2.964 --time 12 --pitch 35 --yaw 90 --weather-at 1800   (noon: the day kept)
dusk-limb:    target/fast/pbd-app --world <w> --view column --at 24.117 -2.964 --height 9000 --pitch -68 --yaw 90 --time 18.6 --weather-at 1800
day-limb:     ... --time 12 (otherwise the same)
```

The log puts the sun at the spawn 5.7 degrees under the horizon at 21.3 and
17.2 under at 22.3. `--weather-at 7200` gives the spawn a full cover with a
broken deck overhead at that hour.

## What they show

- **ground-dusk** (the owner's frame): before, the deck is a bright
  near-white grey, the brightest thing in the frame, and stars show through
  it. After, it is a dark grey with a warm tint where its tops still catch
  the sun's last light, darker than the twilight glow low on the sun's side,
  and the stars are gone behind it.
- **ground-night**: before, a grey deck lit by the old night floor (0.045),
  stars through it. After, the deck is dark, lit by the sky's night light
  (the moon's share) and the ground's floor; it reads as dark cloud against
  the starry sky where it breaks. Very dark: the owner judges whether night
  cloud wants more of the moon.
- **ground-n**, **day-limb**: noon from the ground and from 9 km: the same
  as before, by the design (by day the sky term is today's day blue).
- **dusk-limb**: from 9 km at 18.6, the clouds by the terminator take the
  low sun's warm colour instead of a flat white.

## Puff counts

Unchanged by this change (lighting and the composite only): see
`docs/screenshots/space-clouds/README.md`. No shader error in any log (the
one `flame.png` error predates both changes).
