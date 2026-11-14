# Tasks

## 0. Plan

- [x] 0.1 Proposal, design (with a milestone plan for M1 to M25) and spec delta. Verify: `openspec validate --all`.

## 1. The runner

- [ ] 1.1 `tools/mockup_playtest.js`, check mode: it serves `docs/mockups`, plays the steps, checks assertions, and exits non-zero naming the milestone and assertion. Verify: a playtest with one false assertion fails and names it; the same with it true passes.
- [ ] 1.2 The standing checks (design section 3): errors, the tracker link, one WebGL context drawing per tab, renderers made on first open, tab anchors, idle stillness with `moving` masks, night lamps, Run off by default. Verify: each check fails on a deliberately broken copy of a page in `tools/mockups/playtests/fixtures/`.
- [ ] 1.3 Record mode: a WebM per milestone, the contact sheet with captions and FAILED marks, `playtest.json`, and the SwiftShader label in the cloud. Verify: M20's file records, and its contact sheet shows every shot.
- [ ] 1.4 `--all`. Verify: every committed playtest runs in one command, and a summary prints.

## 2. Gate pages

- [ ] 2.1 `tools/gate_page.py` (`step-videos` 3.3) reads `playtest.json`. Verify: M20's gate page is built and published, listed on the tracker under Misc.

## 3. The playtests

- [ ] 3.1 M20 and M21 first (their scratch scripts exist). Verify: both pass in check mode.
- [ ] 3.2 M15, M16 and M24, folding in their existing scripts, which are then removed. Verify: every check the old script made is an assertion in the new file.
- [ ] 3.3 `window.__test` hooks for `world-map.html` and `held-tools.html`. Verify: their playtests read state through the hook only.
- [ ] 3.4 The rest, M1 to M14, M17 to M19 and M22 to M25, from design section 4. Verify: each passes, or its failure is a mockup bug written up on the tracker.

## 4. The owner's batch

- [ ] 4.1 Record every playtest with `--all --record` and publish the gate pages for the owner's batch review (CLAUDE.md: screenshots stand in until then). Verify: the owner has the links; their verdict is recorded in `proposal.md`, then sync `platform/mockup-playtests` and archive.
