# Tasks

- [x] 1. Measure each town's frame cost from the air (existing flags, windowed, release)
- [ ] 2. Overview captures of the five towns; choose the hour
- [ ] 3. `--route towns`: pick the tour from the world's sites, the keyframes, the spline, the kinematic camera; quits at the end
- [ ] 4. Fly it windowed in real time with `--frame-log`, twice: length, frame times
- [ ] 5. Report the length and frame rate to the owner; record only on their go (`obs-record`)
- [ ] 6. The owner's notes on the first cut (design, "The owner's notes on the first cut"):
  - [ ] 6.1 A pass through each town at rooftop height along its long axis, 12 m/s, the look leading into the streets.
  - [ ] 6.2 Every leg flown by the far-side route's rule (climb, cruise at 25% of the leg within 300 to 1,500 m, glide into the next pass), no cut or jump; its camera rig.
  - [ ] 6.3 The camera's orientation carried as a quaternion, turned by the shortest rotation and eased, its roll held to the horizon, its look never within 20 degrees of straight down or up. Verify: a unit test flying a look through straight down shows no roll jump.
  - [ ] 6.4 Measure the gaps on the sides of hex faces from the air (column views at 60, 300 and 1,000 m over stepped ground) and write the fix up in the change that owns that ground.
  - [ ] 6.5 The jungle village in the tour.

