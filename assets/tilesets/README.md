# Biome terrain source atlases

Fourteen original PNG sheets, each laid out as 16 terrain-material studies.
The eight Tenebris climate biomes are supplemented by redwood, sporewood,
basalt, ice, lunar and asteroid sets. No upstream texture image was copied.

Open [the offline gallery](../../docs/biome-atlas.html) to select a biome,
inspect its material slots and preview repetition. The exact prompts are in
[prompts.json](prompts.json); these assets used the **built-in image_gen tool**.
[biomes.json](biomes.json) records ordered material names, palettes and paths.
[generated-manifest.json](generated-manifest.json) records actual dimensions,
file hashes and production-approval status.

These are generated source atlases, not yet approved texture-array layers.
The requested 1024-pixel canvas and 16-pixel logical texture grid are art
targets; the tool's actual output dimensions and pixel density can differ.
Originals remain unchanged. Equal normalized quarters are sufficient for the
review gallery, but are not a substitute for an exact integer import grid.

Before runtime import, normalize/repaint tiles onto a consistent logical pixel
grid, check opposite edges in repetition, remove unintended directional shading,
review slot identity, and approve palette ramps. Give every final material its
own texture-array layer, nearest magnification and isolated mip chain. Top and
side mapping belong to the hex mesh; do not cut a hex silhouette out of a square
albedo texture. Add emission/normal data separately; bright pigment alone is not
an emissive mask.

`python tools/build_art_catalog.py` checks PNG signatures/chunk checksums,
catalog completeness and actual sizes without editing any images, then rebuilds
the offline gallery and generated manifest. It cannot prove seamlessness,
strict 16×16 pixel art, correct face UVs or visual quality in the game.
