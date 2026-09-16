# Proposal: what the renderer needs before a second body exists

## Why

The surveys so far have covered the hex size, the gravity model, the shader
terms and the LOD tiering. What is left from Tenebris is one theme rather than
a list: **the renderer here assumes there is exactly one body, it sits at the
world origin, and its look is compiled in.** Every item below is that assumption
showing up somewhere different.

This is worth writing down before a second body exists, because each one is
cheap now and a bug hunt later.

## The confirmed latent bug: the camera is not planet-local

`planet.rs` sends the WORLD camera to the surface shader, and both sides then
treat the world origin as the planet centre:

```rust
let camera_position = view.world_from_view.translation();
if camera_position.length() - PLANET_RADIUS > FOLIAGE_DRAW_CUTOFF_ALTITUDE { ... }
```
```wgsl
let altitude = max(length(params.camera.xyz) - params.settings.x, 0.);
```

Correct today only because `PlanetPlugin` puts the body at the origin. Offset
the body and `altitude` becomes the distance from the world origin, so the fog
gate `exp(-altitude/1050)` and the rim gate `1 - air` both saturate, and the
Rust-side foliage cutoff picks the wrong vertex budget. The terrain would render
with its distance haze and limb rim stuck at their far values everywhere.

Tenebris has the scar for exactly this and its `hex_fs_light[3].xyz` is
documented as *"camera pos in PLANET-LOCAL space (relative to centre)"* for that
reason. Its CLAUDE.md records what the same mistake did to Sequoia's water: fog
distance read ~20 km for every fragment so the sheet saturated to the sky
colour, and Fresnel, specular and Snell's-window angles all resolved against a
garbage view vector. It was mis-diagnosed and "fixed" by turning the atmosphere
off before the real cause was found.

Our `CLAUDE.md` already carries the one-line rule ("Water, atmosphere, terrain
lighting, and their cameras use the same declared body-local frame. Test an
offset planet as well as the origin planet"). What it does not carry is that the
code currently violates it, invisibly, because there is only one body and it is
at zero.

## Per-body look is data there and constants here

Tenebris drives a body's entire appearance from YAML, one section per body or
tileset, with omitted fields falling back to a global default:

| File | What it carries per body |
| --- | --- |
| `atmosphere.yaml` | on/off toggle, outer and lower shell radii, Rayleigh and Mie scales, sun intensity, scale height, Mie asymmetry, **`wavelengths` - the sky hue**, sunset strength/tint/glow/horizon, fog colour |
| `lod.yaml` | water colour and full-depth, sky-reflection horizon and zenith tones, rim colour, fog colour and scale, distant shading floor/ceiling/range, distant rim colour/power/intensity/**floor** |
| `clouds.yaml` | colour, density, the altitude the cloud shell wraps at |

Sequoia is the proof that this is not decoration: it is a brown-sky world made
by `rayleigh_scale: 2.1` and `wavelengths: [18.0, 10.0, 4.5]` against Tenebris's
blue `[5.6, 9.5, 19.6]`, with no shader edit between them.

Our sky is five `vec4`s of Rust constants for one planet, and the surface shader
carries its palette as numeric literals. A second body cannot look different
without editing source. This is the same finding as
`preview-scale-and-shader-parity`'s "there is no second planet", arrived at from
the atmosphere side rather than the terrain side, and the two should land
together.

## One predicate for "has air"

Tenebris routes sky scattering, distance fog, clouds AND precipitation through a
single `body_has_atmosphere(body)` (`has_water && atmo_enabled`), specifically so
they cannot drift apart. Our `CLAUDE.md` states the rule - airless bodies have no
sky, weather or clouds - but names no predicate, so today it is four places that
happen to agree.

## The toggle is a diagnostic, not just a setting

`atmosphere.yaml`'s `enabled: 0` strips sky scattering and distance fog and
leaves everything else running. Its comment says what it is for: it is the direct
test for whether the atmosphere is tinting the water, because the sea keeps its
own colour with the air off. A per-body kill switch that isolates one shading
contribution is worth having on its own terms.

## Already covered elsewhere, not repeated here

- The sphere-of-influence handover: Tenebris's `gravity_at` picking the
  strongest-pull body IS its SOI tracker, handing gravity, surface basis and the
  atmosphere shell to a moon without a teleport. That is `gravity-model`.
- The distant tier: `hexagon-lod`.
- Terrain shader terms (rim floor, ambient floor, torch light, underwater
  absorption, Bayer cutout): `preview-scale-and-shader-parity`.
- Baked light propagation: `voxel-engine-foundation`.
- Hex and cell size: the gold-standard rule in `CLAUDE.md`.

## Non-goals

- Weather. Rain and snow are real systems in Tenebris and are not planetary
  rendering; they belong to their own change if they are ever wanted.
- Authoring a second body. This change is what a second body would need, not the
  body.
