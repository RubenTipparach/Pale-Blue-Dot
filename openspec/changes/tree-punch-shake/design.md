# Design

## Measured

- `gather::tree_gives(taken, by_axe)`: an axe takes logs while `taken <
  axe_logs` (4, `gather.ron`); hands take one (`taken == 0`) and are then
  refused (`TreeSpent::BranchTaken`, silent since the plain-text rule).
- A log's break time: `dig.ron` `wood.axe` (0.6 s) over the tier's speed for
  an axe; over `hand_speed` (0.25) by hand, 2.4 s.
- Trees are drawn by `planet_surface.wgsl` (the near model from the kit,
  `tree_sway` for the wind). Nothing moves a tree when it is hit.

## Decisions

1. Hands take logs like an axe, to `axe_logs`; each takes the hand time,
   2.4 s, four times a stone axe's. The last log fells the tree for either.
2. The fell mission (`Kind::Fell`) counts a tree felled by an axe or by hand.
3. The shake: the tree being worked (its cell id) and the time of its last
   hit go to the planet uniform. The near tree's vertex adds a damped sway on
   top of the wind: amplitude growing with height over the base (as
   `tree_sway` does), about 0.12 m at the crown, 9 Hz, falling off over about
   0.6 s; leaves flutter a little more. The mid and far trees do not shake.
4. A hit is each log taken and, while the button is held on a tree, every
   0.5 s, so a long punch keeps the tree moving.
