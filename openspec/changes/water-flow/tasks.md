# Tasks

- [x] Advection and falling-face scroll in `water.wgsl`, reading a per-cell
      `vec2` flow from a storage buffer.
- [x] The buffer, zero-filled, bound to the water draw; `flow_uv_speed_falling`
      in `water.ron`.
- [ ] A test that a zero field leaves the wave sample point exactly where the
      unadvected form puts it. Not written: the term is WGSL and the project
      has no shader execution harness beyond the GPU visibility test; by
      construction the flow enters only as `flow * time * 0.35`, so zero is
      the unadvected form.
- [ ] Blocked on `voxel-engine-foundation` and a river generator: the field.
