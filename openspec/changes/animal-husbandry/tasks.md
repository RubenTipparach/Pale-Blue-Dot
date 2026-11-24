# Tasks

The mockup comes first and is the whole of this change until the owner
approves it. Game code waits for a request (CLAUDE.md, "Write it up before
touching code"). Groups 4 to 8 are the build in slices, listed so the mockup is
judged against what it would cost.

## 1. The mockup

- [x] 1.1 `docs/mockups/husbandry.html`, "Pens, Nests and Pedigrees": three 3D tabs (the farmstead by day and night, taming a wild herd, the hatchery), and the Kingdom, Breeding, Economy, Engine and Decisions tabs; `tools/art/husbandry_art.py`; the game's art in `docs/mockups/husbandry/game/`, M7's, M25's, M9's, M1's and M8's in `husbandry/m7/`, `m25/`, `m9/`, `m1/`, `m8/`, with a README naming every source. Verify: headless Chromium (SwiftShader, no GPU) shows no console errors; every tab opens and draws; one WebGL context for the whole page; the camera stays put when idle; no horizontal scroll at 390 px.
- [x] 1.2 Screenshots of the farmstead by day and at night, the fox at night, taming, eggs hatching, the life stages, the breeding screen, the Kingdom and the economy, in `docs/screenshots/mockup-husbandry/`. Verify: they are there, and say they were rendered on SwiftShader.
- [ ] 1.3 Publish the page, list it on the tracker and in `docs/mockups/mockup-work.md`, and record its link here. Verify: the page carries the tracker link under its title (it does), and the tracker lists it.
- [ ] 1.4 Put Q1 to Q3 in the survey with their screenshots. Verify: they are in `docs/design/survey.html`.

## 2. The owner's review

- [ ] 2.1 Read the tracker's `reviews` and the survey answers, and fold them into this design. Verify: the stage is in `mockup-work.md`, and the quotes are in `proposal.md`.

## 3. Videos (batch)

- [ ] 3.1 A walkthrough video of the page when the batch is recorded (CLAUDE.md, "For now, screenshots stand in"). Verify: on the gate page.

## 4. The kingdom in data (on request, after approval)

- [x] 4.1 (Built 2026-11-14, slice: `class` and `status` on every fish; the land species are their own append-only roster in `assets/config/husbandry.ron`, design 12 S2; only `home` has life in the game. `husbandry::tests::every_species_has_a_class_a_status_and_a_guide_entry_and_airless_bodies_have_none`, `the_land_roster_keeps_its_order`.) `class` and `status` on every species in `fauna.ron`; the land species appended to `home`, Sequoia's, Wick's and Fallow's rosters. Verify: the append-only test still passes; a test that every species has a class, a status and a field-guide entry, and that an airless body's roster is empty.

## 5. Taming and the animal record (on request, after approval)

- [x] 5.1 (Built: `husbandry::genome`. `the_same_seed_gives_the_same_genome_and_the_morph_shows_one_in_64`.) `pbd_core::husbandry` genome (seven genes, two alleles) rolled from a wild animal's seed. Verify: a core test that the same seed gives the same genome, and the morph shows in 1 of 64 over 64,000 rolls within 10%.
- [x] 5.2 (Built: `husbandry::taming`, `pbd_app::husbandry` E key; the record is a `with ... rec @p animal` line. `a_plodder_takes_five_feeds_and_a_spook_costs_ten`, `a_wild_only_or_protected_species_refuses_with_its_reason`, app `a_tamed_animal_survives_quit_and_reload_with_its_genome`.) Taming by feeding: trust, fear radius, decay; the `animal` record written through the journal `@p` on taming. Verify: a test that a tamed animal survives quit and reload with its genome; a test that a wild-only or protected species refuses with its reason.
- [x] 5.3 (Built: `animal` and `fence` at schema 1 in `identity.ron`. `an_older_record_opens_with_the_missing_gene_and_fields_at_their_defaults`, app `a_world_saved_before_opens_and_gains_no_animals`.) The record's schema version in `identity.ron`. Verify: a world saved before it opens and gains no animals; a v1 record opens under a v2 build with the new gene at its default.

## 6. Pens, needs and products (on request, after approval)

- [x] 6.1 (Built: `husbandry::pen`, pieces as `fence` records on hex edges, S5. `a_closed_loop_with_a_gate_is_a_pen_and_a_broken_loop_is_not`.) Pens derived from fence and gate pieces. Verify: a core test that a closed loop with a gate is a pen and a broken loop is not.
- [x] 6.2 (Built as far as the slice needs, S7/S8: a closed form from the record's last player-written state, no steward and no world writes. `the_closed_form_for_n_hours_equals_n_stepwise_hours`, `neglect_never_kills_and_an_unfed_animal_strays_after_five_days`, `a_process_proposal_changing_a_player_written_field_is_refused`. The steward-run process waits for `world-persistence` group 4.) Needs, mood, health and straying; the `husbandry` world process run by the steward. Verify: the catch-up test (a closed form for N hours equals N stepwise hours); a test that a process proposal changing a player-written field is refused.
- [x] 6.3 (Built for eggs, wool and milk, S9. `a_known_herds_output_matches_the_economy_tab`. Compost and the butcher's block deferred.) Products by schedule and yield; compost from manure; the butcher's block. Verify: a test of a day's output for a known herd against the Economy tab's numbers.
- [x] 6.4 (Built: animals within 150 m as entities, rigged and animated, S11; far pens are not drawn. Captures in `docs/screenshots/animal-husbandry/`. `perf_suite` not run: frame cost not measured.) Animals near the player as entities, far pens as numbers. Verify: captures by day and night beside the mockup's; `tools/perf_suite.py` before and after on the owner's hardware.

## 7. Eggs and young (on request, after approval)

- [x] 7.1 Fertile eggs, broody hens, the incubator; life stages from the birth day (design 13, E1-E5, E10). Verify: a test that an egg set on day d hatches on day d + 3 and lays from day d + 13. (Built 2026-11-15, cloud session. Core: `husbandry::young`, `husbandry::breeding` (`farm_events`, broods, incubators, parentage), the `young` and `breeding` blocks of `husbandry.ron`; tests `an_egg_set_on_day_d_hatches_on_d_plus_3_and_lays_from_d_plus_13`, `an_incubator_needs_its_lamp_and_carried_eggs_go_cold`, `records_before_breeding_read_unchanged`. App, `husbandry/breed.rs`: fertile eggs collected from a hen with an adult cockerel in her pen (a carried `brood` stored ahead of the hands); E with them on an adult hen sets up to 8 under her (she sits, drawn resting, and lays again from the hatching); the incubator (workbench, 4 planks and 2 rope; right click sets it, empty hands take it up; a lit lamp within 3 m); hatchings and births realised with the yard's sync and proposed under `@whusbandry:brood/<id>`; the young drawn at their stage's size and named by it. Saves: `a_world_saved_before_breeding_opens_unchanged_and_writes_nothing`, `fertile_eggs_and_their_parentage_survive_a_reload`. Capture: `docs/screenshots/animal-husbandry/eggs-day.png` (`--husbandry eggs`). Departures: a clutch whose lamp goes out waits rather than going cold; nothing far from the player pairs (its fitness is not read). Frame cost not measured.)

## 8. Breeding (on request, after approval)

- [x] 8.1 Pairing, gestation, cooldown; inheritance with mutation; Wright's F from the pedigree (design 13, E6-E8). Verify: a test that catching up in one evaluation writes the same births as evaluating every half day; a test that a world with no broods opens unchanged and writes nothing; a test that the four predicted values for a gene are exactly the ones births produce over many seeds; siblings give F = 0.25. (Built 2026-11-15: `catching_up_in_one_evaluation_writes_what_half_days_write`, `a_world_with_no_broods_writes_nothing`, `the_four_predicted_values_are_the_ones_births_produce`, `siblings_give_f_a_quarter`, all passing. In play, conception reads each animal's mood and health from its pen now (S7); `--husbandry breed` runs forty days of a pen of one ram and three ewes and logs 19 born.)
- [x] 8.2 The breeding screen and the pedigree (design 13, E9). Verify: a capture beside the mockup's Breeding tab. (Built 2026-11-15: `desktop/breeding.rs`, B on foot or `--menu breeding`; species tabs, the herd, the pair with F and its warning, each gene's four young as ticks with the band and the parents' marks, coat, pattern and morph odds, PAIR writing `mate` on the dam as the player's, and a three-generation pedigree. Tests `b_opens_the_breeding_screen_on_foot_only`, `generations_count_from_the_founders`. Capture `docs/screenshots/animal-husbandry/breeding-screen.png` beside `docs/screenshots/mockup-husbandry/breeding.jpg`.)


## 9. The pen in the game's own wood (design 14; owner, 2026-11-15: "some of the fence shot is missing textures")

- [ ] 9.1 Fences, gates and the incubator in the settlement textures (posts `timber`, rails `poles`, gates `boards`, incubator `plank`) at their `repeat_m`, UVs from the builder; the textured pipeline prewarmed. Verify: a builder test that a textured box's UVs span its faces at `repeat_m`; the eggs capture again beside the first take.
- [x] 9.2 The animals in their coats (design 16): lofted smooth-shaded forms in the `coat` texture tinted by the genome, in place of M26's box hide strips. Verify: close-up BEFORE | AFTER captures of every species.


## 10. The herd's art style and the grazing neck (design 17; owner, 2026-11-23: "his neck bent too far lol, alsoo these guys are a little too smooth. can you make me a mockup of animals? so i can approve the art style?")

- [x] 10.1 Measure today's forms off the game's own geometry: the instrument `dump_the_herds_forms_for_the_art_mockup` (ignored test) writes `docs/mockups/husbandry/art/today.json`; the plodder's neck and face angles, poll and muzzle heights standing and grazing in design 17. No game behaviour changes.
- [x] 10.2 M26's Art style tab (`husbandry.html#art`): the four species, a 1.8 m player and a 1 m grid on the game's ground, one 3D view, styles Today, Neck fixed, Soft low-poly and Carved blocky, stand and graze, Walk off by default, day and night lit by lamps; text in its info modal. Screenshots of every style at 11:00 and 22:30, desktop and phone, and the plodder's head and neck Today against fixed, in `docs/screenshots/animal-husbandry/art-*.jpg`; the tracker's M26 at version 2 and survey question A5.
- [x] 10.3 The neck's carriage and graze (design 17, "The owner's pick"), a pose only, in `husbandry::forms`: standing, the head at about the withers; grazing, the neck lowered from the chest (stretching toward the grass) until the muzzle is 3 cm above it, the face turned down at most 75 degrees. Verified: `the_head_grazes_at_the_grass_and_the_face_never_turns_back` (every beast, every step of the graze, walking and lying: muzzle under 10 cm, face never past vertical), `the_herd_is_built_blocky_with_its_head_at_the_grass` (from `husbandry.ron`: horse 0.044 m, plodder 0.030 m, woolback 0.030 m grazing); BEFORE | AFTER captures in `docs/screenshots/animal-husbandry/herd-blocky-*`.
- [x] 10.4 The owner's pick, M26's blocky animals (not the Art style tab's styles: "DO NOT DO THE SOFT CURVY ANIMALS"), in `husbandry::forms`, wherever the herd is drawn (the yard and the wild through `rig::build`, a cart's shafts through `carts::model::puller`, ridden through `riding`): M26's boxes and proportions, M26's hides laid out by `tools/art/herd_hide.py` into `herd_hide.png` (manifest row) and laid by the metre, tinted by the coat. Verified: the forms tests (whole, sized, lying lower, one hide tile a face, coat tints only colour), the triangle counts (horse 228, plodder 200, woolback 168, hen 180); BEFORE | AFTER captures of the herd standing, grazing and at night, the plodder and the horse in the cart's shafts and a ridden plodder, beside M26's Hatchery tab.
