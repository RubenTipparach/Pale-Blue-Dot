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

## Recorded in the skills themselves

- `balatro-pixel-art`: `RubenTipparach/rubens-solitare-collection` at `cdb3b24` (2026-11-12), its
  "In this repository (Pale Blue Dot)" note.
- `mockup-tracker`: Fleet Command at `ebcb205` (2026-11-12), its "In Pale Blue Dot" section.
- The others carry their history in their own `SKILL.md`.
