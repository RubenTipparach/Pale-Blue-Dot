# townsfolk-economy: coins, trade and town jobs in the game (`townsfolk` 5, 6 and 7)

The owner, 2026-11-21: "People approved. Build.phase 3. Do it in parallel if
you can". These are the first frames of the coin count beside the hotbar, the
talk panel, a town trader's panel and a town's job board, in the game's
stone-and-wood kit.

Fixed-step captures in the Claude Code cloud container: **lavapipe, no GPU**,
`--profile fast`, memory-only worlds, a 960 x 600 window. They show the look
only: **frame time was not measured**.

The townsfolk are agent A's work on the same branch and were not in the game
when these were taken, so there is no one standing to talk to. The panels
are opened by a capture instrument, `PBD_TALK=talk|trade|board|hud`
(`desktop/talk.rs`, `capture_open`): once the world is open it stands a
merchant at the nearest held town, puts 48 coins in the purse and three
perch, six wheat and four logs in the pack, and opens that panel. It is never
set in play. The rules the panels show are the game's own
(`pbd_core::economy`), and the trades, jobs and pay are pinned by tests
(`economy::tests` in both crates), not by these frames.

| Shot | Command | What it shows |
| --- | --- | --- |
| `talk.jpg` | `PBD_TALK=talk PBD_WINDOW=960x600 pbd-app --world econ-talk --walk --time 12 --frames 200 --capture talk.png` | The talk panel with Tahal's merchant: the town's name on the lintel, who they are, one plain line, and the buttons they offer (TRADE as the town's trader, WORK as its job giver, BYE). The coin count, 48, sits right of the hotbar. |
| `trade.jpg` | `PBD_TALK=trade`, the same flags (`--world econ-trade`) | Tahal (a desert town) trades: buy on the left, what it makes (stone and fruit at need 0.6 and 0.7) and its bread, each with its stock and price; sell on the right, what the pack holds that it buys: perch at 5, wheat at 2 (it lacks crops, need 1.6), logs at 3. The purse under it. |
| `board.jpg` | `PBD_TALK=board`, the same flags (`--world econ-board`) | Tahal's board for the day: its own job first (a desert town's is a delivery, to Holingmouth), then a second delivery and a hunt (bring 4 wool), each with its reward. |
| `coin-count-close.png` | cropped from `trade.jpg` and scaled 3x (nearest) | The coin beside the hotbar: the gold piece (`assets/items/ui/coin.png`, with the UI's 1 px outline) and the purse's count. |
