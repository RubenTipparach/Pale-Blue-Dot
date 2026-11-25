# Design

## 1. The values

- **Domestic, wild:** `Trust.value` / 100, shown as `taming::hearts`
  (`taming-hearts`).
- **Domestic, kept:** `FarmState.bond`, 0 to 100, saved with the animal
  record (a new field, `#[serde(default)]` so old saves read it as 50). A
  hand feed adds 10. A day it is content adds 2. A day it is hungry takes 5.
  Shown as hearts out of five, the same scale.
- **Happiness:** `FarmState::mood` (0 to 100, 25 per need met), the needs
  from `Conditions` and `fed_until`, and `health` with `sick`.

## 2. The card

A UI card, not a thing in the world (like the hearts). It is placed beside
the F prompt while the aim is on an animal:

```
Yard hen
♥♥♥♡♡            domestic
😊 ▓▓▓▓▓░░░  50   happiness
🌾 💧 🏠 ⬚         food water shelter space (lit when met)
▬▬▬▬▬▬▬▬ health
```

Icons are pixel art in the inventory icons' style (1 px outline). The text
is the animal's name only; the rest is icons and bars, no labels in the game
(CLAUDE.md, "Notes stay out of the game"). A mockup comes first, beside the
F prompt as it is today, for the owner's approval.

## 3. Open question for the owner (with the mockup)

Does the bond do anything yet? Choices: only shown (recommended for now);
more eggs, milk or wool when liked; a liked animal follows when called.
