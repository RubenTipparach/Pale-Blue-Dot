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

## Not done: the owner's open reports, in the order to take them

1. **A taken job cannot be made the active mission** (owner: "I dont see a
   place to select that job as active mission"). The tracker's "Carry a
   parcel" is the town tree's mission of that name, not the board job.
   Design 2 of the change has the plan:
   - add `ShownTree::Jobs` and a third plaque, JOBS (`desktop/mission_tree.rs`,
     the `TreeTabs` row);
   - when shown, despawn the tree layer and list `Economy::jobs.taken`, one
     row each, using `economy::job_line` and the reward;
   - a TRACK plaque sets `jobs.progress.focus = Some(job.mission_id())` and
     stores the `JOBS_KIND` record as `Shop::take` does;
   - the tracker (`desktop/missions.rs`, `show_line`) checks that focus first.
     A job's `Mission` has an empty title and empty labels (`Job::mission`),
     so the tracker's title is `job_line` and its one row the job's count;
   - a tree card's TRACK (`mission_tree.rs` around the `focus_on` calls,
     line ~1483) clears the job focus.
   Not started in code.
2. **The placed torch should look like the held one** (design 4). The
   placed torch is the terrain shader's post and cap (`planet_surface.wgsl`,
   lamp section, `LAMP_TORCH`, `t < 30u`). Plan:
   - draw placed torches within the clutter reach as entities from
     `held/light.rs`'s `haft()`, `torch_head()` and flame cards, with
     `LitByField` and `LitLikeTerrain`;
   - stop the shader drawing a torch's post and cap.
   Not started.
3. **Performance degrades the longer the game runs** (owner: "i suspect it
   is due to data not being released"), seen at a reef at night with a torch
   at p50 89 ms. An audit was started and lost: subagents were refused with
   a billing error (`oauth_org_not_allowed`, "payment is past due").
   - What to look for: per-frame `meshes.add` / `images.add` whose old
     handles are kept, spawns without despawns, and collections that only
     grow.
   - The files that create assets most: `desktop/farm.rs` (17 sites),
     `fish.rs` (16), `desktop/map_screen.rs` (12), `fish/places.rs` (11),
     `townsfolk.rs` (10), `critters.rs` (9), `husbandry.rs` (7),
     `held/light.rs` and `held.rs` (6 each).
   - Measure on the desktop with `--frame-log` over a 20-minute session, and
     log entity and asset counts (`Assets<Mesh>::len()`, `Assets<Image>::len()`,
     the entity count) every minute. A count that only climbs is the leak.
4. **The blacksmith has no station** (owner: "the black smith doesnt have
   hist station"). Likely cause: smithies (posts, forge, anvil) stand only in
   towns stored at layout 3 or later (`pbd_core::settlement::record`,
   `FORGE_LAYOUT = 3`). A town first stood under layout 1 or 2 keeps no
   smithy, by design, so its saves stay as played. Check the town's stored
   `layout` in the owner's save.
   - If it is under 3, the choice of whether old towns gain a smithy is the
     owner's. It is a change the world makes to a stored town, so it must
     never overwrite anything the player built. Write it up with screenshots
     for the survey.
   - If it is 3, the smithy's forge is missing for another reason: look at
     `forge_cell` and the template's `smithies`.

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
