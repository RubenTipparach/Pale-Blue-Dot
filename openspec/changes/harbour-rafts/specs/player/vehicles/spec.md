## MODIFIED Requirements

### Requirement: Harbours moor no Tern
A harbour template's boats SHALL make a Loon for each `canoe` and a raft at
each `rowboat` or `sail` berth whose middle is at least 5.1 m from the rafts
given a berth before it; its cog SHALL make none. A raft's berth SHALL be
numbered apart from the template boat's, so a berth held by a retired record
still gets its raft, and a berth whose craft is held SHALL count as made, so
it is not filled again
(`towns::tests::every_harbours_boats_are_made_once_at_their_berths`).

#### Scenario: A harbour in an old world
- **WHEN** a world saved with harbour Terns, rowboats and a cog is opened and
  the player comes to the harbour
- **THEN** its canoes are moored as before, a raft lies at each raft berth,
  and no boat is made at the cog's berth

#### Scenario: A new world's harbour
- **WHEN** the fleet first meets a harbour in a new world
- **THEN** it moors its canoes and its rafts, no two rafts within 5.1 m of
  each other, each anchored at its berth
