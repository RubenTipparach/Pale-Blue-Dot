# Design

## Planet-local is a frame, and it has to be declared

The fix is not "subtract the centre somewhere". It is that the surface pass
needs to say which frame its inputs are in, and every input has to be in it.

Today the vertex path builds positions as `axis * radius`, which is already
body-local, and then multiplies by `params.clip_from_world` as though they were
world positions. That works only because the two frames are the same. Offsetting
the body requires deciding which one the pass works in:

- **Body-local throughout.** Vertex positions stay `axis * radius`, the camera
  arrives as `camera_world - body_world`, and the clip matrix becomes
  `clip_from_world * world_from_body`. Everything the fragment does - `radial`,
  `toward_camera`, `distance_to_camera`, `altitude` - is then consistent, and
  `altitude` stays `length(camera_local) - sea_radius` unchanged. This is the
  convention Tenebris settled on after the Sequoia saga.
- **World throughout.** Every term that currently assumes the origin has to
  subtract the centre explicitly, and the centre becomes a uniform every one of
  them reads. More places to forget.

Body-local is the one to take, and the same convention then covers the water and
atmosphere passes rather than each choosing for itself. The `f32` precision
argument points the same way: a body-local position is bounded by the body
radius, and a world position is not.

The test that would have caught it is the one our `CLAUDE.md` already asks for
and nothing does: render an offset body and compare against the same body at the
origin. Those two pictures must be identical.

## The format: RON assets

**Decided: RON**, loaded through Bevy's asset server with a small custom
`AssetLoader`. This is new infrastructure, not an extension: the project has
**no runtime data loading at all** today. Confirmed by inspection - no
`assets/config` directory, no `serde` in any of the three manifests, no custom
`AssetLoader` implemented anywhere, and the only `assets.load` calls are two
shaders and one texture atlas. Every tunable is a Rust `const`
(`PLANET_RADIUS`, `ELEVATION_STEP`, `SUBDIVISIONS`,
`FOLIAGE_DRAW_CUTOFF_ALTITUDE`, `EYE_HEIGHT`, `ATMOSPHERE_RADIUS`,
`CLOUD_RADIUS`, ...), which sits in open tension with this project's own rule
that tunable values live "in validated data with units".

Why RON over the alternatives:

- **Typed, through serde.** A malformed body definition is a load error rather
  than a silently wrong planet - which is the failure mode that cost Tenebris a
  bug where an unknown decal kind produced a plain white material and the picture
  was misread twice as a texture authored wrong.
- **`Option<T>` says "inherit" directly.** That is the mechanism this design
  already requires and Tenebris's zero sentinel cannot express: a body that wants
  a rim intensity of *zero* must be able to say so, and `None` vs `Some(0.0)`
  distinguishes them with no convention to remember.
- **Hot reload comes free** through the asset server, which is the whole point of
  Tenebris keeping its sky in a file - dial `rayleigh_scale`, save, watch the
  air thicken.
- Nested vectors and colour triples, which this data is mostly made of, read
  better in RON than in TOML.

New dependencies: `serde` (derive) and `ron`. Bevy already pulls serde
transitively, so this adds one small crate and one loader.

## The shape of per-body data

Tenebris's pattern, worth copying exactly:

- One file per subsystem, one section per body or tileset, matched by name.
- A field omitted means "use the global default", so a body overrides only what
  differs.
- **A zero value is the sentinel for "inherit"** for colours and multipliers -
  `[0,0,0]` and `0.0` both mean "use the global".

That last rule is the one to be careful with here, because our own `CLAUDE.md`
takes the opposite position and is right to: *"zero is a valid value, not an
implicit fallback sentinel"*. Tenebris's sentinel is a real trap - a body that
genuinely wants a rim intensity of zero cannot say so. The pattern to take is
the per-body override with a global default; the mechanism should be an explicit
optional (absent means inherit) rather than a magic zero.

## What a body's rendering record would carry

Grouped by the pass that reads it, so a new body is one record rather than
edits in four files:

- **Atmosphere**: present or absent; outer and lower shell radii as multiples of
  the body radius; Rayleigh and Mie scale; sun intensity; scale height; Mie
  asymmetry; the wavelength ratios that set the sky hue; the sunset set; fog
  colour.
- **Surface**: rim colour, power, intensity and night floor; fog colour and
  density; terminator band; ambient and sun tint; the material palette.
- **Water**: deep and shallow colour, absorption, full depth, sky-reflection
  horizon and zenith tones.
- **Clouds**: colour, density, shell altitude.

`sky.rs` already packs five `vec4`s for the pass it feeds, and `PlanetParams`
already carries a `settings` vector. The work is where the values come from, not
how they reach the GPU.

## The one predicate

`has_atmosphere(body)` answered once and read by the sky pass, the distance fog
term, the cloud shell and any future precipitation. Not four conditions that
happen to agree today.
