# dusk-sky: one twilight for the sky and the ground

Fixed-step captures on the Mac (`--profile fast`), frame 120, in one world made by the
first run and reopened by the rest (the same spot each time): `--at 20 40` (map 20.000S 40.000E), on foot, eye 48.6 m above sea
level and 1.6 m above the ground, facing the sun's bearing 10 degrees up,
`Weather here: cover 0.00, rain 0.00`.

| Sun | `--time` | `--yaw` |
| ---: | ---: | ---: |
| 0.1 deg | 4.62 | -154.9 |
| -5.6 deg | 4.17 | -152.5 |
| -11.3 deg | 3.71 | -149.7 |
| -16.7 deg | 3.26 | -146.5 |
| -22.0 deg | 2.80 | -142.8 |
| -26.9 deg | 2.35 | -138.7 |
| -30.5 deg | 2.00 | -135.0 |

- `before-sweep.jpg`: an orange slab under a dark wedge at -11, the planet's
  shadow edge across the sky at -17, a lit teal corner of high air to -30.5,
  over a ground lit as night from -11.
- `after-sweep.jpg`: a blue dusk at -6 over a dimming ground, dark from -11,
  night below; no slab, no edge, no lit corner.
- `before-after-sun-11.jpg`, `before-after-sun-30.jpg`: the two the owner
  pointed at, side by side.

`pbd-app --world <new> --walk --at 20 40 --time <t> --yaw <yaw> --pitch 10 --frames 120 --capture <png>`
