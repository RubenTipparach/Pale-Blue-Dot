# Proposal: a town's gate passage is walled on both sides

## Why

The owner, 2026-10-08: "walking through gates in cities is busted that needs
to be fixed".

The collision through a gate is sound. `the_walled_towns_wall_stands_and_its_gates_open`
and `a_walker_goes_through_a_gate_and_up_a_tower_onto_the_walk` pass. The gate
is two cells wide, about 5.67 m between the wall cells beside it, and 4 m high,
against a walker 0.6 m across and 1.8 m tall.

What the walker sees is not what it walks against. `cut_masonry`
(`pbd-core/src/settlement/pieces.rs:1155-1158`) draws no side where another
masonry cell stands against it, and `build_town` (`record.rs:976, 985`)
counts a gate cell as masonry. So the side of a wall cell that faces into the
gate passage is never drawn. A gate cell's vault starts 4 m over its ground,
and between that ground and the vault the passage's two side walls are missing.
Town faces are back-face culled. From inside the passage you look into the
wall cells beside it and see through them, while their invisible solids still
stop you.

In the shipped walled template (`assets/settlements/v1/town.json`):

| Gate | Gate cells, ground and vault | Wall cells beside it, ground |
| --- | --- | --- |
| North | (19,6) and (20,6): ground 0, vault from 4 | (18,6) and (21,6): ground 1 |
| South | (19,31) and (20,31): ground 3, vault from 7 | (18,31) and (21,31): ground 3 |

The tundra camp's ice gate, (31,16) in `tundra.json`, is cut by the same code
and has the same hole.

A second, smaller finding: just inside the north gate, (19,7) and (20,7) are
at ground 0 and the next row is at 1. The mockup's "Wall lane, steps" become a
1 m riser, climbable only because the step allowance is 1.05 m (3.5 cm to
spare), and only while the walker is grounded.

On the trial town walk (`town-frame-time`), the scripted walker from
Ashenstead's middle stalled twice (`WALK_TURN stalled 7 s` at 7 m and at 41 m).
Where it stalled is not yet known.

## What

1. Draw each masonry cell's side over the part of its height that its
   neighbour does not fill. Beside a gate that runs from the wall cell's
   bottom to the gate's vault, so the passage is walled on both sides as the
   mockup draws it (`docs/mockups/towns.html`, which gives every wall cell a
   full prism).
2. A test that the faces of every gate passage cover it: for each gate edge,
   the side drawn on the wall cell spans the gate's open height.
3. Before-and-after shots in the passage of a walled town and a tundra camp,
   for the owner.
4. The riser inside the north gate is recorded here. A stair would change the
   template's look, so it is asked about with shots, not made. **Answered
   (G1), by the owner's rule of 2026-11-14:** "where theres a step on a path
   there should be staris". Built in `path-stairs`.

## Not in this change

- Saved worlds are untouched. Masonry geometry is derived from the template
  and the town's stored layout, and is never saved. The v1 template file does
  not change.
- Collision does not change: the solids are already right.

## Seen (2026-10-08)

Ashenstead's north gate, before and after, from the spots
`towns::tests::walled_town_gate_spots` prints (10:00, rain):
`docs/screenshots/town-gates-{before,after}-ashenstead-north-{outside,passage,side}.png`.
Before, the side shot looks along a hollow corridor inside the wall cell, out
to the trees beyond. After, the same shot shows the wall's face. The outside
shot shows the passage walled on both sides up to the vault. The passage shot
shows the 1 m riser of bare earth just inside the gate. That is item 4, left
for the owner.

`a_gate_passage_is_walled_on_both_sides` finds no wall face beside
(19, 6) when the old behaviour is put back, and passes with the change.
