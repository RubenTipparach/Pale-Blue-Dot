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

## The detailed trees (group 7, design 10)

The owner, 2026-11-15: "why didnt you implement new trees for env 2". M13's
approved default is the detailed trees, not the hex prisms. Captured the same
day on the M4 Max Mac (its GPU, Metal), fixed-step `--capture` runs of the
fast build (`claude/env-trees`), each on a new world deleted afterwards, at
2880x1800. Each pair is **the mockup on the left, the game on the right**.
The mockup's own shots are `docs/screenshots/mockup-foliage/after-*.png`
where one exists; the jungle, taiga, desert, autumn and winter views were
rendered from `docs/mockups/foliage.html` for this comparison in headless
Chromium on the Mac's GPU (Metal ANGLE), walk camera, 13:00, wind off.
Game flags: `target/fast/pbd-app --weather-at 1800 --time 10 --world <new>`
and `--walk` or `--fly --height H`, then:

| file | flags | what it shows (honest read) |
| --- | --- | --- |
| `trees-temperate-walk.jpg` | `--walk --spawn fields --yaw 300 --pitch 10` | Oaks and birches as the mockup builds them: forked trunks, limbs, leaf cards cut to leaves on a twig, white birch bark with dark lenticels, fruit on the twigs, impostors behind. The game's light is an overcast morning, so its crowns are darker and less yellow than the mockup's sunlit ones; the forest is far sparser (the fields grow a tree on 13 cells in 256, the mockup's temperate forest on about a third), which is the placement rule saves keep and is not changed. |
| `trees-near-crown.jpg` | the same, full-resolution crop | The leaf cards up close: the fields leaf tile cut by the mask into leaves on a twig, pulled to the tile's mean (0.45), rooted on the branches. |
| `trees-far-crop.jpg` | the same, crop | The far billboards: oak crowns and birch trunks baked from the near models, on the hill behind. |
| `trees-autumn.jpg` | `--walk ... --day 27` | Autumn: each leaf card turns whole to a gold, orange or rust by its id, as the mockup's do, and the impostors' crowns behind take the same turn. Close to the mockup. |
| `trees-winter.jpg` | `--walk ... --day 52` | Midwinter: the deciduous cards have fallen twig by twig; the last few hang on in their autumn colour; the far trees have crossed to their bare bake (bark only). The mockup drops every card in winter at once; here `bare` rises through the season, so the crown thins rather than switching. |
| `trees-jungle.jpg` | `--walk --spawn swamp --yaw 270 --pitch 5` (this spawn lands in jungle) | Kapoks with buttress roots and long limbs, palms with ringed trunks and drooping fronds. Close to the mockup's jungle; the game's jungle leaf tile is bluer and its canopy is lit from above by the shadow cascades, so the understory is darker. Vines are not built (10.7). |
| `trees-jungle-canopy.jpg` | `--fly --height 14 --at 28.706 3.031 --yaw 270 --pitch -12` | The jungle canopy from above: kapok limbs and leaf cards, mangoes and papayas hanging from the model's anchors. |
| `trees-taiga.jpg` | `--fly --height 25 --at 61.862 26.654 --yaw 270 --pitch -20` | Pines as the mockup's: whorls of branches with needle sprays, the red bark showing, a birch among them; the near pine is the model, the rest impostors. The tundra grows a pine on 2 cells in 256, so they stand alone rather than as the mockup's forest. |
| `trees-desert.jpg` | `--walk --spawn beach --yaw 270 --pitch 0` (this spawn lands on desert sand) | A cactus up close, ribbed and armed, in the desert sheet's tile; more on the dunes behind. Cacti are scenery (10.1): not choppable. The mockup's desert palms are not drawn. |
| `trees-rings.jpg` | `--fly --height 9 --spawn fields --yaw 300 --pitch -12`, `scatter.ron` rings set per shot and restored | One view four times: every tree forced to its near model (near 60, mid 120), to the mid ring's crossed cards (near 10, mid 120), to the far billboard (near 10, mid 11), and the default (44, 90). The same trees at every level, in the same colours since the crown's tones were added to the bake's palette (design 10.3); forced this close, the cards show what they are (flat, blockier, the bark lines over the crown on the billboard), which the default rings keep beyond 31 m. |
| `trees-fly-away.jpg` | `--fly --height 8, 16, 30, 45, 70 --spawn fields --yaw 300 --pitch -25` | Rising away from the same grove: the trees pass from model to cards to billboards with no ring of change visible in any frame. A still cannot show a pop; the owner's video (the batch) is what will. |
| `trees-temperate-orbit.jpg` | `--fly --height 60 --spawn fields --yaw 300 --pitch -30` | The forest from above: every tree a billboard at this height. Sparse beside the mockup's orbit (the placement density above), and a billboard seen from steeply above is thinner than the mockup's orbit, which looks from lower. |

**Not seen:** the acacia. It is built and baked, but no tree stands on a beach
cell: the generator makes every beach cell sand (`planet_gen::top_material`),
and a tree needs grass (`grows_tree`), so the beach's tree density has never
grown one, prisms or detailed. The swamp's willows were not caught either:
the swamp found by the biome search was a pool with no grassy cell in view.

Checked: no `ERROR` in any capture; a normal (warming) launch,
`PBD_SETTINGS=<scratch> target/fast/pbd-app --world trees-warm --walk --spawn
fields`, logged `tree kit: 32 models, 6042 KiB, built in 372 ms` behind the
loading screen, `READY playable 5.89 s after launch (104 pipelines)` and no
`LATE_PIPELINE` and no `ERROR` in the two minutes after.

**Frame cost was not measured** (the owner's perf sitting). What it draws:
near model vertices per tree (largest variant, plus 60 for fruit): oak 2904,
birch 1584, kapok 588, palm 492, acacia 696, pine 6126, willow 1296, cactus
588; the near draws list at most 4,096 trees a species (inside 44 m a jungle
holds about 390); every tree past them is 30 vertices (mid cards and
billboard), down from the prisms' 258; the sun cascades draw the near trees
as well.

## The crops and wild plants in the world's style (task 8.1, 2026-11-15)

The M4 Max Mac, the fast build. Each shot is a `--capture` run on a new world:
`--walk --spawn fields --weather-at 1800 --time 10 --grass tufts`. The tufts
are what players see, while captures default to the blades.

| file | flags | what it shows |
| --- | --- | --- |
| `crops-before-field.jpg`, `crops-after-field.jpg` | `--yaw 270 --pitch -24 --farm 6` | Wheat and turnip at all four stages, and a wild wheat and chili at the right. Before: M8's outlined stages, with ink rings, dark seams and near-black shadows. After: the outline-free world stages that M8 and M13 draw. |
| `crops-field-crop.jpg` | | The same rows enlarged, before above and after below. |
| `crops-before-wild.jpg`, `crops-after-wild.jpg` | `--yaw 110 --pitch -12` | A wild frost kale on the terrace and a wild chili above it. They are small at this distance. |

Read: the crops no longer sit on top of the ground as outlined stickers. They
share the tufts' and the trees' soft edges. The goods and seed icons in the
hotbar and pack keep their outline, because they are UI. The floating seed
sack is a dropped item, and a dropped item is an icon.
