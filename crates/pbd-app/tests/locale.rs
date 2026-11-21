//! The locale lint as tests (`openspec/changes/localization` design 4). The
//! rules live in `crates/pbd-app/locale_lint.rs`, which `build.rs` also runs,
//! so a plain `cargo build` fails on text in code too.

#[path = "../locale_lint.rs"]
mod locale_lint;

use locale_lint::*;

#[test]
fn no_words_in_code() {
    let msg = words_in_code();
    assert!(msg.is_empty(), "\n{msg}");
}

#[test]
fn keys_agree_with_the_file() {
    let msg = keys_disagree();
    assert!(msg.is_empty(), "\nlocale keys and code disagree:\n{msg}");
}

#[test]
fn the_rules_classify_text() {
    for ok in [
        "",
        "→",
        "12 / 40",
        "hud.hotbar",
        "textures/ui/kit.png",
        "config.ron",
        "{count} · {name}",
        "{}",
        "pbd-app:bloom",
        "landing-{set}-f",
        "{dir}/{name}.png",
        "interact:KeyF",
        "PBD_RIVER_FILL",
        "PaleBlueDot",
        "Library/Application Support/PaleBlueDot",
        "frame,dt_s,x",
    ] {
        assert!(allowed_text(ok), "{ok:?} should be allowed");
    }
    for bad in [
        "HOTBAR",
        "Catch {count} fish",
        "No more wood",
        "X",
        "and / or",
        "water {:>4.1} m/s  ground {:>4.1} m/s  slip {:>+4.1} m/s",
        "OFF",
        "NE",
    ] {
        assert!(!allowed_text(bad), "{bad:?} should be flagged");
    }
    assert_eq!(format_args_of("{a} {{b}} {c:.1}"), ["a", "c:.1"]);
    assert!(is_dev_module("//! F3 graph.\n//! locale: dev\n\nuse x;"));
    assert!(!is_dev_module("//! F3 graph.\n\n// locale: dev\n"));
}
