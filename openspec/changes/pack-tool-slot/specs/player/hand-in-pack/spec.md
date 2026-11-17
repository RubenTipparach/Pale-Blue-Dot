## ADDED Requirements

### Requirement: The pack shows and changes the tool in hand

The pack's HOTBAR row SHALL begin with the hand cell, showing the tool in hand
or the bare hand, and a click on it SHALL offer bare hands and each tool owned,
holding the one picked and saving it as the G picker does.

#### Scenario: Swapping the tool from the pack

- **WHEN** the player owns a stone axe and a shovel, holds the axe, and picks SHOVEL from the pack's hand list
- **THEN** the shovel is in hand, the hand cell and the HUD show it, and the save's hand line names it

### Requirement: A recipe shows every input's count

Each input of a recipe line SHALL show its count, a count of one included.

#### Scenario: The stone axe's log

- **WHEN** the stone axe recipe is shown
- **THEN** its log well reads 1
