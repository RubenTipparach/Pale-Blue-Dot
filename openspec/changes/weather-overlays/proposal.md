# Proposal: overlays that show the weather working

## Why

The owner: *"design some visual overlays so I can view this happening, along
with some system to display cloud cover, precipitation, humidity and solar
radiation exposure."* Their references are:
- earth.nullschool's wind map: animated streamlines coloured by speed, spirals
  where the storms are;
- a Gulf Stream current map;
- a world map of surface solar radiation.

`atmospheric-circulation` makes a simulation that nobody can see except through
the clouds it drives. Without overlays there is no way to tell a jet from a
cloud street, or to check a gyre.

## What

One overlay at a time, drawn over the planet from the ground or from orbit.
**M** cycles through them, and the pause menu lists them:

| Overlay | Shows | Units |
| --- | --- | --- |
| Wind | surface wind: animated streamlines, coloured by speed | m/s |
| Jet | the cloud-level wind, the same way | m/s |
| Currents | the ocean's surface current, the same way | m/s |
| Cloud | cloud cover | % |
| Rain | precipitation, with snow in its own colour | mm/h |
| Humidity | relative humidity at the surface | % |
| Sunlight | sunlight reaching the ground after clouds | W/m^2 |
| Temperature | surface temperature | deg C |

- **Streamlines** follow the field and scroll along it at a rate that grows with
  its speed, as in the wind map. Storms read as spirals and the jet as a
  ribbon. They are drawn over the scalar's colour, so Wind is speed and
  direction together.
- **A legend**: the overlay's name, a colour bar with its units and range, the
  day and season, and the key.
- **Nothing in the overlay is its own simulation.** It reads the same weather
  data the clouds read, sampled the same way. What it shows is what the clouds
  are doing.

## Out of scope

- Overlays of anything but the weather and the ocean.
- A 2D map projection. The overlay is on the globe, as in the wind map.
