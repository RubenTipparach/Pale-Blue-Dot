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

Every page on it links back to it: an "All Pale Blue Dot mockups (the tracker)" line under the
page's title (the thirteen mockups of 2026-11-12, and the survey). The earlier mockups (M14 to M19)
predate the tracker and do not have that line yet; add it the next time each is republished.

## Mockups

Stages: **Listed** (not built), **Drafted** (published, waiting on the owner), **Approved for build**,
**Approved for game**. "Recorded" means the stage comes from the owner's words before the tracker.

| Id | Mockup | Page | OpenSpec | Stage | Link |
| --- | --- | --- | --- | --- | --- |
| M1 | The Halcyon System | `planets.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/7gnj2sgXFmYnMzNWEPE7pG |
| M2 | Halcyon Transit | `travel.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/7PCRHRjCkBk3mesWsd6Dfq |
| M3 | Forge and Fabricator | `crafting.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/6hj8UvbLY9TUsn7591sKKg |
| M4 | Shipyard and Ship Builder | `ship-crafting.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/ANihdavwCucwXDCq2YzFvY |
| M5 | Townsfolk and Peoples | `npcs.html` | townsfolk (extends it) | Drafted (2026-11-12) | https://claude.ai/artifact/LECf4Jk3ESP54w5JV55LMc |
| M6 | Hex House Builder | `building.html` | player-building | Drafted (2026-11-12) | https://claude.ai/artifact/NPvmoZMkdg6HEc91Vngicp |
| M7 | Herds, Hunters and Glowmoths | `animals.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/GWCsAqGcVi75rHPYVGLye7 |
| M8 | Furrows and Greenhouses | `farming.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/4uqkQQB55Hq8dbEc52scwD |
| M9 | Fish Lab and Aquariums | `fish.html` | fishing-and-equipment (extends it) | Drafted (2026-11-12) | https://claude.ai/artifact/VbVmuUHVTUczkiMYGHEZSn |
| M10 | Stations and Outposts | `stations.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/7tPp8n2btkNzx1tgeQF7P4 |
| M11 | The Long Fall: Games and Jobs | `minigames.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/M6bJTTQmSRcfZoGyjAZZvD |
| M12 | Exchange Terminal | `market.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/MepSVNts31YTwsZ4QmZhPK |
| M13 | Foliage Detail Playground | `foliage.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/PnPpDKgcqqbGyBHdXxsiBF |
| M14 | Pale Blue Dot Lighting | `lighting.html` | lighting-model | Drafted (2026-10-29) | https://claude.ai/artifact/LLzP5j8G8k3Do9j4Z7Qyaj |
| M15 | Tenebris Towns | `towns.html` | tenebris-towns | Drafted; the towns in the game are built from it, every feature on the roadmap (survey, 2026-09-27) | https://claude.ai/artifact/59zS9ERBTkrVrzPedfoUL2 |
| M16 | Tenebris World Map | `world-map.html` | world-map, city-sites | Approved for build, recorded: "good approve" (survey M6, 2026-09-27); sites "approve" (C4, C5, 2026-09-29) | https://claude.ai/artifact/G7QY9BYXYMH8tgaEv8E7az |
| M17 | Hex Tool Bench | `held-tools.html` | hex-held-tools (archived) | Approved for game, recorded: "Perfect", built and archived 2026-09-26 | https://claude.ai/artifact/YEf1BuLEk9Dppk55znZ5gr |
| M18 | Rod, Shovel and School | `fishing.html` | fishing-and-equipment | Drafted (2026-09-24) | https://claude.ai/artifact/9XiEvkxSFJMtqaktg4eovj |
| M19 | Kestrel, Tern and Loon | `vehicles.html` | vehicles, vehicle-models | Approved for game, recorded: approved, and the Rust port built (vehicles design) | https://claude.ai/artifact/UJebv7EdJUg1NLpvfVHi3D |
| M20 | Pits, Plants and Power | `industry.html` | none yet | Drafted (2026-11-12) | https://claude.ai/artifact/JBcP2Js3bdQ2uApN3xUnHV |

The thirteen of 2026-11-12 were published as one batch. Their commit (`d468860`) says "Fourteen",
but it names thirteen subjects, and thirteen pages were published.
