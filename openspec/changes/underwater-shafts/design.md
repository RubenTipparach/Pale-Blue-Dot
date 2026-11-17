# Design: underwater shafts

## 1. The cause

`shafts()` (`assets/shaders/water.wgsl`, at the commit this change started
on, `66780be5`) marches ten samples along the view ray:

```wgsl
let reach = min(dist, view.under.y);                              // line 518
...
for (var i = 0; i < steps; i++) {
    let u = (f32(i)+jitter)/f32(steps);
    let d = reach*u*u;                                            // line 557
    let stretch = 2.0*reach*max(u, 0.5/f32(steps))/f32(steps);    // line 558
    let q = origin + dir*d;
    ...
    let bright = mix(pow(max(a*b, 0.0), 6.0), SHAFT_MEAN, smoothstep(0.8, 2.5, stretch));
    gathered += bright*light_left(depth)*exp(-extinction*d)*stretch;
}
```

`dist` is the distance to the scene behind the pixel: the composite passes
`light_reach = view_dist` (line 1161, the scene depth) and the sheet seen
from below passes `camera_distance` (line 867). Whenever the scene is nearer
than the shafts' reach (`shaft_reach_m`, 40 m), and in shallow water near a
coast it nearly always is, `reach` is the scene depth, and so are all ten
sample positions `d = reach*u*u` and the pattern fade `stretch`.

The shaft pattern is a field of bright cells about 4.5 m across. A pixel on a
pillar 6 m away samples it at 0.02, 0.14, 0.38 ... 5.4 m; its neighbour on
the seabed 25 m away samples it at 0.06, 0.56, 1.6 ... 22.6 m. The two
pixels are lit by unrelated parts of the pattern, and some of the near
pixel's samples are faded to the mean while the far pixel's are not, so the
shaft's brightness jumps wherever the depth jumps. The jump follows the
scene's outline exactly: the silhouettes and dark wedges the owner saw. It
reads as the fog changing with depth because the shaft is added over the
fogged scene (line 1181) in the colour of the water's light.

The fog itself (`mix(deep, color, exp(-extinction*travel))`, line 1155) and
the shadow cascades are not involved: the shafts read no cascade, and the fog
by depth is the ordinary depth fog.

## 2. The fix

The march is laid over the shafts' own reach, `R = view.under.y`, whatever
the scene's distance, and the scene clips it:

- sample `i` of `n` owns the stretch `[R*(i/n)^2, R*((i+1)/n)^2]` of the
  ray and stands at `d = R*u*u`, `u = (i + 0.5)/n`;
- its weight is the part of that stretch in front of the scene,
  `clamp(dist - lo, 0, hi - lo)`; the march stops at the first stretch that
  starts behind the scene;
- the one stretch the scene cuts is sampled in the middle of its part in
  front, `min(R*u*u, lo + weight/2)`, and fades toward the pattern's mean by
  that part's length. An uncut stretch is unchanged by the `min`, because
  the middle of its length is never nearer than `R*u*u`.

So every pixel samples the pattern at the same points along its ray. Two
neighbouring pixels with very different scene depths share every sample in
front of the nearer one, and differ only by the one cut sample and what lies
between the two depths: the in-scatter changes continuously with the
distance to the scene, and the scene's outline is no longer drawn into it.
The total weight is `min(dist, R)`, as before.

Without the second rule the cut sample of a ray ending on the surface seen
from below stood past the sheet, out of the water, and was dropped: the
first after capture lost most of the light just under the surface.

**Sixteen samples, not ten.** With ten samples laid over 40 m, a ray that
meets the surface 10 m away (looking up at 20 degrees from 3 m down) gets
five samples, against all ten before, and the farther ones are faded to the
pattern's mean. The tall bright column of the before frame was mostly gone
(first after capture, scratch only). The old march's sharp columns near the
surface came from the very compression this change removes. Sixteen
samples over the same reach put eight in front of that 10 m surface. Their
stretches are 0.16 to 4.8 m, against 0.4 to 7.6 m, and the columns come back:
the after frame's shafts are as clear as the before's and now run down to
the seabed. The mean brightness of the frame is within 2.5% of the before
(`after-shallow-sun` against `before-shallow-sun`, rows of 150 px: 72.4/72.3,
53.2/54.2, 46.3/46.1, 42.9/42.4, 38.1/40.8, 48.8/50.6, sRGB 0-255).

The cost is six more samples on each underwater pixel that sees open water
to 40 m. A pixel whose scene is nearer stops at it, as it now does. This
moves frame time under water and is not measured here (a cloud container,
lavapipe; CLAUDE.md). The owner's desktop run is task 5.

## 3. What is not changed

The strength (`shafts`, 0.6), the reach (40 m), the pattern and its sway,
the phase toward the sun, the colour, the marine snow, the murk and the fog,
the surface from below and Snell's window. The falling water's look is not
touched (CLAUDE.md, waterfalls are designed in a mockup first).

## 4. Verification

Before and after `--capture` stills from under the water, the before from
`66780be5`'s `water.wgsl` (the same binary: only the shader differs), in `docs/screenshots/underwater-shafts/` (rendered on lavapipe
in a cloud container with no GPU; frame cost not measured). A source test in
`shader_tests.rs` pins that the march's sample distances are set by the
reach and not the scene's distance.
