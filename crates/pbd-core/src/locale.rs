//! The words the player reads (`localization` design 2): every one comes from
//! `assets/locale/locale_<code>.yaml` by a dotted key, never from a literal in
//! code.
//!
//! - The file is a nested map; its dotted paths are the keys
//!   (`hud.hotbar`, `menu.picture.antialiasing`).
//! - `{name}` placeholders are filled by name, so a translation can reorder
//!   them.
//! - A map of plural forms (`one`, `other`, and `zero`, `two`, `few`, `many`
//!   where a language has them) is chosen by the `count` argument.
//! - [`Localized`] is what the UI kit takes. It has no `From<&str>`: a literal
//!   reaches the screen only through [`tr`], [`tr_with`] or, for text that is
//!   not words (a number, an arrow, a name the player typed),
//!   [`Localized::verbatim`].
//!
//! A missing key shows as `⟦key⟧` in a debug build and logs
//! `LOCALE_MISSING key` once; a release build falls back to English, then to
//! the key itself. The lint (`crates/pbd-app/tests/locale.rs`) stops a missing
//! key before a push anyway.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

/// The English file, built in, so the game, its tests and its tools have the
/// words without an asset server. The app may replace it from disk in a debug
/// build (`pbd-app`'s `locale` module).
pub const ENGLISH_YAML: &str = include_str!("../../../assets/locale/locale_en.yaml");

/// The plural forms a plural map may hold (CLDR's categories).
pub const PLURAL_FORMS: [&str; 6] = ["zero", "one", "two", "few", "many", "other"];

/// Text the player reads, made only by [`tr`], [`tr_with`] or
/// [`Localized::verbatim`].
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Localized(String);

