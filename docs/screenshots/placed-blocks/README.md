# placed-blocks-keep-material: before and after

A row of blocks placed from the pack on desert sand: dirt, stone, rock and
sand, left to right (grass did not find a free cell in this row). Rendered on
the Mac's GPU (Metal), `fast` build, 960x600 window at 2x.

- `before-desert.jpg`: the shader of `origin/claude/env-v2`. Dirt is drawn as
  the desert sheet's sandy earth, stone and rock as its striped red sandstone,
  sand as desert sand: the owner's bug.
- `after-desert.jpg`: this change. Dirt is the meadow's earth, stone the
  meadow's grey stone, rock the mountains' slate, sand the beach's sand: each
  as its hotbar icon shows it. The desert's own ground is unchanged.
- `compare-desert.jpg`: the two side by side.

The "before" frame is this branch's binary with the base branch's
`planet_surface.wgsl` swapped in: the old shader ignores the placed sheets the
new CPU side uploads, so it draws exactly what the base branch drew.

Command (both):

```
PBD_WINDOW=960x600 target/fast/pbd-app --walk --spawn desert --time 13 \
  --pitch -22 --place-row dirt,stone,rock,sand,grass 9 --frames 400 \
  --capture <png>
```

`--place-row` is a capture instrument added by this change: it places the
named blocks through `apply_edit`, the path a player's place takes.
