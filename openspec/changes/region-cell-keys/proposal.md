# Proposal: a cell's address is its region's address plus its place in the region

## Why

The owner, 2026-11-20, on why a 300 m or 9,600 m planet has no cell keys
(`planet-sizes` 1.2 and 1.3):

> then break up the planet to have address of addreses
>
> chunk -> addres
> chunk L0 - chunk L1 ->address

Today a cell's key is one 32-bit number. Every bit is spent on rungs 8 to
11 (`cell_key.rs`):
- 5 bits for the face;
- 2 bits for `level - 8`;
- 12 bits each for `i` and `j`.

So rungs 7 and 12 cannot be written down, and so neither can an edit on
a 300 m or a 9,600 m body. Each larger body would need another widening
of the same flat number.

## What Changes

1. **Two addresses, one inside the other.** A cell's key becomes:
   - the address of its **region**: a fixed-size chunk of the planet, 64
     cells on a side;
   - plus its **place in the region**: two 6-bit coordinates, 12 bits,
     the same on every body.
   The region's address is the only part that grows with the planet, and
   it has room to L17, a 300 km radius.
2. **The region is the chunk the save and the network already plan to use.**
   - At the home world's rung, a 64-cell region is exactly
     `world-persistence`'s level-5 region (about 2,000 cells, about 180 m
     across).
   - That is also `multiplayer`'s checksum unit.
   - Defined by its cell count rather than its level, it is the same size
     on every body.
3. **Saves are written per region.** A region's edits are stored under its
   address once, each edit with its 12-bit place. This is the region
   snapshot `world-persistence` design 8 already plans.
4. **Old saves keep working.** Today's 32-bit key turns into a region and
   a place with no loss, as a pure function. An old save is read as it is
   and moves to the new form when it is opened, the way `exact-cell-keys`
   moved saves off the hash. The old reader is kept.

## Capabilities

### New Capabilities

- `world/cell-addresses`: regions, places in a region, and the key made
  of the two.

### Modified Capabilities

None until it is built. The shipped key stays as it is.

## Impact

- **Unlocks** `planet-sizes` rungs 7 and 12, so every body `star-systems`
  plans can be edited and saved.
- **Code:**
  - `pbd-core::cell_key` gains the region address and the place;
  - `Edits` keys by region, then place;
  - the GPU record still carries a 32-bit number for the shaders' seeds,
    and keeps the old hash, which `cell_key.rs` already keeps for that
    purpose.
- **Saves:** migrate forward on open. Nothing a player made is dropped.
