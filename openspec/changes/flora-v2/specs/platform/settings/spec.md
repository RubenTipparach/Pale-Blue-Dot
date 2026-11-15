# Player settings

## ADDED Requirements

### Requirement: Pixel grass is a kept player setting

The settings SHALL include Pixel grass, off by default, shown as a check on
the GRAPHICS page and kept in the settings file at version 5. A file from an
older version SHALL read it as off and keep everything else. A capture or a
benchmark SHALL NOT read it, and SHALL take `--grass tufts|blades` instead.

#### Scenario: An older settings file

- **WHEN** a version 4 settings file is read
- **THEN** Pixel grass is off and every other setting is as the file says
