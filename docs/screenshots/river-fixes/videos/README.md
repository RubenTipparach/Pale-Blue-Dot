# Waterline videos (ocean-environments B18)

The owner, 2026-11-16: "rivers not validated", "I need to see you go above and
below water frame by frame", "validate wherever they merge".

**Where the videos are.** Each place has a before and an after video, H.264 at
1080p60, in the owner's Drive folder
`My Drive/dungeon-crawler-2026/pale-blue-dot-validation/`, named
`river-waterline-<place>-<before|after>-1080p60.mp4`. This folder holds a
12-frame contact sheet of each one, `<place>-<before|after>-sheet.jpg`.

**How they were made.**
- **Renderer:** the Mac (M4 Max, Metal), `--profile fast`.
- **Frame rate:** a fixed 60 fps step, so the videos show looks, not frame
  time.
- **Window:** 960x540, scaled up to 1080p.
- **Weather:** rain is on. The lens drops are off (`PBD_PASS_OFF=lens`).
- **Builds:** before is `origin/claude/env-v2` at `bb16cbd`, with only the
  capture instrument added. After is `claude/river-fixes-2`.
- **Camera:** the camera follows `PBD_EYE_PATH` (`desktop/eye_path.rs`), a
  capture-only instrument. The label in the bottom right shows three things:
  - the eye's height against the water surface at the eye (the drawn top for
    inland water, the tided sheet for the sea);
  - the eye's water state (`eye:water` means the column has water at the eye,
    even where it stands just over the drawn top);
  - the capture frame.
- **The path:** with `PBD_EYE_PATH_RELATIVE=1`, heights are taken over the
  water under the eye. The eye starts 1.5 m over the surface, goes down to
  2.5 m under it and comes back up. The junction videos then travel across
  the junction 0.3 m over the surface and come back 0.4 m under it.

| Place | Command (`video.py` arguments) | What it crosses |
| --- | --- | --- |
| `owner-spot` | `28.25 0.75`, surface 66.81 m, yaw 0 | The owner's spot in the river channel |
| `river-bank` | `28.25 0.75`, relative, yaw 90 | The same river, looking at its bank |
| `sea-shore` | `18.86 27.54`, relative, yaw 180 | A sea shore with a bank of sand |
| `mouth` | `20.75 10.12` to `20.75 9.70`, relative, yaw 270 | A river mouth into the sea |
| `tributary` | `24.00 1.00` to `23.70 1.30`, relative | Where two rivers join |
| `fall-into-pool` | `23.70 -2.80` to `23.78 -2.72`, relative | A riffle fall into the reach below |
