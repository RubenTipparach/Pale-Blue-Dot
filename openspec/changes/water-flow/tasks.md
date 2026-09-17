# Tasks

- [ ] Advection and falling-face scroll in `water.wgsl`, reading a per-cell
      `vec2` flow from a storage buffer.
- [ ] The buffer, zero-filled, bound to the water draw; `flow_uv_speed_falling`
      in `water.ron`.
- [ ] A test that a zero field leaves the wave sample point exactly where the
      unadvected form puts it, so the hook is provably a no-op until driven.
- [ ] Blocked on `voxel-engine-foundation` and a river generator: the field.
