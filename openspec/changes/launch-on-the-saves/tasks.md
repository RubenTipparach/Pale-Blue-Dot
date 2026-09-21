# Tasks

## 1. The front door
- [ ] The opening screen is `Saves` for a plain desktop launch; `--world`,
      a capture and `--menu` keep their own. Test on the launch parse.
- [ ] `OpenedOnSaves`: Escape and CONTINUE on the opening page go to the
      world; a later visit steps back to the pause menu. Test on `back`.
- [ ] No "Preview": a first run opens nothing and the page offers NEW WORLD.

## 2. A name for a new world
- [ ] An in-game text field on the page, prefilled "World N"; Enter makes
      it, Escape leaves the field, a click elsewhere drops focus.
- [ ] The new world is loaded through `LoadRequest` on creation.

## 3. Prove it
- [ ] `--menu saves` capture with the field and CONTINUE; the owner's own
      launch, a new world made by name, a delete, a load.
