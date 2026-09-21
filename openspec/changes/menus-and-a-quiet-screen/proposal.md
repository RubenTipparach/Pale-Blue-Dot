# Proposal: a quiet screen, and menus to put the words in

## Why

**The owner's words: "Clean up UI, remove black bars and excess text. All text
is excess on the UI currently. Inputs should be displayed in the settings menu
under input. Pause menu should have resume, settings, and quit."**

What is on screen today, measured off `hud-before.png` at the default spawn:

| | |
| --- | ---: |
| Dark scrim across the top | 94 px, full width |
| Dark scrim across the bottom | 148 px, full width |
| Screen area under a scrim | **26.9%** of 1440x900 |
| Blocks of floating text | 4 |

The four are a mode-and-FPS line top right, a two-line speed/altitude/latitude
block bottom left, a one-line binding strip above the hotbar, and the full
binding wall behind `H`. None of them is a control. All of them are a readout
for whoever was building the engine, which is what "reads as a tech demo"
meant the first time the owner said it, and the hotbar change answered that
complaint by tidying the readouts rather than deleting them.

**The scrims have no argument left.** They were kept on purpose - the
`hotbar-and-game-hud` proposal says so in as many words: "The existing top and
bottom scrims stay: they are what keeps small text legible over snow and sea
glint, and the owner's complaint was about the content on them rather than the
letterboxing." That is a true reason and it is entirely parasitic on the text.
Take the text away and a scrim is 27% of the window dimmed to make nothing
readable. This change reverses that decision, and the reason it can be
reversed without argument is that its premise is being deleted in the same
commit.

## What

**Off the screen.** Both scrims, the mode-and-FPS line, the speed/altitude
block, the binding strip, and the `H` panel. What is left of the HUD is the
crosshair and the hotbar: one is where a dig lands, the other is what a dig
puts in your hand, and both are controls rather than commentary. The stack
count in a slot's corner stays - that is the inherited slot-grid rule's own
"border, item icon centred, stack count in the corner", and a number that says
how many you are holding is part of the control, not a caption on it.

**Escape opens a pause menu**, with RESUME, SETTINGS and QUIT, drawn as real
hit-testable buttons on a panel rather than as a list of lines. SETTINGS opens
one page with one section, INPUT, holding the binding table the `H` wall used
to carry, laid out as a key column and an action column. Escape steps back one
screen at a time: settings to pause, pause to the world.

**One binding table.** The wall behind `H` was one of three copies of what the
keys do: the `--help` text is another and the context strip was a third. They
had already gone stale in the way three copies always do - **neither the panel
nor `--help` names the dig or the place**, which shipped in the change
immediately before this one, and a player reading either list would conclude
the shovel does not exist. `pbd_app::controls::BINDINGS` is the one source
now; the settings page and `--help` both read it, and a test reads the input
systems' own source and asserts that every key the table names is a key
something actually presses. That is this repository's rule for a second
representation it cannot collapse: check the real artifact, never a second
in-code copy.

## Two bugs this closes, both about who owns the pointer

Neither was reported. Both are in the way of a menu and would have shipped as
"the menu does not work".

- **A click that frees nothing digs a block.** `dig_and_place` reads the raw
  mouse button with no test for whether the game has the pointer, and Bevy's
  UI does not consume the raw mouse. So clicking RESUME would swing the shovel
  through the button at whatever is behind it. A dig now requires the walker's
  pointer to be captured, which is also the honest rule for the case that
  already existed: Escape frees the cursor today, and the click that takes it
  back digs.
- **Escape had two owners.** The walker and the pilot each read it to drop the
  pointer. The menu needs it, so it takes it: `MenuOpen` is the one resource
  that says a menu holds the pointer, both readers stand down while it is set,
  and the frame it clears they take the pointer back, so Escape-RESUME-look is
  one motion rather than three.

## What this is NOT

**It does not pause the simulation**, and the name is the owner's. The world
keeps turning behind the panel: the sun moves, the planet spins, and a player
who opens the menu mid-fall goes on falling. What the menu does stop is INPUT
- the walker reads nothing, so you do not walk off a cliff while reading the
bindings. Freezing the simulation means gating Avian's schedule, the orbit
clock and the LOD streaming together, and half a pause (physics stopped, sun
moving) is worse than none. It is named in the design as the next step.

**Settings has one section, and no stubs.** INPUT is what was asked for and
the only thing there is anything to show. A GRAPHICS tab over a page that says
"(soon)" is a control for a mechanic that does not exist, which is the rule
this repository inherited twice over. The section list is data, so the second
one is a row.

**No rebinding.** The page SHOWS what the keys do. Making them editable needs
a binding registry the input systems read at runtime instead of the `KeyCode`
constants they carry today, which is a real change with a real scar behind it
in the reference: Tenebris kept its scheme in a compiled default AND a shipped
YAML, and the game ran one while the tests checked the other. Doing it
properly is its own change.
