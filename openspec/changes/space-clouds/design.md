# Design: space-clouds

## 1. The opacity, from the code

`pbd_core::cloud_puffs::puffs_at`: `opacity = a * thick * cloud_puff_opacity
* present`, `thick = 1 - exp(-od / cloud_puff_optical_depth)`. With the
shown maps' optical depths (`cloud-puffs` design 2: median 7, 75th
percentile 19, 90th 52) and `cloud_puff_opacity` 0.95:

| od | scale 15 (today) | scale 4 |
| ---: | ---: | ---: |
| 1 | 0.06 | 0.21 |
| 3 | 0.17 | 0.50 |
| 7 (median) | 0.35 | 0.78 |
| 19 (75th) | 0.68 | 0.94 |
| 52 (90th) | 0.92 | 0.95 |

The shader then lays its own round profile over that (`dens`, solid inside
0.45 of the radius, eaten by noise toward the edge), so a puff's middle is
`opacity` and its edge soft. Scale 15 has no recorded reason (it came in with
`lighting-model`, `588ded9`). The test layer's 6 and the test's
`thin_cloud_is_drawn_thin` (od 1 under 0.2, od 20 over 0.9) stand.

## 2. The haze from above, measured

`cloud_air`: the optical depth of an exponential air (0.00036 per metre at
the sea, 1050 m scale height) from the eye to the puff by Simpson's rule on
five points, times `cloud_haze` (3). A cloud at 500 m, the eye over it
(python, the shader's rule against a 4,000-step sum):

| Eye | Pitch | T (5-point) | T (exact) | T behind (cloud to sea) | T' |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 5 km | -90 | 0.50 | 0.50 | 0.65 | 0.82 |
| 5 km | -63 (near the limb) | 0.35 | 0.35 | 0.37 | 0.59 |
| 15 km | -90 | 0.40 | 0.49 | 0.65 | 0.79 |
| 15 km | -78 | 0.34 | 0.38 | 0.47 | 0.65 |

The cloud keeps `T` of its coverage today, and the sea shows through the
rest: from orbit the air, nearly all of it in the last kilometre over the
cloud, halves every cloud. The ground and the sea seen from up there are
hazed only by the eye's own altitude (`horizon-haze` table: "the eye's
altitude only"), which from space is none, so the cloud is hazed into a sea
that is not hazed itself.

### The rule

The composite lays a puff as `L = S (1 - aT) + C a T`: the cloud fades into
`S`, what is behind it, by the air. That stands for the air's own light
where `S` is lit air, the sky. Where `S` is a surface it is not: the air's
light over the cloud is the light of the air between the eye and the cloud,
and the surface shows only as much of that air as it is itself seen through.
So the share of the air that `S` can stand for is the share of the surface's
own light the air behind the cloud would take, `k = 1 - T_behind`, measured
from the cloud to where the ray stops:

`T' = 1 - k (1 - T)`, `k = 1 - T_behind` on a surface, `k = 1` in the sky.

- A cloud with the sky behind it is as today (`horizon-haze`, `cloud-edges`).
- A cloud just over the sea with little air between them: `k` small, the
  cloud keeps its coverage (the table's last column).
- Along the limb the ray's path behind the cloud is long and grazing,
  `T_behind` falls toward 0, `k` toward 1: the limb's clouds fade as today,
  which is what priority 4 asked of them.
- From the ground under the cloud, the ray goes into the sky: unchanged.

`T_behind` is the same `cloud_air`, from the puff's point to the ray's stop
(`far`, which the fragment already has). It runs only where `far` is short of
the far cap, and only for fragments that pass the discard.

## 3. Sizes and clumps

`sites(radius, spacing)` today: direction from the Fibonacci lattice nudged
by up to a third of the spacing; `threshold`, `size`, `seed` independent
hashes of the index.

New, still a pure function of the body and the spacing:

- **A clump field** `clump(d)`: value noise of the direction, two octaves
  (cells of 1,100 m and 550 m on the 5,100 m layer, weights 0.65 and 0.35),
  smoothstepped trilinear on a hashed integer lattice.
- **Thresholds ranked**: each site's key is `0.45 hash0 + 0.55 (1 - clump)`;
  the sites are sorted by key (index breaks ties) and site of rank `k` gets
  `threshold = (k + 0.5) / n`. The thresholds stay exactly uniform, so the
  share of sites carrying cloud is the cover as today
  (`more_cover_carries_more_puffs`, `a_full_low_deck_keeps_lanes...`), but
  under a partial cover the carried sites gather where the clump field is
  high.
- **Sizes ranked**: key `0.5 hash1 + 0.5 clump`, ranked the same way to `u`
  in 0..1, so the big puffs stand in the clumps. The radius factor is
  `0.5 + 1.3 u^2`: 0.5 to 1.8, median 0.83. Its mean square is 1.02 against
  today's 1.01 for `0.8 + 0.4 u`, so the area a full cover lays is the same
  and the deck at full cover is no more of a sheet than it is.
- **The nudge** goes to half the spacing.

What the radius does to a tower (`cloud-pop`): `each = max(1.4 r,
height / 3)` takes the site's radius as before; a small puff under a tall
top now stacks to three sooner, which is the "small towering cumulus" look,
and the stack still grows continuously (`a_tower_grows_and_shrinks_without_re_forming`
holds: it fixes the site).

Measured by tests (section 5): the ratio of the 90th to the 10th percentile
radius under one cover goes from 1.38 to above 2.5 (3.0 measured); with a cover of 0.3,
the share of a carried site's nearest neighbours that are carried goes from
about the cover (0.3) to well above it.

## 4. What is not changed

- `cloud_puff_opacity`, the puff's round profile and erosion, the shading.
- The composite, the history, the distances and the silhouette tests.
- The horizon cull and the frustum cull.
- The deck's lanes (`cloud_deck_floor`), the fade with cover, the stack.

## 5. Tests

- `cloud_puffs::tests::puffs_come_in_many_sizes`: p90/p10 radius > 2.5.
- `cloud_puffs::tests::puffs_gather_in_clumps`: at cover 0.3, carried sites'
  carried-neighbour share > 0.45.
- `cloud_puffs::tests::more_cover_carries_more_puffs` and the others as they
  are.
- `planet_cloud_puffs::tests::puffs_follow_the_map_and_come_far_to_near`:
  the share counted from `PuffCounts::built`.
- `config` test: the shipped settings draw a median column (od 7) at least
  0.75 opaque.
- `shader_tests`: the puff fragment measures the air behind it to the ray's
  stop and scales its haze by it only short of the far cap.
