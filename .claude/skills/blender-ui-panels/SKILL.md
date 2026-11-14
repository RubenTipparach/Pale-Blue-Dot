---
name: blender-ui-panels
description: Build and render UI panel art in Blender, headless (bpy), then pixelate it for Pale Blue Dot's stone-and-carved-wood UI - modelled slabs with rolled or chamfered edges, recessed wells, raised frames, pressed details, proud studs and fittings, lit from the upper left and rendered face-on, with the fit rules (rolled edges never touch, children inside parents, text in the open field) measured not eyeballed. Fleet Command's 1990s instrument-panel method; read the In Pale Blue Dot note first. Use whenever a UI panel, frame, button plaque, slot well or bar is modelled in Blender rather than drawn, a rendered-then-pixelated variant of a panel is wanted, or a panel render "looks flat" or "looks like a primitive stuck on" ("panel in Blender", "render the frame", "bevel", "recess", "pixelate the render").
metadata:
  author: Fleet Command (Claude Code)
  version: "1.0"
---

## In Pale Blue Dot

The owner, 2026-11-14: "pull in UI skills from my fleet-command repo, and develop ui for this game, do
something like with rock and carved wood". Copied from Fleet Command (`RubenTipparach/fleet-command`,
`.claude/skills/blender-ui-panels` at `a4a02f6`, 2026-11-13), where it was written new on 2026-10-30 from a
lighting study of a 1999 music-player skin. Provenance in `.claude/skills/PROVENANCE.md`. Fleet Command's text is
below, unchanged; this note wins.

- **What carries over**: the construction (a plate with a rolled edge, openings cut and the plate following
  them, wells, pressed dimples, proud studs), lighting from the upper left so lips cast a top and left shadow,
  "measure, never eyeball" (rule R: rolled edges never touch; rule T: text in the open field; children inside
  parents), and comparing renders by pixels.
- **What does not**: the backlit film, light linking, brass bezels and lamps under the lip are a 1990s console's;
  Pale Blue Dot's UI is stone and carved wood, lit like the world (sun from above, warm lamplight at night), with
  no backlight. Fleet Command's tools (`tools/blender/ui_shapes.py`, `ui_render.py`, `panel_kit.py`,
  `tools/mockups/formed_metal.py`, `console_fit.mjs`) and its Analog Panel Bible are not copied: none of them is
  in this repository, so read their names below as the upstream record, not as files here.
- **Here the script is `tools/art/ui_panels_blend.py`** (`stone-and-wood-ui`): a stone slab (rough displaced
  face, chamfered edge, a carved well) and a carved wood frame (planks with grain, a bevelled inner lip, iron
  corner plates and rivets), rendered face-on orthographic in Cycles on the CPU, then area-averaged down to the
  pixel size and quantised to the Sweaft-64 palette so it sits beside the hand-placed art from
  `tools/art/ui_art.py`.
- **Blender here is the `bpy` wheel**, not a Blender install: `python3.11 -m venv <dir> && <dir>/bin/pip install
  "bpy==4.5.*" pillow numpy` (4.5.16 LTS worked in a cloud container on 2026-11-14; the wheel needs Python 3.11).
  A cloud container has no GPU, so Cycles runs on the CPU there and says so; on the owner's desktop renders run
  on the GPU (CLAUDE.md, "Render on the GPU when there is one").
- **The result is pixel art, not a painting**: whatever Blender renders is reduced to the game's pixel size and
  palette and drawn at whole-number scales with nearest sampling (`docs/mockups/ART-DIRECTION.md`). A render shown
  smooth is a reference only.

---

*Fleet Command's note and the upstream text, verbatim, below. The original description was: "Build and render 1990s-style instrument panels for Fleet Command's console skins in Blender, headless - a pillowed metal plate with rolled edges over a recessed, backlit display film, formed-metal shapes grown with the display (notches, bridges, lobes), pressed dimples, proud 3D knobs, slot sliders, flat ink that only shows by its backlight, lamps tucked under the lip, light linking between the two layers, and a pixel comparison against a reference. Use whenever making or changing a modelled console skin, a panel with displays in it, a formed-metal shape, the backlight or lamp look, text legibility on a lit display, or when a render \"looks flat\", \"looks like a primitive stuck on\", \"the light washes out the text\", \"the glass has a highlight\" (\"90s UI\", \"backlit display\", \"panel in Blender\", \"notch\", \"formed metal\", \"dashboard look\")."*

