//! The locale lint (`openspec/changes/localization` design 4): no words the
//! player could read are written in code. Every word comes from
//! `assets/locale/locale_en.yaml` by key, through `tr("key")` or
//! `tr_with("key", args)`.
//!
//! It parses every `crates/*/src/**/*.rs` and fails on:
//! - a string literal with a letter in it outside the allowed places (log and
//!   panic messages, tests, attributes, asset paths, ids and keys, render
//!   labels, format strings whose letters are all inside `{}`, `tr` keys, and
//!   modules marked `//! locale: dev`);
//! - a `Localized::legacy(..)` call, the transitional constructor, which is
//!   listed exactly as a literal is;
//! - a key used in code that is not in the file, a key in the file used
//!   nowhere, and a key whose placeholders differ from the names passed.
//!
//! Text from before the rule is listed in `tools/locale/baseline.txt`
//! (`path|literal`, sorted). The lint fails on a violation not in it and on
//! an entry no longer found, so the list only shrinks.
//!
//! `PBD_LOCALE_REPORT=<file.json>` writes every violation (baselined or not)
//! and every key use as JSON, for `tools/locale/`.

use pbd_core::locale::Locale;
use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{Expr, Lit, Token};

/// Key prefixes built from data ids at run time (`tr(&format!("item.{id}.name"))`,
/// `localization` phase 3). A key under one of these counts as used; each
/// table that builds them has a test that every id has its keys.
const DATA_PREFIXES: &[&str] = &[
    // `controls`: key caps by Bevy's key name, group headings and actions by id.
    "controls.key.",
    "controls.group.",
    "controls.does.",
    // `goods::GOODS`: each good's name by its saved code.
    "item.",
    // `inventory::Tool`: each tool's capitals and tiered names by code.
    "tool.",
    // `interact::ACTS`: each act's words by its verb id.
    "interact.act.",
    // `fauna` and `husbandry` species: name, field-guide entry and tip by id.
    "species.",
    // `inventory::material_name`: each block's name by its data id.
    "block.",
    // `fish::keys_for`: the fishing buttons' words.
    "fishing.key.",
    // `husbandry`: products, collections and life stages by their data names.
    "husbandry.collect.",
    "husbandry.collected.",
    "husbandry.waiting.",
    "husbandry.stage.",
    // `farming::CROPS` and `station::STATIONS`: names by saved code.
    "crop.",
    "station.",
    // `station` and `cooking` recipes: each plaque's words by recipe code.
    "recipe.",
    // `desktop::missions::TABS`: the strip's tabs, by key.
    "missions.tab.",
    // `world_map::PLACE_KEY`, `BIOME_KEY` and the map's layers: legend words
    // held as keys in the tables.
    "world_map.place.",
    "world_map.biome.",
    "map.layer.",
    // `vehicle::Kind`: each kind's name and title by its saved key.
    "vehicle.",
];

/// Extensions that make a spaceless literal an asset path.
const EXTENSIONS: &[&str] = &[
    ".png", ".ron", ".json", ".wgsl", ".ogg", ".mp3", ".wav", ".glb", ".gltf", ".ktx2", ".yaml",
    ".txt", ".csv", ".exe", ".toml", ".md", ".html", ".bin", ".jpg", ".jpeg", ".svg", ".mp4",
    ".log", ".py", ".ttf", ".otf", ".hdr", ".exr", ".dds", ".ps1", ".bat", ".sh", ".spv",
];

/// Macros whose words are for the developer: logs, panics, assertions,
/// compile-time inputs.
const DEV_MACROS: &[&str] = &[
    "info",
    "warn",
    "error",
    "debug",
    "trace",
    "println",
    "eprintln",
    "print",
    "eprint",
    "panic",
    "unreachable",
    "todo",
    "unimplemented",
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
    "include_str",
    "include_bytes",
    "env",
    "option_env",
    "compile_error",
    "info_span",
    "debug_span",
    "trace_span",
    "warn_span",
    "error_span",
    "span",
    "event",
    "dbg",
    "info_once",
    "warn_once",
    "error_once",
    "debug_once",
    "trace_once",
];

/// Methods whose string arguments are diagnostics or labels, never shown:
/// panics, an error's payload, a GPU object's debug label.
const DEV_METHODS: &[&str] = &[
    "expect",
    "expect_err",
    "ok_or",
    "ok_or_else",
    "map_err",
    "context",
    "with_context",
    "create_bind_group",
    "create_bind_group_layout",
    "push_debug_group",
    "insert_debug_marker",
];

