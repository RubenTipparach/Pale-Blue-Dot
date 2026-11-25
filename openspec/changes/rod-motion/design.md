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
| Charging | rises to `+draw_rad` (0.9) with the charge: the windup, then a held coil (decision 7) | the fist raised over the shoulder on an arc, `cast_raise_m` (0.28) up and `cast_back_m` (0.18) back |
| `Cast` | from the drawn pitch to `-flick_rad` (-0.35) in `flick_s` (0.14 s), then springs back to the fishing pose | the arm drives forward and down the arc to `cast_reach_m` (0.07) past rest |
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
| `cast_raise_m` | 0.28 | m | fist up from rest at full charge (decision 7) |
| `cast_back_m` | 0.18 | m | fist back toward the head at full charge |
| `cast_reach_m` | 0.07 | m | the arm's stretch and swing down past rest at the flick |
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

**Tools in Hand** (M36, `docs/mockups/tool-motion.html`, https://claude.ai/artifact/CwwZtbNGjZSVtjivQAMuzL), its Fish tab. The owner, 2026-11-17: "Actually make a new tools mockup ... for all the tools stuff for digging, choppin and fishing". One page shows the three tools' motion: Dig, Chop (the `held-swing` STRIKE stroke) and Fish (this change), each with a Today / Proposed toggle in its one view.

- **Models.** The same first-person rod and hand as the game, from the
  generator's data (`tools/gen_held_tools.py`, the same JSON the Hex Tool
  Bench embeds). The hand uses the fixed normals.
- **Scene.**
  - The 3D view looks from the eye over a bank onto water with a float.
  - The view stays where the viewer leaves it, and nothing turns on its own.
  - At night, a lantern on the bank lights the water and the hand, the moon
    is dim, and the float carries a small glow.
- **Today / Proposed** in the Fish tab's one view: Proposed is the motion of section 2, and Today is the still rod the game draws now.
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
  "fix that up" before asking for the mockup. If the owner tunes the mockup, its printed `held.ron` block
  goes into the game unchanged.
- The controls stay as they are: hold the left button to reel a hooked fish,
  and right click winds in an empty line. The owner, 2026-11-17, on the Tools
  in Hand mockup's Fish tab: "are you deciding to do left click to real in
  fish insetad now?", "that seems better imo". This closes the open question
  of whether right click should hook or reel.
- **7. The cast is overhead: the hand rises above the shoulder.** The
  owner, 2026-11-17, on the Tools in Hand mockup: "make some videos of the
  animations. they look freaking awesome, though I think the casting
  fishing line the hand should be raised above shoulder". Before this, the
  charge only pitched the rod back about a fist that rose 5 cm
  (`draw_lift_m`), so the cast was a wrist flick at chest height. Now it is
  the four stages of the `animation` skill, the fist on an arc about the
  shoulder (the `held-swing` stroke's virtual shoulder, eye space
  (0.2, -0.25, 0.05) m; the arc is in its upright plane, its radius and
  angle eased together from the rest fist to the target):
  - **windup** (the first 40 % of the charge, eased out): the fist rises
    `cast_raise_m` (0.28 m) and comes back `cast_back_m` (0.18 m) toward the
    head, to about eye height above the right shoulder, 88 % of the way;
  - **anticipation** (the rest of the charge): a held coil, the last 12 % of
    the arc and the last fifth of the draw, eased in and held at the top;
  - **hit** (on release, over `flick_s`): the arm drives forward and down
    along the arc to chest height, the arm stretched by `cast_reach_m`
    (0.07 m) and swung down as far, fastest at the end, while the rod
    flicks to `-flick_rad`, pointing at the water;
  - **recoil**: the spring's overshoot and the tip's wobble, settling on the
    fishing pose.
  The rod turns with the forearm on the arc; while charging its own pitch is
  set so the rod as drawn stays `draw_rad` (0.9) back from rest, pointing up
  and back. `draw_lift_m` is replaced by `cast_raise_m`, `cast_back_m` and
  `cast_reach_m` in `held.ron`. The sizes were set on the mockup so the fist
  stays in view at the top of the draw (0.25 m back put it half off the
  right edge). Recommendation taken (ask only with screenshots): the owner
  sees it on the mockup and in `docs/screenshots/rod-motion/charging-raised.jpg`.

## 8. The cast's line and the mockup's controls (owner, 2026-11-18)

- "the fish line needs to apeear like it was pulled from behind you and
  arcs into the water" (with a fly-fishing reference): while the float
  flies, the line is drawn along the path the float took, from the rod tip
  back to where it was let go (up and behind the head at the top of the
  draw) and out along its arc; after it lands the line eases into its
  hanging curve over 0.6 s. Drawn only. Mockup first; the game waits.
- "the first few frames of releasing the line the thing gets stuck": a
  mockup bug, the float drawn at the water's edge while it was still over
  the bank. The game has no such rule.
- "left click to hook fish, right click to cancel for fishing. just have
  text on the side be clear": the controls stay as they are in the game
  (left casts, hooks and reels; right winds the line in). The mockup's Fish
  tab lists the steps beside the view, the one happening now lit, and calls
  right click "cancel".
