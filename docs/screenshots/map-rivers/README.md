# The map's rivers hold water (`map-rivers`)

The owner, 2026-11-17: "i see you carved rivers in here, but we need to fill
it with water on this map too!" Before, the M key's map drew a river's carved
bed as the ground under it, so a river showed only as a dark groove from the
hillshade. After, the river's water is painted in the map's own shallows
colour, at the width its water has in the world (`openspec/changes/map-rivers`).

Captured 2026-11-17 in a Claude Code cloud container with no GPU: rendered on
lavapipe (software Vulkan) under `xvfb-run -a -s "-screen 0 1440x900x24"`, at
1440 x 900, from the fast build (`cargo build --profile fast -p pbd-app`), each
a fresh run on a new memory-only world. They show what the map looks like, not
how fast it draws; frame cost was not measured. "Before" is the build of
`66780be5`, the commit this change starts from; "after" is this change.

Launch line, with `<mpp>` from the table:

```text
pbd-app --walk --time 12 --menu map --map-at 29.57 1.37 --map-mpp <mpp> --capture <file>.png
```

| file | `--map-mpp` | what it shows |
| --- | --- | --- |
| `before-mpp3.png` | 3 | Holbrook and Linenleigh at 3 m a screen pixel: the rivers are carved grooves drawn as meadow and sand; only the sea and the river mouths are blue. |
| `after-mpp3.png` | 3 | The same view: every river's water drawn in the shallows blue, from its springs in the hills down to the sea, as wide as its water. The sea is unchanged. |
| `before-mpp12.png` | 12 | The continent at 12 m a pixel, on the cached base level (11.3 m a pixel at the equator). |
| `after-mpp12.png` | 12 | The same: rivers narrower than a pixel show as faint continuous lines, painted by the share of the pixel their water covers. |
