# Vehicle source art

These are original low-poly meshes authored in Blender 4.4, based on
`docs/mockups/vehicles.html` and the current vehicle configuration. The outer
boat surfaces come from the core hull loft. Render assets never supply physics.

## Regeneration

From the repository root, export current authoritative inputs:

```powershell
cargo run -p pbd-core --example vehicle_art --release --offline
```

In Blender 4.4's Python console, run the authoring script with the absolute path
to this checkout (the same code is used through the Blender MCP):

```python
from pathlib import Path
p = Path(r'C:\Users\santi\repos\pbd-codex-vehicles\assets\models\vehicles\author.py')
ns = {'__file__': str(p), '__name__': 'pbd_vehicle_author'}
exec(compile(p.read_text(), str(p), 'exec'), ns)
ns['build']('kestrel')
```

The script owns only the `PBD_AUTHOR` scene. It creates modeled meshes and UVs
in Blender, packs the atlas, and writes the craft's `.blend`, `.glb`, `atlas.png`
and `atlas.json`. The blend contains the scene and its dependencies, including
the packed texture; it can be edited independently of the script. Object names,
parents, pivots and unit scales are part of the export contract. Geometry is
reproducible; internal Blender datablock names/file bytes can vary by session.

After manual source edits, export the active craft scene as Y-up GLB with normals
and UV0, no animation clips or applied runtime scaling, and the existing Closest
image material. Keep the source PNG and manifest synchronized with UV edits.
Generated material sources live in `swatches/generated/`, with exact prompts
and palettes in `swatches/manifest.json`. The normalized 32-square PNGs represent
2 by 2 metres. To rebuild them with Python + Pillow:

```powershell
python assets/models/vehicles/normalize_swatches.py
```

This uses centre-nearest sampling, composites any generated alpha over the
recorded base shade, quantizes to the material ramp without dithering, then
copies the first column/row to the opposite edges for a continuous wrap. The
script writes a repeated/offset nearest preview under `output/vehicles-improve`.
The Blender authoring script reads these committed PNGs directly; it adds only
unique markings (identification, safety bands, boot-top, glints, lashings and
paddle band) and small fasteners on isolated calm patches. Generated seams,
rivets, grain, grilles and cloth remain the actual base imagery.

Inspect a Blender viewport, neutral-light render and nearest-upscaled atlas, then run:

```powershell
cargo fmt --all -- --check
cargo test -p pbd-app --release --offline
cargo test -p pbd-core --release --offline
cargo clippy --all-targets --offline -- -D warnings
```

Changing geometric config requires re-export from Blender. App artifact tests
and loading reject mismatched geometry; the runtime does not stretch models.
Configuration remains authoritative for physical behavior and animation state.

## Pixel art contract

All surfaces use **16 pixels per metre**. Each craft has one tightly packed RGBA
PNG, 12-24 actual colors in small material ramps, isometric face charts and two
texels of extruded edge padding. Construction seams, fasteners, wear, cloth and
wood details use deliberate pixel clusters; no random noise. Identical painted
tiles are reused explicitly, without rescaling charts.
Charts have integer pixel origins and enclosing bounds; no chart is rescaled
to fit. Exact physical dimensions can leave fractional polygon endpoints within
those bounds. Artwork cluster edges stay on the pixel grid. There is no baked
sun, ambient occlusion, photographic noise or gradient shading in albedo.

`atlas.json` identifies every region's chart users (object, face, material), interior
size, padded rectangle and UV origin. Coordinates are in pixels, bottom-left
as in Blender. glTF V is flipped. The app tests actual triangle edge lengths in
metres against UV edge lengths in pixels, as well as padding/nonoverlap/palette.

Blender uses Closest. Its exporter writes NEAREST magnification and
NEAREST_MIPMAP_NEAREST minification; the app creates exactly one mip level with
an explicit nearest sampler. There is no linear filtering or cross-island mip
bleed. Distant aliasing still requires game review.

## Loading

`pbd-app::vehicles::model` uses the cached `gltf` parser/accessor utilities to
convert the rigid GLB subset into Bevy Mesh, Image and StandardMaterial assets.
It preserves the node hierarchy and caches mesh/material/image handles per craft.
It supports named unit-scale nodes, indexed triangles with normals/UV0, one
embedded PNG, opaque albedo materials and double-sided surfaces. Unsupported
data is rejected. Bevy's `bevy_gltf` feature stays disabled. There is no runtime
dependency on Blender, Python, external URLs or `.reference`.

The exporter embeds the same PNG bytes committed as source art. Texture storage
is width times height times four bytes before allocation overhead; no timing or
frame-rate improvement is claimed. CPU parsing/decode happens once per craft.

## Models currently registered

| Craft | Source/export | Appearance |
| --- | --- | --- |
| Kestrel | `kestrel/kestrel.blend`, `kestrel/kestrel.glb` | White/coral tapered tiltrotor, opaque navy canopy, separate controls/nacelles/rotors |

Game screenshots and in-game appearance review are assigned to Claude. Blender
viewport review and artifact tests do not establish in-game lighting, seat-view
clipping, distant readability or owner art approval.
