## 1. Foundations in the core

- [ ] 1.1 `goods.rs`: the append-only goods table with short codes, stack
      limits and icon paths; `Item::Good`; names. Tests: codes are unique
      words, look-up by code round-trips.
- [ ] 1.2 `terrain.rs`: `Material::Tilled` and `Material::Forge`; `dig.rs`
      classes them; `inventory.rs` names them.
- [ ] 1.3 `farming.rs`: the crop roster, climates, `stage`, tillable tops,
      the seed a tilled cell turns up, the plot record body, settings
      (`farming.ron`). Tests: stages from time, climate refusal, the seed is
      the same for a cell, settings file agrees with defaults.
- [ ] 1.4 `station.rs`: the station table, recipes (`recipes.ron`), jobs
      derived from world time, queue and collect on `Slots`. Tests: two jobs
      run in turn, a refused queue takes nothing, a collect that does not fit
      waits, the shipped recipes validate.
- [ ] 1.5 The ore rule: coal seams by hash, placed ore stays ore. Test.

## 2. Saves

- [ ] 2.1 Material codes 18 and 19, the `g` slot prefix, the `with` line and
      its replay. Tests: round trips; a line from before goods still reads;
      an unknown good costs only its line.
- [ ] 2.2 An old save opens unchanged and is dealt kit v3 once (test).

## 3. Drawing the ground

- [ ] 3.1 Render codes 14 (tilled, fields sheet) and 15 (forge, mountains
      sheet) in `render_code` and `planet_surface.wgsl`; the tile-name test
      covers them; slot icons for both blocks.

## 4. Play

- [ ] 4.1 `E` tills with the shovel (seed drop), harvests a ripe crop, says
      what an unripe one is, opens a forge.
- [ ] 4.2 Right button with a seed plants; placing over a crop is refused;
      digging under one uproots it; a busy forge is not broken.
- [ ] 4.3 Ore and tilled soil drop by the rules (coal, dirt).
- [ ] 4.4 Crops drawn: crossed sprites, field-lit, faded.
- [ ] 4.5 The forge's chimney and flame.
- [ ] 4.6 The forge panel (`Screen::Forge`) in the kit.
- [ ] 4.7 Item icons for the goods, copied from the mockups with provenance.
- [ ] 4.8 Bindings table names the use key's new verbs.

## 5. Proof

- [ ] 5.1 A capture instrument (`--farm`, `--forge`) that tills, plants at
      staggered times and places a forge with a job, for headless shots.
- [ ] 5.2 Shots in `docs/screenshots/farming/` and
      `docs/screenshots/basic-smelting/` with a README of commands; no
      `LATE_PIPELINE` in the log.
- [ ] 5.3 Video for the owner's batch (deferred with the others).
