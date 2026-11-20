# Tasks

## 1. Foundation
- [x] 1.1 `assets/locale/locale_en.yaml`; `pbd-core::locale` (load, `tr`, `tr_with`, plurals, missing-key behaviour); tests. Done: `serde_yaml_ng`, English built in by `include_str!`, `⟦key⟧` + `LOCALE_MISSING` in debug; 8 unit tests; builds for wasm32.
- [x] 1.2 `Localized` and the UI kit's constructors taking it; the app resource and debug-build reload. Done: `KitText::new`, `KitText.text`, `plaque_label`, `key_cap`, `Tooltip` take `Localized` (no `From<&str>`); 10 literals moved to keys, the rest wrapped in the transitional `Localized::legacy(..)`, which the lint lists; `pbd-app::locale::LocalePlugin` reads and follows the file in debug builds.
- [x] 1.3 The lint (`crates/pbd-app/tests/locale.rs`, `syn`), its allowed contexts, key and placeholder checks, and `tools/locale/baseline.txt` of today's violations. Done: 953 entries (59 of them `@legacy` calls); `Err`, `ok_or`/`map_err` payloads and GPU labels are allowed as diagnostics; F3 graph, CPU times and the repro record marked `//! locale: dev`.
- [x] 1.4 `tools/locale/locale_lint.py`, `locale_extract.py`, `locale_coverage.py`. Done, with `tools/locale/README.md`; the lint tool reads the test's JSON report (`PBD_LOCALE_REPORT`), `--fast` is a regex approximation.
- [x] 1.5 The rule in CLAUDE.md. (Added 2026-11-20 with the write-up: "No text in code: every word from the locale file".)

## 2. Screens (each shrinks the baseline)
- [ ] 2.1 Menu and settings. - [ ] 2.2 Pack, craft, forge, kitchen, farm, breeding, equipment. - [ ] 2.3 Guide, map, missions, mission tree. - [ ] 2.4 HUD, toasts, compass, time, weather, arrival, loading. - [ ] 2.5 Fishing, husbandry, interaction prompts, vehicles' HUD.

## 3. Data
- [ ] 3.1 Missions and recipes. - [ ] 3.2 Items and species. - [ ] 3.3 Settlement kinds, settings options, key names.

## 4. Close
- [ ] 4.1 The baseline empty and deleted; `PBD_LOCALE=pseudo` stills of the main screens for width.