/// Calls whose string arguments are names for the developer or the GPU: an
/// entity's debug name, an environment variable, a bind group's label.
///
/// `Err(..)`: an error's payload is a diagnostic (a config that does not
/// validate, a save that does not parse), logged, never shown. Text the
/// player reads never travels as an `Err` payload: a refusal the player
/// sees is a key (`tr`), or it is flagged where it is written.
///
/// `Status::Fail(..)`: why a showcase (`desktop/showcase.rs`) stopped, for
/// the log of a capture run.
const DEV_CALLS: &[&str] = &[
    "Err",
    "Status::Fail",
    "Name::new",
    "BindGroupLayoutDescriptor::new",
    "MeshVertexAttribute::new",
    "ShaderDefVal::from",
    "ShaderDefVal::Bool",
    "ShaderDefVal::Int",
    "ShaderDefVal::UInt",
    "env::var",
    "env::var_os",
    "env::set_var",
    "env::remove_var",
];

/// Places in one file whose literals are diagnostics or labels, never shown:
/// `(file, name)`, where the name is an enclosing function, or a function,
/// method or macro the literal is an argument of. Each is a false positive of
/// the general rules, fixed here rather than baselined.
const DEV_SCOPES: &[(&str, &str)] = &[
    // Config and data validators: why a file does not validate, for the log.
    ("crates/pbd-app/src/config.rs", "positive"),
    ("crates/pbd-app/src/config.rs", "non_negative"),
    ("crates/pbd-core/src/vehicle/spec.rs", "positive"),
    ("crates/pbd-app/src/vehicles/model/validate.rs", "require"),
    ("crates/pbd-core/src/audio.rs", "problem"),
    ("crates/pbd-core/src/audio.rs", "problems"),
    ("crates/pbd-core/src/sites.rs", "validate"),
    ("crates/pbd-core/src/fauna/mod.rs", "validate"),
    ("crates/pbd-core/src/settlement/chart.rs", "refuse"),
    ("crates/pbd-core/src/settlement/record.rs", "Damaged"),
    (
        "crates/pbd-core/src/settlement/pieces/dressing.rs",
        "shipyard",
    ),
    // Why a mission is not live yet: checked, logged, never shown.
    ("crates/pbd-core/src/missions.rs", "resolve"),
    // A town template's area names, matched as data.
    (
        "crates/pbd-core/src/settlement/pieces/steps.rs",
        "path_cells",
    ),
    // What a save's writer and record keep for the log.
    ("crates/pbd-app/src/saves/writer.rs", "send"),
    ("crates/pbd-app/src/saves/writer.rs", "drain"),
    ("crates/pbd-app/src/saves/writer.rs", "note"),
    ("crates/pbd-app/src/towns.rs", "unsettled_record"),
    ("crates/pbd-app/src/towns.rs", "stand"),
    // Timing and missing-edit labels for the log (`lod::spent`).
    ("crates/pbd-app/src/planet_lod.rs", "spent"),
    ("crates/pbd-app/src/planet_lod.rs", "report_missing_edits"),
    ("crates/pbd-app/src/planet.rs", "spent"),
    ("crates/pbd-app/src/planet.rs", "report_missing_edits"),
    ("crates/pbd-app/src/planet_tier_gpu.rs", "spent"),
    ("crates/pbd-app/src/field_light.rs", "spent"),
    // GPU objects' debug labels, made by a file's own helpers.
    ("crates/pbd-app/src/planet.rs", "view_bind_group"),
    ("crates/pbd-app/src/planet_shadow.rs", "list"),
    ("crates/pbd-app/src/planet_shadow.rs", "draw_group"),
    ("crates/pbd-app/src/planet_reflect.rs", "target"),
    ("crates/pbd-app/src/planet_weather.rs", "cube"),
    ("crates/pbd-app/src/planet_weather.rs", "pair"),
    ("crates/pbd-app/src/bloom.rs", "pipeline"),
    ("crates/pbd-app/src/planet_water.rs", "specialize"),
    ("crates/pbd-app/src/planet_water.rs", "run"),
    ("crates/pbd-app/src/loading.rs", "watch_pipelines"),
    // Threads' and entities' debug names.
    ("crates/pbd-app/src/river_fill.rs", "name"),
    ("crates/pbd-app/src/planet_trees.rs", "name"),
    ("crates/pbd-app/src/audio/mod.rs", "name"),
    ("crates/pbd-app/src/fish/places.rs", "child"),
    // The settings file's folder, from the environment.
    ("crates/pbd-app/src/settings.rs", "var"),
    // The boat panel's instruments, behind F3.
    ("crates/pbd-app/src/vehicles/hud.rs", "instruments"),
    // A record's author token in the save (`@p`, `@c`).
    ("crates/pbd-core/src/records.rs", "token"),
    // A Tenebris species' kind, an identifier from the reference roster.
    ("crates/pbd-core/src/fauna/mod.rs", "tenebris"),
    // The repro record's timestamp, for a bug report.
    ("crates/pbd-app/src/desktop/repro.rs", "save"),
    // The missions' scripted walk-through for a capture.
    ("crates/pbd-app/src/desktop/missions.rs", "script"),
    // The `--audio-log` line.
    ("crates/pbd-app/src/audio/mod.rs", "mix_frame"),
    // The debug readout's lines (behind the debug key) and the log.
    ("crates/pbd-app/src/planet_lod.rs", "line"),
    ("crates/pbd-app/src/place.rs", "line"),
    // Capture scripts' and the frame log's lines (`--capture`, `--frame-log`).
    ("crates/pbd-app/src/desktop.rs", "switch_line"),
    ("crates/pbd-app/src/desktop.rs", "river_walk_script"),
    ("crates/pbd-app/src/desktop.rs", "fish_script"),
    ("crates/pbd-app/src/desktop.rs", "open"),
    ("crates/pbd-app/src/desktop.rs", "verify_route"),
    ("crates/pbd-app/src/desktop.rs", "verify_flight"),
    ("crates/pbd-app/src/vehicles.rs", "report_steps"),
];

