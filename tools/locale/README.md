# Locale tools

Every word the player reads comes from `assets/locale/locale_en.yaml` by key
(`openspec/changes/localization`, CLAUDE.md "No text in code"). Code shows
text only through `tr("key")` or `tr_with("key", &[("name", &value)])`; the UI
kit takes a `Localized`, which a literal cannot become.

The rule is the test `cargo test -p pbd-app --test locale`. These tools wrap it.

| Tool | What it does |
| --- | --- |
| `locale_lint.py` | Runs the test (on the `fast` profile) and reports by file. `--file <x.rs>` narrows it, `--fast` is a regex approximation without a build. |
| `locale_extract.py <file.rs>...` | Moves a file's flagged literals into the YAML under `<module>.<words>` keys and replaces them with `tr("key")` (or `tr_with` for a `format!` of inline `{names}`). `--dry-run` only proposes. Never touches log or panic text. Review the diff. |
| `locale_coverage.py <code>` | For a translator: keys `locale_<code>.yaml` is missing, keys it has that English does not, and placeholder mismatches. |

There is no baseline: the text from before the rule (`baseline.txt`, once 953
entries) was moved to the locale file and the file deleted
(`openspec/changes/localization` phase 3), so the lint has no exceptions and
a missing baseline reads as none. A false positive (a log or developer label
the lint flags) is fixed in the lint's rules in
`crates/pbd-app/tests/locale.rs`: its text patterns, `DEV_CALLS`,
`DEV_METHODS`, `DEV_MACROS` and the per-file `DEV_SCOPES`.

Data keeps its ids and builds its keys from them (`species.<id>.name`,
`item.<code>.name`, `recipe.<code>.name`, ...). Each such prefix is in the
lint's `DATA_PREFIXES`, so its keys count as used, and the table that builds
them has a test that every id has its keys.

A typical migration of one screen:

```sh
python3 tools/locale/locale_extract.py crates/pbd-app/src/desktop/time_ui.rs
cargo build --profile fast -p pbd-app        # fix any types
cargo test -p pbd-app --test locale
```

Python 3 with PyYAML for the extract and coverage tools.
