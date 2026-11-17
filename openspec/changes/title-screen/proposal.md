# Proposal: a title screen with the logo

## Why

The owner, 2026-11-17: "we should also have a proper 'game menu' with the main
game logo ... the main logo can be in the main menu". A plain launch opens on
the saves page (`launch-on-the-saves`): a list of worlds with no title, no
logo and no way to settings before a world.

## What Changes

- A new front door, `Screen::Title`: the logo (`assets/ui/logo.png`) large over
  the world standing behind it, and an oak panel under it with CONTINUE, NEW
  WORLD, LOAD WORLD, SETTINGS and QUIT.
- CONTINUE plays the world already open behind the menu. NEW WORLD and LOAD
  WORLD open the saves page (its name field focused for a new one), and its
  Back returns to the title. SETTINGS opens settings, whose Back returns to the
  title while the title is the front door. QUIT quits.
- The mockup is M29's Main menu tab (`docs/mockups/ui.html#title`).

## Impact

- `crates/pbd-app/src/desktop/menu.rs` (the screen, its panel, its back
  steps, its actions), `desktop.rs` (the front door), tests.
- No save, no generation and no gameplay change; nothing new is drawn after
  `READY` (the logo is a UI image, the kit's pipeline).
