# Proposal: the game opens on the saves

## Why

**The owner's words: "Where are the save slots when I load up the game?
Otherwise no way to create/delete saves or even pick what to load."**

They are behind Escape: the pause menu's SAVES page lists every world newest
played first, with LOAD, DELETE (asked twice) and NEW WORLD, and it is
reached only from inside a world. A launch with no `--world` opens the most
recently played world straight away, or makes one called "Preview" on a first
run. So the first thing a player sees is a world they did not choose, and the
way to choose one is a key nothing on the screen names.

The screen is built and works; what is missing is that it is the FRONT door
rather than a side door. The capture harness already knows how to open on it
(`--menu saves` starts the app with that page up), which is most of the
change.

## What

- **A desktop launch opens on the saves page** when no `--world` was named
  and no capture is being taken. The most recently played world is still
  opened behind it, because the planet is built from a world's edits before
  the first frame and a page has to sit over something; its row reads
  "(open)" and its LOAD is a CONTINUE. Escape on that page, with a world
  already open, continues into it, which is what a returning player does
  nine times in ten and what pressing nothing should mean.
- **NEW WORLD asks for a name**, in an in-game text field on the page, with
  the number it would have chosen already in the box so Enter alone still
  works. Tenebris's rule, kept: every text input is a drawn box in the
  game's own UI, never a native prompt.
- **A first run has no worlds** and the page says so, with NEW WORLD the
  only button. Nothing is made called "Preview" behind the player's back.
- **`--world <name>` and captures skip the page**, as they do now: a harness
  that says which world it wants does not want a menu.

## What does not change

The pause menu, the settings page, the SAVES page's rows and the delete
confirmation, the load path (`LoadRequest`, `load_world`), the save format,
and the one-table binding list. The page is the same page; it is shown first.
