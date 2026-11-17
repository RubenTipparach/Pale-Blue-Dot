# Design: rod motion and the hand's faces

## 1. The hand's faces

`prism` builds a hexagonal ring of half-widths `w` (along `u`) and `r`
(along `v`). The side face between ring corners `k` and `k+1` gets the normal
`normalize((lo[k1] - lo[k]) × axis)`, flipped if it points toward the
prism's axis line. Shading stays `0.62 + 0.38·max(n·light, 0)` against the
same eye-space light, as the mockup's `prism()` does. Caps keep `±axis`. When
`w == r` the new normal equals the old one, so every finger is unchanged, and
a test pins both facts.

## 2. Where the rod's motion comes from

The line's `Phase` (`pbd_core::fishing`), its charge, its tension and its
events (`Cast`, `Nibble`, `Bite`, `Hooked`, `Caught`, `Snapped`, ...) are
already on the `Fishery` resource each frame. The rod's pose is a function of
these and of time since the last event. It is computed in the app, drawn only,
and never read back by a rule.

The pose is an offset applied to the rod's rest pose, like the chop: a pitch
about the fist (rod tip up and back is positive), a small roll, and a shift
of the fist.

| Phase / event | Pitch about the fist | Other |
| --- | --- | --- |
| Charging | rises to `+draw_rad` (0.9) with the charge, eased | fist up and back `draw_lift_m` (0.05) |
| `Cast` | from the drawn pitch to `-flick_rad` (-0.35) in `flick_s` (0.14 s), then springs back to the fishing pose | |
| Flying / Floating | fishing pose `-settle_rad` (-0.12), a slow bob | idle sway as today |
| `Nibble` | a twitch of `nibble_rad` (0.05) for 0.15 s | |
| `Bite` | a dip of `-bite_rad` (-0.22) at once, shaking at 14 Hz for the window | |
| Hooked | up to `+fight_rad` (0.45), shaking `fight_shake_rad` × tension at 9 Hz; while reeling, the fist turns a small circle (`crank_m` 0.012 at 3 Hz) | roll toward the fish's pull, up to 0.15 rad |
| `Caught` | a lift of +0.6 that eases back to rest over 0.6 s | |
| `Snapped`, `Missed`, `TooEarly` | eases back to rest over 0.3 s | |

The motion is a damped spring toward each target (`spring_hz` 9, damping
0.7). A phase change never pops the rod: the spring carries it from wherever
it is.

## 3. The line follows the tip

`held::rod_tip()` stays the rest tip. A new resource, `RodTipNow`, holds the
tip as drawn this frame, in eye space: the rest tip through the frame's pose.
`fish.rs` reads it where it read `rod_tip()`, both for the line mesh and for
the `Angler::tip` a cast leaves from. The cast still leaves from where the tip
is when the button comes up, so the rule sees no change bigger than the 0.1 m
the tip moves at the end of the charge.

## 4. Tunables

`HeldSettings` gains a `rod` block holding every value in section 2's table,
with the units in the field docs. Each value is validated finite, and
non-negative where it is a size or a rate. Defaults live in one place (the
`Default` impl), and `held.ron` states them.

As built (`RodSettings` in `held.rs`), every value is a size, a rate or a
time and the code gives it its sign, so all are validated finite and
non-negative, and `spring_hz` positive. The table's fixed numbers became
fields beside its named ones, named plainly and as the Rod in Hand mockup
prints them, so its `rod: (...)` block pastes into `held.ron` unchanged.
Each phase's pitch is measured from the rest pose (not from the pose before
it) and written as a size: the fishing pose is at `-settle_rad`, so
`settle_rad: 0.12`, never `-0.12`.

| Field | Default | Unit | What |
| --- | ---: | --- | --- |
| `draw_rad` | 0.9 | rad | pitch back at full charge |
| `draw_lift_m` | 0.05 | m | fist up and back at full charge |
| `flick_rad` | 0.35 | rad | pitch forward on the cast |
| `flick_s` | 0.14 | s | how long the flick is held |
| `settle_rad` | 0.12 | rad | pitch forward while the float is out |
| `bob_rad` | 0.015 | rad | the slow bob's size |
| `bob_hz` | 0.4 | Hz | the slow bob's rate |
| `nibble_rad` | 0.05 | rad | the twitch down on a nibble |
| `nibble_s` | 0.15 | s | how long the twitch lasts |
| `bite_rad` | 0.22 | rad | the dip forward on a bite |
| `bite_shake_rad` | 0.04 | rad | the bite's shake |
| `bite_shake_hz` | 14 | Hz | the bite's shake rate |
| `fight_rad` | 0.45 | rad | pitch up with a fish on |
| `fight_shake_rad` | 0.06 | rad | the fight's shake at full tension |
| `fight_shake_hz` | 9 | Hz | the fight's shake rate |
| `crank_m` | 0.012 | m | the fist's circle while reeling |
| `crank_hz` | 3 | Hz | the circle's rate |
| `roll_rad` | 0.15 | rad | roll toward the fish's pull, at most |
| `catch_rad` | 0.6 | rad | the lift on a catch |
| `catch_s` | 0.6 | s | the catch's ease back to rest |
| `ease_s` | 0.3 | s | the ease back after a snap, miss, strike too early or a line wound in |
| `spring_hz` | 9 | Hz | the spring's natural frequency |
| `damping` | 0.7 | ratio | the spring's damping (1 critical) |

The spring is stepped implicitly, so a long frame does not throw it. The
shakes, the bob and the crank are part of the spring's target, so they too
are carried and never pop. The line's events reach the rod through a
`RodCue` resource that `fish.rs` writes each frame (phase, charge, tension,
whether the reel is wound, which side the fish pulls, and the frame's
events), and the rod is posed after the fishing systems in `Update`. The
cast's `Angler::tip` reads `RodTipNow` as last drawn, a frame old; the line
mesh in `PostUpdate` reads this frame's.

## 5. The mockup

**Rod in Hand** (M36, `docs/mockups/rod-motion.html`):

- **Models.** The same first-person rod and hand as the game, from the
  generator's data (`tools/gen_held_tools.py`, the same JSON the Hex Tool
  Bench embeds). The hand uses the fixed normals.
- **Scene.**
  - The 3D view looks from the eye over a bank onto water with a float.
  - The view stays where the viewer leaves it, and nothing turns on its own.
  - At night, a lantern on the bank lights the water and the hand, the moon
    is dim, and the float carries a small glow.
- **Tabs, one 3D view at a time:**
  - **Proposed** is the motion of section 2.
  - **Today** is the still rod the game draws now.
- **Controls.**
  - Play-through buttons for each phase (Charge, Cast, Nibble, Bite, Fight,
    Catch), plus "Play all".
  - Hold-to-charge and release-to-cast with the mouse, as in the game.
  - A day/night toggle.
- **Sliders** for the section 2 values, so the owner can tune them. The page
  prints the `held.ron` block for the values as set.
- **Info.** Notes are in an info modal, with the tracker link at the top.

## 6. Decisions

- The rod's own bend is not modelled. The rod is a rigid hex model, and
  bending it means a vertex bend along the handle. The pitch and shake carry
  the fight first; a bend can come later if the owner asks.
  Recommendation taken (ask only with screenshots).
- The game moves in the same push as the mockup, since the owner asked to
  "fix that up". If the owner tunes the mockup, its printed `held.ron` block
  goes into the game unchanged.
