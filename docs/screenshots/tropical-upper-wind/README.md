# tropical-upper-wind: the jet's calm band at the equator

The day's mean wind at cloud height, the value the Jet overlay draws, on the
overlay's own ramp (0-45 m/s, `pbd_app::overlay::RAMPS`). It was drawn over
version 5's settled climate at level 5, through one day. Dotted lines mark
the equator and 15 and 30 degrees either side.

The shots come from a scratch instrument, which is not committed. It reads
the model's surface wind and air temperature, and works out the
cloud-level wind under each candidate rule. `openspec/changes/tropical-upper-wind/design.md`
describes it and has the numbers.

| file | what it shows |
| --- | --- |
| `options.png` | the four side by side |
| `jet-built.png` | as built: calm within 6 degrees of the equator, and the jet at its 45 m/s cap from 12 degrees north |
| `jet-a.png` | A: the cap before the fade; the edge spreads out, and the band stays calm |
| `jet-b.png` | B: A, with an 8 m/s easterly inside the band |
| `jet-c.png` | C (recommended): B, with the jet fading in across the tropics and full by 30 degrees |

These are not captures from the game. The game's before-and-after shots of
the globe and the map (task 4.2) follow once a candidate is chosen and built.
