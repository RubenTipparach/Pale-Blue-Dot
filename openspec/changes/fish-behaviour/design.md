## Context

The fish that comes to the lure is a fish of a school (`fauna/school.rs`).
The school steps at 30 Hz (`spawn.step_hz`); the line (`fishing.rs`) steps with
it and, from the first nibble to the landing, sets `School::held` to the lure
point 0.35 m under the float. The drawing (`pbd-app/src/fish.rs`,
`draw::build`) places each fish at `School::drawn` (between its last two steps,
`movement-smoothness`) and turns it along `School::velocities[i]`, the last
step's velocity, with the vertical part halved. The tail is `bodies::sway`,
`Sway::Fish`: `sin(t * 7 + phase - x * 6) * 0.075 * w^2`, always.

Mockup: M37, `docs/mockups/fish-behaviour.html`,
https://claude.ai/artifact/6s47EX9mvDnLoWhQuqPHC2. Its Today side is a port of
the code above with the numbers of `assets/config/fauna.ron`; its Proposed
side is this design.

## 1. What is unnatural, in code terms

Frames of the owner's video at 1/4 speed show it, and Today reproduces it.

| What the owner sees | Why |
| --- | --- |
| The fish snaps onto the lure and stays there | `School::step`: a held fish's position is set to the lure. It is steered at the lure point itself (`flock.lure` 3.0) with a speed floor of half its slowest speed, so it arrives at full speed; the step that takes it inside `nibble_m` (0.45 m) moves it the rest of the way, about 13 m/s at 30 Hz. |
| It stands on its nose or tail at the float | A held fish's velocity is `(lure - position) / dt`, the lure's own movement. While the float dips (`nibble_dip_m`, `bite_dip_m`, eased at `dt * 14`), that is a small vertical vector; `normalize_or` makes it a unit vector, so the fish points straight up or down. When the lure is still the velocity is zero and the fallback is `up.any_orthonormal_vector()`, one fixed heading for every fish. |
| Rotation is snappy | Position is drawn between steps, heading is not: it changes at 30 Hz in jumps, and a boid that reverses does so in one step. |
| No approach, no pecks, no take | The nibble and the bite exist only as float dips; nothing in the fish's motion is tied to them. |
| Robotic swimming | Speed is clamped to the species' range and otherwise constant; the tail beat is fixed at 7 rad/s (1.1 Hz) and 0.075 of a body length whatever the speed; the body never rolls. |
| The fight looks wrong | On the line the fish is held at the lure again, so its facing is the float's direction of travel: backwards while reeled in, and a flip whenever the float stalls between run and rest. |

## 2. Proposed: the fish at the lure, phase by phase

The fishing rules decide when; the behaviour decides how. Every proposed
motion is read off the line's `Phase` and `timer`, so the timing a player
learns (watch for the third dip, strike when the float goes under) is
unchanged.

| Game phase or event | Proposed | Where |
| --- | --- | --- |
| `Event::Interest`, `Floating` with a target | Leaves the school and steers, turn-rate limited, to a stand-off point: on the side it came from, level with the lure, its mouth 15 cm off. Arrival: desired speed falls with distance inside a slow-down radius (1.3 m); no speed floor for this fish. Turns to face the lure as it settles. | sim: `School` steers the lured fish to the stand-off point, not the lure |
| `Floating` to `Nibble` | The rule (`distance < nibble_m`) is unchanged; the stand-off point is inside it. | none |
| `Nibble`, each timer | Hover: drift of a few cm on a slow cycle, tail sculling. Peck: start a dart `dart` (0.14 s) before the timer reaches 0.12 s, so the mouth meets the lure exactly when the game dips the float; head shake (12 deg at 7 Hz) and a tug of 2 cm on the lure while the dip lasts; back off to the stand-off point over 0.35 s. Between pecks, when the next timer leaves room (about 0.45 s), it may turn away and swim a small loop (radius 0.2 m) and come back facing the lure. | draw: a choreography keyed on `phase`, `timer`, the nibble index |
| `Event::Bite`, the hook window | The take: dart onto the lure (0.08 s), a hard turn (520 deg/s) and a run down (22 deg) and away from the angler. The drawn lure goes with the mouth, up to 0.3 m, and pulls the float aside on top of the game's 14 cm dip. | draw: the lure offset is drawn only; the rule's lure point is unchanged |
| `Event::TooEarly`, `Event::Missed` | Bolt: a burst away from the lure at 1.6 times top speed, easing over 1.2 s, then the fish rejoins the school's steering, still spooked (the game already frightens it). | draw, then sim (hand the drawn position and velocity back) |
| `Event::Hooked` | See 3. | |

