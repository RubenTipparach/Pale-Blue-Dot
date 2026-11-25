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

## On the stroke's impact, and twice the size (owner, 2026-11-21)

The owner, on the Tools in Hand mockup's Punch tab (version 16): "punch
animation looks good, just make sure it syncs with the tree shake, maybe add
aplitude tree shake 2x so its a bit more obvious".

- **In step with the stroke** (replaces decision 4): the shake fires on each
  impact of the stroke in hand (`held-swing` decision 10, `StrokeImpact`),
  the hands' and the axe's alike, one shake per impact, while the stroke is
  on a tree. The 0.5 s timer (`HOLD_HIT_S`) and the separate shake on each
  log are gone: the log comes on the stroke's last impact, which is the
  break, so it still shakes once. A log by hand is six punches of 0.425 s,
  a stone axe's two strokes of 0.375 s.
- **Twice the size** (decision 3): `SHAKE_M` 0.12 m to 0.24 m at the crown,
  in `tree_shake.rs` and `TREE_SHAKE_M` in `planet_surface.wgsl` (the test
  holds the two together). The rate, the settle and the leaves' share are
  unchanged. The mockup's shake (`tool-motion.html`, `SHAKE.m`) is doubled
  to match.
- Still drawn only: the shake reads the stroke's impacts, which read the
  dig's beat; no rule reads either.

## A punched log takes twice as long (owner, 2026-11-22)

The owner: "Punching is fine for now. I would double the time to punch to get
a log." A tree gives fists and an axe the same four logs (decision 1). A log
took the stone axe's wood time over the hands' speed: 0.6 s / 0.25 = 2.4 s
by hand, against 0.6 s with a stone axe.

`hand_speed` also sets how fast bare hands dig soft ground, so it is not
the lever. A field of its own, `dig.hand_log_speed`, 0.125, sets the hands'
speed on a trunk only: a punched log takes 4.8 s, a tree's four about 19 s.
That is 8 times the stone axe. Bare hands on dirt are unchanged. A shovel or
pickaxe in hand works a trunk at the hands' log speed too, as before (only
the axe is a wood tool).

## Dust and falling leaves (owner, 2026-11-22)

The owner: "We should probably add some dust fx + leaves falling fx when a
tree despawns or gets hit."

Planned, not built:
- **On a hit** (the stroke's impact, the same moment the shake starts: a
  punch's or an axe's):
  - a small puff of bark dust at the trunk where the ray met it;
  - a few leaves let go from the canopy and drift down, swaying, to the
    ground under the crown;
  - a few for a punch, more for an axe.
- **On the fell** (the last log taken, the tree despawning): a bigger burst
  of dust at the stump, and a shower of leaves from where the crown was.
  The tree itself still goes as it does now. A falling tree is its own,
  later piece of work (`ocean-environments` B29 notes it as not built).
- **The look, by the repository's rules:**
  - pixel particles in the biome's own palette, the leaves taking the tree's
    leaf colour and the dust its bark's and the ground's;
  - no outline;
  - lit as the terrain is (`LitByField`, `LitLikeTerrain`), so a hit in the
    shade or at night is dark;
  - nothing glows.
- **The motion:** dust puffs out and settles in about half a second. Leaves
  fall slowly, swaying side to side, over about two seconds, and fade once on
  the ground.
- **Cost:** a CPU particle list, with a cap on live particles (tens, not
  hundreds) and no new render pass. Measured with `--frame-log` on a chop
  before and after.
- **Mockup first.** The motion is new, so per the `animation` skill it is
  shown in a mockup beside today's hit (no particles) for the owner's
  approval before game code. The mockup goes on the tracker. Tunables
  (counts, sizes, times, sway) are in a RON file with one source of
  defaults.

### Built without the mockup (owner, 2026-11-22)

The owner asked for it in the game the same night ("please get all fixes
that I asked TONIGHT"), so the mockup step above was skipped at the owner's
ask and the effect went straight into the game. If the look is wrong in
play, the mockup comes back as the place to fix it.

As built (`crates/pbd-app/src/tree_fx.rs`, tunables in
`assets/config/tree_fx.ron`, `TreeFxSettings::default()` the one source):
- `digging::shake_on_impact` pushes a `TreeFxEvent::Hit { cell, by_axe }`
  onto `TreeFxQueue` on each impact that shakes the tree; `take_log` pushes
  `TreeFxEvent::Felled { cell }` with the last log. Drawn only: nothing
  reads the queue but the effect.
- The counts: a punch 4 specks and 2 leaves, an axe 7 and 4, the fell 18
  and 22; at most 64 alive over every burst.
- The colours: the species' leaf and bark means from the tree kit (the
  numbers the near tree is pulled to), the leaves at 0.8 of the mean, the
  dust between the bark and a dry earth (0.32, 0.25, 0.17 linear). Autumn
  and winter leaves are not followed yet: a falling leaf is the species'
  summer mean.
- Square opaque cards facing the eye with an up normal, one mesh per tree
  rebuilt each frame, on a white field-lit material with `LitByField` and
  `LitLikeTerrain`; no shadow cast. Leaves land on the generator's surface
  under them (a dug or built cell under a crown is not seen).
- Not measured yet: the frame cost with `--frame-log` on a chop.
