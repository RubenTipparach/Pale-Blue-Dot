# Proposal: ten slots, real item thumbnails, and a HUD that is not a tech demo

## Why

**The owner asked for a cleaned-up UI and ten slots for blocks and equipment.**
The screen today is a survey-flight readout: a title block, a two-line keybinding
list, and a line of flavour text, all floating `Text` nodes with no panel behind
them and nothing a player can act on. There is no inventory anywhere in the
workspace - `grep` for it returns nothing - so a slot is not a reworked widget,
it is a thing that does not exist.

The owner's three complaints, taken as given: the keybinding list is clutter,
the framing reads as a tech demo, and it needs a real game HUD.

## What this is, and what it is NOT

This is the first of two changes. It is the half that does not depend on the
terrain representation:

- **Ten slots** that hold blocks and equipment, selected with `1`-`9`, `0` and
  the wheel.
- **A drawn HUD**: the slot row on a panel, item thumbnails in the slots, and
  the flight and walk readouts kept but tidied.
- **The framing out**: the title, the survey-flight subtitle, the flavour line
  and the keybinding wall.

**Digging and placing are NOT in it**, and that is a fact about the terrain
rather than a scoping preference. `SurfaceContact` carries one `radius` per
direction, so the world is a heightfield: it has no interior, cannot express a
ceiling, and has no block to remove. Caves, overhangs and mining all wait on the
volumetric columns in the companion change, and the owner chose to take the
quick wins first knowing that.

So the slots ship **with blocks in them and nothing to place them with yet**.
That is stated on the screen rather than hidden: a slot holds an item and says
what it is, and the verb arrives with the columns.

**No equipment item ships.** The slots are general and hold one, because that is
what "slots for blocks and equipment" asks for, but a pickaxe that cannot dig is
a control for a mechanic that does not exist. The tool variant exists in the
type; the first real tool lands with digging.

## Item thumbnails cost no new art, which is the nice part

The repo's inherited UI rule is that a panel showing items draws **a grid of
slots with a real icon per item, never a list of names**. That usually means an
icon atlas nobody has drawn yet.

Here it is free. The terrain shader already samples `tilesets/fields.png` as a
**4x4 grid** and already maps every material to a tile in it - grass at (0,0),
stone and snow at (3,0), sand and dirt at (3,2), wood at (2,1), leaves at (0,2).
A slot's thumbnail is a UV rect into that same image, so a block's icon is
literally the texture the ground is drawn with, at 313 px a tile. This is the
same trick the grass blades already use when they sample the ground tile they
stand on: **the art exists, it just had not been asked for twice.**

## What the HUD becomes

A bottom bar rather than scattered text:

- **Ten slots**, centred, each a bordered square with its thumbnail, a stack
  count when it is above one, and the selected one lifted and outlined.
- **The readouts kept**: altitude, speed and mode still matter; they move onto
  the bar's left and stop floating over the terrain.
- **The keybinding wall replaced by one line** naming only what a player needs
  right now, with the full list behind a key.
- **The title, subtitle and flavour text deleted.**

The existing top and bottom scrims stay: they are what keeps small text legible
over snow and sea glint, and the owner's complaint was about the content on
them rather than the letterboxing.

## Non-goals

- Digging, placing, and anything that needs a block to exist. The companion
  change.
- A bag or a crafting screen. Ten slots is what was asked for; a second store
  is the inherited rule's "hotbar and bag are one inventory split in two", and
  it lands when there is more to hold than ten stacks.
- Health, hunger or any status bar for a mechanic that does not exist.
