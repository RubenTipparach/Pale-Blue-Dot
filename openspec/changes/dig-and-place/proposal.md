# Proposal: dig a block out and put one back

## Why

The columns are there, the hotbar is there, and nothing connects them. The
tier holds an authoritative `Column` per cell within ninety metres, the
fragment shader draws its runs, `PlanetContact::stand` walks on them, and
`pbd_core::inventory::Slots` already holds stacks of `Item::Block(Material)`
with a selected slot. What is missing is the verb: point at a cell, take it,
and put it back somewhere else.

This is the first WORLD MUTATION in the repository, so it is also the first
time three standing rules have anything to bite on: one authoritative
implementation shared by single player and a future multiplayer, terrain that
stays authoritative on the CPU, and every accepted mutation entering a durable
path on the frame it happens.

## What

Tenebris's own shape, ported, because the owner named it as the spec:

- **A fixed-step ray march finds the target.** `interact.rs` marches the eye
  ray in 0.25 m steps to a maximum distance, resolves each sample to (tile,
  depth) with a warm hint, and stops at the first solid block. The last
  non-solid sample is where a placed block goes. Ours is the same march
  against (cell, layer), and the hint is `Tier::walk`, which already steps
  cell to cell across the edge a ray is outside of.
- **A dig takes the material into the hotbar; a place spends the selected
  stack.** `Slots::give` and `Slots::take_held` are already written and
  already hotbar-first.
- **The edit is a sparse override, not a rewrite of the column.** The tier is
  regenerated whenever the player moves far enough, so an edit stored in a
  `Column` would be lost on the next rebuild. `pbd_core::column::Edits` maps a
  cell's STABLE ID to the layers that differ from what the generator says, and
  `generate` applies it after the worms carve. That makes an edited column a
  pure function of (direction, worms, edits), which is the property the whole
  tier rests on.
- **Only the edited column and its neighbours are rebuilt.** A neighbour's
  flank is clipped against this column's runs, so it has to be repacked too.
  Everything else in the tier is untouched, and the GPU sees a handful of
  48-byte records rewritten rather than a 3,000-column rebuild.
- **It is durable on the frame it happens.** Every accepted edit is appended
  to a save file before the frame ends and replayed at startup. The rule in
  `CLAUDE.md` is explicit that a queued write is not a save, so the append is
  flushed and acknowledged rather than batched.

## Not in this change

- No block-breaking TIME, no tools that matter, no durability. Tenebris's
  `mining::break_time` and its tool tiers are a system of their own and want
  the tool art with them. A dig here is one click.
- No drops on the ground: a dug block goes straight to the hotbar, as
  Tenebris's own creative path does.
- No multiplayer relay. The rule that decides an edit lives in the core so
  that a server can call the same function, and nothing calls it yet.
