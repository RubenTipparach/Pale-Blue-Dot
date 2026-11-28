## Why

The owner, 2026-11-27: "Would be nice to be able to customize character for
single player too. This would allow me to have an in game model dude for
third person vehicle views like the boat rowing, car driving plane flying
stuff and horse riding." The same day: "humans should be consistent with the
NPCs in the game".

Today the player has no body of their own. In first person only the hands
show. In the chase views of craft the rower is a stand-in, and the cart,
horse, car and plane show nobody at the controls. In multiplayer another
player is drawn as a townsperson whose look is rolled from their player
number (`multiplayer` design 24), so they cannot choose how they look.

## What Changes

- **A character of the player's own**, built from the townsfolk's own figure
  (M5, `pbd_core::townsfolk::look::Look` and `figure`), so the player and the
  NPCs are one art. The player chooses:
  - build and height;
  - skin, hair style and colour, and beard;
  - shirt, trousers and shoes;
  - a hat or none;
  - the face they wear at rest.
  Each choice is from M5's palettes and parts.
- **Where it is made:** a Character page in the Options menu, open at
  any time, from the title or the pause menu (owner, 2026-11-27: "Fully
  customizable in options menu"; "Character fully customizable for
  multiplayer"). A new world offers it once before it loads. It shows the figure turning only when
  dragged, by day and lit at night, with a Random button that rolls a look
  as `look::generate` does.
- **Where it is kept:** one look per install, beside the saves
  (`character.ron`), so every world and every host sees the same person. A
  world written before this change opens with a look rolled from the
  install's key until the player makes one. Nothing in a save changes.
- **Where it is seen:**
  - the third-person views of every craft and mount: rowing (the rower
    replaced), the hand cart and horse cart, riding a horse or a plodder,
    the car, and the plane's cockpit;
  - in multiplayer, by everyone else: `Hello` carries the look, and design
    24's visitors are drawn with it in place of the rolled one.
- **Where it is not seen:** first person stays hands only.

## Capabilities

### New Capabilities
- `player-character`: the player's own look, its screen, where it is kept,
  and where it is drawn.

### Modified Capabilities
- None yet. `multiplayer`'s `Hello` and design 24 change when this is built.

## Impact

- `pbd-core`: a `Look` encoding (RON for the file, bytes for the wire) and
  its validation against M5's parts and palettes.
- `pbd-app`: the Character screen (the UI kit, every word from the locale
  file), the seat poses for each craft and mount, and the rower's stand-in
  replaced.
- `multiplayer` protocol: `Hello` gains the look; this needs a protocol
  version.
- A mockup first: M40 gains a Character tab, the screen and the figure
  seated in a boat, a cart, on a horse, in a car and in a plane, by day and
  at night. No game code until the owner approves it.
