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
- [x] The owner answers Q1 and approves the mockup for build (2026-11-14: "Smooth is the best. Write it down as the decision and start working the current UI as the wood + rendered smooth rock").

## 1. The kit in the game (go-ahead given 2026-11-14: smooth rock in play, smooth oak out of play)
- [x] Confirm the Bevy 0.18.1 names (`ImageNode`, `NodeImageMode::Sliced`,
      `TextureSlicer`, `SliceScaleMode`, `UiScale`) against the crate: design,
      "Section 1 as built" (`BorderRect` is `min_inset`/`max_inset` now; a
      slice corner grows with its node unless `max_corner_scale` caps it).
- [x] `assets/ui/` (the smooth stone, oak and per-panel rock renders the game
      draws, and `slices.json`, copied from `docs/mockups/ui/rendered/` at
      `d69ee33`), `assets/fonts/` (the kit font, VT323 and its OFL text);
      provenance in each folder's README. Pixelify Sans is not used: the kit's
      own font replaced it for every label.
- [x] `ui_kit.rs` (in the library, `crates/pbd-app/src/ui_kit.rs`, since the
      loading screen is there): `UiKit` (handles; LINEAR for the kit's smooth
      pieces, the owner's UI exception; NEAREST for the font; margins from
      `slices.json`), the tokens, `nine`, `whole`, `rock`, plaques in five
      states driven by `Interaction`, the lintel, oak tabs.
- [x] Integer `UiScale` from the window height (`ui_scale_for`), set so the
      window's scale factor times it is the whole number; the pieces baked
      again into the same handles when it moves a step; every `px()` in the
      interface in native pixels.
- [x] The kit font as glyph image nodes (`KitText`: an atlas from
      `kit-font.png`, metrics from `kit-font.json`, nearest, whole native
      pixels, a carved shadow); VT323 installed as the default face for body
      text.
- [x] Rock panels baked at their sizes (the pack, the loading slab), drawn
      whole; the smooth oak set for the menus out of play.
- [x] Tests: no two opposite margins in `slices.json` add up to more than the
      piece, and every copied render is its piece's size times its texels;
      the kit bakes linear at 1 to 4 and the font is nearest; the map frame is
      hollow; the scale for a set of heights; the font has every label glyph;
      the tokens clear 4.5:1; no UI file keeps a colour of its own; the timber
      menus are oak; the pack is framed in its own rock.
- [x] The existing screens restyled (the rest of section 2 is folded in here,
      at the owner's go-ahead): pause, settings (tabs Picture, Weather, World,
      Movement, Input; checkboxes, value plaques, sliders), saves; hotbar, tool
      slot and picker; the pack; the compass; the map frame, drawer and
      readout; the overlay legend; the field guide; the loading slab. Captures
      in `docs/screenshots/stone-and-wood-ui/` (lavapipe, no GPU).
- [x] Capture: a frame at scale 2 draws its corners 28 px, pixel exact,
      measured off the PNG (`tools/art/measure_corners.py` against the frame
      as the game bakes it; README, "Corners").

## 2. Screens, one at a time, each with day and night captures
(Design, "Section 2: Input in sub-tabs with rebinding, sliders, tooltips",
written before the code. Captures in `docs/screenshots/stone-and-wood-ui/`,
lavapipe, no GPU; frame cost not measured.)
- [x] Pause, settings, saves (`menu.rs`) in the timber set, plaques in every
      state, Input in sub-tabs with rebinding (`controls::remap_keys`, the
      rebinds kept in `settings.ron` version 3); the frame limit and render
      scale as stone sliders; no palette copies left (pinned by
      `ui_kit::tests::no_ui_file_keeps_its_own_palette`).
- [x] Slots, tool slot, pack (`slots.rs`, `equipment.rs`, `pack.rs`), tooltips
      (`ui_kit::Tooltip`, standing over the slot).
- [x] Compass bar (`compass_bar.rs`).
- [x] Map frame and overlay drawer (`map_screen.rs`, `overlay_ui.rs`). The
      drawer stays open by default: shutting it is a behaviour change, kept
      as a recommendation (design, "The map's drawer stays open"; the
      change's spec delta).
- [x] Loading screen (`loading.rs`), its images and fonts in the prewarm;
      no `LATE_PIPELINE` after `READY` from the UI (the real-time run in the
      README; the capture's own `screenshot-to-screen` is the one late one).
- [x] Guide, time and weather bars (`guide.rs`, `time_ui.rs`, `weather_ui.rs`).
- [ ] The map's drawer shut by default behind a DRAWER plaque: a
      recommendation, not done (the spec delta's one requirement).
- [ ] `perf_suite.py` on the owner's desktop, before and after (the cloud has
      no GPU; frame cost not measured here).

## 3. Video (batched with the others)
- [ ] The step's video, when the owner calls for the batch.

## 4. Interact on F, and its prompt (owner, 2026-11-16)
(Design, "Interact on F, and the prompt that names it", written before the code.)
- [ ] One INTERACT slot, `interact:KeyF`, replacing `hands:KeyE` and
      `vehicles:KeyF`; the F readers (husbandry, fruit, farm, doors,
      boarding) read F; E free on foot, Q/E still roll.
- [ ] `pbd_app::interact`: the priority table, the offers, `now` with its
      0.15 s hold; each reader acts only on its own row.
- [ ] `farm::plant_on_ray`: crops and wild plants found by their own shape.
- [ ] The prompt under the crosshair, with the bound key; hidden in panels.
- [ ] Settings version 8: the old slots' rebinds migrate.
- [ ] Texts: missions help, notes, showcase presses.
- [ ] Tests: rebinds, an F harvest and pick through the input path, an old
      file's custom use binding kept, F boards with nothing usable aimed.
- [ ] Captures in `docs/screenshots/interact-f/`: the controls page and a
      prompt of each kind.
