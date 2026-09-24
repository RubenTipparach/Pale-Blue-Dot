# Design

## Context

Read before planning: `CLAUDE.md`, all four artifact kinds in the original
`vehicles` change, `player/vehicles/spec.md`, the core rigid body, foil,
Kestrel, Tern, Loon, hull and scenario code, and the app's placement,
controls, drawing, camera, HUD and integration tests. See proposal.md for
motivation. This change preserves the core/app boundary and Newtonian
force integration at 60 Hz with four substeps.

### Findings and baseline

These are source measurements, not claims about subjective play quality:

| Area | Existing behavior / measured discrepancy | Consequence |
| --- | --- | --- |
| Shared foil | Lift direction projects the fixed positive normal onto the flow plane; the coefficient changes sign in reverse flow but that direction does not | A foil meeting reverse flow from below can push downward; the force direction is discontinuous near normal incidence |
| Kestrel authority | At zero throttle, `hover*(0.25+throttle)` allows 2250 N m pitch and 1750 N m yaw torque; roll mixing can command one rotor at 12%, or 2880 N | Rotor steering is available without power |
| Kestrel assist | Centred drift correction divides by gravity magnitude | At zero local gravity, even zero drift can produce NaN |
| Kestrel conversion | Rotor hubs are 0.62 m above the centre of mass; forward thrust generates a nose-down moment while hover cyclic authority fades | Successful transition is not covered by existing tests; measure before tuning |
| Tern instruments | Leeway uses ground velocity; HUD discards true-wind side, omits apparent wind and VMG, labels a ratio only as `hull` | A cross-current can appear as extreme leeway; the player lacks sail-trimming information |
| Loon fluid sampling | Lateral plane and skeg share one water sample and wet/dry gate despite being 2 m apart longitudinally | A dry skeg can exert force, or a wet one lose force, during pitching |
| Loon steering visual | Q/E applies a blade force at the stern; drawing follows only the stroke state | A working rudder can look stowed |
| Drawn wing | 16.96 m^2 planform versus configured 16 m^2; drawn incidence/dihedral 0/0 versus configured 3/4 degrees | Shape does not show the foil configuration |
| Drawn paddle | 0.09 m^2 blade versus configured 0.11 m^2 (-18.2%) | Physical and visible blade differ |
| Drawn sail | 9.405 m^2 triangle versus configured 9.4 m^2 | Already agrees within rounding; retain it |
| Placement | Berth search normalizes render-local walker position without subtracting planet centre | New fleets on translated planets are searched for around the wrong direction |
| Water camera timing | Column water-at-eye publication runs in Update, while vehicle camera following and transform propagation run in PostUpdate | It can read the previous frame's eye after boarding or looking; publish after propagation and test the occupied view |
| Capture discoverability | The parser supports `--aboard`, `--seat` and `--render-offset`, but `--help` does not list them | Document those existing options with the instrument work |
| Coverage | Existing vehicle scenarios exercise settling, takeoff, failed conversion, sailing, paddling and persistence; no raw-look, translated fleet, reverse-flow or rudder-visual regression | Passing tests do not establish these contracts |

The measurement-only `vehicle_probe` example produced these baseline
numbers using the protocol in `measurement-protocol.md` (flat sea, no gusts):

| Scenario | Baseline |
| --- | --- |
| Kestrel 3 s full climb from 100 m | 7.1284 m/s climb; 0.4756 degrees tilt |
| Kestrel centred hover, 10 m/s crosswind, 20 s | 1.2084 m/s ground drift; height 100.1543 m |
| Kestrel 5 s hover, then full forward nacelle command for 6.1 s | 85.0492 m/s airspeed; -15.1632 m/s vertical speed; maximum pitch magnitude 9.2186 degrees; minimum height 87.1383 m; wing share 1.0 |
| Kestrel zero gravity, centred assist, one tick | Nonfinite pose/velocity |
| Kestrel zero gravity, assist off, zero power, pitch+yaw for one tick | 0.016002 rad/s angular speed despite no power |
| Tern 60-degree close reach, 11 m/s wind, sheet .3, heading held, 60 s | 3.1754 m/s ground speed; 1.6425 m/s windward VMG; 19.5870 degrees heel |
| Tern initially comoving with 2 m/s cross-current, one tick | 88.5674 degrees reported leeway |
| Loon alternating strokes, 20 s | 1.3517 m/s; 50 strokes/min in trailing 6 s window |
| Loon stern rudder, 1 s after settling, initial 2 m/s forward | Left +6.8091 degrees; right -6.8091 degrees (signs already correct) |
| Tail foil, velocity (0,-2,-20) m/s / (0,-2,+20) m/s | +365.3430 N / -172.0212 N vertical force; reverse flow pushes down |

