# Design

`FarmState::hand_feed` becomes
`fed_until = min(max(fed_until, day) + hand_feed_days, day + cap_days)`.
The hen's egg product has `period_days: 0.25`. Production is otherwise
unchanged: it runs while content (mood >= `content_mood`, 50) and past the
sick line, at `amount / period_days` a day, capped at `cap_days` of it.
A saved hen reads the new period from the config; no save change.
