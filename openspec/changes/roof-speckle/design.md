# Design: roof-speckle

## 1. Where it shows, and where it did not

On this base (`claude/perf-tuning` at `3b44fb5`, the fast build, an M4 Max
Mac, Metal, 2880 x 1800, fixed-step captures of 150 frames, new worlds):

- Not in the slice 2b spots (`--walk`, the newel, the flight's foot, the
  landing), at 07:00, 11:00, 18:30 or 22:00, at 100 % or 85 % render scale,
  dry or raining, fixed-step or `--real-time`. The walker's `--up 3.2` did
  not stand it upstairs in these runs: every one reported 1.6 m above the
  ground floor, so these shots looked at the first storey's slab.
- Shown with the free camera at the top storey: `--view column --at
  29.63663 0.80156 --height 4.6 --yaw -0.4 --pitch 30 --open-doors --time
  22`. Through the doorway to the newel, the newel's ceiling is dark with
  fine orange speckle in curved fringes.

## 2. Instruments

Two ignored tests in `pbd_core::settlement::tests`, run with
`cargo test --profile fast -p pbd-core --lib <name> -- --ignored --nocapture`:

- `measure_faces_that_can_fight`: every pair of faces in a village building
  that face the same way within 3 cm and overlap. The pairs a camera in a
  room can see whose two sides are lit differently are the top ceiling under
  the soffit (1 cm, room against outside) and slab tops against beam tops
  (1-1.5 cm). Many timber and stone boxes touch exactly, in one material and
  one light, where a fight cannot show.
- `measure_faces_on_a_lights_band_edge`: every level room face within 3 mm
  of a room light's band edge and inside its reach. Only the Fieldstone
  house, and only its two newel sconces:

  | Light | Edge | Faces on it |
  | --- | --- | --- |
  | Sconce at 3.2 m | top, 6.000 m | the top storey's plank ceiling (18), plaster and stone tops (12) |
  | Sconce at 4.2 m | bottom, 3.000 m | stone tops (11), plaster (6), timber undersides (6) |

  A newel's sconces stand at `(s + k/6) * turn + 2.2` with 1.2 m below and
  2.8 m above, so the sconces at k = 2 and k = 4 put their edges on whole
  storeys. A candle's band ends 5 cm past its ceiling and floor and a
  hearth's 5 cm past its ceiling, by hand; a sconce's does not.

The soffit was tested directly, with a throwaway `PBD_EXP_SOFFIT` override
of its height (not committed): lifted 0.5 m, set on the ceiling (0, 1 mm,
3 mm) or dropped 1 m, the stair-well and top-storey views at 22:00 moved by
nothing a viewer could see (under 1,100 of 5.2 M pixels, all in already dark
ceiling). It is not the speckle.

## 3. Why a face on an edge speckles

`room_lights` computes `rise = dot(body - at, normalize(at))` with
`body = world_position - centre`. `body` and `at` are about 4,800 m long and
an f32 steps every 2^-11 m = 0.49 mm there; `world_position` is
interpolated, so `body` lands on that grid differently pixel to pixel. A
face whose true `rise` is within half a millimetre of `span.y` is kept by
some pixels and dropped by the next: lit and unlit speckle. The grid's
interference with the pixels gives the curved fringes the owner saw, and
across a face the light's up is tilted from the building's by the light's
distance from the frame's origin over the radius, so the band's edge
crosses the face along a line, with clean light on one side, clean dark on
the other and speckle where the two are within the step.

## 4. The fix: slack on both edges

`BAND_SLACK` = 5 cm on both edges of every band, in the shader, with one
constant in the core.

- 5 cm is the margin the candles and hearths carry already, and is twice the
  worst tilt over a light's reach in the shipped towns: a building's frame
  spans under 20 m, so a light stands at most 20 m from its frame's origin,
  and over its 7 m reach its up leans 7 x 20 / 4,800 = 2.9 cm from the
  building's.
- What moves: a face within 5 cm outside a band now takes that light. The
  next storey's floor sits 0.21 m over a ceiling (`SLAB_M` + `LIFT_M`), so no
  storey takes the light of the one under it. The top storey's ceiling takes
  its own storey's sconce, as the room's ceiling should.
- Measured in the shader rather than in the data so it holds for every light
  kind and every band a later kit adds.

Rejected: measuring `rise` along the building's own up (needs the frame's up
per light in the storage row; the slack is enough at this radius and costs
nothing); adding 5 cm to the sconce alone (the next band laid on a storey
would speckle again).

## 5. Verification

- `towns::tests::no_room_face_lies_on_a_lights_band_edge` lays Holbrook at
  the game's radius, computes `rise` for every level room face's corners
  in a light's reach as the shader does, in f32, and asserts each is more
  than a centimetre from `span.x - BAND_SLACK` and `span.y + BAND_SLACK`.
  It fails with no slack (the Fieldstone house's two sconces).
- `field_light::tests::the_band_slack_is_the_cores` holds the shader's
  constant to `BAND_SLACK_M`.
- Before and after, the same free-camera shots at 22:00 and 18:30, in
  `docs/screenshots/roof-speckle/`.
