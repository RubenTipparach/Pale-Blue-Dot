# Model evidence

All geometry and viewport work uses Blender MCP with Blender 4.4.3. No game
screenshots are taken by Codex; those remain assigned to Claude.

## Kestrel

The exported model has 50 named objects, 37 meshes, 1,116 triangles and 562 UV
charts. Generated material charts share 114 declared tiles in a 160 by 224
atlas. It uses 16 px/m, two-pixel extruded gutters and 14 actual colors.
Padded tiles occupy 92.44 percent of the sheet. RGBA texels occupy
143,360 bytes before GPU allocation overhead (no mip chain).

| Artifact | Bytes |
| --- | ---: |
| kestrel.blend | 89,688 |
| kestrel.glb | 108,148 |
| atlas.png | 2,579 |
| atlas.json | 108,119 |

Blender viewport inspection covered hover and forward nacelles, deflected
control strips, the tapered body/canopy, gear and palette. The first viewport
exposed a gap between the rear fuselage and configured tail; the rear body was
raised to meet the existing attachment rather than moving the physical tail.
The saved `.blend` was reopened through Blender's library loader: all 50 objects
and 37 UV-bearing meshes were present. The review pose was not saved over the
neutral source/export.

Owner/Claude approved geometry but rejected the first flat-color strips. The
revised atlas composes image-generated source swatches and unique pixel overlays.
It has deliberate panel seams, rivet pairs, anti-slip walkways,
coral edge trim, canopy frame/glints, K-1 side stencils, hazard bands, intake
grilles and rotor tip stripes. Material ramps contain no directional light.
A direct old/new GLB comparison found identical triangles, named hierarchy,
pivots and rigid transforms. The geometry helper/build functions for all three
craft also match their pre-paint source exactly.

Nearest-upscaled PNG and neutral-light Blender render were inspected, in
addition to the material viewport. Review images are local ignored files under
`output/vehicles-improve/` (`kestrel-atlas-nearest.png` and
`kestrel-blender-render.png`), not in-game captures. Viewport/render illumination
accounts for the facets seen in review. Blender exports Closest as NEAREST /
NEAREST_MIPMAP_NEAREST; the app uses one mip level and explicit nearest sampling.

Fourteen original material images were generated through the built-in image
creation tool. The first craft commit includes the six Kestrel swatches; boat
swatches accompany their craft commits. Exact prompts and normalization parameters are in
`assets/models/vehicles/swatches/manifest.json`; originals and 32-square tiles
are committed alongside it as each craft lands. Repeated, offset, nearest previews were inspected
in `output/vehicles-improve/swatch-wraps.png`. The first canopy output had
unwanted reflections and was regenerated with plain opaque panes. The first
white/coral/grille images contained alpha; normalization composites this over
the recorded base shade before palette quantization. Opposite tile edges are
made equal by the recorded one-row/column copy, without blur or dithering.
All fourteen normalized PNGs rebuilt byte-for-byte from their originals.
Re-running Blender authoring also reproduced the Kestrel atlas PNG and manifest
byte-for-byte. Original generated PNGs total 13,184,978 bytes.
The base material imagery in each atlas comes from these normalized PNGs;
unique identifying/safety marks are separate deterministic pixel overlays.

Independent GLB measurements found maximum UV edge-length errors of 0.0000188
pixels (Kestrel), 0.0000201 (Tern) and 0.0000190 (Loon), against the 0.003-pixel
artifact-test tolerance. Both boat hulls match the core-exported loft triangles.
Tern sail projected area measures 9.39999916 m2; Loon blade 0.1099999999 m2.

The first app release run passed 145 tests but exposed a too-strict angular
comparison in the new animation test: identical f32 90-degree quaternions can
produce 0.00069 radians through acos(dot). The test now compares quaternion
components with 1e-6 tolerance. The model motion itself was unchanged. Final
release results are recorded below after the required rerun.
Blender appearance checks do not establish game lighting, seat clipping,
distance readability, subjective handling or owner art approval. Startup/frame
timing, upload bytes and actual GPU allocation were not measured.
