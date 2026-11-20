#!/usr/bin/env python3
"""Names the call stacks in a `PBD_ALLOC_TRACE` table (`memory-leak`).

    python3 tools/alloc_trace.py <table.txt> <pbd-app binary> [--top 25] [--depth 8]
                                 [--before <earlier table.txt>]

The table is written by the `alloc-trace` build (`crates/pbd-app/src/alloc_trace.rs`):
a header with the executable's ASLR slide, then one line per call stack:
`count bytes ip ip ...`. With `--before`, each stack's count and bytes are the
difference between the two tables, so a window of play can be read on its own.
The binary must keep its symbols (`CARGO_PROFILE_FAST_STRIP=none`). macOS only:
names come from `atos`.
"""

import argparse
import re
import subprocess


def read(path):
    rows, slide = {}, 0
    for line in open(path):
        if line.startswith("#"):
            m = re.search(r"slide (0x[0-9a-f]+|-?\d+)", line)
            if m:
                slide = int(m.group(1), 0)
            continue
        parts = line.split()
        if len(parts) < 3:
            continue
        rows[tuple(parts[2:])] = (int(parts[0]), int(parts[1]))
    return rows, slide


def short(symbol):
    """A rough reading of a Rust v0 symbol: its path's identifiers joined."""
    name = re.sub(r" \(in [^)]*\)", "", symbol)
    m = re.match(r"(_R\S+)(.*)", name)
    if not m:
        return name
    mangled, rest = m.groups()
    out, i = [], 0
    while i < len(mangled):
        if mangled[i].isdigit():
            j = i
            while j < len(mangled) and mangled[j].isdigit():
                j += 1
            n = int(mangled[j - 1 if mangled[i] == "0" else i : j])
            if mangled[j : j + 1] == "_":
                j += 1
            word = mangled[j : j + n]
            if n and len(word) == n and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", word):
                out.append(word)
                i = j + n
                continue
            i = j
        else:
            i += 1
    return "::".join(out) + rest


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("table")
    ap.add_argument("binary")
    ap.add_argument("--top", type=int, default=25)
    ap.add_argument("--depth", type=int, default=8)
    ap.add_argument("--before")
    ap.add_argument(
        "--min-avg",
        type=float,
        default=0.0,
        help="only stacks whose mean allocation is at least this many MiB",
    )
    args = ap.parse_args()
    rows, slide = read(args.table)
    if args.before:
        before, _ = read(args.before)
        rows = {
            k: (n - before.get(k, (0, 0))[0], b - before.get(k, (0, 0))[1])
            for k, (n, b) in rows.items()
        }
    rows = {
        k: v for k, v in rows.items() if v[0] > 0 and v[1] / v[0] >= args.min_avg * 2**20
    }
    top = sorted(rows.items(), key=lambda kv: -kv[1][1])[: args.top]
    ips = sorted({ip for stack, _ in top for ip in stack[: args.depth + 3]})
    load = 0x100000000 + slide
    names = {}
    if ips:
        out = subprocess.run(
            ["atos", "-o", args.binary, "-l", hex(load), *ips],
            capture_output=True,
            text=True,
            check=True,
        ).stdout.splitlines()
        names = dict(zip(ips, out))
    skip = re.compile(
        r"(^alloc::|^core::|^std::|::hashbrown::|__rust_|raw_vec|alloc_trace|^0x)"
    )
    for stack, (n, b) in top:
        frames = [names.get(ip, ip) for ip in stack]
        frames = [short(f) for f in frames]
        own = [f for f in frames if not skip.search(f)][: args.depth]
        print(f"{n:7d} allocs {b / 2**20:9.1f} MiB (mean {b / max(n, 1) / 2**20:.2f} MiB)")
        for f in own:
            print(f"        {f[:160]}")


if __name__ == "__main__":
    main()
