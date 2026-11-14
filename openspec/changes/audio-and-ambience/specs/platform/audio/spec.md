## ADDED Requirements

### Requirement: The game plays sound through buses the player can set
The game SHALL play every sound through one of four buses under a master:
effects and ambience (together under a world bus that carries the
environment's filter and reverb), music, and menus. The player's five volumes
SHALL each scale their bus, 0 % SHALL be silent, and a change SHALL be heard
at once. Pausing SHALL pause the world bus and leave the menus sounding.

#### Scenario: Effects off
- **WHEN** the player sets the effects volume to 0 %
- **THEN** no placed sound is heard and the ambience and menus still are

#### Scenario: Paused
- **WHEN** the player pauses with rain falling and a fire burning
- **THEN** the rain and the fire stop at once and resume where they were when play resumes, and the pause menu's clicks are heard

### Requirement: The game runs without a sound device
When no audio device can be opened, the game SHALL log the reason, run the
mixer without output, and play on silently. `--no-audio` SHALL turn the audio
system off entirely.

#### Scenario: A container with no sound card
- **WHEN** the game starts where no audio output exists
- **THEN** it logs `AUDIO off` with the reason and reaches `READY` as it would with sound

### Requirement: Placed sounds are heard from where they happen
Every sound but a menu sound or an ambience bed SHALL play from a body-local
point, given to the audio engine relative to the active camera's eye in the
camera's frame. Its gain SHALL be full within its cue's `full_within_m` and
fall as `full_within_m / d` beyond it, and it SHALL NOT be started past its
cue's `audible_m`. It SHALL be low passed by distance, and it SHALL start
`d / 343` s after its cause when it is more than 10 m away.

#### Scenario: Doubling the distance
- **WHEN** the same cue plays at 20 m and then at 40 m with `full_within_m` of 5 m
- **THEN** the second is 6 dB quieter and its low-pass cutoff is lower

#### Scenario: An origin rebase
- **WHEN** the floating origin moves while a fire crackles 30 m from the player
- **THEN** the fire is heard from the same place, at the same level, before and after

### Requirement: Thunder arrives at the speed of sound
For every lightning strike within 6 km along the ground, thunder SHALL start
after the distance from the eye to the nearest point of the bolt divided by
343 m/s, measured from the strike's flash time, and its take SHALL be chosen
by that distance (a crack under 400 m, a clap to 1.5 km, a roll beyond). At
most three thunders SHALL play at once. A strike beyond the reach of its flash
SHALL still be heard.

#### Scenario: A strike 3 km off
- **WHEN** a strike lands 3 km from the player, its bolt's nearest point 3 km away
- **THEN** its thunder starts 8.7 s after the flash, as a low roll

#### Scenario: Overhead
- **WHEN** a strike lands 100 m from the player
- **THEN** the crack is heard within 0.3 s of the flash

### Requirement: A lightning fire is heard while it burns
When `lightning-fire` reports a fire, the game SHALL play its ignition once
at the fire, a crackle loop at a level following the fire's intensity, a hiss
loop following rain times intensity while rain douses it, and a last puff of
steam when it goes out. At most the four nearest fires within 150 m SHALL be
voiced.

#### Scenario: Rain puts a fire out
- **WHEN** rain falls on a burning tree until it goes out
- **THEN** the crackle falls, the hiss rises and then falls with it, and the steam puff plays once as the fire ends

### Requirement: Footsteps, digging and landing sound like the material
A grounded walker SHALL play one step every stride of distance walked, chosen
from the surface family of the block under its feet (stone, dirt, grass, dry
grass, sand, snow, wood, shallow water), with a wet layer on dirt, grass and
stone when the ground's wetness is over 0.4. Digging, breaking and placing a
block SHALL play that block's family. A landing SHALL play at a level set by
the speed of the fall.

#### Scenario: Sand, then stone
- **WHEN** the player walks from a beach onto rock
- **THEN** the steps change from the sand family to the stone family at the first step on rock

#### Scenario: Walking speed
- **WHEN** the player walks 8 m on flat ground at walking speed
- **THEN** ten steps play

### Requirement: The ambience follows where the listener is
Ambience beds SHALL be chosen and weighted by a pure rule of the listener's
surroundings (design section 5): wind by height and airspeed, rain by its
intensity, surf by the shore, day and night beds by biome and the sun, towns
by distance, a cave bed by depth, and only the underwater bed below the
water's surface. Beds within a group SHALL crossfade at equal power and ease
to their targets without a step in level.

#### Scenario: A meadow, day and night
- **WHEN** the listener stands in the Fields biome at noon and then at midnight with no rain
- **THEN** at noon the meadow birdsong and the low wind play, and at midnight the crickets and the low wind play, quieter

#### Scenario: Under a roof
- **WHEN** the player steps out of heavy rain into a town building
- **THEN** the open rain bed gives way to rain on a roof, and the outdoor beds are muffled

#### Scenario: Into a cave
- **WHEN** the player digs down 10 m in rain
- **THEN** the rain and the birds have faded out and the cave bed plays

#### Scenario: Under water
- **WHEN** the player's eye goes below the sea's surface
- **THEN** only the underwater bed plays, and placed sounds are low passed to 600 Hz

### Requirement: Rain is heard whatever the RAIN SHADOW setting
The rain beds SHALL follow the weather alone. The player's RAIN SHADOW
setting, which turns off the rain volume's picture, SHALL NOT change them.

#### Scenario: RAIN SHADOW off in a storm
- **WHEN** RAIN SHADOW is off and heavy rain falls on the player
- **THEN** the heavy rain bed plays as it does with RAIN SHADOW on

### Requirement: Nothing outside is heard above the air
Outside sounds and beds SHALL fade with the air from 600 m above the surface
to silence at the top of the atmosphere, and above it only the player's own
craft SHALL be heard, through its structure.

#### Scenario: Climbing out
- **WHEN** the player flies from the ground to above the atmosphere in rain
- **THEN** the rain and wind fade to nothing by the atmosphere's top and the craft's own loops are still heard, muffled

### Requirement: Every sound is ready before play
Every short cue SHALL be decoded and every bed opened on the loading screen
before `READY`. A cue asked for in play that is not ready SHALL NOT decode on
the main thread; it SHALL be skipped and logged as `LATE_SOUND <cue>`.

#### Scenario: The first footstep
- **WHEN** the player takes the first step after `READY`
- **THEN** no sound data is decoded on the main thread and no `LATE_SOUND` is logged

### Requirement: The cue data matches the files and the code
Every cue the code names SHALL be in `assets/config/audio/cues.ron`, every
file it names SHALL exist, and every placed cue's file SHALL be mono.

#### Scenario: A missing cue
- **WHEN** code asks for a cue name that `cues.ron` does not have
- **THEN** a test fails naming the cue

### Requirement: Sound can be checked without listening
`--audio-log` SHALL log every voice started, stopped, dropped or late with its
cue, distance, gain, cutoff and delay, and each bus's peak and RMS once a
second. `--audio-wav PATH` SHALL write the mix to a WAV file; under
`--capture` the mixer SHALL advance exactly one frame's samples per captured
frame, so the file is the same on every run and in step with the frames.

#### Scenario: Two captures, one sound
- **WHEN** the same `--capture` run with `--audio-wav` is made twice
- **THEN** the two WAV files are identical and as long as the captured frames
