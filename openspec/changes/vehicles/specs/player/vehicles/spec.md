# Vehicles Specification

## ADDED Requirements

### Requirement: Water aboard is a load that can swamp a boat
Rain over a boat's open area and sea over its rim SHALL add water aboard,
bailing and drains SHALL remove it, and the water SHALL add mass and shift
toward the low side.

#### Scenario: Rain in an open canoe
- **WHEN** 60 mm/h of rain falls on the Loon for one minute
- **THEN** 3.6 kg of water is aboard

#### Scenario: Swamping
- **WHEN** the Loon's rim is held under the sea
- **THEN** water floods in until the canoe floats awash
