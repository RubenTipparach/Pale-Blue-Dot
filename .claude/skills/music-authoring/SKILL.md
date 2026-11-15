---
name: music-authoring
description: Write, arrange, render and mix Pale Blue Dot's music the repository's way, after Tom Lander's pipeline as Crab Breeze rebuilt it and Fleet Command extended it (read the In Pale Blue Dot note first). Motifs and themes are written down first as notes (docs/design/music.md and the MOTIF LIBRARY block of music/motifs.strudel); a cue is a Strudel score whose layers name General MIDI instruments or synths; tools/music exports it to multitrack MIDI and an LMMS project, LMMS renders it headless with real instruments into the final comp, loudness-normalised to OGG for the game and MP3 to share. Use whenever making or changing music, a motif, a theme, a cue, a stinger, a soundtrack, a MIDI file, an LMMS project or a soundfont, or mixing instruments ("make music", "write a motif", "title theme", "night music", "render the soundtrack", "export MIDI", "LMMS", "soundfont").
metadata:
  author: Pale Blue Dot (Claude Code), from Crab Breeze's music-authoring with Fleet Command's additions
  version: "1.0"
---

## In Pale Blue Dot

The owner, 2026-11-14: "using skills from fleet command and crab breeze - spin up a audio sfx and ambient system
session". The two copies were compared on 2026-11-14: Crab Breeze's (`.claude/skills/music-authoring` at
`60c8b2f`) and Fleet Command's (at `08374c4`) carry the same method text word for word; Fleet Command's adds
lessons learned rendering its cues, kept below. So this is Crab Breeze's text, verbatim under the line, with
Fleet Command's lessons in this note. (`crab-breeze-wip-title-screen` holds only a title-screen change and
no skills.) Provenance in `.claude/skills/PROVENANCE.md`. Where the text and this note differ, this note wins.

