# Mockup work

The rolling list behind the mockup tracker (`.claude/skills/mockup-tracker`). It is kept in step with
`MOCKUPS` in `docs/mockups/tracker.html`. A mockup is approved for build first, then for the game.
Both approvals are the owner's: they come from the tracker's `reviews` database (read with
`ArtifactData`) or from the owner's own words. Approval for the game never starts game code by
itself (CLAUDE.md, "Write it up before touching code").

## The tracker

https://claude.ai/artifact/EwNcNzHQU3B7C3jyi629Wk (`docs/mockups/tracker.html`, `db` and `user`).

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

**New account (2026-11-13).** The owner moved to another claude.ai account. The artifacts made this
week were published there as new ones, from the pages committed here: M1 to M13, M20 to M24, the
survey and the three benchmark reports of 2026-11-12. Their links below, in the tracker's lists, in
CLAUDE.md and in the OpenSpec changes are the new ones. On the owner's word ("make sure not to publish
the tracker in this session"; "I only care about the artifacts we made this week"), the tracker was
**not** republished: its page here carries the new links, but its live link above is still the old
account's, and so is every page's "All Pale Blue Dot mockups (the tracker)" line until it is. The
older pages (M14 to M19, the guide, the earlier benchmarks and gate pages) keep their old-account
links. The old tracker's reviews and the old survey's answers could not be read from the new account
and stayed behind; the stages recorded below stand. The survey's pictures now go up from
`docs/screenshots/` (`img/g1-*` the town-gates "after" shots, `img/a1-*` and `img/a2-*` the hangar
and drydock shots, `img/t15` and `img/t16` the lighting-perf full-res sets), in two publishes, since
together they pass the 64 MB limit of one. A published page cannot serve `.glb`: ship-crafting,
drydock and hangar go up with base64 `.glb.txt` copies, which their loaders fall back to.

Every page on it links back to it: an "All Pale Blue Dot mockups (the tracker)" line under the
page's title (the thirteen mockups of 2026-11-12, and the survey). The earlier mockups (M14 to M19)
predate the tracker and do not have that line yet; add it the next time each is republished.

## Mockups

Stages: **Listed** (not built), **Drafted** (published, waiting on the owner), **Approved for build**,
**Approved for game**. "Recorded" means the stage comes from the owner's words before the tracker.

