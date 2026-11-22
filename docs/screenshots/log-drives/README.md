# log-drives: logs on the water in the game (`log-drives` 2.1 to 2.3)

Fixed-step captures in the Claude Code cloud container: **lavapipe, no
GPU**, `--profile fast`, memory-only worlds, 1200 x 750, on the owner's
desert river (`--at 23.644 -2.685`, as `docs/screenshots/river-craft-game/`).
They show the look only: **frame time was not measured**.

`--load-hold` puts twelve logs in the raft's hold through the pack's own
move; `PBD_KEY_SCRIPT` presses the game's keys on capture frames: T drops the
anchor, U puts a log from the hold into the water off the bow, Y loads the
nearest log in reach back into the hold, C opens the boat's menu.

```sh
PBD_KEY_SCRIPT="<keys>" PBD_WINDOW=1200x750 xvfb-run -a -s "-screen 0 1200x750x24" \
  target/fast/pbd-app --capture <shot>.png --aboard raft --at 23.644 -2.685 \
  --load-hold --time 11 --rain 0 --weather-at 7200 <args>
```

| Shot | What it shows | `<keys>`, `<args>` |
| --- | --- | --- |
| `logs-off-the-bow.jpg` | From the rower's seat: a log put in off the bow (the log's `LOG 1 unloaded off raft 7, 3.1 m from it`), a bark cylinder with its cut end, floating half out of the river beside the raft, lit as the water and the planks round it. The raft's panel names Y and U. | `60:KeyT,90:KeyU,100:KeyU,110:KeyU`, `--seat --frames 170` |
| `log-loaded-back-into-the-hold.jpg` | One log put in with U, then taken back with Y (`LOG 1 loaded onto raft 7`): the boat's menu shows the hold's twelve logs again. | `60:KeyT,90:KeyU,96:KeyY,150:KeyC`, `--frames 200` |

Not shown, and why:

- **Logs drifting a long way down the river and coming to rest.** The
  rule is pinned by `log_drive::tests` (a log carried downstream stops in
  the still water where a river ends, grounds in the shallows and on a bank,
  floats off again when the water moves). In the game a log rides the same
  current as the raft (`vehicles::fresh_water`), 2 m/s down a reach, but
  lavapipe draws this river at about two frames a second, so a capture long
  enough to follow a log to still water (a 1,300-frame run) did not finish
  inside its 25 minutes. A shot of it is owed from the owner's GPU, with the
  bot video (task 2.4).
- Q with logs selected, over water, throws one in from the bank; it is the
  same log record and the same drift (`logs::throw_in`).

A normal launch with this build (no `--capture`, lavapipe) reached `READY`
with 126 pipelines and logged no `LATE_PIPELINE` in the six minutes after.
