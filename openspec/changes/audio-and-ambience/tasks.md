## 1. The skills (in the write-up's commit)

- [x] 1.1 `.claude/skills/sound-effects` from Fleet Command `08374c4`, with an
  "In Pale Blue Dot" note (kira, body-local placement, this game's groups).
- [x] 1.2 `.claude/skills/music-authoring`: Crab Breeze `60c8b2f`'s text,
  compared with Fleet Command's copy (the same method text), Fleet Command's
  lessons in the note.
- [x] 1.3 `.claude/skills/PROVENANCE.md`.

## 2. Tools and the first sounds

- [x] 2.1 Port `tools/audio/` from Fleet Command `08374c4` (`fetch_cc0.py`,
  `cut_cc0.py`, `gen_sfx.py` with its building blocks and no sounds,
  `inspect_sfx.py`, `render_sfx.sh`, `list_sources.py`, `gen_sfx_page.py`,
  `audio_page.py`), with empty `cc0_packs.json` and `cc0_cues.json`. Record
  it in `.claude/skills/PROVENANCE.md`. Verify: `render_sfx.sh` runs on the
  Mac and writes nothing. Done 2026-11-14: it printed "no cues yet ...
  nothing written" and left the tree clean. Added on the way (PROVENANCE.md
  says why): `encode_ogg.py` (the Mac's ffmpeg has no libvorbis), Freesound
  CC0 in the fetcher, `takes` in the cutter, 48 kHz.
- [x] 2.2 Find and cut the first set (design section 4 and 5): every step and
  dig family, water, the craft loops, fishing, menus, thunder, fire, and the
  beds. CC0 only. Verify:
  - `inspect_sfx.py check` passes (no clipping, no loop seam);
  - the contact sheet is in `docs/screenshots/audio/`;
  - `AUDIO_SOURCES.md` and both `SOURCES.md` are written by the tools.
  Done 2026-11-14: 117 cues and beds in 191 files (173 placed or menu cues in
  `assets/audio/sfx`, 18 stereo beds in `assets/audio/ambience`, 48 kHz OGG),
  183 cut from 91 CC0 sources (Kenney's impact and RPG packs, 89 Freesound
  sounds, each page read for Creative Commons 0 by `fetch_cc0.py`), 8
  synthesised (the menu tones, `catch_chime`, `stall_warn`) plus three layers
  (`sub_boom`, `sub_roll` under thunder, `room_tone` under the cave). Steps
  have six takes a family, digs three. The check passes for every file; the
  sheet is `docs/screenshots/audio/sfx-sheet.jpg`. Departures, each a
  recommendation taken (ask only with screenshots): beds with birds or
  insects are low passed at 8 kHz, not 3.5 (their calls live at 2 to 6 kHz);
  beds are 30 to 38 s loops; Freesound sounds are its HQ preview MP3s (band
  limited near 16 kHz); OpenGameArt answered 403 to the fetcher and was not
  used. Also made: `gull`, `owl` and `dog_far` scatters (Fauna) for the beds'
  scatters in design section 5.
- [ ] 2.3 The Sounds page (`docs/mockups/sounds.html`): every cue and bed with
  a play button, published with its MP3s as a `files` map, on the mockup
  tracker in the same step, with the tracker link near its top. Owner gate:
  the owner listens and keeps or sends back each take. The page exists
  (2026-11-14, written by `gen_sfx_page.py`, 191 files grouped by where they
  are heard, each with what it is and its source, light and dark, phone
  width), its MP3s are in `docs/mockups/sounds/` and its `files` map is
  `docs/mockups/sounds.files.json`; published 2026-11-14 at
  https://claude.ai/artifact/X85NCbpVzHwxrQNJnkHwPQ and on the tracker as M30;
  the owner's listen is still to come.

## 3. The rules (`pbd_core::audio`)

- [x] 3.1 The data types and validation: `Cue`, `Bed`, `MixConfig` (speed of
  sound, air reference, voice caps, air fade heights, underwater and cave
  filters). `assets/config/audio/cues.ron`, `ambience.ron`, `mix.ron`.
  Verify: validation tests (a cue with no file, a negative distance). Done 2026-11-15: `pbd_core::audio::{Cue, CueBook, Ambience, MixConfig}` with
  `problems()`; a `stream` flag for the long takes (build decision 3); the
  three files. Tests: a cue with no takes, a negative distance, an
  inverted range, a bad mix.
- [x] 3.2 `distance_gain`, `air_cutoff_hz` (with the behind factor),
  `delay_s`, `air_density_scale`. Verify: unit tests (6 dB a doubling, the
  10 m no-delay rule, silent above the atmosphere's top). Done 2026-11-15; tests `six_decibels_a_doubling`,
  `the_air_darkens_far_and_behind`, `no_delay_within_ten_metres`,
  `silent_above_the_atmospheres_top`, and `pan` (build decision 2).
- [x] 3.3 `surface_family(material, wet)` and the stride counter. Verify:
  every `Material` a walker can stand on has a family; 8 m at walking speed
  is ten steps. Done 2026-11-15; every material either has a family or is air or
  a lamp; 8 m at 1.4 m/s is ten steps; the walker's own speeds keep a
  cadence (`stride_m`, build decision 10).
- [x] 3.4 `bed_weights(&Surroundings)`. Verify: the meadow at noon and
  midnight, 300 m up, a cave 10 m down in rain, under water, a beach, a town,
  and equal power across a crossfade. Done 2026-11-15; noon and midnight, 300 m up, a cave 10 m down in
  rain, under water, a beach, a town, and equal power across the wind, rain
  and biome crossfades.
- [x] 3.5 `thunder(eye, strike, cloud_base_m)`: delay, roll length, take,
  level. Verify: overhead, 1 km, 3 km (8.7 s), past 6 km (none). Done 2026-11-15; overhead (crack, under 0.02 s), 1 km (2.9 s, a
  clap), 3 km (8.7 s, a roll), 6.5 km (none).
- [x] 3.6 The voice cap's choice (the quietest gives way to a louder one).
 Done 2026-11-15 (`admit`).
## 4. The engine (`pbd_app::audio`)

- [x] 4.1 `kira = "=0.12.6"` (no default features; `cpal`, `ogg`). The
  manager, the buses, the reverb send and the listener. No device: log
  `AUDIO off`, use the mock backend. `--no-audio`. Verify: the game reaches
  `READY` with no output device (a run with the device disabled) and with
  `--no-audio`. Done 2026-11-15. `kira = "=0.12.6"` with `cpal`, `ogg` and
  `vorbis` (`ogg` alone is the container: every file failed to decode until
  the codec was added; a test now decodes one). No device, or a device that
  panics on open: `AUDIO off: <why>` and the game runs silent (the mixer is
  not run at all, rather than the mock). `--no-audio` reaches `READY`
  (4.62 s, a fresh world). A run with the device disabled was not made on
  the Mac: the fallback path is the same `Err` branch.
- [ ] 4.2 The loading step `Sounds`: decode every short cue, open every bed,
  before `READY`; `LATE_SOUND <cue>`. Verify:
  - no `LATE_SOUND` in a 5-minute `--walk`;
  - the main thread's longest frame when a bed starts part-way in, measured
    with `--frame-log` (Crab Breeze's stall was 20 to 43 ms);
  - static PCM logged at `READY`, under 64 MB. Done 2026-11-15. The bank decodes on its own thread from the
  first frame; the loading screen shows TUNING THE AIR and `READY` waits on
  it. 160 files, 50.3 MB static (under 64), in 0.04 s on the M4 Max; the
  long takes (thunder, the long loops, 126 s of sound) stream (build
  decision 3). No `LATE_SOUND` in a 5-minute `--walk-distance` walk. Beds
  open as streams when first wanted (header only on the main thread); the
  `--frame-log` measurement of a bed starting is still to make, so the
  task stays open.
- [x] 4.3 The listener from the active camera; voices as spatial sub-tracks
  at `emitter - eye`, kira's falloff off, gain and cutoff from 3.2 each frame,
  loops kept alive by asking and faded when not. Verify: an origin-rebase
  test with the mock backend (the same gain and pan either side). Done 2026-11-15. Our own pan and gain (build decision 2); test
  `an_origin_rebase_moves_no_sound`.
- [x] 4.4 Under water, shelter and cave depth drive `world`'s filter and the
  reverb send. Pause, the map's duck, and focus loss. Done 2026-11-15. Under water: `world` to 600 Hz and -9 dB, the
  reverb send up; underground: the reverb send by depth; sheltered: the
  ambience bus to 2.5 kHz and -6 dB; pause (pause, settings, saves) pauses
  `world`; a screen over play ducks it 6 dB under 4 kHz; focus lost mutes
  `world` and `music`.
- [ ] 4.5 A device change mid-play (headphones out): the manager is rebuilt on
  the same buses; no crash. Verify on the Mac.

  Kira 0.12's cpal backend restarts its own stream when the default device
  changes or drops (it checks every 0.5 s); the game logs the device's
  error when one is reported. Not yet verified by pulling headphones out.
## 5. Event sounds

- [x] 5.1 Steps, jumps and landings from `walking.rs`. Done 2026-11-15 (`hooks::walker_sounds`; build decision 10's
  cadence). A walk logged `step_sand`, `step_stone`, `step_dry`,
  `step_wet` under rain, jumps, landings and `land_hard`.
- [x] 5.2 Dig, break, place and pickup from `desktop/digging.rs` and
  `drops.rs`. Done 2026-11-15 (`desktop/sounds.rs`): a dig at the start and at
  each third, a break on `Did(Gather)`, a felled tree `break_wood`, a place
  by `place_cue`; `pickup` on every `Gained`.
- [x] 5.3 Water: enter, exit, strokes, bubbles, breath. Done 2026-11-15 (from the walker and the eye's water). A `--swim`
  capture with `--audio-log` logged `water_enter_big`, `step_shallow`,
  `land_shallow` and `water_exit`; strokes and bubbles are not yet seen in
  a log (the script's swim is short).
- [ ] 5.4 Craft loops and one-shots from each `Telemetry`; the 1 kHz
  structure filter above the air.
  Waits for the owner's first listen (build decision 9).
- [x] 5.5 Fishing from `pbd_core::fishing::Event`. Done 2026-11-15 (`fish::FishHeard`; the reel loop while hooked).
- [x] 5.6 Menus and HUD. Done 2026-11-15: every `Button`'s hover and press, the screens'
  toggles and back, the hotbar slot.
- [x] 5.7 A test that every cue the code names is in `cues.ron`, and a check
  that every file exists and every placed file is mono.
 Done 2026-11-15: `every_cue_the_code_names_is_in_the_book` and
  `every_file_exists_and_placed_files_are_mono` (beds stereo, every bed
  with a level).
## 6. Ambience, thunder and fire

- [x] 6.1 The `Surroundings` read at 4 Hz and the beds played from 3.4, with
  scatters. Done 2026-11-15, with gull, owl and dog scatters.
- [x] 6.2 Thunder from `Air.now.strikes` by 3.5; streamed takes with a
  pre-roll from the strike. Done 2026-11-15, with build decision 11 (the nearest new strike
  of a step, far rolls 4 s apart). Streamed takes, dated from the strike's
  step.
- [ ] 6.3 Port the sound step of Crab Breeze's `obs_record.py` (macOS
  `sck_audio_capture`, added for the take and removed after) into
  `.claude/skills/obs-record`, with `PBD_TAKE_SOUND=1`.
- [ ] 6.4 When `lightning-fire` lands: the strike-hit cues, and ignite,
  crackle, hiss and out from its fire state (design section 6's interface).

  Meanwhile campfires and braziers crackle from the column tier's lamps
  (build decision 7).
## 7. Settings

- [x] 7.1 `settings.rs` version 3: `volumes` with the defaults; a version 2
  file reads with them. `PBD_TAKE_SOUND=1`, `--capture` and `--frame-log` use
  the defaults. Verify: the round-trip and old-file tests, extended. Done 2026-11-15 at version 5 (build decision 1): `Volumes`; a
  version 4 file reads the defaults; captures, benchmarks and
  `PBD_TAKE_SOUND=1` take the defaults. Test
  `a_file_from_before_volumes_reads_the_defaults_and_keeps_the_rest`.
- [x] 7.2 Five rows on the settings screen, stepped by 10, heard at once.
  Verify: a capture of the screen in `docs/screenshots/audio-and-ambience/`.
 Done 2026-11-15: a SOUND tab (the tabs narrowed to 5 px a side to
  fit six), `docs/screenshots/audio-and-ambience/settings-sound.png`.
## 8. Instruments and cost

- [ ] 8.1 `--audio-log` and the bus meters (a custom kira effect writing peak
  and RMS to atomics); the meters on the F3 graph.
  `--audio-log` and the meters (a kira effect per bus, peak and RMS in
  atomics, logged each second) are built; the meters on the F3 graph are
  not.
- [x] 8.2 `--audio-wav PATH`: a tap on `main`; under `--capture`, the mock
  backend advanced one frame's samples per frame with fixed dice. Verify: two
  runs give identical files. Done 2026-11-15. The mock mixer advances from `READY`; what is
  otherwise streamed is decoded whole under a capture. Two `--walk
  --frames 240` captures gave the same samples over their common length
  (3.5 s); the files differ in length by the few frames a capture runs
  after its last counted one.
- [x] 8.3 A storm walk with `--audio-log`: steps every stride, thunder delays
  matching the strikes' distances, the beds' weights. Checked in as a log
  excerpt beside the change. Done 2026-11-15: `storm-walk.log` beside this file.
- [ ] 8.4 `tools/perf_suite.py`, sound on against `--no-audio`, interleaved,
  on the `--release` build; the report in `docs/benchmarks/<date>-audio/`,
  the HTML page published and on the tracker. Audio thread under 10 % of a
  core, main thread under 0.1 ms a frame.

## 9. Close

- [ ] 9.1 The step's video: a real-time take with sound on the Mac
  (`obs-record`, `PBD_TAKE_SOUND=1`): a meadow at noon, dusk into night, a
  beach, a dig into a cave, a swim, a Kestrel climb above the air, a storm
  with a near and a far strike. Each shot names the requirement it shows. For
  now (screenshots stand in) the take waits for the owner's batch; the
  `--audio-wav` mux of a `--capture` run is made with the change.
- [ ] 9.2 Move the requirements into `openspec/specs/platform/audio/` and
  `openspec/specs/platform/settings/` as each is true and tested;
  `openspec validate --all`.
