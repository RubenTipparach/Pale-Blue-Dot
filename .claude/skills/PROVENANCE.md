# Skill provenance

Where each skill copied into this directory came from, so a later copy from the source can be
compared. A copied skill keeps its source's text below an "In Pale Blue Dot" section at the top of
its `SKILL.md`, and that section wins where they differ.

## UI skills from Fleet Command (2026-11-14)

The owner, 2026-11-14: "pull in UI skills from my fleet-command repo, and develop ui for this game,
do something like with rock and carved wood". Copied from `RubenTipparach/fleet-command` at
`a4a02f6` (2026-11-13), every file each skill carries, unchanged below the new section; only the
frontmatter `description` was rewritten so the skill triggers for Pale Blue Dot work. Fleet
Command's own `PROVENANCE.md` row for each is quoted in the last column.

| Skill | Files | Upstream origin (from Fleet Command's PROVENANCE.md) | Licence | Adapted here |
| --- | --- | --- | --- | --- |
| `game-ui-frontend` | `SKILL.md`, `references/frontend-prompts.md`, `references/playtest-checklist.md`, `references/three-hud-layout-patterns.md` | OpenAI `openai/plugins`, `plugins/game-studio/skills/game-ui-frontend` and three files of `plugins/game-studio/references/` at `0722921`; the Codex `agents/openai.yaml` left out upstream | MIT (OpenAI's plugins repository; no licence file was carried by Fleet Command's copy) | Bevy UI in `crates/pbd-app/src/desktop/`; Fleet Command's console column does not apply, the quiet screen does; stone-and-wood look |
| `ui-ux-pro-max` | `SKILL.md`, `LICENSE`, `data/` (17 CSV and JSON files plus 22 stack CSVs, about 3.7 MB, including Google Fonts metadata and licences and the Phosphor icon list), `references/pro-rules.md`, `references/quick-reference.md`, `scripts/{core,design_system,reasoning_contract,search,validate_data}.py` | NextLevelBuilder `nextlevelbuilder/ui-ux-pro-max-skill`, `.claude/skills/ui-ux-pro-max` v2.15.0 at `28b3b8d`; its `scripts/tests/` left out upstream | MIT, `LICENSE` beside it (Copyright 2024 Next Level Builder); the font and icon data keep their own licences as recorded in `data/google-font-licenses.json` and `data/data-provenance.json` | Search from the repo root works here (Python 3.13, standard library); no Bevy stack; `--design-system` returns SaaS answers for a game and is not used |
| `blender-ui-panels` | `SKILL.md` | Written in Fleet Command (2026-10-30) from a lighting study of a 1999 music-player skin; its tools (`tools/blender/ui_shapes.py`, `ui_render.py`, `panel_kit.py`, `tools/mockups/formed_metal.py`) and the Analog Panel Bible stay there | Fleet Command's own | Method kept (model, light top left, render face-on, measure fit); backlit film and lamps dropped; here `tools/art/ui_panels_blend.py` with the `bpy` 4.5 wheel |
| `blender-organic-panels` | `SKILL.md` | Written in Fleet Command (2026-10-31) for the Vhorr Brood's console (`tools/blender/organic_study.py` and the Organic Panel Bible stay there) | Fleet Command's own | Height field with material masks, no primitives, judged over the game; flesh dropped; rule 4 (no 9-slice) does not hold here |
| `panel-textures` | `SKILL.md` | Star Crew Bridge Simulator's textures skill (`8b0bbf1`), retargeted by Crab Breeze (`69ce196`) to a mech cockpit, then by Fleet Command to hull plating and console faces; its kit (`tools/blender/panel_kit.py`, `tools/fonts/`) stays there | The owner's repositories | The 9-slice rule; one RGBA pixel-art image per piece instead of PBR maps; the kit is `tools/art/ui_art.py` |

What runs here (checked 2026-11-14 in a Claude Code cloud container, no GPU):

- `python3 .claude/skills/ui-ux-pro-max/scripts/search.py "<query>" --domain <d>`: works.
- Blender: no Blender install; `python3.11 -m venv <dir> && <dir>/bin/pip install "bpy==4.5.*"
  pillow numpy` gives Blender 4.5.16 LTS as a module, and Cycles renders on the CPU
  (`tools/art/ui_panels_blend.py`, about 3 s a panel at 24 samples).
- The Fleet Command tools these skills name are not copied, so their commands do not run here.

## Audio skills and tools (2026-11-14)

Copied for `audio-and-ambience`. The skills here before then were written in this repository or came with the OpenSpec CLI. The sources were read, never changed.

| Skill | Source | Revision | Copied | Changes here |
| --- | --- | --- | --- | --- |
| `sound-effects` | Fleet Command, `.claude/skills/sound-effects/SKILL.md` (itself new there, after Crab Breeze's `tools/audio/` and its CLAUDE.md sound rule; Crab Breeze has no sound skill) | Fleet Command `08374c4` (branch `claude/planet-bakes`) | 2026-11-14 | New frontmatter and an "In Pale Blue Dot" note: kira not Godot or rodio (so no muffled or far copies), body-local placement in the camera's frame, this game's cue groups, Mac setup. `tools/audio/` ported from the same revision in `audio-and-ambience` task 2.1 (Tools, below). |
| `music-authoring` | Crab Breeze, `.claude/skills/music-authoring/SKILL.md` (after tom-lander `dea3d50`), compared with Fleet Command's copy | Crab Breeze `60c8b2f` (branch `fortress-nuke`); Fleet Command `08374c4` | 2026-11-14 | The two method texts are identical; Crab Breeze's is kept verbatim. Fleet Command's added lessons (`@loop`, the 15-instrument cap, calibration, one render per command, publishing with a `files` map) are in the note. No tools copied: the first music change ports `tools/music/` from Fleet Command `08374c4`. |

### Tools

| Tool | Source | Revision | Copied | Changes here |
| --- | --- | --- | --- | --- |
| `tools/audio/` (`fetch_cc0.py`, `cut_cc0.py`, `gen_sfx.py`, `inspect_sfx.py`, `render_sfx.sh`, `list_sources.py`, `gen_sfx_page.py`, `audio_page.py`, `licence_exceptions.json`) | Fleet Command, `tools/audio/` (itself after Crab Breeze `0e569e3`) | Fleet Command `08374c4` (branch `claude/planet-bakes`) | 2026-11-14 (`audio-and-ambience` 2.1) | Names and paths. 48 kHz, not 44.1. `gen_sfx.py` keeps the building blocks and none of Fleet Command's sounds; a fixed-cutoff filter and the brown noise run through scipy's `lfilter` with the loops' own coefficients (the same output to 1e-13, measured), the reverb's combs and allpasses run a block of their delay at a time (identical output, measured), and `cut_cc0.py`'s moving average runs through an FFT (the same to 1e-17), because a 40 s stereo bed through the loops took minutes; a partial `gen_sfx.py --only` keeps the rest of the manifest (Fleet Command's overwrote it, so a partial rebuild emptied the Sounds page). `fetch_cc0.py`'s Kenney fetch asks Kenney's `download-link` endpoint with the page's cookie (the site no longer puts the zip in the page) and refuses a page without a CC0 link. `inspect_sfx.py` labels and tiles the contact sheet with Pillow (the Mac's ffmpeg has no `drawtext`), and judges a loop seam against the steps right at the join as well as the file's median (a loud frog chorus at the join read as a seam). `fetch_cc0.py` adds Freesound (`freesound/<id>`, refused unless its page links CC0 or the public domain mark; the HQ preview MP3 is what is fetched) beside OpenGameArt, Kenney and the Internet Archive; originals go to `output/sfx-src` (git-ignored), not `/opt/sfx-src`. `cut_cc0.py` adds `takes` (one cue written as `<cue>_1 .. _N`) and per-file sources in the manifest. `inspect_sfx.py` reads each WAV's rate and has no `NO_RING` caps. The OGG encode is a new `encode_ogg.py` (libvorbis through libsndfile, a fixed stream serial so a rebuild is byte-identical), because the Mac's Homebrew ffmpeg has no libvorbis. The Sounds page's MP3s sit beside it in `docs/mockups/sounds/` (committed, relative paths), not in `output/sfx`, and each row prints its sources. `cc0_packs.json`, `cc0_cues.json` and `licence_exceptions.json` started empty: no Fleet Command pack, cue or licence exception carried over. |

Read and not copied:
- `crab-breeze-wip-title-screen`: a title-screen change (`title-screen/`,
  `title.rs`) and no skills. The music skill was taken from Crab Breeze's main
  checkout instead.
- Crab Breeze's game code (`crates/cb-app/src/audio.rs`, `ambience.rs`,
  `music.rs`; `crates/cb-core/src/audio.rs`; `crates/cb-tools/src/audio_check.rs`)
  and its `openspec/changes/audio`, `ambience` and `music`, at `60c8b2f`: read
  for the design of `audio-and-ambience` (the rules-in-core split, cue data,
  the speed-of-sound delay, the voice cap, equal-power bed crossfades, the
  data-against-files check, `CB_TAKE_SOUND`). No code copied.

## Recorded in the skills themselves

- `balatro-pixel-art`: `RubenTipparach/rubens-solitare-collection` at `cdb3b24` (2026-11-12), its
  "In this repository (Pale Blue Dot)" note.
- `mockup-tracker`: Fleet Command at `ebcb205` (2026-11-12), its "In Pale Blue Dot" section.
- The others carry their history in their own `SKILL.md`.
