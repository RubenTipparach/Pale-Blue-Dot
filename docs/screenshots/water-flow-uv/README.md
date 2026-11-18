# Flowing tops streaming with their arrows (`water-flow` 9)

The owner's cascade, frame 23.225, -3.012 (map 23.2255S 3.012W), a throwaway
world, `--walk --arrows --time 13 --rain 0 --pitch -35 --yaw 0`, the lens off
(`PBD_PASS_OFF=lens`), `--capture` at frames 900, 915, 930 and 945 (fixed
step: 0.25 s apart; looks, not frame time). The Mac (M4 Max, Metal), `--profile fast`.
`before` is `origin/main` (2de3c03a), `after` the branch.

- `cascade-{before,after}.jpg`: frame 900, the HUD cropped.
- `cascade-{before,after}-strip.jpg` and `.gif`: the cascade, four frames.

The measure is the audit, not the pictures: `PBD_FLOW_UV_AUDIT=1` logs every
top triangle of the live mesh whose stream runs against (over 90 degrees) or
off (over 45 degrees) its cell's arrow. At this spot, the same mesh, before:
1048 of 2946 triangles against (35.6 %); after: 0 of 2946 against, 0 off.
A per-patch phase correlation of the frames was tried and is too noisy to
call (the waves also change in place as they stream), so it is not used.
