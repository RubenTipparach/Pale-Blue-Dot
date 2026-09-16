# Tenebris reference and the Bevy visual prototype

## Reference captured from the actual application

The reference below is a native Tenebris framebuffer capture, not an illustration or a recreated scene.

![Tenebris native orbit reference](screenshots/tenebris-orbit.png)

The first capture attempt used the existing release executable at `C:/Users/santi/repos/tenebris/tenebris-rs/target/release/tenebris-client.exe`, dated 2026-06-27. That executable did not contain the `TENEBRIS_DEV_SHOT` or `TENEBRIS_DEV_LODCAM` hooks and produced no PNG. The attempt was stopped, and the existing debug executable dated 2026-07-20 was used instead. No Tenebris build was performed and no source file in that repository was modified. The source checkout inspected during the capture was at `82b562718b9ec3a5365654d6ec0d776f4b744334`; the exact source revision of the prebuilt executable is unknown.

The successful capture used the native `dev_shot.rs` framebuffer path with these inputs:

| Setting | Value |
| --- | --- |
| Executable | Existing `tenebris-rs/target/debug/tenebris-client.exe` |
| Working directory | `C:/Users/santi/repos/tenebris/tenebris-rs` so its own assets resolve |
| Window launch | Hidden background process |
| World | `pbd-reference-20260915-orbit-debug` |
| Camera | `TENEBRIS_DEV_LODCAM=650`, aimed at the planet by the source hook |
| Capture | `TENEBRIS_DEV_SHOT=<workspace>/output/captures/tenebris-orbit-debug/orbit.png` |
| Save isolation | Child-only `APPDATA=<workspace>/output/captures/tenebris-orbit-debug/isolated-user-data` |
| Stable copy | `orbit_reference.png`, 1280 × 720 |
| Committed reference copy | `docs/screenshots/tenebris-orbit.png` |

The LOD hook disables normal SP autosaving in the inspected source. Save isolation additionally keeps the capture's profile/world store separate from the user's existing Tenebris save data. Only the processes started for these captures were stopped. Logs and isolated data remain under their capture directories for review.

A second capture used `TENEBRIS_DEV_DIVE=-12`, `TENEBRIS_DEV_TIME=0.46` and clear weather in another isolated world/store. The deepest-ocean camera is on the dark side at that global time; the script's “noon” preset is not local noon at every longitude. This is a useful wave-detail/night-water reference, not a daylight color reference.

![Tenebris native near-water reference](screenshots/tenebris-water.png)

The surface capture shows fine, continuous reflective wave ripples over very dark water, with a blocky forest silhouette beyond the curved shoreline. Its low-light result reinforces that the prototype needs a chosen sunlit landing viewpoint for material review, rather than treating a global clock preset as a camera lighting guarantee.

## What the captured reference actually looks like

- Terrain keeps small, high-contrast pixels at the coast: pale sand borders saturated green vegetation and deep blue water. This visible material contrast matters more than continuous physically based roughness.
- The planet is a readable blocky globe. Cloud silhouettes are chunky squared puffs rather than photorealistic volume noise.
- A nearly black space background carries many small white pixel stars, with a small neighboring moon visible to the right.
- The terminator is strong. The night hemisphere is very dark, while the sunward coast and ocean highlights are bright.
- The atmosphere forms a broad, soft pale-blue/white limb. Parts of this capture's limb and water highlights are overexposed; that is observed behavior, not a requirement to reproduce the clipping.

The runtime reference planet in this shot has radius about 300 m according to its log. The new prototype's radius is 4,000 m. Compare silhouette, materials, lighting and atmosphere rather than claiming that the same 650 m camera altitude gives equivalent framing at both scales.

## Integrated sky material

`crates/pbd-app/src/sky.rs` exports `SkyPlugin`, `SkyMaterial`, and shared environment constants. The plugin registers a real Bevy 0.18.1 `MaterialPlugin` and spawns an atmosphere sphere. The application must add `SkyPlugin` to its visual scene; the headless simulation does not acquire rendering implicitly.

| Contract | Current value |
| --- | --- |
| Planet center | Local-world origin |
| Ocean/solid reference radius | 4,000 m |
| Atmosphere shell radius | 4,800 m, 800 m above sea level |
| Cloud layer radius | 4,600 m, 600 m above sea level |
| Exponential density scale height | 176 m; smooth fade over the outer 28% of the shell |
| Sun direction | Normalized `(0.65, 0.75, 0.35)` |
| Shader asset | `assets/shaders/sky_atmosphere.wgsl` |
| Blend/depth | Premultiplied alpha, depth writes off, reverse-Z GreaterEqual |
| Camera contract | Bevy view position and planet center in the same local world frame |

The integrated shader imports Bevy's mesh output and view binding. It takes the default Bevy mesh vertex stage, traces the shell from the current camera, and adapts the standalone Tenebris atmosphere integration to sixteen view/four sun samples. Its fragment math includes soft planetary shadowing, Rayleigh and Mie terms, warm dusk, an altitude-dependent daylight veil that hides stars near the ground, and sparse quantized cloud patches. It renders front faces from orbit and back faces from inside the shell, discarding the other side so the sphere is not blended twice.

The planet renderer owns ocean geometry and its water shader. There is no duplicate transparent sea sphere in `SkyPlugin`. Its water prototype uses ocean-depth coloration, procedural small waves, Fresnel response and sun glints within the planet surface pass. The more detailed standalone `water.wgsl` remains a separate port; this integration does not claim screen-space depth refraction simply because that standalone asset exists.

