## ADDED Requirements

### Requirement: Items gained are shown as toasts

When an item comes into the pack, a toast SHALL show its icon, its name and
the count gained, for about three seconds before fading. Gains of the same
item while its toast is shown SHALL add to that toast. At most ten toasts
SHALL be shown at once; more SHALL wait and appear as others fade.

#### Scenario: Digging dirt twice

- **WHEN** the player digs two dirt blocks and both are picked up within three seconds
- **THEN** one toast shows the dirt icon and "Dirt x2"

#### Scenario: Twelve different items at once

- **WHEN** twelve different items are gained in one moment
- **THEN** ten toasts are shown and the other two appear as the first fade
