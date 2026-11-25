# Proposal: what planet sizes the game can make, and a viewer to try them

## Why

The owner, 2026-11-19: "what planet sizes can we make in this game", then
"consider performance and stuff, also if its easier make some tools to
mockup planet types to view, a seperate build system in some javacript
electron app or somethign".

The size ladder is set by CLAUDE.md ("Hex size is fixed across every
planet"): a cell is 2.833 m on every body, so a radius sits on
`R = 300 m * 2^(L - 7)`. What the engine can carry today is narrower than
that ladder, and what each rung costs has never been measured.

## What Changes

1. **The supported sizes are written down** (design 1): what the engine
   carries today, what each needs, and the two gaps:
   - a 300 m body (L7), which the cell key cannot address;
   - a 9,600 m body (L12), which needs both a new cell key format and
     four times the home world's cells.
2. **The cost of each size is measured** before any body is built at it
   (design 2):
   - on the CPU in a cloud session: generation, cloud sites, the map,
     memory;
   - on the owner's desktop: frame time, which a cloud session cannot
     measure.
3. **A planet viewer** (design 3): a local web page that runs the game's
   own generator, compiled to WebAssembly, so a planet type can be looked
   at by size, seed and generator version without building the game. It
   is a tool in `tools/planet-viewer/`, not a second generator.

## Capabilities

### New Capabilities

- `platform/planet-viewer`: the viewer, and the rule that it runs the
  game's generator rather than a copy.

### Modified Capabilities

None. No body size changes, and no save changes.

## Impact

- **No game code changes.** The viewer adds a small wrapper crate that
  calls `pbd-core`; `pbd-core` itself is untouched.
- **Saves:** none. The cell key is not changed here; widening it is
  written up as a separate step (design 1.3) and waits for a request.
