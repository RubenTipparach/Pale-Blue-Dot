# Time speed and skipping ahead

## Why

The owner, 2026-11-26: "can you add buttons to help speed up time
advancement in game? this would help me test stuf easier like farming and
such". The pause panel sets the hour (the TIME bar, DAY, NIGHT, L), but a
crop, an egg, a smelt or a meal waits on the world clock running, and at its
own pace a game day is 48 real minutes.

## What changes

- **All of it behind a DEBUG tab** on the settings panel, its TIME
  CONTROLS box off by default and kept in the settings file (owner, the
  same day: "you should have a debug meny where this is toggled off by
  default but can be toggled on for test"). Off, the speeds, the jumps and
  the brackets do nothing and the clock runs at its own pace.
- **A SPEED row** on that DEBUG page: 1×, 10×,
  60×, 300×. The clock runs that many times its own pace until set back
  (300×: a game day in under ten seconds).
- **+1 HOUR and +1 DAY**: the clock jumps ahead, so everything that waits on
  it (crops, animals, the forge, the campfire) is that much further on.
- **`]` held** runs the clock forward at a game hour every five real
  seconds, and **`[` held** runs it back at the same rate, from anywhere
  (owner, the same day: "maybe these bracket buttons if held down advance
  1 hour = 5 seconds"); listed on the Controls page's World group.
- **A small tag** under the compass while the clock runs faster than its own
  pace ("TIME 60×"), so it is never left on by accident.
