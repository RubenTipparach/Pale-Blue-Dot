# Mockup work

The rolling list behind the mockup tracker (`.claude/skills/mockup-tracker`). It is kept in step with
`MOCKUPS` in `docs/mockups/tracker.html`. A mockup is approved for build first, then for the game.
Both approvals are the owner's: they come from the tracker's `reviews` database (read with
`ArtifactData`) or from the owner's own words. Approval for the game never starts game code by
itself (CLAUDE.md, "Write it up before touching code").

## The tracker

https://claude.ai/artifact/PXs3jhCUa5e5tch6Ru4tFu (`docs/mockups/tracker.html`, `db` and `user`).

| Version | Date | What changed |
| ---: | --- | --- |
| 1 | 2026-11-12 | First publish: 19 mockups, 1 guide, 15 misc pages. |
| 2 | 2026-11-12 | The HTML survey added to Misc. |
| 3 | 2026-11-12 | OpenSpec changes named for M5, M6 and M9. |
| 4 | 2026-11-12 | The lighting-perf checkpoint benchmark added to Misc. |
| 5 | 2026-11-12 | M20 Pits, Plants and Power (industry) added. |
| 6 | 2026-11-12 | M21 Rockets to Starships (ship-tiers) added from another session; M20 republished as version 2. |
| 7 | 2026-11-12 | M22 Cranes and Printers, M23 Drydock and Hex Ship Builder and M24 Hangar and Chibi Aircraft (vehicle-yards). |
| 8 | 2026-11-13 | M21 renamed Rockets and Landers (no warp, bigger walkable rockets). |
| 9 | 2026-11-13 | M22 to M24 restored after a republish from main dropped them; M20, M22, M23 and M24 summaries brought up to their latest versions. |
| 10 | 2026-11-13 | M22 to M24 restored on the live page. |
| 11 | 2026-11-13 | M24 version 6: airships, easy flight, more lift, a bigger world, collisions. |
| 12 | 2026-11-13 | One 3D view at a time, in tabs: M24 v7, M1 v6, M5 v7, M9 v4, M17 v11. |
| 13 to 15 | 2026-11-13 to 14 | Published from other branches (the last, from claude/perf-tuning, added the Sounds page as M25). |
| 16 | 2026-11-14 | Every page back on this account: main's lists with this account's links, M25 to M29 added, the Sounds page as M30. |
| 17 | 2026-11-14 | A Roadmap section above the mockups (the owner: "I'm just building a road map for the mockup tracker doc"): the first game loop, environment v2, town building and civil engineering, then the townsfolk. |
| 18 | 2026-11-14 | Town building and civil engineering added after environment v2. |
| 19 | 2026-11-14 | Mass transportation added after civil engineering: airships, sea ships and navigation, hired haulers, conveyors, aqueducts and minecart track. |
| 20 | 2026-11-14 | The age of flight and industry added after mass transportation: aircraft, mass mining, oil, industrialization, advanced power. |
| 21 | 2026-11-14 | Nuclear and rocketry added after the age of flight and industry; advanced power now stops at coal and oil. |
| 22 | 2026-11-14 | The townsfolk moved to right after environment v2 ("townsfolk comes after env v2"). |
| 23 | 2026-11-14 | Space exploration and colonization added after rocketry. |
| 24 | 2026-11-14 | Exo-industry and advanced propulsion added after colonization: exo-industrialization, advanced rocket engines, particle colliders. |
| 25 | 2026-11-14 | Antimatter and interstellar travel added after exo-industry; advanced engines now stop at fusion. |
| 26 | 2026-11-14 | Space freighters added to phase 9 ("9 should also include space freighers"). |
| 27 | 2026-11-14 | Finance and space stations added after rocketry (phase 8): stock market, space trading posts, space casinos. |
| 28 | 2026-11-14 | Space tourism added to phase 9, space exploration and colonization. |
| 29 | 2026-11-14 | Starter missions added to phase 1. |
| 30 | 2026-11-14 | Farming gains seeds from grass and wild plants; those wild plants pulled forward from environment v2. |
| 31 | 2026-11-14 | Phase 1 statuses: farming, herding and cooking built on `claude/phase-1`; tool crafting and starter missions building. |
| 32 | 2026-11-15 | Phase 1 built: crafting tools and starter missions done on `claude/phase-1`. |
| 43 | 2026-11-16 | M23 version 8: a full-size Map window (M) to lay waypoints. |
| 1 (new account) | 2026-11-16 | The owner changed claude.ai accounts: the tracker republished as a new artifact on the new account, every committed page with it, M35 Waterfall mist and bubbles added. |
| 5 (new account) | 2026-11-17 | Tools in Hand Videos (the M36 gate page, https://claude.ai/artifact/1WEmNQzutBwakLrzNWkA3A) added to Misc; M36 version 5, the raised overhead cast. |
| 5 (new account) | 2026-11-17 | M37 Fish Behaviour added (built on the live version 4, which carried M36 Tools in Hand from `claude/peaceful-thompson-fyoxgt`). |
| 7 (new account) | 2026-11-17 | M37 version 2: the Body wave tab (built on the live version 6, which carried M36 version 5 and the Tools in Hand Videos page from another session). |
| 17 (new account) | 2026-11-17 | M37 version 3: the Hooked tab's snapping fixed (built on the live version 16, which carried M36 version 12 from another session; three more bare apostrophes in M36's note escaped, which had stopped the page's script). |

**Back on one account (2026-11-14).** On 2026-11-13 a cloud session republished this week's pages
on a second claude.ai account and moved every link here to it; later sessions added M25 to M29
there. That account cannot be opened from the one that holds the tracker, so on 2026-11-14 ("can
you republish all artifacts (new ones included) and update the tracker?") everything was brought
back to the account that holds the tracker: M1 to M18, M20 to M24 and the Fish Range Atlas were
updated in place at their original links from the pages committed here, M25 to M29 were published
there as new artifacts, the survey (merged from main and claude/perf-tuning) was updated at
https://claude.ai/artifact/Vj1GuwLrhJnXJBTbHo6EyL, and every link below, in the tracker, CLAUDE.md,
the skills and the OpenSpec changes, is that account's. The second account's copies are left as they
are: any tracker reviews or survey answers given there stayed behind and cannot be read from here,
so the stages recorded below stand. M19 (retired), the gate and question pages and the benchmark
reports were already on this account and are unchanged.

Publishing notes: a published page cannot serve `.glb`, so a page's `.glb` files go up as base64
`.glb.txt` copies (ship-crafting, drydock and hangar load them as a fallback). One publish takes at
most 255 files and 64 MB, so world-map, cooking and ui go up in two publishes to the same link. The
survey's pictures come from `docs/screenshots/` (`img/g4a-*` are `mockup-ship-tiers/8-planner-*`;
`img/a1-*`, `img/a2-*`, `img/t15`, `img/t16` as before); files left out of a publish stay up.

Every page on it links back to it: an "All Pale Blue Dot mockups (the tracker)" line under the
page's title. M14, M15, M16 and M18 and the Fish Range Atlas gained theirs on 2026-11-14; M19 is not
republished and has none.

**Two sets of links (2026-11-15).** The links in this table are the tracker's account (main, 2026-11-14,
"Back on one account"). Through 2026-11-15 a cloud session also republished M8, M13, M25, M28 and M29 on
its own account, at the earlier links (M8 `SY1htt7APWUsfQjDVcd94X`, M13 `1QMj6cp1ykHNi1cEPxpBPu`, M25
`JsniCJdwG8b7faX1LRXULR`, M28 `EPXp1JDqhcYez8zHVbfw74`, M29 `TgiEFuMjFZeNcYkjEAUyTh`), with that day's
changes noted in their rows. The pages in this repository are the source of both; the tracker account's
copies of those five are behind until they are republished from here.

