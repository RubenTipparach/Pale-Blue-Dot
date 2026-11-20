# Tasks

## 1. Foundation
- [x] 1.1 `assets/locale/locale_en.yaml`; `pbd-core::locale` (load, `tr`, `tr_with`, plurals, missing-key behaviour); tests. Done: `serde_yaml_ng`, English built in by `include_str!`, `⟦key⟧` + `LOCALE_MISSING` in debug; 8 unit tests; builds for wasm32.
- [x] 1.2 `Localized` and the UI kit's constructors taking it; the app resource and debug-build reload. Done: `KitText::new`, `KitText.text`, `plaque_label`, `key_cap`, `Tooltip` take `Localized` (no `From<&str>`); 10 literals moved to keys, the rest wrapped in the transitional `Localized::legacy(..)`, which the lint lists; `pbd-app::locale::LocalePlugin` reads and follows the file in debug builds.
- [x] 1.3 The lint (`crates/pbd-app/tests/locale.rs`, `syn`), its allowed contexts, key and placeholder checks, and `tools/locale/baseline.txt` of today's violations. Done: 953 entries (59 of them `@legacy` calls); `Err`, `ok_or`/`map_err` payloads and GPU labels are allowed as diagnostics; F3 graph, CPU times and the repro record marked `//! locale: dev`.
- [x] 1.4 `tools/locale/locale_lint.py`, `locale_extract.py`, `locale_coverage.py`. Done, with `tools/locale/README.md`; the lint tool reads the test's JSON report (`PBD_LOCALE_REPORT`), `--fast` is a regex approximation.
- [x] 1.5 The rule in CLAUDE.md. (Added 2026-11-20 with the write-up: "No text in code: every word from the locale file".)

## 2. Screens (each shrinks the baseline)
- [x] 2.1 Menu and settings. Done: `menu.*` (tabs, buttons, sliders, the saves list and its messages), `menu.quality.*`, `menu.frame_limit_value.*`, `saves.refusal.*`; the menu's helpers take `Localized`.
- [x] 2.2 Pack, craft, forge, kitchen, farm, breeding, equipment. Done: `pack.*`, `slots.*`, `craft.*`, `forge.*`, `kitchen.*`, `farm.*`, `fruit.*`, `breeding.*`, `breed.*`, `equipment.*`.
- [x] 2.3 Guide, map, missions, mission tree. Done: `guide.*`, `map.*` (layers' and keys' words held as keys in their tables), `map_sites.*`, `missions.*`, `mission_tree.*`.
- [x] 2.4 HUD, toasts, compass, time, weather, arrival, loading. Done: `toasts.*`, `compass.*` (DIRECTIONS holds ids), `time.*`, `weather.*`, `overlay.*`, `arrival.*`, `loading.*`, `window.title`.
- [x] 2.5 Fishing, husbandry, interaction prompts, vehicles' HUD. Done: `fishing.*`, `husbandry.*`, `interact.act.*` (the prompt's line in capitals through `locale::caps`), `vehicles.hud.*`, `launch.*`, `digging.*`.

Recommendation taken (ask only with screenshots): a line composed from data names that the screen shows in capitals (the interact prompt, a locked recipe's line naming its mission, a player-typed world name) goes through the locale-owned `locale::caps`, and a slot tooltip's first capital through `locale::sentence`; text written whole has its case in the file, and data shown in capitals on its own has a `.caps` key (species, tools, effects, seasons).

## 3. Data
- [x] 3.1 Missions and recipes. Done: `missions.ron`, `recipes.ron` and `cooking.ron` hold ids; words are `mission.<id>.title/help/objective.<n>` and `recipe.<code>.name`, read at load. Missions and saved progress still name species and blocks by their English names, read from the built-in English (`locale::english`) whatever the language, so no save changes.
- [x] 3.2 Items and species. Done: `item.<code>.name` (and `.many` for fruit), `tool.<code>.*`, `block.<id>.name`, `crop.<code>.name`, `station.<code>.name`, `species.<id>.name/lower/caps/entry/tip` (`fauna.ron` and `husbandry.ron` lose their names and guide text), `husbandry.<coat|pattern|morph|adult|stage>.<id>`, `vehicle.<key>.*`. Each prefix is in the lint's `DATA_PREFIXES` and each table has a test that every id has its keys.
- [x] 3.3 Settlement kinds, settings options, key names. Done: `sites.kind.*`, `menu.quality.*`, `controls.key.<bevy name>`, `controls.group.<id>`, `controls.does.<act>`.

## 4. Close
- [~] 4.1 The baseline empty and deleted; `PBD_LOCALE=pseudo` stills of the main screens for width. Done 2026-11-20: 953 entries to none, `tools/locale/baseline.txt` deleted, a missing baseline read as no exceptions; 211 false positives fixed in the lint's rules (ids, constants, CSV headers, paths, labels set after a GPU object is made, and each file's validators, log labels, GPU-label helpers and debug lines in `DEV_SCOPES`). Still owed: the pseudo-locale stills.
