## ADDED Requirements

### Requirement: A full river memo keeps learning

A river memo past its cap SHALL forget its oldest values to keep new ones,
rather than keep nothing new, so ground first reached after the memo filled
is still cached. Every value it holds SHALL be the value its call returns.

#### Scenario: The newest values are held past the cap

- **WHEN** a memo has been given many times its cap of distinct keys
- **THEN** the last keys given are held with their values, the first are
  forgotten, and it holds no more than its cap

#### Scenario: A key is held once

- **WHEN** the same key is given twice and then removed
- **THEN** the memo counts it once while held and holds nothing after
