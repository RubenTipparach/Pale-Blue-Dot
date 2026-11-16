# Water gameplay: nothing grows where the sea or the tide stands (`flora-v2` 12.11)

The owner, 2026-11-16, on the sea-shore waterline video: "is that tree in your
screenshot in the river?"

**How they were made.**
- **Renderer:** the Mac (M4 Max, Metal), `--profile fast`, fixed-step `--capture`.
- **Size:** each shot halved from 2880x1800.
- **Builds:** before is `claude/water-gameplay` at `da33659`, which has no tree
  change. After is the commit that adds `FLOODED_BIT`.
- **Spot and tide:** both at `18.86 27.54`, the sea-shore video's spot, at
  `--time 12`, where the tide stands low and the tide band is dry sand.

| File | Command | What it shows |
| --- | --- | --- |
| `before-shore.jpg`, `after-shore.jpg` | `--fly --at 18.86 27.54 --height 4 --yaw 180 --pitch -8 --time 12 --rain 0` | Before: three acacias on the sand under the high-water line (1.5 m). After: the sand is bare. The cacti and the trees on the higher terraces are unchanged. |
| `before-shore-high.jpg`, `after-shore-high.jpg` | `--fly --at 18.86 27.54 --height 25 --yaw 180 --pitch -30 --time 12 --rain 0` | Before: acacias stand in the shallows and on the tide band. After: none in the water or the band. The cacti and the forest inland are unchanged. |
