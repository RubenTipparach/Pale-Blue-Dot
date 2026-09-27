# Design: a video at every gate

## Context

See `proposal.md` for why. What the design has to work with (2026-09-27):

- **In the game:**
  - `--capture <png>` saves one frame through Bevy's `Screenshot` of the
    primary window after `--frames` steps. With `--fixed-dt` the steps are
    fixed.
  - `--route far-side|scenic|clouds` and `--tour` fly scripted paths.
  - `--walk`, `--dig`, `--place`, `--torch`, `--time`, `--day`, `--spawn`,
    `--yaw` and `--pitch` set up the scenes the captures use.
- **On the owner's machine:** the `obs-record` skill launches the game, starts
  on an output line, stops on another or after a duration, and writes an MP4
  and a contact sheet at the window's full size. It recorded the far-side
  flight, 98 s at 60 fps (`docs/handoff-flight-recording.md`).
- **In a cloud session:**
  - there is no GPU, so the game renders on lavapipe and frame times mean
    nothing (CLAUDE.md);
  - there is no OBS and no system ffmpeg;
  - Playwright's ffmpeg (`/opt/pw-browsers/ffmpeg-1011/ffmpeg-linux`) is
    built to read JPEG frames from a pipe and write VP8 WebM or PNG, and
    nothing else;
  - Python has PIL, and Node has Playwright, which can record a page as
    video.
- **The mockups** (`docs/mockups/*.html`) already have scripted walks run in
  headless Chromium by the tests that measured the towns' stairs.

## Goals / Non-Goals

**Goals:**
- One shot list per gate, played unattended, recorded in the cloud and on the
  owner's machine alike.
- A video that tells the owner what each moment is showing and which
  requirement it proves.
- Honest labelling: a fixed-step recording shows looks, not smoothness.

**Non-Goals:**
- **A replay system, or recording a player's own play.** A showcase is
  scripted.
- **Audio.** The game has none worth recording yet.
- **Committing videos to git.**
- **Replacing `obs-record`.** It stays the real-time recorder, and this change
  only gives it a showcase to record.

## Decisions

**1. A shot list is data, and reuses the rigs the game has.**
- `assets/showcase/<name>.ron` is a list of shots. Each has:
  - `at`: a spawn, a walk from a point along a heading, or a named route;
  - `time`: a clock time plus a rate, so dusk can pass in ten seconds;
  - `camera`: held, panned or orbited, in degrees a second;
  - `actions`: at given seconds, one of place, dig, open, age the world, or
    board;
  - `seconds`: the shot's length;
  - `caption`.
- The player walks through the same systems the `--walk`, `--route` and
  `--tour` flags drive, and an action goes through the same input path a
  player's would. So a showcase shows the real game, not a staged one.
- A shot list names the requirement each shot demonstrates, which the gate
  page prints.

**2. `--record` takes every frame at a fixed step, through the capture path.**
- `--record <dir>` implies `--fixed-dt` at the recording's frame rate
  (default 30). It takes a `Screenshot` of every frame and writes
  `<dir>/000001.png` and on.
- The next frame waits for the previous screenshot's readback, so no frame is
  dropped, however slow lavapipe is.
- The showcase prints `SHOWCASE_START <name>` and `SHOWCASE_END <name>`, which
  are also what `obs-record` starts and stops on.
- The default size is 1280 × 720, and it can be changed. The cost per frame on
  lavapipe is measured by task 1.2. If a three-minute showcase takes over an
  hour, the cloud default drops to 960 × 540 at 24 fps, and the owner's
  recording keeps full size.

**3. `tools/make_video.py` encodes with whatever ffmpeg is present.**
- In the cloud, it reads the PNGs with PIL, writes JPEG to Playwright's
  ffmpeg through a pipe, and gets VP8 WebM.
- Elsewhere it uses imageio-ffmpeg or a system ffmpeg for H.264 MP4, as
  `obs-record` does.
- It writes a contact sheet of one frame per shot, captioned with the shot's
  number and start time.
- It refuses a directory whose frame count does not match the showcase's
  length, so a recording that stopped early is not published as whole.

**4. Mockups are recorded by Playwright.**
- `tools/mockup_video.js <page> <walk.json> <out.webm>` opens the mockup in
  headless Chromium with video recording on, and plays a walk: key presses,
  mouse drags and waits, the same form the stair tests use. It then saves the
  WebM.
- The towns mockup's walks are the first walk files, so the pipeline is
  proven on work that is already done.

**5. A gate page per step.**
- `tools/gate_page.py` builds one HTML page from the shot list, the video,
  the contact sheet and the render label.
- It is published as an artifact, with the video as a file published beside
  it (under the 15 MB a published file may be). A longer showcase is split at
  shot boundaries.
- The PR and the roadmap page link it.
- The contact sheet and the shot list are committed under
  `docs/screenshots/<change>/`. The video is not.

**6. The prototype beside the game.** The owner: "when you do the videos,
post the prototype too so I can compare/contrast stuff".
- A shot list names each shot's place by the names the mockup uses (the inn
  stair, the keep's newel, the harbour gangplank), with a heading and a time
  of day. The mockup's walk file for the same shot uses the same name, which
  is the mockup's `GOTO` list, so the two start in the same spot.
- The mockup is on a flat grid and the game is on the sphere, so the shots
  match by place and heading, not by coordinates.
- `tools/make_video.py --beside <prototype frames>` composes the two
  sequences into one frame, prototype left and game right, each labelled
  across the top. Where one is shorter, its last frame holds.
- A shot with no counterpart in a mockup (the map's live layer, a flight
  from orbit) plays alone and is marked "no prototype".

**7. Which recording is the gate's.**
- In a cloud session, the fixed-step recording is made and published, and
  labelled "rendered at a fixed step without a GPU".
- Where smoothness is the point (the no-pop-in fade, a flight into a town),
  the gate also asks the owner to run `obs-record` on the same showcase. The
  gate is not passed on the cloud video alone.

## Risks / Trade-offs

- [lavapipe is slow enough that a three-minute recording takes hours] →
  Measured first (task 1.2). The fallback is a smaller frame, a lower rate,
  and shorter showcases split per shot group.
- [A fixed-step recording hides a hitch that real time would show] → That is
  why it is labelled, and why the pacing gates ask for the owner's OBS
  recording too.
- [Playwright's ffmpeg changes with a Playwright update] → `make_video.py`
  asks the binary what it can encode, and says clearly when it cannot, rather
  than writing a broken file.
- [Videos published only as artifacts could be lost] → The shot list and the
  scripts are committed, so any gate's video can be remade from its commit.

## Migration Plan

- There is no save or format change. The flags are new, and `--capture` and
  its callers are unchanged.
- Rollback is the previous build. The shot lists stay as data that nothing
  reads.
