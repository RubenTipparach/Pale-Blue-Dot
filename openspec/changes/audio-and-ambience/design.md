# Design: sound effects and ambience

## Context

- **No sound today.** `Cargo.toml` builds `bevy = "=0.18.1"` with
  `default-features = false`; `pbd-app` turns on rendering, UI, windowing and
  input features, never `bevy_audio`. `Cargo.lock` has no `rodio`, `cpal` or
  `kira`. There is no `assets/audio/`.
- **What sound hangs off** (read, not changed, by this design):
  - `weather.rs`: `Weather { rain, wetness, cover, snowing, sheltered, flash,
    here }`. `strike_lightning` picks the newest strike within 3 km for the
    flash; the strikes themselves are `Air.now.strikes`
    (`pbd_core::atmosphere::Strike { direction, step, strength }`), kept a few
    steps of `dt_s` = 1 s. The bolt runs from the cloud base,
    `CLOUD_RADIUS` = R + 300 m, to the ground.
  - `walking.rs`: `GroundState { grounded }`, the jump on its edge and held,
    the swim model under water, `WalkingReadout`.
  - `desktop/digging.rs`: `dig_and_place` with a progress per block, and
    `apply_edit`; `material_of` gives the `pbd_core::terrain::Material` of a
    block (Stone, Soil, Grass, Sand, Snow, Rock, Dirt, Ore, the lamps).
  - `planet_contact.rs`: `open_to_sky(point)`, `stand(feet)`, `sample`.
  - `pbd_core::wind::wind_at(settings, air, point, height, seconds)`.
  - `pbd_core::planet_gen::biome` (Ocean, Beach, Fields, Desert, Jungle,
    Swamp, Mountains, Tundra).
  - `sky.rs`: the clock and the sun; `ATMOSPHERE_RADIUS` = 1.2 R, so the air
    ends 960 m up on this R = 4,800 m planet.
  - `pbd_core::vehicle`: Kestrel (tiltrotor), Tern (keelboat), Loon (canoe),
    Cog (square-rigged ship), Rowboat, each with a `Telemetry` (stall, luffing,
    strokes).
  - `pbd_core::fishing::Event`: Cast, Dry, Splash, Interest, Nibble, Bite,
    TooEarly and the rest.
  - `loading.rs`: the loading thread's `Step`s, `warm`, `READY`, and
    `LATE_PIPELINE` for anything compiled after it.
  - `settings.rs`: the player's file, `VERSION` 2.
- **Crab Breeze's audio**, read at `60c8b2f` (`cb-app/src/audio.rs`,
  `ambience.rs`, `music.rs`, `cb-core/src/audio.rs`, `cb-tools/src/audio_check.rs`
  and its `audio`, `ambience` and `music` changes). What carries over: every
  rule in the engine-free crate and tested there; cues as data
  (`cues.ron`) with a test that every cue the code names exists, and a tool
  that checks the data against the real files (placed files mono); the
  speed-of-sound delay; a voice cap; equal-power bed crossfades; quiet, far
  beds; `CB_TAKE_SOUND=1` for takes. What does not: its engine (below) and its
  pre-rendered muffled and far copies.

## Goals / Non-Goals

**Goals:**
- Every action the player takes on foot, in the water and in a craft has a
  sound that says what it touched.
- Where the player is can be told with the eyes shut: a meadow at noon, a
  jungle at night, a beach, a cave, a town, a storm, under water, high up.
- A storm is heard before it is seen, and a near strike is felt.
- No hitch: no sound decodes on the main thread in play.
- Testable without ears: every decision is a pure function with a test, and
  every sound the game makes can be logged, metered and written to a file.

**Non-Goals:**
- **Music.** The bus and its volume exist; choosing, writing and scheduling
  music is a music change (`music-authoring` skill). Direction is the owner's
  and not chosen.
- **Townsfolk voices, animal calls tied to visible animals, and speech.** No
  land animals exist; birds and insects are in the beds.
