# Interface look: rock and stone in play, carved oak out of play

Section 1 of `tasks.md` is built (2026-11-14). The requirements it made true
moved to `openspec/specs/interface-look/spec.md` with the tests that pin them:
the UI scale from the window height, the smooth kit at the whole-number scale
(which replaces this delta's first "pixel art sampled nearest" requirement:
the owner chose Smooth, sampled linear, for the kit), 9-slice panels with
margins from `slices.json`, a rock frame that never repeats, rock in play and
oak out of play, the kit's pixel font for labels, one palette module, and
legible text. What follows is not built yet.

## ADDED Requirements

### Requirement: Key bindings are grouped by context

The settings' Input page SHALL group the bindings into tabs by context (on foot,
hands, flying, vehicles and ships, menus and view), one row per action with its
key and a way to change it, a key already used in the same tab flagged on the
row, and a reset for the tab. The owner, 2026-11-14: "organize the input menu
better with tabs or something instead of big uber list".

#### Scenario: Rebinding a key

- **WHEN** the player picks Change on "Dig / use tool" and presses E
- **THEN** the row shows E and notes that E is also Interact, and Reset tab puts
  the tab's keys back
