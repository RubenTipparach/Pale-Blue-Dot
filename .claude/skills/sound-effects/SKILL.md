---
name: sound-effects
description: Make, change and judge Pale Blue Dot's sound effects and ambience beds the repository's way, after Fleet Command's sound-effects skill and Crab Breeze's tools/audio. Recorded CC0 sounds first (packs fetched by tools/audio/fetch_cc0.py, which refuses any pack whose page does not say CC0 or public domain), cut, slowed, layered and filtered to a spec in tools/audio/cc0_cues.json; synthesised in tools/audio/gen_sfx.py only where nothing recorded beats it, or as a layer a recording lacks; every original named in assets/audio/sfx/SOURCES.md and AUDIO_SOURCES.md; mono and placed in 3D in the camera's body-local frame except menus, ambience beds and music; judged by measurement and a spectrogram contact sheet because a Claude session cannot listen; auditioned on the Sounds page. Use whenever making or changing a sound effect, a footstep, a dig or place sound, thunder, fire, rain, wind, surf, birds or insects, an engine or a hull, a menu click, an ambience bed, or asking where a sound came from or what licence it has ("make a sound", "sfx", "footstep sound", "thunder", "ambience", "rain sound", "wind sound", "CC0 audio", "audio sources").
metadata:
  author: Pale Blue Dot (Claude Code), from Fleet Command's sound-effects skill (itself after Crab Breeze's tools/audio)
  version: "1.0"
---

## In Pale Blue Dot

The owner, 2026-11-14: "using skills from fleet command and crab breeze - spin up a audio sfx and ambient system
session, and have it design audio". Copied from Fleet Command (`.claude/skills/sound-effects` at `08374c4`);
provenance in `.claude/skills/PROVENANCE.md`. The text below the line is Fleet Command's, verbatim: its method
holds here, its paths and its examples (weapons, shields, a ship's bridge, skins) are that game's. Where they
differ, this note wins.

- **The design is `openspec/changes/audio-and-ambience`.** Read its `design.md` before making a cue: it names
  the buses, the cue list, the ambience beds and the distance law. Nothing is wired into the game until that
  change is applied.
- **The tools are ported** (`audio-and-ambience` task 2.1, from Fleet Command `08374c4`; what changed is in
  `.claude/skills/PROVENANCE.md`): `tools/audio/fetch_cc0.py`, `cut_cc0.py`, `gen_sfx.py`, `inspect_sfx.py`,
  `render_sfx.sh`, `list_sources.py`, `gen_sfx_page.py`, `audio_page.py`, and `encode_ogg.py` (new: the Mac's
  ffmpeg has no libvorbis). No Fleet Command cue or pack was carried over. Originals are fetched to
  `output/sfx-src` (git-ignored), not `/opt/sfx-src`. A Freesound sound is `freesound/<id>` in
  `cc0_packs.json` and `freesound/<id>:<id>.mp3` in a layer; the fetcher refuses it unless its page links CC0.
  A cue with `takes` writes `<cue>_1 .. _N`. The Sounds page's MP3s are in `docs/mockups/sounds/`, and
  `docs/mockups/sounds.files.json` is the publish's `files` map.
- **Where the first set came from** (2026-11-14, on the Mac): Kenney's packs (fetched through Kenney's
  download-link endpoint) and Freesound sounds whose pages say Creative Commons 0. OpenGameArt answered 403 to
  the fetcher that day, so none of its packs were used; it was not worked around. Sonniss GDC bundles are not
  CC0 and were not considered.
- **Beds with birds or insects** keep their low pass at 8 kHz, not 3.5: birdsong lives at 2 to 6 kHz and
  cricket song near 4.5 kHz, and a 3.5 kHz low pass took them out. Wind, rain, surf and the town keep the
  3.5 kHz treatment (rain at 6 kHz, under water at 1.5).
- **The engine is kira, not Godot's and not rodio.** The game plays sounds through kira (design section 1),
  which filters live: a muffled copy for caves, under water or a closed cabin is a low pass on a bus, not a
  second file. Crab Breeze's `muffled/` and `far/` copies (rodio had no filter) are not made here; Fleet
  Command already dropped them.
