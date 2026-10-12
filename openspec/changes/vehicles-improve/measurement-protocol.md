# Investigation protocol (before behavior changes)

The existing vehicle scenario tests assert broad pass bands but print no
flight or handling measurements. A measurement-only example may call the
existing public core API without changing any gameplay code or defaults.
Its purpose is to supply the baseline numbers for this change's design.

Use the shipped defaults, a 4799.5 m spherical sea, 25 m/s^2 gravity,
60 Hz fixed ticks with four force-evaluated substeps, and a deterministic
world clock starting at 1000 s. Report simulated durations and inputs.
Measure Kestrel takeoff/climb, centred crosswind hover, zero-gravity
finiteness, and a pilot-commanded conversion to wing flight. Measure Tern
close-reach speed/VMG and current-relative leeway; measure Loon alternating
strokes and both stern-rudder signs. Include foil force symmetry for
forward/reverse relative flow. Preserve the instrument for before/after
comparison; do not mix fixes or tuning changes into it.

Run the existing release tests offline before the planning commit. Build
and retain the baseline release executable in ignored output storage.
Capture all three craft using the existing --aboard and --capture flags,
with no --world so investigation cannot modify a saved world. Inspect the
actual images, including the Loon paddle and Tern instruments. Record
hardware, resolution, world seed, frame count, and capture limitations.
This is a correctness/handling investigation, not a performance claim.

Repeat the numeric instrument with:

```powershell
cargo run -p pbd-core --example vehicle_probe --release --offline -j 2
```

The optional first argument scales only the instrument's Kestrel torque
ratings for calibration (default 1); it does not edit defaults or assets.
The unchanged executable and the original vehicle RON are retained as
`output/vehicles-improve/baseline-pbd-app.exe` and `baseline-vehicles.ron`.
The executable reads runtime assets: reproducing the baseline requires the
original configuration as well as the executable. The committed baseline
configuration is also available at the planning commit, `e986318`.
