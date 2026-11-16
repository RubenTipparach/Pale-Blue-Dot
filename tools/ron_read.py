#!/usr/bin/env python3
"""Read the RON the game's config files are written in, for the tools.

Enough of RON for `assets/config/*.ron`: structs `( name: value, ... )` and
tuples `( a, b )`, lists `[ ... ]`, maps `{ "key": value }`, strings, numbers,
`true`/`false`, `None`/`Some(x)`, unit enum variants (`River`) and newtype or
tuple variants (`Tenebris("Fish")`), with `//` and `/* */` comments and
trailing commas.

What comes back:
- a struct is a dict (field order kept); a tuple is a list;
- a unit variant is its name as a string, and a variant with fields is
  `{"$variant": name, "$fields": [...]}` (or a dict of its fields when it was
  written with names);
- `None` is None and `Some(x)` is x.

The game's own reader (serde) stays the authority. A tool that reads a file
with this is checked by that tool's own test against what the game reads.

    from ron_read import load
    fauna = load("assets/config/fauna.ron")
"""

import re

_TOKEN = re.compile(
    r"""
    (?P<ws>\s+|//[^\n]*|/\*.*?\*/)
  | (?P<str>"(?:[^"\\]|\\.)*")
  | (?P<num>[-+]?(?:\d+\.\d*|\.\d+|\d+)(?:[eE][-+]?\d+)?)
  | (?P<ident>[A-Za-z_][A-Za-z0-9_]*)
  | (?P<punct>[()\[\]{}:,])
    """,
    re.VERBOSE | re.DOTALL,
)


def _tokens(text):
    pos = 0
    out = []
    while pos < len(text):
        m = _TOKEN.match(text, pos)
        if not m:
            raise ValueError(f"RON: cannot read at {text[pos:pos + 30]!r}")
        pos = m.end()
        kind = m.lastgroup
        if kind == "ws":
            continue
        out.append((kind, m.group(kind)))
    return out


class _Parser:
    def __init__(self, tokens):
        self.t = tokens
        self.i = 0

    def peek(self, k=0):
        return self.t[self.i + k] if self.i + k < len(self.t) else (None, None)

    def take(self, value=None):
        tok = self.peek()
        if value is not None and tok[1] != value:
            raise ValueError(f"RON: expected {value!r}, found {tok[1]!r}")
        self.i += 1
        return tok

    def value(self):
        kind, text = self.peek()
        if kind == "str":
            self.take()
            return bytes(text[1:-1], "utf-8").decode("unicode_escape").encode("latin-1").decode("utf-8")
        if kind == "num":
            self.take()
            return float(text) if any(c in text for c in ".eE") else int(text)
        if text == "[":
            return self.seq("[", "]")
        if text == "{":
            return self.map()
        if text == "(":
            return self.group()
        if kind == "ident":
            self.take()
            if text in ("true", "false"):
                return text == "true"
            if text == "None":
                return None
            if self.peek()[1] == "(":
                inner = self.group()
                if text == "Some":
                    return inner[0] if isinstance(inner, list) else inner
                if isinstance(inner, dict):
                    return dict(inner, **{"$variant": text})
                return {"$variant": text, "$fields": inner}
            return text
        raise ValueError(f"RON: unexpected {text!r}")

    def seq(self, open_, close):
        self.take(open_)
        items = []
        while self.peek()[1] != close:
            items.append(self.value())
            if self.peek()[1] == ",":
                self.take()
        self.take(close)
        return items

    def map(self):
        self.take("{")
        out = {}
        while self.peek()[1] != "}":
            key = self.value()
            self.take(":")
            out[key] = self.value()
            if self.peek()[1] == ",":
                self.take()
        self.take("}")
        return out

    def group(self):
        """A struct `(a: 1)` when its first item is `name:`, else a tuple."""
        if self.peek(1)[0] == "ident" and self.peek(2)[1] == ":":
            self.take("(")
            out = {}
            while self.peek()[1] != ")":
                _, name = self.take()
                self.take(":")
                out[name] = self.value()
                if self.peek()[1] == ",":
                    self.take()
            self.take(")")
            return out
        return self.seq("(", ")")


def loads(text):
    parser = _Parser(_tokens(text))
    value = parser.value()
    if parser.i != len(parser.t):
        raise ValueError("RON: text after the value")
    return value


def load(path):
    with open(path, encoding="utf-8") as f:
        return loads(f.read())