- **HRTF, occlusion by terrain, and reflections.** Sound is panned in stereo,
  low passed by distance and a little behind the head; a hill does not block
  it. A cave is a filter and a reverb, not a traced room.
- **Doppler.** A Kestrel passing at 60 m/s does not bend in pitch in the
  first build; a later change can add it from radial velocity.
- **Multiplayer sound.** Every sound is the local listener's; another player's
  footsteps would be the same placed cues, when there are other players.

## 1. The engine: kira, directly

Four candidates for Bevy 0.18.1, read 2026-11-14 on crates.io and in the
crates' source:

| Option | Bevy 0.18 | Bevy 0.20 | Buses | Live filters | Spatial | Scheduling | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `bevy_audio` (rodio 0.20) | built in | rodio 0.22 | none (one global volume) | none | two ears, `1/d^2` falloff | frame timers | no |
| `bevy_kira_audio` 0.25 (kira 0.10.8) | 0.25.0 | 0.27.1 (kira 0.12) | channels, volume only | none (no effect API) | volume and pan per instance | kira clocks | no |
| `bevy_seedling` 0.7.2 (Firewheel 0.10) | 0.7.x | no release (0.8.0 is Bevy 0.19) | a full graph | yes | yes | yes | not yet |
| **`kira` 0.12.6, directly** | any (no Bevy dependency) | any | sub-tracks | filter, reverb, EQ, compressor, custom | spatial sub-tracks and a listener | clocks, `StartTime` | **yes** |

Why kira, and why without a wrapper:

- **Live filters on buses are what this game needs most.** Under water,
  inside a cave, through a craft's hull and with distance, the world's sound
  is low passed by an amount that changes every frame. rodio has no filter;
  Crab Breeze had to render a muffled and a far copy of every placed file and
  switch between them when a sound starts, which cannot follow a swimmer
  sinking or a cave mouth fading. kira has a filter effect with a cutoff that
  tweens, on any track.
- **No Bevy version in the audio path.** kira depends on cpal and symphonia,
  not on Bevy, so `bevy-0-20` does not touch sound, and a kira upgrade is its
  own step. `bevy_kira_audio` and `bevy_seedling` tie each release to one Bevy
  minor; seedling has no Bevy 0.20 release, so choosing it now would block or
  entangle `bevy-0-20`.
- **`bevy_kira_audio` hides what is needed.** Its 0.25 source has no effect
  API on a channel, and its spatial support sets only volume and panning per
  instance.
- **Bevy's own audio stalls the main thread.** Crab Breeze measured 20 to
  43 ms main-thread stalls starting a bed part-way in
  (`PlaybackSettings::start_position` decodes and throws away everything before
  it, 2026-10-26), and had to write a decoder that skips on the audio thread.
- **kira's spatial falloff can be turned off** (`attenuation_function(None)`
  gives constant volume, read in 0.12.6's `spatial_builder.rs`), so its
  panning is used and the level is this design's own law (section 3), as
  Crab Breeze did by scaling positions into rodio's unit radius.
- **A mock backend** (`kira::backend::mock`) runs the whole mixer without a
  sound card: for tests, for a cloud session with no device, and for
  `--audio-wav` (section 9).

