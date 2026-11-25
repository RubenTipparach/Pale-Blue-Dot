## ADDED Requirements

### Requirement: Each sea-floor model is its own gathered item

A player SHALL gather by hand, with F, a gatherable piece the sea draws
(staghorn, finger, brain and table coral, soft coral, tube and barrel
sponges, sea fans, anemones, algal turf, coral rubble, kelp stands and the
vent's mussel beds) while the eye ray passes the piece as drawn within
reach. Each model SHALL give its own good, named for it. The big anemone
with its clownfish, a kelp bed's rocks, black smokers, bacterial mats and
floes SHALL NOT be offered.

#### Scenario: A staghorn is a staghorn coral

- **WHEN** a player aims at a staghorn piece on a reef within reach and presses F
- **THEN** the pack gains one `staghorn-coral`, not a generic coral

#### Scenario: The clownfish keep their anemone

- **WHEN** a player aims at the big anemone with its clownfish
- **THEN** F offers nothing for it

### Requirement: A gathered piece is gone until it grows back out of sight

A take SHALL be written as a `sea-take` record keyed by the piece's
derived id, with the pack, in one durable write, and acknowledged only
after the save accepts it. The piece SHALL be absent from its tile from
that frame, without any other piece of the tile appearing in its place.
It SHALL be drawn again only once its kind's regrowth days of world time
have passed and its tile is built again.

#### Scenario: A take survives a reload

- **WHEN** a player takes a piece, quits and opens the world again before its regrowth
- **THEN** the piece is not drawn

#### Scenario: Nothing replaces a taken piece

- **WHEN** a piece is taken from a tile whose budget left other pieces out
- **THEN** the tile draws one piece fewer, and no piece it did not draw before

### Requirement: Bed animals are taken by hand and protected ones refused

A crawler (shore crab, lobster, giant clam, octopus, tube worm) SHALL be
taken by hand with F, giving the `Item::Fish` of its species, and SHALL be
removed from its school. A protected species SHALL be refused with the
protected refusal and nothing taken.

#### Scenario: A seal is refused

- **WHEN** a player presses F aimed at a seal
- **THEN** the note says it is protected and the pack is unchanged

### Requirement: Sea goods sell, crush to fertiliser and cook

Every good this change adds SHALL have a trade value. The grinder SHALL
crush every reef piece, kelp and sea lettuce into fertiliser. Fertiliser
used on a sown, unripe, unfed crop SHALL make the rest of its growth run
half again as fast and its harvest give one more produce, once a sowing.
The campfire SHALL cook steamed mussels and kelp crisps.

#### Scenario: A fed crop ripens sooner and gives more

- **WHEN** wheat sown for 6 days' growth is fed after 3 days
- **THEN** it ripens 2 days later, and its harvest gives one more wheat than unfed

#### Scenario: An old plot is unfed

- **WHEN** a plot record written before this change is read
- **THEN** it reads as unfed and grows as it did

### Requirement: Corals, sponges, fans and anemones decorate

A coral, sponge, sea fan or anemone held SHALL be set by right-click on the
aimed ground within reach, on land or on the sea floor, one to a cell, as
an `ornament` record with its body-relative `f64` point. It SHALL be drawn
with the reef's own model of its kind, lit as the terrain under it is
(`LitByField`, `LitLikeTerrain`). F on it SHALL take it back into the pack.

#### Scenario: A coral set on land stays after a reload

- **WHEN** a player sets a brain coral beside their house, quits and reopens the world
- **THEN** the brain coral stands where it was set, as dark as the ground at night
