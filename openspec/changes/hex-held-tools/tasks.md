# Tasks

## 1. Data
- [ ] `gen_held_tools.py` writes `assets/models/held_tools.ron`: 4x hexes, 2x poses (butt, rotation, scale, fist, tip), hand prisms with colours, rendered at the game's 60 degree field of view.
- [ ] `gen_held_tools.py` writes the four tool icons; `gen_item_icons.py` stops; both `--check`.

## 2. Core
- [ ] `hexel` on axial coordinates with any spacing and per-hex depth; side walls only over the difference; the picture sampler removed.
- [ ] `hexel::prism`, flattened or not, shaded against a light in its frame.
- [ ] Tests: closed meshes, the depth step, a prism.

## 3. App
- [ ] `held.rs` loads the model file and builds a tool mesh and a hand mesh per tool under one holder.
- [ ] `held.ron` loses the icon-sampling fields; the defaults test follows.
- [ ] The chop pivots about the fist; the rod does not chop.
- [ ] `fish.rs` takes the rod tip from the model file.
- [ ] Tests: the file parses, one piece per tool, only the tool in hand shown, the line at the rod tip.

## 4. Verify
- [ ] fmt, clippy, tests, `openspec validate --all`, both icon checks.
- [ ] Captures of each tool in hand and one mid-chop.
- [ ] The requirement into `openspec/specs/player/equipment` with its tests named.
