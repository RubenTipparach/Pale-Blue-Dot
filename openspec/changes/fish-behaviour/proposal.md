## Why

The owner, 2026-11-17, watching the under-water video of a fish coming to the
lure (`fish-bite-underwater-before-after-2026-11-17-1080p60.mp4`): "the fish
behavior is super unnatural", "why is rotation all snappy?", "the fish is
glitching like hell".

The glitch (a fish flipping upright at the lure, its heading jumping 30 times
a second) is being fixed in game code by another session. This change is the
**behaviour**: how a fish should come to the lure, peck, take, swim and fight.
It is a visible change, so it is designed in a mockup first and the game waits
for the owner's approval of that mockup (CLAUDE.md, "Write it up before
touching code").

What reads as unnatural today, from the video and the code (design 1):

1. The fish is glued to the lure from the first nibble to the landing.
2. It flips nose-up or nose-down while the float dips, and swings to one fixed
   fallback heading when the lure is still.
3. Its heading jumps at the school's 30 Hz step.
4. It arrives in a straight line at full speed and stops dead; no approach.
5. A nibble is the float dipping; the fish does nothing.
6. At the bite the fish sinks with the float; no take.
7. Every fish swims at one speed with one tail beat; no glide, burst or bank.
8. On the line its facing follows the float's path: backwards while reeled,
   flipping when the float stalls; no head shakes.

## What Changes

- **Mockup M37, Fish Behaviour** (`docs/mockups/fish-behaviour.html`): the
  game's behaviour, ported constant for constant, beside a proposed one, in
  three tabs (At the lure, Swimming, Hooked), Today / Proposed / Split, 1x to
  1/4x, day and night, with every proposed value on a slider with its unit.
  https://claude.ai/artifact/6s47EX9mvDnLoWhQuqPHC2 (version 2, with the Body wave tab).
- **Proposed** (design 2 to 4), for the game once approved:
  - the fish at the lure comes in on a curve, slows, hovers a body length off
    and pecks: darts in so its mouth meets the lure as the float dips, shakes
    its head, tugs, backs off, and may turn away and circle back;
  - the take is a dart, a hard turn and a run that drags the lure and float;
  - a too-early strike or a missed bite makes it bolt;
  - on the line its mouth stays on the lure and its body swings round it:
    head away while it runs, head toward the rod with head shakes and rolls
    while it is reeled;
  - every fish's heading is smoothed every frame and rate-limited, it banks
    into turns, and its tail beat follows its speed;
  - schools swim in bursts and glides, turn at a limited rate and wander a
    little in depth.
- **Body wave** (design 8, M37 tab "Body wave", version 2): the fish's
  body bends in a vertex-shader wave running head to tail (today's fish and
  eel waves run tail to head), with its beat and amplitude set by speed, a
  straight body in a glide, a slow scull when hovering, head recoil, a C-bend
  into turns and normals that turn with the bend; an eel waves along its whole
  body and a ray undulates its wings.
- **Unchanged:** every fishing rule. The interest chance, the nibble count and
  timers, the dips, the hook window, what a strike does in each phase, the
  fight's runs, tension and stamina. Saves: schools are not saved; nothing
  here touches a save.

## Status

Drafted 2026-11-17. **M37 approved for the game by the owner, 2026-11-17: "New fish behavior approved"** (versions 1 and 2: the lure, swimming, hooked and body-wave proposals). Version 3 (the hooked snap) and "good implement", 2026-11-17: built on
`claude/fish-behaviour` (design 9).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `player/fishing`: how the fish at the lure and on the line moves (added
  requirements; the fishing rules are unchanged).

## Impact

- Draw side: `crates/pbd-app/src/fish.rs` (`draw::build`), `fish/bodies.rs`
  (`sway`), a new per-fish pose state for the drawn heading, roll and tail.
- Sim side: `crates/pbd-core/src/fauna/school.rs` (turn limit, glide and
  burst, depth wander, the lured fish's stand-off and arrival), with new
  values in `assets/config/fauna.ron` (`flock`, and a new `behaviour` block).
- Body wave: `fish/bodies.rs` (`emit` lays swimmers rigid, `sway` goes),
  `fish.rs` (`draw::build` writes the swim attributes), `field_light.rs` (a
  `SwimLit` extension with a vertex stage), a new `assets/shaders/swim.wgsl`.
- `crates/pbd-core/src/fishing.rs` is read, not changed.