| Id | Mockup | Page | OpenSpec | Stage | Link |
| --- | --- | --- | --- | --- | --- |
| M1 | The Aubade System | `planets.html` | `star-systems` | Drafted (2026-11-12) | https://claude.ai/artifact/RKNCXCKChRuNhbg37t9TeE |
| M2 | Aubade Transit | `travel.html` | `interplanetary-travel`, `star-systems` (star lanes) | Drafted (2026-11-12); reworked 2026-11-14 after the owner's "you just lifted off with a vtol to another planet that makes no fucking sense": travel by M21 rockets, the lift and transit drives struck | https://claude.ai/artifact/Rc38usecSUMT2RFF9ur6RZ |
| M3 | Forge and Fabricator | `crafting.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/KEMNicnnXHyoQZ24kWRDep |
| M4 | Shipyard and Ship Builder | `ship-crafting.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/VwFbs16wbr76CSWyrvUGLn |
| M5 | Townsfolk and Peoples | `npcs.html` | townsfolk (extends it) | Drafted (2026-11-12) | https://claude.ai/artifact/RQ8WAnXoJMC5K8dftXWXqS |
| M6 | Hex House Builder | `building.html` | player-building | Drafted (2026-11-12) | https://claude.ai/artifact/Jp6s6FcNR9fBJ1QzBKXWme |
| M7 | Herds, Hunters and Glowmoths | `animals.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/4nZhbMyd6TxusGacLoz1KA |
| M8 | Furrows and Greenhouses | `farming.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/SY1htt7APWUsfQjDVcd94X |
| M9 | Fish Lab and Aquariums | `fish.html` | fishing-and-equipment (extends it) | Drafted (2026-11-12) | https://claude.ai/artifact/KiVnkJ5QK2A8RG4DiUTsWH |
| M10 | Stations and Outposts | `stations.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/KcdhQayNyjEv767JYpWRhk |
| M11 | The Long Fall: Games and Jobs | `minigames.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/ALUjY74NxgYAdXmywbmBeQ |
| M12 | Exchange Terminal | `market.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/XnafkzQo2nAo71ed9y8ZgN |
| M13 | Foliage Detail Playground | `foliage.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/1QMj6cp1ykHNi1cEPxpBPu |
| M14 | Pale Blue Dot Lighting | `lighting.html` | lighting-model | Drafted (2026-10-29) | https://claude.ai/artifact/LLzP5j8G8k3Do9j4Z7Qyaj |
| M15 | Tenebris Towns | `towns.html` | tenebris-towns | Drafted; the towns in the game are built from it, every feature on the roadmap (survey, 2026-09-27) | https://claude.ai/artifact/59zS9ERBTkrVrzPedfoUL2 |
| M16 | Tenebris World Map | `world-map.html` | world-map, city-sites | Approved for build, recorded: "good approve" (survey M6, 2026-09-27); sites "approve" (C4, C5, 2026-09-29) | https://claude.ai/artifact/G7QY9BYXYMH8tgaEv8E7az |
| M17 | Hex Tool Bench | `held-tools.html` | hex-held-tools (archived) | Approved for game, recorded: "Perfect", built and archived 2026-09-26 | https://claude.ai/artifact/YEf1BuLEk9Dppk55znZ5gr |
| M18 | Rod, Shovel and School | `fishing.html` | fishing-and-equipment | Drafted (2026-09-24) | https://claude.ai/artifact/9XiEvkxSFJMtqaktg4eovj |
| M19 | Kestrel, Tern and Loon | `vehicles.html` | vehicles, vehicle-models | Approved for game, recorded: approved, and the Rust port built (vehicles design) | https://claude.ai/artifact/UJebv7EdJUg1NLpvfVHi3D |
| M20 | Pits, Plants and Power | `industry.html` | industry-tiers; steer-and-sail (oil) | Drafted (2026-11-12; version 3 adds oil, 2026-11-13; version 4 adds the five mining tiers, the drilling rig, the offshore platform and the transport tab, 2026-11-14; GPU screenshot gate not run) | https://claude.ai/artifact/9F2Tqi41EFA9tsbTRgxiJD |
| M21 | Rockets and Landers (was Rockets to Starships) | `ship-tiers.html` | ship-tiers | Drafted (2026-11-12, another session); revised 2026-11-13: no warp, rockets four rings wide with decks; revised 2026-11-14: a 2.4 km launch field with Pad B and a walk view, the Planner tab (Taper test moon, the Aubade system, maneuver nodes: survey G4a) | https://claude.ai/artifact/4e3jLcjN4yH6NhJmyWKuY5 |
| M22 | Cranes and Printers | `construction.html` | vehicle-yards | Drafted (2026-11-12); answers M3's note | https://claude.ai/artifact/3iCXM8dTvBb4s7ZDYmFwkn |
| M23 | Drydock and Hex Ship Builder | `drydock.html` | vehicle-yards | Drafted (2026-11-12) | https://claude.ai/artifact/KeUTrW429tn9Vuo9dZy9Bb |
| M24 | Hangar and Chibi Aircraft | `hangar.html` | vehicle-yards | Drafted (2026-11-12); the owner on the flight, 2026-11-13: "it feels pretty good so far", "I like it" (a note, not an approval) | https://claude.ai/artifact/EvFLtkABNcoz9SeAqZduZK |
| M25 | Seas, Rivers and Marshes | `ocean.html` | ocean-environments | Drafted (2026-11-14); three owner questions with screenshots on the page (tides, creatures as pixel cards, deep zones) | https://claude.ai/artifact/JsniCJdwG8b7faX1LRXULR |
| M26 | Pens, Nests and Pedigrees | `husbandry.html` | animal-husbandry | Drafted (2026-11-14); three owner questions with screenshots on the page (Kingdom by form or by world, can a fox take a hen, breeding ticks or arrows) | https://claude.ai/artifact/KQV8S91JncntnsZ1sQZCYE |

The thirteen of 2026-11-12 were published as one batch. Their commit (`d468860`) says "Fourteen",
but it names thirteen subjects, and thirteen pages were published.
