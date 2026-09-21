# Design: the saves page as the front door

## What exists

`menu::Screen` is `Playing | Pause | Settings | Saves`, and the resource is
inserted at startup from `--menu` (`desktop.rs`, the `launch.menu` match):
`--menu saves` opens the app with the saves page up over the world. `paint`
shows the page for the current screen and hides the others; `press` carries
out LOAD, DELETE and NEW; `load_world` (in `PreUpdate`, after `toggle`)
swaps the open world for the loaded one, forcing a fine-set rebuild. NEW
makes "World N" with no name asked. `open_world` picks the world at boot:
`--world` by name, else the most recently played, else a new "Preview".

## The change

1. **The opening screen is `Saves` for a plain desktop launch.** The match on
   `launch.menu` gains a default: `Saves` when `launch.world.is_none() &&
   launch.capture.is_none()`, `Playing` otherwise. Captures and named worlds
   keep opening straight into the world.
2. **The open world is CONTINUE.** The row for the world that is open
   already reads "(open)" and hides LOAD; it gets a CONTINUE button
   (`MenuAction::Resume`, which already sets `Screen::Playing`), and Escape
   on the saves page steps to `Playing` rather than to `Pause` when the page
   was the opening screen. `Screen::back` takes that from a resource flag,
   `OpenedOnSaves`, set at startup and cleared on the first CONTINUE or
   LOAD, so a later visit through the pause menu steps back to the pause
   menu as it does now.
3. **NEW WORLD takes a name.** A text field on the page (`Text` with a
   `TextInput` marker, keyboard characters appended while it has focus,
   Backspace, Enter to make, Escape to leave the field), prefilled with
   "World N". `saves::create` and `slot_id` already reject nothing they
   cannot make a directory of, so any name is legal and the id is derived
   as now. Only keyboard text while the field has focus; a click elsewhere
   drops focus, and the walker's keys are not read while a menu is open
   already (`MenuOpen`).
4. **No "Preview".** `open_world` with no name and no worlds opens
   `memory_only` and the page says "no worlds yet"; NEW WORLD then makes the
   first one and loads it through the same `LoadRequest` path.

## Tests

- `Screen::back` from `Saves` is `Playing` when the game opened on it and
  `Pause` otherwise.
- The opening screen is `Saves` with no `--world` and no capture, `Playing`
  with either, and whatever `--menu` names when it is given.
- A typed name makes a slot of that name; Enter with the field untouched
  makes "World N".
- A capture opened with `--menu saves` (the existing `menu-saves.png`)
  shows the field and the CONTINUE button; the owner's launch is the check
  that counts.
