## 1. Saves first

- [x] 1.1 (`crates/pbd-app/src/vehicles/fixtures/kestrel-era-vehicles.ron`: the first three records of a vehicle file the build before this one wrote, its README says how.) A test fixture: a version-1 vehicle file holding a Kestrel, a Tern
      and a Loon, as an existing world's would.
- [x] 1.2 (`Fleet::retired`; `vehicles::tests::a_world_saved_with_a_kestrel_opens_with_its_boats_and_keeps_the_record` and `a_held_record_keeps_its_id_from_every_new_craft`. The saved kind is `"kestrel"`, lower case: design, "Findings".) The loader holds retired records aside (`Retired` on the fleet), and
      the writer appends them unchanged; `next_id` never falls below a held
      ID. Test: load the fixture, move the Tern, save, and the Kestrel's
      record is byte-identical and the world opens with two craft.

## 2. The Kestrel out

- [x] 2.1 (The rule's constants renamed, their values and arithmetic kept; the new build logs the same `PLACE spawn` as the old one, `docs/screenshots/retire-kestrel/`.) `spawn_fleet` places the Tern and the Loon only; the start search in
      `walking.rs` keeps its rule, renamed. The spawn digest tests still pass
      unchanged.
- [x] 2.2 (The authored-model loader stays with no craft registered: design, "Findings".) Remove `Kind::Kestrel`, `CraftState::Kestrel`, `kestrel.rs`, the
      telemetry, the HUD panel, the draw and view arms, the authored model
      and swatches.
- [x] 2.3 (The KESTREL group is gone; the Flying sub-tab stays, because it also holds the free flight's keys, and a kept rebind needed `KeyBinds` to carry what it skips: design, "Findings".) Remove the Kestrel's bindings from the table, `--help` and the
      Input tab, and the Flying sub-tab with them. A saved rebinding of a
      removed action is ignored and kept in the settings file.
- [x] 2.4 (The rig's one test went with it; the moving things' light is pinned on the boats by `vehicles::tests::every_craft_is_lit_by_the_field`.) Remove `--ship-in-cave` and its scene; retarget its lighting test.
- [x] 2.5 Switch the "any craft" tests to the Tern; remove the Kestrel-only
      ones with their requirements.

## 3. Close

- [x] 3.1 Move the spec deltas into `openspec/specs/player/vehicles/spec.md`
      in the commit that makes them true.
- [x] 3.2 Captures: a new world's harbour and start (no Kestrel), and an old
      save opened (its boats, no Kestrel), lavapipe, in
      `docs/screenshots/retire-kestrel/`.
- [x] 3.3 (`desktop::tests::the_screen_reads_the_compass_not_the_frame` fails as it did before this change; it is `coast-walk`'s.) `cargo fmt`, clippy with `-D warnings`, `cargo test --profile fast`,
      `openspec validate --all`; no `LATE_PIPELINE` after `READY`.
- [x] 3.4 (Also its note in `docs/mockups/tracker.html`; the tracker is not republished for this.) M19's row in `docs/mockups/mockup-work.md`: "Kestrel retired
      (owner, 2026-11-14)".
