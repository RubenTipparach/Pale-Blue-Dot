## Why

The owner, 2026-11-17, on a screenshot of a plain white hull lying tipped in
the sea off a forested shore: "delete these boats, we have mockups for new
boats which we will be adding soon", and then "also idk why but that boat is
capsized lol". The new boats are M23 (Drydock and Hex Ship Builder) and M34
(River Craft). They are not built in this change.

The boat in the screenshot is the **Tern**, the sailing dinghy. It is the only
craft in the game with a white hull:
- the Tern is drawn procedurally in off-white (`draw.rs`, `srgb(0.86, 0.84,
  0.78)`), with a salmon deck strip, a dark mast and a white sail;
- the Loon (canoe) is in driftwood, the Rowboat in brown boards, the cog in
  timber, the tube and the raft in M34's felt and logs.

Lying on its side, the Tern shows only its white topsides and bottom, which is
the "plain white open hull" of the screenshot. Captures of each craft are in
`docs/screenshots/retire-tern/`.

"These boats" is every Tern the world makes:
- the one placed near the player in a new world;
- every harbour's sailing boats, which the harbour template calls `sail` and
  `harbour::kind_of` turns into Terns (31 of the 120-odd harbour boats in a
  world).

## What Changes

- **New worlds have no Tern.** A new world's fleet is the Loon. Harbours moor
  their rowboats and canoes and their cog; a `sail` berth is left empty.
- **Old worlds keep their Terns' records, untouched, and do not sail them.**
  `"tern"` joins the retired kinds that `retire-kestrel` made: the record is
  held aside on load, written back byte for byte, never drawn, boarded or
  stepped, and its ID is never given again. Nothing is deleted from a save
  (CLAUDE.md, "Saved games survive every change").
- **A player who saved aboard a Tern is put ashore.** Occupancy is not saved;
  a player who quit aboard comes back on foot at the seat. With the Tern gone
  that is open water, so the load puts them on the nearest dry land instead.
- **A harbour's held Tern keeps its berth.** The berth is not refilled with
  another boat, so the harbour looks the same in an old world and a new one.
- **The Tern's code goes:**
  - its sailing model (`pbd-core/src/vehicle/tern.rs`), spec (`TernSpec`,
    `SailSpec`, `KeelLift`, `CrewSpec`), hull and section of `vehicles.ron`;
  - its draw arm, HUD panel, chase framing and view index;
  - its key group (TERN), its capture boarding (`--aboard tern`);
  - the tests that only test it, and its requirements.
  The foil model stays: the cog's sail, keel and rudder use it.
- **Its art goes too:** the Tern's authored Blender model under
  `assets/models/vehicles/tern/`, which nothing in the game reads
  (`vehicles::model::AUTHORED` is empty). The mockup tools in `tools/art/`
  are left as they are.
- Mockup M19 (Kestrel, Tern and Loon) is marked "Tern retired" in the tracker
  list. Its page is not republished for this.

## Capabilities

### Modified Capabilities
- `player/vehicles`: the Tern's requirements are removed; a new world's fleet
  is the Loon; a retired craft's record is kept and not sailed, and a player
  loaded at one is put ashore.

## Impact

- Code: `pbd-core::vehicle` (`Kind`, `CraftState`, `Telemetry`, `Hulls`,
  `tern.rs`, `spec.rs`, `record.rs`), `pbd-app::vehicles` (place, harbour,
  draw, hud, view, chase, tests), `controls.rs`, `settings.rs`, `desktop.rs`,
  `main.rs` (`--help`), `assets/config/vehicles.ron`.
- Saves: no format change. `RECORD_VERSION` stays 1; the record's `sheet`
  field stays in layout 1 for the held Tern records, as `nacelle` and `brake`
  did for the Kestrel.
- Frame cost: thirty-odd craft fewer to step and draw near harbours. Not
  measured as a speed-up; nothing a player sees except the Terns changes.
