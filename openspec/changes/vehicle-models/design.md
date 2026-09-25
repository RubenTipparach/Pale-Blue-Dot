# Design

## Context

See proposal.md for scope. Read before this design: `CLAUDE.md`, the original
`vehicles` proposal/design/tasks/deltas, the `vehicles-improve` findings and
implementation, the main vehicle spec, `docs/mockups/vehicles.html`, the art
direction in `docs/game-design.md`, `draw.rs`, the core vehicle/hull/foil code,
and `assets/config/vehicles.ron`.

Observed source state:

- The Kestrel has box fuselage/canopy/tail, cylindrical nacelles and box blades.
  The prototype has a tapered ten-sided fuselage, pointed nose, shaped canopy,
  round gear and coral nacelles. There are no animated control-surface objects
  in today's renderer, although physics already computes their deflections.
- Boat hulls use the core loft (28 stations, 10/8 transverse divisions), but
  the Loon has no inner lining, gunwales, seats or thwarts. The Tern has a filled
  deck and simple mast/boom/keel/rudder. Its sail is one flat triangle.
- No vehicle albedo images or authored UVs exist. The current renderer creates
  meshes/materials afresh for each craft. Its corrected wing/paddle tests
  establish an important contract which exported-model tests must preserve.
- The prototypes are visual references, not current physical dimensions:
  Kestrel rotor radius is now 2.0 m, hub is (+/-5.3, 0.62, 0) m and the rotor
  child is (0, 0.8, 0) m. Tern head height is 7.0 m above its mast step, not
  the prototype's 7.2 m. Its triangular 3.3 by 5.7 m sail currently draws
  9.405 m2 against a configured 9.4 m2. Loon blade area is 0.11 m2.
- Wing panels each have area 8 m2, span 5 m, chord 1.6 m, incidence 3 degrees
  and dihedral 4 degrees. Tern hull is 6.2 by 2.3 m; Loon is 5 by 0.92 m.
  These are config-derived measurements, not targets to retune for appearance.
- Blender MCP reports Blender 4.4.3, addon 1.7, an unmodified startup scene and
  the glTF exporter. Bevy 0.18.1 uses default features off; its glTF plugin is
  not enabled. Cached `gltf` 1.4.1 provides parsing/accessor utilities separately.

Handling implementation is parked with its outstanding game review listed in
`../vehicles-improve/review-handoff.md`. The owner reserves every further game
capture for Claude. Blender viewport checks are performed here.

## Goals / Non-Goals

**Goals:** recognizable modeled silhouettes, editable sources, consistent pixel
scale, working moving parts and artifact tests that detect stale geometry.
Preserve the core-generated hull envelope and existing physical behavior.

**Non-goals:** flight tuning, collision from render meshes, skeletal animation,
generic arbitrary glTF support, new save formats, orbit mechanics or camera
smoothing. No runtime scaling/morphing to accommodate changed configuration.

## Decisions

### Fixed exports generated from authoritative inputs

A core example reads and validates the shipped RON with the existing types and
writes JSON authoring inputs, including `WingSpec::panels()` and actual
`Hull::loft()`/deck vertices. Blender consumes these inputs. This avoids copying
the hull equations or canonical defaults into Python. JSON serialization is a
development dependency only; core runtime remains engine-independent.

Author in metres with game axes +X right, +Y up, +Z aft. The Blender mapping is
`(x, -z, y)`; export Y-up glTF so game coordinates and hierarchy are recovered.
Apply object scale. Rest transformations and mesh vertices, not a handwritten
dimension manifest alone, are checked after export.

The owner's explicit decision is **require regeneration**: changing geometric
config requires rerunning the input export, Blender authoring and GLB export.
Artifact validation checks loaded config against actual parts/pivots/dimensions
and fails with a regeneration diagnostic. Negative tests mutate dimensions,
axes and pivot locations to prove stale exports fail. Runtime performs no
reshaping. Pure force/mass changes need no cosmetic rescaling.

