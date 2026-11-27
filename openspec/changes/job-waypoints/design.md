## 1. Where each mark stands

- **Repair:** `special::gap_point` of the job's `Repair` record, the middle
  of the broken wall at its storey. The gap needs the town's chart, so the
  mark is there while the town is held (inside the town ring, which is where
  the board that gave the job is).
- **Rockfall:** `roads::centre` of the first cell of the `Rockfall` record.
  Every cell is a road cell, so the mark sits on the road, and it moves on
  as cells are cleared only if the clear order changes the record. The
  first cell is used as is.

## 2. On the bar and the map

The same pins the lost villager, the trader and the parcel use: up to two of
each kind, the compass naming the faced pin "Repair 120 m" or "Rockfall
1.4 km", and the map showing one pin per mark with its label. Order:
- lost;
- trader;
- parcel;
- repair;
- rockfall;
- site;
- pave;
- the player's own marks.

## 3. Decisions

Names and order are recommendations taken (ask only with screenshots). The
icon is the existing waypoint pin.

## 4. The parcel's distance on the tracker (owner, 2026-11-27)

"Where is carry parcel marker on compass? Map? Would also be nice to see
estimate of distance on carry parcel goal text".

The parcel pin is on the bar and the map only once a parcel is carried. Taking
a delivery job hands you the parcel, and the pin is its town. Until then the
"Carry a parcel" mission has nowhere to point but the towns themselves, and
those are already on the bar.

A deliver objective on the tracker now ends with the carried parcel's town
and its distance, as the compass names it: "Carry a parcel 0/1  Holbrook
1.2 km". It shows the nearest carried parcel and nothing when none is carried.

## 5. Where to get a parcel (owner, 2026-11-27)

"So how could carry parcel show up and not tell you where to go to pick up a
parcel". The deliver mission assumed the player knew a delivery job comes from
a town's job giver, and nothing in the game said so. With no parcel carried,
the deliver line names the nearest town and its distance: "Carry a parcel 0/1
Job from Holbrook 1.2 km". Once a parcel is carried it names the drop-off
instead (design 4). Marking the job giver in the town itself is the survey's
question.

## 6. Every job-made mission says where the job is (owner, 2026-11-27)

"Check for similar gaps in other missions where you might've skipped a
step". The town tree's first-job, three-jobs, lost, mend, lamps and goodwill
all need a job from a town's job giver first, as deliver does, and their
tracker lines said nothing of it. Their objectives (job, escort, repair,
light, goodwill) take the same "Job from Holbrook 1.2 km" ending as deliver.

The audit of every mission's objectives, and what the tracker gives the
player for each, is the change's `audit.md`.

## 7. The job giver pinned (owner, 2026-11-27)

"it is just that delivery missions are so precise and theres only usually a
few specific locations to start and finish, it should just say where to go".
Within 600 m of a town, its job giver's post (`Person::at` of the keeper the
jobs config names) is pinned on the compass bar ("Job giver 40 m") and on the
map. With the tracker's "Job from <town>" line, a delivery now names both
ends: the job giver to start, the drop-off town to finish.
`docs/screenshots/job-waypoints/giver-pin-compass.png`: the red pin by NW,
beside Holingmouth's studs.