impl Localized {
    /// Text that is not words and needs no translation: a number, an arrow,
    /// a name the player typed. The lint still flags a literal with letters
    /// handed to it.
    pub fn verbatim(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// No text.
    pub fn empty() -> Self {
        Self(String::new())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for Localized {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::ops::Deref for Localized {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl From<Localized> for String {
    fn from(text: Localized) -> Self {
        text.0
    }
}

impl AsRef<str> for Localized {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for Localized {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for Localized {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for Localized {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}

/// One key's text: a line, or a line per plural form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Text(String),
    Plural(BTreeMap<String, String>),
}

impl Entry {
    /// Every `{name}` its text (or any of its forms) holds.
    pub fn placeholders(&self) -> BTreeSet<String> {
        match self {
            Entry::Text(t) => placeholders(t),
            Entry::Plural(forms) => forms.values().flat_map(|t| placeholders(t)).collect(),
        }
    }
}

/// A language's words by dotted key.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Locale {
    entries: BTreeMap<String, Entry>,
}

/// Why a locale file could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocaleError(pub String);

impl fmt::Display for LocaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for LocaleError {}

impl Locale {
    /// Read a locale file: a nested map of strings, whose leaves are text or
    /// maps of plural forms.
    pub fn load(yaml: &str) -> Result<Self, LocaleError> {
        let root: serde_yaml_ng::Value =
            serde_yaml_ng::from_str(yaml).map_err(|e| LocaleError(e.to_string()))?;
        let mut entries = BTreeMap::new();
        match &root {
            serde_yaml_ng::Value::Null => {}
            serde_yaml_ng::Value::Mapping(_) => flatten("", &root, &mut entries)?,
            _ => return Err(LocaleError("the top of a locale file is a map".into())),
        }
        Ok(Self { entries })
    }

    /// The built-in English.
    pub fn english() -> Self {
        Self::load(ENGLISH_YAML).expect("assets/locale/locale_en.yaml parses")
    }

    pub fn get(&self, key: &str) -> Option<&Entry> {
        self.entries.get(key)
    }

    /// Every key, sorted.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// `key`'s text with `args` filled in, or `None` when the key is absent.
    /// A plural entry takes its form from the `count` argument (`other`
    /// without one).
    pub fn format(&self, key: &str, args: &[(&str, &dyn fmt::Display)]) -> Option<String> {
        let text = match self.entries.get(key)? {
            Entry::Text(t) => t.as_str(),
            Entry::Plural(forms) => {
                let count = args
                    .iter()
                    .find(|(name, _)| *name == "count")
                    .and_then(|(_, v)| v.to_string().trim().parse::<f64>().ok());
                let form = count.map_or("other", english_plural);
                forms
                    .get(form)
                    .or_else(|| forms.get("other"))
                    .map(String::as_str)?
            }
        };
        Some(fill(text, args))
    }
}

/// English's plural rule: one for exactly 1, other otherwise.
fn english_plural(count: f64) -> &'static str {
    if count == 1.0 { "one" } else { "other" }
}

fn flatten(
    prefix: &str,
    value: &serde_yaml_ng::Value,
    out: &mut BTreeMap<String, Entry>,
) -> Result<(), LocaleError> {
    use serde_yaml_ng::Value;
    let Value::Mapping(map) = value else {
        unreachable!("flatten is called on maps")
    };
    let mut names = Vec::with_capacity(map.len());
    for (k, _) in map {
        names.push(
            k.as_str()
                .ok_or_else(|| LocaleError(format!("a key under '{prefix}' is not a string")))?
                .to_owned(),
        );
    }
    let plural = !names.is_empty()
        && names.iter().any(|n| n == "other")
        && names.iter().all(|n| PLURAL_FORMS.contains(&n.as_str()))
        && map.iter().all(|(_, v)| leaf_text(v).is_some());
    if plural && !prefix.is_empty() {
        let forms = map
            .iter()
            .map(|(k, v)| (k.as_str().unwrap().to_owned(), leaf_text(v).unwrap()))
            .collect();
        out.insert(prefix.to_owned(), Entry::Plural(forms));
        return Ok(());
    }
    for ((_, v), name) in map.iter().zip(names) {
        let key = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}.{name}")
        };
        match v {
            Value::Mapping(_) => flatten(&key, v, out)?,
            _ => {
                let text = leaf_text(v)
                    .ok_or_else(|| LocaleError(format!("'{key}' is not text or a map")))?;
                out.insert(key, Entry::Text(text));
            }
        }
    }
    Ok(())
}

fn leaf_text(v: &serde_yaml_ng::Value) -> Option<String> {
    use serde_yaml_ng::Value;
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// The `{name}` placeholders in `text`. `{{` and `}}` are literal braces.
pub fn placeholders(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = text;
    while let Some(i) = rest.find(['{', '}']) {
        let tail = &rest[i..];
        if tail.starts_with("{{") || tail.starts_with("}}") {
            rest = &tail[2..];
            continue;
        }
        if tail.starts_with('{')
            && let Some(end) = tail.find('}')
        {
            out.insert(tail[1..end].trim().to_owned());
            rest = &tail[end + 1..];
            continue;
        }
        rest = &tail[1..];
    }
    out
}

/// `text` with each `{name}` replaced by its argument. An unknown name stays
/// as written, so a mistake shows rather than vanishing.
pub fn fill(text: &str, args: &[(&str, &dyn fmt::Display)]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find(['{', '}']) {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        if let Some(after) = tail.strip_prefix("{{") {
            out.push('{');
            rest = after;
        } else if let Some(after) = tail.strip_prefix("}}") {
            out.push('}');
            rest = after;
        } else if tail.starts_with('{')
            && let Some(end) = tail.find('}')
        {
            let name = tail[1..end].trim();
            match args.iter().find(|(n, _)| *n == name) {
                Some((_, v)) => out.push_str(&v.to_string()),
                None => out.push_str(&tail[..=end]),
            }
            rest = &tail[end + 1..];
        } else {
            out.push_str(&tail[..1]);
            rest = &tail[1..];
        }
    }
    out.push_str(rest);
    out
}

// ---------------------------------------------------------------- the current locale

struct Current {
    chosen: Arc<Locale>,
    english: Arc<Locale>,
}

fn current() -> &'static RwLock<Current> {
    static CURRENT: OnceLock<RwLock<Current>> = OnceLock::new();
    CURRENT.get_or_init(|| {
        let english = Arc::new(Locale::english());
        RwLock::new(Current {
            chosen: english.clone(),
            english,
        })
    })
}

/// Make `locale` the one [`tr`] reads. English stays behind it for a key it
/// lacks (in a release build).
pub fn set_current(locale: Locale) {
    let mut cur = current().write().unwrap_or_else(|e| e.into_inner());
    cur.chosen = Arc::new(locale);
}

/// Replace the English fallback too: the app's debug build reads the file
/// from disk so an edit shows without a rebuild.
pub fn set_english(locale: Locale) {
    let mut cur = current().write().unwrap_or_else(|e| e.into_inner());
    let english = Arc::new(locale);
    cur.chosen = english.clone();
    cur.english = english;
}

/// Read `yaml` and make it current.
pub fn load_current(yaml: &str) -> Result<(), LocaleError> {
    set_current(Locale::load(yaml)?);
    Ok(())
}

/// The current locale.
pub fn current_locale() -> Arc<Locale> {
    current()
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .chosen
        .clone()
}

/// What a missing key shows: `⟦key⟧` in a debug build, the key in release.
fn missing(key: &str) -> String {
    static LOGGED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    let logged = LOGGED.get_or_init(|| Mutex::new(HashSet::new()));
    if logged
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(key.to_owned())
    {
        eprintln!("LOCALE_MISSING {key}");
    }
    if cfg!(debug_assertions) {
        format!("⟦{key}⟧")
    } else {
        key.to_owned()
    }
}

/// `key`'s text in the built-in English, whatever the current language: for
/// a name that data and saves use as an identifier ("block:dry grass" in a
/// mission, a caught species' name in its progress), which must not change
/// with the player's language. Never shown.
pub fn english(key: &str) -> String {
    let cur = current().read().unwrap_or_else(|e| e.into_inner());
    cur.english
        .format(key, &[])
        .unwrap_or_else(|| key.to_owned())
}

/// A line in capitals, by the language's own rule, for text composed from
/// data names that the file cannot hold in capitals (the interact prompt
/// names any thing: "HARVEST WHEAT"). Text written whole is written in its
/// case in the file; this is only for a composed line.
pub fn caps(text: &Localized) -> Localized {
    Localized(text.0.to_uppercase())
}

/// A data name with its first letter in capitals, by the language's own
/// rule, where it starts a line (a slot's tooltip: "Dirt x 3"). Like
/// [`caps`], only for a composed line.
pub fn sentence(text: &Localized) -> Localized {
    let mut letters = text.0.chars();
    Localized(
        letters
            .next()
            .map(|first| first.to_uppercase().chain(letters).collect())
            .unwrap_or_default(),
    )
}

/// The text of `key` in the current locale.
pub fn tr(key: &str) -> Localized {
    tr_with(key, &[])
}

/// The text of `key` with its `{name}` placeholders filled from `args`, and
/// its plural form chosen by the `count` argument.
pub fn tr_with(key: &str, args: &[(&str, &dyn fmt::Display)]) -> Localized {
    let cur = current().read().unwrap_or_else(|e| e.into_inner());
    if let Some(text) = cur.chosen.format(key, args) {
        return Localized(text);
    }
    if !cfg!(debug_assertions)
        && let Some(text) = cur.english.format(key, args)
    {
        return Localized(text);
    }
    drop(cur);
    Localized(missing(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
menu:
  picture:
    antialiasing: Antialiasing
hud:
  hotbar: HOTBAR
fishing:
  caught: "Caught a {species}"
  catch_n:
    one: "Catch {count} fish"
    other: "Catch {count} fish each"
  reorder: "{b} then {a}"
  braces: "{{literal}} {x}"
"#;

    #[test]
    fn nested_maps_become_dotted_keys() {
        let l = Locale::load(SAMPLE).unwrap();
        let keys: Vec<_> = l.keys().collect();
        assert_eq!(
            keys,
            [
                "fishing.braces",
                "fishing.catch_n",
                "fishing.caught",
                "fishing.reorder",
                "hud.hotbar",
                "menu.picture.antialiasing",
            ]
        );
        assert_eq!(l.format("hud.hotbar", &[]).as_deref(), Some("HOTBAR"));
    }

    #[test]
    fn placeholders_are_filled_by_name() {
        let l = Locale::load(SAMPLE).unwrap();
        assert_eq!(
            l.format("fishing.caught", &[("species", &"trout")])
                .unwrap(),
            "Caught a trout"
        );
        assert_eq!(
            l.format("fishing.reorder", &[("a", &1), ("b", &2)])
                .unwrap(),
            "2 then 1"
        );
        assert_eq!(
            l.format("fishing.braces", &[("x", &7)]).unwrap(),
            "{literal} 7"
        );
        // An unknown name stays as written.
        assert_eq!(
            l.format("fishing.caught", &[]).unwrap(),
            "Caught a {species}"
        );
    }

    #[test]
    fn plurals_follow_count() {
        let l = Locale::load(SAMPLE).unwrap();
        assert_eq!(
            l.format("fishing.catch_n", &[("count", &1)]).unwrap(),
            "Catch 1 fish"
        );
        assert_eq!(
            l.format("fishing.catch_n", &[("count", &3)]).unwrap(),
            "Catch 3 fish each"
        );
        assert_eq!(
            l.format("fishing.catch_n", &[]).unwrap(),
            "Catch {count} fish each"
        );
        assert_eq!(
            l.get("fishing.catch_n").unwrap().placeholders(),
            BTreeSet::from(["count".to_owned()])
        );
    }

    #[test]
    fn placeholders_are_found() {
        assert_eq!(
            placeholders("{a} and {b} but {{not}}"),
            BTreeSet::from(["a".to_owned(), "b".to_owned()])
        );
        assert!(placeholders("no braces").is_empty());
    }

    #[test]
    fn bad_files_are_refused() {
        assert!(Locale::load("- a list").is_err());
        assert!(Locale::load("a: [1, 2]").is_err());
        assert!(Locale::load("").unwrap().is_empty());
    }

    #[test]
    fn english_is_built_in_and_tr_reads_it() {
        let en = Locale::english();
        assert!(!en.is_empty());
        let key = en.keys().next().unwrap().to_owned();
        assert!(!tr(&key).as_str().starts_with('⟦'));
    }

    #[test]
    fn a_missing_key_is_loud_in_debug() {
        let shown = tr("locale.test.no_such_key");
        if cfg!(debug_assertions) {
            assert_eq!(shown.as_str(), "⟦locale.test.no_such_key⟧");
        } else {
            assert_eq!(shown.as_str(), "locale.test.no_such_key");
        }
    }

    #[test]
    fn localized_compares_with_text() {
        let l = Localized::verbatim("12");
        assert_eq!(l, "12");
        assert_eq!(l, String::from("12"));
        assert_eq!(l.to_string(), "12");
        assert!(Localized::empty().is_empty());
    }
}
