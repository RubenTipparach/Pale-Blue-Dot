---
name: hand-drawn-illustration
description: Draw icons and small illustrations in the style of the owner's friend's hand-drawn upgrade icons (Mining Mike's resources/sprites/upgrade icons/): flat saturated blocks, loose marker hatching in the shape's own lighter and darker tones, form-following shadow and light crescents, highlight dabs, a bright inner rim, edges in a deep tone of the same hue, sparks and speed lines, no frame. Painted in code by a PIL crayon kit, judged against the friend's originals on a contact sheet, approved on a page before anything replaces art. Use whenever a new upgrade, card, ability, item or research icon is wanted "in the same style", "hand drawn", "like my friend's drawings", or an existing placeholder icon needs real art ("draw an icon for", "make art for the new upgrade", "illustrate this", "match the hand-drawn icons", "replace the placeholder icons"). Not for Tactical Airsuite card art, which is pixel art: use balatro-pixel-art there.
metadata:
  author: Claude Code, for the owner's projects
  version: "1.0"
---

> **Retired here (2026-11-12).** The owner asked for the solitaire collection's pixel-art style
> instead; the mockups' art is now drawn with `balatro-pixel-art`. This skill stays for reference.

> **In this repository (Pale Blue Dot).** Copied from `RubenTipparach/rubens-solitare-collection`
> (`.claude/skills/hand-drawn-illustration`, commit cdb3b24) on 2026-11-11, at the owner's word:
> "create textures (pull in hand drawn art skill from the rubens-soliatire-repo)". Here it paints the
> textures and icons for the design mockups in `docs/mockups/` (scripts in `tools/art/*_art.py`,
> PNGs beside each page). The notes below about Tactical Airsuite and godot-sandbox are that
> repository's history and are kept as they were.

> **Retired here for card art (2026-10-21).** The owner: "Rewrite skill based on balatro pixel art and
> redraw everything in this style". Ships, trick emblems and item icons are now pixel art drawn by
> the `balatro-pixel-art` skill's kit; `tools/art/*.py` no longer use the crayon kit. This skill stays
> for reference and for the owner's other repositories.
>
> **In this repository (Tactical Airsuite).** Copied from `RubenTipparach/godot-sandbox`
> (`.claude/skills/hand-drawn-illustration`, commit 8e87e76) on 2026-10-21, at the owner's word:
> "Gonna need art for the shop items. Pull in the hand drawing skills from the godot-sandbox repo on
> my account". Here it paints the shop's ammunition and refit cards (`tools/art/shop_icons.py`,
> output in `assets/art/items/`), and, since the owner's "3d drawings not doing it for me, change all
> card art to hand drawn skill" (2026-10-21), every ship on the cards (`tools/art/ship_art.py`: one
> plane per fighter rank, the three airships, each faction and suit; `--sheet` lays them beside the
> friend's icons). Seven of the friend's icons are copied to `references/theirs/`
> so `scripts/sheet.py` works without godot-sandbox; the process stills and the kit are as they
> were. Paths below that name `godot-sandbox/` mean that repository. This game's palette leans
> warm (sand, brass, rust) but the friend's saturated colours stay: an item should pop on its card.


# Hand-drawn illustration, in the friend's style

The owner, 2026-10-18: "Can you make me an illustration skill based on my friends
drawings?", with a time-lapse of the friend drawing Mining Mike's upgrade icons
(`80503d14-drawings.mov`, 2 min 36 s, 512 x 512). This skill is what that video
teaches: what the drawings look like, the order they are built in, and a kit
that paints the same way.

**Be honest about what this is.** Claude cannot draw by hand. The kit paints
with code, copying the friend's method stroke by stroke; it gets close, and the
owner judges whether it is close enough. Say so when presenting, and never pass
a drawing off as the friend's.

## The friend's drawings

The finished drawings are the 24 icons in
`godot-sandbox/resources/sprites/upgrade icons/` that carry no `source` tag in
their PNG (`armory-crayon` marks the ones this kit painted). Among them:
`inferno`, `death aura`, `plating`, `orbital lasers`, `magnetic field`,
`evasion`, `power output`, `laser drill`, `vitality`, `corroder`,
`critical hit`, `cryo rounds`, `multi beam`, `eagle eye`, `arc coil`,
`mineral attractor`. **Read several before drawing anything.**

`references/process/` holds stills from the time-lapse, each a strip of the
same drawing at successive stages:

| Strip | Shows |
| --- | --- |
| `shield_plating.png` | block, then bands (rim, dark band, face) and the yellow accent line, then rivets and highlights |
| `heart_health.png` | heavy pale highlight hatching, then a saturated pass, a bright rim and a darker edge |
| `fireball_inferno.png` | the flame block with a yellow inner line, then the face added |
| `pickaxe_mining.png`, `laser_drill.png` | the object first, then the effect (chips, sparks, the burst) |
| `wrench_repair.png`, `orbital_lasers.png`, `gear_cycle.png`, `coil_power.png`, `acid_corroder.png` | a plain subject, then the symbol or energy that says what it does |

`references/kit_vs_original.png` is the kit's worked example (`scripts/example.py`)
beside the friend's `vitality` and `plating`: the standard to beat.

## How a drawing is built (the order in the video)

