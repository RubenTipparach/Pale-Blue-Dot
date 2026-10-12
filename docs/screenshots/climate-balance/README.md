# Climate balance: a new world opens on the settled climate

For `openspec/changes/climate-balance`, decisions 8 and 8a (survey K6).
Captured 2026-09-28 in a cloud container on lavapipe, at 1440x900, with no
GPU. They show what the game looks like, not how smoothly it runs. Frame cost
was not measured (CLAUDE.md).

| file | what it shows |
| --- | --- |
| `new-world-l5-before.jpg` | A new world at the game's level 5, from orbit with the globe's temperature overlay (`--view orbit --overlay temperature --time 12`), with `settled-l5.ron` moved aside. The log says "no settled climate is shipped for level 5 ... spinning up from rest": 900 s of weather from a climatology by latitude, the sea still cold and the land still warm, a year away from its balance. |
| `new-world-l5-after.jpg` | The same frame from the shipped state. The log says "the settled climate for level 5, 14.95 C over the whole surface, 360 steps to the clock": the planet as it stands after two fast years and two true ones, stepped the six minutes to the capture's noon. |
| `settle-l5.log` | The run that made the state, from its day-10 checkpoint: the fast years, the two true years (15.95 °C, then 14.84 °C after the sea came down 0.95 K), and a new world's first 30 days from the written state, 14.78 to 14.99 °C. |

The overlay's ramp runs from -30 °C (blue) through white to 40 °C (red). 15 °C
is two-thirds of the way along, a light orange.
