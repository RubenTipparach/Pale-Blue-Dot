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
- **A JOBS tab** beside them (`ShownTree::Jobs`, a third plaque in the
  `TreeTabs` row, shown with the other two once the town tree is open).
  Shown, it takes the tree's place: the tree layer is despawned and a list
  of `Economy::jobs.taken` stands instead, one row each: the job's item in
  a slot well, its board line (`economy::job_line`, which carries its
  counts), the town that posted it, its reward, and a TRACK plaque. With
  no job taken the list says so in one line. It is rebuilt whenever the
  jobs change.
- **TRACK.** The plaque sets `jobs.progress.focus` to the job's
  `mission_id()` and stores the `town_jobs` record as `Shop::take` does
  (the player's author, the record's kind noted). It clears both trees'
  focus, so the HUD has one. The tracked job's plaque is latched; pressing
  it again stops tracking it. Picking an open card on either tree (the
  `focus_on` calls in `steer`) clears the job's focus and stores it.
- **The tracker** (`desktop/missions.rs`, `show_line`) looks at the job
  focus first. A job's `Mission` has an empty title and empty labels
  (`Job::mission`), so the tracker fills them: the title is the job's name
  without counts (`jobs.track.<kind>`, "Take the parcel to {town}"), or
  "Hand it in at {town}" once it is done; each objective's row reads
  "Done {n}/{count}".
- **A job handed in or abandoned** clears the focus (`TakenJobs::abandon`
  already does; `hand_in` now does too), and the tracker falls back to the
  trees. The focus is in the existing `town_jobs` record, so no save
  changes.
- **Tests.** An app test: a taken job tracked through its TRACK press is
  the tracker's title and row; a press on a tree card clears it. A core
  test: handing a tracked job in clears the focus.

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
(`planet_surface.wgsl`, the lamp section, `LAMP_TORCH`, `t < 30u`): a
four-sided post 0.55 m tall and a two-triangle cap at full lamp, 0.16 m
high. The held torch (`held/light.rs`) is a six-sided bark haft, a wound
rag head and three pixel flame cards on frames of one sheet.

**Change.**
- Placed torches within the clutter's reach (`scatter.ron`'s
  `clutter_radius_m`, 120 m) are drawn as entities built from
  `held/light.rs`'s meshes: `haft()`, `torch_head()` and the flame cards
  (`flame_cards`) on the flame sheet.
  - Which torches: the column tier's lamp spots (`ColumnTier::lamps`, the
    lamps the field lights from) whose cell holds `Material::Torch`, so a
    torch the player placed and a torch a town stands (the tundra's) are
    both found, from the same list the light comes from. Rebuilt when the
    tier is re-lit or the fine set moves, and looked through four times a
    second for the near ones.
  - The haft and head are the held meshes with their normals made smooth
    round the haft (the held ones are flat per face, and the owner does not
    want flat shading in the world) and their baked shading taken out, so
    the field shades them. Their textures are the held torch's own
    (`assets/textures/held_light/haft.png`, `head.png`), nearest sampled.
  - Each torch root carries `LitByField`, `LitLikeTerrain` and `Faded`,
    and fades in and out over 0.6 s as a crop does, so nothing pops.
  - The flame is the held flame's three cards, unlit as a lamp's flame is,
    facing the eye about the torch's up, rebuilt each frame for the torches
    in reach. It flickers on the held torch's `flicker`, seeded by the
    torch's cell, so no two burn in step.
  - The model is scaled by 1.4 so its foot is on the floor and the head is
    where the shader's head was (0.55 to 0.71 m up); the torch is turned
    about its up by its cell so a row of them is not all alike.
- The shader no longer draws a torch's post and cap: its branch goes, and
  the torch's 30 vertices are left degenerate as every unused lamp vertex
  is. Every other lamp keeps its shader body.
- The light the torch casts does not change: it is the field's and the
  shader's lamp terms, which read the column record, not the drawn body.
- A torch past the reach keeps its light, as now; at that distance its
  body was under a pixel.
- Nothing is saved; this is drawn only.

## 5. A sleeper's eyes are shut (owner, 2026-11-25: "do the dudes not have their eyes closed when they sleep?")

**Today.** A person lying in a bed (`Posture::Lie`) keeps the face of a person
standing: it blinks, glances about and keeps its expression's lids.

**Change.**
- The face has an asleep state (`Face::set_asleep`). Asleep, the lids close
  over 0.4 s and stay shut through blinks and glances, and the mouth is
  closed.
- A person lying in a bed is asleep, except while the player talks to them:
  then they open their eyes and talk.

## 6. The blacksmith's station (owner, 2026-11-25: "the black smith doesnt have hist station")

**Finding (from the code).**
- A smithy (its open shed, the anvil and the forge, a brazier in the field)
  is stood only in a town whose stored record names layout 3 or later
  (`pbd_core::settlement::record::FORGE_LAYOUT`). Layout 3 came in on
  2026-11-24 (`f7ec99f5`, "the town forge, for towns laid from now on").
- A town is laid once, when the world first reaches it, and keeps the
  layout it was laid with (saves survive). Every town in a world played
  before 2026-11-24 is at layout 1 or 2, so it has no smithy, by design.
- The smith is a person of the template (`village.json` and `town.json`
  both have one, at work with a hammer beside the smithy's cells). People
  are derived, not stored, so the smith stands in every village and town,
  old or new, at the place the smithy would be. In an old town that place
  is bare ground: the owner's report.
- In a town laid at layout 3 the forge stands (see the captures). So this
  is not a bug in layout 3.

**Recommendation (for the survey, with the two captures).** Old towns
gain their smithy, without touching anything the player built:
- a new layout, 4, that a town at 1 or 2 is moved up to in place, the
  smithy being the only thing layout 3 added; its record is rewritten by
  the world (`Author::World`) through the durable path;
- the smithy's cells are checked first against the player's edits and
  builds (the yield set), and a smithy any of whose cells the player has
  touched is not stood, so nothing the player made is moved or covered;
- the rest of the town (its ground, buildings and terrace) is read by the
  rules it was laid with, so it does not move.

Until the owner answers, nothing changes: old towns keep no smithy.

Captures (lavapipe, 11:00, the same spot by Holbrook's smith, a capture
world laid fresh and then reopened with Holbrook's stored record set back
to layout 2, nothing else changed):
- `docs/design/img/blacksmith-old-layout.jpg`: layout 2, the smith at work
  on bare ground, no shed, no anvil, no forge: the owner's report;
- `docs/design/img/blacksmith-layout3.jpg`: layout 3, the forge, the anvil
  and the shed round the smith.

## 7. A parcel's pick-up and drop-off

The design is `townsfolk` design 47.
