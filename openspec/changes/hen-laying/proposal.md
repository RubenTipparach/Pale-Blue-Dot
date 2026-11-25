# Hens lay by how well they are fed

## Why

The owner, 2026-11-20: "i think hens should lay an egg based on how much
food they are fed. they shuold lay about 1 egg per 6 hours of ingame time if
they are at least 50% happy". Today a hen lays one egg a game day (48
minutes of play), and a second feed adds nothing to the first.

## What changes

- A hen lays one egg per 6 in-game hours (a quarter day) while her
  happiness is 50 or more and she is not sick. 50 is today's "content" line
  (two needs met). Her laying period goes from 1 day to 0.25 days in
  `husbandry.ron`.
- Feeding adds up. Each hand feed adds `hand_feed_days` (2 days) to the
  food she has, up to `cap_days` (4) ahead. Today a feed only reset it to 2
  days from now. More food is more laying time: a hen fed once lays about 8
  eggs over the 2 days, and one fed twice about 16 over 4.
- Grazing still counts as food while she has grass in her pen.
- Waiting eggs pile up to `cap_days` worth (16 for a hen), as today.

## Not in this change

- Feeding other animals more (the same stacking applies to them; their
  product periods stay).
