# ocean-life: water life and sea places (`ocean-environments` B8 to B12)

Fixed-step captures on the M4 Max Mac (Metal, GPU), `--profile fast`, each in
a memory-only world (nothing saved), generator version 7. Looks only: frame
time was NOT measured. Compare with the mockup's `docs/screenshots/mockup-ocean/`
(`reef-day-after.jpg`, `kelp-day.jpg`, `deep-vents-day.jpg`,
`polar-under-ice-day.jpg`, `marsh-ducks-after.jpg`), which were rendered on
SwiftShader.

`pbd-app --walk --at <lat> <lon> --time <hour> --frames <n> --pitch <deg> [--life <id>[:metres]] --capture <png>`

## The bodies (B8)

- `lineup-seabed.jpg`: `--life lineup` on the 3.4 m seabed at (3.5, -18.15),
  11:00, pitch -32. Every body 0.5 m long (a worm 0.8 m tall) whatever its
  species' size, three-quarters to the eye. Front rows: minnow, silverfin,
  perch, ray, eel; reef fish, deepback, serpent, shore crab, lobster; giant
  clam, reef turtle, sea otter, reed dabbler drake and hen; pond diver,
  octopus, plume worm, seal; then the poses: a clam shut, a dabbler upended
  and asleep, a worm drawn in, an octopus inking, a seal hauled out.
- `lineup-surface.jpg`: the same from a 0.7 m shore at (73.858, 151.125),
  12:00, pitch -18, the floaters (ducks, otter, seal) on the water.
- `lineup-seabed-close.jpg`: the seabed lineup cropped, full resolution.
- `life-<id>.jpg`: one species put a few metres ahead with `--life`:
  `shore_crab` (1.2 m), `lobster` (1.8 m), `eel` (2.5 m), `giant_clam`
  (5 m, open) on the 3.4 m seabed; `tube_worm` 6 m off at the vent (the
  plumes in, the eye within 5 m of the nearest). The crab, lobster and eel
  were taken before the depth dimming of B12 went in.

## The places (B10)

- `reef.jpg`: (13.096, 94.896), a 4.2 m reef, 11:00, pitch -12.
- `kelp.jpg`: (71.308, -8.196), 10.5 m, 12:00, looking up 10 degrees.
- `vent.jpg`, `life-tube_worm.jpg`: (37.195, -59.018), 90 m, 12:00, after
  2,800 frames (the walker sinks about 25 m a thousand frames).
- `sea-ice-below.jpg`: (89.778, 137.508), 15.8 m under polar water, looking up
  35 degrees at the floes.

## Honest read

Judged beside the mockup's screenshots:

- **The bodies read as the mockup's creatures**: the fish have their
  bodies, fins and tails in the mockup's colours, the eel and serpent are
  long, the ray has its wings, the crab its claws and legs, the lobster its
  feelers and fan, the turtle its flippers, the octopus its arms, the ducks
  their green or brown heads and white rings, the clams a blue mantle and a
  lid. Under water every colour takes the sea's cyan cast, so the
  clams and the pale fish come out paler and bluer than the mockup's; that
  is the game's own water, which the seabed shares.
- **Swimmers are hard to catch close**: put 2.5 m ahead they swim off in the
  half second before the capture, so the fish are judged on the lineup,
  which is static and enlarged, not at their real sizes.
- **The floaters**: `--life` could not put ducks, otters or seals on the
  0.7 m shore (too shallow 4 m out); the surface lineup shows them on the
  water.
- **The reef is dense and alive with colour** like the mockup's
  `reef-day-after.jpg`: branching and finger clumps, boulders with corals on
  them, soft tufts, sponges, fans, anemones, turf and grass, crowded edge to
  edge. Not yet the mockup's: the table-coral skyline is sparser, there is
  no big anemone with clownfish, no anthias cloud, and nothing sways.
- **Kelp** stands to the surface with a canopy; its fronds are blockier and
  greener than the mockup's swaying brown-gold stipes.
- **The vent** is now dark, as 90 m is, with the chimneys' glowing mouths
  and the white tubes of the plume worms; no smoke and no light cast from
  the glow, so it is darker and plainer than `deep-vents-day.jpg`.
- **Sea ice** is a field of hex floes seen from under it with the sun's
  shafts between them; there are no seals hauled out on them yet (a seal
  lies out on the water) and the floes cannot be stood on (pieces, not
  cells).
- **Pop-in**: tiles appear whole as they come within 46 m, with no dither
  fade.

## Second pass (B13): fades, the reef's life, smoke, seals, the chart

- `fade-distance.jpg`: the same reef seen from (13.30, 94.896), (13.40), (13.50)
  and (13.60) facing north (`--yaw 90 --pitch -3`), about 17, 25, 34 and 42 m
  off: the pieces go through the terrain's screen-door mask as they pass
  34 to 46 m, so the far reef thins to a dither and is gone before its tile
  is dropped. The landing fade (0.8 s from wholly masked) is not caught in a
  capture: the fixed-step harness has every tile landed before frame 60; it
  is pinned by `a_tile_fades_in_when_it_lands_and_out_before_it_is_dropped`.
- `reef-skyline.jpg`: (13.096, 94.896) facing north: the table-coral skyline,
  now three a tile on the deeper rock.
- `reef-anemone.jpg`, `anemone-sway.jpg`: the big anemone at (12.845,
  94.9168), and its tentacles 30 frames apart, bent differently (the sway).
  Its two clownfish circle above it at 0.8 m but are hard to make out among
  the tentacles at this size.
- `reef-anthias.jpg`: a cloud of anthias over the reef at (13.04, 94.9297).
- `kelp-sway.jpg`: the kelp at (71.308, -8.196), 60 frames apart; 5% of the
  pixels changed by more than 20 of 255 between them.
- `vent-smoke.jpg`: the black smokers' plumes rising from their mouths at 90 m.
- `seals-on-floes.jpg`, `seals-on-floes-close.jpg`: seals hauled out on the
  floes from a 0.8 m polar shore at (86.965, -17.827). The floes nearest the
  eye stand tall: they are hex slabs of 0.25 to 0.6 m freeboard.
- `map-sea-places.jpg`: the map's "Sea places" layer (`--walk --menu map
  --map-layer "sea places" --map-mpp 40`), with each place's share in the key.

Honest read: the far reef dithers out rather than popping; the reef now has
its skyline, a big anemone and its fish clouds; the smoke reads as rising
puffs but is blocky, and blue-green rather than black, from the water between;
the seals on floes are small at 10 m and pale; the chart's polar sea fills
the map's bottom rows (the projection stretches the pole). The unlit vent
glow and smoke cannot take the mask, so they appear and go at half fade.
