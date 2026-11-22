# townsfolk-economy: talking, trade and the job board with a real townsperson (`townsfolk` 5 to 7, design 35)

Fixed-step captures in the Claude Code cloud container: **lavapipe, no GPU**,
`--profile fast`, new memory-only worlds, a 1200 x 750 window. They show the
look only: **frame time was not measured**. A capture skips the loading
screen's warm-up, so each logs its pipelines as `LATE_PIPELINE` by design.

Taken by `tools/capture_talk.sh` (2026-11-22). They replace the frames of
2026-11-21, which were posed by the `PBD_TALK` instrument before the
townsfolk stood (that instrument is removed). **The one thing set is where
the walker stands**: face to face with Linenleigh's merchant (its trader)
and its reeve (its job giver), 1.6 m off, from the spots
`towns::tests::print_where_to_stand_for_the_townsfolk_shots` prints. The
rest is played: `PBD_KEY_SCRIPT` presses F on frame 120 to talk, and a
`press:<button>` step presses the talk panel's TRADE or WORK button as a
click on it would. The pack and purse are a new world's: empty, 0 coins.

| Shot | What it shows |
| --- | --- |
| `talk.jpg` | F on Linenleigh's merchant, who stands in the town's market hall: the talk panel (the town's name on the lintel, MERCHANT, "Want to trade?", TRADE and BYE). The merchant's mouth is open: a townsperson talks while the panel is open with them (design 30). The coin count, 0, right of the hotbar. |
| `trade.jpg` | The same, then TRADE pressed: what Linenleigh sells (rope, its dishes and the wooden bucket, sixteen of each, at its prices) and, to sell, nothing: the pack is empty. The purse, 0, and BACK. |
| `board.jpg` | F on the reeve (standing in a lane's shade), then WORK pressed: Linenleigh's board for the day, a walled town's own job first (cook a grilled fish), then bring four bronze ingots and cook a flatbread, each with its reward and TAKE. |

What is wrong in them, honestly:

- The merchant stands inside the market hall, so `talk.jpg` and `trade.jpg`
  are dim: the hall is lit as the field lights it, with no lamp near.
- The talk panel sits over the townsperson's head and covers the top of it.
- The pack is empty, so the trade's sell column has nothing in it and no
  coins change hands: task 5.2's "coins in the purse" is not shown by a
  played capture yet.
- A board's line reads "Bring 4 bronze ingot": the job lines do not take a
  plural yet.
- These are stills; the mouth's movement is only seen as one open frame.
