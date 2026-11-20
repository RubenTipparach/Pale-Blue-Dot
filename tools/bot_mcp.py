"""MCP server for the showcase bot's control channel (`bot-control`).

Speaks MCP's stdio transport by hand (JSON-RPC 2.0, one message a line): the
`mcp` package on the owner's desktop does not import (a pydantic clash), and
this needs nothing past the standard library. Each tool sends one command to
the game on 127.0.0.1:47811 (`PBD_BOT_PORT`) and returns its answer as text.

Registered in the repository's `.mcp.json`. The main loop is hot reload: edit
`assets/showcase/<name>.ron` and the running bot plays it at once; these tools
are for a look (`bot_status`) or a nudge when it is stuck.
"""

import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import bot_ctl  # noqa: E402

TOOLS = [
    {
        "name": "bot_status",
        "description": "The running showcase bot in a few lines: shot and act, phase, where it stands, what it aims at, the F prompt, the tool in hand, the focused mission's counts, paused or not.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "bot_do",
        "description": "Run one act now, before the shot's own, written as the shot files write it, e.g. Hand(\"shovel\") or Mine(Have(\"coal\", 6), 12).",
        "inputSchema": {
            "type": "object",
            "properties": {"act": {"type": "string"}},
            "required": ["act"],
        },
    },
    {"name": "bot_skip", "description": "End the current act (or the queued one) as done.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "bot_skip_shot", "description": "End the current shot and go on to the next.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "bot_pause", "description": "Hold the bot: it presses nothing.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "bot_resume", "description": "Let the bot go on; the current act starts over.", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "bot_reload", "description": "Read the shot file again now (it is also reloaded on its own when the file changes).", "inputSchema": {"type": "object", "properties": {}}},
    {
        "name": "bot_still",
        "description": "Save a screenshot of the game window; returns its path.",
        "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}},
    },
]


def call(name, args):
    cmd = name.removeprefix("bot_")
    body = {"cmd": cmd}
    if cmd == "do":
        body["act"] = args.get("act", "")
    if cmd == "still" and args.get("path"):
        body["path"] = args["path"]
    try:
        answer = bot_ctl.send(body)
    except OSError as error:
        return f"no bot listening on 127.0.0.1:{bot_ctl.PORT} ({error})", True
    text = bot_ctl.brief(answer) if cmd == "status" else json.dumps(answer)
    return text, not answer.get("ok", False)


def reply(id_, result=None, error=None):
    message = {"jsonrpc": "2.0", "id": id_}
    if error is not None:
        message["error"] = error
    else:
        message["result"] = result
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        method, id_ = msg.get("method"), msg.get("id")
        if id_ is None:
            continue  # a notification
        if method == "initialize":
            reply(id_, {
                "protocolVersion": msg.get("params", {}).get("protocolVersion", "2024-11-05"),
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "pbd-bot", "version": "1.0"},
            })
        elif method == "tools/list":
            reply(id_, {"tools": TOOLS})
        elif method == "tools/call":
            params = msg.get("params", {})
            text, is_error = call(params.get("name", ""), params.get("arguments") or {})
            reply(id_, {"content": [{"type": "text", "text": text}], "isError": is_error})
        elif method == "ping":
            reply(id_, {})
        else:
            reply(id_, error={"code": -32601, "message": f"no method {method}"})


if __name__ == "__main__":
    main()
