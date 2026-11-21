#!/usr/bin/env python3
"""Move one file's text into the locale file (`openspec/changes/localization`
design 5).

    python3 tools/locale/locale_extract.py crates/pbd-app/src/desktop/time_ui.rs
    python3 tools/locale/locale_extract.py <file> --dry-run   # only propose

For each literal the lint flags in the file it:
- proposes a key `<module>.<words_snake>` (`time_ui.time`), reusing a key that
  already holds the same text;
- writes the entry into `assets/locale/locale_en.yaml` in its nested place;
- replaces the literal with `tr("key")`, and a `format!("... {name} ...")`
  whose arguments are all inline names with
  `tr_with("key", &[("name", &name)])`.

It never touches log, panic or assertion text (the lint does not flag them).
A literal in a `const`/`static`, a format string with positional arguments or
format specs, or one inside `write!` is left in place and listed as manual.

The result is a diff to review: types may need a touch (`tr` returns a
`Localized`, which the UI kit takes; a `String` slot wants `.into_string()`).
Then run `python3 tools/locale/locale_lint.py --baseline` to drop the
baseline entries that went, and build.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import locale_lint  # noqa: E402

ROOT = locale_lint.ROOT
YAML = ROOT / "assets" / "locale" / "locale_en.yaml"
STOP = {"a", "an", "the", "to", "of", "and", "or", "in", "on", "it", "is", "for", "by", "at"}

try:
    import yaml  # PyYAML
except ImportError:  # pragma: no cover
    yaml = None


def flat_keys() -> dict[str, object]:
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from locale_coverage import flatten

    if yaml is None:
        sys.exit("locale_extract: needs PyYAML (pip install pyyaml)")
    return flatten(yaml.safe_load(YAML.read_text(encoding="utf-8")))


def module_of(rel: str) -> str:
    p = Path(rel)
    stem = p.parent.name if p.stem in ("mod", "lib", "main") else p.stem
    return re.sub(r"[^a-z0-9_]", "_", stem.lower())


def words_key(text: str) -> str:
    plain = re.sub(r"\{[^}]*\}", " ", text)
    words = [w.lower() for w in re.findall(r"[A-Za-z0-9]+", plain)]
    kept = [w for w in words if w not in STOP] or words or ["text"]
    key = "_".join(kept[:5])
    if key[0].isdigit():
        key = "n" + key
    return key


# ------------------------------------------------------------------ YAML insert

KEY_LINE = re.compile(r"^( *)([A-Za-z0-9_-]+):(.*)$")


def block_end(lines: list[str], start: int, indent: int) -> int:
    """The index after the last line of a block whose children are deeper than `indent`."""
    end = start
    for i in range(start, len(lines)):
        s = lines[i]
        if not s.strip() or s.lstrip().startswith("#"):
            continue
        if len(s) - len(s.lstrip(" ")) <= indent:
            break
        end = i + 1
    return end


def insert(lines: list[str], path: list[str], value: str) -> None:
    lo, hi, indent = 0, len(lines), 0
    for depth, name in enumerate(path):
        last = depth == len(path) - 1
        found = None
        after = None  # insert before the first sibling sorting after `name`
        for i in range(lo, hi):
            m = KEY_LINE.match(lines[i])
            if not m or len(m.group(1)) != indent:
                continue
            if m.group(2) == name:
                found = i
                break
            if after is None and m.group(2) > name:
                after = i
        if found is not None:
            if last:
                raise ValueError(f"{'.'.join(path)} exists")
            lo, hi = found + 1, block_end(lines, found + 1, indent)
            indent += 2
            continue
        new = []
        for d, n in enumerate(path[depth:]):
            pad = " " * (indent + 2 * d)
            if depth + d == len(path) - 1:
                new.append(f"{pad}{n}: {json.dumps(value, ensure_ascii=False)}")
            else:
                new.append(f"{pad}{n}:")
        if after is not None:
            at = after
        else:
            at = lo
            for i in range(lo, hi):
                if lines[i].strip():
                    at = i + 1
        lines[at:at] = new
        return


# ------------------------------------------------------------------ Rust edit


def offsets(text: str) -> list[int]:
    """Char offset of each line's start (1-based lines)."""
    starts = [0, 0]
    for i, c in enumerate(text):
        if c == "\n":
            starts.append(i + 1)
    return starts


