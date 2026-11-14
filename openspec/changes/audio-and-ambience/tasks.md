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
- [ ] 2.2 Find and cut the first set (design section 4 and 5): every step and
  dig family, water, the craft loops, fishing, menus, thunder, fire, and the
  beds. CC0 only. Verify:
  - `inspect_sfx.py check` passes (no clipping, no loop seam);
  - the contact sheet is in `docs/screenshots/audio/`;
  - `AUDIO_SOURCES.md` and both `SOURCES.md` are written by the tools.
- [ ] 2.3 The Sounds page (`docs/mockups/sounds.html`): every cue and bed with
  a play button, published with its MP3s as a `files` map, on the mockup
  tracker in the same step, with the tracker link near its top. Owner gate:
  the owner listens and keeps or sends back each take.

## 3. The rules (`pbd_core::audio`)

- [ ] 3.1 The data types and validation: `Cue`, `Bed`, `MixConfig` (speed of
  sound, air reference, voice caps, air fade heights, underwater and cave
  filters). `assets/config/audio/cues.ron`, `ambience.ron`, `mix.ron`.
  Verify: validation tests (a cue with no file, a negative distance).
- [ ] 3.2 `distance_gain`, `air_cutoff_hz` (with the behind factor),
  `delay_s`, `air_density_scale`. Verify: unit tests (6 dB a doubling, the
  10 m no-delay rule, silent above the atmosphere's top).
- [ ] 3.3 `surface_family(material, wet)` and the stride counter. Verify:
  every `Material` a walker can stand on has a family; 8 m at walking speed
  is ten steps.
- [ ] 3.4 `bed_weights(&Surroundings)`. Verify: the meadow at noon and
  midnight, 300 m up, a cave 10 m down in rain, under water, a beach, a town,
  and equal power across a crossfade.
- [ ] 3.5 `thunder(eye, strike, cloud_base_m)`: delay, roll length, take,
  level. Verify: overhead, 1 km, 3 km (8.7 s), past 6 km (none).
- [ ] 3.6 The voice cap's choice (the quietest gives way to a louder one).

## 4. The engine (`pbd_app::audio`)

- [ ] 4.1 `kira = "=0.12.6"` (no default features; `cpal`, `ogg`). The
  manager, the buses, the reverb send and the listener. No device: log
  `AUDIO off`, use the mock backend. `--no-audio`. Verify: the game reaches
  `READY` with no output device (a run with the device disabled) and with
  `--no-audio`.
- [ ] 4.2 The loading step `Sounds`: decode every short cue, open every bed,
  before `READY`; `LATE_SOUND <cue>`. Verify:
  - no `LATE_SOUND` in a 5-minute `--walk`;
  - the main thread's longest frame when a bed starts part-way in, measured
    with `--frame-log` (Crab Breeze's stall was 20 to 43 ms);
  - static PCM logged at `READY`, under 64 MB.
- [ ] 4.3 The listener from the active camera; voices as spatial sub-tracks
  at `emitter - eye`, kira's falloff off, gain and cutoff from 3.2 each frame,
  loops kept alive by asking and faded when not. Verify: an origin-rebase
  test with the mock backend (the same gain and pan either side).
- [ ] 4.4 Under water, shelter and cave depth drive `world`'s filter and the
  reverb send. Pause, the map's duck, and focus loss.
- [ ] 4.5 A device change mid-play (headphones out): the manager is rebuilt on
  the same buses; no crash. Verify on the Mac.

## 5. Event sounds

- [ ] 5.1 Steps, jumps and landings from `walking.rs`.
- [ ] 5.2 Dig, break, place and pickup from `desktop/digging.rs` and
  `drops.rs`.
- [ ] 5.3 Water: enter, exit, strokes, bubbles, breath.
- [ ] 5.4 Craft loops and one-shots from each `Telemetry`; the 1 kHz
  structure filter above the air.
- [ ] 5.5 Fishing from `pbd_core::fishing::Event`.
- [ ] 5.6 Menus and HUD.
- [ ] 5.7 A test that every cue the code names is in `cues.ron`, and a check
  that every file exists and every placed file is mono.

## 6. Ambience, thunder and fire

- [ ] 6.1 The `Surroundings` read at 4 Hz and the beds played from 3.4, with
  scatters.
- [ ] 6.2 Thunder from `Air.now.strikes` by 3.5; streamed takes with a
  pre-roll from the strike.
- [ ] 6.3 Port the sound step of Crab Breeze's `obs_record.py` (macOS
  `sck_audio_capture`, added for the take and removed after) into
  `.claude/skills/obs-record`, with `PBD_TAKE_SOUND=1`.
- [ ] 6.4 When `lightning-fire` lands: the strike-hit cues, and ignite,
  crackle, hiss and out from its fire state (design section 6's interface).

## 7. Settings

- [ ] 7.1 `settings.rs` version 3: `volumes` with the defaults; a version 2
  file reads with them. `PBD_TAKE_SOUND=1`, `--capture` and `--frame-log` use
  the defaults. Verify: the round-trip and old-file tests, extended.
- [ ] 7.2 Five rows on the settings screen, stepped by 10, heard at once.
  Verify: a capture of the screen in `docs/screenshots/audio-and-ambience/`.

## 8. Instruments and cost

- [ ] 8.1 `--audio-log` and the bus meters (a custom kira effect writing peak
  and RMS to atomics); the meters on the F3 graph.
- [ ] 8.2 `--audio-wav PATH`: a tap on `main`; under `--capture`, the mock
  backend advanced one frame's samples per frame with fixed dice. Verify: two
  runs give identical files.
- [ ] 8.3 A storm walk with `--audio-log`: steps every stride, thunder delays
  matching the strikes' distances, the beds' weights. Checked in as a log
  excerpt beside the change.
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