- **Nothing here yet.** Pale Blue Dot has no `music/`, no `tools/music/` and no `docs/design/music.md`. The
  first music change ports `tools/music/` (from Fleet Command `08374c4`, which has `fold_loop.py` and
  `music_page.py` beside Crab Breeze's set) and `music/instruments.json`, and writes the motifs first.
- **Direction is not chosen.** Crab Breeze's moody cyberpunk and Fleet Command's orchestral space opera are
  those games'. Pale Blue Dot's is a question for the owner with something to listen to beside it (a short
  sketch of two directions on the Music page), per "Ask the owner with a survey". Build on a quiet, sparse
  recommendation until then: a small planet, walked and flown alone.
- **Where music plays.** `openspec/changes/audio-and-ambience` reserves the music bus and its volume
  setting; it does not choose or schedule any music. A music change does that.
- **Fleet Command's lessons, kept:**
  - `// @loop` in a score's header makes `render.sh` cut the render at its scored length and fold the tail
    onto the start (`tools/music/fold_loop.py`), for cues the game loops.
  - A score has at most 15 melodic instruments (MIDI's 16 channels, one for drums):
    `strudel_to_midi.mjs` refuses more.
  - New instruments are calibrated with `tools/music/calibrate.sh --write`; keep the earlier values of
    instruments already in `music/instruments.json` (they move by measurement noise) so existing cues render
    unchanged.
  - Render one score per command: a `render.sh` piped into `head` can stall LMMS on a closed pipe.
  - Publish the Music page with its tracks beside it (`docs/mockups/music.files.json` as the publish's
    `files` map). Never inline the audio: a page carrying megabytes of data URIs never loaded in the viewer
    (Fleet Command, 2026-11-05).
- **Paths to read differently below**: "CLAUDE.md section 1 / 8" means this repository's CLAUDE.md
  ("Ask the owner with a survey", "End every reply with the artifact links"); "the Crab Breeze Music page" is
  a Pale Blue Dot Music page on the mockup tracker; `openspec/changes/music` is this repository's music
  change when it exists.
- **Setup on the Mac** (not yet tried here): LMMS (`brew install --cask lmms`), `brew install fluid-synth ffmpeg`,
  then `(cd tools/music && npm install)` and `tools/music/fetch_soundfonts.sh`; whether LMMS renders headless on
  macOS without a display is the first thing to check. The `apt-get` line below is for a Linux container.
- Sound effects and ambience are the `sound-effects` skill.

---

*Crab Breeze's text, verbatim, below.*


# Music authoring

From Tom Lander (`audio.md` and `tools/strudel/` at `dea3d50`, 2026-10-25), rebuilt for Crab
Breeze; provenance in `.claude/skills/PROVENANCE.md`. Tom Lander's flow was: motifs transcribed
into a reference file, arranged in Strudel, rendered by a small Node synth, and for orchestral
work each score copied its patterns into its own MIDI exporter. Here the instrument rides on the
note (`s("gm_trumpet")`), so **one exporter serves every score**, and LMMS renders the final comp.

## The pipeline

```
notes on paper           docs/design/music.md + the MOTIF LIBRARY block of music/motifs.strudel
   |
Strudel score            music/<cue>.strudel   (plays as written at https://strudel.cc)
   |  tools/music/strudel_to_midi.mjs        tools/music/strudel_to_lmms.mjs
   v                                          v
multitrack MIDI          music/<cue>.mid      LMMS project  music/<cue>.mmp
(any DAW)                                     |  lmms render (headless, xvfb)
                                              v
                                   final comp -> loudnorm -16 LUFS
                                              |-> assets/audio/music/<cue>.ogg  (the game's copy)
                                              '-> output/music/<cue>.mp3        (share copy)
```

`tools/music/render.sh music/<cue>.strudel` runs the whole chain. `--preview` renders the MIDI
through fluidsynth instead (seconds, rougher, for comparing arrangements).

## 1. Motifs first, in notes

Before any score, write the motif down: its notes and rhythm, the chords under it, what it means
and where it plays. That goes into `docs/design/music.md` and, as Strudel, into the `MOTIF
LIBRARY` block of `music/motifs.strudel` (the motif sheet, which plays every motif in turn). Read
`docs/design/music.md` before writing anything: it holds the key (D minor, the lift into D major),
the spine, the stride rhythm, the crab material and the rules for new music.

Ask the owner about direction with the survey, never in chat (CLAUDE.md section 8). Build on the
recommendation while a question is open.

## 2. A cue is a Strudel score

A score is one self-contained `.strudel` file, so it can be pasted into strudel.cc and heard. Its
header:

```js
// @title Crab Breeze: Sortie          the title in the file's tags
// @cycles 64                          bars to render = the sum of the bars in arrange()
// @reverb 0.35                        the soundfont's reverb for this cue (0..1)
// @mix gm_lead_2_sawtooth vol=72      this cue's comp: LMMS track volume (%) and pan
// @drumkit 25                         the soundfont's drum kit (0 Standard, 24 Electronic, 25 808/909, 26 Dance)
// @voice gm_pad_poly TripleOscillator/HiPad.xpf   this cue plays that role on another LMMS preset
// @space gm_pad_poly 0.35             a reverb on that track, 0..1 wet
// @layer bd Kicker/KickPower.xpf      every bd hit also on an LMMS drum synth (track layer_bd)
// @fx drums comp+drive                calf compressor and/or TAP tube drive on a track
// @fx master glue                     a gentle compressor on the master: the music ducks under the kick
```

**Melodies follow `docs/design/music.md` "Writing a melody"**: one short idea, built into a sentence
(idea, idea again, continuation, cadence), rhythm first, one high point, mostly steps. **Drums
follow "The drums: 90s weight"**: layered, compressed, driven, played hard, level with the music
(solo-render the drum tracks and the rest of a copy of the `.mmp` and compare their RMS).

The score is electronic and futuristic (CLAUDE.md section 1): every melodic role is a General MIDI
name that `music/instruments.json` `voices` maps to an LMMS synth preset for the final comp
(`gm_lead_2_sawtooth` SuperSawLead, `gm_pad_poly` TINTNpad, `gm_synth_bass_2` DirtyReece,
`gm_lead_8_bass_lead` LB302 acid, and so on: the table is in `docs/design/music.md`). The GM name
keeps the strudel.cc preview working; the comp gets the real synth. Echoes are written as notes
(`stack(p, p.late(3/16).gain(0.35), ...)`): LMMS 1.2's Delay effect renders silence headless.

then `setcps(BPM/60/4)` (one cycle is one 4/4 bar), the `MOTIF LIBRARY` block copied word for
word, the sections as `stack(...)`, and the form as `arrange([bars, section], ...)`.

**Every layer names its instrument with `s(...)`**, which is how the exporters split the score
into tracks:

| `s(...)` | Plays as | In strudel.cc |
| --- | --- | --- |
| `gm_<name>` (Strudel's General MIDI names, e.g. `gm_french_horn`, `gm_string_ensemble_1`, `gm_distortion_guitar`, `gm_choir_aahs`, `gm_timpani`; the table is `GM_NAMES` in `tools/music/score_lib.mjs`) | That GM program, through the soundfont, on its own MIDI channel and LMMS track | The same GM sound |
| `bd sd hh oh cr rd rim ht mt lt ft tb sh ...` (`DRUM_MAP`) | The GM drum kit, channel 10 | Strudel's drum samples |
| `sawtooth square triangle sine supersaw` | The LMMS preset `music/instruments.json` gives it (TripleOscillator), with a GM stand-in in the MIDI file | Strudel's synths |

Anything else stops the export with its name: a dropped part would ship a cue missing a voice.
Loudness of a note is `gain` (and `velocity`): MIDI velocity = 100 x velocity x gain, so keep
leads near 1.0 to 1.1 and accompaniment at 0.4 to 0.8. The same instrument in two layers is one
track (strings low and high merge); give a layer its own instrument to give it its own fader.

Strudel details that matter here: `@3` weights a step (a dotted quarter in an eighth grid), `~` is
a rest and takes a weight too (`~@3`), `<a b c>` is one per bar, `!2` repeats, `/2` stretches a
pattern over two bars, `.early(12)` starts a 16-bar motif at its bar 13, `.add(note(12))` and
`.sub(note(12))` move it by octaves, `.struct(STRIDE)` re-rhythms chords, and a mini-notation
string in `gain(...)` gives per-step accents. Check new syntax on a two-bar test before a long
score: `node tools/music/strudel_to_midi.mjs test.strudel /tmp/t.mid` prints every part and its
note count.

## 3. Render and look

```sh
tools/music/render.sh music/<cue>.strudel
node tools/music/check_motifs.mjs          # every score carries the motif library unchanged
```

`render.sh` writes the `.mid` and `.mmp` beside the score, renders with LMMS, refuses a raw mix
that clips (`check_mix.py`), brings it to -16 LUFS with a plain gain and a peak limiter at -1.5
dBFS, and prints the loudness per 5 s. (Not ffmpeg's `loudnorm`: it falls back to dynamic
compression whenever a linear gain would break its peak limit, and flattened every cue.) Then look before handing it over:

- **The profile** has the cue's shape (a quiet intro, loud peaks, a falling ending). A flat or
  empty one means a wrong `@cycles`, a section that never sounds, or an unknown instrument.
- **A spectrogram** shows the lead line above the accompaniment:
  `ffmpeg -i output/music/<cue>.mp3 -lavfi "showspectrumpic=s=1600x400:fscale=log:stop=5000" spec.png`.
  If the melody's contour is not visible, the comp buries it (step 4).
- To hear one instrument alone, set `muted="1"` on the other `<track>`s of a copy of the `.mmp`
  and render it; `astats` gives its level.

A render that passes these is checked, not approved: only the owner's ears approve music. Say so.

## 4. The final comp in LMMS

LMMS is where the instruments and the mix are chosen. The project `strudel_to_lmms.mjs` writes has
one instrument track per part:

- **GM parts and drums** play through LMMS's Sf2Player on the soundfont in
  `music/instruments.json` (GeneralUser GS 2.0.3, fetched by `tools/music/fetch_soundfonts.sh` to
  `/opt/soundfonts`, pinned by commit and SHA-256), with its reverb set by `@reverb`.
- **Voiced parts** (every GM name in `voices`, every `@voice` line, and Strudel's own synth names
  through `synths`) play an LMMS preset, a path under `/usr/share/lmms/presets` (TripleOscillator,
  LB302, Monstro ...). The exporter switches off a preset's own chord and arpeggio generators (the
  score writes every note) and pins its base note, then plays it back by its measured pitch offset.
  Pick modern presets: an LMMS 0.x preset (`<multimediaproject>`, like TranceLead) cannot be used.
- **Reverb**: `@space <role> <wet>` puts a ReverbSC on that track. The soundfont's own reverb is
  `@reverb`.
- **Levels and pitch are calibrated**, not tuned: `tools/music/calibrate.sh --write` plays one
  phrase on every soundfont instrument and every LMMS preset the scores use at volume 100, measures
  it and sets `mix.<instrument>.vol` or `preset_vol.<preset>` so all sit at -30 dBFS. For presets it
  also renders A3 and measures how far it really sounds (`tools/music/pitch.py`: autocorrelation and
  harmonic product spectrum must agree), storing `preset_offset_st`: many presets sound an octave
  or two from their key. Rerun it after changing the soundfont or adding a voice; a `WARN pitch
  unclear` means pick another preset. A cue's balance is then its own `@mix` lines (lift the lead, trim the bass) and its
  gains. Never hand-edit the `.mmp` and keep it: it is regenerated from the score.
- To mix by hand in the LMMS GUI (on a desktop), open the `.mmp`, then carry the volumes you chose
  back into the score's `@mix` lines.

Headless facts (from Tom Lander): LMMS refuses root without `--allowroot`, always wants a display
(`xvfb-run -a`), and cannot import MIDI headless (so the project is written directly). It renders
to the last note: give a cue a held final chord rather than trailing silence.

## 5. Where things go and what to commit

| Path | What | Commit |
| --- | --- | --- |
| `music/<cue>.strudel` | The score, the source of truth | yes |
| `music/<cue>.mid`, `music/<cue>.mmp` | Exports, regenerated by `render.sh` | yes (for a DAW on another machine) |
| `music/instruments.json` | Soundfont, synth presets, calibrated levels | yes |
| `assets/audio/music/<cue>.ogg` | The game's copy, -16 LUFS | yes |
| `output/music/<cue>.mp3` | Share copy | no (`/output` is ignored); send it or publish it on the music page |

After a render worth hearing, republish the Crab Breeze Music page (link in
`docs/design/README.md`) with the new MP3s and end the reply with its link (CLAUDE.md section 8).
Music is not wired into the game until `openspec/changes/music` is applied.

## Setting up a session

```sh
apt-get install -y --no-install-recommends lmms fluidsynth fluid-soundfont-gm ffmpeg xvfb
(cd tools/music && npm install)       # @strudel/core, mini, transpiler; postinstall patches kabelsalat
tools/music/fetch_soundfonts.sh        # GeneralUser GS into /opt/soundfonts
```

About two minutes in a cloud session. Rendering needs no GPU, so a cloud session can make and
check music; only listening is the owner's.

## What is not here yet, and where to get more

- **Sample-based orchestra (SFZ)**: VSCO 2 Community Edition (CC0) and others are SFZ, which LMMS
  1.2 cannot load. Either convert to sf2 with Polyphone or render those stems with sfizz; neither
  is set up. `docs/analysis/audio-sources.md` lists the candidates and their licences.
- **ZynAddSubFX** ships inside LMMS and makes far richer pads and leads than TripleOscillator; its
  presets would need their XML embedded in the project. The next step for richer synths.
- **LMMS's Delay effect** renders silence headless in 1.2.2 (measured 2026-10-25): echoes are notes.
- **Tempo changes inside a cue**: one tempo per score. Split a cue whose tempo changes.
