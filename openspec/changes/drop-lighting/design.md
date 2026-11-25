# Design

## Measured

- Drops are spawned with `field_light::LitByField` only. That mode adds the
  cell field's sky and lamp light to Bevy's own lighting ("Bevy's picture"),
  whose sun is not shadowed by the terrain. A drop at the bottom of a pit is
  lit by full sun while the pit's walls are in shade, and the sand tile reads
  near white with bloom (`docs/screenshots/drop-lighting/before-pit.jpg`).
- `LitLikeTerrain` (`sun-shadows` decision 7) lights a mesh by the terrain's
  own fill and sun, with the terrain's sun shadow maps, instead of Bevy's
  picture. Town pieces use it. `take_the_field` honours it on the tagged
  entity itself, not only its children.

## Decision

The drop's entity carries `LitLikeTerrain` too. The held tool and hand keep
their own light (they are in the eye's space, `hand-model`).
