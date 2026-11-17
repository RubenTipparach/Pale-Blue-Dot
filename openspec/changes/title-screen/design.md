# Design

## Measured

- `menu::opening_screen` opens a plain desktop launch on `Screen::Saves`;
  `FrontDoor` marks that page as the front door, and `Screen::back` steps from
  it into the open world (or nowhere if none is open).
- A world is always open behind the menu: `open_world` opens the last played
  slot (or a memory-only world on a first run) and the loading screen builds
  it before `READY`. Loading another world goes through `load_world` and
  `loading::begin_switch` in place, with the loading screen up again.

## Decisions

1. The title is the front door. `opening_screen` returns `Screen::Title`
   where it returned `Saves`; `--menu title` opens it for captures.
2. The panel: the logo 300 UI units wide at its own aspect, 10 units from the top
   of the screen (owner, 2026-11-17: "this logo should be closer to top of the
   screen"), then, centred in the space under it, an oak frame
   120 units wide with CONTINUE (focused), NEW WORLD, LOAD WORLD, SETTINGS,
   QUIT. No lintel text: the logo is the title.
3. Back steps: Esc on the title stays on the title. Saves and Settings step
   back to the title while it is the front door; once a world is entered
   they step back to the pause menu as before.
4. The logo is decoded at spawn like the loading screen's emblem (`image`, a
   linear-sampled `Image`); it is a UI image, so it adds no pipeline.
5. The pause menu's QUIT still quits (a "quit to title" is a behaviour change
   for the owner to ask for).
