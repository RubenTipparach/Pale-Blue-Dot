//! The game's words at run time (`localization` design 2): `pbd-core::locale`
//! holds the current language, built in from `assets/locale/locale_en.yaml`;
//! this plugin reads the file from disk at start-up in a debug build, and
//! reloads it when it changes, so an edited word shows on the next screen
//! built without a rebuild. A release build keeps the built-in copy.

use bevy::prelude::*;
use std::path::PathBuf;
use std::time::SystemTime;

/// The locale file a debug build follows, and when it last changed.
#[derive(Resource, Debug)]
pub struct LocaleFile {
    pub path: PathBuf,
    pub modified: Option<SystemTime>,
    /// Seconds until the file is looked at again.
    pub check_in_s: f32,
}

/// How often a debug build looks at the file, seconds.
const CHECK_EVERY_S: f32 = 1.0;

/// The English file under the assets folder.
pub fn english_path() -> PathBuf {
    crate::paths::assets_dir()
        .join("locale")
        .join("locale_en.yaml")
}

fn modified(path: &PathBuf) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// Read the file and make it English, keeping the built-in copy on an error.
fn read(path: &PathBuf) {
    match std::fs::read_to_string(path).map(|t| pbd_core::locale::Locale::load(&t)) {
        Ok(Ok(locale)) => {
            info!("LOCALE loaded {} ({} keys)", path.display(), locale.len());
            pbd_core::locale::set_english(locale);
        }
        Ok(Err(e)) => warn!(
            "LOCALE {} does not parse, keeping the last: {e}",
            path.display()
        ),
        Err(e) => warn!(
            "LOCALE {} unreadable, keeping the built-in: {e}",
            path.display()
        ),
    }
}

/// Load the words at start-up and, in a debug build, follow the file.
pub struct LocalePlugin;

impl Plugin for LocalePlugin {
    fn build(&self, app: &mut App) {
        if !cfg!(debug_assertions) {
            return;
        }
        let path = english_path();
        read(&path);
        app.insert_resource(LocaleFile {
            modified: modified(&path),
            path,
            check_in_s: CHECK_EVERY_S,
        })
        .add_systems(Update, reload_locale);
    }
}

/// Reload the file when it changes (debug builds).
pub fn reload_locale(time: Res<Time>, mut file: ResMut<LocaleFile>) {
    file.check_in_s -= time.delta_secs();
    if file.check_in_s > 0.0 {
        return;
    }
    file.check_in_s = CHECK_EVERY_S;
    let now = modified(&file.path);
    if now != file.modified {
        file.modified = now;
        read(&file.path);
    }
}
