# Design: who owns the pointer, and where the words go

## The one thing this change is really about

A menu is not a panel. A panel is easy - it is a `Node` with children. What a
menu IS, is a second claimant on the mouse and the keyboard, and everything
hard here is that claim.

Today the world takes the pointer and never lets go of the question: the
walker reads `Escape` and drops it, a left click takes it back, and
`dig_and_place` reads the raw button with no test at all. Drop a panel with
buttons into that and every one of those three does the wrong thing - the
button press re-grabs the mouse, the shovel swings through the button, and
`Escape` is read twice on the frame it is pressed.

So the rule, written once: **`MenuOpen` says whether a menu holds the pointer,
and while it is set the world does not listen.**

```rust
// crates/pbd-app/src/controls.rs
#[derive(Resource, Default)]
pub struct MenuOpen(pub bool);

/// What an input reader does with its capture flag across a menu.
pub fn hand_over_pointer(open: bool, was_open: &mut bool, captured: &mut bool);
```

Three readers, one rule:

| reader | what the gate does to it |
| --- | --- |
| `walking::read_walking_input` | no look, no movement, no click-to-capture |
| `flight_view::input::read_pilot_input` | the same, for the pilot |
| `desktop::digging::dig_and_place` | no dig and no place unless the pointer is captured |

The third is gated on CAPTURE rather than on the menu, and that is the more
general rule of the two: the pointer is also free when the window loses focus,
and a shovel that swings when you click back into the window is the same bug
wearing a different hat.

`hand_over_pointer` exists because the interesting half is the EDGE, not the
state. Holding is obvious. Releasing has to hand the pointer BACK, or Escape
to resume leaves the player looking at a free cursor and needing a click to
play - and that click, before the digging gate, would have dug. Each reader
keeps its own `Local<bool>` memory of the last answer, because Bevy gives a
system no other place to keep one, and the rule that reads it is written once
rather than twice.

**Why not have the menu write `captured` directly?** It can reach
`WalkingState.captured`, which is public, and it cannot reach
`FlightInputState.captured`, which is not. Writing one and gating the other
is exactly the divergent path that gets one of them wrong later.

## Escape belongs to the menu, and it is taken rather than shared

`menu::toggle` runs in `PreUpdate` after `InputSystems`, which is before
`RunFixedMainLoop` where both input readers run, and it calls
`clear_just_pressed(KeyCode::Escape)` on the frame it acts. So downstream
never sees the key at all, instead of every reader having to know which
screens are open. The `Escape` arms in the walker and the pilot are deleted
rather than gated - a key with one owner needs no agreement.

The step is one screen at a time (settings to pause, pause to the world),
which is the behaviour every player already expects and the reason the screen
is one enum rather than two booleans: two flags is how a settings page ends up
open over a closed pause menu.

## What a menu is made of

One panel per screen, both built at startup and hidden, shown by setting
`display`. `Display::None` and never `Visibility::Hidden`, which the slot row
already learned: a hidden node is still laid out and still picked, so an
invisible panel goes on swallowing clicks over the middle of the screen.

A row is a `Button` carrying its own `MenuAction`, so what a button DOES is
data on the entity rather than a position in a list that a reorder would
silently change. `Interaction` drives the fill, which is what makes a button
look like one.

The buttons are the owner's three and nothing else. QUIT writes `AppExit`,
which is already imported for the capture path's own exit.

## The binding table

`pbd_app::controls::BINDINGS` is a slice of groups, each a heading and its
rows, and a row is a `KeyCode` list plus what it does:

```rust
pub struct Binding { pub keys: &'static [KeyCode], pub does: &'static str }
pub struct Group { pub heading: &'static str, pub rows: &'static [Binding] }
```

`KeyCode` and not a string, for the one reason that makes the test possible:
a `KeyCode` has a name a test can look for in the source of the system that
presses it. `a_named_key_is_a_key_something_reads` walks every row, formats
its keys back to `KeyCode::Space` and friends, and asserts each appears in
`walking.rs`, `flight_view/input.rs`, `slots.rs` or `digging.rs` - the four
files that read the keyboard. A mouse button is its own variant and checked
the same way.

That is the weakest test that catches the real failure, which is the one that
already happened: a list that names a key nothing reads, or misses a key
something does. It does NOT prove the table's prose is right, and it cannot -
`F` genuinely swapping to fly is a fact about the code no grep establishes.
What it does establish is that the list is not stale about its keys, and
staleness is what put a `H  keys` line on screen for a panel this change
deletes and left dig and place off both lists for a release.

`--help` prints the same table through `controls::help_text()`, so the third
copy goes at the same time as the second.

## What is left on the screen

The crosshair and the hotbar. Both are controls: the crosshair says where a
dig lands and the row says what is in your hand. `hud.rs` keeps its `setup`,
loses its `update` entirely, and with it the last reader of `FlightReadout`,
`WalkingReadout` and `TourProgress` outside the systems that publish them.
`FrameStats` stays because `capture` reports the frame percentiles at the end
of a run, which is a measurement rather than a HUD, and is where a frame time
belonged all along.

## Held, and why

**The simulation does not stop.** A pause that halts the walker's input but
not the world is what this ships, and the honest reason is the size of the
alternative: Avian's `PhysicsSchedule`, the orbit clock on `Time`, the LOD
streaming tasks and the weather all advance independently, and stopping some
of them is a worse picture than stopping none (a frozen player under a moving
sun). The next change gates the movers behind one run condition, the way the
reference project does, and the gate it reads is `MenuOpen`, which this change
already establishes.

**Rebinding is not here.** The keys live as `KeyCode` constants inside the
systems that read them. Making the table authoritative means the systems read
IT, which is a real refactor with a named scar behind it in the reference:
Tenebris kept the scheme in a compiled default and in a shipped YAML that
overrode it, remapped one and not the other, and the game ran the old scheme
while every test stayed green. Doing this properly means one authority and a
test that loads the shipped artifact. It is worth doing and it is not this.
