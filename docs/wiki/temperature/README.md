# Surface temperature maps

Drawn by `tools/temperature_map.py` from the fields that
`crates/pbd-core/examples/fish_ranges.rs` writes. Each map is one period's
mean surface temperature: the sea surface over the sea, the ground at sea
level on land. The instrument and the numbers are in
`openspec/changes/climate-balance/design.md`, "Measured".

## Today's build

The shipped `assets/config/atmosphere.ron` at the shipped level 5.

| map | period | whole surface | range |
| --- | --- | ---: | ---: |
| `day1.png` | day 1 | about +13 °C | |
| `year1.png` | days 1 to 100 | −10.5 °C | |
| `year2.png` | days 101 to 200 | −24.5 °C | −48 to −16 °C |

## Preview: both heat leaks closed (2026-09-27)

These are **not** the fix. The fix is `climate-balance`, and it is not built.
They preview it with the shipped code and the instrument's `ATMOSPHERE`
override, at level 4 (2,562 cells), with no thermostat:

```text
ATMOSPHERE='(level: 4, heat_spread: 0.0, evaporation_cooling: 15750.0,
             solar_wm2: 1360.0, cloud_albedo: 0.35, cloud_greenhouse: 40.0)'
```

- `heat_spread` 0 stops the spread between cells. Today it moves temperature,
  not heat, and drains the sea at every coast.
- `evaporation_cooling` 15,750 J/kg charges the ground what its air column
  gives back when the water condenses. Today the ground is charged 80,000.

| map | period | whole surface | sea | range |
| --- | --- | ---: | ---: | ---: |
| `preview-leaks-closed-year1.png` | days 1 to 100 | 11.7 °C | 15.5 °C | −21.9 to 29.9 °C |
| `preview-leaks-closed-year2.png` | days 101 to 200 | 12.1 °C | 15.8 °C | −21.0 to 29.0 °C |

`before-after-year2.png` puts today's year 2 above the preview's. The
thermostat, once built, lifts the whole surface the remaining 3 °C to the
owner's 15 °C.