Hull envelopes come directly from the core loft, with added inner surfaces,
rim thickness and fittings. The Loon's `lateral` foil describes its hull's
effective hydrodynamic lateral plane, not a separate fin to invent visually;
the skeg is a modeled appendage. The Kestrel has no configured fuselage length:
its cosmetic envelope follows the prototype and configured attachment/eye
points. Do not introduce a parallel physical fuselage dimension.

### Art and UV contract

Use **16 pixels per metre on every surface of every craft**, including machinery,
crew and sail. One compact PNG atlas per craft, packed to the smallest useful
rectangle in 16-pixel increments at the same density. Prefer 256-square or
smaller when the painted islands fit; do not waste a 512-square sheet merely
to keep power-of-two dimensions.
Manifests record image dimensions, palette, semantic region/object/face, integer
packing rectangle, UV chart origin and padding. Commit PNGs as source art beside
their `.blend` and `.glb`; embed the same image in the GLB.

Create planar face charts (triangulate nonplanar faces), project isometrically
at exactly 16 px/m, and pack without scaling or overlapping distinct regions.
Identical painted tiles may be explicitly shared by multiple charts; record
every user in the region manifest. Shared tiles retain each chart's isometric
coordinates and density; this is intentional reuse, not accidental overlap.
Use rotation and rectangle packing to reduce waste. Chart
origins and enclosing rectangle edges sit on integer pixel coordinates. Use
at least two texels of extruded edge padding; pixel marks/cluster edges lie on
the integer texture grid. UV edge lengths must equal geometric edge lengths
times 16, within floating-point export tolerance. No stretched tiny islands.

Exact physical dimensions and arbitrary angled faces cannot in general put
every polygon vertex at an integer pixel coordinate while retaining exact
density and zero stretch. Pixel alignment here means integer chart origins,
pixel artwork and padded packing bounds; fractional polygon endpoints are
retained rather than snapping the physics geometry or distorting its UVs.
This interpretation is explicit and testable in the exported artifacts.

Use compact, deliberate panel/seam/fastener clusters and flat material colors.
No random noise, gradients implying light, ambient occlusion or baked sun in
albedo. Kestrel white/coral with dark navy canopy and mechanical details; Tern
coral deck, pale hull/sail, restrained dark fittings; Loon teal hull with pale
paddle and light seats. Flat mesh normals supply the chunky facets. Silhouette
comes from tapered/lofted geometry, not texture shading.

Owner/Claude review on 2026-09-24 approved the saved Kestrel/Tern geometry,
parts and pivots but rejected the first atlases as flat color strips. Preserve
that geometry. Redo painting with 3-5-color material ramps and roughly 12-24
actual colors per craft. Dark seam pixels and light wear/bevel pixels describe
material construction, never directional illumination. Every sufficiently
resolved surface gets deliberate pixel detail; sub-texel thickness faces keep
their physical density rather than being enlarged to fit decorative pixels.

Kestrel: panel seams and rivet rows, wing walkway/anti-slip, coral edge trim,
canopy frame and glint pair, K-1 stencil, rotor hazard chevrons, nacelle grilles,
and blade tip stripes. Tern: coral deck planks, gunwale strip, darker hull
antifouling below the hull-reference waterline (y=0) with a crisp boot-top stripe,
pale cloth seam panels/batten, and tiller wood grain. Loon: teal canvas/plank
clusters and inner rib marks, thwart grain/lashings, and paddle edge band.
Patterns are deterministic construction marks, not stochastic noise.

Owner addendum: material painting starts with image-generation swatches, one
per distinct material (wood may be shared by tiller and thwarts). Generate
flat-lit, top-down, tileable pixel art without perspective, shadows or baked
lighting. Commit original generated PNGs and normalized swatches under
`assets/models/vehicles/swatches/`, with exact prompts, original dimensions,
nearest-neighbour sample mapping, physical repeat size and 3-5-color ramps.
Normalize to 32 by 32 texels for a 2 by 2 metre repeat at 16 px/m, then quantize
to the recorded shared material ramp. Repair wrap-edge continuity in the
normalized tile if necessary; record the deterministic edge operation and
inspect repeated/offset previews. Generated material marks remain the base
of the actual craft atlases. Author.py composes these committed swatches
without an image-model dependency during rebuild. Unique K-1 stencil, hazard
chevrons, boot-top stripe, canopy glint and paddle edge band remain deliberate
pixel overlays. Tests validate source palette, tiling edges and manifest links
alongside the actual exported atlas contract. No procedural substitute for the
generated material imagery is used.


