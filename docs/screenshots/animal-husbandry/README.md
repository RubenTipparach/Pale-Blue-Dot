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

## The herd in its coats (design 16, `land-transport` design 12)

`--husbandry close` is a capture instrument: one wild animal of each species held still
where it is set (it neither flees nor wanders), broadside a few metres ahead. Fixed-step
`--capture` in a Claude Code cloud container on lavapipe (no GPU, no frame times), 1440 x
900, the `fast` build, a new memory-only world, `--rain 0`. BEFORE is the build before
design 16 (the box rig), AFTER this branch; one image each, labelled in the frame. A
capture does not warm, so `LATE_PIPELINE` lines are in both logs by design.

| Shot | Command | What it shows |
| --- | --- | --- |
| `herd-before-after-16h00.jpg` | `--husbandry close --at -37.5 -84 --time 16 --pitch -14 --frames 200` | The meadow hen (left), the woolback, the plodder grazing and the horse: before, boxes of flat colour with their pattern and eyes as boxes stuck on; after, one smooth lofted form each in the `coat` strip tinted by its genome, heads down to graze. |
| `hen-before-after-16h00.jpg` | the same, the hen cropped and enlarged 3x | The hen: an egg-shaped body in feathers, wings folded along her sides, the comb and wattle grown from her head, the beak tapering out of the face, thin legs with three toes. |
| `herd-before-after-03h48.jpg` | the same, `--time 3.8` | The herd at night, lying down and roosting, lit as the ground is. |

## The Art style tab (M26 `#art`, design 17; mockup, not the game)

Chromium on SwiftShader in a cloud container (no GPU): the look, never the speed. Each is the page's `#art-<style>-<pose>-<day|night>`.

| File | Shows |
| --- | --- |
| `art-close-plodder-neck-graze-today-vs-fixed.jpg` | The plodder grazing, TODAY (face turned back under its chest, muzzle 0.36 m above the grass) beside NECK FIXED (muzzle at the grass, face never past vertical). |
| `art-close-plodder-neck-stand-today-vs-fixed.jpg` | The same standing: the fixed head carried with its poll level with the withers. |
| `art-close-plodder-soft-vs-carved-graze.jpg`, `-stand.jpg` | The plodder close up in Soft low-poly beside Carved blocky. |
| `art-<today,neck,soft,carved>-<stand,graze>-<11h00,22h30>.jpg` | The four species, the 1.8 m player and the 1 m grid in each style and pose, by day and at 22:30 by the two lanterns. |
| `art-today-graze-phone-11h00.jpg`, `art-soft-graze-phone-11h00.jpg`, `art-soft-stand-phone-22h30.jpg` | Phone width (390 px). |
| `art-info-modal.jpg` | The tab's info modal. |

## The herd in M26's blocky style (design 17, "The owner's pick")

The owner picked M26's Farmstead and Hatchery animals ("make animals like this! I love this
look", then "please go back to blocky style for animals", "DO NOT DO THE SOFT CURVY
ANIMALS"). Fixed-step `--capture` in a Claude Code cloud container on lavapipe under
`xvfb-run` (no GPU: the look, never the speed; frame cost not measured), 1440 x 900, the
`fast` build, a new memory-only world, `--rain 0`. BEFORE is `claude/phase-2-1` before this
change (design 16's lofted forms), AFTER this change; each image is one file, labelled in the
frame. `--husbandry close-stand` and `close-graze` are capture instruments added with it: the
`close` scene with every head held up or down to graze and looking ahead. M26's own panels are
Chromium on SwiftShader (`docs/mockups/husbandry.html#hatch`, `#farm`).

| File | Command | Shows |
| --- | --- | --- |
| `herd-blocky-stand-before-after-16h00.jpg` | `--husbandry close-stand --at -37.5 -84 --time 16 --pitch -14 --frames 200` | BEFORE, AFTER and M26's Hatchery tab: the hen (left), the woolback, the plodder and the horse standing; each one creature of boxes in its pixel hide, the head carried at the withers. |
| `herd-blocky-graze-before-after-16h00.jpg` | the same with `close-graze` | Grazing: before, the face turned back under the chest; after, the neck lowered and stretched, the muzzle at the grass, the face 75 degrees down. |
| `herd-blocky-night-before-after-03h48.jpg` | `close-stand`, `--time 3.8 --lamps 3 -6` | At 22:00 by the lamps (the held torch in front): lying and roosting, lit as the ground is. |
| `herd-blocky-cart-plodder-before-after-16h00.jpg` | `--walk --carts --cart-close plodder --time 16 --pitch -14 --frames 200` | A plodder in the horse cart's shafts. |
| `herd-blocky-cart-horse-before-after-16h00.jpg` | `--walk --carts --cart-close horse ...` | The horse in the shafts and a loose horse. |
| `herd-blocky-ride-before-after-16h00.jpg` | `--walk --carts --cart-close ride --pitch -38 ...` | Riding a plodder: its back, the blanket, the neck and horns ahead. |
