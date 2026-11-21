## ADDED Requirements

### Requirement: A feed shows hearts for how much the animal likes the player
Each feed of an animal SHALL show a row of hearts above its head that rise and fade. The number SHALL be `ceil(trust / 20)` clamped to 1 to 5 for a wild animal, and 5 for a feed that tames it or for a kept animal.

#### Scenario: More trust, more hearts
- **WHEN** a wild woolback at 0 trust is fed once (trust gain 25)
- **THEN** two hearts show over it, and a second feed (trust 50) shows three

#### Scenario: Taming shows full hearts
- **WHEN** the feed brings trust to 100 and tames it
- **THEN** five hearts show

### Requirement: A fed wild animal follows the player for a while
A wild animal that eats from the player's hand SHALL follow the player for `taming.wild_follow_s` seconds without being scared off, then go back to grazing.

#### Scenario: Follows, then wanders
- **WHEN** a wild woolback is fed and the player walks away
- **THEN** it follows for 20 s and then stays and grazes