## In Fleet Command

New in this repository (owner, 2026-10-30: "Start writing up a guide/bible. UI design Bible for using blender to
render UI stuff the way they did in the 90s ... The Bible should compliment the skill"). This skill is the procedure;
the bible is the shared vocabulary and the diagrams: the Analog Panel Bible, `docs/design/ui-bible.html`,
published at https://claude.ai/artifact/Gzo38Dh7o1EqmfzfpJr8rF (record in `docs/artifacts/ui-bible/README.md`;
its diagrams come from `tools/mockups/ui_bible_svgs.py`). Use the bible's part names and numbers (P1 plate, W3 lip, N2 shoulder and so
on) when talking to the owner, so "the shoulder is too tight" means one thing.

It was learnt on a lighting study of a 1999 music-player skin the owner chose as a reference (two layers: "the top
metal layer, and the back display layer that looks like it was indirectly lit from the top like a dashboard in an old
car"). The study is not committed: it copies someone else's layout (CLAUDE.md sections 1 and 10). What is kept here is
the construction and the numbers. Every skin built with it is our own design.

Where it sits: `tools/blender/ui_render.py` (render harness), `tools/blender/ui_shapes.py` (`NodeKit`, `Shaper`,
`rrect`, `circle`, and the panel kit below), `tools/blender/panel_kit.py` (scene reset, camera). Skins:
`tools/blender/build_ui_<id>.py`. Units: 1 px of the 1x slot is 1 mm; builders take slot pixels with y running down;
renders are at 2x.

**The panel kit is in `tools/blender/ui_shapes.py`**, one implementation for every skin (CLAUDE.md section 6.1): `Panel`
(a `Shaper` with `plate`, `finish`, `dimple`, `cut`, `bevel`, `bezel`, `knob`, `film`, `flat`, `stroke`, `label`,
`lip_light`, `edge_lamp`; its `pt`, `roll`, `dz`, `lamps` and `lip` settings; materials by role in `K.m`: plate, film,
ink, groove, knob, pointer, bezel), `offset_poly`, `notched`, `rr`, the materials `metal`, `satin`, `printed`,
`film_mat`, `film_ink`, and `link_layers` (the two layers, the softbox, the ceiling and the main light). The code
fragments below show how each works; call the kit, never copy it. The bible's tiles (`tools/blender/ui_tiles_tail.py`)
and the Aelith Conclave (`tools/blender/build_ui_aelith.py`) are built with it.

## 1. The two layers

A panel is two things stacked, each lit on its own:

| Layer | What it is | Lit by |
|---|---|---|
| Metal (the faceplate) | Pillowed plates with windows cut through, knobs, rivets, bezels | A soft room light from the top, a softbox and a large overhead area light for the polished parts to reflect |
| Display (the film) | A cream film in a well behind each window, its printing, the slot streaks | Its own glow (backlight), the lamps under each lip, and the main light (a sun from the room light's direction, upper left) so the lips cast the top and left shadow |

Numbers that work (study, 2026-10-30):

| Name | Value | Meaning |
|---|---|---|
| `PT` | 8.0 mm | the plate's top face |
| `ROLL` | 3.8 mm | the radius every windowed plate rolls over with |
| `DZ` | -12.0 mm | the film's surface: 12 mm below the plate top |
| Film well | the film slab is the window grown by 8 px, 2 mm thick | so the gaps between plates stay dark |
| Chassis | a dark slab (`#2e271c`) at `DZ - 6` | behind everything, so gaps read as deep slots |

**Light linking** keeps the layers apart (Blender 4.x, Cycles):

```python
metal_layer = bpy.data.collections.new("metal_layer")      # every mesh, curve and text not printed on the film
display_layer = bpy.data.collections.new("display_layer")  # the film, its ink, the streaks
for ob in bpy.context.scene.objects:
    if ob.type == "LIGHT":
        ob.light_linking.receiver_collection = display_layer if ob.name.startswith("lip") else metal_layer
    elif ob.active_material and ob.active_material.name in ("display", "glowline", "ink", "listink"):
        ob.light_linking.receiver_collection = display_layer   # emissive objects are emitters too
```

Text objects are type `FONT`: include `"FONT"` wherever objects are sorted into the layers, or the printing gets the
room light.

## 2. Footprint-true rolled edges (the plate)

Blender's curve bevel grows a filled 2D curve by its bevel radius (measured: a 100 mm square with bevel 2, 5, 8 mm
came out 104, 110, 116 mm wide). So draw the outside smaller by the roll and every opening larger by it, then bevel:

```python
def plate(outlines, z0, z1, m, edge):        # outlines[0] is the outside, the rest are openings
    offset = [offset_poly(o, edge if i == 0 else -edge) for i, o in enumerate(outlines)]
    cu = K.slab(offset, min(z0, z1 - 2 * edge - 0.4), z1, m, bevel=edge)   # a full round of radius edge
    # then: new_from_object, remove_doubles, recalc normals, flat faces shade flat, link, delete the curve
```

`offset_poly(pts, d)` moves a closed outline d px inward along each point's mitred normal (negative: outward), and
checks the result's area to pick the inward side. It works on any shape, which is what makes formed metal possible.

- A window inset 8 px from the plate edge with `ROLL` 3.8 mm leaves one round bar between them: the frame bar.
- **Corners rounder than twice the roll** (found rebuilding the Krai, 2026-10-31): `plate` offsets the outline by the roll and
  then bevels by the roll, so a corner tighter than about 2 x roll turns inside out and spikes. Ease every corner at least
  that round, and clean each outline of near-duplicate points before it is offset (the mitred offset has no defence
  against them).
- **Lamps round a well come from one helper**, `lip_runs(well, seg, faces)` in `ui_shapes.py`: every edge for rim light,
  `ny < 0` for the lower lips (bottom light), `nx > 0` for the left lips (side light). Never walk a rim by hand.
- Flat faces must shade flat (`f.smooth = abs(f.normal.z) < 0.999`), or the fill's long triangles pick up the
  bevel's normals and show a streak across the plate.

## 3. Cut, bake, dissolve, then bevel (finish)

Carve with Exact boolean cutters (`hide_render`, `use_hole_tolerant = True`), then:

```python
def finish(ob, w, seg=6, angle=20):
    me = bpy.data.meshes.new_from_object(ob.evaluated_get(depsgraph))   # bake the cuts
    bmesh.ops.dissolve_limit(bm, angle_limit=math.radians(0.5), verts=bm.verts, edges=bm.edges)  # clean flat areas
    ob.modifiers.clear(); ob.data = me
    bevel(ob, w, seg, angle)                                              # round the edges the cuts made
```

Without the dissolve, a boolean leaves a long, slightly bent triangle in a flat face and the bevel catches its edge
as a bright sliver (seen beside the bottom dimples in the study). Shading it flat does not fix it; the dissolve does.

## 4. Formed metal: the notch and its family

Never add a primitive on top of a plate to make a bump. Change the window's outline and let the plate's roll follow
it, so the metal is one piece. The notch (the bible's chapter 5):

```python
win = notched(x0, y0, x1, y1, r, cx, hw, fall, h)   # study: notched(11, 11, 539, 134, 6, 340, 74, 30, 30)
# bottom edge y = y1 - lift(x); lift is h within hw of cx, 0 beyond hw + fall, smoothstep t*t*(3-2t) between:
# the smoothstep gives a concave foot fillet and a convex shoulder from one curve
p = plate([sec, win], 0.0, PT, plate_m, ROLL)
dimple(p, 290, 126, 20, 20, PT, 1.2)   # shallow dimples for the knobs that sit in it
finish(p, 1.4)
bezel(win)                              # the brass bezel follows the same outline
```

Keep `fall` at least `h` (30 px over 30 px here); steeper reads as a cut, not a pressing. Keep the knob that sits in
it inside the flat (`hw`) with at least 6 px of metal round it. The bible's catalogue gives nine more shapes built
the same way (bridge, ear, saddle, scallop, cradle, peninsula, terrace, bay, swept edge), each rendered in 3D:
describe the window as a smoothed outline, then plate, bezel and film all follow it.

**The fit check is a required step before rendering a formed shape** (owner, 2026-10-31: "the guide illustrations
violates some guidelines. Like the button overlapping with the bevels ... implement these as 3d renderings and verify
the overlap rule"). `tools/mockups/formed_metal.py --out <dir>` builds each outline (shapely: unions and differences,
every corner rounded with `buffer(f).buffer(-2f).buffer(f)`), checks it, writes `tiles.json` and a check map per
shape, and exits with an error on any failure:

| Rule | Must hold | Numbers |
|---|---|---|
| K | each knob's footprint (radius x 1.14 for the collar, plus MARGIN) lies on flat metal: inside the plate, clear of every roll | ROLL 3.8 mm, MARGIN 2.5 mm |
| R | **rolled edges never touch or overlap** (owner, 2026-10-31: "Bevels should never intersect", "You must measure elements like this carefully", "Avoid two rolled edges from touching or overlap"): a window or slot to its plate's edge and to every other window; a pressed dimple to the plate's edge, a window or a raised plate; a raised plate inside its parent and clear of its openings | 2 x ROLL + LAND = 10.6 mm; ROLL + dimple rim 1.4 + LAND = 8.2 mm; ROLL + LAND = 6.8 mm |
| L | every metal tip is rounder than the roll: the metal opened by ROLL changes by 0.6 mm or less | 0.6 mm |
| P | printing on a display (a gauge, a scale, a hologram) stays PAD inside its window | PAD 3.0 mm |
| B | every layer behind a plate (film, well, sub-display, glow plane, backing) stays inside the plate's footprint, rounded corners included (owner, 2026-10-31: "Some times the UI layer/plane can stick outside its bounds. It must measured properly to fit inside the parent panel"): clamped when built (`behind_layer()`, `Panel.film(inside=...)` or `shape=`), and on the render no lit pixel outside the plates | INSET 4 mm |
| T | text never reaches a piece's ends (owner, 2026-11-04: "these text popups need to be a bit wider so they dont get thier letters cut off"): a live label's room is the piece's open field, its width less both end caps at the drawn height (the 9-slice margins as drawn), 4 px clear each side; a key or popup sized to its text is the text plus both caps plus the gaps, the text centred in the open field (a slanted back and a point differ in width). The battle console does it with `keyCaps()` | GAP 4 px |

**Measure, never eyeball.** Every rule is measured on the geometry (shapely), not judged from a render: the study's
time display sat 5 px from its plate's bottom edge (two 3.8 mm rolls need 7.6 before any flat), most of its windows had
7-8 px of land against 10.6, raised plates sat 2 px from their parent's edge, and a film grown 8 px past a window 8 px
from the panel's edge showed as a cream sliver below the plate. All looked fine at a glance. A piece too thin for 3 mm of
land (a 16 px bar) shrinks its roll and keeps a positive land; two rolls never share a line. `formed_metal.fit()` takes
any plate, windows, knobs, prints, layers, dimples and raised plates, so a whole skin's kit is checked the same way
(`tools/mockups/aelith_shapes.py`, which also measures rule B on the renders with `--verify`).

`--selftest` proves the checker catches eight faults: the first drawings' (a knob on the ear's edge, two windows 4 mm
apart whose rolls overlap, a key on a 12 mm shelf, a bar pinched to 10 mm) and the study's (a window 5 mm from the
plate's edge, a dimple 8 mm from a window, a raised plate 2 mm from its parent's edge, a film grown past the plate).
Render the checked tiles with `tools/blender/ui_tiles_tail.py` (`TILES=formed`). Then look at the check map beside each
render: flat metal light, rolls dark, each knob's clearance a green ring.

