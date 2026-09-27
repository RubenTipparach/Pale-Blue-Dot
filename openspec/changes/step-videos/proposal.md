# Proposal: a video at every gate

## Why

**The owner (2026-09-27): "each major step needs a video recording showcasing
this stuff so I can verify they are done correctly, before moving on to the
next step."**

Every step of the cities plan ends at an approval gate. Until now a gate
offered a mockup or a set of `--capture` stills. A still shows a lantern at
midnight, but it cannot show:
- the lanterns coming on at dusk;
- a town fading in as a ship approaches;
- a stair climbed without a jolt;
- a save reloaded with every edit in place.

Those are the things the gates exist to check.

The repository can already record, in two ways that do not meet:
- **On the owner's machine:** `.claude/skills/obs-record` records a game
  window with OBS in real time and makes an MP4. It recorded the far-side
  flight. It needs Windows, OBS and a GPU.
- **In a cloud session:** the game renders headless on lavapipe, with no GPU.
  `--capture` saves one PNG after N fixed steps, and `--route` and `--tour`
  fly scripted paths. Nothing turns a run into a video. The only ffmpeg is
  Playwright's (VP8 WebM from JPEG frames), and there is no OBS.

## What Changes

- **A showcase is a shot list the game plays by itself.** `--showcase <name>`
  plays `assets/showcase/<name>.ron`. Each shot has:
  - a place to stand, walk or fly;
  - the time of day and how fast it runs;
  - a camera move;
  - actions (place a lantern, dig, open a door, age the world);
  - a caption burned into the frame naming what the shot shows.

  The same shot list serves both recorders.
- **`--record <dir>`** writes every rendered frame at a fixed simulation step,
  numbered. `tools/make_video.py` turns the frames into a video:
  - VP8 WebM with Playwright's ffmpeg in the cloud;
  - H.264 MP4 with imageio-ffmpeg or a system ffmpeg on the owner's machine.

  It also writes a contact sheet.
- **On the owner's machine**, `obs-record` runs the same showcase in real time.
  The showcase prints start and end lines for the skill's `--start-on` and
  `--stop-on`. That recording is the one that shows smoothness. A cloud
  recording shows what things look like, and is labelled as such.
- **Mockup gates are recorded from the mockup.** `tools/mockup_video.js`
  drives a scripted walkthrough of an HTML mockup in headless Chromium and
  records it with Playwright.
- **A gate page** for each step, published as an artifact and linked from the
  PR and the roadmap page. It holds:
  - the video;
  - the contact sheet;
  - the shot list, with each shot's time, what to look for, and the
    requirement it demonstrates;
  - where it was rendered.
- **CLAUDE.md records the rule** beside "Mockups are lit at night": every
  major step ends in a video the owner watches before the next step starts.

## Capabilities

### New Capabilities
- `platform/recording`: showcases (scripted shot lists the game plays
  unattended), fixed-step frame recording, and the rule that a recording's
  shots name what they demonstrate.

### Modified Capabilities
- None in the main specs. Every change in the cities plan gains a video task
  at its gate. Those are task changes, not requirement changes.

## Impact

- **`pbd-app`:** `--showcase` and `--record` in `desktop.rs`, beside
  `--capture`, `--route` and `--tour`, reusing the existing screenshot path.
  Also a caption bar in `bevy_ui`, and a showcase player that reuses the
  route and tour rigs for flights and walks.
- **Assets:** `assets/showcase/*.ron`, one per gate, written with each change.
- **Tools:**
  - `tools/make_video.py` (frames to video and a contact sheet);
  - `tools/mockup_video.js` (mockup walkthroughs);
  - `tools/gate_page.py` (the gate page from a shot list and a video).
- **Repository size:** videos are not committed, because they are several to
  tens of megabytes each and the repository has no LFS. The shot list, the
  scripts and the contact sheet are committed. The video is published with
  the gate page, and on the owner's machine it goes to Drive as `obs-record`
  already does.
- **Time:** a cloud recording renders every frame on lavapipe. Its cost is
  measured by the first task, before any gate depends on it.
