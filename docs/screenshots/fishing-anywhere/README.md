# fishing-anywhere

`river-cast.png`: a cast from a river bank lands on the river and floats.
- The walker stands on the west bank of the river at lat 28.25 in the
  capture world, facing east across it.
- The cast's log runs Ready, Charging, then Floating, with the float at
  66.7 m altitude: the river's surface, 66 m above the sea.
- Before `fishing-anywhere`, the float's water was the sea's surface
  everywhere, so a float cast here fell to the riverbed and nothing could
  bite.

How it was taken:
- In-game capture on 2026-11-17, in a cloud container on lavapipe with no
  GPU, 1440 x 900, fast build, a new memory-only world.
- The river was found with the ignored instrument
  `world_map::river_bank_probe::print_the_spawn_river_banks`.

Launch line:

    pbd-app --capture river-cast.png --walk --at 28.25 0.63 --yaw 180 --pitch -15 --cast --frames 320 --time 11 --rain 0 --weather-at 7200

`--cast` takes up the rod and casts once (a capture instrument). Frame cost
was not measured.