The conversion reaches wingborne speed but loses 12.86 m while the pilot
holds no pitch input. This does not justify automatic nacelle limiting or
extra thrust: preserving stall and direct pilot control is the chosen
boundary. Add a repeatable conversion regression and a HUD cue that makes
the transition state legible; human assessment of the sink remains open.
The strongest flight fixes are the demonstrated nonfinite state, unpowered
torque and wrong reverse-flow force, rather than speculative tuning.

Baseline verification passed offline: `cargo fmt --all -- --check`,
`cargo test -p pbd-core --release` (166 passed, 6 existing reports ignored),
`cargo test -p pbd-app --release` (130 library and 14 desktop tests passed,
13 existing reports/GPU-specific tests ignored), and
`cargo clippy --all-targets -- -D warnings`. OpenSpec validates all 56
items. Builds used two Cargo jobs; the core probe/suite used
`--target-dir target/vehicle-probe` to avoid the app build lock.
The sea GPU test was also run with output visible: it compared 360 points
on the actual GPU (it did not take its no-adapter skip path).

At the planning commit, the standalone baseline executable is still in
release compilation, after the successful release test build. Its unchanged
library has already been compiled. Visual findings above are geometry/source
measurements; in-engine appearance is not yet claimed verified. Preserve
that executable when its build completes and inspect baseline captures before
visual edits, then record before/after evidence with the visual commit.

Baseline captures subsequently inspected, before any app visual edits:
`output/vehicles-improve/before-{kestrel,tern,loon}.png`, using the retained
baseline executable, `--aboard KIND --frames 180 --time 12`, no saved world.
The scene uses seed `0x5eed2026`, 1440x900, i7-9700F, 32 GiB system RAM,
RTX 3070 8192 MiB, driver 595.97. The first 60 frames are the capture
instrument's warm-up; no performance improvement is claimed (other builds
were running, upload bytes and GPU memory were not measured).

Visible findings: unbacked instrument text is difficult to separate from
rain/water/terrain; the Tern's rig is cut off at the top of its default
chase view; the Loon's default chase eye is inside the shoreline, with
foreground terrain hiding almost the whole canoe. The Kestrel's flat
single wing is visible. Add two concrete view corrections to the visual
phase: target the Tern's rig with enough default stand-off to fit its mast,
and shorten a chase boom immediately when its segment meets terrain.
Use the existing body-local ground query, a small positive eye clearance
and sampling finer than a voxel layer; keep the requested zoom distance so
the view returns when clear. This is camera collision, not craft collision,
and introduces no mouse smoothing. Test a blocked and clear boom and actual
Tern mast projection before claiming the captures improved.

### Spec/code gaps kept separate

The original proposal's status still says no Rust exists, whereas its design
section 11 and checked tasks describe the port. The implemented code is a
core rigid body, not an Avian craft; the design already documents this.
The original change still contains unproven swamping and vehicle-camera
requirements, and a sea alias-filter delta. Per-vertex sea-state maps,
sleeping and craft collisions are not implemented. This change will prove
the occupied-camera case; it will not claim the other open work is done.
Main vehicle specs already claim point-local fluid forces; the Loon sampling
fix closes that gap with a new regression rather than weakening the claim.
Configuration and record validation are also partial (for example, record
loading normalizes a finite but potentially zero quaternion). This pass
does not widen the save/configuration contract; malformed-data hardening
remains separate from the measured handling and view defects addressed here.

