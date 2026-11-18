## Why

The owner, 2026-11-18, after the rowboats and the cog were retired
(`retire-rowboat-and-cog`): "Can you spawn rafts and new rowboats at harbor
for now?" With the rowboats, the cog and the Tern gone, a harbour moors its
three canoes and nothing else; its quays stand empty.

Two boats are asked for:
- **rafts**: the raft is built (`small-boats`, M34's), so a harbour can moor
  them now;
- **new rowboats**: the new rowboat is M23 Drydock's "Rowing launch", built
  from hex cells. It is not built and not approved for the game, so it is
  not built here. When it is, harbours moor it too (design 4).

## What Changes

- **Harbours moor rafts** at the berths the rowboats and sailing boats used,
  one raft to a berth, skipping a berth whose raft would touch one already
  moored there (design 1): up to eleven a harbour, fewer where the water is
  too shallow. Each is a harbour craft like the canoes: made once when the
  fleet first meets the harbour, anchored at its berth, tagged with its
  berth, saved, stowed and woken, boardable by anyone ("you can use any boat
  you find").
- **Old worlds get them too.** A raft's berth is numbered apart from the
  template boat it stands in for, so a berth held by a retired rowboat's or
  Tern's record does not stop it, and the held record is never touched. The
  retired boat is not in the world, so nothing overlaps.
- **The cog's berth stays empty**: a raft there would lie in the middle of
  the ship's deep berth, away from the quay.
- A harbour raft is not the player's: it is not taken up into the pack
  (only a made boat is, `small-boats` design 4), and its hold is usable.

## Capabilities

### Modified Capabilities
- `player/vehicles`: harbours moor canoes and rafts.

## Impact

- Code: `pbd-app::vehicles::harbour` (`boats_for`, a raft berth rule),
  `towns/tests.rs`, `vehicles/tests.rs`.
- Saves: new harbour craft records (`kind: "raft"`, a `berth`); no format
  change. Worlds that met a harbour before keep what they have, and gain its
  rafts the next time the fleet meets it (each session, `moor_harbours`).
- Frame cost: up to eleven decked craft a harbour, asleep at rest
  (`town-frame-time` decision 6), stowed past 1.5 km. Not measured.