const HINT: &str = "move it to assets/locale/locale_en.yaml and call tr(\"key\")";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// One literal with words where words are not allowed, or one transitional
/// `Localized::legacy(..)` call.
#[derive(Clone, Debug)]
struct Violation {
    file: String,
    line: usize,
    col: usize,
    end_line: usize,
    end_col: usize,
    /// The literal's value, or `@legacy <tokens>`.
    text: String,
    kind: &'static str,
    /// The innermost call, method, macro or field it stands in (`Err`,
    /// `.push`, `format!`, `label:`), for the report.
    context: String,
    /// For a literal that is a `format!`'s format string: the macro's name
    /// and span, and whether every argument is an inline `{name}`.
    format: Option<FormatSite>,
}

#[derive(Clone, Debug)]
struct FormatSite {
    mac: String,
    line: usize,
    col: usize,
    end_line: usize,
    end_col: usize,
    /// The format string's arguments are all inline names (`{count}`), with
    /// no positional or explicit arguments and no format specs.
    simple: bool,
}

/// One `tr`/`tr_with` with a literal key.
#[derive(Clone, Debug)]
struct KeyUse {
    file: String,
    line: usize,
    key: String,
    /// The names passed to `tr_with`, when they are all visible; `Some(empty)`
    /// for `tr`.
    names: Option<BTreeSet<String>>,
}

/// Whether a literal's text may stand in code.
fn allowed_text(s: &str) -> bool {
    if !s.chars().any(char::is_alphabetic) {
        return true;
    }
    // An id or a key, or a list of them (a CSV header).
    if s.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "_.:-,".contains(c))
    {
        return true;
    }
    // A namespaced id (`interact:KeyF`), a constant's or a shader def's or an
    // environment variable's name (`SUN_DISC`, `PBD_BOUNCE`), or a camelCase
    // identifier (`PaleBlueDot`, `wingR`).
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || "_.:-,".contains(c))
    {
        let upper_snake = s.contains('_')
            && s.chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
        let camel = s.chars().all(|c| c.is_ascii_alphanumeric())
            && s.as_bytes()
                .windows(2)
                .any(|w| w[0].is_ascii_lowercase() && w[1].is_ascii_uppercase());
        if s.contains(':') || upper_snake || camel {
            return true;
        }
    }
    // An asset path.
    if !s.chars().any(char::is_whitespace) {
        let lower = s.to_ascii_lowercase();
        if s.contains('/') || s.contains('\\') || EXTENSIONS.iter().any(|e| lower.ends_with(e)) {
            return true;
        }
    }
    // A path with a space in a folder's name (`Library/Application
    // Support/PaleBlueDot`): segments joined by `/` with nothing round it.
    let first = s.split('/').next().unwrap_or_default();
    if s.split('/').count() > 2
        && !first.is_empty()
        && !first.contains(|c: char| c.is_whitespace() || c == '{')
        && s.split('/').all(|seg| {
            !seg.is_empty()
                && !seg.starts_with(char::is_whitespace)
                && !seg.ends_with(char::is_whitespace)
        })
    {
        return true;
    }
    // A format string with every letter inside its `{}`, or an id or a
    // path built by one (`landing-{set}-f`, `{dir}/{name}.png`).
    let bare = outside_braces(s);
    !bare.chars().any(char::is_alphabetic) || (bare.len() < s.len() && allowed_text(&bare))
}

