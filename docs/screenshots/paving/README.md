# Marking and paving: captures

For `openspec/changes/townsfolk` (design 39 to 41, 43 to 45, tasks 13.3 to 13.8).
The models are textured in the towns' pixel textures (timber, plank, barrel,
iron, stone, cloth), as the owner asked on 2026-11-23.
Captured 2026-11-23 in a Claude Code cloud container with no GPU: software
rendering (lavapipe under Xvfb), fixed-step `--capture` runs of the fast
build at 1440x900, each on a new world. They show what things look like,
never how fast; frame cost was not measured.

`--paving [metres]` is the capture instrument: once the ground round the
player is in, it sets a grinder that far ahead to the left and a mixer to
the right through the edit path, drives three survey stakes 9 m apart along
the view from 4 m beyond them through `roads::paving::drive_stake` (the
place button's path), and paves the plan's first six cells through
`roads::paving::pave` (the interact key's path), one gravel each. The rest
of the plan stays marked.

`tools/capture_paving.sh docs/screenshots/paving <exe>` takes them all as PNG; they are
kept as JPEG at quality 88:

| file | flags | what it shows |
| --- | --- | --- |
| `paving-day.jpg` | `--paving 5 --pitch -20 --time 11` | The plan along the view: the paved gravel cells, cut by the world road's rule, the marked cells beyond with the lime band along the plan's line, and the pegs with their cloth ties. The grinder and the mixer stand at the edges. |
| `paving-2230.jpg` | the same at 22:30 | At night the gravel, the band and the pegs are as dark as the ground round them: no lamp near. |
| `marks-close.jpg` | `--paving 0.5 --pitch -25 --time 11` | The paved run close: gravel cells cut by the world road's rule, turning with the plan, and the first stake, a peg of split timber with its frayed head and a tie of the towns' cloth; the marked cells go on past it with their lime band. |
| `stations-day.jpg` | `--paving 7 --pitch -2 --time 11` | The grinder (left: splayed posts, hopper, two stone rollers, crank, apron) and the mixer (right: a bellied drum with iron hoops on two A-frames, crank, trough), each on its timber block, lit by the sun with the ground's shadows. |
| `stations-2230.jpg` | the same at 22:30 | The stations at night, dark as the ground. |

The spawn the shipped seed gives `--spawn fields` is on desert sand here, so
the band is pale on pale; on grass it reads more strongly.
