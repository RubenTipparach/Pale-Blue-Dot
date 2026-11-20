# Tasks

## 1. Foundation
- [ ] 1.1 `assets/locale/locale_en.yaml`; `pbd-core::locale` (load, `tr`, `tr_with`, plurals, missing-key behaviour); tests.
- [ ] 1.2 `Localized` and the UI kit's constructors taking it; the app resource and debug-build reload.
- [ ] 1.3 The lint (`crates/pbd-app/tests/locale.rs`, `syn`), its allowed contexts, key and placeholder checks, and `tools/locale/baseline.txt` of today's violations.
- [ ] 1.4 `tools/locale/locale_lint.py`, `locale_extract.py`, `locale_coverage.py`.
- [ ] 1.5 The rule in CLAUDE.md.

## 2. Screens (each shrinks the baseline)
- [ ] 2.1 Menu and settings. - [ ] 2.2 Pack, craft, forge, kitchen, farm, breeding, equipment. - [ ] 2.3 Guide, map, missions, mission tree. - [ ] 2.4 HUD, toasts, compass, time, weather, arrival, loading. - [ ] 2.5 Fishing, husbandry, interaction prompts, vehicles' HUD.

## 3. Data
- [ ] 3.1 Missions and recipes. - [ ] 3.2 Items and species. - [ ] 3.3 Settlement kinds, settings options, key names.

## 4. Close
- [ ] 4.1 The baseline empty and deleted; `PBD_LOCALE=pseudo` stills of the main screens for width.
