# Provenance of the fish icons

Five of the eight fish are reused from Tenebris, with its own 16x16 icons,
copied byte for byte. Nothing in the build reads the reference checkout: these
copies are the source now, and `fish::tests::the_copied_tenebris_icons_are_unchanged`
recomputes each one's FNV-1a hash against this table, so an edit in place fails
until the record is updated with it.

- Source: [RubenTipparach/tenebris](https://github.com/RubenTipparach/tenebris)
  at `ef9865166349c7a3c4ae09acdd20640daf85b4fd` (2026-08-15), the pin in
  `docs/source-migration.md`. Tenebris's Rust tree is MIT.
- Generator there: `tenebris-rs/tools/gen-fish-sprites.py`.

| File | Source path | Tenebris `AnimalKind` | Bytes | FNV-1a 64 | SHA-256 |
| --- | --- | --- | ---: | --- | --- |
| `minnow.png` | `tenebris-rs/assets/textures/items/fish_minnow.png` | `Fish` | 131 | `0a7b2c228683c225` | `12d5ebab372b55aa7ec8fb0be6c9aac8d01e6671c27de714fb047804052456f4` |
| `reef.png` | `tenebris-rs/assets/textures/items/fish_reef.png` | `LargeFish` | 142 | `709cf425ddae26d7` | `a2fc702ef09e181f9da091ccca92c3c969bc9de2fb79f02d5ec74fd54c83745d` |
| `eel.png` | `tenebris-rs/assets/textures/items/fish_eel.png` | `LongFish` | 132 | `083766415f5072dd` | `a7cf8314e6c6bb837a64d0cfadd557192e3a7bc0ba79c6fe98941ea7711a4f20` |
| `ray.png` | `tenebris-rs/assets/textures/items/fish_ray.png` | `FlatFish` | 132 | `dc0aabd09cb6d717` | `8112354c7bfee754bf38a530c6f5106e4d8f9e95d49790f3b40594bb0e1f0446` |
| `serpent.png` | `tenebris-rs/assets/textures/items/fish_serpent.png` | `SerpentFish` | 154 | `de6c42d6a31481c3` | `51c237190148237ebb21c21500880e4b5b4e514eb67f7428cb1b3fabff7bbcdc` |

The other three (`silverfin.png`, `perch.png`, `deepback.png`) are new to Pale
Blue Dot and drawn by `tools/gen_item_icons.py`, on the same rules as the
Tenebris generator: head to the right, a dark, mid and light ramp, a 1 px eye,
a transparent ground.