## Goals / Non-Goals

Correct observable force/control inconsistencies, give the player useful
instruments, align moving visuals with simulation, and add boundary-case
tests. Prefer physical fixes over increasing thrust, damping or sail area.
No new dependencies, save version, orbital mechanics, engine APIs in core,
camera interpolation, shader changes, or wholesale vehicle art replacement.

## Decisions

1. **Orient foil lift with flow and span.** Use a signed cross product of
   relative flow and foil span, preserving the current forward-flow result
   while handling reverse flow continuously. Test forward/reverse,
   positive/negative angle, perpendicularity and nonpositive fluid-relative
   work. Keep the existing coefficient/stall model.
2. **Power bounds rotor authority.** Limit differential power by available
   collective and upper headroom, and scale cyclic/yaw by actual rotor
   delivery. Ground effect and inflow remain physical multipliers. Avoid
   an arbitrary minimum torque floor. At negligible gravity, omit the
   gravity-based tilt correction; retain finite attitude/rate controls and
   commanded thrust. Test no powered torque/thrust at zero collective,
   powered response, hover and unattended landing. Conversion measurements
   establish a baseline; do not automatically tilt nacelles or suppress stall.
3. **Sample each wet foil where it acts.** Use the foil's world point and
   immersion depth independently. Share the sampling helper between boats
   so keel, rudder, lateral plane and skeg have one rule. Test a pitched
   canoe with only one foil submerged and a wave/current sample.
4. **Name reference velocities explicitly.** Preserve existing ground-speed
   fields for callers, add water-speed readings, and calculate Tern leeway
   from water-relative tangent velocity. HUD labels both references. Wind
   angle displays include port/starboard; VMG remains over ground and is
   labeled as such. Test a comoving cross-current and opposite wind sides.
5. **Render the physics geometry and working blade.** Expose the core wing
   panel geometry to drawing instead of copying incidence/dihedral math.
   Draw two area-derived panels. Size the paddle blade from configured
   area. Add explicit active-rudder telemetry so a blade held motionless
   in still water is still visible in the correct place; do not infer it
   from nonzero force. Keep stroke/recovery animation.
6. **Keep the camera immediate and the planet frame explicit.** Subtract
   the frame centre in f64 before deriving berth direction. Verify raw
   displacement changes the occupied camera on that update, regardless of
   fixed-tick progress or ship response. Test menu suppression and active
   camera selection in water-state preparation, including a translated
   frame. Use existing capture flags, not a new gameplay automation mode.
   Order vehicle placement/following after the planet-frame update and before
   transform propagation, then publish water-at-eye after propagation. This
   prevents an origin change from mixing the previous centre with the new eye.
7. **Readable vehicle panel.** Add a subdued backing and concise state/action
   text to the existing vehicle-only panel; retain binding hints from the
   authoritative table. Keep controls discoverable without changing key
   assignments. Publish whether the Kestrel is using climb hold from the
   controller itself, so the panel can explain climb-rate versus power-lever
   input without duplicating its transition thresholds. Show the Loon's
   stroke/recovery/rudder state. Tests pin meaningful values and state hints;
   screenshots assess legibility and geometry.

## Risks / Trade-offs

Implementation measurement: the first power-bounded cyclic correction exposed
a conversion regression (33.7 m height loss and 22.1 degrees pitch, versus
12.9 m / 9.2 degrees baseline). Do not relax the conversion acceptance band.
Measure the effect of calibrating the configured full-power cyclic/yaw
ratings against the old controller's authority at nominal hover collective
(0.625 for 1200 kg at 25 m/s^2 with two 24000 N rotors). The old formula
delivered 0.875 times the rating there; a power-linear model needs 1.4 times
the full-power rating to match that nominal point. Only change these validated
ratings if the probe restores the conversion/hover bands; retain zero torque
at zero delivered thrust and keep thrust, nacelle limits and stall unchanged.
The measured 1.4 calibration restores conversion to 13.03 m loss and 9.17
degrees maximum pitch, with 7.1283 m/s climb and 1.2102 m/s crosswind drift.
Adopt 12600 N m pitch / 9800 N m yaw as the full-power ratings, regenerate
the shipped RON from defaults, and pin conversion below 20 m loss / 12
degrees pitch. No thrust or nacelle-rate tuning changes.