**A new account (2026-11-16).** The owner changed claude.ai accounts, and the old account's artifacts can no
longer be read or updated from here (a read of the old tracker, `EwNcNzHQU3B7C3jyi629Wk`, returns "not found").
Every page committed in this repository was republished as a new artifact on the new account: the tracker
(above), M1 to M34 (M19 included, so its link still opens), the Fish Range Atlas, the survey, the bug list,
the phase checklist and the twelve benchmark reports. Every link here, in the tracker, CLAUDE.md, the skills,
the OpenSpec changes and the handoffs is the new account's. The old tracker's `reviews` and the old survey's
`answers` could not be read, so they stayed behind: the stages recorded below stand, and the new databases
start empty. The gate and question pages with no page in the repository (T14's caves, the lights, the map,
taller mountains, exact cell keys) and the Claude Doc survey stay on the old account and keep their old links.

## Mockups

Stages: **Listed** (not built), **Drafted** (published, waiting on the owner), **Approved for build**,
**Approved for game**. "Recorded" means the stage comes from the owner's words before the tracker.

| Id | Mockup | Page | OpenSpec | Stage | Link |
| --- | --- | --- | --- | --- | --- |
| M1 | The Aubade System | `planets.html` | `star-systems` | Drafted (2026-11-12) | https://claude.ai/artifact/8af8Ptmnk85Df5Z7fHBYbf |
| M2 | Aubade Transit | `travel.html` | `interplanetary-travel`, `star-systems` (star lanes) | Drafted (2026-11-12); reworked 2026-11-14 after the owner's "you just lifted off with a vtol to another planet that makes no fucking sense": travel by M21 rockets, the lift and transit drives struck | https://claude.ai/artifact/2Dx7NDZkPCEQNsJgFKyEvL |
| M3 | Forge and Fabricator | `crafting.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/SbqjJReUi6hMU1CGkisqbg |
| M4 | Shipyard and Ship Builder | `ship-crafting.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/9zhqTWtaKXS7YENoMem5ei |
| M5 | Townsfolk and Peoples | `npcs.html` | townsfolk (extends it) | Drafted (2026-11-12) | https://claude.ai/artifact/5aJfbfT7LgVuKpmEPhAZrv |
| M6 | Hex House Builder | `building.html` | player-building | Drafted (2026-11-12) | https://claude.ai/artifact/SWgY1ooB8f1nk8rhkoEnCm |
| M7 | Herds, Hunters and Glowmoths | `animals.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/QQSF6emzmzeK9Hej4p9nv3 |
| M8 | Furrows and Greenhouses | `farming.html` | none yet | Drafted (2026-11-12); art-style pass 2026-11-15 after the owner's "outlines are for sprites in the ui inventory system, there is no need for that in the world": the plot draws outline-free world stages (`farming/world/`, `farming/world_art.py`), icons keep their outline, the reading text moved into an About modal; shots `docs/screenshots/mockup-farming/` | https://claude.ai/artifact/M2LQYN69c7YtLvQJp9fQQ7 |
| M9 | Fish Lab and Aquariums | `fish.html` | fishing-and-equipment (extends it), ocean-environments | Drafted (2026-11-12); the library of everything underwater (owner, 2026-11-15: "keep that mockup updated since that represents our \"Fish\" library", "fish and aquariums doc should have at least 1 of everything"): darter and anthias added, a Reef & sea floor tab with the game's own 41 models in the game's atlas, text in modals; the game's fish built from it (version 6, 2026-11-15) | https://claude.ai/artifact/Ay54k8LEf4sdteFpZkeSsG |
| M10 | Stations and Outposts | `stations.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/Rrf9Da4tPFJkpejGJrzHB8 |
| M11 | The Long Fall: Games and Jobs | `minigames.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/R7453rhoQ2iJxEH3fJaYZv |
| M12 | Exchange Terminal | `market.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/6nCcnt9gLFhcksg72pfjRZ |
| M13 | Foliage Detail Playground | `foliage.html` | flora-v2 | Approved for build and for the game (owner, 2026-11-15, in chat: "m13, 25,28 water and env v2 are apopoved locked and loaded"); building on `claude/env-v2`. Drafted (2026-11-12); art-style pass 2026-11-15 ("first default to pixel tufts, make sure its color matches the ground"; "harsh outlines and contrasting details, this is bad"): tufts by default, coloured from each biome's ground tile; no outlines in the world (`foliage/world_art.py`), plant contrast pulled to each texture's mean; new Crops and gardens layer (M8's crops and stages) and Wild fruit layer with regional fruit per biome (`foliage/fruit_art.py`); shots `docs/screenshots/mockup-foliage/`. Fix pass 2026-11-15 (owner's six notes): fallen autumn leaves in the terrain shader (pixel leaves in the ground's texels, thicker under trees, Fallen leaves / Gather under trees / Leaf size sliders, Let them fall off by default, browned on winter snow, none in spring or summer); painted flowers off the fields, swamp and tundra ground tiles (`foliage/no_flowers.py`, `ART_V` 10); far trees baked in no season, so summer no longer keeps an autumn bake (peach far trees, reproduced in Chromium), and the canvas and `readPixels` read-backs tested before use (Firefox resistFingerprinting); savanna acacias and bushes in a dry olive; fungal forest in violet caps, warm stalks and dark violet moss, its glow from the cap spots; fruit and pine cones hang from each tree's branches; before and after `fix-*.png`. Approved 2026-11-15 ("commence implementation of all env changes"); into the game on `claude/env-flora` (`flora-v2` design 9, tasks 6): tufts from the ground tile, contrast to the mean, painted flowers off the atlas, clutter reach doubled, fallen leaves, fruit and cones on the trees, dry olive beach crowns; not built: the winter sod lip (no seasonal snow in the game), unbaked far trees (no impostors), the fungal forest (a new generator version) | https://claude.ai/artifact/P4jA4m4MAMSFnWS4cc9Pb1 |
| M14 | Pale Blue Dot Lighting | `lighting.html` | lighting-model | Drafted (2026-10-29) | https://claude.ai/artifact/PR6epQcL3YtE8UWRsmc9zg |
| M15 | Tenebris Towns | `towns.html` | tenebris-towns | Drafted; the towns in the game are built from it, every feature on the roadmap (survey, 2026-09-27) | https://claude.ai/artifact/B5FqbGTTMsznmUiP4rSLUv |
| M16 | Tenebris World Map | `world-map.html` | world-map, city-sites | Approved for build, recorded: "good approve" (survey M6, 2026-09-27); sites "approve" (C4, C5, 2026-09-29) | https://claude.ai/artifact/V6hhxR5fov5XGWVFD2s2Eq |
| M17 | Hex Tool Bench | `held-tools.html` | hex-held-tools (archived) | Approved for game, recorded: "Perfect", built and archived 2026-09-26 | https://claude.ai/artifact/V5xpTns8YqzGJPumBaij6f |
| M18 | Rod, Shovel and School | `fishing.html` | fishing-and-equipment | Drafted (2026-09-24) | https://claude.ai/artifact/1PQXEj8bs9HSzFhGLRyxtz |
| M19 | Kestrel, Tern and Loon | `vehicles.html` | vehicles, vehicle-models, retire-kestrel | Approved for game, recorded: approved, and the Rust port built (vehicles design). Kestrel retired (owner, 2026-11-14): the game has the Tern and the Loon; the page is the design record of the retired craft and is not republished | https://claude.ai/artifact/AbkZFGps6XzcwhfbAYPwDk |
| M20 | Pits, Plants and Power | `industry.html` | industry-tiers; steer-and-sail (oil) | Drafted (2026-11-12; version 3 adds oil, 2026-11-13; version 4 adds the five mining tiers, the drilling rig, the offshore platform and the transport tab, 2026-11-14; version 5 builds the freighter and tanker of hex cells and carries cargo in hex pods (owner, 2026-11-14); version 6 builds the fission plant's cooling tower and containment dome of hex cells (owner, 2026-11-14); GPU screenshot gate not run) | https://claude.ai/artifact/FUwKjVy7eaLZKCTeEH8kVK |
| M21 | Rockets and Landers (was Rockets to Starships) | `ship-tiers.html` | ship-tiers | Drafted (2026-11-12, another session); revised 2026-11-13: no warp, rockets four rings wide with decks; revised 2026-11-14: a 2.4 km launch field with Pad B and a walk view, the Planner tab (Taper test moon, the Aubade system, maneuver nodes: survey G4a) | https://claude.ai/artifact/Af9oUZTdnhRzF1UYtxqUq6 |
| M22 | Cranes and Printers | `construction.html` | vehicle-yards | Drafted (2026-11-12); answers M3's note | https://claude.ai/artifact/42o1sitc2KvFvVCpfQyHSv |
| M23 | Drydock and Hex Ship Builder | `drydock.html` | vehicle-yards | Drafted (2026-11-12) | https://claude.ai/artifact/K2uLF3kCJGu57tg4bk6ect |
| M24 | Hangar and Chibi Aircraft | `hangar.html` | vehicle-yards | Drafted (2026-11-12); the owner on the flight, 2026-11-13: "it feels pretty good so far", "I like it" (a note, not an approval) | https://claude.ai/artifact/WUugWkXMLPRKMGVMd4w9mG |
| M25 | Seas, Rivers and Marshes | `ocean.html` | ocean-environments | Approved for build and for the game (owner, 2026-11-15, in chat: "m13, 25,28 water and env v2 are apopoved locked and loaded"); building on `claude/env-v2`. Drafted (2026-11-14); revised for the owner's review of 2026-11-15 (design 11): animals 3D (Q2 answered; a dropped catch is a 2D item sprite), a packed instanced reef, the waterfall's z-fighting fixed, a Water sim tab on Luanti's liquid rule (shared with M28 via `water-flow/liquid.js`), text in modals; two questions open with screenshots (tides, deep zones); the owner's phone notes on M28 of 2026-11-15 taken in its Water sim too (water-flow design 7b, 7c): the sim patch's edge walled to a stone base with still water walls where water meets it, falls and caps sorted by a depth-only pass, an "Arrows: on / off" toggle, flow on every flowing cap, and Tenebris's water tile panned along each cap's flow and down each fall (the river's and dam's caps too; the sea and coast unchanged); the owner's ledge note of 2026-11-15 ("you missed a critical part of the luanti/minetest water implementation that shows the water falling", "water should generate blocks vertically as it falls too"; water-flow design 7e): Luanti's liquid mesher (`drawLiquidNode`) ported to hex in `water-look.js`, every water cell drawn with corner-averaged sloped tops, walls where water meets air and each falling cell a whole block, the flow from the corner slope, the rule unchanged; screenshots `mesher-*.png`; the owner's merge note of 2026-11-15 ("water that flows - make sure the rivers use that too, so merge these artifacts together so we get it more consistent"; ocean-environments design 12): the river, the dam's stream and pond and the marsh pools simulated by `liquid.js` and drawn by `liquidMesh`, settled before the first frame (the river steps a layer every 10 m on the meanders, as the rule needs; the dam spills over a spillway and breaches live), and M28 folded into the Water sim tab with all its presets and controls; screenshots `river-sim-*`, `dam-sim-*`, `marsh-sim-*`, `sim-presets-*`; to be republished | https://claude.ai/artifact/1vWj3uLNoqYh7vgguNjKai |
| M26 | Pens, Nests and Pedigrees | `husbandry.html` | animal-husbandry | Drafted (2026-11-14); three owner questions with screenshots on the page (Kingdom by form or by world, can a fox take a hen, breeding ticks or arrows) | https://claude.ai/artifact/RuC2dK1SikwQ7tyXWbne1J |
| M27 | Hearth and Kitchen | `cooking.html` | cooking | Drafted (2026-11-14); three owner questions with screenshots on the page (buffs or a hunger bar, fifteen stations or five, heat as the cook's choice) | https://claude.ai/artifact/FSzNd9iJoTZUks2rNvRW4R |
| M28 | Water That Flows | `water-flow.html` | water-flow | Approved for build and for the game (owner, 2026-11-15, in chat: "m13, 25,28 water and env v2 are apopoved locked and loaded"); building on `claude/env-v2`. Drafted (2026-11-14); the owner asked for it ("bring in code from Luanti ... in a water flow demo mockup"); renew at three and range 8 taken as recommendations, with a toggle and a slider on the page; synced with M25's Water sim (2026-11-15, "sync m28 with these changes"): frothy fall walls, flow streaks and Flow arrows, drawn by `water-flow/water-look.js` which both pages load; text in a modal; the owner's phone notes of 2026-11-15 (water-flow design 7b, 7c): the patch cut with walls to a stone base and the outer sea sheet removed ("The very front part of the cell doesn't have any geometry"), falls and caps sorted by a depth-only pass ("zdepth issues with the waterfall"), an "Arrows: on / off" toggle (the game's overlay proposed, task 3.4), every flowing cap pointing the way its water goes ("the edges also dont have flow"), and Tenebris's water tile panned along each cap's flow and down each fall ("UV scrolling ... how tenebris c does it"; the game's proposed, task 3.5); screenshots `flowtex-*.png`; the owner's ledge note of 2026-11-15 ("you missed a critical part of the luanti/minetest water implementation that shows the water falling", "water should generate blocks vertically as it falls too"; water-flow design 7e): Luanti's liquid mesher (`drawLiquidNode`) ported to hex in `water-look.js`, every water cell drawn with corner-averaged sloped tops, walls where water meets air and each falling cell a whole block, the flow from the corner slope, the rule unchanged; screenshots `mesher-*.png`; merged into M25 (owner, 2026-11-15): its sim, patch, presets and controls are M25's Water sim tab, and `water-flow.html` is a short page pointing to `ocean.html#sim` with the tracker and licence links; to be republished | https://claude.ai/artifact/3jCWBJYacgu5eqjwz6vux2 |
| M29 | Stone and Timber | `ui.html` | stone-and-wood-ui | Approved for build (2026-11-14: "Smooth is the best. Write it down as the decision and start working the current UI as the wood + rendered smooth rock"); Q1 answered: Smooth | https://claude.ai/artifact/P2KRZ5FuSwVCy2U3seCHHU |
| M30 | Pale Blue Dot Sounds | `sounds.html` | audio-and-ambience | Drafted (2026-11-14, published from claude/perf-tuning; version 2 carries the owner's first notes). It was listed there as M25 before main's M25 to M29 took those ids | https://claude.ai/artifact/39Zs5Q6ja1sGwQvCMANshJ |
| M31 | Waterfall Look | `waterfall.html` | water-flow | Pour tab approved (2026-11-15: "the waterfall look is pretty good, need to be a bit careful with perf on that"); drafted (2026-11-15); the owner's rule that falls are decided in a mockup first ("make mockups from now on for how we're gonna do the waterfall part"); variants A today, B stretched 2x ("stretched out, like maybe 2x of the original"), C lean, D lip and speed-up; version 2 (2026-11-15) adds the Pour tab, a live pour that pops as today or eases between ticks ("is there a better way to lerp the sim?", water-flow design "Smooth water between ticks") | https://claude.ai/artifact/R29tnUCPw5mnJ7uC2hwzTR |
| M32 | Rain Splashes | `rain-splashes.html` | weather-rain | Approved (2026-11-15: "approve rain splashes"); drafted (2026-11-15; "one of the things were missing are puddle splashes on the ground", "the more gpu native rain we can have the better") | https://claude.ai/artifact/PRtdnwAzXzS7JX5RYbcAPB |
| M33 | Mission Tree | `mission-tree.html` | starter-missions | Approved for the game (2026-11-15, version 3: "that tree is better, I approve", "implement in game"); drafted (2026-11-15; "Maybe make a mock up of mission tree for me") | https://claude.ai/artifact/V1yL4DmywiEhC7Pv3AKjfY |
| M34 | River Craft | `river-craft.html` | river-craft | Version 2 (2026-11-16: the Tube tab, "also what about innertubes?"); drafted (2026-11-16; "can you make a mock up of stuff like rafting physics? or design a boat that can tug other boats? log hauling? boats with nets, also anchoring") | https://claude.ai/artifact/93iZ1pBszzY8u6gmWZFRyJ |
| M35 | Waterfall mist and bubbles | `waterfall-mist.html` (on `claude/fall-splash-order`) | water-flow | Approved for build and for the game (owner, 2026-11-16, in chat: "alright, I'm happy with this, send it yo", on version 22); the game port requested in the same message | https://claude.ai/artifact/WzzTsEVPa3SviHQBfnanMo |
| M36 | Tools in Hand | `tool-motion.html` | held-swing, rod-motion, hand-model | Version 12 (2026-11-18: the rod's flex and follow-through from the owner's lure-rod GIF; right click cancels at the bite too and the prompt shows both buttons, built in the game too; the thumb's root sunk into the palm; the cast's line drawn along the float's path from behind the head; the Fish tab's steps beside the view, right click called cancel; the float no longer sticks at the water's edge for the first frames of the cast, a mockup bug; the owner: "the first few frames of releasing the line the thing gets stuck"). Version 11 (2026-11-17: the owner on version 10: "You know what that hand looks good"; the shovel's arm lowered, `fist_drop_m` 0.16, `arm_follow` 0.7). Version 10 (2026-11-17: the hand's shape from the owner's own fist: a traced palm joined to the thumb, slimmer tapered fingers, a smaller thumb with a round tip, stronger palm and back tones, the palm unwrapped with its lines, baked AO, the fingertips flushed, a grip turn of 45°; hand-model design 5.1 and 5.2; drafted, the game waits). Version 9 (2026-11-17: the palm and back on the Hand tab, hand-model design 5; the owner: "usually back of the hand and fingers a little darker than palm, and palm has kind of that palm texture"; drafted, the game waits). **Approved for the game** (owner, 2026-11-17, in chat: "tools in hand approved, implment"), at version 8: held-swing (the four-stage strokes, the shoulder arc, the steady beat, the sliding shovel) and hand-model (the proposed hand). Version 8 (2026-11-17: the thumb's two segments equal). Version 7 (2026-11-17: the shovel arm follows the whole turn, a straight wrist). Version 6 (2026-11-17: the shovel tips down, slides forward into the dirt, tips and pulls it up; the proposed thumb in two segments). Version 5 (2026-11-17: the cast raised above the shoulder, rod-motion decision 7; the step's videos at https://claude.ai/artifact/1WEmNQzutBwakLrzNWkA3A). Version 3 (2026-11-17: the game's reel fight, a Hand tab with a proposed hand, the four-stage strokes and the shovel's scoop); drafted (2026-11-17; "make a new mockup for all the tools stuff for digging, choppin and fishing") | https://claude.ai/artifact/Xx3xB3ouaKdJ2NdnxAL9PX |
| M37 | Fish Behaviour | `fish-behaviour.html` | fish-behaviour | Drafted (2026-11-17; the owner on the under-water bite video: "the fish behavior is super unnatural", "why is rotation all snappy?", "the fish is glitching like hell"). Today (the game's rules, ported) beside Proposed: approach, pecks timed to the float, the take, the fight, banking and a tail tied to speed. Version 2 (2026-11-17) adds the Body wave tab (the owner: "the fish now turns correctly, but for some reason doesnt move convincingly?"): today's tail sway beside a vertex-shader body wave for the perch, eel and ray. **Approved for the game (2026-11-17; the owner: "New fish behavior approved")**, versions 1 and 2 (the lure, swimming, hooked and body-wave proposals) Version 3 (2026-11-17; the owner on the Hooked tab: "theres some snapping around in the hooked version"): the float's swing jump eased as the game's `DrawnFloat` does, the fight's turn rate eased, shake and flash faded in and out on the frame's clock and turned about the mouth, the replay angler no longer pumps the reel (design 3.1). Game code waits for a request | https://claude.ai/artifact/BkznFkfte6Ph8GLKWfczUz |

The thirteen of 2026-11-12 were published as one batch. Their commit (`d468860`) says "Fourteen",
but it names thirteen subjects, and thirteen pages were published.
