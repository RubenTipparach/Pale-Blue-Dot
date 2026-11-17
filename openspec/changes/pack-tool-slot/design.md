# Design

## Measured (investigation of 2026-11-17)

- **The missing count.** `craft::paint` writes each well's `WellCount` as
  `n.to_string()` only when `n > 1` (`craft.rs`, the slot-stack rule, where one
  item needs no number). The stone axe is `inputs: [("stone", 3), ("fibre", 2),
  ("log", 1)]`, so the log well is always blank. `forge.rs` and `kitchen.rs`
  copy the rule for their inputs. No other system writes the count.
- **The hand.** The tool in hand is not a slot stack. It is
  `fish::ToolSlot(Equipment)` (the tiers owned and the tool held). The HUD
  draws it in a cell left of the hotbar (`equipment::spawn`, painted by
  `equipment::paint`). The pack's HOTBAR row is the ten slot cells and nothing
  else. The only way to change the tool is the G picker, which calls
  `Equipment::hold` and saves with `record_hand`.

## Decisions

1. `need_count(role, n)`: a need well shows `n` always; other wells show it
   above 1. One function, used by the crafting, forge and campfire rows.
2. The hand cell is factored out of `equipment::spawn` as `tool_cell`, used by
   the HUD and the pack, so both are painted by the same system.
3. The pack's HOTBAR row is the hand cell, a 4 px gap, then the ten slots. The
   pack column grows by one cell.
4. A click on the pack's hand cell opens a list of plaques beside it: BARE
   HANDS, then each owned tool by name. A pick goes through one pure function,
   `equipment::choose(&mut Equipment, Option<Tool>) -> bool` (refuses a tool not
   owned; false when nothing changes, so nothing is saved), then `record_hand`.
   A second click on the cell, Escape or a pick closes it.
5. Tools stay out of the slots: dragging a tool into a slot would make tools
   stacks, a save-format change, and is not this fix.
