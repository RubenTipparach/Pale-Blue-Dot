# Tasks

## 1. Coverage that tracks the cover
- [ ] Equalize the column's shape before the remap; `tools/drawn_cover.py` updated
      to the new shader and showing the drawn share within 0.05 of the cover.
- [ ] Finer peaks at low cover (cumulus-sized), without the deck floor there.

## 2. Fair-weather cover
- [ ] `fair_cover_max` and `fair_rh` knobs; `cover` takes the largest of the
      three; nothing condenses and nothing rains from it.
- [ ] Tests: fair humid air is thinly cloudy and dry; the water budget is
      unchanged; the sea stays cloudier than the land.

## 3. Check
- [ ] Climate report (histogram, raining share, sea against land) at both
      solstices; orbit captures before and after.
- [ ] fmt, clippy, tests, `openspec validate --all`; owner's look in game.
