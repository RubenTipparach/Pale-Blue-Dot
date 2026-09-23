# Proposal: a storm you can stand in, watch and call

## Why

The owner, on the first build of `overcast-and-rain`:

- "I want to see the puddle shaders! ... I'm supposed to see rain drops on the
  ground when it's raining! ... I barely see puddles."
- "can you also add a weather toggle or slider for me too?"
- "it should snow in the snow biome, not rain lol"
- "use volumetric fog instead of these rain quads for distant rain please ...
  animate the rain volumetric fog so it looks like rain ... and make sure it is
  sorted correctly according to regular clouds"
- "if you can do some lightning, lightning should light up clouds and ground"
- "clouds aren't rendering over water, fix this also"

Each is a real gap, and three of them are my own regressions or bugs:

- **Rings went missing.** `overcast-and-rain` put the rain rings only inside
  puddles, and grass never puddles, so a meadow in a storm shows none. And the
  puddle mask covers far less than it claims: `rain_puddle_share` 0.35 gives
  13.9% of a soaked floor and 0.6% of a half-soaked one, measured by
  transcribing the mask, because the threshold treats a sum of value noise as
  uniform when it is bunched round 0.5.
- **Distant rain is quads.** The far curtains are 300 m sheets on a lattice;
  from inside a storm their tops fan overhead as bands, unlit at night.
- **Clouds vanish over the sea.** The sea is drawn by a pass after the main
  pass, and the cloud shell writes no depth, so the sheet paints straight over
  every cloud in front of it. From orbit the whole ocean is cloudless.
- **Snow falls as rain.** The field already knows a cold column (`Precip::Snow`)
  and `Weather.snowing` carries it; nothing draws it.
- **No way to call the weather** except the P key's three steps.
- **No lightning.**

## What

Two parts, the first a correction of the change before it:

1. **Rain on the ground you can see** (`overcast-and-rain` section 7): rings on
   every wet upward face again, grass included at a lighter touch, as bends of
   the sky reflection rather than dark lines; the puddle mask calibrated so its
   share is the share you get.
2. **The storm** (this change):
   - one cloud march, shared by the sky and the sea, so clouds cover the ocean;
   - volumetric rain in the after-scene pass, from a rain map the CPU fills off
     the field, animated as falling streaks, lit by the sky, layered under the
     clouds and in front of the ground; the curtains removed;
   - snow where the field says snow: slow white flakes near, a white volume
     far, no lens drops and no wet ground;
   - lightning: a strike chosen off the field in the heaviest rain nearby,
     flashing the clouds, the rain and the ground, with a bolt;
   - a weather slider in the pause menu.