- Reverse-flow forces affect sternway and sailing at broad angles: rerun
  every existing scenario and compare the deterministic probe.
- Removing unpowered steering can alter assisted hover response: preserve
  the existing climb/crosswind/unattended tests and measure before/after.
- Screenshot captures at rest cannot prove underway feel or storm handling:
  report that limit explicitly; deterministic scenarios supplement them.
- No human playtest can be substituted by numeric acceptance bands. Avoid
  claiming that the owner has approved handling or the new captures.
- The original change remains open for unrelated requirements. Do not
  archive it or migrate unproven deltas.

## Migration and commit plan

First commit contains only these planning artifacts and baseline findings.
Next commit fixes and tests core force/control/sampling/telemetry behavior,
with just the matching main-spec requirements and the reproducible probe.
Then commit frame/camera/control fixes and their tests/specs. Finally commit
the visual/instrument changes and tests/specs with before/after evidence.
Every commit follows fmt check, both release package test suites offline,
and all-target Clippy. Validate OpenSpec as well. No push or branch switch.
Rollback is a normal revert; no saved data migration is needed.

## Implementation measurements

The same release probe after the core corrections (default torque scale 1):

| Scenario | After | Comparison |
| --- | --- | --- |
| Kestrel climb, 3 s | 7.1283 m/s; 0.4916 degrees tilt | Essentially unchanged |
| Kestrel crosswind, 20 s | 1.2102 m/s drift; 100.1550 m height | Essentially unchanged |
| Kestrel conversion | 85.6500 m/s air; -15.1420 m/s climb; 9.1738 degrees maximum pitch; 86.9740 m minimum height; wing share 1 | Baseline range retained after rating calibration |
| Zero gravity / unpowered controls | Finite / 0.000000 rad/s | Both demonstrated defects removed |
| Tern close reach | 3.1754 m/s; 1.6425 m/s VMG; 19.5870 degrees heel | Unchanged |
| Tern cross-current leeway | -0.7339 degrees | Down from 88.5674 degrees; small windage during the tick remains physical |
| Loon alternating strokes | 1.3517 m/s; 50 strokes/min | Unchanged |
| Loon stern rudder | +/-6.8091 degrees in 1 s | Unchanged, correct directions |
| Forward / reverse tail foil force | +365.3430 / +176.1025 N upward | Forward unchanged; reverse corrected from -172.0212 N |

The core release suite now has 176 passing tests and 6 existing ignored
reports. New regressions cover the force direction/continuity, unpowered
controls, vanishing gravity, conversion envelope, water-relative readings,
rudder signs and independently immersed, locally sampled wet foils.
Raw probe/check logs and retained executables are local ignored evidence in
`output/vehicles-improve/`; the example and this table are committed so the
measurements can be repeated without those files.

Core commit verification: fmt and all-target Clippy pass offline; the core
release suite passes. The first app run terminated with Windows
`STATUS_ACCESS_VIOLATION`, without a Rust assertion failure. Its retained
test executable passed all 130 library tests serially. The subsequent full
`cargo test -p pbd-app --release --offline -j 2` run passed 135 library and
14 desktop tests, including the separately prepared integration regressions.
The process crash's underlying cause is not established; it is not silently
counted as a passing run. OpenSpec validates all 56 items.

Integration evidence: all five new regressions pass in the 135-library /
14-desktop release app suite. Fleet placement, boarding and the seat camera
are checked at centre (8192, -4096, 2048) m; an additional origin change of
(512, -256, 128) m is consumed by camera following on the same update.
Raw look changes the seat camera with a zero-duration update while physical
orientation stays unchanged. Tests also exercise view switching, the three
control mappings and menu suppression. A real generated water column reads
the occupied camera's newly propagated underwater eye while ignoring the
parked camera, then reads air when camera activity switches back.
