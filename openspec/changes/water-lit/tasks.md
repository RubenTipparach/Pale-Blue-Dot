# Tasks

- [x] 1.1 Before shots at the owner's beach: noon in rain, dusk, night, at the waterline.
- [x] 1.2 Crab-breeze's water read (design 1).
- [x] 2.1 Data: the new albedos, shallow depth, absorption, horizon strength; the retired knobs gone; validation and the config test.
- [x] 2.2 The water uniform: the cover-mixed coefficients, `water_light`, the haze's weather.
- [x] 2.3 `water.wgsl`: lit murk, lit body with shallows, the sky's radiance in the reflection, the terrain's haze.
- [x] 2.4 `planet_surface.wgsl`: the seabed tint lit (new model).
- [x] 3.1 After shots (`docs/screenshots/water-lit/`).
- [x] 3.2 Tests: the shader lights the body and murk and mirrors the sky's radiance; the light on open water falls at dusk, at night and under cloud.
- [ ] 3.3 Open: at night the sea mirrors the L2 sky's overcast glow, which lights the ground too, but the dome draws none, so the sea is about twice the night sky over the horizon (still darker than the beach). The night dome under cloud is its own step.
- [ ] 3.4 Found: the shader tests passed with `load_depth` missing from `water.wgsl` (the game caught it at run time); they do not compile the whole module.
