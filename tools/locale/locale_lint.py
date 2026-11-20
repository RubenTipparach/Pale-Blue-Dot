#!/usr/bin/env python3
"""The locale lint, run on its own (`openspec/changes/localization` design 5).

The rules live in one place: `crates/pbd-app/tests/locale.rs`. This runs that
test with `PBD_LOCALE_REPORT` set, reads the JSON it writes, and reports by
file.

    python3 tools/locale/locale_lint.py              # report, exit 1 on failure
    python3 tools/locale/locale_lint.py --all        # also list baselined text
    python3 tools/locale/locale_lint.py --baseline   # drop stale baseline entries
    python3 tools/locale/locale_lint.py --fast       # regex approximation, no build

`--baseline` only ever removes entries, and deletes the file when none is
left: a literal not in the baseline is a violation to fix, never one to add.
The baseline was emptied and deleted with `localization` phase 3, so the lint
has no exceptions; a missing file is read as none.
"""

from __future__ import annotations

import argparse
import collections
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BASELINE = ROOT / "tools" / "locale" / "baseline.txt"
HEADER = (
    "# Text from before the locale rule (`openspec/changes/localization`):\n"
    "# `path|literal`, sorted. It only shrinks; never add to it. When it is\n"
    "# empty, delete it. `python3 tools/locale/locale_lint.py --baseline` drops\n"
    "# entries that are gone.\n"
)


def run_report() -> dict:
    """Run the Rust lint and return its JSON report."""
    with tempfile.TemporaryDirectory() as tmp:
        out = Path(tmp) / "report.json"
        env = dict(os.environ, PBD_LOCALE_REPORT=str(out))
        proc = subprocess.run(
            # The everyday `fast` profile, so the lint shares its build with
            # `cargo build --profile fast` rather than keeping a debug tree.
            ["cargo", "test", "-q", "--profile", "fast", "-p", "pbd-app", "--test", "locale", "--", "no_words_in_code"],
            cwd=ROOT,
            env=env,
            capture_output=True,
            text=True,
        )
        if not out.exists():
            sys.stderr.write(proc.stdout + proc.stderr)
            sys.exit("locale_lint: the test did not write a report (does it build?)")
        return json.loads(out.read_text(encoding="utf-8"))


def read_baseline() -> list[str]:
    if not BASELINE.exists():
        return []
    return [l for l in BASELINE.read_text(encoding="utf-8").splitlines() if l and not l.startswith("#")]


def write_baseline(lines: list[str]) -> None:
    BASELINE.parent.mkdir(parents=True, exist_ok=True)
    BASELINE.write_text(HEADER + "".join(l + "\n" for l in sorted(lines)), encoding="utf-8")


def split(violations: list[dict], baseline: list[str]):
    """New violations, baselined ones, and stale baseline entries (multisets)."""
    left = collections.Counter(baseline)
    new, old = [], []
    for v in violations:
        if left[v["baseline"]] > 0:
            left[v["baseline"]] -= 1
            old.append(v)
        else:
            new.append(v)
    stale = list(left.elements())
    return new, old, stale


# ------------------------------------------------------------------ fast mode

STRING = re.compile(r'"((?:[^"\\\n]|\\.)*)"')
DEV = re.compile(
    r"\b(info|warn|error|debug|trace|println|eprintln|print|eprint|panic|unreachable|todo|"
    r"unimplemented|assert|assert_eq|assert_ne|debug_assert\w*|include_str|env|dbg)!\s*\(|"
    r"\.expect(_err)?\(|\btr(_with)?\(|Name::new\(|env::var|label:\s*Some\(|#\[|\bErr\(|"
    r"\.ok_or(_else)?\(|\.map_err\(|Status::Fail\(|create_bind_group(_layout)?\(|"
    r"BindGroupLayoutDescriptor::new\(|ShaderDefVal::|MeshVertexAttribute::new\("
)
ID = re.compile(r"^[a-z0-9_.:-]+$")