## 3. Proposed: on the line

`Line` in `Hooked` already models runs (0.8 to 2.0 s at the species' `pull`)
and rests (1.2 to 3.0 s), tension, stamina and the float's swing across the
line. The fish is drawn from that, never changing it:

- its **mouth** is at the lure and its body swings round that point;
- **running** (`run > 0`): head away from the rod along the line, diving
  14 deg, full tail beat;
- **resting, being reeled** (`run == 0`): the hook turns the head toward the
  rod; head shakes (18 deg at 3.5 Hz) and a roll (30 deg) that flashes the
  flank;
- the turn between the two is rate-limited (300 deg/s) and eased (0.22 s), so
  a change from run to rest is a turn about the mouth, not a flip.

### 3.1 Nothing switches in one frame (the owner, 2026-11-17: "theres some snapping around in the hooked version")

A per-frame probe of the mockup's Hooked tab (Chrome on the GPU, 60 Hz, 60 s
at 1x with 26 run/rest switches, and 120 s at 1/4x) found three snaps in
Proposed, all draw side:

| Cause | Seen as | Count before (1x) |
| --- | --- | --- |
| The line's swing goes on or off in one step when a run starts or stops (`swing = sin(1.7 t) * 0.12 * run`); at 9 m of line the float, the lure and the mouth jump 0.9 m | the fish lurches sideways | 46 frames moving > 0.1 m |
| Shake and flash switched on and off with `run`, at full amplitude | head and roll jump up to 23 and 32 deg in a frame | 6 frames turning > 20 deg, 13 rolling > 20 deg |
| Shake and flash phased off the line's clock, which moves only on 30 Hz steps | the shake and roll stutter, held every other frame at 1x and for 8 frames at 1/4x | 252 frames turning > 10 deg at 1/4x |

So the fight's drawing is:

- **The drawn float** is the game's `DrawnFloat` (`movement-smoothness` 7.1):
  a jump of the line's float when `run` changes, and on the first hooked step
  after the strike, goes into an offset that is eased out, critically damped
  at 14/s. The mouth follows the drawn lure.
- **The turn rate is eased** as well as the heading: the rate wanted is
  `min(error / 0.22 s, sqrt(2 a error), 300 deg/s)`, and the rate moves toward
  it at no more than `a` = 1200 deg/s^2, yaw and pitch each (pitch at 0.6 of
  the cap). The rate a turn ends with carries into the next.
- **Shake and flash share one envelope**, eased toward 1 while reeled and 0
  while running with a 0.35 s time constant, multiplying both. Their phase is
  integrated per frame (`phase += 2 pi * 3.5 Hz * dt`), never read from the
  30 Hz line clock.
- **A shake turns the body about the mouth**, not its middle, so the mouth
  stays on the lure.

After: 0 frames over 20 deg (worst 10 deg, a 3.5 Hz shake at its fastest),
none rolling over 10 deg, no position step over 5 cm, at 1x; at 1/4x the worst
frame turns 2.5 deg. The float still changes speed in one step when the reel
is pressed or let go: that is `Line::step`'s rule, left alone.

## 4. Proposed: every fish

