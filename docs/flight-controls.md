# Walking and flying the desktop planet preview

The explorer starts on dry land in **first-person walking mode**. Left-click
the window to capture the cursor, use the mouse to look, and use WASD to walk.
Press F to switch between walking and assisted flight. The planet is 8 km in
diameter, with a closed spherical hex surface including twelve pentagons.

```powershell
.\run.bat
.\run.bat --walk
.\run.bat --fly
```

## Camera and controls

Mouse look applies the current frame's raw motion at **0.002 radians per pixel**.
There is no camera easing, interpolation delay, or rotational smoothing. The
camera turns immediately; flying retains the ship's bounded physical rotation
independently of the view. Pixel textures use explicit nearest/point filtering,
and terrain atlas sampling uses integer texel loads.

Interactive presentation uses VSync with a requested maximum frame latency of
one. There is no additional frame-pacing sleep. Captures disable VSync so their
timings remain uncapped; the queue setting is not a measured latency guarantee.

| Input | Walking | Flying |
| --- | --- | --- |
| Left mouse button | Capture cursor | Capture cursor |
| Mouse | Immediate look; planet up, pitch limited to +/-89 degrees | Immediate free look, including through the poles |
| W / S | Walk forward / backward along the ground | Thrust forward / backward relative to the view |
| A / D | Strafe left / right along the ground | Thrust left / right relative to the view |
| Either Shift | Sprint | Select the 600 m/s cruise target while held |
| Space | Jump | Thrust along view up |
| Either Ctrl | No walking action | Thrust along view down |
| Q / E | No walking action | Roll left / right |
| F | Switch to flight | Switch to walking |
| X | No walking action | Toggle inertial damping and gravity compensation |
| Hold B | No walking action | Brake and compensate gravity |
| R | Reset the active mode to its starting pose | Reset the active mode to its starting pose |
| Esc | Release cursor and stop walking input | Release cursor and apply protective braking/hover assistance |
| F12 | Save a screenshot under `output/captures/` | Save a screenshot under `output/captures/` |

Losing window focus also releases control. Esc leaves the application open;
left-click again to resume. Diagonal movement is normalized. Walking follows
the planet's local up direction, so looking up or down does not redirect WASD
into the air or ground. Flying movement follows the immediate view, including
its roll. X affects translational assistance; physical rotational stabilization
remains enabled.

F is a **creative movement-mode switch**, not boarding or exiting a modeled
vehicle. Switching from flight to walking places the player on the dry ground
below, or nearby dry land when over water. Switching modes retains the ship
entity. Inventory, ownership, a ship interior, and an interaction-based boarding
system remain future work.

## Walking and ground contact

| Quantity | Current value |
| --- | ---: |
| Walking speed | 8 m/s |
| Sprint speed | 14 m/s |
| Eye height | 1.6 m |
| Automatic step height | 0.6 m |
| Jump velocity | 12 m/s |
| Surface gravity | 9 m/s^2, decreasing with inverse-square distance |

At 9 m/s^2, a 12 m/s jump reaches approximately 8 m above its launch point on
level ground. The current terrain has roughly 19 m cell spacing and 6 m height
steps: large terraces require a jump, or F to fly. A walking character cannot
automatically step up an entire terrace.

Ground contact queries the exact current rendered cell caps across the capsule
footprint from the CPU's authoritative terrain and topology. Avian integrates
movement, with a custom swept radial contact pass for the surface and step
handling. This is a surface controller, not a complete collider representation
for caves, overhangs, trees,
or editable blocks. Water blocks ground entry while swimming is unimplemented;
switch to flight to cross an ocean. The default spawn selects dry land.

## Flight limits and terrain protection

| Quantity | Current value |
| --- | ---: |
| Normal flight target speed | 120 m/s |
| Cruise target / absolute safety speed | 600 m/s |
| Controlled linear acceleration | 80 m/s^2 |
| Physical angular speed | 1.5 rad/s, about 86 degrees/s |
| Controlled angular acceleration | 3 rad/s^2 |
| Physics frequency | 60 Hz, with four Avian substeps |
| Manual flight clearance | Approximately 1.6 m above terrain or ocean |
| Automated tour clearance | Approximately 45 m above terrain or ocean |

These angular limits apply to the physical ship; they do not slow mouse look.
The 80 m/s^2 acceleration budget overrides the original design document's
20 m/s^2 tuning proposal so the fast circumnavigation demonstration is feasible.

Releasing Shift while above 120 m/s requests bounded braking toward the lower
target rather than snapping velocity. Ideal braking from 600 m/s at 80 m/s^2
takes 7.5 seconds and 2.25 km; stopping from 120 m/s takes 90 m. Ordinary damping
slows gradually near rest. B requests a firmer stop within the same acceleration
budget.

With inertial assistance enabled, gravity compensation lets a released ship
hover. With it disabled, gravity and momentum remain active. Speed and
acceleration limits remain enabled in either case. Ships use local gravity and
Newtonian motion; orbital insertion and rocket transfer mechanics are absent.

A protective flight sweep samples the travelled segment against the terrain
and ocean clearance shell. If necessary, it moves the ship to a safe radius and
removes inward radial velocity. These recorded safety corrections are separate
from bounded thruster acceleration and may redirect a low pass. They are not
landing or detailed terrain-collision mechanics, and their capped sampling is
not a guarantee for arbitrary teleports or unbounded simulation steps.

## Tours, captures, and verification

```powershell
.\run.bat --tour
.\run.bat --tour --view pole
.\run.bat --verify-flight
.\run.bat --verify-flight --view pole
.\run.bat --capture output/captures/walk.png --walk --frames 180
.\run.bat --capture output/captures/orbit.png --view orbit --frames 180
.\run.bat --tour --fixed-dt --capture output/captures/tour.png --frames 1800
.\run.bat --capture output/captures/tern.png --aboard tern --frames 240
```

`--walk` explicitly selects the default walking mode and captures its ground
spawn when combined with `--capture`. `--fly` starts manual flight and captures
its flight spawn when combined with `--capture`. Without `--walk`, `--fly`, or
`--tour`, a capture uses a static photograph; its available `--view` values are
`orbit`, `coast`, `surface`, `night`, and `pole`. `--aboard kestrel`, `tern` or
`loon` walks, then boards that craft as soon as the world's craft are placed and
captures its chase view; add `--seat` for the view from the seat.

The ordinary flight tour follows a great circle reaching approximately
28.6 degrees north and south. The polar tour follows a meridian through both
poles. Both request approximately 1,000 m sea-level altitude and 600 m/s cruise
through the same `ShipController` and Avian integration used by manual flight.
Guidance supplies tangential thrust, centripetal acceleration, and altitude
correction; it does not move the ship by assigning successive orbit positions.

Completion requires at least 360 degrees of accumulated changes in the actual
physical position direction. The graphical tour continues after its first lap;
the headless verification prints a report and stops. Static photos are not
evidence of a completed flight. Reports record duration, angle, clearance,
speed, controlled acceleration, angular motion, and safety corrections.

The [validation record](validation.md) contains executed checks and remaining
limitations. The controls above describe the current implementation contract;
they do not establish that a new build, manual input check, or performance
measurement has passed. Use the [performance harness](performance-harness.md)
for comparable release measurements rather than screenshot timing alone.

