# Proposal: a performance rig, so a change cannot quietly cost frame rate

## Why

The owner, after running the current build: "mega oof, the frame rate is bad
and the ghosting is bad ... we should follow [godot-sandbox's] style so we
dont end up with regression when we run the game and end up with worse fps
than before." Then: "set up harnesses to fly into clouds and run tests to
collect perf data."

Today every number in this repository is measured by hand: a `--capture` run
here, a `--frame-log` there, each with its own flags, and nothing says what
the last build measured on the same flight. The frame log reports wall time
only, so a slow frame cannot be split into the clouds' GPU cost and the rest.
A change can therefore move frame time and nobody sees it until the owner does.

## What changes

A rig in the style of godot-sandbox's battle bench (studied in
`C:\Users\santi\repos\godot-sandbox`: fixed scenarios, a warm-up that is
thrown away, a check that the stressful part really happened, JSON evidence,
old/new runs interleaved in one sitting, budgets per scenario):

1. **GPU time per pass** (an instrument). The water composite node's passes
   (water compose, cloud march, clouds composite, rain, overlay, lens) record
   Bevy render-diagnostic spans, on when `--frame-log` is given. The frame log
   gains the clouds' GPU milliseconds and the whole frame's.
2. **Scenarios that end themselves.** `--route` runs with `--frame-log` quit
   at `ROUTE_COMPLETE`, like `--walk-distance` quits at `WALK_DONE`. The
   scenic route is the "fly into clouds" scenario: it plans a leg through the
   thickest cloud it can reach, and the report checks that it got there.
3. **A pricing switch** (an instrument): `PBD_NO_CLOUDS=1` skips the cloud
   march and composite, so a report can say what the clouds cost on the same
   flight, in the way godot-sandbox's `BENCH_NO_FOG` prices its fog.
4. **`tools/perf_suite.py`**: runs the scenarios windowed, uncapped, from a
   fresh world with a fixed name and clock, for one or more builds or
   variants, interleaved A/B/A/B; writes one JSON per run and a report
   (Markdown plus PNG) with p50/p95/p99/p99.9, the budget misses, the clouds'
   GPU share, and the in-cloud stretch of the flight.

Nothing here changes what the game draws or how it plays when the flags and
the variable are absent.

## Out of scope

Making the clouds cheaper or ghost-free. Those are priorities 1 and 2 in
`CLAUDE.md` and get their own change; this one exists so their effect can be
measured, and so a regression is visible before the owner runs the build.
