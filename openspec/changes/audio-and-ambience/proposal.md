# audio-and-ambience: sound effects and ambience, placed on the planet

## Why

The owner, 2026-11-14: "using skills from fleet command and crab breeze - spin
up a audio sfx and ambient system session, and have it design audio". In the
same conversation the owner asked for lightning that strikes trees or ground
and starts a short fire the rain puts out in a few seconds (`lightning-fire`,
written separately), whose sounds are designed here: the strike's crack, the
thunder rolling in late by the distance at the speed of sound, the fire's
crackle and the hiss as the rain douses it.

Pale Blue Dot has no sound at all. The workspace builds Bevy with
`default-features = false` and never turns `bevy_audio` on; there is no
`assets/audio/`. A storm is a flash and silence, a footstep on sand and on
stone are the same nothing, and flying through a cloud at 80 m/s is as quiet
as standing in a field at night.

Crab Breeze, the owner's other Bevy game, has had sound since 2026-10-25 and
Fleet Command since 2026-11-04, both by the same method: recorded CC0 sound
first, every original named, placed sounds mono and in 3D, judged by
measurement because a Claude session cannot listen. This change brings that
method here (the `sound-effects` and `music-authoring` skills, copied with
provenance) and designs the game side for a small spherical planet.

## What Changes

- **An audio engine.** kira 0.12, used directly from `pbd-app`, not through a
  Bevy plugin (design section 1). `pbd-core` gains an engine-free `audio`
  module with every rule (distance, delay, air absorption, bed weights,
  stride, thunder timing), tested there.
- **Buses and five volumes.** Master, effects, ambience, music and menus, set
  by the player in the settings menu and kept in the player's settings file
  (version 3). Music is a bus with a volume and nothing on it yet.
- **Placed sound in the camera's frame.** Every effect but a menu sound plays
  from a body-local point, given to the engine relative to the camera's eye.
  Its level falls with distance by our own law, it is low passed by the air
  it crosses, and it starts after the time sound takes to arrive (343 m/s).
- **Ambience beds** chosen by where the listener is: wind by height and
  airspeed, rain by intensity (on a roof when sheltered), surf near the shore,
  open water from a boat, birds and insects by biome and the sun, the night
  roster, towns, caves, and a muffled world under water. Nothing outside is
  heard above the air.
- **Event sounds**: footsteps by the material underfoot (wet when the ground
  is wet), dig, break and place by material, jump and land, swimming, every
  craft's own loops, fishing, menus, thunder, and the lightning-fire sounds.
- **Sourcing**: recorded CC0 first, cut and layered to a spec, synthesised
  only where nothing recorded serves, every original named in
  `AUDIO_SOURCES.md` (the `sound-effects` skill). A Sounds page plays every cue
  for the owner, on the mockup tracker.
- **Loading**: every short sound decoded and every bed opened on the loading
  screen, before `READY`; a sound first decoded in play is logged as a bug,
  as a late pipeline is.
- **Instruments, since nobody here can listen**: `--audio-log` (every voice
  started, stopped or dropped, and each bus's level once a second), level
  meters on every bus, `--audio-wav PATH` (the mix written to a file, sample
  for sample, in step with `--capture` frames), `--no-audio`, and a recorded
  take with sound through `obs-record`.
- **The skills**: `.claude/skills/sound-effects` (from Fleet Command) and
  `.claude/skills/music-authoring` (Crab Breeze's text with Fleet Command's
  lessons), with `.claude/skills/PROVENANCE.md`. These are in this commit.

## Capabilities

### New Capabilities
- `platform/audio`: the audio engine, buses and volumes, placing a sound on
  the planet, the ambience beds, the event sounds, loading before play, the
  cost budget, and the audio instruments.

### Modified Capabilities
- `platform/settings`: five volume settings, file version 3.

## Impact

- `Cargo.toml`: `kira = "=0.12.6"` (no default features; `cpal`, `ogg`).
  Bevy's own `bevy_audio` stays off.
- `crates/pbd-core/src/audio.rs` (new): the rules and the data's types.
- `crates/pbd-app/src/audio/` (new): the engine, the listener, voices, beds,
  and the systems that turn game state into sound.
- Hooks, read-only into existing state: `walking.rs` (stride, jump, land,
  swim), `desktop/digging.rs` (hit, break, place), `weather.rs` and
  `pbd_core::atmosphere` (rain, strikes), `planet_contact.rs` (open to sky),
  `planet_water.rs` and `sea.rs` (the sea under the eye), `pbd_core::wind`,
  `sky.rs` (the sun), `towns.rs`, `vehicles/`, `fish.rs`, `desktop/menu.rs`,
  `loading.rs` (a sounds step and `LATE_SOUND`), `settings.rs` (version 3).
- `assets/audio/` (new): `sfx/`, `ambience/`, each with `SOURCES.md`;
  `AUDIO_SOURCES.md` at the root. `assets/config/audio/` (new): `cues.ron`,
  `ambience.ron`, `mix.ron`.
- `tools/audio/` (new, ported from Fleet Command `08374c4`).
- No change to any save. Sound is presentation: it reads the world and writes
  nothing to it.
- Depends on nothing unbuilt. `lightning-fire` supplies the fire's state;
  until it lands the fire sounds have nothing to play from and the thunder
  plays alone.