- **Where a sound is.** "Placed in 3D" here means at a body-local point, given to the audio engine relative to
  the camera's eye in the camera's frame, as every render path is (CLAUDE.md, "Water, atmosphere, terrain
  lighting, and their cameras use the same declared body-local frame"). Read Fleet Command's "named mount node"
  as an entity's `Transform` here: a vehicle's engine, a walker's feet, a fire's cell.
- **Groups here** are `Steps`, `Ground` (dig, place, land), `Water`, `Weather` (thunder, rain hits, fire),
  `Craft`, `Fishing`, `Fauna` (placed), and `Menu`, `Ambience` (not placed; a bed's name starts `amb_`). A cue
  name is the contract between the data (`assets/config/audio/*.ron`) and the files, as there.
- **Sound a thing by what it is made of** (Fleet Command's rule, kept): a hex of soil is not stone, sand is not
  snow. Footsteps and digs are one cue family per `pbd_core::terrain::Material`, not one generic thud.
- **Beds are quiet and far** (Crab Breeze's rule, kept): a bed is compressed, low passed near 3.5 kHz, given a
  diffuse reverb and cut about 11 dB down, so it sits behind every placed sound.
- **Night is quieter, never silent.** Crickets, frogs near water, an owl now and then; the wind stays.
- **Listening.** Testing now runs on the owner's Mac (memory, 2026-11-13), which has speakers; Claude still
  cannot hear. Judge by `inspect_sfx.py` and the contact sheet, then by the game's `--audio-log` (design
  section 9), and say plainly that only the owner's ears approve a sound.
- **Publish the Sounds page** like a mockup: on the tracker in the same step, with its `files` map, and
  "All Pale Blue Dot mockups (the tracker)" near its top (CLAUDE.md, "The mockup tracker").
- **Setup on the Mac**: `brew install ffmpeg`, Python 3 with numpy. The cloud setup line below is for a Linux
  container.

---

*Fleet Command's text, verbatim, below.*


# Sound effects

The owner, 2026-11-04: "pull in audio and music skills from crab breeze, lets get some audio for this
game!" Crab Breeze keeps its sound method in its CLAUDE.md and `tools/audio/`, not in a skill; this
skill writes it down for Fleet Command. The tools are ported from Crab Breeze `0e569e3`
(`.claude/skills/PROVENANCE.md`). Music is the `music-authoring` skill.

## The rules

From CLAUDE.md section 11 and Crab Breeze's sound rule (owner, 2026-10-25: "all audio and sfx
approved", "verify all sounds are 3d based (except ambience and music and ui)"):

- **Recorded CC0 first.** A cue is cut from real recordings wherever one exists, slowed and layered
  to the size of the thing making it. It is synthesised only where nothing recorded beats it (console
  tones, alerts) or as a layer a recording lacks (a beam's held buzz, a shield's shimmer).
- **Every original is named.** `assets/audio/sfx/SOURCES.md` (one row per cue: each file and the
  edits) and `AUDIO_SOURCES.md` (one row per pack: its page, who recorded it, its licence) are
  written by the tools from the spec. Edit the spec, never the tables.
- **CC0 or public domain only**, checked on the pack's own page when it is fetched. A pack whose page
  does not say so is refused, and nothing is copied from another game. **The one way past it**: the owner asks for a
  pack by name; it goes into `tools/audio/licence_exceptions.json` with the owner's words, its licence and the credit
  its page asks for, which the Sounds page, `AUDIO_SOURCES.md` and the game's credits then carry (the first: Little
  Robot Sound Factory's Sci-Fi Sound Effects Library, CC-BY 3.0, 2026-11-07).
- **Placed in 3D**: every battle sound (weapons, impacts, the ship) plays from its ship or its named
  mount node (CLAUDE.md section 10, named nodes), mono. Only the console keys, the alerts, the
  ambience beds and the music are unplaced, and only beds may be stereo.
- **A cue name is a contract.** The game's data names cues; a better recording replaces the file and
  keeps the name.
- **Deterministic.** Each synthesised cue seeds its generator from its own name; a rebuild writes the
  same sound.
- **A skin's console keys are its own** (survey U6, decided: a skin changes pictures, colours, type
  and button sounds). `ui_chirp` is the Concord's soft chirp, `ui_clunk` the modelled metal skins'.
  A new skin's key sound comes from how its makers build things, never from the species' body
  (CLAUDE.md section 1).

## The pipeline

```
tools/audio/cc0_packs.json --fetch_cc0.py--> /opt/sfx-src (originals, never committed) + licences.json
tools/audio/gen_sfx.py      ---------------> output/sfx/<cue>.wav + sfx.json   (synthesised cues and layers)
tools/audio/cc0_cues.json   --cut_cc0.py---> output/sfx/<cue>.wav               (recorded cues replace them)
                                             assets/audio/sfx/SOURCES.md, assets/audio/ambience/SOURCES.md
render_sfx.sh: ffmpeg -----> assets/audio/sfx/<cue>.ogg (Vorbis q5, mono), assets/audio/ambience/amb_*.ogg (stereo, q4)
                             output/sfx/<cue>.mp3 (share copies), AUDIO_SOURCES.md (list_sources.py)
gen_sfx_page.py -----------> docs/mockups/sounds.html (every cue with a play button) + sounds.files.json
inspect_sfx.py check ------> clipping, loop seams, output/sfx/sheet.png (look at it)
```

One command rebuilds everything: `tools/audio/render_sfx.sh` (or `tools/audio/render_sfx.sh <cue> ...`).

## Adding or changing a cue

1. **Name it and place it.** Group `Weapons`, `Impacts`, `Ship` (placed) or `Console`, `Alerts`,
   `Ambience` (not placed; a bed's name starts `amb_`). Write its `what` from the player's side:
   what they hear, then what makes it. A loop's `what` contains `(loops)`.
2. **Find recordings.** Search OpenGameArt with the CC0 filter
   (`https://opengameart.org/art-search-advanced?keys=<words>&field_art_type_tid[]=13&field_art_licenses_tid[]=4`),
   add the pack to `tools/audio/cc0_packs.json` as `oga/<slug>`, then
   `python3 tools/audio/fetch_cc0.py --dest /opt/sfx-src oga/<slug>`. Kenney packs (`kenney/<slug>`)
   and Internet Archive items (`archive/<item>:<file>,...`) work the same way where the network allows.
3. **Choose by measurement**, since a cloud session cannot listen:
   `python3 tools/audio/inspect_sfx.py inventory` lists every file with its length and brightness
   (spectral centroid); `python3 tools/audio/inspect_sfx.py envelope 'oga/<slug>:<file>' ...` prints a
   0.1 s loudness strip and the share of energy under 250 Hz and over 4 kHz, which shows where a take
   starts, how many shots it holds and how heavy it is. A heavy thing (a torpedo tube, a hull) wants
   low energy; a beam or a spark wants high.
4. **Write the spec** in `tools/audio/cc0_cues.json` (its `_about` lists every field): layers of
   `{"src": "pack:file"}` or `{"synth": "<layer>"}` with `start`, `dur`, `pitch` (tape speed: below
   1 is bigger and slower), `gain`, `lp`, `hp`, fades and `delay`; the cue's `len`, `drive`, `loop`,
   `stereo`, `distance` and `peak_db`. Say the edits in `edits`. A bed that should be busy (the bridge's bleeps and
   scans) uses `scatter`: groups of short sounds placed at seeded times, gains and pans over the bed.
   **Keep weapons short and front-loaded** (owner, 2026-11-06: "weapons all are too long, lance fire should sound
   more punchy"): start each take at its peak, put a kick under it (`punch_thump`), and cut to about half a second
   (1.2 s at most for a missile's roar). A hit gets a short alternative (`<cue>_short`) for rapid fire.
5. **Synthesise only what is missing.** A synthesised cue or layer is a function in
   `tools/audio/gen_sfx.py` registered with `@sound(name, group, what, peak_db)`; a layer's group is
   `_layer`, so it is mixed into recorded cues and never written alone. Use the building blocks
   already there (oscillators, `sweep`, `adsr`, the SVF `lp` and `hp`, `ring`, `reverb`, `chorus`,
   `tone`); never a second copy of one.
6. **Build and check**: `tools/audio/render_sfx.sh`, then `python3 tools/audio/inspect_sfx.py check`
   (fails on clipping or a loop seam bigger than a few ordinary sample steps) and **look at the
   contact sheet** `output/sfx/sheet.png`: a launch should rise, a hit should strike and decay, a loop
   should be even, a console tone should fade to nothing (a hard vertical edge at its end is a click).
   Copy the sheet to `docs/screenshots/audio/sfx-sheet.jpg` at half size.
7. **Audition and record**: the page is rewritten by the build; it is an artifact like a mockup
   (CLAUDE.md section 4): commit it with its shots, its record in `docs/artifacts/sounds/` and its row
   in `mockup-work.md` and the tracker, and publish it with the audio beside it: pass
   `docs/mockups/sounds.files.json` as the publish's `files` map (each MP3 at the same path the page names). Never
   inline audio in the page: version 1 did (1.9 MB of data URIs) and never loaded in the viewer (2026-11-05).

## Softening (Crab Breeze's two rules)

When the owner says a sound is too harsh, too bright or too foreground (2026-11-07: "soften bridge ambient along with a
lot of the other weapons, check the procedure in crab breeze how audio was softened with low passes"):
- **A bed** gets Crab Breeze's ambience treatment: `"distance": {"lp": 3500, "wet": 0.35, "threshold_db": -18,
  "ratio": 3}`, a cue `"lp": 3500, "lp_order": 4` so scattered sounds are softened with it, and `"peak_db": -14`.
- **A sound heard from the bridge through the hull** (the weapons) gets Crab Breeze's sealed-cockpit filter:
  `"lp": 4000, "lp_order": 4, "peak_db": -7` (4 kHz twice over, 6 dB down), and less `drive`.
- Measure before and after (energy above 4 kHz, spectral centre, RMS, the first 20 ms of a shot) and say the numbers.
- **Prefer a recorded bed** the owner points to: the bridge took four synthesised tries (rumble, water, buzz) before
  the owner sent a recording (2026-11-07). A synthesised stack of harmonics reads as mains buzz.
- **A bed is a hum, not a noise and not a rumble** (owner, 2026-11-07, over three versions of the bridge: "low rumble
  bridge ambient, it hurts my ears a bit", then of pink noise "now sounds like water!", then "I want it to sound like a
  hum"). Make a ship's bed tonal: a warm tone between about 120 and 400 Hz with soft harmonics (`ship_hum`), or a
  recorded hum moved there by `pitch` (a 76 Hz hum at 2.0 is 152 Hz); cut it below about 110 Hz (`"hp": 110,
  "hp_order": 4`) and drop any steady tone under 100 Hz (they hurt); no broadband noise wash, filtered or not (it reads
  as water or rain); nothing that moves in pitch (it reads as whirling). Judge scattered sounds over it A-weighted, as
  the ear hears them, at most about 10 dB over the bed, and set their gain against the bed's average, not its peak.

## What a thing is decides how it sounds

Sound a thing by what it is made of, not by what sounds sci-fi (owner, 2026-11-11, of a shield hit with a glassy ring:
"IT IS NOT GLASS< NOT CERAMIC< NOT METAL< IT CANNOT DING", "IT IS ELECTROMAGNETIC"). A shield is a field: noise and
electricity (a whump, a fizz, a crackle), never a ringing tone. A hull is metal: it may ring. Before keeping a layer,
check its ringing (`python3 tools/audio/inspect_sfx.py check` prints it for every cue; `NO_RING` caps the shield
sounds), and do not trust a pack's name: every take in "space ship shield sounds" rang 58-100%.

## Levels

Cues are peak-normalised when written: `-1 dBFS` for weapons and blasts, about `-9 dBFS` for
console keys, `-4` to `-8` for alerts, `-6` for the bridge bed. The mix between cues is set by each
cue's volume in the game's data, not by re-cutting the file.

## Setup in a fresh cloud session

`apt-get install -y --no-install-recommends ffmpeg p7zip-full unzip` and Python with numpy (about a
minute); `python3 tools/audio/fetch_cc0.py` fetches every pack (about 110 MB) into `/opt/sfx-src`.
On 2026-11-04 OpenGameArt was reachable from a cloud session and kenney.nl answered 403; the Kenney
packs used here are Kenney's own uploads on OpenGameArt.
