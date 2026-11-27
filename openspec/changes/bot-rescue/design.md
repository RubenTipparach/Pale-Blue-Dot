## 1. The watch

`Bot::rescue_watch` runs after the act's frame, at both places the bot's
intent is applied (an injected act and the shot's own). It tracks
`rescue_anchor` (where it last made 3 m, and when):
- the anchor is reset when the goal is not a point or is within 5 m;
- nothing inside an act (a replan, up and out, a dig, a select loop) resets it.

That is the difference from the 60 s watchdog in `walk_to`, which re-anchors
itself when it fires and so can loop forever.

## 2. The rescue

`out.rescue` takes a direction 20 m on toward the goal. The system queues
`walking::put_ashore`, which finds the nearest dry land to it as a new
world's spawn does and stands the walker there. Every plan, climb and route is
dropped, so the act plans afresh from dry ground.

## 3. Numbers

180 s is past every existing escape's own time (60 s watchdog, 90 s up and
out, 20 s swim out), so the rescue only fires after they have all failed.
Recommendation taken (ask only with screenshots).
