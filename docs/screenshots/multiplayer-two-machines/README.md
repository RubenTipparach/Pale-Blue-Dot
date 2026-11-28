# Multiplayer on two machines (task 6.2)

`openspec/changes/multiplayer` design 20, run 2026-11-23 on the owner's own computers, at the owner's
instruction ("Multiplayer test should use Mac and laptop mining mike sessions"): the M4 Max MacBook Pro
hosting, the Windows laptop joining, over the home Wi-Fi, both on the `fast` build of `8fa0ac35`. The
sessions coordinated through the `session-mailbox` branch and PR #148. Real GPUs, real time; frame
cost was not measured.

| file | machine | what it shows |
| --- | --- | --- |
| `mac-host.png`, `mac-host.log` | Mac, `--world mp-test --host 47123` (192.168.0.76) | The host's player 2.4 m down in the hole the laptop dug, walls cut, night sky above. The log: the laptop joins as player 1, the Mac digs two layers by hand, then four `edits from the network applied`: the laptop's digs. |
| `laptop-joined.png`, `laptop-joined.log` | Laptop, `--join 192.168.0.76:47123` | The joined player in the same hole at night, facing its wall. The log: `joined ... as player 1`, the Mac's two digs applied before the laptop dug anything, then four `sent to the host: Dig` each applied about 80 ms later. |

No `refused` and no `SERVER_DISAGREES` on either side.

Found in the run, for the follow-ups:
- With `--join` the game still opens on the title screen; CONTINUE enters the joined world.
- The joined player's hotbar is empty while the server gave them the starting kit: the memory-only
  world the joining game opens deals no kit. Its pack and the server's copy disagree until the
  inventory message (task 8.2). Fixed 2026-11-27 (`multiplayer` design 22 and 23): the server
  sends a joined player their own slots, and a joined player's dig makes a drop that everyone
  sees and their magnet can pick up. The kit is empty since 2026-11-17, so a first join still
  starts empty-handed. Found with it: a guest's slots were written into the host's save as the
  host's; fixed in the same change. Not yet seen on two machines.
- Bare hands dug only straight down for the laptop's player; looking a little ahead did nothing. Not
  seen in single player yet; to check whether the joined game's aim or the request path drops it.
