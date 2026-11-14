# Proposal: Aubade, its neighbours, and Tenebris's worlds ported

Status: **planning only**. Nothing here is built, and no game code changes. The
mockup is `docs/mockups/planets.html` (M1 on the tracker), with a star-lane
section added to `docs/mockups/travel.html` (M2).

## Why

The owner, 2026-11-14, verbatim:

> update to the halcyon system: need a more original name, you use that name a
> LOT in my games. also look at the planets from tenebris rust, Seqoua mainly
> among other planets, their textures, flora and fauna, should also be ported
> over, just mockups for now. and design at least 2 more star systems near by

Three asks: rename the home star; port Tenebris's planets (Sequoia first) into
the mockups with their textures, flora and fauna; and design at least two more
star systems nearby.

## What changes

1. **The home star is Aubade** (an aubade is a dawn song). Recommendation taken
   (ask only with screenshots), for the owner's reason: Halcyon is overused in
   their other games. Every "Halcyon" in the mockups, the minigames art script
   (`pl_halcyon.png` became `pl_aubade.png`, the same bytes), the tracker, the
   mockup list and the mockup-tracker skill's example now says Aubade. Not
   touched by this change, on instruction: `ship-tiers.html` (another thread
   renames it) and `ocean.html` (another agent was writing it).
2. **A local star map** (planets.html, Star map tab): Aubade and three
   neighbours, light years apart, in 3D.
3. **Nocturne, 4.6 ly**: the star Tenebris called "Sun", named, holding every
   body Tenebris defines (`tenebris-core/src/planet.rs`): Tenebris, Sequoia,
   Crag, Frostbite, their six moons and the ten-rock belt, in Tenebris's own
   order round the star. Each keeps its name, look, gravity and life; each
   radius moves onto the ladder (design 2). Their textures are Tenebris's own,
   copied byte-identical into `docs/mockups/planets/tenebris/` (README there).
4. **Tallow, 6.3 ly** (new): a flaring red dwarf; Wick, a tidally locked world
   with a twilight ring of black-leaved life; Sconce, an iron world of
   flare-fused emberglass; Censer, a dark red giant, and its moon Snuff.
5. **Gemel, 8.1 ly** (new): an orange dwarf with a white-dwarf companion; Fallow,
   an empty golden steppe to settle under two suns; Shale, tide-cut slate; Lode,
   a platinum asteroid.
6. **Star lanes**: how a ship gets there without a warp drive (design 4).
7. **A Tenebris worlds tab**: a lit hex vignette of each ported world, day and
   night, in Tenebris's textures, with its species as box animals wearing
   Tenebris's animal atlas, icons for every species, and the radius mapping.

## Capabilities

- **New (delta only, not built)**: `world/star-systems`, in
  `specs/world/star-systems/spec.md`: systems as separate frames, every body on
  the ladder in every system, star lanes rather than ship speed, ported worlds
  as new generators.

## Impact

- Docs and mockups only: `docs/mockups/planets.html`, `travel.html`,
  `minigames.html`, `industry.html`, `tracker.html`, `mockup-work.md`; new art in
  `docs/mockups/planets/` (maps, portraits, `species/`, `tenebris/`);
  `tools/art/star_systems_art.py` (new), `tools/art/minigames_art.py` (rename);
  `.claude/skills/mockup-tracker/SKILL.md` (example title).
- Saves: none. Every new body would be a new world with its own generator id at
  v1; nothing here touches Pale Blue's generator (TENEBRIS v5) or any save
  (CLAUDE.md, "Saved games survive every change").
- Frame time: none measured. This is a mockup; the cloud session renders on
  SwiftShader and its frame numbers mean nothing.
