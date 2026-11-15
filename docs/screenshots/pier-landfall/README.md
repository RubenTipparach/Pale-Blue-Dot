# pier-landfall: a pier's planks lie on the beach, never in it

The owner (2026-11-14): "this board thing seems to be clipping in the
terrain". The write-up is `openspec/changes/pier-landfall/`.

Each pair is the same shot before (top, branch base `c54f1df`) and after
(bottom). In-game fixed-step captures (`--capture --frames 200`) on the M4
Max Mac (Metal), the fast build, a new world for every shot, on foot. They
show what things look like, not how smoothly they run. `--time` is the clock
at longitude 0: Holinghaven (115 W) is night at 11 and day at 22,
Marenstrand (172 E) is day at 0.

| File | Where | Shows |
| --- | --- | --- |
| `pair-holinghaven-pier-night-rain.jpg` | Holinghaven main pier, `--walk --at 23.6299 -115.1628 --yaw 113.1 --pitch -15 --time 11 --rain 0.8` | The owner's view: night, rain, looking along the pier to the quay. Before, the planks over the two beach rows are in the sand's plane and the wet sand is drawn over them. After, they lie on the sand, 0.12 m proud, and step down onto their piles where the beach drops. |
| `pair-holinghaven-pier-day.jpg` | the same, `--time 22 --rain 0 --weather-at 7200` | The same by day. Before, the planks and the sand fight for the same pixels (here the planks mostly win; at night the wet sand did). |
| `pair-holinghaven-side-night.jpg` | the beach beside a pier, `--at 23.5952 -115.0475 --yaw 23.1 --pitch -10 --time 11` | From the side: the pier crossing the frame. Before, its right half is flush with the sand; after, it lies on it. The green speckle on the water behind is the separate finding below. |
| `pair-marenstrand-bank-day.jpg` | Marenstrand main pier, `--at 36.8932 171.7947 --yaw 96.3 --pitch -12 --time 0` | The one bank in six harbours where the planet's ground stands a metre over the deck. Before, the sand bank is drawn through the planks. After, the 0.75 m of planks over it lie on it, a metre over the boardwalk either side: no clipping, but a raised box on the pier. The owner's call (design, "What it costs"). |
| `speckle-sea-reflection-on-off.jpg` | crop of the side shot, the shallows round the boats | Top: as built. Bottom: `PBD_REFLECT=sea`, the sea's screen reflection off. The speckle is unchanged: not the reflection. |
| `speckle-refraction-on-off.jpg` | the same crop | Top: as built. Bottom: `refract_amount: 0.0` (a temporary `water.ron` edit, not committed). Most of the per-pixel noise goes: the refraction is the main cause. Not fixed here (design, "Not this change"). |

Nothing stored changes: the pier is a template piece, cut from the stored
chart; the town record, its ground and every player edit are as they were.
