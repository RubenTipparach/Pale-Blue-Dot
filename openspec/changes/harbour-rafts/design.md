## Context

A harbour template (`assets/settlements/v1/coast.json`) moors 23 boats:
14 `rowboat`s, 6 `sail`s and 3 `canoe`s, at `(x, z)` with a heading, and a
`cog`. After `retire-rowboat-and-cog` only the canoes are made (Loons);
`harbour::boats_for` tags each craft with its berth `(site, n)`, `n` its
index in the template, and the fleet's "made" set (live, stowed and retired
records' berths) stops a berth being filled twice.

The raft (`Kind::Raft`, `small-boats` design 2) is 4.4 m long and 2.4 m
across, decked (it does not swamp), floats with its logs half under, and
has a hold. Its half-diagonal is 2.51 m.

## Decisions

### 1. Which berths get a raft

The template's `rowboat` and `sail` berths, in template order, each taking
a raft unless its middle is within `RAFT_APART_M` = 5.1 m of a raft already
given a berth before it. Two rafts whose middles are 5.1 m apart cannot
touch, at any heading (twice the half-diagonal is 5.02 m). The template
packs its rowboats 3.6 to 4.5 m apart, closer than a raft allows, so the
rule picks 11 of the 20 berths in the shipped template (0, 3, 6, 7, 8, 10,
12, 13, 14, 21, 22). The depth check is the canoes' (`DRAWS_M`, 0.5 m of the
drawn sea), so a berth in water too shallow is skipped and counted as now.

- *Recommendation taken (ask only with screenshots).* The alternative, a
  raft at every berth, overlaps neighbours; one raft per harbour leaves the
  quays empty.

### 2. A raft's berth number

A raft at template berth `n` is tagged `(site, RAFT_BERTH + n)`, with
`RAFT_BERTH` = 65 536, past any template index and short of the cog's
`u32::MAX`. So:
- in a new world each raft berth is made once, as a canoe's is;
- in an old world a template berth held by a retired rowboat's or Tern's
  record (`(site, n)`) does not stop the raft, whose key differs, and the
  held record stays as it was. The retired boat is not drawn or stepped, so
  the raft at its place overlaps nothing. This is the coordinator's
  recommendation ("a held-retired berth gets a raft moored next to it only if
  that doesn't overlap"): moored at the berth itself, since nothing is there.

### 3. Made like a canoe

Anchored to the seabed under it with the anchor's rode, put on the tide
(`place::on_the_tide`), `made: false`. The same stow and wake as every
harbour craft. Boarding, poling and the hold are the raft's own.

### 4. The rowing launch, later

M23's "Rowing launch" (Drydock, hex-cell hull) is the new rowboat the owner
means. When it is built and approved for the game, `kind_of` gives it the
template's `rowboat` berths and the rafts move to the `sail` berths; that
is its change's to decide. Nothing here builds it.

## Risks / Trade-offs

- **More craft near a harbour**: up to eleven rafts, each a rigid body.
  They sleep at rest beyond 30 m of the camera and are stowed past 1.5 km.
- **Rafts at sea berths**: a raft in a swell rides on its logs; it is
  decked and does not take water, so it does not swamp at its berth as an
  open boat can.
