//! Where the game's files are at run time (`itch-deploy` design 2 and 3).
//!
//! A shipped build is a folder holding the exe with `assets/` beside it. A dev
//! build is an exe under `target/` with no `assets/` beside it, which reads the
//! checkout it was compiled in. The choice is made once per run, so every
//! reader agrees on it.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The checkout's `assets/`, as compiled in. Tests and examples use the
/// checkout directly; the game goes through [`assets_dir`].
pub const DEV_ASSETS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets");

/// The dev build's saves folder, relative to the working directory: where
/// every dev world has always been written (`saves::ROOT`).
pub const DEV_SAVES: &str = "saves";

/// The folder of a shipped build: the exe's own directory, when it holds an
/// `assets` directory. `None` for a dev build.
pub fn shipped_root() -> Option<&'static Path> {
    static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    ROOT.get_or_init(|| shipped_in(std::env::current_exe().ok().as_deref()))
        .as_deref()
}

/// The rule, apart from the running exe so it can be tested: the directory of
/// `exe` when an `assets` directory sits in it.
fn shipped_in(exe: Option<&Path>) -> Option<PathBuf> {
    let dir = exe?.parent()?;
    dir.join("assets").is_dir().then(|| dir.to_path_buf())
}

/// The asset root: `<exe dir>/assets` in a shipped build, else the checkout's.
pub fn assets_dir() -> PathBuf {
    assets_for(shipped_root())
}

fn assets_for(shipped: Option<&Path>) -> PathBuf {
    shipped.map_or_else(|| PathBuf::from(DEV_ASSETS), |root| root.join("assets"))
}

/// Where the worlds live: `<exe dir>/saves` in a shipped build, whatever the
/// working directory; in a dev build `saves` in the working directory, exactly
/// as before, so every existing dev world opens where it is.
pub fn saves_dir() -> PathBuf {
    saves_for(shipped_root())
}

fn saves_for(shipped: Option<&Path>) -> PathBuf {
    shipped.map_or_else(|| PathBuf::from(DEV_SAVES), |root| root.join("saves"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pbd-paths-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// An exe with `assets/` beside it is a shipped build: assets and saves
    /// are both beside the exe.
    #[test]
    fn an_exe_beside_assets_is_shipped() {
        let dir = temp("shipped");
        std::fs::create_dir(dir.join("assets")).unwrap();
        let exe = dir.join("pbd-app");
        let root = shipped_in(Some(&exe));
        assert_eq!(root.as_deref(), Some(dir.as_path()));
        assert_eq!(assets_for(root.as_deref()), dir.join("assets"));
        assert_eq!(saves_for(root.as_deref()), dir.join("saves"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// An exe with no `assets/` beside it (a dev build in `target/`) reads the
    /// checkout's assets and keeps today's working-directory `saves`.
    #[test]
    fn an_exe_without_assets_is_a_dev_build() {
        let dir = temp("dev");
        let exe = dir.join("pbd-app");
        assert_eq!(shipped_in(Some(&exe)), None);
        assert_eq!(shipped_in(None), None);
        assert_eq!(assets_for(None), PathBuf::from(DEV_ASSETS));
        assert_eq!(saves_for(None), PathBuf::from(DEV_SAVES));
        // A FILE named assets is not a shipped build's folder.
        std::fs::write(dir.join("assets"), b"").unwrap();
        assert_eq!(shipped_in(Some(&exe)), None);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// The test exe lives in `target/<profile>/deps`, with no assets beside
    /// it, and the dev root is the checkout's real `assets/`.
    #[test]
    fn the_test_run_is_a_dev_build_on_the_real_assets() {
        assert_eq!(shipped_root(), None);
        assert!(
            assets_dir().join("config").is_dir(),
            "{}",
            assets_dir().display()
        );
        assert_eq!(saves_dir(), PathBuf::from(DEV_SAVES));
    }
}
