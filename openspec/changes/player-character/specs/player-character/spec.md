## ADDED Requirements

### Requirement: The player's character is a townsperson's figure
The player's character SHALL be a `townsfolk::look::Look`, drawn by the
same figure code as the townsfolk, with every choice drawn from M5's parts
and palettes.

#### Scenario: A chosen look
- **WHEN** the player chooses a build, a hair style and a shirt colour
- **THEN** their figure is drawn with them by the townsfolk's own code

### Requirement: The look belongs to the install
The look SHALL be kept beside the saves, read by every world and sent to
every host. A world SHALL need nothing new for it. An install with no saved
look SHALL roll one from its key, the same each time.

#### Scenario: An old world
- **WHEN** a world saved before this change is opened
- **THEN** it opens unchanged, and the player is drawn with their saved or
  rolled look

### Requirement: The body is seen only from outside
The player's figure SHALL be drawn in third-person and chase views and in
other players' games, seated in the pose of the craft or mount it rides.
First person SHALL show only the hands.

#### Scenario: Rowing in the chase view
- **WHEN** the player rows and looks from the chase view
- **THEN** their own figure sits at the oars with its hands on the handles
