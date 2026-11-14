## 1. Saves first

- [ ] 1.1 A test fixture: a version-1 vehicle file holding a Kestrel, a Tern
      and a Loon, as an existing world's would.
- [ ] 1.2 The loader holds retired records aside (`Retired` on the fleet), and
      the writer appends them unchanged; `next_id` never falls below a held
      ID. Test: load the fixture, move the Tern, save, and the Kestrel's
      record is byte-identical and the world opens with two craft.

## 2. The Kestrel out

- [ ] 2.1 `spawn_fleet` places the Tern and the Loon only; the start search in
      `walking.rs` keeps its rule, renamed. The spawn digest tests still pass
      unchanged.
- [ ] 2.2 Remove `Kind::Kestrel`, `CraftState::Kestrel`, `kestrel.rs`, the
      telemetry, the HUD panel, the draw and view arms, the authored model
      and swatches.
- [ ] 2.3 Remove the Kestrel's bindings from the table, `--help` and the
      Input tab, and the Flying sub-tab with them. A saved rebinding of a
      removed action is ignored and kept in the settings file.
- [ ] 2.4 Remove `--ship-in-cave` and its scene; retarget its lighting test.
- [ ] 2.5 Switch the "any craft" tests to the Tern; remove the Kestrel-only
      ones with their requirements.

## 3. Close

- [ ] 3.1 Move the spec deltas into `openspec/specs/player/vehicles/spec.md`
      in the commit that makes them true.
- [ ] 3.2 Captures: a new world's harbour and start (no Kestrel), and an old
      save opened (its boats, no Kestrel), lavapipe, in
      `docs/screenshots/retire-kestrel/`.
- [ ] 3.3 `cargo fmt`, clippy with `-D warnings`, `cargo test --profile fast`,
      `openspec validate --all`; no `LATE_PIPELINE` after `READY`.
- [ ] 3.4 M19's row in `docs/mockups/mockup-work.md`: "Kestrel retired
      (owner, 2026-11-14)".
