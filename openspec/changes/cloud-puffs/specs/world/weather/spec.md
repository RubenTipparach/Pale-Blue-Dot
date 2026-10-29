## ADDED Requirements

### Requirement: Clouds can be drawn as lit puffs
With `cloud_renderer: Puffs` the renderer SHALL draw the cloud layer as one
camera-facing billboard per puff, upright on the sphere and lit by the sun and
the sky, instead of marching it.

#### Scenario: The same weather lays the same puffs
- **WHEN** the sites are made twice for the same body
- **THEN** every site has the same direction and hashes
  (`cloud_puffs::tests::the_same_body_lays_the_same_sites`)

### Requirement: Puffs are laid out by the shown weather
Puffs SHALL stand on fixed sites over the sphere. The cover, cloud top and
optical depth of the shown weather maps at a site SHALL decide whether it
carries a puff, its size, height, opacity and darkness, and a puff SHALL fade
in and out with the cover rather than appear in one frame.

#### Scenario: More cover, more puffs
- **WHEN** puffs are laid out for a cover of 0.2 and of 0.8 over every site
- **THEN** about a fifth and about four fifths of the sites carry a puff
  (`cloud_puffs::tests::more_cover_carries_more_puffs`)

#### Scenario: A puff fades with the cover
- **WHEN** a site's cover rises through its threshold in small steps
- **THEN** its puff's opacity passes through several intermediate values
  (`cloud_puffs::tests::a_puff_fades_in_with_the_cover`)

#### Scenario: A low deck keeps lanes
- **WHEN** a full cover is laid out with a low top and with a tall one
- **THEN** the low deck leaves about `cloud_deck_floor` of its sites open and
  the tall one none
  (`cloud_puffs::tests::a_full_low_deck_keeps_lanes_and_a_tall_one_does_not`)
