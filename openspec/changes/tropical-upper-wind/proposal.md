# Proposal

## Why

The owner, 2026-09-29, on the map's Jet overlay: "why is jet stream truncated
in the equator?" The overlay shows a dark band about 12 degrees wide along the
equator. It is flat and has hard edges, and the fastest wind on the planet
runs along its northern edge.

The band is built into the wind at cloud height (design, "Measured"). That
wind is the surface wind plus the thermal wind. The thermal wind divides by
the Coriolis parameter, which is nought at the equator. So the model turns it
off within 6 degrees of the equator and lets it back in by 20 degrees. What
is left inside the band is the doldrums' surface wind, half a metre a second,
which the overlay's 0-45 m/s ramp draws as black. The edge is hard because
the 45 m/s cap is applied after the fade-in. Just outside the band the
uncapped wind is 90-110 m/s, so it hits the cap after a third of the fade.

The band is not only on the map. Cloud rides mostly on this wind, so
tropical cloud barely drifts, while cloud 12 degrees north is carried at
full jet speed. On Earth the tropics blow easterly aloft at 5-10 m/s, and the
jets sit at the edge of the tropical circulation, not against the equator.

## What Changes

- The wind at cloud height blows everywhere. Inside the tropics it is a
  gentle easterly, against the planet's turn. It changes into the jets over
  a band of latitude, not at an edge. The candidates are measured and drawn
  in the design, and the owner picks one (survey W1).
- The jet's fade-in comes after its cap, so the fade shows at its full width.
- One new setting, the tropical easterly's speed in m/s, in `atmosphere.ron`.
- The settled climates that new worlds start from are made again, because
  cloud drifts differently in the tropics.
- Saved worlds are not reset. The wind at cloud height is worked out afresh
  every step and is not saved, so an old world's weather carries on under the
  new wind from its next step.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `world/weather`: an added requirement. The wind at cloud height has no calm
  band at the equator and no hard edge at the jet.

## Impact

- `pbd_core::atmosphere` step's `aloft` stage, and `AtmosphereSettings`
  (one knob, validated, with its unit).
- `assets/config/atmosphere.ron`.
- `assets/climate/settled-g*-l*.{bin,ron}`, remade.
- The Jet overlay on the globe and the map, and the map mockup's weather
  frames. Nothing reads the cloud-level wind except cloud drift and the
  overlay.