1. **Block the silhouette** in one flat, saturated mid tone, with a soft edge. One
   bold shape that reads at 48 px.
2. **Split it into bands**: a rim, a darker inner band, a face. Each band is its
   own flat tone. Add **one contrasting accent line** (the shield's yellow edge,
   the fireball's yellow inner flame line).
3. **Hatch** each band with loose marker strokes: long strokes laid mostly one
   way, in lighter and darker versions of the band's own colour, then a lighter
   cross pass. Strokes show; they are texture, not noise.
4. **Shade with the form**: a darker scribbled crescent along the shadow edge
   (lower right) and a lighter one toward the light (upper left). Never a
   straight band across the shape.
5. **Highlights**: a few fat light dabs on the lit side; on glossy things (the
   heart) a bright rim just inside the edge.
6. **Edge** in a deep tone of the same hue (navy on the steel shield, deep green
   on the heart, burnt orange on the flame). Black ink only on mechanical parts.
7. **Details and energy last**: rivets, dots, teeth, a face; then what the icon
   DOES around it: sparks, four point twinkles, dashed arcs, speed lines, a
   symbol (an arrow, a plus, a bolt).

What the drawings share:

- a transparent ground, **no frame, no tile, no letters**; the subject alone,
  filling most of the square;
- one subject, often with a character (the grinning fireball, the skull);
- saturated, slightly chalky colour: lime green, hot orange, cyan, violet,
  steel blue, industrial yellow; never grey mush;
- light from the upper left, every time.

## The kit

`scripts/crayon.py` (PIL only). Work on a 512 canvas, saved down to 128
(`Icon.save`), tagged `armory-crayon` in the PNG. Read the whole file. The
friend's method is these calls, in this order:

| Step | Call |
| --- | --- |
| 1 to 6 for one shape | `ic.drawn(poly, color, light=(-1,-1), hatch, shade, highlight, rim_color, edge="tone", edge_w)` |
| 1 alone | `ic.flat(poly, color)` |
| 3 | `ic.scribble(poly, color, var, width, length, angle, spread, density)` |
| 4 | `ic.crescent(poly, direction, amount)` then `ic.scribble_in(mask, color, ...)` |
| 5 | `ic.dabs(poly, color, direction)`, `ic.rim(poly, color)` |
| 6 | `ic.outline(poly, tone(color), width)` (`tone()` is the deep same-hue edge) |
| 7 | `ic.line`, `ic.dot`, `ic.sparkle`, `ic.speed`, `ic.zap`, `ic.glow` |

Geometry helpers: `circle`, `ellipse`, `arc`, `ring`, `rect`, `roundrect`,
`star` (with `jag` for a burst), `rotate`, `translate`, `scale`, `centroid`;
colour: `mix`, `darker`, `lighter`, `tone`.

The older calls (`fill`, `shade`, `shape`) are the first version of the kit,
which painted the 36 Prototype Armory icons; they stay exactly as they are so
those approved icons rebuild pixel for pixel (`godot-sandbox/tools/armory_art/`
loads this file). New work uses `drawn()`.

`scripts/example.py` is the pattern: one function per icon, shapes back to front,
`drawn()` for each, details on top.

## Doing the work

1. **Read the references**: three or four finished icons near the subject, and
   the process strip closest to it.
2. **Write the drawing** as a module of functions (`ICONS = {"file name": (fn,
   seed)}`, as `godot-sandbox/tools/armory_art/icons_*.py` do), in the project
   that needs it, never inside this skill.
3. **Sheet it beside theirs**: `python3 scripts/sheet.py OUT.png drawn/*.png --ref inferno --ref plating`,
   then Read the sheet. Judge at 48 px as well as big: does it read at a glance,
   does it look like the same hand, is it distinct from what is already there?
4. **At least three passes.** Fix what the sheet shows and re-sheet. "It runs" is
   not done.
5. **Show it for approval** before it replaces anything: a page with each new
   icon on the screen it lives on, at its real size, beside the friend's icons
   and the placeholder it replaces (`godot-sandbox/docs/mockups/gen_armory_icons.py`
   is the pattern). Publish it and add it to the README's published pages.
6. **Install only after approval**, and only over placeholders.

## Rules

- **Never overwrite, redraw or "improve" the friend's drawings** (CLAUDE.md,
  "Art: Never Replace Hand-Made Art"). A PNG with no `source` tag is a person's;
  this kit only writes where there is none, or over its own `armory-crayon`
  output, or over a placeholder.
- **Once a person paints over a kit drawing it is theirs.** Do not run a build
  over it again.
- **Placeholders stay PNGs**, never shapes drawn in code at runtime.
- **The kit's defaults are frozen** for the approved Armory icons. Add new
  behaviour as new calls or opt-in arguments, and re-prove the Armory rebuild
  (`python3 godot-sandbox/tools/armory_art/build.py /tmp/x`, every PNG identical
  to the installed one).
- No em or en dashes in anything written (CLAUDE.md).

## Where this has been used

| When | What | Where |
| --- | --- | --- |
| 2026-10-18 | The 36 Prototype Armory icons (24 weapons, 12 level-up cards), first version of the kit, approved | `godot-sandbox/tools/armory_art/`, the Armory Icons page |
| 2026-10-18 | `drawn()` and the friend's method, from the time-lapse | this skill |
