# Tasks

The first thing built in the plan, before the lights, because the lights'
gate is the first to need a video.

## 1. Frames

- [ ] 1.1 `--record <dir>` at a fixed step, taking every frame through the screenshot path, with the next frame waiting on the last one's readback. Verify: an app test that a 60-frame recording of `--tour` writes 60 numbered frames, and that two runs start each second on the same frame number.
- [ ] 1.2 Time a 10-second recording at 1280 × 720 and at 960 × 540 in the cloud session, and set the cloud default from the result. Verify: the seconds per frame are recorded in this design's risk note.

## 2. Showcases

- [ ] 2.1 The shot list format and `--showcase <name>`, reusing the walk, route and tour rigs and the player's input path for actions. It prints `SHOWCASE_START` and `SHOWCASE_END` and exits. Verify: an app test plays a two-shot showcase and checks its actions went through the edit path.
- [ ] 2.2 The caption bar. Verify: a `--capture` mid-shot shows the shot's caption.
- [ ] 2.3 A first showcase of features that already exist: walking at dusk, digging, placing a torch, and the lit tunnel at night. Verify: it records whole in the cloud.

## 3. Video, mockups and pages

- [ ] 3.1 `tools/make_video.py`: frames to WebM with Playwright's ffmpeg or MP4 elsewhere, a contact sheet, and a refusal on a short recording. Verify: a test encodes 30 synthetic frames and reads back the frame count, and refuses 29.
- [ ] 3.2 `tools/mockup_video.js` with walk files. The towns mockup's inn stair, the harbour gangplank and a night overview are the first. Verify: the three videos play, and their contact sheets show the walks.
- [ ] 3.3 `tools/gate_page.py`: video, contact sheet, the shot list with times, requirements and what to look for, and the render label. Verify: the page for 2.3's showcase is built and published, and it is linked from the PR.

## 4. The rule and the owner's check

- [x] 4.1 CLAUDE.md gains "Every major step ends in a video" beside "Mockups are lit at night". Verify: done in the planning commit, with the owner's words quoted.
- [ ] 4.2 The owner watches the first gate page (2.3's showcase and the towns mockup videos) and approves the format. Verify: the quote is in `proposal.md`. Sync `platform/recording`, and archive.
