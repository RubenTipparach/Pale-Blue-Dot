# Weather: the clouds' history respects silhouettes

## ADDED Requirements

### Requirement: The clouds' history never crosses a silhouette

The cloud history SHALL blend a previous frame's texel into this frame's only
when that texel's march saw what this frame's march sees: its march reached as
far as this frame's, and its cloud lies no further than this frame's march can
see, both measured from the previous frame's eye. The test SHALL be the one the
composite's upsample applies within a frame. A texel with no previous texel
passing the test SHALL use this frame's march alone.

#### Scenario: Turning with a tool held against the sky

- **WHEN** the walker's view turns steadily with the held tool in front of
  cloudy sky
- **THEN** no cloudless print of the tool trails its outline, and no cloud is
  drawn over it

#### Scenario: A hill moves in front of a cloud

- **WHEN** the view moves so that terrain stands in front of cloud that the
  previous frame saw
- **THEN** that cloud is not carried over the terrain from the history

#### Scenario: At rest

- **WHEN** the view does not move
- **THEN** the history accumulates exactly as before, and the picture is
  unchanged within the capture's run-to-run floor
