# World: crops

## Purpose

The crops a kitchen needs, joining the farming mockup's roster, and the
perennial rule. Proposed with the hearth and kitchen mockup; nothing here is
built.

## ADDED Requirements

### Requirement: New crops are appended seeds

Tomato, durum wheat, soy, bok choy, onion, garlic, sugar cane, sugar beet,
herbs, potato, olive and tea SHALL be appended to the crop roster, never
inserted, so a planted crop saved by its index keeps its species. They SHALL
come only as seed the player plants; the generated ground SHALL NOT change,
so no terrain generator version changes.

#### Scenario: Appending keeps saved fields

- **WHEN** a world with planted wheat and rice is opened by a build that adds
  the twelve crops
- **THEN** the same cells hold the same wheat and rice at the same stage

#### Scenario: No new ground

- **WHEN** the ground digest of a world is taken before and after the crops
  are added
- **THEN** the digests match

### Requirement: Crops grow by climate and season outdoors

Each crop SHALL grow outdoors only in its climates and seasons, SHALL need
water each day as the farming rules say, and SHALL grow anywhere inside a
greenhouse.

#### Scenario: Sugar by climate

- **WHEN** sugar cane is planted outdoors on tundra
- **THEN** it does not grow, and sugar beet planted beside it does

### Requirement: Perennials stay after harvest

An olive tree and a tea bush SHALL stay on their cell after harvest and
ripen again (olives every 6 days, tea every 3) without replanting. Frost
SHALL kill a young olive and SHALL NOT kill a grown one.

#### Scenario: Picking tea

- **WHEN** a ripe tea bush is harvested
- **THEN** it stays, and is ripe again three growing days later
