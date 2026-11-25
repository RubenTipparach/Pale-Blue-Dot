# Localization Specification

## ADDED Requirements

### Requirement: Every word the player reads comes from the locale file
The game SHALL take all text it shows the player from `assets/locale/locale_en.yaml`
(or the chosen language's file) by key, and SHALL NOT show a string literal
written in code or a name written in a data file.

#### Scenario: A new label in code
- **WHEN** a commit adds `KitText::new("Inventory", ...)` to a screen
- **THEN** it does not compile, and `cargo test -p pbd-app --test locale` names the file and the literal

### Requirement: Keys and the file agree
Every key the code or data uses SHALL exist in `locale_en.yaml`, every key in
it SHALL be used, and each key's placeholders SHALL match the arguments given.

#### Scenario: A key missing from the file
- **WHEN** code calls `tr("menu.picture.sharpen")` and the file has no such key
- **THEN** the locale test fails naming the key

### Requirement: The baseline only shrinks
The lint's baseline of hard-coded text left from before the rule SHALL NOT
gain entries, SHALL lose each entry once its literal is gone, and SHALL be
deleted when empty.

#### Scenario: A screen migrated
- **WHEN** a module's literals are moved into the locale file but their baseline entries are left in place
- **THEN** the locale test fails until the stale entries are removed
