## 1. Saves first

- [ ] 1.1 `"tern"` in `Kind::RETIRED`. Test: the Kestrel-era fixture (a
      Kestrel, a Tern and a Loon) opens with the Loon only, and both held
      records are written back byte for byte.
- [ ] 1.2 A fixture from the build before this one with harbour Terns in it,
      and a test that every Tern record in it is held and written back, and
      that `next_id` stays above them.
- [ ] 1.3 A walker loaded over the sea within 6 m of a held record is put on
      the nearest dry land (design 2). Test: the rule's pure part (over water
      and near a held record moves; on land, or far from one, does not).

## 2. The Tern out

- [ ] 2.1 `spawn_fleet` places the Loon only; its berth search is unchanged.
- [ ] 2.2 `harbour::kind_of` makes no craft for `sail`; the berths of held
      records count as made. Test: a harbour moors no Tern, and old berths
      are not refilled.
- [ ] 2.3 Remove `Kind::Tern`, `CraftState::Tern`, `Telemetry::Tern`,
      `Hulls::tern`, `tern.rs`, `TernSpec` and the types only it uses, and
      its section of `vehicles.ron`.
- [ ] 2.4 Remove the draw arm, HUD panel, chase framing, view index, the
      TERN key group and `--aboard tern`. A saved `tern:` rebind is ignored
      and kept (test).
- [ ] 2.5 Remove the Tern's authored model and its builder; `vehicle_art`
      exports the Loon's hull only.
- [ ] 2.6 Switch the "any craft" tests from the Tern to the Loon; remove the
      Tern-only tests with their requirements.

## 3. Close

- [ ] 3.1 Move the spec deltas into `openspec/specs/player/vehicles/spec.md`
      in the commit that makes them true.
- [ ] 3.2 Captures, before and after, in `docs/screenshots/retire-tern/`: a
      new world near a harbour, and an old world (made by the build before)
      opened by the new one.
- [ ] 3.3 `cargo fmt`, clippy with `-D warnings`, `cargo test -p pbd-core -p
      pbd-app`, `openspec validate --all`.
- [ ] 3.4 M19's row in `docs/mockups/mockup-work.md` and the tracker list:
      "Tern retired (owner, 2026-11-17)".
