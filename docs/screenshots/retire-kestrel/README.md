# retire-kestrel

Screenshots for `openspec/changes/retire-kestrel` (task 3.2): a new world's
start and boats with no Kestrel, and a world saved with a Kestrel opened by
the new build. Each shot is shown beside the same shot from the build before
the change.

How they were taken, 2026-11-15:
- in a Claude Code cloud container, under `xvfb-run` on lavapipe (Mesa's
  software Vulkan). **No GPU was used.** The shots show what the game looks
  like, not how smoothly it runs; frame cost was not measured;
- at 1440 x 900, from the fast build (`--profile fast`);
- "before" is the build at `2bdade5`, the commit before this change; "after"
  is this branch;
- every shot with `--time 12 --rain 0 --weather-at 7200 --frames 120`.

| Shot | Flags | What it shows |
| --- | --- | --- |
| `start-before.jpg` / `start-after.jpg` | `--walk --yaw 75 --pitch -8`, a new memory-only world | The start, looking where the Kestrel was placed. Before, the Kestrel stands on the red level ground 16 m away. After, the same ground is empty. Both logs read `PLACE spawn at 24.117 -2.964 ... 65.6 m elevation`: the start has not moved (requirement "A new world's fleet is the boats"). |
| `loon-before.jpg` / `loon-after.jpg` | `--aboard loon`, a new memory-only world | The boats are where they were. Both logs read `Tern placed at 1362.7 m` and `Loon placed at 883.2 m`. Before, the log also reads `Kestrel placed at 16.2 m`; after, nothing is placed for it. |
| `old-save-before.jpg` / `old-save-after.jpg` | `--walk --world <a copy of the old world>` | A world made by the old build, its walker turned to face the Kestrel (only the saved heading and pitch were changed in that copy's `world.ron`; its vehicle file is the old build's). Before, the Kestrel is there. After, it is not drawn, and the log reads `craft 1 is a retired kestrel: kept in the save, not in the world` (requirement "A retired craft in a save is kept and not flown"). |
| `old-save-boats-after.jpg` | `--aboard loon --world "Kestrel era"` | The old world's boats under the new build: its Loon, boarded, where the old build left it. |
| `realtime-before.jpg` / `realtime-after.jpg` | `--aboard loon --real-time --frames 60` | The LATE_PIPELINE check below. In both builds the Loon lies swamped against the rock (heel +60 deg, 700 kg of water aboard) after the long real-time warm on lavapipe. The two shots are the same, so this is not from this change; it is noted here and not looked into. |

## The old world's vehicle file

The world "Kestrel era" was made by the old build
(`--walk --world "Kestrel era"`). It wrote 113 craft: the Kestrel (ID 1), the
Tern (2), the Loon (3) and the 110 boats its harbours moored, with
`next_id: 114`. The new build opened it twice, and wrote the file both times.
Each time:
- the file still held all 113 records and `next_id: 114`;
- the Kestrel's record was byte for byte the old build's;
- it was the last record in the file.

The first three records of the old build's file are the test fixture
`crates/pbd-app/src/vehicles/fixtures/kestrel-era-vehicles.ron`.

## LATE_PIPELINE

A `--capture` in fixed steps lifts the loading screen without warming, so
every such capture logs late pipelines. The before and after builds log the
same ones. `realtime-after.jpg` is a launch that warms behind the loading
screen as a player's does. Its log:
- `READY playable 111.81 s after launch at frame 423 (world 20.97 s, warm 90.54 s, 78 pipelines)`;
- no `LATE_PIPELINE` before READY;
- one after it, `screenshot-to-screen`, the capture's own instrument.

The build before logs the same: `READY playable 111.78 s ... (warm 90.35 s, 78 pipelines)` and only `screenshot-to-screen` after it. On lavapipe the warm runs into its 90 s cap, so this is a check, not a proof.
