## 1. Measured

See the proposal. Instrument: `--frame-log` (the `fine_version` column steps
on each landing, on the same clock as `cpu_main_ms` and `cpu_render_ms`).

## 2. The trace

`--features trace` on the fast profile in its own target folder
(`target/trace`), the same 1 km walk. Rank spans in the frames over 33 ms.
Findings go here before any fix.

## 3. The playthrough scenario

`play-soak` is a showcase: the bot plays through the player's inputs. It is
a measurement instrument (CLAUDE.md, "Write it up before touching code") and
changes nothing in the game.
