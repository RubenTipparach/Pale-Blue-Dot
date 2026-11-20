# Design: SMAA as the last step

## Context

- The camera has no multisampling by default, since `ocean-environments`
  B24. `PBD_MSAA=2|4` still brings it back for comparison.
- The water's node runs after Bevy's main pass. It covers compose, cap,
  lens, clouds and rain, and reads the scene's depth.
- Bevy's SMAA node runs in the post-processing chain after tonemapping,
  so after the water's node. Expect `Node3d::Smaa` after
  `Node3d::Tonemapping`; to be confirmed against the graph when built,
  and pinned by a test.

## Decisions

1. **SMAA, not FXAA.** SMAA finds edges by shape, and softens nearest-
   sampled texel boundaries far less than FXAA's luma blur. High preset;
   Ultra can come later if wanted.
2. **A player setting, OFF by default.** It joins the settings screen
   beside REFLECTIONS and SHADOWS. The settings file gains `antialiasing`,
   with a version bump, and an older file reads OFF. A capture or
   benchmark runs OFF (`PlayerSettings::ISOLATED`), with
   `--antialiasing smaa` to ask for it.
3. **Never with multisampling.** SMAA is the only antialiasing. If
   `PBD_MSAA` asks for samples, SMAA is not added, so the two are never
   stacked.
4. **Prewarmed.** The loading screen compiles SMAA's pipelines whatever
   the setting, so switching it on adds no `LATE_PIPELINE`.
5. **Measured on the desktop.** A frame-time A/B of OFF against SMAA comes
   from the owner's machine. A cloud session has no GPU, and says so.

## Risks

- **Texture softening.** SMAA may still soften some high-contrast texel
  edges in the atlas. The owner judges that on stills of OFF against SMAA,
  which is why it stays off by default.
