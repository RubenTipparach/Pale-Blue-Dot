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

## The icons in the pack's slot well (design 38a, 2026-11-22)

`trade-wells.jpg` and `board-wells.jpg` are `trade` and `board` retaken
(`tools/capture_talk.sh`, `ONLY=trade` and `ONLY=board`; lavapipe, frame time
not measured) after the owner's "you need proper icons for trade": every
trade row's good and every job's good now sits in the pack's own 24 px stone
slot well, its 16 px icon on the well's floor, as in the pack and the hotbar
(the board's rows had no icon before). The panels are wider by the wells
(trade 490, board 450 native px). `talk.jpg`, `trade.jpg` and `board.jpg`
stay as the frames the owner approved.

## Long lists scroll, and each column filters (design 38b and 38c, 2026-11-22)

`trade-scroll-top.jpg`, `trade-scroll-partway.jpg` and `trade-filter-food.jpg`
are `tools/capture_talk.sh` with `ONLY=scroll` (lavapipe, frame time not
measured), after the owner's "opt for a scroll bar incase theres lots of
products" and "some filtering on each side would be nice: food, organics,
raw materials, tools, etc" (the plan approved: "approved"). Played as
before: F on Linenleigh's merchant, TRADE pressed; a `wheel:buy:-1` step
turns the wheel one notch over the BUY column as the wheel would, and
`press:buy-food` presses the BUY column's Food tab.


| Shot | What it shows |
| --- | --- |
| `trade-scroll-top.jpg` | Linenleigh's merchant sells nine goods; the BUY column shows seven (rope to fried egg) in its fixed window, with the stone scroll bar beside it, its thumb seven ninths of the track at the top. Above the list, ALL (chosen), Food and Goods: the merchant's goods stand on those two shelves. The SELL column has nothing (a new world's pack is empty), so no tabs and no bar, but the same height: "Nothing they want" lines up with the first row. PURSE and BACK under both. |
| `trade-scroll-partway.jpg` | The same, one wheel notch down over BUY: grilled fish to roast meat, the rope gone off the top, the thumb moved down; the panel the same height. |
| `trade-filter-food.jpg` | The Food tab pressed: the seven dishes, from the top; seven rows fit the window, so there is no bar. |

Not in these frames: the job board's scroll (Linenleigh's board posts three
jobs, which fit; the mockup's `v13-board-*` shows it overflowing), a drag of
the thumb (a capture has no pointer drag; the mockup's
`v13-trade-dragged-night` shows it), and the tooltip (a capture holds the
pointer, so nothing is hovered). The thumb is 4 px wide at 1x, a thin bar
at this scale.
