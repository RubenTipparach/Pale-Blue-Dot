## The ceiling

For a river cell `k` of the drainage map (its reach runs from its centre to
its downstream cell's), sample `S` = 8 points along the reach's line,
meander included, at `t = (i + 0.5) / S`. Each sample's clamp is
`top_here`'s: a layer under the lowest unrivered ground of the ring of
radius half width + bank round it.

- `in(k)` = the least of its own reach level and every feeder's last ceiling;
- `ceil(k, i)` = the least of `in(k)` and the clamps of samples `0..=i`;
- a point on reach `k` at `t` has the water top `ceil(k, floor(t * S))`, and
  nothing lower or higher from its own ring.

Feeders are done before the cells they feed: the priority flood's order,
reversed. A confluence takes the lower of its feeders, so neither stream
rises into it.

The water top is then non-increasing along every river's line, by
construction: within a reach the samples are a running minimum; across a
reach boundary `in(down)` is at most the upstream reach's last ceiling.
Across the river at any point it is one value, so the water stands flat.

## What it costs

Made with the map: 5,025 river cells x 8 samples x 6 ground samples, on
every core. Not in the cache file: remade when the map is read (the samples
are independent; about 50 ms on eight threads).

## Proof

- `version_11_rivers_never_rise_downstream`: down every river's line, every
  2 m, the water's top never rises.
- `version_11_is_version_10_off_the_rivers`: ground altitude identical
  wherever no river is near.
- Versions 8 to 10 unchanged: their ground digests the same as before.
- The owner's river: the instrument reads no rises.

## Built (2026-11-18)

- **The table** is as above, plus one rule found by the test: each reach's
  last point takes the first point of the reach it runs into. Near a
  junction a point can be nearer the end of the reach that joins than the
  reach it is on. A stream higher than the river it joins (or a reach whose
  next starts under a dipped bank) falls in its last stretch, so no reading
  there stands over the river's. Before it: 4 rises in 40 rivers; after: none.
- **The owner, mid-build:** "think of rivers as a line with series of nodes
  a-> B ->c ->d ... is current node higher than any other previous node? ...
  the node path ends at a ocean/beach". That is the rule: eight points to a
  reach, each no higher than any before it, to the sea's cell.
- **Measured.**
  - The owner's river: 787 points with water, **0 rises** (version 10: 49
    rises, 54 m, worst 6 m).
  - Forty rivers spread over the body: no rise in 13,416 water points; they
    keep 98% of version 10's water points (13,683). Below a dipped bank a
    river runs lower, and it is never dried out.
- **Tests:** `version_11_rivers_never_rise_downstream` (no rise, and at least
  90% of version 10's water kept), `version_11_is_version_10_off_the_rivers`.
