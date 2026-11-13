# Weather: what a drop on the lens shows

## ADDED Requirements

### Requirement: A drop on the lens bends only what is behind it

A rain drop on the lens SHALL show the scene behind its own outline, bent
toward its centre by `rain_lens_bend` times the pixel's distance from that
centre at the middle, and by none at the rim, alike for every size of drop
and alike across and down the screen. `rain_lens_bend` SHALL be validated
within 0..=2, so no pixel in a drop samples from outside it; below 1, a drop
SHALL be an upright magnifier.

#### Scenario: A drop against an overcast sky

- **WHEN** the walker stands in heavy rain with grey sky above a dark treeline
- **THEN** a drop on the lens over the sky shows sky, slightly magnified
- **AND** it is not a dark blob, nor rimmed with the trees below it

#### Scenario: A drop straddling the treeline

- **WHEN** a drop on the lens sits across the edge between sky and trees
- **THEN** it shows that edge magnified and the right way up at the default bend
- **AND** it does not show the sky above it on its upper rim or the ground below it on its lower rim

#### Scenario: An illegal bend

- **WHEN** `weather.ron` sets `rain_lens_bend` below 0, above 2, or to NaN
- **THEN** loading the config refuses it

### Requirement: A drop on the lens shows its edge and the sky's light under it

A rain drop on the lens SHALL lose `rain_lens_rim` of its light where its
alpha passes a half, and SHALL gather `rain_lens_glint` times the sky's mean
radiance on the lower part of that edge, and on no other part. Neither term
SHALL sample the scene outside the drop. `rain_lens_rim` SHALL be validated
within 0..=1 and `rain_lens_glint` as finite and not negative.

#### Scenario: Drops against an overcast sky

- **WHEN** the walker stands in heavy rain facing open grey sky
- **THEN** each drop reads as a faint ring of sky, darker above and lighter below

#### Scenario: Drops over wet ground

- **WHEN** the drops on the lens sit over dark wet soil under an overcast sky
- **THEN** each shows a light crescent along its lower edge

#### Scenario: Drops at night

- **WHEN** it rains at midnight
- **THEN** the light under the drops is no brighter than the night sky's mean light
