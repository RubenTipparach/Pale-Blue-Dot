# Playtest fixes: design

None of these came with a choice to look at, so each is a recommendation
taken (ask only with screenshots). Screenshots go to the owner with the
reply.

## 1. Townsfolk stop to talk

**Today.** A person's place is a pure function of the town clock
(`Person::at(clock)`). Talking (`Townsfolk::set_talking`) only switches the
face, so a walker keeps walking their route mid-sentence.

**Change.** Each live person keeps their own clock offset.
- While talking, they hold the place where the talk began. They turn to the
  player at the turn rate they already use, and stand rather than walk.
- When the talk ends, the seconds spent talking are added to their offset,
  and they go on from `Person::at(clock - offset)`. So they pick up where
  they stopped, with no jump along the route.
- A person driven by a caller (a lost villager) or resting in a seat or bed
  is left as they are.
- The offset is in memory only. A world reopened has everyone back on
  their schedule, which is what it does today.

## 2. A job as the active mission

**Today.**
- `TakenJobs` holds the taken jobs as a chain with its own `Progress`,
  focus included.
- The tracker reads only the starter and town trees.
- The Missions screen has two tabs, STARTER and TOWN.

**Change.**
- **A JOBS tab** beside them lists every taken job, one row each: its name
  (the board's own line for it), its town, its objectives with counts, and
  its reward.
- **TRACK.** A TRACK plaque on each row makes that job the active
  mission, saved in `TakenJobs`'s progress focus. Tracking a mission on
  either tree clears it.
- **The tracker** shows the tracked job first, its title the job's line
  and its rows its objectives.
- **A job handed in or dropped** clears the focus, and the tracker falls
  back to the trees.

## 3. Wild plants grow back

**Today.**
- A picked wild crop (`wild` record) is gone for good.
- A glowing mushroom (`glow-plant` record) grows back after 8 game days, and
  only where its cell is next built more than the clutter's reach from the
  camera.
- A game day is 2,880 s (48 minutes), so 8 days is six and a half hours of
  play.

**Change.**
- Both grow back after `WILD_REGROW_DAYS` = 2 game days, about an hour and a
  half of play.
- The glowing mushroom keeps its out-of-sight rule: a due pick stands
  picked while its cell is near the camera, and is back the next time the
  cell is built farther away.
- A wild crop's bush is a separate entity that fades in when it appears, as
  a new plant does. So a due wild crop simply comes back and dissolves in,
  never popping (*built differently from the plan*).
- Laying out a building site still counts a picked wild plant as gone (it
  has no clock to hand); a grown-back one blocks a site once the farm draws
  it again.
- The `wild` record keeps the time it was picked (`Picked::picked_s`, which
  it already has). No save format changes.

## 4. The placed torch is the held torch

**Today.** A placed torch is drawn in the terrain shader's column pass
(`planet_surface.wgsl`, the lamp section): a four-sided post 0.55 m tall and
a two-triangle cap at full lamp.

**Change.**
- Placed torches within the clutter's reach are drawn as entities built from
  `held/light.rs`'s meshes: the six-sided bark haft, the wound rag head and
  the pixel flame cards on their frames.
  - They are lit by the field (`LitByField`, `LitLikeTerrain`).
  - The flame flickers on the same `flicker` as the held one, seeded by the
    cell, so no two burn in step.
  - The model is scaled so its foot is on the floor and its head is where
    the shader's head was. The light the torch casts does not move.
- The shader no longer draws a torch's post and cap. Every other lamp keeps
  its shader body.
- A torch past the reach keeps the light it casts, as now; at that distance
  its body was under a pixel.
