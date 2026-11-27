# Design

## 1. One clock

Everything that waits does so on `Sun::clock` (`farm`, `husbandry`, `craft`
jobs, `kitchen`): seconds since day 0. So a faster clock and a jump are the
only two things needed; nothing else learns about speed.

- `sky::TimeSpeed(f32)`, a resource, 1 by default, multiplies the clock's
  advance. A showcase's own `ClockRate` still wins where one is set.
- The pause panel's speeds are `[1, 10, 60, 300]`. Not saved: a world
  opens at 1×.
- `]` held adds `DAY_S / 24 / 5` world seconds per real second on top of
  the clock's run (an hour every five seconds); `[` held takes them away,
  as the TIME bar can already set an earlier hour. Ignored while a world's
  name is typed, as L is.
- +1 HOUR and +1 DAY call `Clock::advance` with 120 s and 2,880 s.

## 2. Text

Locale keys `time.speed`, `time.rate` (`"{rate}x"`, the kit font has no ×), `time.hour_on`,
`time.day_on`, `time.fast` (`"TIME {rate}x"`); `controls.time_back` and
`controls.time_forward`.

## 3. Checks

A test: the clock at 60× advances sixty times as far in the same frame
time, and +1 DAY moves the day by one.
