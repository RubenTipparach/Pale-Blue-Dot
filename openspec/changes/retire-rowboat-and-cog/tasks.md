## 1. Saves first

- [x] 1.1 `"rowboat"` and `"cog"` in `Kind::RETIRED`. Test: a fixture from
      the build before this one with harbour rowboats, a cog and a made
      rowboat opens with its live craft only, every rowboat and cog record
      written back byte for byte, `next_id` above them all.
- [x] 1.2 A made rowboat's refund (design 2): the recipe's materials and its
      hold into the slots, with a `refund` record, in one line; once; all or
      nothing, retried while the pack is full. Tests: refunded once and not
      again after a reload; a full pack waits, then refunds when room is
      made; a hold's stacks come with it.
- [x] 1.3 A rowboat good in the slots becomes its materials (design 3), and
      a queued rowboat job collects as them. Tests for both.
- [x] 1.4 The put-ashore reach is 9 m for a held cog (design 4). Test: a
      walker on a held cog's deck over the sea goes ashore.

## 2. The rowboat and the cog out

- [x] 2.1 Harbours make a craft for a `canoe` only; no cog. The town cuts no
      ship or gangplank; `over_water` unchanged. Tests: a harbour moors its
      canoes only, held berths are not refilled, the harbour's cells over
      water are as before.
- [x] 2.2 Remove the rowboat recipe; "Take a boat" asks for a canoe at a
      quay and counts a harbour's canoe.
- [x] 2.3 Remove `Kind::Rowboat`, `Kind::Cog`, their states, telemetry,
      hulls, specs, `cog.rs`, `pieces/cog.rs`, and their sections of
      `vehicles.ron`.
- [x] 2.4 Remove the draw arms, HUD panel, view index, ROWBOAT and COG key
      groups, `--aboard rowboat|cog`, `--sail`, `--sail-for`, `--cog-swing`,
      the swing and berth code, `decks.rs` and the walker's deck riding, the
      perf suite's `sail` scenario. A saved `rowboat:`/`cog:` rebind is
      ignored and kept (test).
- [x] 2.5 Regenerate `inputs.json`; switch the "harbour boat" tests to the
      canoe; remove the rowboat- and cog-only tests with their requirements.

## 3. Close

- [x] 3.1 Move the spec deltas into `openspec/specs/` in the commit that
      makes them true.
- [x] 3.2 Captures, before and after, in
      `docs/screenshots/retire-rowboat-and-cog/`: a new world at a harbour,
      and an old world (made by the build before, with a made rowboat)
      opened by the new one.
- [x] 3.3 `cargo fmt`, clippy with `-D warnings`, `cargo test -p pbd-core
      -p pbd-app`, `openspec validate --all`.
- [x] 3.4 Whether the Loon swamps at its spawn and at a harbour berth:
      measured on the runs, reported, not changed.
- [x] 3.5 The tracker's and `mockup-work.md`'s rows for the mockups that
      show the rowboat or the cog: none names them (they are part of the
      towns mockup's harbour, and M19 is the Kestrel, Tern and Loon), so
      nothing changes there.
