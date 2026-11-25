# Bot control: design

## 1. The channel

A thread started with the showcase binds `127.0.0.1:<port>` and accepts one
client at a time. Each line it reads is a JSON object `{"cmd": ..., ...}`;
it passes the command, with a one-shot reply sender, to the game over a
`std::sync::mpsc` channel and writes the reply line back. The game drains the
channel once a frame in the bot's own system, so a command acts on the bot
between frames, as a key press would: never from the thread.

The bind failing (the port taken) is logged and the run goes on without it.

## 2. Commands

| `cmd` | Fields | Does | Reply |
| --- | --- | --- | --- |
| `status` | | | shot, act and its RON, phase, seconds in each; feet lat/lon and height; aimed cell; F prompt; tool in hand; screen; the HUD tracker's mission and counts; paused |
| `do` | `act`: RON text | parses it as an `Act` and runs it before the shot's current act; a parse error is the reply | `queued` or the error |
| `skip` | | ends the current act as done | the next act |
| `skip_shot` | | ends the shot | the next shot |
| `pause` / `resume` | | paused, the bot presses nothing and every held key comes up | |
| `still` | `path` (optional) | a window screenshot | its path |
| `reload` | | the shot list read again from its file; the bot's memory kept, the same shot and act numbers | the shot count, or the parse error |

## 3. MCP

`tools/bot_mcp.py` implements MCP's stdio transport by hand (JSON-RPC 2.0:
`initialize`, `notifications/initialized`, `tools/list`, `tools/call`), since
the `mcp` package on the owner's desktop fails to import (a pydantic clash).
Each tool opens the socket, sends one command and returns the reply as text.
A game not running is a tool error saying so.

## 4. Safety

Local only (`127.0.0.1`), showcase runs only, and every command acts through
the bot, which acts only through the player's inputs: the channel sets no
item, count, mission or world state.
