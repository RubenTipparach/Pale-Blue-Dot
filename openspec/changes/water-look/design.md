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

## Second round: the owner's view against a photograph

After the sea was deepened the owner sent their own frame beside a photograph
of open ocean and said the water was still too shiny and too light. The photo
was measured the same way, in four bands from the horizon down:

| photo band | sRGB | saturation |
| --- | --- | ---: |
| at the horizon | 58, 177, 209 | 151 |
| upper | 25, 151, 191 | 166 |
| middle | 9, 100, 143 | 134 |
| foreground | 4, 58, 91 | 87 |

**The photo is not dark, it is saturated.** Its red channel is under 60
everywhere and under 10 over most of the sea, while its blue runs to 209. The
owner's frame (the `shore` preset from 12 m) measured 84, 115, 138 at
saturation 55: the same lightness as the photo's middle band with four times
the red in it. "Too light" named a symptom; the number is red.

Every knob was then rendered one at a time at that view, on the deepened sea,
150 frames each, the far sea band (rows 110-190) measured:

| variant | far sea | saturation |
| --- | --- | ---: |
| shipped | 71.4, 106.2, 136.2 | 64.7 |
| sky colours to a clear-day blue (0.20/0.42/0.70, 0.06/0.22/0.55) | 58.1, 100.9, 135.7 | 77.7 |
| `deep_color` darkened to 0.003/0.08/0.18 | 61.8, 101.0, 129.9 | 68.1 |
| waves calmed (steepness 0.45, slope cap 0.7) | 69.5, 104.6, 135.1 | 65.6 |
| specular halved, sun tint cooled | 71.4, 106.2, 136.2 | 64.7 |
| Fresnel floor 0.35 to 0.22 | 66.0, 102.5, 133.5 | 67.4 |
| all five together | 40.1, 91.5, 125.6 | 85.5 |

Together they take 31 levels of red out and reach saturation 85, against the
photo's 134. **Darkening the body colour is the wrong direction**: it moves the
frame five levels and reads as grey. The lever that was left is the body colour
made brighter and purer, since the reflected sky is now as blue as a sky gets
and the tone mapper takes saturation out of everything that goes through it:

| variant (on top of the five) | far sea | sat | near shallows | sat |
| --- | --- | ---: | --- | ---: |
| `deep_color` 0/0.16/0.36 | 42.0, 112.5, 152.4 | 110 | 75.2, 133.7, 139.2 | 64 |
| `deep_color` 0/0.24/0.48 | 47.6, 127.7, 164.7 | 117 | 76.1, 139.0, 144.7 | 69 |
| 0/0.18/0.40 and absorption 0.90/0.25/0.08 | 43.4, 116.8, 155.8 | 112 | 55.3, 130.1, 142.7 | 87 |
| the same, sky 0.10/0.36/0.72 and 0.03/0.18/0.55 | 37.6, 115.8, 156.1 | 119 | 54.6, 129.9, 142.7 | 88 |

Two things fall out. A body colour with **no red in it at all** is what a
saturated sea needs under this tone mapper, and Tenebris's 0.02 of red is
worth ten levels on screen. And the **harder red absorption** (0.90/m against
0.60) is what moves the near shallows, which no sky or body value reaches: the
sand under a metre of water is red, and only the water in front of it can take
that out.

**The value shipped is the last row's sky and absorption with the body at
0/0.12/0.28**, which is sRGB 0, 97, 143 unmapped: the photo's middle band, and
darker than the brightest row above, because the owner asked for darker and
because `deep_color` is also the colour the underwater view saturates to.
Tenebris's is 0.02/0.10/0.22, which is 38, 89, 130 on screen; ours is the same
lightness with the red taken out, which is the direction every measurement
above points.

## Third round: the night side, and under the surface

Two more reports after the colour landed: the sea still reflects too much light
on the night side, and the underwater should be a darker blue.

### The night reflection is one colour; the sky it mirrors is not

The `nightshore` frame, measured across the sky alone, top row of the frame:

| where | sRGB |
| --- | --- |
| away from the sun (left) | 20, 36, 45 |
| middle | 40, 66, 84 |
| toward the sun (right) | 58, 90, 114 |
| horizon, toward the sun | 79, 116, 138 |

A three-to-one range in one frame, because this engine's night sky is lit by
its own upper atmosphere: with the shell at 1.2 R, a sample high in it sees the
sun over the limb up to about 146 degrees from the sub-solar point, so the
night side is a twilight that fades with distance from the terminator and
with the direction looked in. The water reflects `night_sky_color` at every
point and in every direction, 0.035/0.070/0.100 linear, which is about
52, 75, 90 on screen: right in the middle of that range, so it is brighter
than the sky on the side away from the sun and under it toward the sun. The
owner's own night screenshot was under a black sky, which is the far end of
the same scale, where the constant is simply a glowing sheet.

**Decision: the night reflection follows the reflected ray, by the sky's own
rule.** The sky shader lights an atmosphere sample when the sun's ray from it
clears the planet (`sun_visibility`), and the water can ask the same question
of one point: where the reflected ray leaves the atmosphere. Lit, the
reflection is `night_sky_color` scaled by how much the ray faces the sun,
which is where the twilight is; unlit, it is nothing, and the sea under a
black sky is only its own body under the ambient floor. One extra number in
the uniform (the atmosphere radius, in the spare lane of `night_sky`), one
function, no second sky model: the test the sky already applies, applied to
one more ray.

### Underwater is the deep colour at any depth, day or night

Seen from below, and in the composite's murk, the far water is `deep_color`
exactly: at half a metre under and at eight metres, at noon and at midnight.
Two things are wrong with that and both are the same omission. The seabed is
already darkened by the water above it (`hex_terrain.wgsl` attenuates by
`absorption * water_depth`), so a diver sees a floor that darkens with depth
under a murk that does not, and the murk wins the frame. And at night the
sea's body is lit by the ambient floor while the murk stays at full daylight
colour: a glowing blue room under a dark sky.

**Decision: the murk is the deep colour attenuated by the eye's own depth and
by the same day/night level the surface uses.** `deep * exp(-absorption *
eye_depth) * lit`, in one function called from both the cap seen from below
and the composite, so the two cannot disagree. At the shipped values that is,
by depth, unmapped:

| eye depth | murk (linear) | sRGB |
| ---: | --- | --- |
| 0 m | 0, 0.120, 0.280 | 0, 97, 143 |
| 2 m | 0, 0.073, 0.239 | 0, 76, 133 |
| 4 m | 0, 0.044, 0.204 | 0, 60, 123 |
| 8 m | 0, 0.016, 0.148 | 0, 36, 106 |

which is a darker blue the deeper the dive, from the same knob the surface
reads, and nothing new to author. The surface itself does not move: the eye
depth is zero for every pixel seen from above.

## What "shiny" turned out to mean

Nothing in the frame is clipping: the brightest sea pixel is 174 of 255 and not
one pixel passes 200. There is no highlight to turn down. The complaint is
about a large area sitting at a flat desaturated mid-value, and the cause is
that the area is a sand flat under a film of water, seen through a tone curve
that takes the remaining saturation out. **A look complaint names a symptom,
and the first thing to measure is whether the term you are about to tune has
any authority over the pixels in question.**
