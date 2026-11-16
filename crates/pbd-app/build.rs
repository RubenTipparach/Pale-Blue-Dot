//! The git commit the build came from, as `PBD_GIT_COMMIT`, for a repro
//! record (`repro-capture`). Without git (a source tarball) nothing is set
//! and the record says "unknown".

use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

fn main() {
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
