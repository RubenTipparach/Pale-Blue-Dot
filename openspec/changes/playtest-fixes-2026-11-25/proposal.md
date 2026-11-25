# Playtest fixes, 2026-11-25

## Why

The owner, playing the merged build (2026-11-25):
- "NPCs dont stop what theyre doing to talk to me, a guy walking continues
  walking away", "need to fix this";
- "picked up a job from an npc, I dont see a place to select that job as
  active mission";
- "if wild plants like glowy mushrooms and stuff are picked off the ground,
  after a while they should grow back";
- "torch on the ground should match my torch I'm carrying".

## What changes

1. A townsperson spoken to stops, turns to the player, and stands until
   the talk ends. Then they carry on from where they stopped.
2. Taken jobs are listed in the Missions screen, under their own JOBS tab.
   One can be made the active mission, and the HUD tracker shows it.
3. A picked wild plant (a wild crop or a glowing mushroom) grows back two
   game days later, out of sight.
4. A placed torch is drawn as the held torch: the same haft, head and
   flame.
