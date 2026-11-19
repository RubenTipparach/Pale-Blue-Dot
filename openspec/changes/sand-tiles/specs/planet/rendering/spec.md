## ADDED Requirements

### Requirement: Beach sand and desert sand are drawn apart

The beach sheet's sand tiles the game samples, its top (0,0), side (1,0) and
wet sand (2,0), SHALL hold no blue or teal texel: no texel whose blue channel
exceeds its red, or whose green exceeds its red. The beach top SHALL carry
shells and shell bits in warm colours, with no outline. The desert's top (0,0)
SHALL hold no shell, and its mean colour SHALL differ from the beach top's
mean by a hue a player sees: the desert's red-to-blue ratio SHALL be at least
twice the beach's (today it is 1.5 times; the candidates 2.4). Both tops SHALL
tile with no seam.

#### Scenario: No blue on the beach

- **WHEN** the committed atlas's beach tiles (0,0), (1,0) and (2,0) are read
- **THEN** no texel in them has blue over red or green over red

#### Scenario: Two sands

- **WHEN** the mean linear colours of the beach top and the desert top are
  read off the committed atlas
- **THEN** the desert's red over blue is at least twice the beach's

#### Scenario: The tufts follow

- **WHEN** the sand tiles are repainted
- **THEN** `TUFT_RAMPS` for the beach and the desert are the ramps read off the
  new tiles (`the_plants_colours_are_the_atlas_s`)
