# Tasks

## 1. Make the surface pass planet-local
- [ ] Hand the shader `camera_world - body_world`, and build clip from
      `clip_from_world * world_from_body`, so vertex positions stay body-local.
- [ ] Move the Rust-side foliage cutoff onto the same body-local altitude.
- [ ] The same convention for the sky pass and, when it is bound, the water
      pass. One declared frame, not one per pass.

## 2. Prove it with an offset body
- [ ] Render a body at the origin and the same body offset, and require the two
      pictures to match. This is the test `CLAUDE.md` already asks for and
      nothing performs, and it is what makes the rule enforceable rather than
      aspirational.

## 3. Per-body rendering data
- [ ] Add `serde` (derive) and `ron`, and a small Bevy `AssetLoader`. This is the
      project's first runtime data loading of any kind: there is no
      `assets/config`, no serde, and no custom loader today.
- [ ] One record per body carrying the atmosphere, surface, water and cloud
      values listed in `design.md`, as a RON asset per body.
- [ ] Per-body override with a global default, using an explicit optional rather
      than a zero sentinel - Tenebris uses zero and this project's own rules
      reject that, since a body that wants a rim intensity of zero must be able
      to say so.
- [ ] Land this with `preview-scale-and-shader-parity`'s task 2, which lifts the
      terrain shader's literals into the same uniform. They are one job seen
      from two sides.

## 4. One atmosphere predicate
- [ ] `has_atmosphere(body)` answered once; the sky pass, the distance fog term
      and the cloud shell all read it.
- [ ] A per-body off switch that strips sky scattering and distance fog and
      leaves the rest running, because it is the direct test for whether the
      atmosphere is tinting the water.
