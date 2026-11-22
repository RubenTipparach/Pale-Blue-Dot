# Multiplayer: single player through the in-process server

For `openspec/changes/multiplayer` task 4.2 (design 18). Captured 2026-11-22
in a Claude Code cloud container: **lavapipe, no GPU**, under `xvfb-run` at
1440x900, from the fast build, a fixed-step `--capture`. It shows what
happened, not how smoothly; frame cost was not measured.

| file | flags | what it shows |
| --- | --- | --- |
| `dig-through-server.jpg` | `pbd-app --world mp-local-a2 --at 28.3 1.5 --walk --dig 3 --place 2 --time 12 --pitch -60 --frames 400` | Three layers dug under the camera and two stones stacked in the hole, every edit decided by the in-process server with the save as its store. Rain is falling. |

Checked in the log: `scripted dig 1` to `3` taken (cell 1327571860, layers
208 to 206), and no `SERVER_DISAGREES`, `edit refused` or `edit BLOCKED`
line: the server took every edit the client predicted. Nothing on screen
differs from a build before the change; the point of the step is that it
does not.
