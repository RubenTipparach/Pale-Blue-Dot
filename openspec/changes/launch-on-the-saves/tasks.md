# Tasks

## 1. The front door
- [x] The opening screen is `Saves` for a plain desktop launch; `--world`,
      a capture and `--menu` keep their own (`menu::opening_screen`, tested).
- [x] `FrontDoor`: Escape and CONTINUE on the opening page go to the world,
      and nowhere when none is open; a later visit steps back to the pause
      menu (`Screen::back`, tested).
- [x] No "Preview": a first run opens a memory-only world and the page
      offers NEW WORLD.

## 2. A name for a new world
- [x] An in-game text field on the page (`NameField`, `name_input`),
      prefilled "World N"; Enter makes it, Escape leaves the field, a click
      elsewhere drops focus. The typing rules are tested.
- [x] The new world is loaded through `LoadRequest` on creation.

## 3. Prove it
- [x] `--menu saves` capture with the field (`docs/screenshots/menu-saves-front.png`).
- [ ] The owner's own launch: the page first, a new world made by name, a
      delete, a load, CONTINUE.
