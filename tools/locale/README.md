# Locale tools

Every word the player reads comes from `assets/locale/locale_en.yaml` by key
(`openspec/changes/localization`, CLAUDE.md "No text in code"). Code shows
text only through `tr("key")` or `tr_with("key", &[("name", &value)])`; the UI
kit takes a `Localized`, which a literal cannot become.

The rule is the test `cargo test -p pbd-app --test locale`. These tools wrap it.

| Tool | What it does |
| --- | --- |
| `locale_lint.py` | Runs the test and reports by file. `--all` lists baselined text too, `--file <x.rs>` narrows it, `--baseline` drops entries that are gone (it never adds), `--fast` is a regex approximation without a build. |
| `locale_extract.py <file.rs>` | Moves a file's flagged literals into the YAML under `<module>.<words>` keys and replaces them with `tr("key")` (or `tr_with` for a `format!` of inline `{names}`). `--dry-run` only proposes. Never touches log or panic text. Review the diff. |
| `locale_coverage.py <code>` | For a translator: keys `locale_<code>.yaml` is missing, keys it has that English does not, and placeholder mismatches. |

`baseline.txt` is the text from before the rule, `path|literal`, sorted. It
only shrinks: a new literal is moved to the locale file, never added here, and
a false positive (a log or developer label the lint flags) is fixed in the
lint's rules in `crates/pbd-app/tests/locale.rs`. When the list is empty,
delete it. `Localized::legacy(..)` calls are listed the same way, as
`path|@legacy <expression>`: they are text not yet moved.

A typical migration of one screen:

```sh
python3 tools/locale/locale_extract.py crates/pbd-app/src/desktop/time_ui.rs
cargo build --profile fast -p pbd-app        # fix any types
python3 tools/locale/locale_lint.py --baseline
cargo test -p pbd-app --test locale
```

Python 3 with PyYAML for the extract and coverage tools.
