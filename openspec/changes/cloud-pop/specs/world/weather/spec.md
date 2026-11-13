## ADDED Requirements

### Requirement: A cloud tower grows and shrinks without re-forming
As the weather a cloud stands under changes from frame to frame, each of its
billboards SHALL move, resize and change opacity by an amount bounded by the
change in the weather; no billboard SHALL jump to a new place or size in one
frame. A tower that grows taller SHALL gain its top billboard by fading it
in.

#### Scenario: A tower crosses a step
- **WHEN** a site's cloud top rises by a hundredth of the layer's thickness
- **THEN** no billboard's centre moves more than a few metres and none of its sizes or its opacity changes by more than a few percent
