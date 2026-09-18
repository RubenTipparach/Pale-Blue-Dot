# Proposal: water is a wall, and it should be something you swim in

## Why

**The owner's report: "there's some weirdness with not being able to walk into
water. Need to make parity with tenebris-rs and allow swimming and diving."**

It is not weirdness, it is one clause. `walking.rs`'s swept ground resolution
treats a wet footprint exactly as it treats a cliff:

```rust
let (support, water) = footprint(&terrain, candidate);
...
if water || (rise > 0.03 && !can_step && ...) {
    // Keep the last accepted angular position ...
```

So the sea is a vertical wall at the waterline. You can look at it, the shader
draws it beautifully from both sides, the `wade` and `dive` capture presets
photograph it, and the player cannot reach any of it. The composite pass that
fogs the view underwater, the Snell's window, the emerge drips: all of it is
already built and none of it can be triggered by playing.

That clause was defensible when it was written, because there was no swim
model to hand the player to and walking into a sea with ordinary gravity would
have marched them along the seabed. What it needs is the model, not a better
wall.

## What Tenebris does, measured

Measured off `tenebris-rs` (`player_ctrl.rs`'s planet branch and
`assets/config/lod.yaml`). What is notable is how little there is: **no swim
state machine, no swim mode, no buoyancy force, no breath or drowning, no
camera change.** Three probes down one column and four multipliers.

| probe | height above the feet | what it gates |
| --- | ---: | --- |
| feet | 0.0 | nothing yet, reserved for footstep and dive sounds |
| body | **0.50 m** | speed, gravity, drag |
| eyes | **1.60 m** | the swim thrust, and `grounded` forced false |

| knob | value | effect |
| --- | ---: | --- |
| `water_movement_mult` | 0.50 | walk 8 to 4 m/s, sprint 14 to 7 |
| `water_gravity_mult` | 0.30 | 25 to 7.5 m/s^2 |
| `water_drag_per_s` | 3.0 | exponential, on the vertical too, where dry is ballistic |
| `water_swim_force_mps2` | 20.0 | continuous while the jump key is held and the eyes are under |
| `water_submerged_jump_mult` | 0.30 | a jump off the seabed is 12 to 3.6 m/s |

Those numbers make the feel, and it is worth stating it as a feel rather than
as a table: **hold the jump key to rise at about 4.2 m/s, release it and sink
at about 2.5 m/s.** There is no neutral buoyancy and nowhere to hover. Both are
the terminal speeds of `v' = (v - g_w dt) e^{-k dt}` with gravity at 7.5 and
drag at 3.0, and the 14 m/s clamp on the thrust never binds on a 1 g body.

Collision is the other half, and it is the part this project gets wrong:

- **Water is passable.** It never blocks a step and never counts as support.
- **The seabed is an ordinary walkable floor.** Their floor scan defines one as
  the top of a solid voxel with air *or water* above it, in those words, so
  standing on the bottom works and wading is just walking.
- **`grounded` is forced false whenever the eyes are under**, at every site
  that would otherwise ground the player, even standing on the seabed. That is
  what makes a submerged player always take gravity and never get the one-shot
  jump.

There is one thing we cannot port: their exit from deep water next to a bank is
**the jetpack**, unlocked the instant the eyes clear the surface, and this
project has no jetpack.

## What changes

1. **Delete the wall.** A wet footprint stops blocking tangential motion.
2. **Three probes and four multipliers**, as above, against the rendered water
   sheet radius rather than a voxel column, since this preview is a heightfield
   and the sheet is where the picture puts the surface.
3. **`grounded` is false while the eyes are under**, so a submerged player
   takes gravity and cannot jump.
4. **The exit, diverging on purpose.** With no jetpack, the swim thrust is
   gated on the **body** rather than the eyes when the player is not grounded,
   so that floating at the surface still rises and can clear a bank. Gated on
   the eyes alone, and with no jetpack behind it, a player in deep water beside
   a ledge would be stuck bobbing, which is the one thing the reference's
   design notes are careful to avoid.

## What is deliberately not ported

- **Breath, oxygen, drowning.** The reference has none. Unlimited submersion.
- **Buoyancy and surface snapping.** The reference has neither, and its one
  config key that sounds like it (`water_surface_float_height_m`) is parsed and
  never read. Do not implement one to match a value nothing reads.
- **The water current.** It rides their per-voxel flow automaton, which is
  `water-flow`'s subject and does not exist here yet.
- **Tangential drag.** Both engines set horizontal motion directly from input
  each frame rather than accumulating it, so a tangential damping rate has
  nothing to act on here. Only the vertical drag is ported, and the reason is
  recorded so nobody "restores" the other half.
