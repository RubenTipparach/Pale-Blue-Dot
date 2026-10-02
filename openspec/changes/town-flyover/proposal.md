# Proposal: one continuous flyover that tours every kind of town

## Why

The owner (2026-10-02): "start work on designing a flyover camera animation
for each new city, one continuous shot, tour each new city type in the game
let me know how long this shot takes, fps, etc, before committing to record
it."

Five kinds of town are built into the game (the `cities-in-the-world` and
`tenebris-towns` work): the **village**, the **walled town**, the
**harbour**, the **desert town** and the **tundra camp**. Jungle, swamp, cliff
and cave sites are placed on the map but have no template, so nothing stands
there to film. No route visits a town today: `--route far-side|scenic|clouds`
each fly one great circle or one terrain-following arc, and the only way to a
town is `--at lat lon`.

## What changes

A new route, `--route towns`: a scripted camera flight, one shot from start to
finish, that visits the nearest town of each built kind in the order that
makes the shortest tour, sweeps an arc round each, and flies low between them.

- **Which towns.** Chosen in the world from its stored site list, so it works
  in any world: one town of each built kind, the set and order with the
  shortest total flight (all 4,320 sets and orders are scored; it is
  instant). On the shipped seed (generator 6) that is **Coringport**
  (harbour) to **Mirhan** (desert) to **Dunmouth** (village) to **Ashenstead**
  (walled town) to **Ulvevik** (tundra camp): legs of 1,188, 622, 865 and
  5,568 m, 8.2 km over the ground.
- **The shot.** At each town the camera descends to about 70 m and sweeps half
  a circle round it, about 130 m out, looking at the town's centre, so each
  town is seen from the side it is approached on round to the side it is left
  by. Between towns it climbs, cruises and descends along one smooth path,
  higher and faster on a longer leg. The camera's path, aim and speed are all
  one spline in time: no cut, no stop, no hard turn.
- **Measured before it is recorded.** The route quits when it ends, like the
  other routes, so `--frame-log` measures it in real time; the duration and
  frame times are reported to the owner before anything is recorded.

## Not in this change

- Recording it (`obs-record`): only after the owner has the numbers and says
  go.
- Building the unbuilt kinds (jungle, swamp, cliff, cave).