/// `s` with each `{...}` removed.
fn outside_braces(s: &str) -> String {
    let mut out = String::new();
    let mut depth = 0;
    for c in s.chars() {
        match c {
            '{' => depth += 1,
            '}' if depth > 0 => depth -= 1,
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// The `{...}` arguments of a format string, `{{` escapes skipped.
fn format_args_of(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            if chars.peek() == Some(&'{') {
                chars.next();
                continue;
            }
            let mut arg = String::new();
            for d in chars.by_ref() {
                if d == '}' {
                    break;
                }
                arg.push(d);
            }
            out.push(arg);
        } else if c == '}' && chars.peek() == Some(&'}') {
            chars.next();
        }
    }
    out
}

/// The literal keys an expression picks between: the arms of a `match`, the
/// branches of an `if`, a block's value. Empty when any choice is not a
/// literal (a key built at run time, which a data prefix covers).
fn chosen_keys<'a>(expr: &'a Expr, out: &mut Vec<&'a syn::LitStr>) {
    fn walk<'a>(expr: &'a Expr, out: &mut Vec<&'a syn::LitStr>) -> bool {
        match expr {
            Expr::Lit(syn::ExprLit {
                lit: Lit::Str(s), ..
            }) => {
                out.push(s);
                true
            }
            Expr::Match(m) => m.arms.iter().all(|arm| walk(&arm.body, out)),
            Expr::If(i) => {
                let then = i.then_branch.stmts.last().is_some_and(|st| match st {
                    syn::Stmt::Expr(e, None) => walk(e, out),
                    _ => false,
                });
                then && i.else_branch.as_ref().is_some_and(|(_, e)| walk(e, out))
            }
            Expr::Block(b) => b.block.stmts.last().is_some_and(|st| match st {
                syn::Stmt::Expr(e, None) => walk(e, out),
                _ => false,
            }),
            Expr::Paren(p) => walk(&p.expr, out),
            _ => false,
        }
    }
    let mut found = Vec::new();
    if walk(expr, &mut found) {
        out.extend(found);
    }
}

fn is_ident(s: &str) -> bool {
    let mut c = s.chars();
    c.next().is_some_and(|f| f.is_alphabetic() || f == '_')
        && c.all(|d| d.is_alphanumeric() || d == '_')
}

/// An expression's tokens as one stable line for the baseline: no spaces,
/// and no trailing comma, so `rustfmt` cannot change it.
fn tokens_text(tokens: &TokenStream) -> String {
    let mut t: String = tokens.to_string().split_whitespace().collect();
    for close in [")", "]", "}"] {
        t = t.replace(&format!(",{close}"), close);
    }
    t
}

fn path_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn is_test_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        let p = a.path();
        if p.is_ident("test") {
            return true;
        }
        if p.is_ident("cfg")
            && let syn::Meta::List(list) = &a.meta
        {
            // `cfg(test)`, `cfg(all(test, ..))`; never `cfg(not(test))`.
            let t = list.tokens.to_string().replace(' ', "");
            return t == "test"
                || (t.starts_with("all(") && t.contains("(test,") && !t.contains("not("));
        }
        false
    })
}

struct Linter<'a> {
    file: &'a str,
    violations: Vec<Violation>,
    keys: Vec<KeyUse>,
    /// Out-of-line `#[cfg(test)] mod x;` declarations: the module names.
    test_mods: Vec<(String, Option<String>)>,
    /// The format macro whose format string is being visited.
    format: Option<FormatSite>,
    /// The calls, methods, macros and fields being visited, innermost last.
    ctx: Vec<String>,
    /// Inside a developer context (a log, a panic, an `Err`): literals pass,
    /// and `tr` keys are still recorded.
    dev: usize,
}

impl<'a> Linter<'a> {
    fn new(file: &'a str) -> Self {
        Self {
            file,
            violations: Vec::new(),
            keys: Vec::new(),
            test_mods: Vec::new(),
            format: None,
            ctx: Vec::new(),
            dev: 0,
        }
    }

    /// Whether `name` is one of this file's `DEV_SCOPES`.
    fn scoped(&self, name: &str) -> bool {
        DEV_SCOPES
            .iter()
            .any(|(file, n)| self.file == *file && *n == name)
    }

    fn check_lit(&mut self, lit: &syn::LitStr) {
        let value = lit.value();
        if self.dev > 0 || allowed_text(&value) {
            return;
        }
        let (start, end) = (lit.span().start(), lit.span().end());
        self.violations.push(Violation {
            file: self.file.to_owned(),
            line: start.line,
            col: start.column,
            end_line: end.line,
            end_col: end.column,
            text: value,
            kind: "literal",
            context: self
                .ctx
                .iter()
                .rev()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(" < "),
            format: self.format.take(),
        });
    }

