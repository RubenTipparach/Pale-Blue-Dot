# Tasks

## 1. The screen goes quiet
- [x] Delete both scrims, the mode-and-FPS line, the speed/altitude block and
      the context binding strip. `hud::setup` keeps the crosshair; `hud::update`
      goes entirely.
- [x] Delete the `H` panel and its toggle. The bindings move to settings.
- [x] Keep the hotbar row and its stack counts: a slot's count is part of the
      inherited slot-grid rule, not a caption on it.

## 2. One table for the bindings
- [x] `pbd_app::controls::BINDINGS`: groups of rows, each row a `KeyCode` (or
      mouse button) list and what it does.
- [x] `controls::help_text()`, printed by `--help`, so the binding list there
      stops being a second copy.
- [x] Test: every key the table names appears in the source of a system that
      reads the keyboard. The real artifact, not an in-code copy.

## 3. Who owns the pointer
- [x] `controls::MenuOpen` and `controls::hand_over_pointer`, initialised by
      the app plugin so every consumer has it.
- [x] The walker and the pilot stand down while it is set, and take the pointer
      back on the frame it clears.
- [x] Their `Escape` arms are deleted: the key has one owner now.
- [x] `dig_and_place` requires the walker's pointer to be captured, which also
      closes the click-to-recapture dig that exists today.

## 4. The menus
- [x] `desktop::menu`: one `Screen` enum, a pause panel and a settings panel,
      both built at startup and hidden with `Display::None`.
- [x] Pause carries RESUME, SETTINGS and QUIT as hit-testable buttons with
      hover and press states. QUIT writes `AppExit`.
- [x] Settings draws one section, INPUT, as a key column and an action column
      read off `BINDINGS`, and a BACK button.
- [x] Escape steps one screen at a time and is consumed in `PreUpdate`, before
      either input reader runs.

## 5. Prove it
- [x] `--menu pause|settings` so a headless capture can photograph a screen a
      player opens with a key.
- [x] Captures: the quiet screen against `hud-before.png`, the pause menu and
      the settings page.
- [x] The suites: fmt, clippy at all targets, both test crates, the shader
      validator, `openspec validate --all`.
- [ ] The owner's in-game confirmation, which is the only thing that counts
      for feel: that Escape lands where they expect, that RESUME returns them
      to mouse look without a click, and that nothing they wanted to read is
      gone.

## 6. Held, with the reason in the design
- [ ] Freezing the simulation behind the menu. One run condition over the
      movers, reading `MenuOpen`.
- [ ] Rebinding, which needs the input systems to read the table instead of
      their own `KeyCode` constants.
- [ ] A second settings section, when there is a second thing to set.
