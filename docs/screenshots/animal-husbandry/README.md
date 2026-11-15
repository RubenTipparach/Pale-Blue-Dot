# animal-husbandry: the first playable slice

Headless `--capture` shots of design decision 12 (`openspec/changes/animal-husbandry`),
taken 2026-11-14 on the M4 Max Mac (Metal, a real GPU), `target/fast/pbd-app`, window
960x600 (saved at 1920x1200, reduced to 1280 wide). Frame cost was not measured.

Every shot stands at `--at -37.5 -84` (the body frame's lat and lon), a grass meadow by
a sea inlet, where local noon is about `--time 16` and local 22:00 is `--time 3.8`.
Each run is its own `--world` name. `--husbandry <scene>` is a capture instrument
(`pbd_app::husbandry::script`): it sets a scene in front of the walker through the
player's own paths (`feed_wild`, `settle`, `place_piece`, `use_on`), so the records,
the with-lines and the messages are the ones a player's hands make. Its liberties: it
places the wild animals it shows (a capture cannot wait for a derived herd to walk into
frame), it gives the hands the feed and pieces the scene needs where `--lamps` took the
kit, it sets the ring of fence from outside the ring without the reach check, and
`collect` re-bases every kept animal three world days back ("as if kept three days")
before collecting from a hen.

| File | Shows | Launch |
| --- | --- | --- |
| `wild-day.jpg` | A wild flock of yard hens (one cockerel) and woolbacks by day; one woolback top right is a herd derived from the world seed (S10). | `pbd-app --world ah-motion2 --husbandry wild --at -37.5 -84 --time 16 --pitch -12 --frames 200 --capture wild-day.png` |
| `tame-day.jpg` | Taming: five feeds of turf (trust 24, 49, 73, 97%, then full) tamed a wild woolback; it is the player's (a `with ... rec @p animal` line) and follows (HUD line). Turf 64 to 59. | `pbd-app --world ah-tame-3 --husbandry tame --at -37.5 -84 --time 16 --pitch -20 --frames 100 --capture tame.png` |
| `pen-day.jpg` | A pen: 17 fences and a gate (the pale piece in front, braced) on the hex edges round seven cells, three hens, two woolbacks and a plodder tamed and let go inside. The log: `pen Some(930103307) (7 cells, herd 6): grazing: true, space: true`. | `pbd-app --world ah-pen-4 --husbandry pen --at -37.5 -84 --time 16 --pitch -16 --frames 300 --capture pen.png` |
| `collect-day.jpg` | A product collected: E on a hen kept three days gives `Yard hen: collected 2 eggs` (her yield gene is 0.75 of the species' egg a day). | `pbd-app --world ah-collect-3 --husbandry collect --at -37.5 -84 --time 16 --pitch -16 --frames 200 --capture collect.png` |
| `collect-pack.jpg` | The same, the pack open: the two eggs, the kit's fences and gate, the forge, coal, seeds and turnips from `farming-and-smelting`. | as above with `--menu pack` |
| `pen-night.jpg` | The pen at 22:00, lit by its own lamps (`--lamps 3 -6` places a row of every light, which falls inside the ring): the woolbacks lie down, the hens roost low. | `pbd-app --world ah-night-2 --husbandry pen --at -37.5 -84 --time 3.8 --pitch -16 --lamps 3 -6 --frames 300 --capture pen-night.png` |
| `wild-night.jpg` | The wild flock at night beside a row of lamps: hens roosting, woolbacks lying. | `pbd-app --world ah-wildnight --husbandry wild --at -37.5 -84 --time 3.8 --pitch -14 --lamps 5 -4 --frames 300 --capture wild-night.png` |
| `motion-hens-sheet.jpg` | Motion (the owner on M7: "these chickens arent animated at all"): every 5th frame of 80 (`PBD_CAPTURE_RUN`). A russet hen walks across, legs swinging with her steps; the white hens glance about and peck. | `PBD_CAPTURE_RUN=<dir>,100,80` added to the `wild-day` launch |
| `motion-woolbacks-sheet.jpg` | The same run, every 10th frame: woolbacks graze head down, turn, walk off. | as above |
| `motion-stride-peck.jpg` | Six frames, 3 apart, of the walking hen (mid-stride: the legs alternate) and the hen in front (mid-peck and glance), 2x. | as above |

How it is animated (S11): each animal is one mesh rebuilt per frame from boxes hung on
pivots. Legs swing from the hips on a phase advanced by the distance walked over the
species' stride (`stride_m`), so a foot never slides; a walking hen bobs her head with
her steps; a standing hen pecks (a 0.28 s dip at random 0.4-2 s), glances (a new head
turn every 0.8-2.5 s) and flicks her tail (every 1-3 s), and potters 0.4-1.5 m every
1.5-4 s; a beast grazes head down while it stands, chewing, lifts its head to look
about, swishes its tail and shifts its weight; at night (daylight under 0.12) all lie
down or roost over 1.7 s. Each fades in and out over 1 s through the terrain's
screen-door mask (`Faded`), never popping.

Logs were checked: no `panicked`, no error but the pre-existing missing
`textures/settlement/flame.png`. A capture does not warm, so every pipeline logs
`LATE_PIPELINE` there by design; a normal launch at the same place (`--husbandry pen`,
and again with no scene, 60-90 s each) reached `READY` with 96 pipelines and logged no
`LATE_PIPELINE` after it: the animals' field-lit, vertex-coloured mesh is prewarmed by
a hidden template from `Startup`.
