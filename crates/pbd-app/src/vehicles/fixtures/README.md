# Vehicle file fixtures

`kestrel-era-vehicles.ron` is a world's vehicle file (`vehicles.ron`, layout
version 1) as the build before `retire-kestrel` wrote it to the disk. It
pins that a save made before the Kestrel was retired still opens, with the
Kestrel's record kept byte for byte
(`vehicles::tests::a_world_saved_with_a_kestrel_opens_with_its_boats_and_keeps_the_record`).

How it was made, 2026-11-15:
- the build at `2bdade5` (`cargo build --profile fast -p pbd-app`) ran
  `--capture old-save-made.png --walk --world "Kestrel era" --time 12
  --rain 0 --weather-at 7200 --frames 120` under `xvfb-run` on lavapipe, a
  new world that placed its Kestrel (ID 1), Tern (2) and Loon (3) and wrote
  them, with the 110 boats its harbours moored, to
  `saves/kestrel-era/vehicles.ron`;
- the first three records were copied out of that file unchanged, and
  `next_id` set to 4 for a file that holds only them. Nothing else was
  edited: every record's text is the old build's own.

`tern-era-vehicles.ron` is a world's vehicle file as the build before
`retire-tern` wrote it to the disk. It pins that every Tern in a save, the
one a new world placed and a harbour's, is held aside and written back byte
for byte, and that a held harbour Tern's berth is not filled again
(`vehicles::tests::a_world_saved_with_harbour_terns_keeps_every_record`).

How it was made, 2026-11-17:
- the build at `2de3c03a` (`origin/main`; `cargo build --profile fast -p
  pbd-app`) ran `--capture old-make.png --walk --world rb-old --at 37.07
  171.25 --pitch -5 --frames 600 --time 23 --rain 0 --weather-at 7200` on
  the M4 Max Mac's GPU, a new world beside the harbour Marenstrand that
  placed its Tern (ID 1) and Loon (2) and wrote them, with the 110 boats
  its harbours moored, to `saves/rb-old/vehicles.ron`;
- the first four records (the Tern, the Loon, and Marenstrand's Tern and
  rowboat at berths 3 and 4) were copied out of that file unchanged, and
  `next_id` set to 5 for a file that holds only them. Nothing else was
  edited.
