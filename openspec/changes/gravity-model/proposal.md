# Proposal: adopt Tenebris's two-field gravity model

## Why

Three findings, measured off both trees. See the gravity section of
`docs/tenebris-comparison.md` for the full table.

**One field cannot do two jobs.** Tenebris runs two: `gravity_at` picks a single
anchor body with a hard edge (full pull inside `1.4 R`, linear taper to zero by
`1.8 R`, nothing beyond), and that absence IS its `is_in_space` flag - it drives
the surface basis, the atmosphere shell, the walker's down, and the flip into
6DOF flight. `orbital_gravity_at` sums true inverse-square pull from every body
plus a central star, uncapped, which is what lets a prograde burn raise a real
transfer arc. We have one uncapped inverse-square well, so there is no radius at
which the field itself says "you are in space now", and the walking/flight
handoff has to be decided by something else that can drift out of step with it.

**The surface constant is not Earth's and ours is not deliberate.**
`SURFACE_GRAVITY_MPS2_PER_G` is **25.0** there, arcade-scaled to ~300 m planets,
with the orbital field anchored to the same value so space and surface agree at
the surface - one shared constant, and its comment says that is the point. Ours
is 9.0 on a body thirteen times larger, and nothing records why.

**The walker floats, independently of the tile size.** Both projects jump at
12 m/s. At 25 m/s^2 that is a 2.88 m apex clearing 2.9 one-metre blocks; at
9.0 m/s^2 it is 8.00 m clearing 1.3 six-metre steps. Dropping one step height
takes 0.28 s there and **1.15 s** here. Fixing the hex scale alone leaves this:
9.0 m/s^2 against a 6 m step is slow whatever the tiles do.

## What changes

- A second, mode-anchoring field beside the inverse-square reference, with the
  band shape and strongest-multiplier selection Tenebris uses. Walker, assisted
  ship, and hover compensation use the anchor field; the reference is not
  summed into gameplay ship motion.
- The space transition becomes a property of that field rather than a separate
  decision.
- The surface constant becomes a declared value with a recorded reason, shared
  by both fields so they agree at the surface by construction.

## Decided: `tenebris-rs` is the definitive spec for gravity

The owner's call, on the same terms as the hex size. The standard is therefore:

| Quantity | Value |
| --- | ---: |
| Surface gravity at 1 g | **25.0 m/s^2** |
| Full pull out to | **1.4 R** |
| Linear taper to zero by | **1.8 R** |
| Fields | **two**, anchored to one shared surface constant |

So 9.0 m/s^2 goes. A one-metre fall takes Tenebris's approximately 0.28 s;
the current preview's six-metre step falls in approximately 0.69 s instead of
1.15 s. Bringing the step itself down to one metre belongs to the separate
scale change.

The constant is taken as-is rather than retuned for a larger body. Its own
comment ties 25 to a ~300 m planet, which is an argument for retuning it on a
4 km one - but gravity is felt at human scale, not planet scale, and a walker
who steps off a ledge should fall the same way on every body for the same reason
a hex is the same size on every body. A body that wants to feel different says
so with `gravity_g`, which is what that field is for.

## Non-goals

- Patched conics, maneuver nodes, transfer-window planning or n-body ship
  simulation. The inverse-square reference does not add a second ship force.
- Re-tuning jump, walk and sprint. Those are the avatar's, and they are shared
  with the scale question in `preview-scale-and-shader-parity`.
- Multi-body scenes. The selection rule is worth having before there are two
  bodies to select between, but nothing here adds one.
