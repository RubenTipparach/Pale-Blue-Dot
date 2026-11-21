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
   screen"), then right under it ("move the menu text up too") an oak frame
   120 units wide with CONTINUE (focused), NEW WORLD, LOAD WORLD, SETTINGS,
   QUIT. No lintel text: the logo is the title.
3. Back steps: Esc on the title stays on the title. Saves and Settings step
   back to the title while it is the front door; once a world is entered
   they step back to the pause menu as before.
4. The logo is decoded at spawn like the loading screen's emblem (`image`, a
   linear-sampled `Image`); it is a UI image, so it adds no pipeline.
5. The pause menu's QUIT still quits (a "quit to title" is a behaviour change
   for the owner to ask for).
6. The title hides the in-play HUD: the hotbar, the hand cell and the
   crosshair (owner, 2026-11-17: "why would you have hot bar in title
   screen?"). They carry `InPlayHud`; `hide_hud_on_title` hides them while the
   title is up.

## The splash, and back to the main menu (owner, 2026-11-20)

The owner: "also need a way to quit to main menu from game world, and also
need some cool splash screen for menu instead of loading into a world", and
"thats top priority".

- **The splash.** The title no longer stands over the open world. It stands
  on deep space: the store cover's field, nebula and stars without the logo,
  at 1920 x 1080 (`assets/ui/splash-space.jpg`, made by
  `tools/art/logo/splash_space.py` from `logo_cover.py`'s own field and seed),
  stretched to the window, opaque. The logo and the oak panel sit on it as
  before. The HUD is already hidden on the title.
- **MAIN MENU in the pause menu**, between SETTINGS and QUIT. It opens the
  title as the front door again (`FrontDoor(true)`), so its SETTINGS and LOAD
  WORLD step back to it, and CONTINUE returns to the world. Nothing is saved
  on the way: every change is already on the durable path as it is made.
- **What is not changed yet.** The world still loads behind the splash at
  launch, so the loading screen comes first and then the splash. Launching to
  the splash with no world loaded at all, and loading one only on CONTINUE or
  LOAD, is a larger change to how a run opens its world (`open_world`,
  `drive_loading`), written up as the next step and not built here.
