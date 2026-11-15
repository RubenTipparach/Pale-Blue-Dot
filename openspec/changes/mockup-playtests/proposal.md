# Proposal: automated and recorded playtests for every mockup

## Why

The owner, 2026-11-14: "in adiition come up with automated + recorded video
playtest plans to verify each mockups milestones".

Twenty-five mockups are on the tracker. Each is checked today by hand: a
scratch Playwright script, a handful of stills, and a written claim that the
page has no console errors. The scripts live in a session scratchpad and are
lost. Nothing re-checks a page after a later edit, and nothing checks the
standing mockup rules (lit at night, nothing spins on its own, one 3D view at
a time) on every page.

`step-videos` plans how the game records its gates and gives mockups one
tool, `tools/mockup_video.js <page> <walk.json> <out.webm>`. It plans no
assertions, and it covers only the towns mockup.

## What changes

- **One playtest file per mockup**, `tools/mockups/playtests/<page>.json`,
  committed. It lists the mockup's milestones. Each milestone has steps (open a
  tab, click, drag, press a key, run a page hook, wait), assertions (a check on
  page state or on pixels that must hold), and shots (a caption, the
  requirement it shows, and its length).
- **One runner**, `tools/mockup_playtest.js <page> [--milestone M] [--record]`.
  - Without `--record`, it plays the steps headless, checks every assertion,
    and fails with the milestone and assertion named.
  - With `--record`, it plays the same steps with Playwright's video on, and
    writes one WebM per milestone plus a contact sheet (one frame per shot,
    captioned).
  - It is `step-videos` 3.2's `mockup_video.js`, grown: a walk file is a
    playtest with no assertions.
- **The standing rules are checked on every page**, without a line in any
  playtest file: no page errors, a tracker link near the top, one WebGL
  context drawing, idle stillness, lamps at night.
- **A milestone plan for every mockup, M1 to M25** (design section 4). This
  is the plan; building the files is tasks section 3.
- **A playtest gate page**: per mockup, its milestones with pass or fail,
  the video and contact sheet, and a link to the live mockup. Built by
  `step-videos`' `tools/gate_page.py`, so there is one gate page format.

## What does not change

- The game's showcases and `--record` stay in `step-videos`.
- The owner's batch rule stands: videos wait to be watched together, and
  screenshots stand in until then (CLAUDE.md, "For now, screenshots stand in").
  Playtests make that batch cheap: one command records them all.
- Approvals stay the owner's, on the tracker. A passing playtest is evidence
  for a gate, never the gate.

## Impact

- New: `tools/mockup_playtest.js`, `tools/mockups/playtests/*.json`, and a
  `window.__test` hook on pages that lack one.
- Hooks pages already have are reused, not replaced: `__build`, `__town`,
  `__npcs`, `__sail`, `__hangar`, `MG`, `__pbd`, `setLoad`, and the classic
  script globals.
- Hardware: the cloud has no GPU, so cloud runs render with SwiftShader and
  every video says so. Frame timing is never asserted in the cloud.
