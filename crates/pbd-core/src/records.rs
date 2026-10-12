//! What a world stores beyond its edits: records, who changed what, and the
//! rule that the world yields to the player (`world-persistence` decisions 5,
//! 6 and 11).
//!
//! A **record** is something generated once that must then stay put or be
//! able to change: a site, later a settlement, a landmark, a named zone. It
//! has a kind, a stable id, its kind's schema version and a body. The body is
//! one line of RON kept as TEXT: a build parses the bodies of the kinds it
//! knows and holds every other record as it found it, so a newer save opened
//! by an older build loses nothing.
//!
//! Every line of the journal has an **author**: the player, a world process
//! acting for a record, or the world's creation. A player's line may carry no
//! token at all, which is how every line was written before authors, and
//! reads as player 0's.
//!
//! The **yield set** is what the player has touched: cells, and record
//! fields. A world process proposes changes; the proposal is checked whole
//! against the set and refused whole if it would change anything the player
//! did.

use crate::edits::Edit;
use std::collections::{BTreeMap, BTreeSet};

/// A record line's leading word, which no cell number can be.
pub const REC: &str = "rec";

/// Who made a journal entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Author {
    /// A player; 0 is the single player.
    Player(u32),
    /// The world's creation: what is generated once when a world is made or
    /// first opened under a new version.
    Creation,
    /// A world process, acting for one record.
    World {
        process: String,
        kind: String,
        id: u64,
    },
}

impl Author {
    /// The line's leading token: `@p`, `@p<n>`, `@c`, or
    /// `@w<process>:<kind>/<id>`.
    pub fn token(&self) -> String {
        match self {
            Author::Player(0) => "@p".into(),
            Author::Player(n) => format!("@p{n}"),
            Author::Creation => "@c".into(),
            Author::World { process, kind, id } => format!("@w{process}:{kind}/{id}"),
        }
    }

    /// The author a token names, or `None` for a token that names nobody.
    pub fn parse(token: &str) -> Option<Self> {
        let rest = token.strip_prefix('@')?;
        if rest == "c" {
            return Some(Author::Creation);
        }
        if let Some(n) = rest.strip_prefix('p') {
            return if n.is_empty() {
                Some(Author::Player(0))
            } else {
                n.parse().ok().map(Author::Player)
            };
        }
        let (process, record) = rest.strip_prefix('w')?.split_once(':')?;
        let (kind, id) = record.split_once('/')?;
        if !is_word(process) || !is_word(kind) {
            return None;
        }
        Some(Author::World {
            process: process.into(),
            kind: kind.into(),
            id: id.parse().ok()?,
        })
    }

    pub fn is_player(&self) -> bool {
        matches!(self, Author::Player(_))
    }
}

/// A kind's or a process's name: lower-case letters, digits and hyphens.
pub fn is_word(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// A line's author and the rest of it. A line with no leading token is the
/// player's (every line before authors). `None` for a token that names
/// nobody, which makes the line damaged, not the player's.
pub fn split_author(line: &str) -> Option<(Author, &str)> {
    let line = line.trim_start();
    if !line.starts_with('@') {
        return Some((Author::Player(0), line));
    }
    let (token, rest) = line.split_once(char::is_whitespace)?;
    Some((Author::parse(token)?, rest.trim_start()))
}

/// One stored thing: generated once, then kept or changed through the
/// journal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    /// What it is: `site`, `site-list`; later `settlement`, `landmark`.
    pub kind: String,
    /// Stable for the life of the world: a site's is its anchor cell.
    pub id: u64,
    /// The kind's schema version the body is written in.
    pub schema: u32,
    /// One line of RON, held as text.
    pub body: String,
}

impl Record {
    /// A record of a kind this build writes, its body serialised.
    pub fn of<T: serde::Serialize>(kind: &str, id: u64, schema: u32, body: &T) -> Self {
        Self {
            kind: kind.into(),
            id,
            schema,
            body: ron::to_string(body).expect("a record body is plain data"),
        }
    }

    /// The body read as a kind this build knows.
    pub fn read<T: serde::de::DeserializeOwned>(&self) -> Option<T> {
        ron::from_str(&self.body).ok()
    }

    pub fn key(&self) -> (String, u64) {
        (self.kind.clone(), self.id)
    }

    /// `rec <author> <kind> <id> <schema> <body>`, one line. The author
    /// is always written on a record line.
    pub fn line(&self, author: &Author) -> String {
        format!(
            "{REC} {} {} {} {} {}\n",
            author.token(),
            self.kind,
            self.id,
            self.schema,
            self.body
        )
    }

