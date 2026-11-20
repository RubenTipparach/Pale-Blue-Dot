# Beavers and otters (M25 version 2, `ocean.html#critters`)

Screenshots of the "Beavers and otters" tab of Seas, Rivers and Marshes
(https://claude.ai/artifact/Ky4u9ybCSUm31t72NRMAA1), the mockup for
`openspec/changes/ocean-environments` design B28 and task group 19.

Rendered in a Claude Code cloud container with no GPU: headless Chromium on
SwiftShader, 1280 x 800 (the UI shots 1440 x 1000 and 390 x 844). They show
poses and the look, never how smoothly anything moves. Each frame opens the
page afresh with the default seed (7), holds the clock still, and steps the
scene in fixed 1/30 s steps to the named moment, so the frames repeat.

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
