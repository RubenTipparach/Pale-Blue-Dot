# Proposal: the composite pass, so the water can be seen from inside it

## Why

`docs/tenebris-comparison.md` ("Water: five systems in Tenebris, one branch
here") measured the whole Tenebris water system against this project. Binding
the cap-pass port, which `preview-scale-and-shader-parity` plans, restores the
sheet seen from above. It does not restore the sea seen from **inside**: in
Tenebris that is the composite pass (`composite.fs.glsl`, 339 lines), a
full-screen pass that runs before the water draws and again after it, and this
project has no post-process pass of any kind.

Without it a camera below the surface sees a crisp seabed to the horizon under
an opaque lid, the sky shows through the water while wading, and rain on the
lens and drips after surfacing have nowhere to be drawn.

## What changes

One render-graph node after Bevy's main pass owns three sub-passes in Tenebris's
order: **compose** (underwater fog, distortion and depth blur into the
post-process destination, a plain copy where the pixel is dry), the **water
cap** draw over it, and a **lens** pass (rain-on-glass droplets and
emerge-from-water drips) that samples the post-water image, so drops refract
the real sea. The camera's submersion is decided on the CPU from its body-local
position as a tri-state: dry, straddling the surface, fully under.

Every term is Tenebris's, term for term; the knobs live in `assets/config/water.ron`
with the units and defaults Tenebris ships.

## Non-goals

- Rain itself. The lens droplets read a rain intensity; the field that supplies
  it is `weather-rain`. With no weather the lens pass is skipped.
- Screen-space caustics, which Tenebris does not have either.
