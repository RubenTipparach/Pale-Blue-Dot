# Flora v2: captures

For `openspec/changes/flora-v2`. Captured 2026-11-15 on the M4 Max Mac (its
GPU, Metal), fixed-step `--capture` runs of the fast build, each on a new
world deleted afterwards, at 2880x1800, saved as JPEG (before: full size;
after: 1920 wide). They show what things look like, not how smoothly they
run; frame cost was not measured.

Common flags: `target/fast/pbd-app --walk --weather-at 1800 --time 10
--world <new> --spawn <biome>`, then:

| file | flags | what it shows |
| --- | --- | --- |
| `before-fields.jpg`, `before-jungle.jpg`, `before-swamp.jpg` | `--yaw 270 --pitch -12` | The build this branch started from (`claude/phase-1`, 6632239). |
| `after-fields-blades.jpg` | `--yaw 270 --pitch -12` | Pixel grass off (the default): the blades as before, now with flower colours, mushrooms and ferns. |
| `after-fields-tufts.jpg` | `... --grass tufts` | Pixel grass on: the Play demo's tufts in the fields' ramp (design 1). |
| `after-jungle-tufts.jpg`, `after-swamp-tufts.jpg`, `after-beach-tufts.jpg`, `after-tundra-tufts.jpg` | `--spawn <biome> --grass tufts` | The tufts in each biome's ramp; ferns under the canopy; no grass on the tundra's snow. |
| `after-tree-near.jpg` | `--yaw 300 --pitch 20 --grass tufts` | The hex-prism trees as today: the leaf cards were taken off at the owner's word (below). |
| `after-fields-day27.jpg`, `-day52.jpg`, `-day77.jpg` | `--pitch 5 --grass tufts --day N` | The fields' calendar (design 5): autumn colour on the prism leaves, winter (the leaves dulled toward twigs, straw grass), spring green. |
| `after-menu.jpg` | `--menu settings --settings-tab picture --grass tufts` | The Pixel grass check on the GRAPHICS (PICTURE) page. |

Checked in the logs: no `ERROR` in any capture. A normal (warming) launch,
`PBD_SETTINGS=<scratch> target/fast/pbd-app --world flora-warm-a1 --walk
--spawn fields --grass tufts`, logged `READY playable 5.75 s after launch
(97 pipelines)` and no `LATE_PIPELINE` in the 20 s after it: the change adds
no pipeline.

**Leaf cards removed (owner, 2026-11-15):** "Hmm trees have weird leave
structures on them I don't really like". Every after shot was retaken without
them on the same day; a second warming launch logged `READY playable 5.76 s
(97 pipelines)`, no `LATE_PIPELINE`, no `ERROR`.

For the owner to judge: whether Pixel grass should become the default; the
strength of the autumn orange; the winter crown (the prism's leaves stay, so
winter is a dull olive rather than bare branches).

## M13 approved into the engine (group 6, design 9)

The owner approved M13 on 2026-11-15. Captured the same day on the M4 Max Mac
(its GPU, Metal), fixed-step `--capture` runs of the fast build, each on a new
world deleted afterwards, at 2880x1800, saved as 1920-wide JPEG (crops at
full resolution). `before` is `claude/env-v2` at 430a8aa (the binary this
branch started from); `after` is this branch. Every shot: `target/fast/pbd-app
--walk --weather-at 1800 --time 10 --grass tufts --world <new>`, then:

| file | flags | what it shows (honest read) |
| --- | --- | --- |
| `m13-*-fields.jpg`, `m13-*-fields-lip.jpg` (crop) | `--spawn fields --yaw 270 --pitch -12` | The painted daisies are gone from the grass top and the sod lip (9.5); the tufts take the pasture tile's own ramp (9.1), so they sit in the ground's greens with lighter tips instead of standing out bluer; the clutter reaches about twice as far (9.3). The tufts are now close to the ground's colour and read mostly by their light tips: that is what "match the ground" gives, and the owner's to judge. |
| `m13-*-tree-near.jpg`, `-tree-near-crop.jpg` | `--spawn fields --yaw 300 --pitch 20` | The crowns and trunks pulled toward their tile's mean (9.2): the leaf tile's salt-and-pepper is softer, close to the mockup's crowns. Plums and cherries hang under the lower leaf layer of about half the trees (9.6). |
| `m13-*-autumn.jpg`, `-autumn-ground.jpg`, `-autumn-fruit.jpg` | `--spawn fields --pitch 5 --day 27` | Autumn: fallen leaves in the cap's own texels, rust, ochre, tan and brown, thicker in the cell under each tree (9.4); fruit hanging, a few fallen. The fall is the mockup's default 0.45, a scatter rather than a carpet; in tree shade the leaves read darker than in the mockup's sunlit shots. |
| `m13-*-winter.jpg` | `--spawn fields --pitch 5 --day 52` | Midwinter: the fall thinned to 0.6 and browned; fruit almost all gone (one plum kept), straw tufts. No snow: the game lays no seasonal snow, so the mockup's "leaves on snow" and the sod-lip snow fix have nothing to act on here (9.9). |
| `m13-*-tundra.jpg` | `--spawn tundra --yaw 270 --pitch -5` | Pine cones hang under the pines' lower skirt (9.6). The cones are dark red-brown and small; they read as cones close up. |
| `m13-*-swamp.jpg` | `--spawn swamp --yaw 270 --pitch -12` | This spawn lands among tall jungle kapoks (broad leaves, mangoes): mangoes and papayas hang in the canopy, softer leaf contrast. Not a swamp shot, despite the flag. |
| `m13-*-jungle.jpg` | `--spawn jungle --yaw 270 --pitch -12` | This spawn lands on open grass by a desert: the tufts in the ground's ramp at twice the reach. |
| `m13-*-beach.jpg` | `--spawn beach --yaw 270 --pitch -12` | Sand: unchanged, as it should be (no deciduous trees, no flowers painted on sand). **Not shown:** the dry olive beach crowns (9.7) are built, but beach trees need grassy cells in the beach band and none was in view from this spawn or three other headings. |

Checked in the logs: no `ERROR` in any capture (the `LATE_PIPELINE` lines in
capture runs are Bevy mesh pipelines that captures do not warm, the same 22-26
before and after). A normal (warming) launch, `PBD_SETTINGS=<scratch>
target/fast/pbd-app --world flora-warm-m13 --walk --spawn fields --grass
tufts`, logged `READY playable 5.58 s after launch (97 pipelines)` and no
`LATE_PIPELINE` and no `ERROR` in the 40 s after: no new pipeline.

**Frame cost was not measured** (the owner's perf sitting): the clutter's
reach is doubled (four times the area), every tree draw carries 60 more
vertices (most of them collapsed), and the cap fragment walks up to eighteen
leaf candidates in deciduous biomes in autumn and winter.
