## 1. The float's clock

`Fishery` gains `float_wait_s` and `float_due_s`. In `Phase::Floating` the
wait counts up on the frame's dt. When it passes the due time, one school
spawns at the float, the wait goes back to 0, and a new due time is drawn from
`spawn.float_s`. In any other phase both are 0, so each cast draws a fresh
time once it is floating.

## 2. Where, and which fish

`try_spawn` takes an optional float point. With one, it uses:
- the float as the centre;
- a distance from `spawn.float_m`;
- only species that can bite;
- no ring caps (fish, life, per species).

Without one, it is the ring's spawn as before. The place rule (fresh water's
top, the sea's class, temperature, depth and `ocean::places::environment`) is
the same code for both, so the float's fish are the fish that live there.

## 3. Owner's numbers

10-30 s is the owner's own range ("random between 10 and 30"). 4-8 m sits
outside the lure's 3 m scent reach, so the school swims in on the
`fish-behaviour` approach and is seen arriving rather than appearing at the
hook. It also fades in by the usual school fade.
