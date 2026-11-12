# Proposal: frame time back above 60 in towns and through weather changes

## Why

The owner, 2026-10-08: "fps is sub par with the city and weather changes to
day, so we need to get it back up to 60+ again", and "monitor for weather
stutters or abrupt changes in clouds. I want my fps hitting the hundreds
again".

The last performance suite was run on 2026-09-26. Since then sun shadows
(`sun-shadows`, 09-30), towns with field-baked light (`cities-in-the-world`,
`tenebris-towns`, 09-29 to 10-07), the compass bar and smooth weather
(`compass-bar`, `smooth-weather`, 10-08) have landed, and none has been
benchmarked. The suite has no scenario that stands in a town, so the case the
owner reports is the one it cannot see.

This change is a measurement first. Nothing below is measured yet: it is a
read of the code, ranked by how likely each item is to cost frames. The
numbers decide which fixes are made.

## What

1. **A town scenario in the suite** (`town`): walk through a walled town by
   day with the weather made to change during the run, so the city and the
   weather changes the owner names are on screen together.
2. **Instruments, and nothing else, first** (see the design): `SPENT` timing
   around the main-thread suspects, a log line per weather publish, and
   weather/bake columns in the frame log, so a slow frame can be lined up
   with what ran in it.
3. **A before/after sitting** against the 2026-09-25 baseline exe, the last
   build the owner saw "in the hundreds", on `town`, `clouds`, `storm` and
   `walk`, with `--no-shadows` and `PBD_NO_CLOUDS=1` as variants to price
   those two.
4. **Fixes, chosen by the numbers**, written into the design before code. The
   candidates are listed in the design with what each would cost to fix.

## Not in this change

- No tuning constant, shader term or default moves before the numbers exist.
- Nothing about how towns, clouds or weather look. A fix that would change a
  picture is written up and shown with before-and-after shots first.
- Saved worlds are untouched: none of this changes generation or the save.