    /// Literals in tokens that would not parse as expressions.
    fn check_tokens(&mut self, tokens: TokenStream) {
        let mut prev_tr: Option<String> = None;
        let mut hash = false;
        for tt in tokens {
            let after_hash = std::mem::take(&mut hash);
            match tt {
                // An attribute (`#[doc = ".."]` in a macro's body).
                TokenTree::Group(g)
                    if after_hash && g.delimiter() == proc_macro2::Delimiter::Bracket => {}
                TokenTree::Punct(p) if p.as_char() == '#' => {
                    hash = true;
                    prev_tr = None;
                }
                TokenTree::Group(g) => {
                    if let Some(name) = prev_tr.take()
                        && let Ok(args) = syn::parse::Parser::parse2(
                            Punctuated::<Expr, Token![,]>::parse_terminated,
                            g.stream(),
                        )
                    {
                        self.tr_call(&name, &args);
                        continue;
                    }
                    self.check_tokens(g.stream());
                }
                TokenTree::Literal(l) => {
                    if let Ok(Lit::Str(s)) = syn::parse2::<Lit>(TokenTree::Literal(l).into()) {
                        self.check_lit(&s);
                    }
                    prev_tr = None;
                }
                TokenTree::Ident(i) => {
                    let name = i.to_string();
                    prev_tr = (name == "tr" || name == "tr_with").then_some(name);
                }
                TokenTree::Punct(_) => prev_tr = None,
            }
        }
    }

    /// A `tr`/`tr_with` call: its key and the names it passes.
    fn tr_call(&mut self, name: &str, args: &Punctuated<Expr, Token![,]>) {
        let mut it = args.iter();
        let Some(first) = it.next() else { return };
        let key = match first {
            Expr::Lit(syn::ExprLit {
                lit: Lit::Str(s), ..
            }) => Some(s),
            _ => None,
        };
        let mut names = Some(BTreeSet::new());
        if name == "tr_with" {
            names = None;
            if let Some(arg) = it.next() {
                let mut found = BTreeSet::new();
                let mut all = true;
                let inner = match arg {
                    Expr::Reference(r) => &*r.expr,
                    e => e,
                };
                if let Expr::Array(arr) = inner {
                    for el in &arr.elems {
                        match el {
                            Expr::Tuple(t) if t.elems.len() == 2 => {
                                if let Expr::Lit(syn::ExprLit {
                                    lit: Lit::Str(n), ..
                                }) = &t.elems[0]
                                {
                                    found.insert(n.value());
                                } else {
                                    all = false;
                                }
                                self.visit_expr(&t.elems[1]);
                            }
                            e => {
                                all = false;
                                self.visit_expr(e);
                            }
                        }
                    }
                    if all {
                        names = Some(found);
                    }
                } else {
                    self.visit_expr(arg);
                }
            }
        }
        match key {
            Some(k) => self.keys.push(KeyUse {
                file: self.file.to_owned(),
                line: k.span().start().line,
                key: k.value(),
                names,
            }),
            None => {
                // A key chosen in place: `tr(match x { A => "a.key", .. })`.
                let mut chosen = Vec::new();
                chosen_keys(first, &mut chosen);
                if chosen.is_empty() {
                    self.visit_expr(first);
                }
                for k in chosen {
                    self.keys.push(KeyUse {
                        file: self.file.to_owned(),
                        line: k.span().start().line,
                        key: k.value(),
                        names: names.clone(),
                    });
                }
            }
        }
    }

    fn format_site(
        &self,
        mac: &syn::Macro,
        name: &str,
        args: &Punctuated<Expr, Token![,]>,
    ) -> Option<FormatSite> {
        let fmt_index = usize::from(matches!(name, "write" | "writeln"));
        let Some(Expr::Lit(syn::ExprLit {
            lit: Lit::Str(s), ..
        })) = args.iter().nth(fmt_index)
        else {
            return None;
        };
        let simple =
            args.len() == fmt_index + 1 && format_args_of(&s.value()).iter().all(|a| is_ident(a));
        let start = mac.path.segments.first()?.ident.span().start();
        let end = mac.delimiter.span().close().end();
        Some(FormatSite {
            mac: name.to_owned(),
            line: start.line,
            col: start.column,
            end_line: end.line,
            end_col: end.column,
            simple,
        })
    }
}