## 5. Pressed details

- **Dimple**: an ellipsoid cutter sunk `depth` mm (`dimple(ob, x, y, rx, ry, top, depth, rz=7)`); keys 22 x 16 px at
  2.6 mm, knob seats 20 px at 1.2 mm, a big dish 24 px at 2.8 mm with `rz` 9. Then `finish`.
- **Slot**: an opening in the plate (`rr(x - 7, y0, x + 7, y1, 6)`, 14 px wide); it rolls in like a window. Behind
  it, a 2 px streak at `DZ` glowing `#e8b448`, 13 tick dots each side at 7.7 px pitch, 13 px off the slot.
- **Bezel**: brass `#b0915a`, from 0.8 px inside the metal wall to 1.5 px into the opening, at `PT - ROLL - 1.8` to
  `PT - ROLL + 0.2`, so it shows as a thin gold line under the roll.
- **Rivet**: a small blob, 2.2 px radius, 1.3 mm tall. **Bead**: a round tube along an inset outline, 1.7 mm.

## 6. Knobs that stand proud

```python
def knob(x, y, r, z, pointer=None, collar=True):
    K.cyl(x, y, r * 1.14, z - 0.6, z + 0.7, groove)            # dark collar
    K.blob(x, y, r, r, r * 0.85, knobmetal, z=z + r * 0.3)     # near-full sphere, centre above the plate
```

