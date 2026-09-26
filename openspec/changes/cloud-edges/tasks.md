# Tasks

- [x] 1. `cloud_previous` returns the kept texels' cloud distance (the resolve writing it for a history-only texel is `cloud-entry` task 1)
- [x] 2. The composite hazes to `span.x` where no kept texel has a distance
- [x] 3. `cloud_texel_sees_to`: the history's far test against the point read
- [x] 4. Cloudless texels read the history at its own cloud distance
- [x] 5. Before and after at `--route far-side --frames 3000`; a cloud-hop recording; the `--turn` check; GPU tests
- [x] 6. What remains on entry (the cloud-hop recording on this build, 18.3-19.2 s): no salt over the ground, but banding while the camera passes through a thin cloud, speckle along the fringes of distant clouds while moving fast, and a vertical curtain at a cloud's side. The march's raw noise and the unclamped history: the resolve pass (`cloud-close-up` section 2) combined with `cloud-ghosting`, and less raw noise (blue-noise jitter, the finest octave faded against the step). Taken up: the clip is `cloud-history-clip` (the other machine's), the rest `cloud-entry`