impl<'ast> Visit<'ast> for Linter<'_> {
    fn visit_attribute(&mut self, _: &'ast syn::Attribute) {}

    fn visit_pat(&mut self, _: &'ast syn::Pat) {}

    fn visit_item(&mut self, item: &'ast syn::Item) {
        let attrs: &[syn::Attribute] = match item {
            syn::Item::Fn(i) => &i.attrs,
            syn::Item::Mod(i) => &i.attrs,
            syn::Item::Impl(i) => &i.attrs,
            syn::Item::Const(i) => &i.attrs,
            syn::Item::Static(i) => &i.attrs,
            syn::Item::Struct(i) => &i.attrs,
            syn::Item::Enum(i) => &i.attrs,
            syn::Item::Use(i) => &i.attrs,
            syn::Item::Macro(i) => &i.attrs,
            syn::Item::Trait(i) => &i.attrs,
            _ => &[],
        };
        if is_test_attr(attrs) {
            if let syn::Item::Mod(m) = item
                && m.content.is_none()
            {
                let path = m.attrs.iter().find_map(|a| {
                    if a.path().is_ident("path")
                        && let syn::Meta::NameValue(nv) = &a.meta
                        && let Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(s), ..
                        }) = &nv.value
                    {
                        return Some(s.value());
                    }
                    None
                });
                self.test_mods.push((m.ident.to_string(), path));
            }
            return;
        }
        visit::visit_item(self, item);
    }

    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        if !is_test_attr(&f.attrs) {
            let dev = usize::from(self.scoped(&f.sig.ident.to_string()));
            self.dev += dev;
            visit::visit_impl_item_fn(self, f);
            self.dev -= dev;
        }
    }

    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        let dev = usize::from(self.scoped(&f.sig.ident.to_string()));
        self.dev += dev;
        visit::visit_item_fn(self, f);
        self.dev -= dev;
    }

    fn visit_expr_assign(&mut self, assign: &'ast syn::ExprAssign) {
        // A GPU object's debug label set after it is made:
        // `descriptor.label = Some("…")`.
        if let Expr::Field(f) = &*assign.left
            && matches!(&f.member, syn::Member::Named(n) if n == "label")
            && let Expr::Call(call) = &*assign.right
            && let Expr::Path(p) = &*call.func
            && p.path.is_ident("Some")
        {
            self.visit_expr(&assign.left);
            return;
        }
        visit::visit_expr_assign(self, assign);
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let Expr::Path(p) = &*call.func {
            let path = path_string(&p.path);
            let last = p.path.segments.last().map(|s| s.ident.to_string());
            match last.as_deref() {
                Some(name @ ("tr" | "tr_with")) => {
                    self.tr_call(name, &call.args);
                    return;
                }
                Some("legacy") if path.ends_with("Localized::legacy") => {
                    let arg = call.args.first();
                    let literal = matches!(
                        arg,
                        Some(Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(_),
                            ..
                        }))
                    );
                    if !literal {
                        let tokens = arg
                            .map(|a| tokens_text(&a.to_token_stream()))
                            .unwrap_or_default();
                        let (start, end) = (
                            p.path.segments.first().unwrap().ident.span().start(),
                            call.paren_token.span.close().end(),
                        );
                        self.violations.push(Violation {
                            file: self.file.to_owned(),
                            line: start.line,
                            col: start.column,
                            end_line: end.line,
                            end_col: end.column,
                            text: format!("@legacy {tokens}"),
                            kind: "legacy",
                            context: self
                                .ctx
                                .iter()
                                .rev()
                                .take(3)
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(" < "),
                            format: None,
                        });
                    }
                }
                Some(last)
                    if DEV_CALLS
                        .iter()
                        .any(|d| path == *d || path.ends_with(&format!("::{d}")))
                        || self.scoped(last) =>
                {
                    self.dev += 1;
                    visit::visit_expr_call(self, call);
                    self.dev -= 1;
                    return;
                }
                _ => {}
            }
        }
        let name = match &*call.func {
            Expr::Path(p) => path_string(&p.path),
            _ => "call".into(),
        };
        self.ctx.push(name);
        visit::visit_expr_call(self, call);
        self.ctx.pop();
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let method = call.method.to_string();
        if DEV_METHODS.contains(&method.as_str()) || self.scoped(&method) {
            self.visit_expr(&call.receiver);
            self.dev += 1;
            for arg in &call.args {
                self.visit_expr(arg);
            }
            self.dev -= 1;
            return;
        }
        self.ctx.push(format!(".{}", call.method));
        visit::visit_expr_method_call(self, call);
        self.ctx.pop();
    }

    fn visit_field_value(&mut self, field: &'ast syn::FieldValue) {
        // A GPU object's debug label: `label: Some("…")`. A bare
        // `label: "…"` may be a menu's word, and is checked.
        if let syn::Member::Named(name) = &field.member
            && name == "label"
            && let Expr::Call(call) = &field.expr
            && let Expr::Path(p) = &*call.func
            && p.path.is_ident("Some")
        {
            return;
        }
        let name = match &field.member {
            syn::Member::Named(n) => format!("{n}:"),
            syn::Member::Unnamed(i) => format!("{}:", i.index),
        };
        self.ctx.push(name);
        visit::visit_field_value(self, field);
        self.ctx.pop();
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let name = mac
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        let dev = DEV_MACROS.contains(&name.as_str()) || self.scoped(&name);
        self.dev += usize::from(dev);
        self.ctx.push(format!("{name}!"));
        self.visit_macro_body(mac, &name);
        self.ctx.pop();
        self.dev -= usize::from(dev);
    }

    fn visit_lit_str(&mut self, lit: &'ast syn::LitStr) {
        self.check_lit(lit);
    }
}