Use a near-neutral pale cloth ramp for the sail, check its front normal and
double-sided material, and inspect an actual Blender render under neutral
lighting as well as the material viewport. Inspect each PNG upscaled with
nearest filtering. Tests count actual PNG colors and check painted variation,
chart padding, sharing declarations, density and packing utilization. Record
the owner's geometric approval separately from outstanding texture/game review.

Blender image nodes use Closest interpolation. GLB samplers and Bevy images use
nearest magnification and minification, with one mip level initially. Padding
and nonoverlap prevent atlas bleed. Distant aliasing remains a visual-review
item; no filtering improvement is claimed without game evidence.

### Geometry and part/pivot table

Positions below are craft-reference coordinates unless marked parent-local.
`F = [chord cross normal, normal, -chord]` is a foil's orientation basis.
The neutral wing/tail/fin planforms include their separate trailing surfaces.
Use a 25 percent chord trailing strip, hinged at local z = chord/4. The fixed
and moving meshes together retain the full configured neutral area and span.

| Craft / exported part | Parent / origin | Driven axes and authority |
| --- | --- | --- |
| All / body, fittings | craft / reference origin | fixed; craft physics pose |
| All / crew | craft / `seat.eye` | occupancy/seat visibility; Tern crew hike along X |
| Kestrel / wing_left, wing_right | craft / each configured panel `at`, basis F | fixed incidence/dihedral |
| Kestrel / flaperon_left, flaperon_right | corresponding wing / (0,0,chord/4) | local X; physics flap +/- aileron deflection |
| Kestrel / tail | craft / `tail.at`, basis F | fixed |
| Kestrel / elevator | tail / (0,0,chord/4) | local X; actual tail deflection |
| Kestrel / fin | craft / `fin.at`, basis F | fixed |
| Kestrel / rudder | fin / (0,0,chord/4) | local X in foil frame; actual fin deflection |
| Kestrel / nacelle_left, nacelle_right | craft / (+/-rotor.at.x, rotor.at.y, rotor.at.z) | X = nacelle angle - pi/2 |
| Kestrel / rotor_left, rotor_right | corresponding nacelle / (0,0.8,0) | local Y; existing throttle-driven spin; radius from config |
| Kestrel / gear fittings | craft / configured gear contact points | fixed, wheel bottom on contact |
| Tern / hull, deck, cockpit | craft / reference origin | core loft envelope with interior detail |
| Tern / mast | craft / `sail.mast` | fixed; height `head_height_m` |
| Tern / keel | craft / `keel.at`, basis F | fixed foil dimensions/axes |
| Tern / rudder, tiller | craft / `rudder.at` | separate objects, both Y = physics tiller angle |
| Tern / boom | craft / mast + Y * boom_height | Y = physics boom angle |
| Tern / sail | boom / local origin | inherits boom; separate mesh |
| Loon / hull, lining, gunwales, seats, thwarts | craft / reference origin | fixed, core loft outer envelope |
| Loon / skeg | craft / `skeg.at`, basis F | fixed configured foil |
| Loon / paddle | craft / blade centre, neutral origin | existing stroke/recovery/rudder transform; occupancy visibility |

The Kestrel control deflections are exposed from the force calculation as plain
core telemetry values; the renderer does not reproduce assist/controller math.
Preserve all existing moving pivots and drive axes. Crew geometry is positioned
relative to the exported eye origin, and is hidden in the occupied seat view.

The sail will be exported, not procedural. A pale, flat low-poly sail follows
the actual boom and has configured area 9.4 m2. With a 5.7 m rise, use foot
`2*area/rise`, about 3.29825 m, just inside the 3.3 m boom. This corrects the
existing 0.005 m2 discrepancy without changing physics or mast height. No cloth
simulation or prototype-only luff deformation is added.

### Loading choice and cost

