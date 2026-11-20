# Beavers and otters (M25 versions 2 and 3, `ocean.html#critters`)

Screenshots of the "Beavers and otters" tab of Seas, Rivers and Marshes
(https://claude.ai/artifact/Ky4u9ybCSUm31t72NRMAA1), the mockup for
`openspec/changes/ocean-environments` design B28 and task group 20.

Rendered in a Claude Code cloud container with no GPU: headless Chromium on
SwiftShader, 1280 x 800 (the UI shots 1440 x 1000 and 390 x 844). They show
poses and the look, never how smoothly anything moves. Each frame opens the
page afresh with the default seed (7), holds the clock still, and steps the
scene in fixed 1/30 s steps to the named moment, so the frames repeat.

## Version 3 (`v3-*.jpg`, `contact-sheet-v3.jpg`)

The owner's notes on version 2: "Some weird geometry poking out from middle of them. Also need textures for
both. Some fur tuffs would be nice too", and "See mining mike how we did that for the wolf". Each animal is one
sculpted, skinned skin with painted pixel-art fur and fur tufts (`tools/art/critter_sculpt.py`, design B28
version 3). Same tool and `shots.json` (every name now has the `v3-` prefix; the unprefixed files are version 2).
Rendered the same way: headless Chromium on SwiftShader, no GPU.

| Frame | What it shows |
| --- | --- |
| `v3-beaver-close-side.jpg`, `v3-beaver-close-above.jpg` | The beaver walking, close, from the side and from above: one skin, the legs grown out of it, the fur texture, the tufts on the back and nape. |
| `v3-beaver-gnaw-owner.jpg`, `v3-beaver-gnaw-back.jpg` | The gnaw from the owner's angle (where version 2's slabs stood out of the flank) and from behind. |
| `v3-otter-close-side.jpg`, `v3-otter-close-above.jpg` | The river otter bounding, close. |
| `v3-otter-wet-close.jpg` | The river otter wet on its rock after the catch: the tufts clumped into spikes. |
| `v3-beaver-shake.jpg` | Wet spikes the moment before the shake flattens them. |
| `v3-sea-otter-close-side.jpg`, `v3-sea-otter-close-above.jpg` | The sea otter on its back, the pale head and throat. |
| `v3-dusk-beaver.jpg`, `v3-dusk-valley.jpg` | Dusk at 17:30: a warm low sun. |
| `v3-night-*.jpg` | Night, lit by the lantern, the camp, the boat's lamp and fireflies. |
| the other `v3-*.jpg` | The version 2 frames retaken with the new bodies and the aspens' bark. |

Made with `tools/mockup_critters_shots.js` from `shots.json` here:

```sh
THREE_JS=<three.min.js r128> NODE_PATH=$(npm root -g) \
  node tools/mockup_critters_shots.js $PWD/docs/mockups $PWD/docs/screenshots/beaver-otter-mockup \
  docs/screenshots/beaver-otter-mockup/shots.json
```

`contact-sheet.jpg` has every scene frame on one page.

| Frame | What it shows |
| --- | --- |
| `beaver-gnaw-anticipation.jpg`, `beaver-gnaw-hit.jpg` | Reared on hind legs and tail, forepaws on the aspen: the held beat, then the bite with chips flying. |
| `beaver-felling-creak.jpg` | The tree leans back over the cut while the beaver scrambles clear. |
| `beaver-felling-fall.jpg`, `beaver-felling-hit.jpg` | The fall, slow then fast, and the hit with leaves flying. |
| `beaver-drag.jpg` | A branch in its teeth, the leafy end dragging on the ground, on the waddle. |
| `beaver-swim-branch.jpg` | Swimming with only the head and the branch above water, rings behind. |
| `beaver-dam-push.jpg` | Rearing against the dam as the branch goes onto the crest. |
| `beaver-tail-slap-raise.jpg`, `beaver-tail-slap-hit.jpg` | The tail held high, then the slap: splash and rings. |
| `beaver-shake.jpg` | Shaking off on the bank, drops flying. |
| `otter-bound.jpg` | The bounding gallop along the ridge. |
| `otter-slide.jpg`, `otter-splash.jpg` | On its belly down the mud slide, and into the pond. |
| `otter-porpoise.jpg` | Porpoising out of the pond. |
| `otter-eat.jpg` | On its rock, a trout in its forepaws, a bite on the beat. |
| `otter-groom.jpg`, `otter-look.jpg` | On its back grooming; reared up looking round. |
| `sea-otter-crack-windup.jpg`, `sea-otter-crack-hit.jpg` | On its back in the kelp: the urchin raised, then brought down on the stone. |
| `sea-otter-dive.jpg` | Rolling belly down to dive. |
| `sea-otter-sleep-kelp.jpg` | Asleep, wrapped in a kelp frond, paws over its eyes. |
| `night-pond.jpg`, `night-beaver-lantern.jpg`, `night-sea-otter-boat.jpg` | 22:30: the bank's lantern, fireflies and the boat's lantern light what is near them; the moon stays dim. |
| `dusk-valley.jpg` | The valley at 18:00 from above. |
| `today-pond.jpg` | Today's motion: the rigid models as the dam and river tabs move them. |
| `ui-desktop.jpg`, `ui-phone.jpg` | The tab with its side panel and Tune open; the phone layout (no horizontal scroll at 390 px). |
