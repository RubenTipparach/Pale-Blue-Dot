# Tasks

## 0. Planning (this step)
- [x] Port the UI skills from Fleet Command (`a4a02f6`): `game-ui-frontend`,
      `ui-ux-pro-max`, `blender-ui-panels`, `blender-organic-panels`,
      `panel-textures`, each with an "In Pale Blue Dot" note and a description
      that triggers here; provenance in `.claude/skills/PROVENANCE.md`.
- [x] Check what runs: `ui-ux-pro-max`'s search (Python 3.13, standard
      library); Blender through the `bpy` 4.5.16 wheel under Python 3.11 (CPU,
      no GPU in the cloud container).
- [x] Study the UI as it is (`desktop/*.rs`, `loading.rs`, the screenshots) and
      the mockups' panels that will become screens.
- [x] Draw the kit: `tools/art/ui_art.py` into `docs/mockups/ui/` with
      `slices.json` (every colour Sweaft-64, contrast measured).
- [x] Render the frame and the rock slab in Blender and pixelate them:
      `tools/art/ui_panels_blend.py`.
- [x] The mockup `docs/mockups/ui.html` ("Stone and Timber"): kit, HUD by day
      and night before and after, pack, pause and settings, map, station,
      talk and trade, loading.
- [x] Verify it in headless Chromium (SwiftShader, no GPU): no console errors,
      every tab draws, no WebGL context, no horizontal scroll at 390 px, every
      9-slice and pixel image at a whole-number scale and pixelated. Shots in
      `docs/screenshots/mockup-ui/`.
- [x] Fold in the owner's Q2 answer (2026-11-14, "I like the stone kit more,
      can you make the stone more blue?"): menus in rough rock, slate-blue
      stone and rock ramps.
- [x] Fold in "phones cant run this lol, mostly stone with some occasion
      timber for accent" (2026-11-14): remove the Phone tab, the phone-width
      layouts and the phone requirement (the game is desktop only; integer UI
      scale from the window height); redraw every piece stone-first with
      three oak accents (lintel, open tab's cap, compass pointer), the oak
      draft kept behind `ui_art.py --oak`; re-measure every text pair (all
      4.5:1 or more, the run fails otherwise); re-render the Q1 rock in the
      blue ramp.
- [x] Verify again in headless Chromium (SwiftShader, no GPU): no console
      errors, every tab draws, no horizontal page scroll at 390 px (scenes
      scroll in their own box), every pixel image and 9-slice border at a
      whole-number scale and pixelated. Shots replaced in
      `docs/screenshots/mockup-ui/` (phone and blue-step shots removed).
- [x] The base rock (2026-11-14): the first render minus the strata rings,
      rendered at each in-play panel's size, no outline; approved by the owner
      from the parent session's sample ("Ok good continue with the mockups").
- [x] The whole kit rendered under the base rock's material
      (`ui_panels_blend.py`): every stone piece, the oak set for the timber
      menus, the rock per panel; pixelated and smooth (8x; the rock 4x); every
      text pair 4.5:1 or more in both, measured.
- [x] The drawn rock's dashes replaced by irregular cracks and speckle
      (`ui_art.py rock_panel`).
- [x] The kit font (`ui_art.py`, `ui/font/`) for titles, labels, tabs, the
      compass and key caps.
- [x] Timber out of play (main menu, pause, settings, saves), rock in play.
- [x] The page: Drawn | Rendered | Smooth switch; Play and Main menu tabs; every
      menu interactive; Settings, Input in sub-tabs with rebinding.
- [x] The Play demo (three.js r128): a hillside to walk, dig and place, five
      machines and a chest, a smith to talk and trade with, day to night.
- [x] Verify in headless Chromium (SwiftShader, no GPU): no console errors in
      any look, every tab draws, no horizontal page scroll at 390 px, whole-number
      scales, every menu flow in all three looks, the demo's checks. Shots in
      `docs/screenshots/mockup-ui/`.
- [ ] Publish the page, add it to the mockup tracker, put Q1 ("which rock") in
      the survey with `q1-which-rock-hud.jpg` and `q1-which-rock-pack.jpg`, and
      move Q2 to "Already decided".
- [ ] The owner answers Q1 and approves the mockup for build.

## 1. The kit in the game (waits for the owner's go-ahead)
- [ ] Confirm the Bevy 0.18.1 names (`ImageNode`, `NodeImageMode::Sliced`,
      `TextureSlicer`, `SliceScaleMode`, `UiScale`) against the crate.
- [ ] `assets/ui/` (the kit's PNGs and `slices.json`, copied from
      `docs/mockups/ui/`), `assets/fonts/`
      (Pixelify Sans, VT323, their OFL texts).
- [ ] `desktop/ui_kit.rs`: `UiKit` (handles, nearest sampler, margins read
      from `slices.json`), the tokens, `nine(piece)`.
- [ ] Integer `UiScale` from the window height; native-pixel constants.
- [ ] The kit font as a Bevy bitmap font (atlas from `kit-font.png`, metrics
      from `kit-font.json`), VT323 for body text.
- [ ] Rock panels baked at their sizes (or a long baked edge, only cropped),
      once Q1 is answered; the timber set for the menus out of play.
- [ ] Test: no two opposite margins in `slices.json` add up to more than the piece; every PNG
      is on the palette with alpha 0 or 255 (the art file itself, not a copy).
- [ ] Capture: a frame at scale 2 draws its corners 28 px, pixel exact.

## 2. Screens, one at a time, each with day and night captures
- [ ] Pause, settings, saves (`menu.rs`) in the timber set, plaques in every
      state, Input in sub-tabs with rebinding; delete the palette copies as
      each file moves to the tokens.
- [ ] Slots, tool slot, pack (`slots.rs`, `equipment.rs`, `pack.rs`), tooltips.
- [ ] Compass bar (`compass_bar.rs`).
- [ ] Map frame and overlay drawer (`map_screen.rs`, `overlay_ui.rs`).
- [ ] Loading screen (`loading.rs`), its images and fonts in the prewarm;
      no `LATE_PIPELINE`.
- [ ] Guide, time and weather bars (`guide.rs`, `time_ui.rs`, `weather_ui.rs`).
- [ ] `perf_suite.py` on the owner's desktop, before and after.

## 3. Video (batched with the others)
- [ ] The step's video, when the owner calls for the batch.
