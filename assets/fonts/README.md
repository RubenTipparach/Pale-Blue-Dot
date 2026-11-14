# Fonts

| File | What | Source | Licence |
| --- | --- | --- | --- |
| `kit-font.png`, `kit-font.json` | The kit's own 5 x 7 pixel font for titles, plaque and tab labels, section labels, compass letters and key caps (`stone-and-wood-ui`, Typography; the owner of a "PACK" that read "PAOK": "the c there is kinda weird"). White glyphs on clear, one row; the JSON gives each glyph's x, width and rows | copied byte for byte from `docs/mockups/ui/font/` at commit `d69ee33`, drawn by `tools/art/ui_art.py` | this repository's |
| `VT323-Regular.ttf` | VT323 by Peter Hull, the body face: lines of text, settings rows, notes | Google Fonts, `ofl/vt323/` of github.com/google/fonts (main), fetched 2026-11-14 through cdn.jsdelivr.net; sha256 `cf4de751ada78ceac033dbe16a687742939995b77bc2a052ae17a4957958594d` | SIL Open Font License 1.1, `VT323-OFL.txt` |

The pixel font is sampled nearest and drawn at the whole-number UI scale
(`crates/pbd-app/src/ui_kit.rs`, `KitText`). VT323 is installed in place of
Bevy's default face, so every `Text` reads in it.
