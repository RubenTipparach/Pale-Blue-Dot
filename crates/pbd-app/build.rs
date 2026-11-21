//! The git commit the build came from, as `PBD_GIT_COMMIT`, for a repro
//! record (`repro-capture`). Without git (a source tarball) nothing is set
//! and the record says "unknown".
//!
//! And the locale lint (`localization` design 4; the owner, 2026-11-20: "it
//! belongs in the build step"): a word the player could read written in code,
//! a key missing from `assets/locale/locale_en.yaml` or unused in it, or a
//! placeholder that does not match fails the build itself, not only the
//! tests. The rules are `locale_lint.rs`, shared with `tests/locale.rs`.

#[path = "locale_lint.rs"]
mod locale_lint;

use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

fn main() {
    locale();
    // Rerun when HEAD moves or the branch it names does, and not on every
    // source edit (the crate rebuilds for those anyway).
    for path in ["HEAD", "packed-refs"] {
        if let Some(file) = git(&["rev-parse", "--git-path", path]) {
            println!("cargo:rerun-if-changed={file}");
        }
    }
    if let Some(head) = git(&["symbolic-ref", "-q", "HEAD"])
        && let Some(file) = git(&["rev-parse", "--git-path", &head])
    {
        println!("cargo:rerun-if-changed={file}");
    }
    if let Some(commit) = git(&["rev-parse", "--short=12", "HEAD"]) {
        println!("cargo:rustc-env=PBD_GIT_COMMIT={commit}");
    }
}

/// Fail the build on text in code. Reruns when a source file, the locale
/// file or the lint itself changes.
fn locale() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for path in [
        "crates/pbd-app/src",
        "crates/pbd-core/src",
        "crates/pbd-app/locale_lint.rs",
        "assets/locale/locale_en.yaml",
        "tools/locale",
    ] {
        println!("cargo:rerun-if-changed={}", root.join(path).display());
    }
    let mut msg = locale_lint::words_in_code();
    let keys = locale_lint::keys_disagree();
    if !keys.is_empty() {
        msg.push_str("locale keys and code disagree:\n");
        msg.push_str(&keys);
    }
    if !msg.is_empty() {
        for line in msg.lines() {
            println!("cargo::error={line}");
        }
        std::process::exit(1);
    }
}
