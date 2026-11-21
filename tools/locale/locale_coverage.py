#!/usr/bin/env python3
"""A translator's coverage report (`openspec/changes/localization` design 5).

    python3 tools/locale/locale_coverage.py de      # assets/locale/locale_de.yaml

Lists the keys English has that `locale_<code>.yaml` lacks (missing), the keys
it has that English does not (stale), and keys whose `{placeholders}` differ.
Exits 1 when anything is missing, stale or mismatched.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LOCALE = ROOT / "assets" / "locale"
PLURAL_FORMS = {"zero", "one", "two", "few", "many", "other"}
PLACEHOLDER = re.compile(r"(?<!\{)\{([^{}]+)\}(?!\})")

try:
    import yaml  # PyYAML
except ImportError:  # pragma: no cover
    sys.exit("locale_coverage: needs PyYAML (pip install pyyaml)")


def flatten(node, prefix="", out=None) -> dict:
    """Dotted key -> text, or -> {form: text} for a plural map."""
    out = {} if out is None else out
    for k, v in (node or {}).items():
        key = f"{prefix}.{k}" if prefix else str(k)
        if isinstance(v, dict):
            if "other" in v and set(map(str, v)) <= PLURAL_FORMS and all(not isinstance(x, dict) for x in v.values()):
                out[key] = {str(f): str(t) for f, t in v.items()}
            else:
                flatten(v, key, out)
        else:
            out[key] = "" if v is None else str(v)
    return out


def load(code: str) -> dict:
    path = LOCALE / f"locale_{code}.yaml"
    if not path.exists():
        sys.exit(f"locale_coverage: no {path.relative_to(ROOT)}")
    return flatten(yaml.safe_load(path.read_text(encoding="utf-8")))


def placeholders(entry) -> set[str]:
    texts = entry.values() if isinstance(entry, dict) else [entry]
    return {m.strip() for t in texts for m in PLACEHOLDER.findall(t)}


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("code", help="the language code, e.g. de for locale_de.yaml")
    args = ap.parse_args()
    en, other = load("en"), load(args.code)
    missing = sorted(set(en) - set(other))
    stale = sorted(set(other) - set(en))
    mismatched = sorted(k for k in set(en) & set(other) if placeholders(en[k]) != placeholders(other[k]))
    for k in missing:
        print(f"missing   {k}: {en[k]!r}")
    for k in stale:
        print(f"stale     {k}")
    for k in mismatched:
        print(f"mismatch  {k}: en {sorted(placeholders(en[k]))} vs {args.code} {sorted(placeholders(other[k]))}")
    done = len(en) - len(missing)
    pct = 100.0 * done / len(en) if en else 100.0
    print(f"{args.code}: {done}/{len(en)} keys ({pct:.1f}%), {len(missing)} missing, "
          f"{len(stale)} stale, {len(mismatched)} placeholder mismatch(es)")
    return 1 if missing or stale or mismatched else 0


if __name__ == "__main__":
    sys.exit(main())
