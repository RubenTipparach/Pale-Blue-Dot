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
- [ ] 3.2a Side by side: `make_video.py --beside` composes a prototype sequence and a game sequence into one labelled frame. Shot lists and mockup walk files share place names from the mockup's `GOTO` list. Verify: a composed test video of the inn stair, prototype left and game right, both labelled, with the shorter side holding its last frame.
- [ ] 3.3 `tools/gate_page.py`: video, contact sheet, the shot list with times, requirements and what to look for, the render label, and a link to the live prototype. Verify: the page for 2.3's showcase is built and published, and it is linked from the PR.

## 5. Played demos (design 7; owner, 2026-11-15)

- [ ] 5.1 Showcase actions through the player's input path: move, look, hotbar select, use/dig/place, open a panel by its key and press its controls, cast and reel, eat. Verify: an app test plays a short showcase whose chop, craft and catch land in the save through the normal edit path.
- [ ] 5.2 `assets/showcase/phase-1-loop.ron`: a new world through crafting, fishing, farming, herding and cooking with the missions ticking, captioned. Verify: it runs to `SHOWCASE_END` with every mission it names done; the recording and its contact sheet.
- [ ] 5.3 `assets/showcase/phase-2-tech.ron`: one consolidated, captioned tour of environment v2 (grass and trees by biome and season, reefs and sea life now; water flow, falls, tides and rivers after the water smoothing is in the game). Verify: the recording and its contact sheet.
- [ ] 5.4 No debug readout in a showcase; audio where the platform allows. Verify: a mid-shot capture has no readout.

## 4. The rule and the owner's check

- [x] 4.1 CLAUDE.md gains "Every major step ends in a video" beside "Mockups are lit at night". Verify: done in the planning commit, with the owner's words quoted.
- [ ] 4.2 The owner watches the first gate page (2.3's showcase and the towns mockup videos) and approves the format. Verify: the quote is in `proposal.md`. Sync `platform/recording`, and archive.
