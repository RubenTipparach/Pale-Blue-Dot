# Bot control: drive the showcase bot live

## Why

The owner, 2026-11-19, after a day of playtesting the mission tree with the
showcase bot: "do youthink theres a way for you to update the bot in realtime
via MCP SERVER?", "this iteration loop is kind of annoying for bot logic", and,
offered a hot-reload, a live control channel or scripted bot logic, "ok do mcp
for bot control" and "drive the bot".

Each stall cost a stop, a rebuild (30 s to 6 min), a resume that lost the
bot's memory (its field, its forge, its home) and a replay of acts already
done. Most stalls were a situation, not a bug in an act: in the sea, under
rock, at a bluff, after a resume with the wrong goal. Seen and nudged live,
the run goes on.

## What changes

- **A control channel in the game.** A `--showcase` run listens on
  `127.0.0.1` (port 47811, `PBD_BOT_PORT` to change it) for one-line JSON
  commands and answers each with one line of JSON. Nothing listens in a plain
  run, and nothing listens beyond the local machine.
- **Commands:** `status` (shot, act, phase, where it stands, what it aims at,
  the F prompt, what is in hand, the screen, the focused mission's line);
  `do` (run one act now, written as the shot files write it, before the
  shot's own); `skip` (the act) and `skip_shot`; `pause` and `resume` (the
  bot lets go of every key); `still` (a window screenshot, its path
  returned); `reload` (the shot list read again from disk, the bot keeping
  its memory and its place).
- **An MCP server**, `tools/bot_mcp.py`, with no dependencies, speaking the
  protocol over stdio and passing each tool to the channel: `bot_status`,
  `bot_do`, `bot_skip`, `bot_pause`, `bot_resume`, `bot_still`,
  `bot_reload`. Registered in the project's `.mcp.json`.
- **A command line** on the same channel, `tools/bot_ctl.py`, for a session
  that has not loaded the MCP server.
- **Hot reload, the main loop.** The owner: "I dont want you to use too many
  tokens driving it via mcp, maybe some sort of hot reload path for the bot".
  A shot file edited while the run plays is read again within a second and
  the current shot starts over on it, the bot keeping its memory: a fix to
  the bot's plan lands without a command, a rebuild or a restart. The tools
  are for the rare look or nudge, and `bot_status` answers in a few lines.

## Not in this change

- The bot planning from a mission's open objectives (next).
- Bot logic as a hot-loaded script.
- Changing tunables over the channel.
