# glowing-mushrooms: mockup shots

These are from the Glow and carried light tab of `docs/mockups/foliage.html`
(M13, version 2, `#glow`), taken on 2026-11-19 with headless Chromium and
Playwright at 1280 x 760. A cloud container has no GPU, so they were
**rendered in software (SwiftShader)**. They show the look, not the frame
rate. Poses were set by the screenshot instrument (`window.__glow.api`). The
picking, eating and fades ran through the page's own functions.

| File | What it shows |
| --- | --- |
| `game-today-glow-plant.png` | The game today (`lamps-and-lanterns`' `meadow-glow-crop.png`): the glowing plant at night, a cyan point over its warm level-3 pool |
| `patch-night.jpg`, `patch-night-overview.jpg` | Today at night: the plants as the game draws them (a stalk and a four-sided cap) with their pools, and no carried light |
| `pick-aim.jpg`, `pick-toast.jpg` | Aiming at a plant (`F  Pick`), then the item toast with the icon and the count, and the mushroom in the hotbar and in hand |
| `held-today.jpg`, `held-torch.jpg`, `held-lantern.jpg`, `held-mushroom.jpg`, `held-compare.jpg` | The same view at night: a torch held today (no light), then a held torch (14), lantern (13) and glowing mushroom (3) |
| `glow-walk.jpg`, `glow-walk-today.jpg` | The body glow after eating one (level 10, the cap's cyan), lighting the ground, the terrace and the hand, beside the same view without it |
| `glow-plus-torch.jpg` | The glow and a held torch, added into one light at the torch |
| `glow-overview.jpg` | From above: the glow's pool around the player |
| `fade-30s.jpg` to `fade-out.jpg`, `glow-fade.jpg` | The glow's last 20 s: full at 32 s left, fading at 11 s, nearly out at 4 s, out |
| `wall-torch-overview.jpg`, `wall-today-overview.jpg` | A torch held beside the stone wall: the near side and the wall's faces are lit, and the far side stays dark. The same view today |
| `doorway-torch.jpg`, `doorway-torch-fp.jpg` | In the doorway: the light passes through the gap and nowhere else along the wall |
| `day.jpg` | By day: the caps pale, and no dusk-lit light |
| `cap-today.png`, `cap-proposed.png`, `cap-today-day.png`, `cap-proposed-day.png`, `cap-compare.jpg` | For the survey: today's four-sided cap against the proposed eight-sided dome, by night and by day, from a low camera |

## Version 3: no shadow, and a torch that burns

On the owner's notes on version 2 (2026-11-19: "no need for shadow caster from
my light source, please improve torch look"). Taken the same day with
Playwright and headless Chromium at 1280 x 760, **rendered in software
(SwiftShader)**: they show the look, not the frame rate, and the flame's
flicker and the sway can only be judged live. The before halves of the
two before-and-after pictures are version 2 rendered from the page as it was
at `e8abc95e`.

| File | What it shows |
| --- | --- |
| `torch-v3-night-fp.jpg` | First person at night: the held torch in the approved hand, its wrapped and charred head, the pixel flame, embers and the warm light on the ground |
| `torch-v3-flame-close.jpg` | A close-up of the flame and the head (the view narrowed and the hand moved to the middle by the screenshot instrument): the red rim, orange, yellow and white heart, the charred crown with live embers, the rag's wraps |
| `torch-v3-before-after.jpg` | BEFORE (v2): the flat stick, pale cone and box hand. AFTER (v3) |
| `torch-v3-wall-before-after.jpg`, `torch-v3-wall-overview.jpg` | From the far side of the one-cell stone wall, the player holding a torch against its other face. BEFORE (v2): the far side dark, the light walked and stopped. AFTER (v3): the light reaches through the wall to the ground on this side |
| `torch-v3-placed.jpg` | The Placed switch: left, the game's placed torch as `planet_surface.wgsl` draws it today (a plain post and one flat triangle); right, the same torch as the held one. The match is a recommended future look change, not made |
| `torch-v3-lantern.jpg` | The lantern held by its bail, its panes lit at night |
| `torch-v3-mushroom.jpg` | The glowing mushroom held by its stalk, the plant's own cap on a round stalk |
| `torch-v3-day.jpg` | The held torch by day |