impl Linter<'_> {
    fn visit_macro_body(&mut self, mac: &syn::Macro, name: &str) {
        match mac.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated) {
            Ok(args) => {
                let site = self.format_site(mac, name, &args);
                for (i, arg) in args.iter().enumerate() {
                    let fmt_index = usize::from(matches!(name, "write" | "writeln"));
                    self.format = if i == fmt_index { site.clone() } else { None };
                    self.visit_expr(arg);
                    self.format = None;
                }
            }
            Err(_) => self.check_tokens(mac.tokens.clone()),
        }
    }
}

/// Every `.rs` under `dir`, sorted.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn rel(path: &Path) -> String {
    let root = root().canonicalize().unwrap();
    path.canonicalize()
        .unwrap()
        .strip_prefix(&root)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/")
}

/// Where a file's child modules live.
fn child_dir(file: &Path) -> PathBuf {
    let stem = file.file_stem().unwrap().to_string_lossy();
    let dir = file.parent().unwrap();
    if matches!(stem.as_ref(), "mod" | "lib" | "main") {
        dir.to_path_buf()
    } else {
        dir.join(stem.as_ref())
    }
}

/// A module whose leading inner docs carry `//! locale: dev`.
fn is_dev_module(source: &str) -> bool {
    source
        .lines()
        .take_while(|l| l.trim_start().starts_with("//!") || l.trim().is_empty())
        .any(|l| l.trim() == "//! locale: dev")
}

struct Scan {
    violations: Vec<Violation>,
    keys: Vec<KeyUse>,
}

fn scan() -> Scan {
    let root = root();
    let mut files = Vec::new();
    let mut crates: Vec<_> = std::fs::read_dir(root.join("crates"))
        .unwrap()
        .flatten()
        .map(|e| e.path().join("src"))
        .collect();
    crates.sort();
    for src in &crates {
        rust_files(src, &mut files);
    }
    // Pass 1: parse, and find the files that are test modules.
    let mut parsed = Vec::new();
    let mut test_paths: Vec<PathBuf> = Vec::new();
    for path in &files {
        let source = std::fs::read_to_string(path).unwrap();
        let tree = syn::parse_file(&source)
            .unwrap_or_else(|e| panic!("{}: does not parse: {e}", path.display()));
        let name = rel(path);
        let mut finder = Linter::new(&name);
        finder.visit_file(&tree);
        for (module, at) in finder.test_mods {
            match at {
                Some(p) => test_paths.push(path.parent().unwrap().join(p)),
                None => {
                    let dir = child_dir(path);
                    test_paths.push(dir.join(format!("{module}.rs")));
                    test_paths.push(dir.join(&module));
                }
            }
        }
        parsed.push((path.clone(), source, tree));
    }
    let test_paths: Vec<String> = test_paths
        .iter()
        .filter(|p| p.exists())
        .map(|p| rel(p))
        .collect();
    let in_test = |name: &str| {
        test_paths
            .iter()
            .any(|t| name == t || name.starts_with(&format!("{t}/")))
    };
    // Pass 2: lint.
    let mut out = Scan {
        violations: Vec::new(),
        keys: Vec::new(),
    };
    for (path, source, tree) in &parsed {
        let name = rel(path);
        if in_test(&name) {
            continue;
        }
        let mut linter = Linter::new(&name);
        linter.visit_file(tree);
        out.keys.extend(linter.keys);
        if !is_dev_module(source) {
            out.violations.extend(linter.violations);
        }
    }
    out
}

/// A baseline line's literal, escaped onto one line.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '|' => out.push_str("\\|"),
            c => out.push(c),
        }
    }
    out
}

fn baseline_line(v: &Violation) -> String {
    format!("{}|{}", v.file, escape(&v.text))
}

fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn write_report(path: &str, scan: &Scan) {
    let mut out = String::from("{\"violations\":[\n");
    for (i, v) in scan.violations.iter().enumerate() {
        let format = match &v.format {
            Some(f) => format!(
                "{{\"mac\":{},\"line\":{},\"col\":{},\"end_line\":{},\"end_col\":{},\"simple\":{}}}",
                json_str(&f.mac),
                f.line,
                f.col,
                f.end_line,
                f.end_col,
                f.simple
            ),
            None => "null".into(),
        };
        let _ = write!(
            out,
            "{}{{\"file\":{},\"line\":{},\"col\":{},\"end_line\":{},\"end_col\":{},\"kind\":{},\"text\":{},\"baseline\":{},\"context\":{},\"format\":{}}}",
            if i == 0 { "" } else { ",\n" },
            json_str(&v.file),
            v.line,
            v.col,
            v.end_line,
            v.end_col,
            json_str(v.kind),
            json_str(&v.text),
            json_str(&baseline_line(v)),
            json_str(&v.context),
            format
        );
    }
    out.push_str("\n],\"keys\":[\n");
    for (i, k) in scan.keys.iter().enumerate() {
        let names = match &k.names {
            Some(n) => format!(
                "[{}]",
                n.iter().map(|s| json_str(s)).collect::<Vec<_>>().join(",")
            ),
            None => "null".into(),
        };
        let _ = write!(
            out,
            "{}{{\"file\":{},\"line\":{},\"key\":{},\"names\":{}}}",
            if i == 0 { "" } else { ",\n" },
            json_str(&k.file),
            k.line,
            json_str(&k.key),
            names
        );
    }
    out.push_str("\n]}\n");
    std::fs::write(path, out).unwrap();
}

fn read_baseline() -> Vec<String> {
    let path = root().join("tools/locale/baseline.txt");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    text.lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

#[test]
fn no_words_in_code() {
    let scan = scan();
    if let Ok(path) = std::env::var("PBD_LOCALE_REPORT") {
        write_report(&path, &scan);
    }
    let baseline = read_baseline();
    let mut sorted = baseline.clone();
    sorted.sort();
    assert!(
        sorted == baseline,
        "tools/locale/baseline.txt is not sorted (sort it; never add to it)"
    );
    // Multisets: the same literal may stand twice in a file.
    let mut allowed: BTreeMap<&str, usize> = BTreeMap::new();
    for line in &baseline {
        *allowed.entry(line.as_str()).or_default() += 1;
    }
    let mut new = Vec::new();
    for v in &scan.violations {
        let line = baseline_line(v);
        match allowed.get_mut(line.as_str()) {
            Some(n) if *n > 0 => *n -= 1,
            _ => new.push(v),
        }
    }
    let stale: Vec<_> = allowed
        .iter()
        .filter(|(_, n)| **n > 0)
        .map(|(l, n)| format!("  {l}  (x{n})"))
        .collect();
    let mut msg = String::new();
    if !new.is_empty() {
        let _ = writeln!(
            msg,
            "{} string literal(s) with words in code. Every word the player reads comes from \
             assets/locale/locale_en.yaml (CLAUDE.md, \"No text in code\"):",
            new.len()
        );
        for v in &new {
            let what = if v.kind == "legacy" {
                "Localized::legacy(..) is transitional; use tr(\"key\")".to_owned()
            } else {
                HINT.to_owned()
            };
            let _ = writeln!(
                msg,
                "  {}:{}: {}\n      {what}",
                v.file,
                v.line,
                json_str(&v.text)
            );
        }
        let _ = writeln!(
            msg,
            "  (a log, panic or developer label that is flagged is a false positive: fix the \
             lint's rules in crates/pbd-app/tests/locale.rs, never the baseline; \
             tools/locale/locale_extract.py <file> moves a file's text for you)"
        );
    }
    if !stale.is_empty() {
        let _ = writeln!(
            msg,
            "{} baseline entr(y/ies) no longer found: remove them from tools/locale/baseline.txt \
             (python3 tools/locale/locale_lint.py --baseline):\n{}",
            stale.len(),
            stale.join("\n")
        );
    }
    assert!(msg.is_empty(), "\n{msg}");
}

#[test]
fn keys_agree_with_the_file() {
    let yaml = std::fs::read_to_string(root().join("assets/locale/locale_en.yaml")).unwrap();
    let locale = Locale::load(&yaml).expect("assets/locale/locale_en.yaml parses");
    let scan = scan();
    let mut msg = String::new();
    let mut used = HashSet::new();
    for k in &scan.keys {
        used.insert(k.key.as_str());
        let Some(entry) = locale.get(&k.key) else {
            let _ = writeln!(
                msg,
                "  {}:{}: key \"{}\" is not in assets/locale/locale_en.yaml",
                k.file, k.line, k.key
            );
            continue;
        };
        if let Some(names) = &k.names {
            let want = entry.placeholders();
            if &want != names {
                let _ = writeln!(
                    msg,
                    "  {}:{}: key \"{}\" has placeholders {:?} but is given {:?}",
                    k.file, k.line, k.key, want, names
                );
            }
        }
    }
    for key in locale.keys() {
        if !used.contains(key) && !DATA_PREFIXES.iter().any(|p| key.starts_with(p)) {
            let _ = writeln!(
                msg,
                "  assets/locale/locale_en.yaml: key \"{key}\" is used nowhere in code (remove it, \
                 or add its data prefix to DATA_PREFIXES in crates/pbd-app/tests/locale.rs)"
            );
        }
    }
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
