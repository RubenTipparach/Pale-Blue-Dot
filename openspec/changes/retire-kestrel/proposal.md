## Why

The owner, 2026-11-14, on the Kestrel tiltrotor seen in the stone-and-wood UI
captures: "I hate that plane model", then "can we remove it, since we are
building better plane systems?". The better systems are `vehicle-yards` (M24,
Hangar and Chibi Aircraft: "it feels pretty good so far") and `ship-tiers`
(M21, rockets and landers). The Kestrel is the only aircraft in the game
today, and it is the one the owner wants gone, not reskinned.

## What Changes

- **New worlds have no Kestrel.** The fleet a new world places is the Tern
  and the Loon. The harbours' boats are unchanged.
- **Old worlds keep their Kestrel's record, untouched, and do not fly it.**
  A save that names a Kestrel still opens. The record is held aside on load
  and written back byte for byte whenever the vehicle file is rewritten. It is
  not drawn, not boardable and not simulated. Nothing is deleted from a save,
  so a later change can give the record to the new aircraft
  (CLAUDE.md, "Saved games survive every change").
- **The Kestrel's code goes:**
  - its flight model (`pbd-core/src/vehicle/kestrel.rs`);
  - its authored model and swatches;
  - its HUD panel;
  - its bindings and the Input tab's Flying sub-tab, while no craft flies;
  - its capture modes (`--ship-in-cave`), and the tests that only test it.
  The shared foil model stays: the sails and keels use it.
- **Kept as it is:** where a new world starts the player. The level plain the
  Kestrel's pad was looked for on is still the start, under a name that no
  longer mentions a pad, so new worlds start where they did.
- Mockup M19 (Kestrel, Tern and Loon) is marked "Kestrel retired" in the
  tracker list. Its page is not republished for this.

## Capabilities

### Modified Capabilities
- `player/vehicles`: the Kestrel's requirements are removed; the fleet is the
  boats; a retired craft's record is kept and not flown.

## Impact

- Code: `pbd-core::vehicle` (`Kind`, `CraftState`, `kestrel.rs`, records),
  `pbd-app::vehicles` (place, draw, hud, view, model, tests), `controls.rs`,
  `desktop.rs` (capture modes, the moving-lights scene), `walking.rs` (the
  start's name only).
- Saves: no format change. `RECORD_VERSION` stays 1. A save with a Kestrel
  opens, and is pinned by a test that loads one and writes it back identical.
- Frame cost: a craft fewer to step and draw in a new world. Not measured; the
  cloud has no GPU.
