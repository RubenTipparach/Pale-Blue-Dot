# Design

`FarmState::hand_feed` becomes
`fed_until = min(max(fed_until, day) + hand_feed_days, day + cap_days)`.
The hen's egg product has `period_days: 0.25`. Production is otherwise
unchanged: it runs while content (mood >= `content_mood`, 50) and past the
sick line, at `amount / period_days` a day, capped at `cap_days` of it.
A saved hen reads the new period from the config; no save change.

## Feed and time, nothing hidden (owner, 2026-11-20)

"the game is stuck, chicken wont lay anything, I'd rather it be more
deterministic based on feed + time". A penned, content hen laid nothing for
days because her health, sunk while she was loose, had to climb past the
sick line first, unseen. Production now runs while the beast is content and
has food (a hand feed's time or grazing), and nothing else. Health still
falls and rises, and still shows as sick, but no longer holds the product
back. This applies to every kept beast's product.