The new sky is deliberately restrained at the limb, using a thin scale-height density profile over a larger geometric shell. The revised positive terrain elevations are halved before 6 m quantization, with a sampled peak near 426 m; the cloud layer at 600 m and air shell at 800 m clear those observed peaks. Blue scattering ratios, lower Mie strength and reduced radiance target the initial bright-white rim. Terrain edges and pixel colors should stay readable beneath it. Cloud patches are an inexpensive stylized approximation on a spherical layer, not the source's individual cube clouds or a volumetric cloud simulation. The material assumes the planet is at the stated local origin; moving/rebasing the planet requires updating its center uniform and mesh transform together.

## Native prototype capture history

The first release desktop build produced actual `pbd-orbit.png`, `pbd-coast.png` and `pbd-surface.png` screenshots under `output/captures/`. These images were inspected against the Tenebris reference. They demonstrate that GPU column compaction/indirect drawing, the Bevy-composed sky material, textured hex/pentagon surfaces, ocean shading, cosmetic trees and HUD rendered in the application. They used subdivision 7: 163,842 cells and approximately 20 MiB of uploaded column data.

The first orbit view showed a complete textured globe and a blue limb, but mountain tops protruded above the original 4,260 m shell, and the original 4,108 m clouds lay below much of the terrain. Extensive snow/rock materials produced the bright upper hemisphere; it was not all atmospheric glare. The surface view showed textured stepped columns and trees beneath blue sky. The original coast camera was 700 m above local ground and outside most of the dense air, so black space above its horizon was a camera-altitude effect.

The revised source uses subdivision 8: **655,362 columns**, approximately **80 MiB** of column storage, and roughly **19 m** average cell width. Draws above 3,200 m camera altitude use **54 vertices per visible column**; nearer views use **162** to include cosmetic tree cubes. Actual revised orbit, coast, surface, night and pole captures were produced and inspected. They show clearer coastal greens/sand, a shell above the mountain peaks, visible cloud patches and finer surface columns. The original screenshots remain historical evidence for the first build, not measurements of this configuration.

Night-side review found sharply separated diagonal atmosphere bands. These were consistent with too few integration samples and binary shadow rejection, rather than the intended cloud pattern. The integrated WGSL was refined to sixteen midpoint samples and a smooth 36–60 m planetary penumbra; its interfaces and Rust material stayed unchanged. The final native recapture shows a smooth limb with those diagonal wedges removed. All views were rerun after this change. The standalone source-mapped atmosphere shader retains the original eight-view/four-sun march.

## Final native views

![Pale Blue Dot native orbit view](screenshots/pbd-orbit.png)

The orbit view preserves the reference's saturated land, dark blue ocean, pixel stars and small moon. The revised air shell clears the mountains, and blue scattering leaves the green and rocky surface legible. Its wispy cloud patches differ visibly from Tenebris's block-shaped clouds; they are a deliberate inexpensive approximation.

![Pale Blue Dot native coast view](screenshots/pbd-coast.png)

The coast view separates turquoise shallows, pale sand and green inland terraces. The brighter sunlit camera makes the water/material transition easier to assess than the captured Tenebris night-water reference. Water retains warm sun glints, but uses the simpler documented ocean shader rather than the standalone refraction port.

![Pale Blue Dot native surface view](screenshots/pbd-surface.png)

The surface view shows pixel textures, stepped hex columns and block-shaped trees below blue sky and elevated clouds. The approximately 19 m cells remain visibly large; this is a whole-globe surface preview, not the planned metre-scale editable voxel world.

![Pale Blue Dot native night view](screenshots/pbd-night.png)

The night view retains a dark hemisphere and a bright twilight rim, with a warm ocean highlight on the lit side. The final atmosphere has a continuous shadow transition after the sampling/penumbra correction. Separate pole and completed survey-tour captures also rendered the closed globe without missing seams in the reviewed views.

## Acceptance and comparison boundaries

The original five standalone shaders and the two integrated planet pipeline modules (`planet_surface.wgsl`, `planet_visibility.wgsl`) have explicit contracts in the dedicated Naga validation tool. `sky_atmosphere.wgsl` is explicitly deferred by name because it includes Bevy imports and material substitutions; it must be validated through Bevy's shader composer and the running visual application rather than treated as standalone WGSL. Unknown shader names are still errors, so adding a new module cannot silently skip validation.

The seven-module Naga 27.0.3 validation run passed, including the planet pipelines' semantic, binding, entry-point and storage-layout checks. Separately, the final native desktop runs compiled and rendered the Bevy sky material and the full 655,362-column globe without reported GPU pipeline errors in orbit, coast, surface, night, pole and survey-tour views. The final Rust verification run passed 35 tests, and the two scripted circumnavigation checks completed with zero protection events. These checks validate the tested prototype paths; they do not establish full parity with Tenebris or universal hardware performance.

The reviewed coast and surface views show textured grass/sand/stone, stepped hex columns, block trees, blue sky and elevated cloud patches. Orbit/pole views retain a closed globe and surrounding air layer; the ocean uses the documented simplified reflection/glint model. Future render changes should repeat these views and inspect logs for pipeline errors. Measured frame timing with hardware/configuration details is performance evidence; an illustration, HUD counter or shader parser pass cannot substitute for a benchmark.
