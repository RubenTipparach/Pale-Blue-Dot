# Design: the ablation, and which lever actually moves the sea

## The experiment

`water.ron` is read at runtime, so a look knob can be changed and photographed
without a rebuild. Four variants were rendered against the shipped values, each
on two presets, at a fixed frame count on the same binary, and the sea's mean
sRGB was measured over a fixed band of the frame. This is the measurement
instrument the rules allow: no behaviour was changed, the file was restored, and
what follows is the number the write-up needed.

The two presets are the two cases: `shore` is an eye 1.6 m up looking across a
shelf that is **one metre deep to the horizon**, and `wade` is an eye at the
waterline looking out over deeper water.

## What each knob is worth

**`shore`**, the sea band, mean sRGB and saturation:

| variant | sea | saturation | moved |
| --- | --- | ---: | ---: |
| shipped | 124.4, 149.1, 139.5 | 24.7 | |
| `specular_intensity` 0.12 → **0** | 123.5, 148.5, 139.1 | 24.9 | **~1** |
| `sky_horizon_strength` 0.35 → **0.05** | 121.2, 146.7, 134.4 | 25.5 | **~4** |
| sky reflection colours cut 40% | 121.2, 146.5, 137.1 | 25.3 | **~3** |
| absorption doubled, `deep_color` darkened | **93.6, 132.7, 135.3** | **41.7** | **~31** |

Turning the sun glint off entirely moves the sea by **one level out of 255**.
Cutting the grazing Fresnel by seven and the reflected sky by nearly half move
it by three or four. **Every shine knob in the shader, together, is worth about
2% of the colour of that frame.** What the sheet is showing is the sand under
it, and no reflection knob can take that out.

The fourth row is the one that matters. Doubling the **absorption** and
darkening the deep colour moves the sea by **31 levels of red and 17 of
saturation**, roughly ten times what all three shine knobs do together. That is
the proof of what the sheet is made of: it is the transmitted path, the seabed
seen through the water, and the only two things with authority over it are how
much water is in front of the sand and how hard a metre of that water tints.

**`wade`**, where there is water under the view, tells the other half:

| variant | far sea | saturation | near sea | saturation |
| --- | --- | ---: | --- | ---: |
| shipped | 90.9, 127.9, 149.0 | 58.1 | 124.3, 151.8, 152.1 | 27.8 |
| `specular_intensity` → 0 | 95.3, 130.1, 151.0 | 55.7 | 115.8, 146.4, 140.9 | 30.6 |
| `sky_horizon_strength` → 0.05 | **69.0, 115.4, 138.7** | **69.8** | 110.7, 143.6, 135.4 | 32.8 |
| sky colours cut 40% | 82.2, 121.5, 145.4 | 63.2 | 112.2, 143.8, 138.4 | 31.6 |
| absorption doubled | 91.0, 117.8, 143.0 | 52.0 | **86.6, 128.7, 136.4** | **49.8** |

Here the Fresnel floor is worth **22 levels of red and 12 of saturation**. So
the knob is not useless; it is powerless *over a sand flat*, which is what the
shore frame is entirely made of. Absorption is again the strongest term, and
this row also shows why it is the wrong lever: it takes 38 levels of red out of
the NEAR water, where a player is standing in the shallows and should be able
to see their own feet.

## The decision

1. **Deepen the sea first.** `OCEAN_RELIEF` is what made the shore frame a sand
   flat, and until it moves, nothing in the shader can be judged: every
   candidate value will measure as "no change" on the frame the owner is
   looking at. The land relief does not move with it. The two were compressed
   together in the rescale for no reason beyond being adjacent in the same
   expression.
2. **Then re-judge the Fresnel floor, and only that one.** `sky_horizon_strength`
   is the single knob with measurable authority over deep water at a grazing
   angle. The ablation at 0.05 is too far, it takes the sky out of the water
   altogether; something around **0.15 to 0.20** is the range to photograph,
   against the reference's 0.50 and our current 0.35.
3. **Absorption is the one knob with real authority, and it is the wrong one to
   reach for.** It is worth 31 levels on the shore frame, more than everything
   else measured put together, because it multiplies the path the light takes
   through the water. But absorption per metre is a property of water, not of a
   planet, and ours is the reference's exactly. Raising it to compensate for a
   seabed that is too close is fixing the symptom, and it would then be wrong
   everywhere the depth is right: the `wade` and `dive` frames would go black.
   It is recorded here as the measured alternative, and as the fallback if the
   owner wants the frame changed without touching the terrain.
4. **Leave the specular alone.** One level out of 255 is not a look, and ours is
   already 60% below the reference's.
5. **Do not copy the reference's numbers back in.** Tenebris applies no tone
   mapping anywhere: its composite ends on a bare write and the framebuffer
   clips. Its near-white horizon reflection (0.85, 0.92, 0.98) and its 0.405
   specular peak are authored *for that clip*. Under `TonyMcMapface` the same
   values would lift and desaturate into exactly the pale sheet this change
   exists to remove.

## What "shiny" turned out to mean

Nothing in the frame is clipping: the brightest sea pixel is 174 of 255 and not
one pixel passes 200. There is no highlight to turn down. The complaint is
about a large area sitting at a flat desaturated mid-value, and the cause is
that the area is a sand flat under a film of water, seen through a tone curve
that takes the remaining saturation out. **A look complaint names a symptom,
and the first thing to measure is whether the term you are about to tune has
any authority over the pixels in question.**
