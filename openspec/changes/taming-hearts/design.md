# Design

## 1. How many hearts

`pbd_core::husbandry::taming::hearts(trust) -> u8`: `ceil(trust / 20)`,
clamped to 1..=5 (a feed always shows at least one). A feed that tames, and
a feed of a kept animal, show 5. One rule in core, tested.

## 2. The hearts on screen

The hearts are a UI emote, not a thing in the world: a row of small pixel
hearts (`assets/ui/heart.png`, a 1 px outlined icon like the pack's) drawn as
UI nodes and placed each frame where the animal's head projects on the screen
(`Camera::world_to_viewport`). They rise 0.6 m over 1.5 s and fade over the
last half second. Hidden when the head is behind the camera. Being UI, they
are not lit by the terrain and keep the icon outline, as inventory icons do
(CLAUDE.md, "No outlines in the world" and "Everything in the world is lit as
the terrain is" cover meshes in the world).

The farmyard queues `(beast, hearts)` on a feed; a desktop system spawns and
moves the rows.

## 3. Following after a feed

A wild animal fed but not yet tamed goes to `Mode::Follow` with
`mode_s = taming.wild_follow_s` (new, 20 s, validated > 0). While it follows,
the wild block's fear and approach do not change its mode (it is not
spooked by a walking player, as a following tamed one is not). When the time
is up it settles as today (`settle` makes a wild animal idle). Trust still
decays per hour as before.
