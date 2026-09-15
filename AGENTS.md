# Agent Guide

These instructions apply to the entire repository. More specific `AGENTS.md` files may refine them for their directory.

## Mission

Build Tenebris as a data-oriented Rust game using Bevy and Avian. Preserve useful domain ideas from earlier prototypes, but do not reproduce their engine coupling or rocket orbital simulation. Optimize for deterministic world identity, bounded local simulation, debuggability, and incremental delivery.

## Required workflow

1. Read `docs/ENGINE_ARCHITECTURE.md` and the relevant section of `docs/GAME_DESIGN.md` before changing behavior.
2. Inspect neighboring modules and tests; prefer established patterns over new abstractions.
3. Keep changes narrow. Do not mix formatting, dependency upgrades, and features in one patch.
4. Add or update tests for deterministic generation, coordinate conversions, state transitions, and serialization changes.
5. Run formatting, lints, and the smallest relevant tests before the full workspace test suite.
6. Update design docs when a change modifies an invariant, component ownership, player-facing rule, or save/network schema.

## Rust and Bevy conventions

- Use stable Rust, `rustfmt`, and Clippy-clean code. Avoid `unsafe` unless a measured bottleneck requires it and the safety invariant is documented.
- Organize functionality as Bevy plugins with explicit schedules, run conditions, events, resources, components, and system sets.
- Components contain state; systems perform behavior. Keep large algorithms in ordinary Rust modules that can be tested without starting a Bevy `App`.
- Prefer newtypes for coordinate spaces, identifiers, seeds, distances, and simulation time. Never pass an unlabelled `Vec3` across coordinate-space boundaries.
- Use `f64` for astronomical positions, ephemerides, world anchors, and generation inputs. Use rebased `f32` values for rendering and Avian's active physics bubbles.
- Avian is authoritative only inside active local physics bubbles. Never add a collider or rigid body for every generated voxel or celestial body.
- Express cross-plugin communication with typed events/messages and narrow public APIs; do not mutate another plugin's private resources.
- Keep frame-rate-dependent presentation in `Update`, fixed simulation in `FixedUpdate`, and expensive streaming/generation in asynchronous tasks with explicit budgets.
- Derive reflection and serialization only where they are deliberately part of tooling or persistence. Version persisted data.
- Avoid global singletons, stringly typed state, hidden randomness, blocking I/O in systems, and order-dependent queries.

## Determinism and performance

- Every procedural result must be a pure function of the declared world seed, generator version, stable object ID, and integer/canonical coordinates.
- Use isolated RNG streams per generation stage. Adding a decoration must not reshuffle terrain, resources, or settlements.
- Sort inputs before hashing or serializing when collection order is otherwise unstable.
- Profile before optimizing, but treat unbounded per-frame work, mesh rebuild storms, per-voxel entities, and cross-world scans as defects.
- Streaming work must be cancellable and discard stale task results by request/version ID.

## Flight invariant

Celestial bodies and stations follow authored or deterministic on-rails trajectories. Player and AI ships do not. Ships integrate thrust and gravity in the active local frame, then flight-assist controllers apply acceleration, speed, inertial, and rotational limits. Assists are explicit, tunable control systems—not teleportation, orbital solvers, or instantaneous velocity clamps.

## Quality gates

Once an executable workspace exists, the expected baseline is:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Document any platform-specific check that cannot run. Do not silently weaken or delete a failing test.

## Documentation

- Explain intent and invariants, not syntax.
- Use units in names or types and state budgets numerically.
- Record architectural decisions under `docs/decisions/` when alternatives or migration consequences matter.
- Do not claim parity with an older prototype unless an acceptance test demonstrates it.

## Commits and pull requests

- Use an imperative, scoped commit subject (for example, `docs: define streamed planetary architecture`).
- In the pull request, summarize user-visible and architectural effects, list exact checks run, and call out migrations, risks, and deferred work.
