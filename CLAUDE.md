# Pale Blue Dot contributor instructions

These instructions adapt the useful engineering rules from Tenebris and
swarm-demo. The current user's Bevy, Avian, GPU geometry, and assisted-flight
requirements take precedence over either upstream's historical rules.
See [source-migration.md](docs/source-migration.md) for pinned provenance and
[engine-architecture.md](docs/engine-architecture.md) for the target design.

**This file is the only copy of these rules.** `AGENTS.md` is a pointer to it
and must stay one: an agent that looks for `AGENTS.md` finds its way here, and
there is nothing in it to fall out of step. Write a rule here, never there, and
never restore a second copy "for the other tool" - two files that have to agree
are two files that will not, and the one that is wrong is always the one nobody
is reading. If some tool insists on generating `AGENTS.md`, let it own that file
outright and leave the pointer to this one at the top of what it writes.

## Architecture and authority

- Work in this repository's Rust workspace. `.reference/` contains read-only
  source snapshots, not runtime dependencies. Copy any required source or
  asset into this repository with provenance; never include files from the
  reference checkout in a build.
- `pbd-core` owns engine-independent rules, stable IDs, generation inputs,
  coordinate conversion, and flight policy. `pbd-app` adapts these rules to
  Bevy ECS and Avian. Rendering and platform APIs never become core dependencies.
- ECS entities represent bodies, ships, actors, and active chunks. Individual
  terrain voxels, face records, and cosmetic particles live in packed arrays
  or GPU buffers. Query only the components a system needs.
- Authoritative terrain and edits remain on the CPU. GPU geometry, visual
  light propagation, and particles are derived state. Do not require GPU
  readback for collision, mining, persistence, or multiplayer decisions.
- Use one authoritative implementation of a gameplay rule across single
  player and future multiplayer. Search before adding parallel functionality.
  If Rust/WGSL layouts or algorithms must agree, validate actual artifacts and
  their interface rather than two hand-written copies of an expected value.

## Non-negotiable world and flight invariants

- Planets, moons, and stations can use analytic on-rails orbits. Ships use
  local gravity falloff, Newtonian motion, configurable linear/angular limits,
  and inertial/rotational dampeners. Do not restore rocket patched conics,
  maneuver nodes, transfer-window requirements, or n-body ship simulation.
- Store system coordinates and orbital time as `f64`; render and simulate
  collision in bounded local `f32` frames. Subtract the origin before casting.
  Keep frame identity explicit in poses, velocities, persistence, and uploads.
- Free rotation is quaternion-backed. Rebase poses, interpolation history,
  physics state, and frame-relative velocities together at a tick boundary.
  Station passengers simulate in the station's local frame.
- Start the desktop explorer in first-person walking mode on dry land. Player
  mouse look uses the current frame's raw displacement, with no camera easing,
  smoothing, interpolation delay, or dependence on ship angular response.
  Physical ship limits still apply independently. Terrain textures use nearest
  point sampling; do not replace it with linear filtering.
- Exiting a ship changes its occupancy, never its existence. Persist its ID,
  pose/frame, inventory, damage, and ownership so it remains boardable.
- A spherical hex world includes twelve pentagons. Use explicit 5/6 degree
  neighbor tables and shared edge ownership; do not fake a sphere by wrapping
  one flat axial grid. Topology/generator versions are part of saved world IDs.
- Airless bodies have no atmospheric sky, weather, or clouds. Life and biome
  rosters are explicit planet data; avoid accidental species/texture fallback
  across planets. Airless barren, frozen, and asteroid catalog worlds are lifeless.
- Water, atmosphere, terrain lighting, and their cameras use the same declared
  body-local frame. Test an offset planet as well as the origin planet.
- Every accepted world mutation enters the durable transaction path immediately.
  A queued write alone is not a durable save; acknowledge commitment only after
  the storage backend succeeds. Do not rely on save-on-exit or an autosave timer.

## Implementation and verification

- Keep modules focused, public contracts documented, tunable values in validated
  data with units, and one source for defaults. Missing values use explicit
  optional overrides; zero is a valid value, not an implicit fallback sentinel.
- Use readable WGSL files, explicit Rust/WGSL alignment, bounds checks, versioned
  chunk jobs, and finite-value guards. Publish complete geometry generations;
  handle capacity overflow without drawing partial data or writing out of bounds.
- Authoritative iteration/order must be stable. A hash map used only for keyed
  lookup is acceptable; never let randomized iteration order choose saved IDs,
  simulation outcomes, or replay hashes. GPU atomic ordering is not deterministic.
- Keep pixel-art sources as committed PNG assets with tile manifests. Follow
  the atlas dimensions and filtering rules in the art specification; generated
  imagery still needs seam, palette, transparency, and in-engine checks.
- Run `cargo fmt --all -- --check`, relevant tests, and appropriate Clippy checks.
  Validate WGSL and its actual pipelines when shader interfaces change. A parser
  check does not establish render-graph correctness or visual parity.
- Measure performance in reproducible release scenes, reporting hardware,
  resolution, scene seed, warm-up, percentiles, upload bytes, and memory. Do not
  describe an unmeasured design as a speedup. Keep a baseline binary for comparisons.
- Clearly distinguish implemented behavior, validated behavior, and proposed
  work in documentation. Visual tests supplement unit tests; unfinished visual
  verification is a stated limitation, not an inherited approval workflow.
