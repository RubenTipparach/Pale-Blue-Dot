## Context

The townsfolk's figure (`pbd_core::townsfolk::figure`, drawn by
`pbd-app/src/townsfolk.rs`) is approved for the game (M5's Faces tab, owner
2026-11-21). Since design 24 of `multiplayer`, another player is drawn with
it too. M40 version 2 shows the other player as that figure (owner,
2026-11-27: "humans should be consistent with the NPCs in the game").

## Decisions

1. **One figure, one art.** The player's character is a `look::Look`, the
   same struct a townsperson is drawn from. No second model is made. A
   choice the townsfolk's parts cannot draw is not offered.
2. **The look is the install's, not the world's.** It lives in
   `saves/character.ron` beside `install.key`, so a player is the same person
   in every world and on every host. A world needs nothing new, and old saves
   are untouched. The rolled fallback uses the install's key as its seed, so
   it is stable until the player chooses.
3. **Chosen from M5's palettes.** Skin, hair and cloth colours come from
   the peoples' palettes (`look::People`), across every people. The screen
   shows swatches, not a colour picker, so a character always sits in the
   game's palette.
4. **Seen only from outside.** First person stays hands only. The body is
   drawn in a chase or third-person view and in other players' games. A
   part is in exactly one view, as the rower's arms are today.
5. **Seated poses per craft and mount.** Each craft names a seat (its
   position, facing and posture) and the figure's limb poses. The figure
   reads them; nothing reads the figure.
   - Rowing: the rower's arms follow the oar handles.
   - Cart: the hands are on the shafts or the reins.
   - Horse or plodder: astride, the hands on the reins.
   - Car: seated, the hands on the wheel.
   - Plane: seated in the cockpit, the hands on the stick.
6. **On the wire.** `Hello` carries the look: its parts as small codes and
   its colours as `u32`. It is checked against the parts on arrival, and a
   look the host cannot draw falls back to the rolled one. This is a
   protocol version.

7. **The page in the game (task 3.1).** CHARACTER is the last tab of the
   settings screen, which the title and the pause menu both open (the
   Options menu). It follows M40's Character tab, in the game's UI kit:
   - two columns of rows. Each row is its label, its value and the kit's
     `<` and `>` buttons. A colour's value is its swatch, not a name;
   - the rows are Build, Height, Skin, Hair, Hair colour, Beard, Face,
     then Nose, Ears, Shirt, Trousers, Shoes and Hat colour;
   - a row steps through what `pbd_core::townsfolk::character::step` allows:
     the M5 parts, and the colours of every people's palette. A value the
     figure cannot draw (long hair with a beard) is stepped past, and a
     build clamps the height into its own range. So every look the page
     makes is `valid`, and a test steps every row all the way round;
   - RANDOM rolls a fresh look;
   - each change is written to `character.ron` and sent to the host at once
     (design 6). There is no Apply.
   - **The preview is the figure itself, in the world.** While the page is
     open, the player's own figure stands about 3 m in front of the camera
     and to the right of the panel, facing the camera. It is drawn by the
     townsfolk's code, as another player is, so it is lit as the ground there
     is: by the sun, the cell light field and the lamps at night. A second
     camera, render target and light just for the preview would be lit
     differently from the world, and that is the look the owner turned down.
     With no world behind the menu (the title before a world is loaded),
     the page shows the rows alone.
   - Offered once on a new world: the first time a world is entered with no
     `character.ron`, the page opens before play. Closing it keeps the
     rolled look, and writes it, so it is not offered again.
   - The mockup's hat kinds (hood, headset and the others) are not offered:
     the townsfolk's parts have one hat (decision 1). Hair has HAT as one of
     its styles, and the hat's colour is its own row.

8. **Seats in the game (tasks 4.1 and 4.2).** M40's Character tab poses
   the figure by aiming each arm, from its shoulder, straight at where its
   hand goes, and by turning the legs and leaning the body. The game does the
   same with the townsfolk's own figure:
   - the townsfolk get a posture `Ride`: the figure's hips on a seat, its
     legs turned forward by the seat's angles and spread apart (astride), its
     body leaned forward, and each hand aimed at a point in the figure's own
     space. The pose gains a lean, a leg spread and two optional aimed hands.
     A figure with none of them is posed exactly as today;
   - the player's own figure is a visitor of [`PLAYERS_SITE`], as the
     preview is. It stands while the player is in a craft's seat and the view
     is from outside, and goes when the view is first person, so first person
     stays hands only (decision 4);
   - each craft names its seat in its own space: the hip point, the facing,
     the legs' angles and spread, the lean, and where the hands go. The
     rower's hands go to the oar grips, which move with the stroke. The cart
     and the horse's hands go to the shafts or the reins, the car's to the
     wheel's rim, the plane's to the stick;
   - the rower's boxes in `vehicles/draw.rs` are no longer drawn. The
     first-person hands on the oars stay as they are.
   The seat values are the mockup's (`character.js`, `POSE`).
   - **Seen by the others (protocol 3).** A pose names its seat: standing,
     at a raft's oars, on a cart's bench or astride a mount. Seated, its
     place is the hips: the raft's seat facing the bow, or under the rider's
     eye on a bench or a mount. The other games draw the player sitting
     there in that seat's pose, the hands held at rest on the controls, since
     the live oar strokes are not sent. The craft or animal is drawn by its
     own record, as today.
   - **What waits.** Carts and mounts have no view from outside in the
     player's own game yet, so there the seat is seen only by the others.
     The car and the plane are not in the game (phase 5); their seats come
     with them, from the same mockup values.

## Open questions (recommendations taken, ask only with screenshots)

- Where the screen opens on a new world: before the world loads
  (recommended), or on first spawn.
- Whether a hat is worn in a vehicle: yes, unless it is a helmet that a
  cockpit would clip (recommended).

These go to the survey with the mockup's screenshots.