def allowed_text(s: str) -> bool:
    if not any(c.isalpha() for c in s):
        return True
    if ID.match(s):
        return True
    if not any(c.isspace() for c in s) and ("/" in s or re.search(r"\.[A-Za-z0-9]{2,5}$", s)):
        return True
    return not any(c.isalpha() for c in re.sub(r"\{[^}]*\}", "", s))


def fast_scan() -> list[dict]:
    """A regex approximation: no build, a few seconds, rougher edges (it does
    not see multi-line macro calls or nested test modules)."""
    out = []
    for path in sorted((ROOT / "crates").glob("*/src/**/*.rs")):
        text = path.read_text(encoding="utf-8")
        if "//! locale: dev" in text.split("\n\n", 1)[0]:
            continue
        rel = path.relative_to(ROOT).as_posix()
        in_test = False
        skip_depth = 0  # open parentheses of a developer call still running
        for n, line in enumerate(text.splitlines(), 1):
            if re.match(r"\s*#\[cfg\((all\()?test", line):
                in_test = True
            if in_test or line.lstrip().startswith("//"):
                continue
            code = STRING.sub('""', line)
            if skip_depth > 0 or DEV.search(line):
                m = DEV.search(line)
                tail = code[m.start():] if m and skip_depth == 0 else code
                skip_depth = max(0, skip_depth + tail.count("(") - tail.count(")"))
                continue
            for m in STRING.finditer(line):
                s = bytes(m.group(1), "utf-8").decode("unicode_escape", errors="ignore")
                if not allowed_text(s):
                    out.append({"file": rel, "line": n, "text": s, "kind": "literal"})
    return out


# ------------------------------------------------------------------ main


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--all", action="store_true", help="also list baselined literals")
    ap.add_argument("--baseline", action="store_true", help="drop stale entries from the baseline")
    ap.add_argument("--fast", action="store_true", help="regex approximation without building")
    ap.add_argument("--file", help="only report this file (a path under crates/)")
    args = ap.parse_args()

    if args.fast:
        found = fast_scan()
        by_file = collections.Counter(v["file"] for v in found)
        for f, n in sorted(by_file.items(), key=lambda kv: -kv[1]):
            if not args.file or f.endswith(args.file):
                print(f"{n:5}  {f}")
        print(f"{len(found)} literal(s) with words (approximate; the test is the rule)")
        return 0

    report = run_report()
    violations = report["violations"]
    baseline = read_baseline()
    new, old, stale = split(violations, baseline)
    if args.baseline:
        if stale:
            left = collections.Counter(baseline) - collections.Counter(stale)
            if left:
                write_baseline(list(left.elements()))
            else:
                # Empty: the file goes, and the lint has no exceptions left.
                BASELINE.unlink()
        print(f"baseline: dropped {len(stale)} stale entr(y/ies), {len(baseline) - len(stale)} left")
        baseline = read_baseline()
        new, old, stale = split(violations, baseline)

    shown = new + (old if args.all else [])
    if args.file:
        shown = [v for v in shown if v["file"].endswith(args.file)]
    by_file = collections.defaultdict(list)
    for v in shown:
        by_file[v["file"]].append(v)
    for f in sorted(by_file, key=lambda f: (-len(by_file[f]), f)):
        print(f"{f}  ({len(by_file[f])})")
        for v in sorted(by_file[f], key=lambda v: v["line"]):
            tag = "" if v in new else "  [baseline]"
            print(f"  {v['line']:5}: {json.dumps(v['text'], ensure_ascii=False)}{tag}")
    if new:
        print(f"\n{len(new)} new literal(s) with words: move each to assets/locale/locale_en.yaml "
              "and call tr(\"key\") (tools/locale/locale_extract.py <file> does it for you)")
    for s in stale:
        print(f"stale baseline entry: {s}")
    print(f"baseline: {len(baseline)} entries ({len(old)} still found, {len(stale)} stale); "
          f"{len(report['keys'])} key uses")
    return 1 if new or stale else 0


if __name__ == "__main__":
    sys.exit(main())
