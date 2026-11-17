# Rod motion and the hand's faces

## Why

The owner, 2026-11-17: "The dudes hand kinda of has weird normals. Also the
fishing rod isnt animated when casting or when a fish is bighting let's fix
that up. Make a new fishing mockup".

- **The hand.** `hexel::prism` shades each side face by the direction from
  the prism's middle to the middle of that side. On a round prism, those two
  directions are the same. The back of the hand is a flattened prism (width
  1.97, height 0.70 tool units). On its four slanted faces the two directions
  are up to about 50° apart: a face that points mostly up is shaded as if it
  pointed sideways. That is the patchy shading on the back of the hand. The
  approved Hex Tool Bench mockup (M17) shades by the true face normal, so the
  game does not match what was approved.
- **The rod.** `held::hold` pitches every tool in a chop while a block breaks,
  and zeroes the chop for the rod. Nothing else moves the rod. It stands still
  through the charge, the cast, a nibble, a bite and the fight. The line
  always leaves the rod tip's rest position (`held::rod_tip`), so even a moved
  rod would leave the line behind.

## What changes

- The side faces of a prism are shaded by their true normal: the side edge
  crossed with the axis, pointing outward. Only flattened prisms change, and
  the back of the hand is the only one.
- The rod in hand moves with the line's phase:
  - it draws back while a cast charges;
  - it flicks forward on the cast;
  - it settles into a fishing pose while the float flies and floats;
  - the tip twitches on a nibble and dips hard on a bite;
  - it is held up and shakes with the tension while a fish is on;
  - it lifts on a catch.
- The line leaves the rod tip where the rod is drawn this frame.
- The motion's numbers are tunables in `held.ron`.
- A new mockup, **Rod in Hand** (M36, `docs/mockups/rod-motion.html`). It
  plays every phase on the game's own hand and rod, beside today's still rod,
  at day and at night by a lantern.

## Impact

- `crates/pbd-core/src/hexel.rs` (`prism`).
- `crates/pbd-app/src/held.rs`, `crates/pbd-app/src/fish.rs` (where the line
  leaves from).
- `assets/config/held.ron`, `config.rs`.
- A new mockup and tracker row.
- No save, generation or rule changes: the catch rules in
  `pbd_core::fishing` are untouched, and the motion is drawn only.
