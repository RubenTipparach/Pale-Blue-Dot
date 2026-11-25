## ADDED Requirements

### Requirement: Looking at an animal shows how much it likes the player and how happy it is
Looking at an animal SHALL show a card with its domestic level (trust when wild, bond when kept, as hearts out of five) and its happiness (mood out of 100 with each need shown as met or not, and its health).

#### Scenario: A penned hen recovering
- **WHEN** the player looks at a penned hen grazing with space but no water or shelter, health 20
- **THEN** the card shows happiness 50 with food and space lit, water and shelter dark, and a red health bar
