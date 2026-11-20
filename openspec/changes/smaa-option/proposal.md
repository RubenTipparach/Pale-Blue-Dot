# Proposal: antialiasing as the last step, a player's choice

## Why

The owner, 2026-11-20, after multisampling was turned off to cure the
underwater outlines (`ocean-environments` B24):

> so how do we apply AA on top of that? it seems the mask and the color
> layer were applied at different times? and not together?

and, of the options (SMAA at the end, supersampling, per-sample fog, TAA):
"recommended seems best".

The 4x multisampling resolved the colour right after the main pass, while
the water's passes kept reading per-sample depth. So the mask and the
colour were finished at different times. Antialiasing has to come after
everything that reads depth: the water, the fog, the clouds and the lens.

## What Changes

1. **No antialiasing stays the default.** Hard pixel edges suit the
   nearest-sampled look, and the default costs nothing.
2. **A new player setting, ANTIALIASING: OFF / SMAA.** SMAA is Bevy's
   `Smaa` (high preset), and it runs on the finished frame after every
   water and cloud pass, so nothing can mismatch it.
3. **No late pipelines.** The loading screen prewarms SMAA's pipelines
   too, so switching it on in play compiles nothing (CLAUDE.md, "A loading
   screen on boot").
4. **The setting is saved** with the rest. Older settings files read OFF.

## Capabilities

### New Capabilities

- `platform/antialiasing`: the setting, where SMAA runs, and its default.

## Impact

- `pbd-app`: `settings.rs` (the setting, its file field, the follow
  system), the settings screen row, `desktop.rs` (the camera), and the
  loading prewarm.
- **Saves:** no world data. The settings file gains one field with a
  default, and its version moves up by one.
- **Look:** off by default, so nothing changes until a player chooses
  SMAA. Comparison stills of OFF against SMAA come with the build.
