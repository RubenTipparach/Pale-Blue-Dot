# Design: localization

## 1. The file

`assets/locale/locale_en.yaml`, a nested map whose dotted paths are the
keys:

```yaml
menu:
  picture:
    antialiasing: Antialiasing
hud:
  hotbar: HOTBAR
fishing:
  caught: "Caught a {species}"
  catch_n:
    one: "Catch {count} fish"
    other: "Catch {count} fish"
mission:
  rod:
    title: Make a fishing rod
```

- **Placeholders** are `{name}`, filled by name, never by position, so a
  translation can reorder them.
- **Plurals** are a map with `one` and `other` (and whatever forms a
  language needs: `few`, `many`), chosen by the `count` argument.
- **Case is in the text.** A screen that shows a title in capitals has it
  in capitals in the file. Code never calls `to_uppercase()` on text it
  shows, because case rules differ by language.
- **One file per language:** `locale_<code>.yaml`. English is the source,
  and every other file has its keys.

## 2. The lookup (`pbd-core::locale`)

Engine-independent, so tests and tools can use it without Bevy.

- `Locale::load(yaml) -> Result<Locale>`, using a maintained YAML crate
  (`serde_yaml_ng`).
- `tr(key) -> Localized` and `tr_with(key, &[(name, value)])`.
- A global current locale, set at start-up from the player's setting,
  English for now. `pbd-app` wraps it in a resource and reloads it when
  the file changes in a debug build.
- **A missing key.** In a debug build it shows as `⟦key⟧` and logs
  `LOCALE_MISSING key`. In a release build it falls back to English, then
  to the key itself. The lint stops it before a push anyway.
- **`Localized`** is a newtype over `String`, with no `From<&str>`. The UI
  kit's constructors (`KitText::new`, the button labels, the tooltip and
  the toast) take `Localized`, so `KitText::new("HOTBAR", ...)` is a
  compile error. Text that is not a word, such as an arrow, a number or a
  player's typed world name, goes through `Localized::verbatim(..)`,
  which the lint allows only for values that are not literals with words.

## 3. Data: ids in RON, words in the locale

- Missions, recipes, items, species, towns' kinds, settings options and
  key names keep a stable `id` in their RON or Rust table. Their shown
  text is `tr("<kind>.<id>.name")` (and `.desc`, `.hint`).
- A data loader test checks that every id has its keys.
- Generated names (a town's name from the seed) are built from syllables
  or parts that live in the locale file too, so a language can carry its
  own.

## 4. The lint (hard)

`crates/pbd-app/locale_lint.rs`, run by `pbd-app`'s `build.rs`, so every
`cargo build` fails on it (the owner, 2026-11-20: "it belongs in the build
step. the way merges work right now is fine"), and by
`crates/pbd-app/tests/locale.rs` with the same rules. It parses each `crates/*/src/**/*.rs` with `syn`. A string literal
is **player text** if it holds a letter, and it is allowed only:

- inside `tr("…")` or `tr_with("…", …)` as the key, which must exist;
- in log, trace and diagnostic macros (`info!`, `warn!`, `error!`,
  `debug!`, `trace!`, `println!`, `eprintln!`), and in `panic!`,
  `expect`, `unreachable!` and `assert*` messages;
- in `#[cfg(test)]` modules, `tests/` and attributes;
- as an asset path (it contains `/` or ends in a known extension), an id
  or a key (`^[a-z0-9_.:-]+$`), a shader def, a label for the render
  graph or a pipeline, or a format string with no letters outside its
  `{}`;
- in a module marked `//! locale: dev` (the F3 frame graph and the debug
  readout): developer tools that sit behind a key and are not the game's
  text (CLAUDE.md, "Notes stay out of the game": "A debug overlay is a
  developer tool and stays behind its key"). Recommendation taken (ask
  only with screenshots).

It also fails when:
- a key used in code is not in `locale_en.yaml`;
- a key in `locale_en.yaml` is used nowhere, neither in code nor built
  from a data id;
- a key's placeholders differ from those passed for it;
- a name in a data RON is not a key.

**The baseline.** `tools/locale/baseline.txt` lists today's violations
(`file|literal`, sorted).
- The lint fails on any violation not in it, so no new hard-coded text
  gets in from the first commit.
- It fails on any entry no longer found, so the list only shrinks.
- When the list is empty the file is deleted, and the lint has no
  exceptions left.

## 5. Tooling (`tools/locale/`)

- **`locale_lint.py`** runs the same rules in seconds without building,
  through the same rule set exported as JSON by the test.
  - It reports by file, and `--baseline` rewrites the baseline, which can
    only drop entries.
- **`locale_extract.py <file>`** finds a file's literals and proposes a key
  for each, from the module and the words (`menu.picture.antialiasing`).
  - It writes the entries into `locale_en.yaml` and replaces the literal
    with `tr("key")`, leaving a diff to review.
  - Formatted lines become `tr_with` with named placeholders.
- **`locale_coverage.py <code>`** lists the keys missing from or stale in
  `locale_<code>.yaml`, for a translator.

## 6. Migration order

1. Foundation: the file, `pbd-core::locale`, `Localized`, the UI kit
   taking it, the lint with the baseline, and the tools.
2. Screens, one module at a time, each commit shrinking the baseline:
   menu, pack, craft, forge, kitchen, farm, breeding, equipment, guide,
   map, missions, mission tree, toasts, HUD, compass, time, weather, the
   arrival card, the loading screen, vehicles' HUD, fishing and husbandry
   lines, and interaction prompts.
3. Data: missions, recipes, items, species, settlement kinds, settings
   options and key names.
4. The baseline is empty and deleted.

## Risks

- **Over-matching.** A path or an id with capitals might be flagged. The
  allowed patterns are the lint's own table, and a false positive is
  fixed there, never by baselining it.
- **Text built from parts** ("Caught a " + name) does not translate. The
  lint flags the literal part, and the fix is one `tr_with` with the name
  as an argument.
- **Width.** Other languages run longer. Panels already size to their
  text, and a check with a pseudo-locale (every string padded by 40%,
  `PBD_LOCALE=pseudo`) shows what breaks.
