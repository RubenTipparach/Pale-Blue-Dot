# Design: fishing anywhere there is water

## 1. One rule for water: the column's

`PlanetWater::surface(point)` reads the column under the point:
- the ground there (the bed rule fishing already memoises);
- whether the column holds water just over that ground
  (`PlanetContact::column_water`).

Water whose top stands more than 0.25 m over the sea's own cells' top
(altitude 0, the sea layer's bottom) is fresh: the surface is its top,
which is where the river mesher draws it. Otherwise it is the sea's drawn
sheet as before, with the swell and the tide. This is the same rule the
small boats float by (`vehicles::fresh_water`, `small-boats` design 2b).

Off the column tier the height field answers as before, so far casts are
unchanged.

## 2. Schools in fresh water

`try_spawn` asks the same column at the place it picked.
- **In fresh water** the class is `River`, the depth is the water's own
  (top less ground), and the school is anchored at the water's top.
- **Otherwise** the sea's classes stand as before.

The species filters (`lives_in`, `fits(depth)`, temperature) are unchanged,
so a shallow pond only holds what lives in shallow river water.

## 3. Smaller water, smaller fish

`School::scale` (1 by default) multiplies `length_cm`, so the catch, the
field guide and the save all see the smaller fish. The app sets it when a
school spawns in fresh water:
- **reach**: the furthest of 3, 6, 12 and 24 m round the anchor at which at
  least half of eight bearings are still water;
- **depth**: the water's depth at the anchor;
- **scale** = 0.35 + 0.65 x min(reach / 24 m, depth / 3 m), clamped to
  0.35 to 1.

A puddle's fish are a third of their species' length; a river 24 m across
and 3 m deep has full-size fish. Sea schools keep 1. The drawn fish use the
same scale.

## 4. A jingle on the hook, a chime on the catch

The owner (2026-11-17): "make sure you can actually catch a fish and theres
some sort of jingle when a fish is on the hook", "and another jingle if you
successfuly caught it".

- **Catching works.** The core holds that a patient reel lands the fish
  that bit (`fishing::tests::a_patient_reel_lands_the_fish_that_bit`). In
  the world a catch failed in rivers because the float found no water there
  (sections 1 and 2).
- **The hook.** `Event::Hooked` played nothing. It now plays `hook_jingle`:
  a quick bright lift of two mallet notes with a shimmer, synthesised in
  `tools/audio/gen_sfx.py` beside the catch chime, on the menu bus.
- **The catch.** `Event::Caught` keeps `catch_chime`, three rising mallet
  notes and a ring, and the splash.

## 5. Not measured here

Captures on lavapipe in a cloud container; frame cost not measured. The
surface read is one column lookup over the memoised bed, about what the bed
read already costs.

## 6. Swamps, lakes and rivers are fresh, whatever their height (owner, 2026-11-23)

The owner: "swamps, lakes and rivers should all be fresh water" (on the
bot's two days at "Sea fishing"'s fresh half: "posible that to code
doesnt understand fresh vs sea water perhaps?").

**How the game tells them apart.** Nothing marks water as sea or fresh; it
is judged by height, in three places with three lines:
- the catch, for the mission (`fish::is_fresh`): the float more than
  0.6 m over sea level;
- the schools (`fish::fresh_top`, design 1): the column's water more than
  0.25 m over the sea's cells;
- the bot's search (`showcase::fresh_shore`): over 0.6 m, and over 1.2 m
  deep.

**Measured** (`planet_gen::tests::swamps_and_rivers_are_fresh_and_the_sea_is_not`,
generator 16, 200,000 directions): 495 swamp pools and 897 river cells, and
none of either stands under the 0.6 m line. So the game already counts
every swamp, lake and river as fresh, as the owner asks. What failed was
the bot: every swamp pool is one layer deep (1 m), as are 150 of the 897
river cells, and the bot wanted 1.2 m, so it passed every swamp for a lake
240 to 350 m away and was lost on the way there.

**The fix.**
- The bot's `fresh_shore` takes water one layer deep (over 0.9 m); river
  fish live from 0.2 to 0.3 m down, so a pool holds them.
- `planet_gen::fresh_water_at(cfg, direction)` names what the generator
  makes fresh (a river's water, a swamp's pool; never in a town), and the
  test above holds that all of it stands over the catch's line. If a later
  generator puts fresh water at the sea's level, the test fails, and the
  catch and the schools must then ask `fresh_water_at` rather than the
  height. They do not today: asked at every sea point it would cost time on
  every float's surface read for no change.

What the sea floods into a hole dug at the shore stands at the sea's level
and stays sea. Poured water over the sea's level stays fresh.
