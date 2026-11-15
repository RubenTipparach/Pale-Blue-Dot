# Proposal: steer what you build, sail by the wind, navigate by the map

## Why

The owner, 2026-11-13, on the vehicle mockups (M21 rockets, M23 drydock, M24
hangar):

> as part of this mockup work, Iwant to actually be able to rotate rockets
> (via thrust swivle) adnd control planes via control surfaces, and sail ships
> (via wind currents/be able to gps navigate using the planet map and steer the
> boat accordingly to get through a series of waypoints.
>
> different levels of ships: rowing, sailing, steam turbines, oil (yes add oil
> to the game/energy systems), and eventually fission and battery powered

Today the mockups build craft and show their numbers, but nothing is flown or
sailed in them. The game already has the physics this needs:
- every lifting surface is one foil model (wings, rotors, sails, keels,
  rudders: `openspec/specs/player/vehicles`);
- a sailing boat is driven by the apparent wind on its sail;
- the atmosphere publishes a wind field;
- the map is drawn compass-north up, and the compass bar gives bearings
  (`compass-bar`).

What is missing is how a built craft is steered, the propulsion between
oars and fission, oil, and a way to follow a course.

## What Changes

1. **Rockets turn by swivelling their engines** (thrust vectoring).
   - Each engine gimbals up to 6 degrees.
   - The torque is thrust x sin(gimbal) x the engine's distance from the
     centre of mass, so a long rocket with its engines far aft turns
     readily.
   - Fins add authority in air. Nothing else turns a tier-1 rocket: it has no
     reaction wheels.
2. **Planes fly by control surfaces.**
   - Ailerons on the outer wing cells, an elevator on the tailplane and a
     rudder on the fin.
   - Each surface is the foil it sits on with a deflection of up to 20
     degrees, so its authority grows with airspeed squared and is nothing on
     the ground.
   - The builder's checks add: a plane needs ailerons, an elevator and a
     rudder to be controllable.
   - Helicopters steer by cyclic, collective and tail rotor, as the Kestrel
     already does in the hover.
3. **Ships sail by the wind and are steered to waypoints.**
   - Sails are foils in the apparent wind, as the Tern's and the cog's are,
     read from the atmosphere's wind field. Tacking, beating and running
     follow from it.
   - The player puts waypoints on the planet map. The route is a stored
     record on the craft.
   - A GPS readout gives the bearing and distance to the next waypoint, the
     cross-track error and the craft's course over the ground. The compass
     bar shows the waypoint.
   - The player steers to it. There is no autopilot at this tier.
4. **Six levels of ship power**, as blocks in the hex ship builder:

   | Level | Drive | Fuel |
   | --- | --- | --- |
   | 1 | Rowing | Crew stamina |
   | 2 | Sailing | Wind |
   | 3 | Steam turbine | Wood or coal |
   | 4 | Oil engine | Fuel oil |
   | 5 | Fission | Fuel rods |
   | 6 | Battery and electric motor | Charged at a dock from the power grid |

5. **Oil enters the game.**
   - Oil fields lie under the ground, read from the world's seed as a field,
     not as cells.
   - A pumpjack draws crude from them, and a refinery splits it into fuel
     oil, diesel and polymer.
   - An oil power plant joins the power ladder between coal and wind (M20).
   - Because no cell changes, every world, old saves included, has its oil
     fields without a new generator version.

## Capabilities

### New Capabilities

- `player/craft-control`: rocket gimbals, control surfaces, and helm and
  throttle for ships.
- `player/navigation`: waypoints on the map, the GPS readout, routes as
  records.
- `player/ship-power`: the six levels of ship propulsion.
- `world/oil`: oil fields, the pumpjack, the refinery and the oil plant.

### Modified Capabilities

None until built.

## Impact

- **Mockups**, updated and republished:
  - M21 Rockets and Landers: steer the launch with the gimbals.
  - M24 Hangar and Chibi Aircraft: fly a built plane by its surfaces.
  - M23 Drydock and Hex Ship Builder: the power blocks, and a "Sail it"
    view with wind, the map, waypoints and the GPS readout.
  - M20 Pits, Plants and Power: oil.
- **Saves**: a route and an oil well are stored records. Oil fields are
  derived from the seed and never saved. No cell of an existing world
  changes.
- **No game code** until the owner approves the mockups and asks.
