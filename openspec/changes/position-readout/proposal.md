# Proposal: where the camera is, on the HUD and in the log

## Why

The owner (relayed from the desktop session, 2026-11-13) asked for latitude
and longitude, elevation and height above ground on the HUD's debug line, and
in the log at spawn, at `READY` and at each capture, so that every screenshot
can say where it was taken and be taken again.

## What changes

- One readout of the active 3D camera's place (`pbd_app::place`): the
  latitude and longitude `--at` takes (the body frame's, so a spot can be
  reproduced exactly), the map's reading of the same place (compass latitude,
  N or S, and E or W), the eye's elevation above sea level, and its height
  above the ground under it (the generated surface, as the rain's gate reads
  it).
- The HUD's debug line gains it as a second line.
- The log gains `PLACE <when> ...` at spawn, at `READY` and with each capture's
  `saved` line.

## Not in this change

- No change to `--at`, to the map or to the compass. The two latitudes differ
  in sign (`geo.rs`: the compass's north is the frame's -Y); the readout
  prints both, labelled, rather than changing either.
