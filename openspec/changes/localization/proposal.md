# Proposal: every word the player reads comes from `locale_en.yaml`

## Why

The owner, 2026-11-20:

> one big refactor: make it a hard lint rule or something to strictly
> prevent hard coded text in game. dictionary look ups and configs are
> fine, but I want all strings to be put in a locale_en.yaml file, so if we
> want to port the entire game to to other languages its easy to do
>
> implement this rule in game and on claude md, and add some tooling for it
> as well.

Today the game's text is written inline:
- about 475 capitalised literals in `pbd-app` alone (`KitText::new("HOTBAR"
  ...)`, `Text::new("wheel to choose, let go of G")`, `format!` lines for
  the HUD);
- names and prose in data files (`assets/config/missions.ron`,
  `recipes.ron`, the species, items and towns).

Porting the game means finding every one of them.

## What Changes

1. **One file for the words: `assets/locale/locale_en.yaml`.** Dotted keys
   (`menu.picture.antialiasing`, `hud.hotbar`, `mission.rod.title`), with
   named placeholders (`"Catch {count} fish"`) and plural forms. Another
   language is `locale_<code>.yaml` beside it, with the same keys.
2. **A small lookup the whole game uses.**
   - `pbd-core::locale` loads the file and answers `tr("key")` and
     `tr_with("key", args)`. A missing key falls back to English, and in
     a debug build it is shown loudly.
   - The UI kit's text takes only a `Localized` value, made by those calls,
     so a raw string literal handed to the UI does not compile.
3. **Data carries ids, not prose.** Missions, recipes, items and species
   keep their ids in RON, and their names and descriptions move to the
   locale file under keys built from those ids. Numbers, tunables and
   asset paths stay in their configs: "dictionary look ups and configs are
   fine".
4. **A hard lint, run as a test** (`cargo test -p pbd-app --test locale`,
   so CI and every push run it).
   - It parses every source file and fails on any string literal with
     words in it outside the allowed places: log and panic messages,
     tests, attributes, asset paths, ids and keys, and `tr` calls.
   - Every key used must exist in `locale_en.yaml`, every key there must be
     used, and placeholders must match.
   - Data RON names must be keys.
5. **Tooling** in `tools/locale/`:
   - `locale_lint.py`, the same check runnable on its own, with a report
     by file;
   - `locale_extract.py`, which proposes keys for the literals it finds
     and writes them into the YAML and the code for review;
   - a coverage report for a second language, listing missing and stale
     keys.
6. **The rule in CLAUDE.md**: no player-facing text in code or data, only
   keys into the locale file.

## Capabilities

### New Capabilities

- `platform/localization`: the locale file, the lookup, the lint and its
  tooling.

## Impact

- Every screen and HUD module in `pbd-app/src/desktop`, plus `fish.rs`,
  `husbandry.rs`, `interact.rs`, `loading.rs`, `vehicles/hud.rs` and
  `ui_kit.rs`.
- The data RONs with names, and their loaders.
- **Saves:** none. Ids are what saves hold, and they do not change.
- **Look:** none. The same words come out of the file.
- **Migration:** the lint starts with a baseline of today's literals, so it
  blocks every new one from the first commit. The baseline must shrink and
  may never grow, and it is deleted when it reaches zero.
