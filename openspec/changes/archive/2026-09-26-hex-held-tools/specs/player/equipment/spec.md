# Equipment: the hex held tools

## MODIFIED Requirements

### Requirement: The tool in hand is drawn as a hex-pixel model of its icon
The tool in hand SHALL be drawn in first person from its hex model: hexagonal
prisms on a quarter-unit hex grid whose row 0 is the handle's line, so a
handle is straight. Each hex has its own colour and depth, and a side wall is
built only where a hex stands above its neighbour. A right hand SHALL grip it,
drawn as its own mesh that moves with the tool. The tool's slot icon SHALL be
drawn from the same hexes, so the two cannot differ. The tool SHALL swing while
a block is being broken, and the fishing line SHALL leave from the rod model's
tip.

#### Scenario: Changing tools
- **WHEN** the player equips the pickaxe
- **THEN** the pickaxe's hex model and the hand are in hand, and no other
  tool's model is (`held::tests::only_the_tool_in_hand_is_shown_with_its_hand`)

#### Scenario: A thin handle
- **WHEN** any tool's model is built
- **THEN** it is one connected piece, and its handle is whole rows of hexes
  (`held::tests::every_tool_is_one_piece_with_a_hand`,
  `held::tests::a_handle_is_whole_rows`,
  `hexel::tests::a_round_handle_is_closed_and_one_piece`)

#### Scenario: The line and the rod
- **WHEN** a line is cast
- **THEN** it starts at the tip of the rod model being drawn
  (`held::tests::the_line_leaves_the_rod_model_tip`)

#### Scenario: The icon and the model
- **WHEN** a tool's shape changes
- **THEN** its slot icon and its model change together, both drawn from its
  hexes (`tools/gen_held_tools.py --check`)