Polished `#d6d0c4`, roughness 0.16, and a large soft overhead light in the metal layer, or a polished part seen face
on reflects the dark room and renders black. Sizes in the study: push buttons 8 px, play, pause and stop 16 px, next
and previous 10 px, slider knobs 11 px. A pointer is a red line on the crown.

## 7. The film: backlight, lamps, ink

**Film** (`display`): base and emission `#f4d9a9`, emission strength `0.85 * FILM_GLOW`, multiplied by ambient
occlusion (distance 7 mm, power 2.2) so it darkens in a thin contact line at the rim. Two separate scales: `FILM_GLOW`
(0.78, with the main light also on the film) on the film, its ink and the slot streaks, which sets the film's colour
(clear film `#edd2a2` against the reference's `#eacf9f`), and `LAMPS` (0.25) on the lamps. One shared scale was a mistake: dimming the lamps darkened
the film's colour with them.

**Main light** (`MAIN` 0.8, `MAIN_SOFT` 60 deg): a sun from the room light's direction (upper left) linked to the
display layer too, so the top lip casts a shadow on the film's top edge and the left lip a thinner one on its left.
It is what makes a well read as deep. Its angular size `MAIN_SOFT` keeps the shadow soft (owner: "soft shadow
please", "not harsh"): at 2 deg it ended in a hard line. Raise `MAIN` for more shadow and lower `FILM_GLOW` to keep
the colour.

**Sub-displays** (`sub`): a readout box is its own film sunk 1.5 mm into the main one through a hole, with a small
lamp along its top; its printed frame sits on the step, which throws a contact line round it.

**Lamps** (`lip_light`): a short area light, not a point and not a strip.

| Knob | Value | Effect |
|---|---|---|
| `LIP_LEN` | 10 mm (owner: "10 mm 180 degrees cone") | length of the source line |
| `LIP_W` | 20 mm | depth of the source: bigger is softer |
| `LIP_TILT` | 90 deg | from the film's normal toward screen-down; 30-35 deg gives dots, 60 deg starts a spill, 80-90 deg a fan |
| `LIP_CONE` | 180 deg | the area light's `spread`; 180 deg with a 10 mm source is the chosen look, 120 deg with 40 mm also works, under 100 deg is too tight |
| depth | `DZ + 8` mm | 8 mm off the film, inside the well, just inside the window edge |
| `SPILL` | 0.2 (the owner's pick: "Power x0.2 is the sweet spot") | lamp power multiplier |

- On this panel a strip lamp along the whole top edge made a highlight across the glass that read as curved glass.
  A strip is still a lighting option for other skins (below).
- A wall round the well blocks lamps that sit in it: put lamps inside the window footprint (they are invisible to the
  camera) and any well wall outside it.

**Lighting options** (owner: "One strip lamp is ok to do, it's just not for this UI, but in the future it's a lighting
option. Add alternatives like all sides, bottom lit, rim lighting etc"). `edge_lamp` in `tools/blender/ui_tiles_tail.py`
is a strip lamp along any window edge, aimed inward along the film; its power is per mm of edge (times `LAMPS`), so a
longer edge stays as bright. Each option is rendered in the bible's chapter 10 (`TILES=lighting`), tuned so no more of
the film burns to white (251 or brighter) than the spots do:

| Option | Lamps | Burnt to white |
|---|---|---|
| spots (this UI) | 3 x `lip_light`, 10 mm, 180 deg, power 0.3 | 3.2% |
| strip | 1 edge lamp along the top, 0.0004 W/mm, 120 deg | 0.0% |
| all sides | 4 edge lamps, 0.00018 W/mm each | 0.0% |
| bottom | 1 edge lamp along the bottom, 0.0004 W/mm | 3.9% |
| rim | 4 edge lamps, 0.0004 W/mm, 30 deg, 1.5 mm off the film | 0.0% |
| side | 1 edge lamp along the left, 0.0006 W/mm | 3.7% |
| backlight only | none, `FILM_GLOW` alone | 0.0% |

The Aelith Conclave uses rim light on a dark void film: 0.0008 W/mm of edge, spread 120 deg (30 deg left visible rays
between the short lamps), the lamps walked round any well outline in 8-12 px segments, each aimed along its inward
normal, so a formed well is lit evenly all the way round. A dark film needs more lamp than a cream one.

A first pass at 3 to 10 times these powers burnt 9% to 28% of the film white: a strip puts all its light right
against the film's edge, so it needs far less power per lamp than a spot.

**Ink** (`film_ink`): printing on the film absorbs. Its surface reflects 35% of its colour; its emission carries the
full colour at `0.9 * FILM_GLOW`, so it glows faintly through the film and a lamp right above it cannot wash it out.
Sampled from the reference: text and scales `#9a7f4f`, box outlines `#7d6845`, list text `#b39867`, a red trace
`#a83020`, never black.

**Flat print**: everything on the film is flat. Text has `extrude = 0`, lines are flat ribbons (`stroke`), boxes are
flat rings (`flat`), indicators are flat dots; all set `visible_shadow = False`. Fonts whose glyphs are built from
overlapping outlines (Instrument Sans, Work Sans, Big Shoulders at small sizes) fill with holes in Blender; use
DejaVu Sans Bold or Tektur on the film.

**The legibility test** (owner: "Place some text directly under the light and tweak brightness/diffusal until you see
the text"): print a line of text right under the middle lamp, render the top band only (render border), measure
`(film - ink) / film` from the 92nd and 6th luminance percentiles under the lamp.

| Setting | Contrast under the lamp |
|---|---|
| point spot, 1 mm source | 0.14 (text washed out) |
| line source 40 px x 4 mm, `SPILL` 0.8 | 0.34 |
| line source 20 mm deep, `SPILL` 0.3, absorbing ink | 0.59 |
| line source 20 mm deep, `SPILL` 0.2, absorbing ink | 0.62, lamp peak 237 (no clipping) |

Keep it at 0.5 or more, and keep the lamp peak under 250 of 255.

**Children fit inside their parents** (owner: "make sure the UI fits well inside parent elements ... not a hard
rule, but generally we dont want that overlap, it cheapens the look"): every bar, key, readout and knob sits wholly
inside its parent's inner edge with an even margin. A reach-over is formed metal, grown with its parent (section 4),
never a child spilling out. On a page, measure it on the page (`tools/mockups/strip_fit.mjs` does the strips: the
Concord top strip's bars ran across its thin bar until its readouts were padded into the 44 px beside it). A hybrid
skin's vector layer (the Conclave's holograms) is clipped to its wells, so its glow never reaches the metal.

## 8. Compare pixels against the reference

When a render "does not look right", measure before changing:

1. Resize ours to the reference's size and put them side by side (LANCZOS); look first.
2. Sample matching regions: film colour in a clear area, metal colour on a plain plate, ink colours (median and
   darkest quarter of pixels 40 levels under the film).
3. Contrast: the standard deviation of luminance over a metal area (reference 54, ours went 26 to 61).
4. Detail: the share of pixels with a luminance gradient over 40 (reference 17%, ours 7% to 9%: the rest is lettering).
5. Profiles: luminance down a column through a window and across a plate edge. The reference's edge read
   120, 66, 254, 199, 81, 254, 51: two crisp highlights and dark lines, a stepped moulding.

The study's sampled palette: film `#eacf9f` (ours `#e9ce9f`), plate `#bdb8ae` metallic 0.75 roughness 0.22, brass
`#b0915a`, chassis `#2e271c`, engraving `#3a3328`.

## 9. Before calling it done

- On the page, the whole skin passes `tools/mockups/console_fit.mjs` (owner, 2026-10-31: "remember the overlap rule?"):
  every live label, bar, key, title and lamp lies on plain art 3 px clear of every edge. It found, and the kits now
  avoid: a strip whose open band (between rail and lip) is thinner than a label over a bar (lay the label beside the bar,
  `--bar-flow: column`, and set the strip padding to the measured band); a bar groove narrower than its fill; a title
  set on a frame's bevel; a 9-slice key whose noise texture or whose top and bottom margins leave seams across its face
  (a stretched piece's centre is plain, and an x-stretch key's vertical margins are 0); and a 154 px panel that grew
  into the 3D view because its frame and key caps needed more.
- Every formed shape and every kit piece passes the fit check (K, R, L, P, B) before it is rendered; look at its check
  map. After rendering, no lit pixel outside the plates (rule B).
- Every label drawn on a piece sits in its open field (rule T): hover the longest names and look.
- Render the top band at 24 samples while tuning (render border), the whole panel at 64 to finish.
- Look at a crop at 2x of every changed area; check text under every lamp.
- Keep each earlier version of a study beside it while iterating, so "change it back" is one copy.
- Name parts with the bible's names in replies.
