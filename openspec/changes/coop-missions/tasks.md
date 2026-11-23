## 1. Before code

- [ ] 1.1 Mockup of the missions panel's CREW tab (decision 7) in `docs/mockups/`, lit at night, its text in a modal, on the tracker; the two open questions in the survey beside its screenshots. Verify: the owner approves it for build.

## 2. The crew record and counting (after `multiplayer` 8.1, 8.2 and the piece writer)

- [ ] 2.1 The `crew_missions` world record and its per-author counts (decisions 1, 2), counted by the server from accepted changes through `missions::count`. Verify: a test where two players' digs count into one crew objective with each share right, and a reload keeps it.
- [ ] 2.2 `hands` and `each` (decision 3), with `grace_s` in validated config. Verify: a test that one player alone cannot meet a `hands: 2` objective and two can; a test that `each` waits for the late joiner's grace.

## 3. The house chain

- [ ] 3.1 H1 to H4, H6, H7 in `missions.ron`'s `crew` section, and the co-owned pieces on H7 (decision 5). Verify: a server test of two players building it from requests.
- [ ] 3.2 The `beam` piece and its rule in `building::rules`. Verify: a test that a beam needs a post at each end.
- [ ] 3.3 `Together` as a held transaction (decision 4), `together_s` in config, skipped alone. Verify: a test that two requests within 3 s settle as one change with two authors, one alone is refused as `Alone`, and single player meets it.

## 4. Played and seen

- [ ] 4.1 `pbd-headless` bots play the house chain (decision 8). Verify: a soak of two and of four bots finishes the chain with every digest matching.
- [ ] 4.2 The CREW tab in the game, from the approved mockup. Verify: screenshots of two players' panels mid-chain.
- [ ] 4.3 The gameplay demo (`multiplayer` 9.2): two bots, "build a house together and light its hearth". Verify: the owner watches it.
