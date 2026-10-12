# Tasks

## 1. The knobs, in data

- [x] `assets/config/scatter.ron` and a `ScatterSettings` in `config.rs`
      mirroring `tenebris-core/src/scatter.rs` field for field for the kinds
      this change ships: `grass_chance`, `grass_blades`, `grass_height_m`,
      `grass_blade_w_m`, `grass_segments`, `grass_base_shade`,
      `flower_chance`, `flower_height_m`, `rock_chance`, `rock_size_m`,
      `bush_chance`, `bush_size_m`, plus this project's own `clutter_radius_m`
      and `clutter_fade_m`. Shipped values are the reference's shipped values.
- [x] Validation on the same terms as `WaterSettings`: finite, in range, and a
      test that parses the real shipped file rather than a literal, since a
      config that overrides the defaults and a test that reads the defaults is
      the drift this repository has a rule about.

## 2. The eligibility decision, in the compute pass

- [x] `args` from `array<DrawArgs,3>` to `,4`, the fourth cleared to
      `vertex_count = 216`, `first_vertex = 258`. The pinned count in
      `planet_visibility_tests.rs` moves in the same commit.
- [x] A `clutter` index list bound beside `visible`, `foliage` and `water`, one
      `u32` per cell, with the same capacity check at the top of
      `compact_visible`.
- [x] `has_clutter(cell, center)`: finest level only, both owners fine, material
      2, 3 or 7, inside `clutter_radius_m`, and a frustum test at a blade's own
      bound rather than a tree's 15 m.
- [x] `planet.rs`: the fourth `draw_indirect` at offset 48 on the clutter bind
      group.

## 3. The blade, in the vertex shader

- [x] The `vertex >= 258u` branch: blade and vertex-within-blade from the index,
      the reference's seven hash decisions over the cell id, and the row
      function (curve on `f*f`, taper on `1 - 0.55*f`, UV row, base-to-tip
      shade) ported as written.
- [x] Blades past this cell's rolled count collapse to a degenerate triangle,
      the way a tree part a cell does not carry already does.
- [x] Winding chosen per vertex against the camera, so one pipeline gives a
      two-sided blade at half the vertices.
- [x] The normal is the surface up, as the reference's `push_tri_2side` has it,
      so a blade shades like the cap it stands on.
- [x] The atlas slice: the cell's own ground column, a narrow vertical crop per
      blade, through the existing `pixel_tile` nearest sampling.
- [x] The edge fade on the blade height.
- [x] Flowers, rocks and bushes in the same draw once grass is right: a flower
      is a blade plus a four-triangle head, and a rock and a bush are hex prisms
      the tree branch already knows how to build.

## 4. Prove it

- [x] A test that the eligibility rule is the rule, not a snapshot of one hash:
      seeds either side of each clause, the way the foliage fixtures were
      rewritten when the record repacked.
- [x] A test that the vertex budget and `first_vertex` agree with the branch's
      own arithmetic, so the shader and the `DrawArgs` cannot drift.
- [x] `--view meadow` before and after, and a close crop at 5 m where a blade is
      26 pixels wide.
- [x] The frame cost at the meadow preset in a release scene, reported against
      the same scene with `clutter_radius_m: 0`. The predicted 0.18 M vertices at
      60 m is arithmetic; whether it shows is a measurement.
- [x] `docs/tenebris-comparison.md`: a section measuring our sward against the
      reference's own frame at the same tile size, camera height and crop.

## 5. Held, and named so the gap is visible

- [ ] The eight biome specials: cactus, fern, reed, dead shrub, kelp, seaweed,
      vine. Each is its own vertex budget, so each is its own draw or a shared
      upper bound the common case pays for - a decision worth taking on its own
      once grass has a measured cost.
- [ ] Sway. It needs a wind direction in the weather field, which the clouds
      should read too so that it is one fact in one place.
- [ ] Gathering. The eligibility rule is a pure function of the cell, so a
      taken-bit can be added beside it the day there is an inventory to put a
      flint pebble in.
