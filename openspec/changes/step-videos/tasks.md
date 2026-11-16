# Tasks

The first thing built in the plan, before the lights, because the lights'
gate is the first to need a video.

## 1. Frames

- [ ] 1.1 `--record <dir>` at a fixed step, taking every frame through the screenshot path, with the next frame waiting on the last one's readback. Verify: an app test that a 60-frame recording of `--tour` writes 60 numbered frames, and that two runs start each second on the same frame number.
- [ ] 1.2 Time a 10-second recording at 1280 × 720 and at 960 × 540 in the cloud session, and set the cloud default from the result. Verify: the seconds per frame are recorded in this design's risk note.

## 2. Showcases

- [x] 2.1 The shot list format and `--showcase <name>`, reusing the walk, route and tour rigs and the player's input path for actions. It prints `SHOWCASE_START` and `SHOWCASE_END` and exits. Verify: an app test plays a two-shot showcase and checks its actions went through the edit path.
  - Built as a bot rather than a rig (design 7): `desktop/showcase.rs` and `showcase/file.rs`. A shot has a caption, the missions it must finish (`until`), `repeat`, `max_s` and the clock's `rate`; its acts are `Wait`, `Look`, `Walk`, `Hand`, `Select`, `Chop`, `Dig`, `Craft`, `Place`, `Bench`, `Cook`, `Shore`, `Fish`, `Till`, `Plant`, `Pick`, `Harvest`, `Tame`, `Pen`, `Lead`, `Collect`, `Eat`, `Missions` and `Still`. It also prints `SHOT <n> <caption>`, `SHOT_DONE`, and `SHOWCASE_STALL` (no `SHOWCASE_END`, exit code 1) when a shot runs past `max_s` or ends with its missions not done.
  - Verified by `showcase::tests::a_short_showcases_chop_and_craft_land_in_the_save`: a two-shot showcase on the real ground at the fields spawn; the bot's held button on a trunk goes through `dig_and_place` and its I, drop-down and CRAFT through `pack::open` and `craft::press`; the tree's record, the hands' queue and the planks are in the save. `file::tests` parse every file in `assets/showcase`.
- [x] 2.2 The caption bar. Verify: a `--capture` mid-shot shows the shot's caption.
  - Verified by the run's stills (`--showcase-stills`, below) instead of a `--capture`, since a showcase runs in real time: each shows its caption over the hotbar.
- [ ] 2.3 A first showcase of features that already exist: walking at dusk, digging, placing a torch, and the lit tunnel at night. Verify: it records whole in the cloud.

## 3. Video, mockups and pages

- [ ] 3.1 `tools/make_video.py`: frames to WebM with Playwright's ffmpeg or MP4 elsewhere, a contact sheet, and a refusal on a short recording. Verify: a test encodes 30 synthetic frames and reads back the frame count, and refuses 29.
- [ ] 3.2 `tools/mockup_video.js` with walk files. The towns mockup's inn stair, the harbour gangplank and a night overview are the first. Verify: the three videos play, and their contact sheets show the walks.
- [ ] 3.2a Side by side: `make_video.py --beside` composes a prototype sequence and a game sequence into one labelled frame. Shot lists and mockup walk files share place names from the mockup's `GOTO` list. Verify: a composed test video of the inn stair, prototype left and game right, both labelled, with the shorter side holding its last frame.
- [ ] 3.3 `tools/gate_page.py`: video, contact sheet, the shot list with times, requirements and what to look for, the render label, and a link to the live prototype. Verify: the page for 2.3's showcase is built and published, and it is linked from the PR.

## 5. Played demos (design 7; owner, 2026-11-15)

- [x] 5.1 Showcase actions through the player's input path: move, look, hotbar select, use/dig/place, open a panel by its key and press its controls, cast and reel, eat. Verify: an app test plays a short showcase whose chop, craft and catch land in the save through the normal edit path.
  - Keys and buttons are pressed in `ButtonInput` in `PreUpdate` after the input systems; the look turns by `AccumulatedMouseMotion` at a hand's rate; slots by the number keys, tools by G and the wheel, items from the pack by its own pick-up and drop; a panel's button by its `Interaction` after the UI's focus pass. Nothing sets an item, a count or a mission.
  - The app test covers the chop and the craft. The catch is verified in the game run (the bot's cast, hook and reel landing three fish, `catch-fish` done), not in the app test: the fishing system needs the sea, its schools and the water rule, which the test app does not build.
- [ ] 5.2 `assets/showcase/phase-1-loop.ron`: a new world through crafting, fishing, farming, herding and cooking with the missions ticking, captioned. Verify: it runs to `SHOWCASE_END` with every mission it names done; the recording and its contact sheet.
  - The bot's goal follows the mission tree (`starter-missions` T.5): the trunk, then fishing, farming, herding and cooking, each section opened by a caption naming its goal; sped-up waits are captioned ("Six days later: the wheat is ripe"). It starts on a level pasture by the sea (`--at 11.5085 -25.0992`, measured by `showcase::tests::find_a_coastal_field`), in a new world kept in memory unless `--world` names one; `--showcase-from N` resumes a `--world` at shot N (a development aid).
  - Played on the old line (before the tree), one clean run did all fourteen missions it could, `gather` to `pen`, each closing in the log ("mission done: ..."), with the fish caught, the fish grilled, the hen tamed and penned.
  - On the tree, a clean run plays the trunk and fishing whole (`gather`, `axe`, `wood`, `planks-rope`, `crafting-table`, `stone-tools`, `rod`, `go-fishing`, about 11 minutes) and farming's `find-seeds`; its stills, taken at the moments of use (the pack's Craft row with the stone axe, the bench making the pickaxe, a fish on the line with the tension bar), are in `docs/screenshots/step-videos/`. Farming, herding and cooking were each played through in resumed runs (`--showcase-from`), but not yet in one run: the sowing, and the hen across the hills at this start, still stall it some runs.
  - What the bot found in the game, fixed here: a block could not be set on level ground beside the player (the body test looked at the layer and not the cell); a campfire could not be opened with E (a light is passed by the eye ray, so E on the ground under it now opens it). Not fixed: there is no ore in any generated ground, so `smelt` and the mining branch cannot be played (`starter-missions` H.3).
  - Still open: the tree run to `SHOWCASE_END`, and the OBS recording and its contact sheet (the main session films it).
- [ ] 5.3 `assets/showcase/phase-2-tech.ron`: one consolidated, captioned tour of environment v2 (grass and trees by biome and season, reefs and sea life now; water flow, falls, tides and rivers after the water smoothing is in the game). Verify: the recording and its contact sheet.
- [x] 5.4 No debug readout in a showcase; audio where the platform allows. Verify: a mid-shot capture has no readout.
  - The near-field and position lines are hidden in a showcase; the stills show none. A showcase runs windowed in real time with the game's sound on, so OBS records it with audio.

## 4. The rule and the owner's check

- [x] 4.1 CLAUDE.md gains "Every major step ends in a video" beside "Mockups are lit at night". Verify: done in the planning commit, with the owner's words quoted.
- [ ] 4.2 The owner watches the first gate page (2.3's showcase and the towns mockup videos) and approves the format. Verify: the quote is in `proposal.md`. Sync `platform/recording`, and archive.