Choose **load-time conversion with `gltf` (names + utils, default features off)**
inside `pbd-app`, keeping `bevy_gltf` disabled. Read the GLB, preserve its named
node tree and transforms, convert triangle position/normal/UV/index buffers to
Bevy Mesh assets, and decode the embedded PNG through Bevy's image support with
an explicit nearest sampler. Cache converted assets per craft and instantiate
the hierarchy under the existing vehicle entity. Attach existing moving-part
components by exported name. The finished fleet has no primitive fallback.

Support a deliberately narrow, validated export profile: triangle meshes,
unit scale, normals, UV0, one embedded PNG, opaque/double-sided rough materials,
and rigid node hierarchy. Reject unsupported/missing data clearly. Skins,
animation clips, external buffer fetching, tangents/normal maps and arbitrary
material extensions are outside this pipeline.

Alternative considered: enable `bevy_gltf`, load SceneRoot and bind parts after
SceneInstanceReady. It provides broader asynchronous asset support, but expands
this feature-disabled build and introduces asynchronous hierarchy setup for
three small rigid assets. The narrow converter uses the established glTF parser
and accessor reader rather than hand-parsing GLB binary layouts.

Costs: one synchronous parse/PNG decode per model, CPU mesh storage, texture
upload and an entity per named node/primitive; loader maintenance for the stated
subset. RGBA storage costs width times height times four bytes before GPU allocation overhead, with
no mip chain. Record actual file sizes/triangles/atlas occupancy after export.
No startup-time, frame-rate or memory speedup claim is made without measurement.
No model data enters collision, buoyancy, save state or controller decisions.

### Verification against actual artifacts

Read committed GLBs in tests and run the same conversion used by the app.
Validate required names, hierarchy, rest origins/unit scales, moving axes,
configured wing/tail/fin/keel/rudder/skeg dimensions and neutral projected areas,
rotor radii, mast/boom lengths, hull loft vertices, paddle projected area and
crew eye points. Replace procedural-only geometry tests with these checks;
retain paddle motion, translated-frame and immediate-look integration tests.

Inspect actual indexed triangles and UVs, not just declared metadata: finite
attributes, triangle validity, isometric texel density, atlas bounds/padding,
nonoverlapping distinct tile rectangles and declared chart sharing, PNG dimensions/palette and nearest samplers.
Use loaded ECS hierarchies to test moving pivots and state updates. Prove that
altered geometric config fails validation and forces regeneration.

Blender viewport screenshots verify silhouette, palette, materials and moving
assemblies. Save review evidence and measured counts beside the change. Do not
take game screenshots: Claude owns those. Unit/Blender checks cannot establish
in-game lighting, seat clipping, distance readability or owner art approval.

## Risks / Trade-offs

- Export axis/hierarchy mistakes -> inspect GLB transforms and animate real
  loaded parts in tests, including nonzero rest rotations.
- UV packing can waste space on tiny faces -> report occupancy, keep facets
  purposeful and use the same density rather than rescaling islands to fit.
- Geometry config is no longer immediately reflected visually -> fail the
  contract test/load validation and provide a documented regeneration command.
- Narrow converter rejects future complex art -> extend its documented profile
  with artifact tests, or switch to Bevy glTF when broader support is needed.
- Flat sail and opaque canopy are deliberate chunky-style simplifications ->
  record them for art review, without claiming cloth/glass simulation.
- No game appearance evidence in this task -> explicit handoff to Claude;
  leave review tasks open instead of treating tests as visual approval.

## Migration Plan

Finish/park the tested handling commits, then commit only this write-up. First
craft commit supplies the common authoring/export/loading/validation pipeline
and Kestrel assets/animation telemetry. Next commits replace Tern then Loon.
Temporary procedural support remains only for craft not yet converted; remove
it when the final craft lands. Main-spec requirements move only with the code,
assets and passing tests that establish them. No asset download is required.

Before each commit run fmt check, both release package suites offline and
all-target Clippy (also enforce warnings as errors), plus OpenSpec validation.
Build the final release app offline; checking `--help` is allowed, game captures
are not. Record unresolved review separately. Rollback uses the corresponding
craft commit revert; there is no save migration. Never push or leave this branch.
