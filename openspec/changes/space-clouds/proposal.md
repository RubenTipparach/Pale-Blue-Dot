# Proposal: clouds from space that are solid and of many sizes (space-clouds)

## Why

The owner, 2026-11-14, with a screenshot from space at dusk, the limb and the
stars in view: "clouds are also a bit too transparent from space" and "their
size is kinda uniform". The clouds read as soft, half-transparent blobs of
nearly one size, spaced like a grid, with the sea and the land showing
clearly through them. Wanted: opaque where they are thick, with a natural
spread of sizes, from small fair-weather cumulus to large clumped masses.

This sits under priority 4 in `CLAUDE.md` (clouds seen from space) and must
not undo `cloud-puffs`, `puff-haze`, `cloud-pop` or `horizon-haze`.

## What was found

Reproduced on this branch's base (`claude/perf-tuning` 3b44fb5) with fixed-step
stills from 9 km and 6 km up (`docs/screenshots/space-clouds/`). Three causes,
each measured in the design:

1. **A median cloud is drawn a third opaque.** A puff's opacity is
   `1 - exp(-od / cloud_puff_optical_depth)` with the scale at 15, and the
   shown maps' median column optical depth is 7 (`cloud-puffs` design 2): a
   median puff is 0.37 opaque (0.35 with `cloud_puff_opacity`). A column of
   optical depth 7 lets through e^-7 of the light: it is solid.
2. **From above, the air in front turns the cloud transparent to the sea.**
   Each puff is faded by the air between it and the eye (`puff-haze`,
   `cloud_air`), and a faded cloud shows what is behind it. That is right
   where the sky is behind a cloud: the sky is the colour of the air
   (`cloud-close-up`, `horizon-haze`). Looking down from space the thing
   behind the cloud is the sea 300 to 750 m below it, and the air in front
   is the same air the sea is seen through. Straight down from 9 km a cloud
   keeps 0.49 of its coverage (0.40 from 15 km, by the five-point rule), so
   a median puff ends at about 0.17 coverage over the sea.
3. **Every puff is nearly one size, on a near-regular lattice.** A puff's
   radius is `spacing (0.5 + 0.35 cover)(0.8 + 0.4 size)`: under one cover
   the biggest is 1.5 times the smallest. The sites are a Fibonacci lattice
   nudged by a third of the spacing, and which sites carry a puff is an
   independent hash per site, so at any cover the carried ones are scattered
   evenly over the lattice: the grid of puffs.

## What changes

- **Thick cloud is solid.** `cloud_puff_optical_depth` goes from 15 to 4: a
  median column (od 7) is 0.83 opaque, a thin one (od 1) 0.22, a storm whole.
- **A cloud over a surface fades only by the air the surface does not
  share.** Where the ray stops on the ground or the sea, a puff's haze is
  scaled by how much of the air beyond it, between it and that surface,
  would hide the surface: `T' = 1 - (1 - T_behind)(1 - T)`. Where the ray
  goes on into the sky, nothing changes: the cloud still fades into the sky,
  as `horizon-haze` decided. Straight down from 9 km a cloud keeps 0.82 of
  its coverage instead of 0.49; along the limb, where the air behind is
  long, it fades as today.
- **Sizes spread and puffs gather.** The sites gain a smooth "clump" field
  over the sphere (value noise, about a kilometre across). Which sites carry
  cloud first, and how big they are, is ranked from a mix of the site's own
  hash and the clump field, so the thresholds stay uniform (a cover of 0.3
  still carries 30 % of the sites) but the carried ones gather into groups,
  and the big ones sit in the groups. A puff's radius factor becomes
  `0.5 + 1.3 u^2` (u uniform): 0.5 to 1.8, many small and a few big, with
  the same mean area as today's `0.8 + 0.4 u`. The sites' nudge goes from a
  third of the spacing to a half, so the lattice does not show.
- The pre-existing failing test `puffs_follow_the_map_and_come_far_to_near`
  counts the puffs built, not those left over the horizon from an eye at
  the cloud layer (the cull, `cloud-pop`'s neighbour, came after the test):
  fixed here, since this change is where the puff counts are argued.

The puffs' placement is derived from the body and the weather, never saved:
no save changes, and terrain generation is untouched.

## Not in this change

- The cloud's lighting at dusk and night, and stars seen through cloud:
  `dusk-clouds`, on the same branch.
- The march renderer (`PBD_CLOUD_RENDERER=march`), which is not the shipped
  one.
- Frame time: not measured (no perf runs on this Mac while other agents
  capture). The puff count moves a little (the log's `PUFFS built` line, in
  the README), and the puff fragment gains a second five-point air integral
  where the ray ends on a surface.
- Recommendation taken (ask only with screenshots): the values above. The
  before and after stills go to the owner.
