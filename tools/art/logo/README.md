# The Pale Blue Dot logo

The owner, 2026-11-16: a logo built on the menu's blue marble, the planet from the game inside it,
the rock round it rendered at high quality, PALE BLUE carved in stone with cave scenes of everything
the game is about, and DOT with the moon as its O. Iterated in conversation the same day (one
slab, raised letters, the scenes etched into the frame with no repeats, a space elevator from the
planet to the moon, a comet that starts the story and a starship that ends it, a crystal ball with
point-light glints).

    # once: Blender as a Python module (Python 3.11), with numpy, pillow, scipy
    python3.11 -m venv bpyenv && bpyenv/bin/pip install "bpy==4.5.*" pillow numpy scipy
    # the stone as height fields; preview.png is a quick hillshade
    bpyenv/bin/python tools/art/logo/logo_height.py out
    # the render (Cycles, CPU): the planet is a --view orbit capture with PBD_NO_CLOUDS=1
    (cd tools/art/logo && PLANET_CX=801 PLANET_CY=810 PLANET_R=566 \
        ../../../bpyenv/bin/python logo_render.py ../../../out planet.png ../../../out/logo.png 48)
    # the dark rim, the drop shadow and the deliverables
    python3 tools/art/logo/logo_finish.py out/logo.png docs/promo

Outputs are in `docs/promo/`: the transparent logo at full size and 1024 px wide, and a copy on the
game's night blue.

Fonts: Young Serif (SIL Open Font License), read from the system's canvas-fonts folder at build
time and not committed here. The planet is the game's own render.

The key (`docs/promo/pale-blue-dot-logo-key.jpg`), every etched scene numbered and the story under it:

    python3 tools/art/logo/logo_key.py out/logo.png out/glyphs.json docs/promo/pale-blue-dot-logo-key.jpg

The covers (itch.io's 630 x 500, and 3150 x 2500, five times it), on space and transparent:

    bpyenv/bin/python tools/art/logo/logo_cover.py docs/promo/pale-blue-dot-logo.png docs/promo
