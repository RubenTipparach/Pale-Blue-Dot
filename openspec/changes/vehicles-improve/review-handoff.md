# Handling review handed to Claude

The owner redirected work to new Blender vehicle models and explicitly assigned
all further in-game captures and review images to Claude. The handling change
therefore remains open for visual review; its tested implementation is committed
before the model planning commit. No further game captures are taken by Codex.

## Implemented and measured

The design contains the deterministic before/after handling measurements. The
final probe is unchanged by the shared wing geometry refactor. Rendered wing
planform area changes from 16.96 to the configured 16 m2, with 3 degree incidence
and 4 degree dihedral. Paddle blade area changes from 0.09 to 0.11 m2. A held
stern rudder is visible even at zero water speed and during stroke recovery.

Chase camera tests cover clear and obstructed booms, restoring the requested
distance after obstruction, and projection of the Tern hull and masthead. HUD
tests cover water/ground speed labels, apparent wind side/angle, ground VMG,
hull-speed percentage, and authoritative Kestrel climb-hold state. These are
automated geometry and content checks, not an appearance approval.

## Remaining review

- Capture and inspect all three craft after the handling/model work, including
  chase and seat views, and a translated render frame. Check the Loon beside
  shore, Tern mast framing, paddle motion and HUD readability.
- Compare against the already inspected baseline images in local ignored
  `output/vehicles-improve/before-{kestrel,tern,loon}.png`. The protocol records
  resolution, seed, warm-up and baseline executable/config reproduction.
- Human underway and storm playtesting remains unverified. Deterministic core
  scenarios establish force and control behavior, not subjective feel.
- Frame-time percentiles, upload bytes and GPU memory were not measured; this
  change makes no performance claim.

The new CLI help text documents the existing `--aboard`, `--seat` and
`--render-offset` options. Final executable help execution is separate from
game capture and may be checked after the model build.

One repeated full app run failed three existing save-writer tests after their
drain calls returned without the expected files/commit marks. All five writer
tests passed immediately in isolation without a code change. The failure log
is retained locally as `visual-final-app.txt`; its cause is not established.
This is separate from the earlier recorded Windows process crash.
