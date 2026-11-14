# Skill provenance

Where a skill in this directory came from, so a later copy from its source can
be compared. Started 2026-11-14 with the audio skills; the skills here before
then were written in this repository or came with the OpenSpec CLI, and are not
listed. Every skill copied from another of the owner's repositories keeps the
source's text verbatim below an "In Pale Blue Dot" note, and the note wins where
they differ. The sources were read, never changed.

| Skill | Source | Revision | Copied | Changes here |
| --- | --- | --- | --- | --- |
| `sound-effects` | Fleet Command, `.claude/skills/sound-effects/SKILL.md` (itself new there, after Crab Breeze's `tools/audio/` and its CLAUDE.md sound rule; Crab Breeze has no sound skill) | Fleet Command `08374c4` (branch `claude/planet-bakes`) | 2026-11-14 | New frontmatter and an "In Pale Blue Dot" note: kira not Godot or rodio (so no muffled or far copies), body-local placement in the camera's frame, this game's cue groups, Mac setup. `tools/audio/` is not copied yet: `audio-and-ambience` task 2.1 ports it from the same revision, with empty pack and cue lists. |
| `music-authoring` | Crab Breeze, `.claude/skills/music-authoring/SKILL.md` (after tom-lander `dea3d50`), compared with Fleet Command's copy | Crab Breeze `60c8b2f` (branch `fortress-nuke`); Fleet Command `08374c4` | 2026-11-14 | The two method texts are identical; Crab Breeze's is kept verbatim. Fleet Command's added lessons (`@loop`, the 15-instrument cap, calibration, one render per command, publishing with a `files` map) are in the note. No tools copied: the first music change ports `tools/music/` from Fleet Command `08374c4`. |

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
