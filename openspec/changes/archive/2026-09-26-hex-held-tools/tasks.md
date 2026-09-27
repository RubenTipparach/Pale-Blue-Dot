# Tasks

## 1. Data
- [x] `gen_held_tools.py` writes `assets/models/held_tools.ron`: 4x hexes, 2x poses (butt, rotation, scale, fist, tip), hand prisms with colours, rendered at the game's 60 degree field of view.
- [x] `gen_held_tools.py` writes the four tool icons; `gen_item_icons.py` stops; both `--check`.

## 2. Core
- [x] `hexel` on axial coordinates with any spacing and per-hex depth; side walls only over the difference; the picture sampler removed.
- [x] `hexel::prism`, flattened or not, shaded against a light in its frame.
- [x] Tests: closed meshes, the depth step, a prism.

## 3. App
- [x] `held.rs` loads the model file and builds a tool mesh and a hand mesh per tool under one holder.
- [x] `held.ron` loses the icon-sampling fields; the defaults test follows.
- [x] The chop pivots about the fist; the rod does not chop.
- [x] `fish.rs` takes the rod tip from the model file.
- [x] Tests: the file parses, one piece per tool, only the tool in hand shown, the line at the rod tip.

## 4. Verify
- [x] fmt, clippy, tests, `openspec validate --all`, both icon checks.
- [x] Captures of each tool in hand (`docs/screenshots/held-*.png`). The mid-chop capture looked at the aircraft, not a block, so it shows no chop: not verified by picture.
- [x] The requirement into `openspec/specs/player/equipment` with its tests named.
