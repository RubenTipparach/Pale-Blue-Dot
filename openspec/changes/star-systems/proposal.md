# Proposal: Aubade, home of Tenebris and Sequoia, and its neighbours

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

Later the same day the owner corrected the first draft, verbatim:

> pale blue = tenebris, so call it that, sequoia is the sister planet

> also there is no landing on gas giants, some kind of air ship could deploy
> in the atmosphere though, but at a certain point in the atmosphere should
> just be an ocean of hydrogen

> gas giants will generally destroy any spacecraft that reaches a certain
> depth, well before they get to the ocean

> make note that the moons should not be intersecting rings. maybe just shrink
> the planet+ring for now, but keep the proportions for that system (gas giant
> + moons is a system as opposed to solar system)

The first draft had put a second copy of Tenebris, with Sequoia and the rest of
Tenebris's bodies, in a neighbouring system, Nocturne. The home world already
is Tenebris (its generator is `TENEBRIS` v5), so that copy is gone.

## What changes

1. **The home star is Aubade** (an aubade is a dawn song). Recommendation taken
   (ask only with screenshots), for the owner's reason: Halcyon is overused in
   their other games. Every "Halcyon" in the mockups, the minigames art script
   (`pl_halcyon.png` became `pl_aubade.png`, the same bytes), the tracker, the
   mockup list and the mockup-tracker skill's example now says Aubade. Not
   touched by this change, on instruction: `ship-tiers.html` (another thread
   renames it) and `ocean.html` (another agent was writing it).
2. **A local star map** (planets.html, Star map tab): Aubade and two
   neighbours, light years apart, in 3D.
3. **The home world is Tenebris** (the owner's words), everywhere it is the
   planet: planets.html, travel.html and industry.html. "Pale Blue Dot" stays
   the game's name. Its radius (4,800 m, L11), orbit, look and generator
   (`TENEBRIS` v5) are unchanged; nothing about it is a new generator, and no
   save changes (no save stores the display name).
4. **Tenebris's bodies join it in Aubade** (design 3), from the owner's Tenebris
   repository (`tenebris-core/src/planet.rs`): **Sequoia**, its sister, on the
   next orbit out (380 km), with its identity, textures, flora, fauna, vignette
   and roster; Crag innermost, Frostbite outermost; Selene as the home world's
   second moon, Sprout round Sequoia, Quartz round Crag, Hoar (Tenebris's
   Rime, renamed beside Amber's Rime) round Frostbite; the ten belt rocks in
   Aubade's belt. Lantern stands in Luna's place; Verdant is left out (one
   letter from Verdance). Radii go onto the ladder at sixteen times Tenebris's
   (design 2). Their textures are Tenebris's own, copied byte-identical into
   `docs/mockups/planets/tenebris/` (README there).
5. **Tallow, 6.3 ly** (new): a flaring red dwarf; Wick, a tidally locked world
   with a twilight ring of black-leaved life; Sconce, an iron world of
   flare-fused emberglass; Censer, a dark red giant, and its moon Snuff.
6. **Gemel, 8.1 ly** (new): an orange dwarf with a white-dwarf companion; Fallow,
   an empty golden steppe to settle under two suns; Shale, tide-cut slate; Lode,
   a platinum asteroid.
7. **Nocturne is retired**: it held only the copy of Tenebris's bodies; Tallow
   and Gemel are the two more systems the owner asked for.
8. **Star lanes**: how a ship gets there without a warp drive (design 4).
9. **A "Tenebris and Sequoia" tab**: a lit hex vignette of each of Tenebris's
   bodies, day and night, in Tenebris's textures, with its species as box
   animals wearing Tenebris's animal atlas, and the radius mapping.
10. **Gas giants have no landing** (design 6): a "Gas giants" tab with each
    giant's descent profile (cloud decks, the airship band, the limit where
    every craft is destroyed, the hydrogen sea far below it) and a 3D view of
    an airship working the decks, with a hull-pressure gauge and a lost state
    past the limit.
11. **A giant and its moons are drawn as their own system** (design 7): Amber
    and its rings shrunk together with its moons' orbits, so no moon crosses
    the rings.

## Capabilities

- **New (delta only, not built)**: `world/star-systems`, in
  `specs/world/star-systems/spec.md`: systems as separate frames, every body on
  the ladder in every system, star lanes rather than ship speed, ported worlds
  as new generators beside the home world's unchanged one, gas giants with no
  landing and an airship band above a limit depth, and no moon crossing a ring.

## Impact

- Docs and mockups only: `docs/mockups/planets.html`, `travel.html`,
  `industry.html` (the home world's name), `minigames.html` (the first
  draft's rename), `tracker.html`, `mockup-work.md`; art in
  `docs/mockups/planets/` (maps, portraits, `species/`, `tenebris/`);
  `tools/art/star_systems_art.py`, `tools/art/minigames_art.py` (rename);
  `.claude/skills/mockup-tracker/SKILL.md` (example title). The correction
  removed the duplicate Tenebris's, Luna's, Verdant's, the separate belt's and
  Nocturne's art, and renamed Frostbite's moon's files to `hoar`.
- Not touched, on instruction: `ship-tiers.html` (another session owns it; it
  names the home world "Pale Blue" in its orbit HUD, labels, nav computer and
  tables, and the star "Halcyon") and `ocean.html` (another agent is writing
  it; it does not name the home world). The ship-tiers change's own design,
  proposal and spec delta also say "Pale Blue", for that session to rename.
- Saves: none. The home world keeps `TENEBRIS` v5, cell for cell; every body
  added to Aubade would be a new world with its own generator id at v1
  (CLAUDE.md, "Saved games survive every change").
- Frame time: none measured. This is a mockup; the cloud session renders on
  SwiftShader and its frame numbers mean nothing.
