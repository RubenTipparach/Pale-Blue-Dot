# Design: which fields are a PLANET and which are LAND

## The rule

A field sampled on the unit sphere at scale `s` makes its coarsest feature
`R / s` metres across, and `lacunarity^(octaves-1)` times finer at the top of
its stack. So a scale is an ANGLE, and an angle held fixed while the body grows
is a feature that grows with it.

Each field is therefore declared as one of two kinds:

- **Planet-scale.** Its size is an angle. A world has a handful of continents
  whatever its radius. The scale is authored and does not move.
- **Land-scale.** Its size is a number of METRES a player walks over. The
  scale is `R / feature_metres`, derived, so the same generator on a bigger
  body makes MORE hills rather than bigger ones.

The same split applies to amplitude. A planet-scale term carries its height as
a share of the body's relief budget; a land-scale term carries it in metres,
so raising the summit does not also make the ground underfoot rougher.

## The numbers

`R` is 4,800 m and the reference's is 300, so a fully metric field's scale is
sixteen times the reference's. Slope is the honest way to compare, because it
is what a walker feels and it is dimensionless: amplitude over wavelength.

| field | kind | reference | ours now | proposed | why |
| --- | --- | ---: | ---: | ---: | --- |
| continent | planet | 0.8 | 0.8 | **0.8** | the map the owner has seen |
| mountain | land, slope-matched | 2.5 | 2.5 | **7.0** | see below |
| hill | land | 5.0 | 5.0 | **80.0** | 16x: 60 m across, as there |
| detail | land | 12.0 | 12.0 | **192.0** | 16x: 25 m across, as there |
| river | land | 1.3 | 1.3 | **20.8** | 16x: a channel a few cells wide |
| moisture | land, coarsened | 1.6 | 1.6 | **6.4** | see below |
| rocky region | planet | 2.0 | 2.0 | **2.0** | a mountain COUNTRY is a region |

And the two land-scale amplitudes stop being a share of the summit and become
the reference's metres. Our `land_scale_m` is 210 against the reference's 40,
so a weight that means 6 m there means 31.5 m here:

| term | reference | ours now | proposed | metres |
| --- | ---: | ---: | ---: | ---: |
| hill weight | 0.15 (6 m) | 0.15 (31.5 m) | **0.0286** | 6 m |
| detail weight | 0.05 (2 m) | 0.05 (10.5 m) | **0.0095** | 2 m |

Dropping those two out of the normalised sum lowers the summit by about a
tenth, so `land_scale_m` is re-measured to hold the owner's 150 m rather than
guessed; the roughness report is the instrument.

**The mountain field is the one judgement call.** Fully metric it would be 40.0
and put 180 m of ridge across a 120 m wavelength, which is a wall rather than a
mountain. Kept angular at 2.5 it is 180 m across 1,920 m, a slope of 0.09
against the reference's 0.27, which is the "smooth" the owner is looking at.
The proposal matches the reference's SLOPE instead: 7.0 puts our taller ridge
over a 686 m wavelength at 0.27, the same steepness on a bigger mountain.

**The moisture field is the other.** Fully metric at 25.6 a biome patch is
188 m, which on this body is about two thousand of them: from orbit that is
speckle rather than regions, and the world stops reading as a map. At 6.4 a
patch is 750 m, so a walk of a few hundred metres crosses biomes while the
globe still shows continents of climate. This is the one place the design
deliberately does not match the reference, and the reason is that the
reference never had to draw a body this size from space.

## What this is expected to do to the three complaints

Predicted, to be checked against the instrument rather than asserted:

- The finest feature in the height field goes from **180 m (64 cells)** to
  about **12 m (4 cells)**, which is the reference's.
- Adjacent cells differing by a block or more goes from **14.2%** toward the
  reference's **44%**, and the mean step from **0.17 m** toward **0.56 m**.
- A river channel goes from a **462 m** estuary to a **29 m** watercourse.
- Biome patches go from **375 m** to **94 m** of finest structure, so the rare
  biomes (swamp 0.1%, mountains 0.5% of the sphere) gain patches and a walk
  crosses several.

## The knobs belong in data

Every figure above is a tuning number, and this change is the second time they
have had to be re-authored. They belong in a validated RON asset beside
`water.ron` and `weather.ron`, loaded at startup, with the code holding the
defaults and a test pinning the file to them - which is the rule this
repository already keeps for every other tunable, and which would have made
this change a file edit and a screenshot instead of a rebuild per candidate.

## What is deliberately not proposed

- **Raising the summit.** The owner fixed it at 150 m. The land-scale terms
  carry their own metres precisely so that the ground can be as rough as the
  reference's without the mountains growing.
- **Changing the continent map.** Its scale does not move, so the coastlines
  the owner has already looked at stay where they are; they gain fine
  structure at their edges, which is what a coastline is.
- **Per-body variation.** One body, one config. A second body is a second
  value of it, which is what the RON asset is for.