- **Heading** is a per-fish pose, turned every frame toward the drawn
  velocity by yaw and pitch, each eased (0.12 s) and rate-limited
  (260 deg/s), pitch held inside 35 deg. Turning by yaw and pitch, not about
  the cross product, means a half turn goes round level and never through
  nose-down.
- **Bank**: roll toward the inside of a turn, 14 deg at 90 deg/s, capped at
  28 deg, eased over 0.18 s.
- **Tail**: frequency `1.3 Hz + 1.0 Hz per body length a second`; amplitude
  0.025 hovering, 0.05 cruising, 0.11 bursting, 0.012 gliding (in body
  lengths, today's sway shape).
- **Sim side**: the flock step limits how far a fish's velocity turns in a
  step (220 deg/s); each fish swims a burst-glide cycle of its own (1.8 s,
  35% burst, glide drag 0.9/s) inside the species' speed range; and wanders
  up to 0.18 m in depth round the school's goal.

## 5. Port plan

**Draw side** (`pbd-app`): a `FishPose` per drawn fish (heading, roll, tail
phase and amplitude, yaw rate), kept in a component beside the school's mesh
and keyed by the fish's stable id so a fish removed from the school does not
shift another's pose; `draw::build` turns it every frame instead of taking the
last step's velocity; `bodies::sway` takes the tail phase and amplitude. The
choreography of the line's fish (2, 3) is a function of the line's phase,
timer, nibble index and the frame's lure, evaluated every frame. It needs the
nibble index and the time since the phase began on `Line` (read-only
accessors, no rule change).

**Sim side** (`pbd-core`, deterministic, 30 Hz): the lured fish steers to the
stand-off point with arrival and no speed floor; the turn limit, the
burst-glide cycle and the depth wander in `School::step`; values in a new
`behaviour` block of `fauna.ron`, with units. School state is not saved, so
no save format moves. A per-fish phase is hashed from the school's seed and
the fish's id, as its length is.

**Unchanged**: `Line::step` and its rules, `FishingSettings`, the hook
window, what a strike does, the events, the HUD and the sounds that hang off
them.

**Interaction with `movement-smoothness`**: its requirement "a fish held at
the lure SHALL be drawn at the lure" becomes, once this lands, "its mouth is
at the lure"; that change's spec is updated in the same commit as the game
code.

## 6. Risks

- The nibble dip is detected per 30 Hz step; the peck is timed against the
  frame's interpolated timer, so the mouth may meet the lure up to a step
  early or late. At 1/4 speed in the mockup it lines up.
- The loop between pecks needs about 0.45 s of gap; with `nibble_s` at its
  shortest (0.6 s) there is no loop, only hover. That is the timing rule, left
  alone.
- The tug and the take move the drawn lure and float, not the rule's. A
  multiplayer peer sees the same drawn motion only if the choreography stays a
  pure function of line state, which this design keeps.

## 7. Open questions

Asked only with the mockup beside them (CLAUDE.md, "Ask the owner with a
survey"): the owner's review of M37 on the tracker answers whether the
proposed behaviour is right and which slider values to take.

## 8. The body wave (M37, tab "Body wave")

The owner, after PR #109 fixed the heading: "the fish now turns correctly, but
for some reason doesnt move convincingly? perhaps its how the model just turns
without matching proper fish movement, make a mockup for using shaders to
twist a fish around when its swimming so it moves convincingly".

### 8.1 Today

`bodies::sway` adds a sideways shear to the model on the CPU in `emit`, and
`draw::build` makes flat normals from the result. Per kind:

| Sway | Offset (model units, body length 1) | Beat |
| --- | --- | --- |
| `Fish` | `Z * sin(t*7 + phase - x*6) * 0.075 * w^2`, `w = clamp((0.3 - x)/0.8, 0, 1)` | 1.11 Hz, always |
| `Eel` | `Z * sin(t*3.5 + phase - x*13) * 0.06 * w`, `w = clamp((0.55 - x)/1.0, 0.15, 1)` | 0.56 Hz, always |
| `Ray` | wing `Y * 0.07 sin(ph - 2.5 s) (|z|/0.44)^2`, whip behind `s > 1` | 0.25 Hz, always |

Two things make it read as a turning plank with a wagging tail:

- **The fish and eel waves run backwards.** A crest of `sin(7t - 6x)` sits at
  `x = (7t - c)/6`, which grows with `t`: it moves toward +X, the head. A
  swimming fish sends its wave from head to tail. (The ray's `sin(ph - 2.5 s)`
  already runs nose to tail.)
- **Nothing follows the swim.** The beat and amplitude never change with speed,
  the body never straightens in a glide, never curves into a turn, the head
  never recoils, and the shear moves vertices straight sideways, so a swung
  tail is thinner than the body it came from.

### 8.2 Proposed: the vertex shader

`u` is the place along the body, 0 at the nose to 1 at the tail (beyond 1 a
ray's whip); `d` the vertex's distance off the spine across the body; `phi`,
`A`, `B` the fish's wave phase (rad), amplitude and turn bend (body lengths).
Per species: `lambda` waves along the body, envelope `h`, `p`, recoil `r`
reaching `ur` from the nose.

```text
E(u)  = h + (1 - h) u^p                          amplitude envelope
R(u)  = -r (max(1 - u/ur, 0))^2 sin(2 pi lambda - phi)   head recoil, against the tail
q     = (u - 0.35) / 0.65                        C-shape about the pivot
f(u)  = A (E sin(2 pi lambda u - phi) + R) + B q^2      spine offset
f'(u) = A (E' sin(.) + E 2 pi lambda cos(.) + R') + 2 B q / 0.65
s = -f'/L   c = 1/sqrt(1 + s^2)                   slope dz/dx (x runs nose + to tail -)
x' = x - d s c     z' = f + d c                    the cross-section turns with the spine
n' = (n.x c - n.z s c, n.y, n.x s c + n.z c)       the normal turns by the same angle
```

Crests sit at `u = (phi + pi/2 + 2 pi k) / (2 pi lambda)`, which grows with
`phi`: the wave runs head to tail. A ray instead lifts its wings,
`y += A (|d|/span)^pw sin(2 pi lambda_w (1 - flap) u - phi)`, the wing inside
a turn beating less, the whip swinging sideways, and the normal taking the
height field's slope, `n' = normalize(n.x - g_x n.y, n.y, n.z - g_z n.y)`.
The mockup checks `f'` against a numerical derivative (error < 1e-9).

WGSL, as the life material's vertex stage would carry it:

```wgsl
struct Swim { u: f32, d: f32, phase: f32, amp: f32, bend: f32 }
struct Profile { waves: f32, expo: f32, head: f32, recoil: f32, recoil_u: f32, spine: f32 }

fn swim_lateral(s: Swim, k: Profile) -> vec2<f32> {
    let u = clamp(s.u, 0.0, 1.0);
    let um = max(u, 1e-3);
    let e = k.head + (1.0 - k.head) * pow(um, k.expo);
    let de = (1.0 - k.head) * k.expo * pow(um, k.expo - 1.0);
    let a = TAU * k.waves * u - s.phase;
    let tail = sin(TAU * k.waves - s.phase);
    let r = max(1.0 - u / k.recoil_u, 0.0);
    let q = (u - 0.35) / 0.65;
    let f = s.amp * (e * sin(a) - k.recoil * r * r * tail) + s.bend * q * q;
    let df = s.amp * (de * sin(a) + e * TAU * k.waves * cos(a) + 2.0 * k.recoil * r / k.recoil_u * tail)
        + s.bend * 2.0 * q / 0.65;
    return vec2(f, df);
}

// fwd, side: the fish's axes in mesh space, scaled by its length; the vertex
// was laid rigid by `emit`.
fn swim_body(p: vec3<f32>, n: vec3<f32>, fwd: vec3<f32>, side: vec3<f32>, s: Swim, k: Profile)
    -> array<vec3<f32>, 2> {
    let fl = swim_lateral(s, k);
    let slope = -fl.y / k.spine;
    let c = inverseSqrt(1.0 + slope * slope);
    let sn = slope * c;
    let p2 = p + side * (fl.x + s.d * (c - 1.0)) - fwd * (s.d * sn);
    let nf = dot(n, normalize(fwd));
    let ns = dot(n, normalize(side));
    let n2 = n + normalize(fwd) * (nf * (c - 1.0) - ns * sn) + normalize(side) * (nf * sn + ns * (c - 1.0));
    return array(p2, normalize(n2));
}
```

### 8.3 Speed, glide, hover and turn (draw side, CPU)

Per drawn fish, kept in the `FishPose` of 5 beside its heading and roll:

- **Phase** integrated, `phi += 2 pi f dt`, never `omega t`, so a change of
  speed never jumps the tail. `f = f0 + fk * (speed / length)`, capped at
  `fmax`; hovering, `f = f_hover`.
- **Amplitude** eased (0.18 s) toward: hovering below `hover_below` lengths a
  second; gliding when slowing faster than `glide_acc` lengths a second
  squared (`A_glide`, near zero, so the body straightens); otherwise between
  cruising and bursting by speed across the species' range.
- **Bend** eased (0.15 s) toward `-bend_gain * yaw_rate / 90 deg/s`, capped at
  `bend_max`; a left turn bows the body left.

All of it reads the speed, its change and the yaw rate, which `draw::build`
already has per fish (`School::velocities`, `DrawnHeadings`).

| Default | Perch (carangiform) | Eel (anguilliform) | Ray (rajiform) |
| --- | --- | --- | --- |
| `f0`, `fk`, `fmax` | 0.5 Hz, 1.0 Hz per len/s, 8 Hz | 0.4, 1.0, 4 | 0.4, 0.6, 3 |
| `A` cruise, burst, hover, glide | 0.075, 0.11, 0.025, 0.004 len | 0.07, 0.09, 0.03, 0.01 | wing tip 0.08, 0.12, 0.03, 0.006 |
| `h`, `p`, `lambda` | 0.04, 2.0, 0.95 | 0.3, 0.8, 1.25 | wing `pw` 1.6, 0.7 waves a chord |
| recoil, reach | 0.35, 0.4 | 0, - | - |
| bend at 90 deg/s, most | 0.06, 0.14 len | 0.10, 0.22 | 0.04, 0.10 |
| glide threshold, hover below | 0.35 len/s2, 0.4 len/s | 0.3, 0.3 | 0.3, 0.3 |

At a perch's cruise (0.8 m/s, 2.1 len/s) the beat is 2.6 Hz and the wave runs
back at `f L / lambda` = 1.05 m/s, a slip of 0.76; at its burst (2.4 m/s) 6.8 Hz
and 2.7 m/s, 0.88. Real carangiform slip is about 0.7 to 0.9.

### 8.4 Where it goes in the game

- **The mesh** stays one mesh a school, rebuilt each frame by `draw::build`,
  but `emit` lays each creature rigid (no `sway`) and writes, per vertex, two
  more attributes: `swim_axes: vec4` (the fish's `side * scale` and `u`) and
  `swim_wave: vec4` (`phi`, `A`, `B`, `d`), with the forward axis recovered as
  `cross(side, up)` from a third, `swim_up: vec4` (`up * scale` and the
  profile index). That is 48 bytes a vertex on top of the 48 the mesh carries
  today (position, colour, uv, normal): a 10-fish perch school, about 7,200
  vertices, uploads about 350 KB a frame more. `compute_flat_normals` runs on
  the rigid mesh, which is the input the shader turns.
- **The profile** (`lambda`, `h`, `p`, recoil, span, wing values) is a small
  uniform array in the material, indexed by the profile index, one row per
  sway kind and species, filled from `fauna.ron` at startup.
- **The material**: the life material is `ExtendedMaterial<StandardMaterial,
  FieldLit>` (`field_light.rs`, fragment `field_lit.wgsl`). A swimmer's
  meshes get their own extension, `SwimLit`, with the same fragment shader and
  a `vertex_shader()` (and `prepass_vertex_shader()`, so the shadow and depth
  prepass bend the same way) that calls `swim_body` before Bevy's
  `mesh_position_local_to_world`. Its `specialize` adds the three attributes
  to the vertex layout; the reef, kelp and places keep `FieldLit`. The
  loading screen prewarms the new pipeline (CLAUDE.md, "A loading screen on
  boot").
- **Crabs, birds, turtles, otters, octopus** keep their CPU part rig
  (`animate`); their `swim` attributes are zero and the shader leaves them as
  they are (`A = 0`, `B = 0`).
- **A later step**, not needed for this: one instanced base mesh per species
  with a per-fish instance buffer (transform, `phi`, `A`, `B`) would drop the
  per-frame rebuild altogether.

**Cost**: vertex stage only, two `sin`, one `cos`, two `pow` and about 40
ALU a vertex. At 7,200 vertices a school and a few schools in view that is
tens of thousands of vertices: a few microseconds on the RTX 3070, under the
noise of `perf_suite.py`. It is measured with the suite when built, not
claimed (CLAUDE.md, "Implementation and verification").

**Not changed**: where a fish is, its heading, its school's rules, the
fishing rules. The bend never moves the fish's position or facing, only its
body round its spine.

**Waits** for the owner's approval of M37's Body wave tab; no game code until
then and a request.


## 9. Built (2026-11-17, branch `claude/fish-behaviour`)

The owner, on M37 version 3: "good implement". Built as sections 2 to 4 and
8 say, with these changes found in the porting (each is the design now):

- **The line steps every frame**, not at the school's 30 Hz: `fish()` calls
  `Line::step` with the frame's `dt`. So the peck is timed against the
  line's own timer, read through the new read-only `Line::timer()`, and risk
  1 of section 6 does not arise. The dip's 0.12 s is now the named
  `fishing::NIBBLE_DIP_S` (the same value; no rule changed).
- **Where it lives.** The pose, the line's fish's choreography, the body
  wave's CPU twin and the drawn float are `pbd_core::fauna::swim` (engine
  free, testable without Bevy: `Pose`, `LineFish`, `Drawing`, `DrawnFloat`
  moved from the app). The app drives `Drawing::frame` once a frame before
  the school meshes are built (`fish::draw::schools`) and keeps it in
  `DrawnSwim`, rebased with the fishery (`world-frames`).
- **The sim-side values are in `flock`**, not the new `behaviour` block:
  `turn_deg_s`, `burst_s`, `burst_share`, `glide_drag_per_s`,
  `depth_wander_m`, and the approach's `approach`, `slow_m`, `standoff_m`,
  `standoff_max_m`. That keeps `School::step`'s signature, so the fishing
  tests run as they are. The draw-side sliders are `behaviour` (`pose`,
  `lure`, `fight`, and a wave per kind). Both validated, with units.
- **A long fish still nibbles.** The rule starts a nibble when the fish's
  middle is inside `nibble_m` (0.45 m). The sim's stand-off is the mouth
  15 cm off, but the middle never more than `standoff_max_m` (0.36 m) off,
  validated inside `nibble_m`; an eel or a serpent stops there. The drawn
  choreography uses the true stand-off (mouth 15 cm off), so a long fish
  backs off a little as its first nibble starts.
- **The choreography keeps state** (where a peck started, the loop, the
  take's drag): draw state, stepped from the line's phase, timer and run
  each frame. A peer sees the pecks land on the same dips; the hover drift
  between them may differ (section 6, risk 3).
- **The bolt** owns its fish (`School::owned`) for 1.2 s and hands it back
  with `School::place` where it was drawn. Schools are not saved.
- **Tail numbers**: design 8.3's wave (a profile per kind: perch, eel, ray)
  replaces design 4's tail line, since `sway` is gone.
- **The shader.** One body-wave table row per model (`bodies::wave_rows`),
  its spine and span; the per-vertex attributes are in body lengths (the
  axes scaled to metres a spine), so the shader needs no spine. The table is
  a storage binding (106) of `FieldLit`, written once. `FieldLit::specialize`
  gives `swim_vertex.wgsl` only to a mesh with the swim attributes; every
  other field-lit mesh keeps Bevy's vertex stage. The game has no depth or
  normal prepass and fish are not sun-cascade casters, so there is no second
  vertex stage to bend; a prepass added later must take the swim vertex too.
  The school pipeline is warmed on the loading screen (a hidden school mesh
  with the attributes); a real-time run logs no `LATE_PIPELINE`.
- **Meshes drawn without the attributes** (the sea's places, the lineup)
  are bent on the CPU by the same wave, cruising on the world clock. A seal
  (a floater) cruises the same way.
- **The lure is drawn**: a leader on from the float to a baited hook (M37's
  Proposed), the float leaning toward a lure the fish tugs or drags.
- **The angler** of `--fish-play` strikes 0.3 s into a bite, as M37's does.

### 9.1 Measured (M4 Max, real time, windowed, `--fish-play --hold-pointer
--watch-hook --time 10 --no-vsync`, 60 s, fresh worlds; the machine was busy
with other agents' builds and games, 15 to 60 fps)

`PBD_MOTION_TRACE` and `tools/motion_graph.py` (the fish the line works, by
phase; before = `main` with the instrument alone, `104de7a8`):

| | before | after |
| --- | --- | --- |
| move on the frames a nibble starts | 1.8, **41.8**, 2.1, **34.9** cm | 0.5, 0.4 cm |
| moves over 5 m/s, nibble | 2 | 0 |
| up flips over 20 deg, approach | 3 (worst 113 deg) | 0 (worst 2.3) |
| nearest mouth to lure in each dip | 26.0, 49.0 cm | 1.1 cm |
| turns over 1000 deg/s, any phase | 1 | 0 |
| hooked: frames over 20 deg | 0 (never turned: held) | 14, all on frames of 40 to 120 ms |

On the line the fish now turns (300 deg/s and an 18 deg shake at 3.5 Hz):
on a slow frame that is over 20 deg in one frame, never over 1000 deg/s.
The core test runs the same at 60 frames a second and finds no frame over
20 deg in any phase. Hooked, the fish's fast moves are the drawn float's
(19 of 26 frames over 5 m/s are the float's own; before, 11 of 11): the
float itself steps up to 30 cm a frame while reeled, which is the line's
place (`feet + away * line_m`), left alone here and worth its own look.

### 9.2 The video

`fish-behaviour-before-after-2026-11-17-1236p60.mp4` (Drive,
`pale-blue-dot-validation`; `output/videos/`): `main` with the instrument
(BEFORE) and this branch (AFTER), each `--fish-play --hold-pointer
--watch-hook --time 10 --no-vsync` on a fresh world, recorded with OBS; each
from the approach to the start of the fight at 1x, then side by side aligned
on the strike at 1x and at 1/4 speed. Limits, as seen frame by frame:

- The fish differ: on `main` an eel came, on the branch a smaller fish (the
  sim's turn limit and bursts change which fish of which school arrives
  first). Each nibbled once before its bite, so one peck each, not three.
- The machine was busy with other agents' builds and games: the recordings
  have a median of 39 (before) and 47 (after) new frames a second.
- The watch camera stays where the float landed; on the branch the run took
  the fish and float out of frame within a second of the strike, so the
  fight is short on screen. The core test covers the fight frame by frame.
- On `main` the strike throws the eel and the float at the camera in one
  frame (the pop of 9.1); on the branch the take drags the lure down and
  aside and the fish turns away with it.