The cost of going direct is the glue: a Bevy `Resource` holding kira's
`AudioManager` (it is `Send + Sync`; kira's own `sync_send` test checks it),
a listener, and the systems in section 4. The manager is generic over its
backend, so the resource holds an enum of the cpal and mock managers, or the
glue is written against the handles, which are the same type for both. Crab Breeze's glue on `bevy_audio` is 800 lines; this is the same size of
job.

Pinned: `kira = { version = "=0.12.6", default-features = false, features =
["cpal", "ogg"] }`. kira 0.12 brings glam 0.34 beside Bevy's; positions cross
as `mint` vectors, so the two never meet. If kira's backend cannot open a
device (a headless container, no output), the game logs `AUDIO off: <why>`,
runs the mock backend instead and plays silently; nothing else changes.

## 2. Buses and volumes

```
main (master volume)
 |- world      the environment: one low-pass filter and one reverb send, set every frame
 |   |- effects     every placed sound (one spatial sub-track per voice)
 |   |- ambience    the beds
 |- music      (nothing on it yet)
 |- menus      menu and HUD sounds, never filtered
reverb         a send from `world` (caves, under water, a small amount outdoors)
```

- `world` carries the environment, so under water or in a cave the footsteps
  and the beds are muffled together while the menus and the music are not.
- **Five player volumes**: master, effects, ambience, music, menus, each 0 to
  100 % in steps of 10, as a gain on its bus (`0 %` is silent, not -60 dB).
  Defaults: master 80, effects 100, ambience 80, music 60, menus 70.
- **They live in the player's settings file**, version 3 (section 11), shown
  as five rows on the settings screen beside RENDER SCALE, stepped like it.
  A change is heard at once and written when the player changes it, as every
  setting is.
- **Pause** pauses `world` (kira tracks pause and resume with a 0.1 s tween);
  the menus keep sounding. The map and inventory over a running world duck
  `world` 6 dB and low pass it at 4 kHz, so the planet is still there.
- **Unfocused window**: `world` and `music` duck to silence over 0.3 s, and
  come back over 0.5 s. Recommendation taken (ask only with screenshots); a
  player who wants sound in the background gets a row for it later.
- `--capture` and `--frame-log` runs take default volumes, never the
  player's, as every setting is ignored there.

## 3. Placing a sound on a small planet

**Frames.** Every placed sound has a body-local position (the frame the
terrain, water and weather already share). Each frame the audio system takes
the active camera's eye and orientation in that same frame and gives kira:

- the listener at the origin, turned as the camera;
- each emitter at `emitter - eye`, an f32 difference of body-local points,
  rotated into nothing else.

So positions handed to kira are always small (metres from the ear), an origin
rebase never moves a sound, and a craft's frame or a deck's frame is just
another way of getting a body-local point. A sound riding something (a
craft's rotor, a walker's feet, a fire) reads its point every frame.

**Level.** kira's own falloff is off. The gain is
`pbd_core::audio::distance_gain`: full within `full_within_m` (per cue,
default 3 m), then `full_within_m / d` (6 dB per doubling), and silent past
the cue's `audible_m`. A loud thing (thunder) has a large `full_within_m`.

**Air.** A low pass whose cutoff falls with distance, per voice:
`cutoff_hz = 20000 * (air_ref_m / (air_ref_m + d))`, `air_ref_m` = 250 m by
default, floored at 300 Hz: a footstep next to you is bright, a strike 3 km
off arrives as a low roll. Behind the listener the cutoff is multiplied by
`behind_factor` (0.6) at full behind, so front and back differ a little in a
stereo pan.

**Delay.** A placed sound starts `d / 343` s after its cause
(`speed_of_sound_m_s` in `mix.ron`). Within 10 m it is not delayed: it would
be under a frame.

**Air density.** Every outside sound and bed is scaled by the air at the
listener: full below `air_full_m` (600 m above the surface), falling to
silent at the top of the atmosphere (`ATMOSPHERE_RADIUS`, 960 m). Above it,
the only sounds are the player's own craft, heard through its structure
(section 4, Craft). Airless bodies have no beds at all (CLAUDE.md: they have
no weather, so nothing for a bed to read).

**Under water.** When the eye is below the water surface, `world` is low
passed to 600 Hz and 9 dB down, the reverb send rises, and `amb_underwater`
plays. Crossing the surface is a 0.15 s tween, with a splash cue each way.

**Shelter and caves.** Two inputs from the column:
- *sheltered*: something solid over the eye (`Weather::sheltered`, from
  `open_to_sky`). Outdoor beds go through a 2.5 kHz low pass and 6 dB down; the
  rain becomes rain on a roof.
- *underground depth*: how far the eye is under the surface height at its
  column. From 2 m to 12 m deep, the outdoor beds fade to nothing, the cave
  bed (`amb_cave`: drips, low room tone) fades in, and the reverb send on
  placed sounds rises to `cave_reverb` (a long, dark tail). A town building is
  sheltered and shallow, so it is muffled and dry; a mine is deep and wet.

**Voices.** At most 40 placed voices and 10 beds. A new voice past the cap
takes the place of the quietest playing one if it is louder, else it is
dropped. Each cue has its own cap (a footstep: 4; thunder: 3). A loop asked
for in a frame keeps playing; one not asked for fades out over 0.15 s and
stops.

All of this is `pbd_core::audio`: pure functions of distances, depths, flags
and data, tested there.

## 4. The cues

Cue names are the contract between the code, `assets/config/audio/cues.ron`
and the files. A test lists every cue the code names and checks each is in
`cues.ron`; a tool checks every file in `cues.ron` exists and that every
placed file is mono. Each cue has one or more files (one picked at random,
never the same twice running), a level in dB, a pitch range, a voice cap,
`full_within_m`, `audible_m`, and whether it loops.

**Steps** (placed at the feet). A stride is taken every `stride_m` walked
while grounded (0.8 m walking, 1.1 m running, from `WalkingConfig`'s speeds),
so the steps keep time with the legs at any speed. The surface family comes
from the block under the feet:

| Family | Materials | Note |
| --- | --- | --- |
| `stone` | Stone, Rock, Ore | hard, short |
| `dirt` | Soil, Dirt | soft thud |
| `grass` | Grass, JungleGrass | soft, a brush of blades |
| `dry` | DryGrass | crisp |
| `sand` | Sand | a hiss of grains |
| `snow` | Snow | a squeak and crunch |
| `wood` | a town's floors, stairs and decks | hollow |
| `shallow` | wading water under the knees | a splash |

`step_<family>` has six takes. When the ground is wet (`Weather::wetness` over
0.4), `dirt`, `grass` and `stone` add a wet layer (`step_wet`) under the step.
`jump_<family>` is a push-off, `land_<family>` a landing at a level set by the
speed of the fall, with `land_hard` added over 9 m/s.

**Ground** (placed at the block). Digging plays `dig_<family>` at each third of
a block's progress, `break_<family>` when it comes out, and `place_<family>`
when one is set; a lamp is `place_lamp`, and a brazier lights with
`fire_ignite_small`. Picking a drop up is `pickup` (a menu sound, unplaced).

**Water** (placed). `water_enter_small` and `_big` by the speed into the
water, `water_exit`, `swim_stroke` at the surface, `swim_stroke_under` and
`bubbles` below it, `breath` on surfacing after 10 s under.

**Craft** (placed at the craft's parts, looped, level and pitch from its
telemetry each frame, eased with a 0.12 s time constant):
- Kestrel: `rotor_loop` per rotor (pitch with rotor speed, level with power),
  `nacelle_motor` while the nacelles tilt, `airframe_wind` by airspeed,
  `stall_warn` (a cockpit tone at the seat) when its telemetry says stall.
- Tern and Cog: `hull_water` by speed through the water, `bow_splash` on
  waves, `sail_luff` when luffing, `sail_fill` as the sail fills,
  `rigging_creak` by heel, `sheet_run` as the sheet runs out.
- Loon and Rowboat: `paddle_stroke` or `oar_stroke` on each stroke,
  `hull_lap` at rest, `water_slosh` with water in the hull.
- Free flight (the assisted flight body): `thruster_loop` by thrust.
- Above the air, a craft's own loops pass through a 1 kHz low pass (heard
  through the structure) and nothing outside is heard.

**Fishing** (placed at the rod or the float): `cast_whoosh` (pitch by cast
speed), `float_plop` on Splash, `float_dry` on Dry, `nibble_tick` on Nibble,
`bite_splash` on Bite, `reel_loop` while reeling, `line_snap`, `catch_splash`
and a menu chime when the fish is landed.

**Menus** (unplaced, the menus bus): `ui_hover`, `ui_click`, `ui_back`,
`ui_toggle`, `ui_slot` (hotbar), `inv_pick`, `inv_drop`, `map_open`,
`map_close`.

**Weather** (placed): `rain_hit_water` scattered on the sea near the
listener in rain, and the thunder and fire cues of section 6.

## 5. The ambience beds

A bed is a long stereo loop, unplaced, on the ambience bus. The bed rule,
`pbd_core::audio::bed_weights`, reads a `Surroundings` at 4 Hz:

| Input | From |
| --- | --- |
| height above ground, m | the eye against `surface_height` |
| airspeed, m/s | wind at the eye minus the listener's own velocity |
| rain, 0..1, and snowing | `Weather::liquid`, `Weather::snowing` |
| sheltered, underground depth, under water | section 3 |
| biome weights | `biome` at the eye's direction and a ring of 6 points 150 m out, so a border crossfades over about 300 m |
| sun elevation | `sky.rs` |
| distance to the shore | the nearest point within 200 m whose surface is below sea level (the same `surface_height` reads as `coast_walk`) |
| on open water | a boat and no land within 200 m |
| distance to a town | the nearest town centre against its radius (`towns.rs`) |
| air density | section 3 |

The beds and their rule:

- **Wind** (always, never zero below the air's top): `amb_wind_low` (a soft
  ground breeze, a rustle of leaves weighted by trees near) and
  `amb_wind_high` (an open, hollow wind), crossfaded by height above ground
  from 5 m to 80 m. Level and a small pitch range (0.9 to 1.15) follow
  airspeed, so flying fast is loud air.
- **Rain**: `amb_rain_light` and `amb_rain_heavy` crossfaded by rain, or
  `amb_rain_roof` when sheltered; none in snow (snow is the wind alone,
  quieter by 3 dB). The player's RAIN SHADOW setting turns off a picture,
  not a sound: rain is heard with it off.
- **Shore**: `amb_surf` by nearness to the shore within 200 m and height
  under 40 m, louder with wind; `amb_open_water` on a boat away from land.
- **Day** (by biome, weighted by the sun from 6 degrees below the horizon to 6
  above, and by `1 - rain`): Fields `amb_meadow_day` (birdsong), Jungle
  `amb_jungle_day`, Swamp `amb_swamp_day` (frogs, insects), Beach and Ocean
  `amb_gulls`, Desert `amb_desert_day` (sparse insects), Mountains and
  Tundra none (the wind carries them). Birds fade out above 60 m.
- **Night** (the same weights, the sun the other way): Fields and Jungle
  `amb_night_crickets`, Swamp and near fresh water `amb_night_frogs`, an
  `owl` scatter in Fields and Jungle; Desert, Mountains and Tundra only the
  wind. Night is quieter than day by 4 dB, never silent.
- **Towns**: `amb_town_day` and `amb_town_night` (distant work, a dog, a
  door) by distance to the town's edge within 150 m.
- **Cave**: `amb_cave` by underground depth (section 3).
- **Under water**: `amb_underwater` replaces every other bed.

Weights within a group sum to one, and a crossfade keeps loudness
(equal-power, `bed_db = volume_db + 10 log10(weight)`, Crab Breeze's rule). A
bed eases toward its target over 3 s, starts at a random point in its loop
(the seek happens on kira's streaming thread, not the main thread) and stops
below -40 dB. A **scatter** (an owl, a gull, a distant dog) is a short placed
cue started at random 30 to 200 m away at a seeded random interval, so the
bed is not one loop heard again.

Beds are made quiet and far (the `sound-effects` skill's softening): each is
compressed, low passed near 3.5 kHz, given a diffuse reverb and cut 11 dB down,
so it sits behind every placed sound.

## 6. Thunder, and the lightning-fire sounds

**Thunder.** The audio system keeps the step of the newest strike it has
heard and, each frame, takes every newer strike in `Air.now.strikes` within
`thunder_reach_m` (6 km along the ground). For each:

- the bolt is the segment from the cloud base over the strike to the ground
  under it;
- `near` is the distance from the eye to the nearest point of the bolt,
  `far` to the farthest end;
- the thunder starts `near / 343` s after the strike's flash time (the time
  `strike_lightning` dates it from), and rolls for about `(far - near) / 343`
  s, at least 2 s;
- the take is chosen by `near`: `thunder_crack` (under 400 m: a tearing crack
  and the boom on top of it), `thunder_close` (400 m to 1.5 km: a sharp clap
  and a roll), `thunder_roll` (past 1.5 km: a low roll), the one whose length
  is closest to the roll;
- its level is `distance_gain` with `full_within_m` = 150 m and
  `strength`'s 0..1 as +0 to +6 dB; its cutoff is section 3's air.

So a strike 3 km off flashes, and its roll arrives 8.7 s later, dark and long;
a strike 6 km off, past the 3 km reach of the flash, is heard and not seen.
Up to three thunders at once; the fourth is dropped. The rule is
`pbd_core::audio::thunder(eye, strike, cloud_base_m)`, tested on known
geometries (overhead, 1 km, past reach).

**A strike that hits something** (`lightning-fire` decides what was hit and
where). Within 400 m, `strike_hit_tree` (a crack, a split of wood, a sizzle)
or `strike_hit_ground` (a crack and a thud of earth) plays at the point hit,
delayed by its distance, together with the thunder.

**Fire** (needs `lightning-fire`'s fire state: a position, an intensity 0..1
and how fast rain is putting it out):
- `fire_ignite`: a whoomph as it catches, placed, once;
- `fire_crackle`: a loop at the fire, level by intensity, pitch a little
  lower for a bigger fire; the four nearest fires within 150 m are voiced,
  the rest silent;
- `fire_hiss`: a loop of steam, level by rain times intensity, so it swells
  as the rain hits and fades as the fire shrinks;
- `fire_out`: a last puff of steam and a pop when it goes out.

The interface asked of `lightning-fire`: a component or resource listing each
burning fire's body-local point, intensity and douse rate, and an event (or a
flag the audio can see change) when a fire starts and when it goes out. Until
that change lands, none of these play.

## 7. Where the sounds come from

The `sound-effects` skill, adapted here (provenance in
`.claude/skills/PROVENANCE.md`):

- **Recorded CC0 first**, fetched by `tools/audio/fetch_cc0.py` (which refuses
  a pack whose page does not say CC0 or public domain), cut, slowed, layered
  and filtered to a spec in `tools/audio/cc0_cues.json` by `cut_cc0.py`.
  Footsteps, digging, water, rain, wind, surf, birds, insects, fire and
  thunder are all well served by CC0 field recordings (OpenGameArt's CC0
  filter, Kenney's packs, Freesound's CC0 filter, Sonniss GDC bundles only if
  a bundle's licence is checked to allow it and is recorded as an exception
  the owner names).
- **Synthesised only where nothing recorded serves**, in `gen_sfx.py`: menu
  tones, the stall warning, and layers a recording lacks (a sub-bass under a
  close thunder crack).
- **Every original named**: `assets/audio/sfx/SOURCES.md` and
  `assets/audio/ambience/SOURCES.md` per cue, `AUDIO_SOURCES.md` per pack,
  written by the tools from the spec, never by hand. A credits screen lists
  any CC-BY pack the owner admits as an exception.
- **Formats**: OGG Vorbis, 48 kHz. Placed cues mono (q5), beds stereo (q4).
  Short cues are peak-normalised (-1 dBFS for impacts, about -9 for menu
  tones, -14 for beds); the mix between cues is the data's `volume_db`.
- **Judged by measurement** (`inspect_sfx.py`: clipping, loop seams, a
  spectrogram contact sheet), then by the owner's ears on the **Sounds page**
  (`docs/mockups/sounds.html`, every cue with a play button, published with
  its MP3s beside it and listed on the mockup tracker). Only the owner
  approves a sound.

## 8. Loading: before `READY`, never in play

A new loading step, `Sounds` ("TUNING THE AIR"), on the loading thread
before `Shaders`:

- every cue that is not a bed is decoded whole (`StaticSoundData`) from its
  OGG into memory;
- every bed is opened as a stream (`StreamingSoundData`), its first buffer
  read;
- the kira manager, the buses, the reverb and the listener are made.

All of it before `READY`. A cue asked for that is not ready is not played and
is logged `LATE_SOUND <cue>`, a bug to fix by adding it to the step, as
`LATE_PIPELINE` is. The first footstep in play costs what the hundredth does.

Memory: kira keeps a static sound as stereo f32 frames, 384 KB a second at
48 kHz. The first set (section 4) is about 150 short cues of 0.3 to 2 s,
roughly 120 s of sound, about 46 MB. Thunder takes (up to 12 s each) are
streamed, not static, with a pre-roll started at the strike so they are ready
when the delay ends. Budget: at most 64 MB static, logged at `READY`.

## 9. Testing without ears

Claude cannot listen, and the owner should not have to listen to find a
missing footstep. In order of cost:

1. **Unit tests in `pbd-core`** for every rule: distance gain and cutoff,
   the delay, the stride, the material families, the bed weights (a meadow at
   noon is birds and low wind; the same place at midnight is crickets; at 300
   m it is high wind only; a cave 10 m down is the cave bed and no rain),
   thunder timing and choice, the voice cap's choice, air density.
2. **Data against files** (`pbd-tools` or a test reading `assets/`): every
   cue the code names is in `cues.ron`, every file exists, placed files are
   mono, beds loop without a seam.
3. **`--audio-log`**: one line per voice started, stopped, dropped or late
   (`AUDIO start step_grass at 1.2 m gain -3.1 dB cutoff 19.9 kHz delay 0
   ms`), one line a second per bus with its level, and the bed weights when
   they change. A scripted run (`--walk`, `--coast-walk`, a capture path) with
   `--audio-log` is checked by reading it: did a step sound every stride, did
   the thunder start 8.7 s after a 3 km strike.
4. **Meters**: a small custom kira effect on each bus measures peak and RMS
   into atomics. `--audio-log` prints them; the F3 frame graph gets a row of
   bus meters, so a silent bus is seen.
5. **`--audio-wav PATH`**: a tap on `main` writes the mix, sample for sample,
   to a WAV. Under `--capture` the mixer runs on the mock backend, advanced
   exactly one frame's samples per captured frame (800 at 48 kHz and 60
   frames a second), and the presentation-only dice that place scatters and
   pick takes are seeded from a constant, so the WAV is in step with the
   frames, the same on every run, and needs no sound card: a cloud session can make a video with sound by
   muxing the frames and the WAV with ffmpeg. `inspect_sfx.py` and a
   spectrogram read the WAV like any other file.
6. **A real-time take** with `obs-record`: the owner's Mac records the window
   and the desktop's sound. Pale Blue Dot's `obs_record.py` has no sound step;
   Crab Breeze's adds the desktop's sound on macOS (`sck_audio_capture`, type
   0) for the take and removes it after. Task 6.3 ports that. A take runs with
   `PBD_TAKE_SOUND=1`, which plays at default volumes whatever the player
   saved (Crab Breeze's `CB_TAKE_SOUND`).
7. **The owner's ears** on the Sounds page and the take. The step's gate.

`--no-audio` turns the whole system off (no device opened), for an A/B of its
cost.

## 10. Cost

- **Audio thread** (kira's cpal callback, not a Bevy thread): 40 voices with
  a filter each, 10 streaming beds, one reverb, five meters. Budget: under 10 %
  of one core on the owner's Mac (M4 Max) and the desktop. Measured with the
  OS's per-thread CPU, logged in `--audio-log`.
- **Main thread**: the listener, voices' gains and cutoffs (a few hundred
  commands a frame into kira's ring buffers), the bed rule at 4 Hz. Budget:
  under 0.1 ms a frame. kira's command queues have fixed capacities
  (`sub_track_capacity`, `sound_capacity`), set from the voice caps so a full
  queue never blocks.
- **Disk**: about 25 MB of OGG for the first set.
- **The perf suite**: the change runs `tools/perf_suite.py` with sound on and
  `--no-audio`, interleaved, on the `--release` build (CLAUDE.md), and says
  what sound costs. Sound is on in `--frame-log` runs from then on, at default
  volumes, because players have it on.

## 11. Saves and settings

- **Saves**: nothing. Sound reads the world and writes nothing to it, so no
  save changes and every save opens as before (CLAUDE.md, "Saved games survive
  every change"). Scatter timing is presentation-only random, not the
  world's hash, and never chooses anything saved.
- **Settings**: `VERSION` 3 adds `volumes: (master, effects, ambience, music,
  menus)` in percent. A version 2 file reads with the defaults for the
  volumes and keeps everything else; an older build reading a version 3 file
  already keeps what it knows and ignores what it does not (the existing
  rule).

## Decisions

Every decision here is a recommendation taken (ask only with screenshots):
sound cannot be shown in a screenshot, so the owner judges them by ear on the
Sounds page and the first take, and any of them can be turned there.

1. **kira 0.12.6 directly**, not `bevy_audio`, `bevy_kira_audio` or
   `bevy_seedling` (section 1).
2. **Five volumes** with the defaults in section 2, in the player's file.
3. **Thunder is physical**: delayed at 343 m/s from the nearest point of the
   bolt, heard to 6 km, so strikes past the flash's 3 km reach are heard and
   not seen.
4. **Nothing outside is heard above the air**, and under water the world is
   one low pass: the two places the game's sound changes most.
5. **Rain is heard with RAIN SHADOW off**: that setting is a picture's cost,
   not the weather.
6. **Recorded CC0 first**, every original named, synthesised only where
   nothing recorded serves; no CC-BY without the owner naming the pack.
7. **Muted when the window loses focus.**
8. **No music in this change**; the bus waits for a music change.

## Risks / Trade-offs

- **kira 0.12 is an extra dependency with its own cpal**, and the first audio
  in the build. A device that fails, changes (headphones unplugged) or is
  absent must not take the game down: the backend error is caught, logged and
  replaced by the mock backend. A device change mid-play is a task (kira
  reports a stream error; the manager is rebuilt on the same buses).
- **Stereo panning only.** Front and back are told apart by a small low pass,
  not by HRTF. If a take shows that is not enough, Steam Audio or `oddio` are
  the next candidates (Crab Breeze's note), as their own change.
- **A hill does not block a sound.** On a planet whose horizon is 140 m from a
  standing eye, thunder over the curve is heard through the ground. The air's
  low pass hides most of it; a curvature term is cheap to add if a take shows
  it.
- **Beds by biome are a guess at taste.** Which birds a meadow has is the
  owner's to hear; the Sounds page plays each bed alone so they can be judged
  one by one.
- **The bed seek on a streaming thread** is kira's behaviour as read, not
  measured here. Task 4.2 measures the main thread when a bed starts part-way
  in; Crab Breeze's 20 to 43 ms stall is the number to beat.
- **Memory**: 46 MB of decoded short sounds is fine on the desktop and the
  Mac; if it grows, short cues can be stored mono and panned by kira, which
  halves it, at the cost of a custom `SoundData`.

## Open questions

None for the owner now. When the Sounds page exists, its first version is
the place to ask which takes and beds to keep, with each one playing beside
the question.
