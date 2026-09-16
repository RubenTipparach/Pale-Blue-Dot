# Foundation validation

Validated on Windows with Rust 1.96.0. This records the scope actually exercised;
it is not a rendered-game acceptance report or a GPU performance measurement.

| Check | Result |
| --- | --- |
| `cargo test --workspace --locked` | 18 passed: 14 core tests and 4 Bevy/Avian integration tests |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo fmt --all -- --check` | Passed |
| `cargo run -p pbd-app --locked -- 600` | Passed; 600 physics ticks, 10 simulated seconds |
| `cargo run --manifest-path tools/validate_shaders/Cargo.toml --locked` | All 5 WGSL modules passed semantic, binding, entry-point and layout checks |
| `cargo test --manifest-path tools/validate_shaders/Cargo.toml --locked` | 3 validator regression tests passed |
| Shader-validator formatting | Passed |
| `python tools/build_art_catalog.py` | All 14 PNGs and 224 named material slots checked; manifest/gallery generated |
| Offline gallery JavaScript and browser rendering | Script parsed; default page rendered and inspected in headless Microsoft Edge |
| Local Markdown links / `git diff --check` | Passed |

Core coverage includes gravity falloff, diagonal vector limits, dampener
enable/disable, braking after lower-speed mode selection, local-to-world thrust,
negative chunk coordinates, deterministic sampling, high-magnitude f64 frame
conversion, and hierarchical circular orbital state. Integration coverage checks
that Avian integrates acceleration once, colliders stop a moving body, rotation
remains normalized/bounded, and the shared clock follows variable fixed steps,
physics time scale and pause behavior.

The final smoke run printed:

```text
Pale Blue Dot: headless Bevy 0.18.1 / Avian 0.6.1 foundation
ticks=600 simulated_seconds=10.000 rails_bodies=3
ship_local_m=Vec3(93.68827, 4124.373, -0.28498206) speed_m_s=13.452 angular_rad_s=0.075
peak_command_acceleration_m_s2=13.427 safety_clips=0 terrain_checksum=f3da8cbe8d848a45
Renderer, GPU meshing dispatch, spherical topology and gameplay are not connected in this foundation.
```

The generated art originals are 1254×1254, not the requested 1024×1024.
Consequently all 14 are flagged for production pixel-grid normalization.
Hashes and actual dimensions are in
[generated-manifest.json](../assets/tilesets/generated-manifest.json).
Visual inspection confirms distinct palettes and a four-by-four material layout;
strict 16×16 pixel structure and opposite-edge seamlessness are not certified.

Not exercised: backend GPU shader pipelines, indirect drawing, render-graph
ordering, actual terrain streaming/collision, spherical seams, save durability,
station walking, multiplayer, full origin rebasing, or in-game water/atmosphere
appearance. The acceptance work is listed in [shader-port.md](shader-port.md)
and the [game design milestones](game-design.md#17-milestones-and-acceptance-gates).
