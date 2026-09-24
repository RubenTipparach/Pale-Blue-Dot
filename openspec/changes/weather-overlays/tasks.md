# Tasks

- [x] Core functions for each overlay field (humidity, sunlight, temperature,
      rain rate): `Atmosphere::sample` already computes each one, and
      `pbd_core::overlay::Overlay::texel` picks it, with its unit and range.
- [x] The overlay cube, filled only while an overlay shows (`overlay.rs`, off
      the main thread; in place for a capture).
- [x] The overlay pass: colour ramps and streamlines; in the body-local frame.
      It is compiled by Bevy's composer at startup (the water shader has
      `#ifdef`s, so it is not in the raw naga tests), and the captures ran with
      no pipeline error.
- [x] `OverlayMode`, the M key, a pause-menu row per overlay, `--overlay`.
- [x] The legend, with the ramp image held to the shader's table.
- [x] Captures from orbit (`docs/screenshots/overlay-*.png`): wind, jet, currents,
      cloud, rain, sunlight, temperature. Humidity not yet captured.
- [ ] Streak alignment measured (mean angle between a streak and the flow
      under it); not done, the captures are judged by eye so far.
- [x] fmt, clippy, tests, `openspec validate --all`.
- [ ] Owner's in-game check.
