# Handoff: playtest fixes on `claude/blissful-mayer-r0fo9a` (2026-11-25)

From a cloud session (no GPU: nothing here was played on real hardware, and no
frame cost was measured). The branch is main of 2026-11-25 plus the commits
below. There is no open PR for it yet; CLAUDE.md says one PR, so open one when
asked.

## Get it

```bat
git fetch origin
git checkout claude/blissful-mayer-r0fo9a
git pull
build-play.bat
```

If the link step fails with `undefined symbol ... .llvm.<n>`, two builds wrote
`target\fast` at once: build once, alone; if it still fails delete
`target\fast\incremental`.

## Done on this branch (newest last)

| What | Where | Proof |
| --- | --- | --- |
| Mission ding: one short ding when a mission completes (owner: "just the ding", "keep it short") | `tools/audio/gen_sfx.py` `mission_done`, `assets/audio/sfx/mission_done.ogg`, played on `missions::MissionDone` in `audio/hooks.rs` | `inspect_sfx.py check` ok; not listened to here |
| Main merged in; main's mission tracker kept, moved to the left under the position readout | `desktop/missions.rs` (`left: 6, top: 62`) | builds; mission tests pass |
| Goal text as instructions ("Pull fiber", "Set it down", "Catch fish on bait"), both trees, 109 labels; distances in metres | `assets/locale/locale_en.yaml` `mission.*` and `town_mission.*` `objective`; `missions.objective_metres` | core and app mission tests |
| Townsfolk stop to talk: they hold where they stood, face you, and walk on from there after (no jump) | `townsfolk.rs` `hold_for_talk`, `place_of`, `Live::held`, `Live::behind` | `a_talk_holds_a_person_and_they_go_on_from_where_they_stopped` |
| Picked wild plants grow back after 2 game days (wild crops fade back in; glowing mushrooms keep their out-of-sight rule, 8 days to 2) | `pbd_core::farming::WILD_REGROW_DAYS`, `Picked::grown_back`, `farm::wild_picked`; `flora::GLOW_PLANT_REGROW_DAYS` | `a_picked_wild_plant_grows_back_in_two_days`, the edits test |
| A townsperson asleep in bed has their eyes shut (open again if you talk to them) | `pbd_core::townsfolk::face::Face::set_asleep`, set from `Posture::Lie` in `townsfolk.rs` | `a_sleeping_face_keeps_its_eyes_closed` |

The write-up for all of these is `openspec/changes/playtest-fixes-2026-11-25/`
(proposal, design sections 1 to 5, tasks).

## Done after (2026-11-25, on `claude/phase-2-1`)

| What | Where | Proof |
| --- | --- | --- |
| A JOBS tab in the Missions screen (beside STARTER and TOWN): every taken job, its town and reward, and TRACK, which makes it the tracker's mission; a card picked on a tree, or the job handed in or given up, clears it (design 2) | `desktop/mission_tree.rs` (`ShownTree::Jobs`, `paint_jobs`, `press_track`), `desktop/missions.rs` (`tracked_job_mission`), `economy::track_job`, `TakenJobs::hand_in` | `a_tracked_job_is_the_trackers_until_a_card_is_picked`, `a_tracked_job_handed_in_is_no_longer_tracked`; `jobs-tab.jpg`, `jobs-tracker.jpg` |
| The placed torch is the held torch: haft, head and flame, lit like the terrain, faded in and out within the clutter's reach; the shader draws no torch body; the light is unchanged (design 4) | `held/placed.rs`, `planet_surface.wgsl` lamp section | `the_placed_torch_stands_where_the_shaders_did`, `the_haft_is_not_flat_shaded`; `placed-torch-day.jpg`, `placed-torch-night.jpg`, `placed-torch-row-night.jpg` |
| The blacksmith's station: a finding, not a bug. Smithies stand only in towns laid at layout 3 (from 2026-11-24); a town laid before keeps none, while its smith (derived) stands at work on bare ground. Recommendation and captures for the survey (design 6) | `openspec/changes/playtest-fixes-2026-11-25/design.md` 6 | `docs/design/img/blacksmith-old-layout.jpg`, `blacksmith-layout3.jpg` |
| "Carry a parcel": the parcel waits with the posting town's trader and is handed over when you talk to them; the job's town is marked on the compass and the map ("Parcel drop-off"); the tracker reads "Get the parcel from the trader", then "Take the parcel to {town}". A save from before keeps its parcel and gains the marker (`townsfolk` design 47) | `TakenJobs::waiting`, `Shop::pick_up`, `economy::parcel_directions`, `desktop/talk.rs`, `compass_bar.rs`, `map_screen.rs` | `a_parcel_waits_until_picked_up_and_an_old_record_has_none_waiting`, `a_delivery_hands_its_parcel_to_the_other_town`, `a_parcel_taken_before_pick_ups_keeps_its_parcel_and_gains_the_marker`; `parcel-*.jpg` |

Captures are in `docs/screenshots/playtest-fixes/` (lavapipe in a cloud
session: what things look like, never how fast). They include the ones
owed from before: the tracker on the left (every capture shows it), a
talker held facing you (`talker-stopped.jpg`, the merchant 200 frames into a
talk), a sleeper's shut eyes (`asleep-night.jpg`) and a wild plant picked a
day ago, gone, and picked over two days ago, grown back (`wild-*.jpg`).

## Still open

1. **Performance degrades the longer the game runs** (owner: "i suspect it
   is due to data not being released"), seen at a reef at night with a torch
   at p50 89 ms. Taken by another session on real hardware.
   - What to look for: per-frame `meshes.add` / `images.add` whose old
     handles are kept, spawns without despawns, and collections that only
     grow.
   - Measure on the desktop with `--frame-log` over a 20-minute session, and
     log entity and asset counts every minute. A count that only climbs is
     the leak.
2. **Old towns and their smithy** waits for the owner's answer in the
   survey (design 6, with its two captures). Nothing changes until then.
3. **The placed torch's frame cost** was not measured (no GPU here): each
   near torch is two field-lit meshes and a flame rebuilt each frame, at
   most 48. Run `perf_suite.py` on the desktop with a row of torches.

## Checks run here

- `cargo check -p pbd-app --all-targets`, `cargo fmt --check`,
  `openspec validate --all`: clean at the last commit.
- Tests run: core missions, farming, edits, flora and townsfolk; app missions,
  mission tree, hooks, town missions, locale, farm, glow, construction,
  planet column and townsfolk. All pass. The full suites were not run after
  the merge (this container ran out of disk twice); run them on the desktop.
- Nothing was captured in the game after the merge: no screenshots of the
  tracker on the left, the regrowth, the stopped talker or the shut eyes.
  Those are the first things to look at in play.
