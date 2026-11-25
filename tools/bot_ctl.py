"""Talk to a running showcase bot (`openspec/changes/bot-control`).

    python tools/bot_ctl.py status
    python tools/bot_ctl.py do 'Hand("shovel")'
    python tools/bot_ctl.py skip | skip_shot | pause | resume | reload
    python tools/bot_ctl.py still [path]

The game listens on 127.0.0.1:47811 (`PBD_BOT_PORT`) while a `--showcase`
run plays. A shot file edited during the run is played at once without any
command (hot reload); these are for a look or a nudge.
"""

import json
import os
import socket
import sys

PORT = int(os.environ.get("PBD_BOT_PORT", "47811"))


def send(body, timeout=25.0):
    """One command, one answer (a dict)."""
    with socket.create_connection(("127.0.0.1", PORT), timeout=timeout) as sock:
        sock.sendall((json.dumps(body) + "\n").encode())
        data = b""
        while not data.endswith(b"\n"):
            chunk = sock.recv(65536)
            if not chunk:
                break
            data += chunk
    return json.loads(data.decode() or "{}")


def brief(status):
    """The status in a few lines: what a look needs, no more."""
    if not status.get("ok"):
        return json.dumps(status)
    m = status.get("mission") or {}
    acts = status.get("acts") or []
    act = acts[status["act"] - 1] if 0 < status["act"] <= len(acts) else "(end of shot)"
    lines = [
        f"shot {status['shot']}/{status['shots']} {status.get('caption')!r}, act {status['act']}/{len(acts)}: {act}"
        + (" [PAUSED]" if status.get("paused") else ""),
        f"  phase {status['phase']} ({status['phase_s']:.1f} s), act {status['act_s']:.0f} s, shot {status['shot_s']:.0f} s",
        f"  at {status['lat']:.4f} {status['lon']:.4f}, {status['height_m']:.1f} m; {status['screen']}; hour {status['hour']:.1f}",
        f"  aim {status.get('aim')} {status.get('aim_material')}; F {status.get('f')}; in hand {status['in_hand']}; selected {status.get('selected')}",
        f"  mission {m.get('title')} {m.get('counts')} of {m.get('of')}",
    ]
    if status.get("injected"):
        lines.append(f"  queued: {status['injected']}")
    return "\n".join(lines)


def main(argv):
    if not argv:
        print(__doc__)
        return 2
    cmd, rest = argv[0], argv[1:]
    body = {"cmd": cmd}
    if cmd == "do":
        body["act"] = " ".join(rest)
    elif cmd == "still" and rest:
        body["path"] = rest[0]
    try:
        answer = send(body)
    except OSError as error:
        print(f"no bot listening on 127.0.0.1:{PORT} ({error})")
        return 1
    print(brief(answer) if cmd == "status" else json.dumps(answer))
    return 0 if answer.get("ok") else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