    /// A record line's author and record, or `None` where the line is not
    /// one or is damaged. The body is everything after the schema, byte for
    /// byte, so a body of a kind this build does not know comes back as it
    /// was.
    pub fn parse_line(line: &str) -> Option<(Author, Record)> {
        let line = line.trim_end_matches(['\n', '\r']);
        let rest = line.strip_prefix(REC)?.strip_prefix(' ')?;
        let (token, rest) = rest.split_once(' ')?;
        let author = Author::parse(token)?;
        let (kind, rest) = rest.split_once(' ')?;
        let (id, rest) = rest.split_once(' ')?;
        let (schema, body) = rest.split_once(' ')?;
        // Every body is RON, whatever its kind, so a body that is not is a
        // line torn by a crash (or two lines run together after one), and is
        // damaged rather than a record with half a value.
        if !is_word(kind) || ron::from_str::<ron::Value>(body).is_err() {
            return None;
        }
        Some((
            author,
            Record {
                kind: kind.into(),
                id: id.parse().ok()?,
                schema: schema.parse().ok()?,
                body: body.into(),
            },
        ))
    }
}

/// Every record in a world, by kind and id: the last value the journal gave
/// each.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Records {
    by_key: BTreeMap<(String, u64), Record>,
}

impl Records {
    pub fn new() -> Self {
        Self::default()
    }

    /// Put a record's new value, returning the one it replaced.
    pub fn put(&mut self, record: Record) -> Option<Record> {
        self.by_key.insert(record.key(), record)
    }

    pub fn get(&self, kind: &str, id: u64) -> Option<&Record> {
        self.by_key.get(&(kind.to_string(), id))
    }

    /// Every record of a kind, by id.
    pub fn of_kind<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a Record> + 'a {
        self.by_key
            .range((kind.to_string(), 0)..=(kind.to_string(), u64::MAX))
            .map(|(_, r)| r)
    }

    /// Every record, by kind and id.
    pub fn iter(&self) -> impl Iterator<Item = &Record> {
        self.by_key.values()
    }

    pub fn len(&self) -> usize {
        self.by_key.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_key.is_empty()
    }
}

/// A body's top-level fields that differ from `before`'s, or every field
/// where there was no `before`. A body that is not a RON struct is one field,
/// `*`: the record as a whole.
pub fn changed_fields(before: Option<&str>, after: &str) -> BTreeSet<String> {
    let fields = |body: &str| -> Option<BTreeMap<String, ron::Value>> {
        match ron::from_str::<ron::Value>(body).ok()? {
            ron::Value::Map(map) => map
                .into_iter()
                .map(|(k, v)| match k {
                    ron::Value::String(name) => Some((name, v)),
                    _ => None,
                })
                .collect(),
            _ => None,
        }
    };
    let Some(new) = fields(after) else {
        return BTreeSet::from(["*".to_string()]);
    };
    let old = before.and_then(fields).unwrap_or_default();
    new.into_iter()
        .filter(|(name, value)| old.get(name) != Some(value))
        .map(|(name, _)| name)
        .chain(
            before
                .filter(|b| fields(b).is_none())
                .map(|_| "*".to_string()),
        )
        .collect()
}

/// A world process's changes for one period: cell edits and records, taken
/// or refused as one.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Proposal {
    pub edits: Vec<Edit>,
    pub records: Vec<Record>,
}

/// Why a proposal was refused: the first thing it would have changed that
/// the player had.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    Cell(u32),
    Field {
        kind: String,
        id: u64,
        field: String,
    },
}

/// What the player has touched, which the world may not change.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct YieldSet {
    cells: BTreeSet<u32>,
    fields: BTreeMap<(String, u64), BTreeSet<String>>,
}

impl YieldSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// A player's line changed this cell.
    pub fn touch_cell(&mut self, cell: u32) {
        self.cells.insert(cell);
    }

    /// A player's `rec` line gave `after` in place of `before`: the fields it
    /// changed are the player's.
    pub fn touch_record(&mut self, before: Option<&Record>, after: &Record) {
        let changed = changed_fields(before.map(|r| r.body.as_str()), &after.body);
        self.fields.entry(after.key()).or_default().extend(changed);
    }

    pub fn has_cell(&self, cell: u32) -> bool {
        self.cells.contains(&cell)
    }

    /// Whether the world may make `proposal` over `records`: every cell it
    /// edits is one the player never touched, and every record field it
    /// changes is one the player never changed. The first conflict refuses
    /// the whole proposal; nothing of it is to be written.
    pub fn check(&self, proposal: &Proposal, records: &Records) -> Result<(), Refusal> {
        if let Some(edit) = proposal.edits.iter().find(|e| self.cells.contains(&e.cell)) {
            return Err(Refusal::Cell(edit.cell));
        }
        for record in &proposal.records {
            let Some(touched) = self.fields.get(&record.key()) else {
                continue;
            };
            let before = records
                .get(&record.kind, record.id)
                .map(|r| r.body.as_str());
            let changed = changed_fields(before, &record.body);
            let clash = changed
                .iter()
                .find(|f| touched.contains(*f) || touched.contains("*") || f.as_str() == "*");
            if let Some(field) = clash {
                return Err(Refusal::Field {
                    kind: record.kind.clone(),
                    id: record.id,
                    field: field.clone(),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