def add_import(text: str, names: set[str], core: bool) -> str:
    if not names:
        return text
    have = set()
    for m in re.finditer(r"^use [^;]*\blocale::\{?([^;}]*)\}?;", text, re.M | re.S):
        have |= {n.strip() for n in m.group(1).split(",")}
    for m in re.finditer(r"^use [^;]*ui_kit::\{([^}]*)\}", text, re.M | re.S):
        have |= {n.strip() for n in m.group(1).split(",")}
    need = sorted(names - have)
    if not need:
        return text
    base = "crate::locale" if core else "pbd_core::locale"
    line = f"use {base}::{{{', '.join(need)}}};\n" if len(need) > 1 else f"use {base}::{need[0]};\n"
    uses = list(re.finditer(r"^use [^;]*;\n", text, re.M | re.S))
    at = uses[-1].end() if uses else 0
    if not uses:
        m = re.match(r"(?://![^\n]*\n|\s*\n)*", text)
        at = m.end() if m else 0
    return text[:at] + line + text[at:]


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("files", nargs="+", help=".rs files under crates/ (one lint run for all)")
    ap.add_argument("--dry-run", action="store_true", help="propose keys, change nothing")
    args = ap.parse_args()
    report = locale_lint.run_report()
    for file in args.files:
        extract(file, report, args.dry_run)
    return 0


def extract(file: str, report: dict, dry_run: bool) -> None:
    path = (Path.cwd() / file).resolve()
    rel = path.relative_to(ROOT).as_posix()
    found = [v for v in report["violations"] if v["file"] == rel and v["kind"] == "literal"]
    if not found:
        print(f"{rel}: nothing to extract")
        return

    existing = flat_keys()
    module = module_of(rel)
    # Reuse a key for the same text only within the module: the same word on
    # two screens may translate differently.
    by_text = {
        t: k for k, t in existing.items() if isinstance(t, str) and k.startswith(module + ".")
    }
    text = path.read_text(encoding="utf-8")
    starts = offsets(text)
    lines_of = text.split("\n")

    edits, entries, manual, used = [], {}, [], set()
    for v in sorted(found, key=lambda v: (v["line"], v["col"])):
        src_line = lines_of[v["line"] - 1]
        before = src_line[: v["col"]]
        if re.search(r"\b(const|static)\b", before):
            manual.append((v, "in a const or static"))
            continue
        fmt = v.get("format")
        if fmt and (fmt["mac"] != "format" or not fmt["simple"]):
            manual.append((v, f"{fmt['mac']}! with positional arguments or specs"))
            continue
        value = v["text"]
        key = by_text.get(value) or entries.get(value)
        if key is None:
            base = f"{module}.{words_key(value)}"
            key, n = base, 2
            while key in existing or key in entries.values():
                key, n = f"{base}_{n}", n + 1
            entries[value] = key
        if fmt:
            names = locale_lint_names(value)
            args_src = ", ".join(f'("{n}", &{n})' for n in names)
            call = f'tr_with("{key}", &[{args_src}])' if names else f'tr("{key}")'
            used.add("tr_with" if names else "tr")
            span = (fmt["line"], fmt["col"], fmt["end_line"], fmt["end_col"])
        else:
            call = f'tr("{key}")'
            used.add("tr")
            span = (v["line"], v["col"], v["end_line"], v["end_col"])
        edits.append((span, call, key, value, v["line"]))

    for _, call, key, value, line in edits:
        print(f"  {rel}:{line}: {json.dumps(value, ensure_ascii=False)} -> {call}")
    for v, why in manual:
        print(f"  {rel}:{v['line']}: {json.dumps(v['text'], ensure_ascii=False)}  MANUAL ({why})")
    if dry_run:
        print(f"{len(edits)} to extract, {len(manual)} manual (dry run: nothing changed)")
        return

    # Rust: apply from the end so earlier offsets hold. Overlapping spans
    # (two literals in one format!) keep the outer one.
    done_spans = []
    for (l0, c0, l1, c1), call, *_ in sorted(edits, key=lambda e: e[0], reverse=True):
        a, b = starts[l0] + c0, starts[l1] + c1
        if any(a < y and x < b for x, y in done_spans):
            continue
        text = text[:a] + call + text[b:]
        done_spans.append((a, b))
    text = add_import(text, used, rel.startswith("crates/pbd-core/"))
    path.write_text(text, encoding="utf-8")

    # YAML: each new entry in its nested place.
    ylines = YAML.read_text(encoding="utf-8").split("\n")
    for value, key in sorted(entries.items(), key=lambda kv: kv[1]):
        insert(ylines, key.split("."), value)
    YAML.write_text("\n".join(ylines), encoding="utf-8")

    print(f"{len(edits)} extracted ({len(entries)} new keys), {len(manual)} manual. "
          "Review the diff, build, then: python3 tools/locale/locale_lint.py --baseline")


def locale_lint_names(fmt: str) -> list[str]:
    names = []
    for m in re.finditer(r"(?<!\{)\{([A-Za-z_][A-Za-z0-9_]*)\}", fmt):
        if m.group(1) not in names:
            names.append(m.group(1))
    return names


if __name__ == "__main__":
    sys.exit(main())
