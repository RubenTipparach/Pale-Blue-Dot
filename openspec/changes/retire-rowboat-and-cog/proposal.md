## Why

The owner, 2026-11-18, after the Tern was retired (`retire-tern`, PR #116):
"Rowboats and cog yes. Remove them. We are going to replace them with
craftable ones." The replacements are M23's rowing launch (Drydock, built
from hex cells) and M34's river craft. They are not built in this change.

Two craft go:
- the **rowboat** (`Kind::Rowboat`): every harbour moors fourteen, and the
  workbench makes one (`small-boats` design 3: 16 planks and 2 rope);
- the **cog** (`Kind::Cog`, `sail-the-cog`): the harbour's ship, one per
  harbour, moored on its swing at its gangplank, sailed from its helm, its
  deck walked on.

## What Changes

- **New worlds have no rowboat and no cog.** A harbour moors its three
  canoes (Loons) and nothing else; its rowboat berths, its sailing berths
  and its cog's berth are left empty. Nothing else in the game makes either.
- **The rowboat's recipe goes from the workbench.** A saved station job for
  one, and a rowboat good in the pack, give the recipe's materials back
  (design 4).
- **Old worlds keep every rowboat and cog record, untouched, and do not sail
  them.** `"rowboat"` and `"cog"` join `Kind::RETIRED`: a record is held
  aside on load, written back byte for byte, never drawn, boarded or
  stepped, and its ID is never given again. A held harbour boat's berth
  counts as made and is not filled again.
- **A rowboat the player made is refunded, once.** On load the player gets
  its recipe's materials back in full (16 planks, 2 rope), with anything in
  its hold. The refund is written with the player's slots in one line, so it
  is never given twice and never half given. If the pack has no room, the
  refund waits in the save until it has.
- **A player saved aboard a rowboat or a cog, or on a cog's deck, loads on
  the nearest dry land**, by the Tern's rule.
- **The "Take a boat" mission** asks for a canoe at a town's quay, which
  every harbour still moors, so it stays possible.
- **Their code goes:**
  - the cog's sailing model (`vehicle/cog.rs`), its spec (`CogSpec`,
    `SquareSailSpec`) and its section of `vehicles.ron`; the rowboat's spec
    and section;
  - the cog's ship and gangplank cut into the harbour (`pieces/cog.rs`), its
    swing at the mooring, its berth, making fast, its stern lantern;
  - the moving decks a walker rides (`decks.rs`, and its part of
    `walking.rs`), which only the cog had;
  - their draw arms, HUD panel, view index, key groups (ROWBOAT, COG),
    `--aboard rowboat|cog`, `--sail`, `--sail-for`, `--cog-swing`, the perf
    suite's `sail` scenario;
  - the tests that only test them, and their requirements.
- **Kept:** the Loon, the inner tube and the raft; the harbour's canoes; the
  towns' beached and landed rowboats, which are dressing on the shore and
  not craft; the template's cog and boat entries, so a harbour's ground is
  cut cell for cell as before; the `rowboat` good and its icon, so a saved
  stack of one still reads.

## Capabilities

### Modified Capabilities
- `player/vehicles`: the rowboat's requirements go; harbours moor canoes
  only; a retired made boat is refunded once; the put-ashore rule covers a
  cog's deck.
- `player/walking`: the moving deck requirement goes with the cog.

## Impact

- Code: `pbd-core::vehicle` (`Kind`, `CraftState`, `Telemetry`, `Hulls`,
  `cog.rs`, `spec.rs`, `record.rs`), `pbd-core::settlement` (`pieces/cog.rs`,
  `record.rs`, `sea.rs`), `pbd-core::station` (the recipe, a retired
  recipe's refund), `pbd-app::vehicles` (place, harbour, draw, hud, view,
  tests), `decks.rs`, `walking.rs`, `towns.rs`, `controls.rs`, `desktop.rs`,
  `desktop/missions.rs`, `main.rs` (`--help`), `assets/config/vehicles.ron`,
  `recipes.ron`, `missions.ron`, `assets/models/vehicles/inputs.json`,
  `tools/perf_suite.py`.
- Saves: no format change. `RECORD_VERSION` stays 1; the record's `yard`
  field stays for the held cog records. One new record kind, `refund`
  (schema 1), keyed by the craft's ID, marks a made boat refunded.
- Frame cost: some fifteen craft fewer per harbour to step and draw. Not
  measured as a speed-up.
