# Mission audit: does the tracker say where to go? (2026-11-27)

The owner: "Check for similar gaps in other missions where you might've
skipped a step". Every objective in `missions.ron` and `town_missions.ron`
was read against what the player sees while it is in focus: the tracker's
title and objective line, the compass bar and the map. A mission's help text
(`mission.<id>.help`) is shown only in the guide and the tree, not on the
tracker.

## Fixed in this change

| Mission | Gap | Now |
| --- | --- | --- |
| deliver | no word of where a parcel comes from | "Job from <town> <distance>"; carrying, the drop-off town and distance |
| first-job, three-jobs | a job giver in a town, unmarked | "Job from <town> <distance>" |
| lost, mend | a lost or repair job must be taken first; posted only some days | "Job from <town> <distance>"; once taken, the compass and map mark the villager or the wall |
| lamps | the lamps job, posted at dusk | "Job from <town> <distance>" |
| goodwill | earned by jobs and trade in one town | "Job from <town> <distance>" |

## Open: a place the player has to know

The tracker gives no direction for these. Each needs a kind of ground the
player may not have seen. Proposed: the same "nearest <place> <distance>"
ending, from the place search the bot already uses (`seek_target`: hot,
temperate and cold dry ground, a reef, a town), moved out of the bot into
`pbd_app` so both read one rule.

| Mission | Needs | Help says |
| --- | --- | --- |
| reef | a warm reef, dived to | "Dive to a warm reef" |
| cactus | a desert | "Hold the straw to a desert cactus" |
| three-crops | hot ground (chili), cool ground (frost kale) | "Chili grows where it is hot, frost kale where it is cool" |
| sea-fishing | the sea, then fresh water | "Cast from the shore into the sea, then from a river bank or a lake" |
| logs | a river | "Throw logs in a river" |
| horse | a larger town's stables | "Stables in the larger towns sell horses" |
| flock | woolbacks or plodders, in open country | "Woolbacks and plodders graze in open country" |
| ore | ore and coal underground | "Copper and tin show in the rock as mixed ore" |
| find | a chest in a town house | "Open a chest in a town house" |

## Open: words that do not match the game

- **first-job's help says "a town's board"**, and three-jobs' says "Boards
  post new jobs". There is no board in the world: jobs come from one
  townsperson, the job giver, through WORK in the talk panel. Either the
  words change to the job giver, or a board is built in each town for the
  job giver to stand at. That is the owner's call, with screenshots.
- **The job giver is unmarked**: nothing over their head, nothing on the
  compass. In the bot's runs, one town's job giver stood past the drawing
  range and the bot asked four others first. In the survey with screenshots.
- **The tracker repeats its title**: a one-objective mission shows its title
  and then the same words as its line ("CARRY A PARCEL" over "CARRY A PARCEL
  0/1").
- **Stores close at once in towns**: in the bot's run every chest, crate,
  drawer and closet opened and the screen was back to play within 0.4 s. Not
  yet checked with a player's F.
