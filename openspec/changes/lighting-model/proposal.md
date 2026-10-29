## Why

The owner, 2026-10-28: "lighting is kinda weird, how do we get better lighting
in this game?", then "harsher shadows, svogi?", "light probes, reflection
spheres?", "We might need tone mapping and color grading balance too", "As well
as some sort of simulated hdr", and "In your mockup show what is it now vs new
lighting models".

The meadow captured at 09:00, 13:00, 17:30 and 19:00 (release build, `--view
meadow`) shows why it reads as weird:

- **The light has no time of day.** At 19:00, with the sun a few degrees up,
  the meadow is lit like noon. The sun is one fixed colour, `(1.12, 1.03,
  0.87)` in `planet_surface.wgsl`, from sunrise to sunset. Only its angle and
  a smoothstep on elevation change. There is no golden hour and no red
  sunset on the ground, although the sky above it does turn.
- **The fill is a constant, not the sky.** Shade is lit by a fixed blue-grey
  (`0.16, 0.21, 0.27` on caps and `0.30, 0.32, 0.34` on walls). It is the same
  grey under a deep blue noon sky, an orange dusk and a storm. Nothing is lit
  from below, so the ground's colour never bounces into the shade.
- **Shadows are faint.** The fill is high next to the sun, and the 3 x 3
  filter makes every edge 1.5 texels soft. On the far cascades that is metres.
  A tree's shadow is a smudge, not a shape on the ground.
- **No light bounces.** Under a canopy, an overhang or a doorway the shade is
  the same flat fill. A sunlit field does not lift the underside of what
  stands over it.
- **Nothing reflects the world.** Water, wet ground and glass reflect an
  analytic sky, never the trees or hills around them.
- **One fixed exposure and curve.** TonyMcMapface at a fixed exposure means
  noon, dusk, a cave and a lantern-lit night all go through one curve. There
  is no adaptation and no grade, and the brightest things (the sun's disc, a
  flame) do not glow.

## What Changes

Six parts, each separate and each measured against the 16.7 ms budget
(`frame-budget`). The mockup (`docs/mockups/lighting.html`) shows the game's
model as it is beside each new part, toggle by toggle.

- **L1 Sun colour from the air.** The sun's colour and strength come from the
  same atmosphere the sky is drawn from: transmittance along the sun's path.
- **L2 Sky light probe and ground bounce.** The fill comes from the sky's
  irradiance, second-order spherical harmonics (SH) updated as the sky
  changes, plus the light the sunlit ground sends back up.
- **L3 Harsher, contact-hardening shadows.** The fill is cut back where the
  sun is the light, and the filter is sharp at contact and widens with
  distance from the caster (PCSS-lite), scaled by the sun's size and the
  cloud cover.
- **L4 Sun bounce in the voxel field.** A third channel in the column tier's
  light field. Sunlit faces inject their own colour, which floods a few
  cells like lamp light, re-baked in slices as the sun moves. This is the
  game's version of SVOGI: the voxels are already there. Full voxel
  cone-tracing is written down and rejected on cost.
- **L5 Probes.** One reflection probe, a low-res cube of the sky and the
  world around the eye refreshed a face at a time, prefiltered for water,
  wet ground, glass and metal. Placed probes in towns later.
- **L6 Simulated HDR, tonemapping and grade.** Auto-exposure (eye adaptation
  from a luminance histogram), bloom for what is brighter than white, a
  tonemapper chosen on captures (TonyMcMapface, AgX or Khronos Neutral), and
  a time-of-day grade (white balance, saturation, lift) kept in data.

- **L7 Lamp bake with shadows and bounce.** Lamp light per vertex with line
  of sight and N·L, plus one bounce, baked a slice at a time and faded in.
- **Lit rain fog.** The rain's haze takes the new light: sky, shadowed sun or
  moon (shafts), and lamps glowing in it.

No save changes: all of this is derived render state.
