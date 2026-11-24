# Multiplayer on one Mac: a headless server, the live game and bots

`multiplayer` design 21, task 6.5; 2026-11-23, the owner's MacBook (M4 Max),
`fast` build, the game windowed at 1440x900 on the GPU. Everything over
loopback (`127.0.0.1:47124`).

1. `pbd-headless serve --port 47124`: a dedicated server, the world in
   memory, seed 77, each player dealt 64 stone (pickups wait for task 8.2).
2. `pbd-app --join 127.0.0.1:47124`: the live game joins it.
3. Three bots, `pbd-headless bot --role i/3 --at 24.117,-2.964 --pace 0.4
   --stress 30`, build a hut beside the live player, then dig and place
   at random round it for 30 s, then check their copy against the server.

| Shot | What it shows |
| --- | --- |
| `1-join-opened-on-title.png` | Bug 1: `--join` opened on the title screen (CONTINUE there would open a saved world). Fixed. |
| `2-joined-before-the-bots.png` | The live game in the server's world, before the bots. |
| `3-three-bots-build-the-hut.png` | The hut the three bots built, seen live from the game, 8 s after they joined. |
| `4-the-hut-from-the-door.png` | The same hut after the stress, from the doorway side. |
| `5-late-joiner-sees-the-hut.png` | The game restarted after the bots left: it joins late and is sent the hut as a snapshot. Before bug 3 was fixed it saw an empty desert. |

Results:
- `bots.txt`: all three `BOT_OK`, 387 requests, every one answered once, every
  region's changes in order, each bot's copy equal to the server's at the
  end; round trip p50 5-6 ms, p99 9-10 ms, max 64 ms.
- `soak-7-bots-60s.txt`: seven bots in one process for 60 s of stress:
  1715 requests, all `BOT_OK`, 1597 changes each in order, p99 23-33 ms.
- `serve.log`: the server's side, including the live game's own dig
  (request 388, accepted) and its rejoins after a restart.
- `live-game.log`, `late-joiner.log`: the game's join and its applied edits
  (302 "edits from the network applied" lines; the late joiner took 144
  edits on join).

Found and fixed on this run (each in the same commit):
1. `--join` opened on the title screen (`desktop.rs`, the opening screen).
2. A restarted game rejoining as the same player was sent its welcome on
   the dead connection, and the old connection's close would have taken the
   player out (`net_play.rs` and the headless server;
   `a_player_who_rejoins_is_served_on_the_new_connection`).
3. A joining game dropped every message that came in the same batch as the
   welcome, which is where the snapshots of the world's edits are
   (`net_play::join`): a late joiner saw none of what others had built.

Seen, not fixed: the joined game logs Bevy "Could not despawn entity"
warnings (12) while many network edits land in a burst; harmless so far.
Not shown by one machine: a real network's loss and delay, firewalls,
Windows against macOS. Those stay with the two-machine runs.