- The rod flexes (owner, 2026-11-18, a GIF of a lure rod cast from the
  surf: the blank loads into a deep curve, whips through bent, springs
  straight and ends pointing out low over the water). In the mockup the
  rod's mesh bends past the fist by `bend · L · s²` toward the side a
  positive pitch turns the tip; `bend` is a damped spring (`bend_hz` 4.5,
  `bend_damping` 0.3) toward −`flex` (0.07) × the rod's pitch rate, capped
  at `bend_max_rad` 0.6, so it loads on the draw, lags through the flick
  and wobbles straight. The line leaves the bent tip. While the float flies
  the rod is held out low (`follow_rad` 0.55 below rest) with the arm
  reaching (`follow_reach` 0.8), then settles when it lands. Mockup first;
  the game waits.
- The right button cancels at the bite too, and only the left button
  hooks; the game's prompt reads "RIGHT CLICK CANCEL" while the float is
  out and adds "LEFT CLICK HOOK" at the bite, and the bite's line reads
  "Bite! Left click" (owner, 2026-11-18: "this shouldnt show to confuse
  players, right click should be clearly shown to cancle, and left click to
  hook fish"; `fishing-and-equipment` design, "Why no fish"). Built in the
  game with the mockup, as the owner asked for the game's prompt directly.

## 9. As built (2026-11-18): design 8 in the game

The owner on the mockup: "cool now implement all these".

- **Settings.** `RodSettings` and `held.ron` `rod` take the mockup's six:
  `flex` 0.07, `bend_max_rad` 0.6, `bend_hz` 4.5, `bend_damping` 0.3,
  `follow_rad` 0.55, `follow_reach` 0.8. The test that `held.ron` is the
  defaults holds them together.
- **The bend.** `RodMotion` carries `bend`: a damped spring toward
  `-flex` times the rod's own pitch rate, clamped to `bend_max_rad`, as the
  mockup's `rodPose`. The rod's own rate is the pose's pitch rate less the
  forearm's turn on the cast's arc: the game's spring carries the drawn
  pitch (the rod's and the arm's together), so the arm's turn rides the
  same spring as a sixth value and is taken off, which is the mockup's
  `spring.p.v`. The mockup steps the bend once a frame (semi-implicit
  Euler); the game steps it the same way in steps of at most 1/240 s, so a
  long frame cannot throw it. `held::bend_rod` moves the rod model's
  vertices past the fist by `bend · L · s²` square to the rod, toward the
  side a positive pitch turns its tip, on the CPU whenever the bend changes
  (the mockup's `bend`). The hand is not bent.
- **The tip.** `RodTipNow` is the bent tip, the mockup's `tipBend` (the
  straight tip moved by `bend · L` along eye x crossed with the rod), so
  the line and the cast leave it.
- **The follow-through.** While the float flies, after the flick, the
  rod's target is `-follow_rad` with the arm at `-follow_reach` down the
  cast's arc; it settles on the fishing pose when the float lands. The
  game's other phases are unchanged (the mockup's fight dip, catch dip and
  flick wobble are page values the game does not read, as before).
- **The cast's line.** `fish::draw::CastTrail` keeps the float's place
  where it left the tip and one every 0.15 m while it flies (planet-local,
  moved with a rebase). `line_points` lays each point between the tip and
  the float along the path tip, release, trail, float (`trail_points`,
  evenly along its length), all of it while flying and easing to the
  hanging curve over 0.6 s once the float is down (`drawLine`), then held
  over the water as before. Drawing only: the line's rule never reads it.
  The game's line has 24 points, the mockup's 40.

Differences from the mockup: the bend's substeps and the 24 points, above.
Frame cost was not measured (a cloud container, no GPU).


## The line's speckle over water (owner, 2026-11-21)

The owner, with a night shot of a cast over the sea: "need to fix these weird
fx around the line". A cloud of pale cyan dots surrounds the whole line, out
to a few tens of pixels either side, only where the line is over water.

**Finding (from the code, not yet captured):** the line is an opaque, unlit
`LineStrip` (`fish.rs`, "Fishing line"), so it is in the scene colour the
water reads behind it (`water.wgsl`, `scene_color`). The water bends that
sample by up to `refraction.y` of the screen and rejects a sample of
geometry in front of the sheet by the scene depth. A one-pixel line rarely
covers its pixel's depth sample (MSAA sample 0, or no depth prepass for line
topology), so the rejection misses it: water pixels round the line pick up
the line's white through their refraction offset, tinted by the water. The
spread matches the refraction offset's reach.

**Plan:** draw the line after the water reads the scene (the transparent
phase, `AlphaMode::Blend` at full alpha, or its own pass after the water), so
the water never refracts it; check whether the hook and float need the same.
Verify with a night capture of a cast over the sea before and after, the
speckle gone and the line itself unchanged. A look fix with no player-visible
change other than the artefact gone, so it needs no survey.
