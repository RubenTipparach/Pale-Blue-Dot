//! Starter missions (`starter-missions`): a chain of small goals that walks a
//! new player through the first game loop, counted from what the game
//! already records.
//!
//! - **The chain** is data (`assets/config/missions.ron`): each mission has
//!   an id, a title, a line of help, the missions it needs and its
//!   objectives. An objective is a kind (`catch`), what it counts (`fish`)
//!   and how many.
//! - **Pending rows** (design 8): an objective naming something this build
//!   lacks - a good, a recipe, an action - is pending, when the file says it
//!   is awaited. A mission with a pending objective is skipped, and counts as
//!   met for the missions that need it.
//! - **Progress** is the player record `missions`: the ids done and the
//!   counts of the open missions. Counting is a pure function of the chain,
//!   the record and one [`Happened`], so a future multiplayer counts the
//!   same way.

use crate::farming::CROPS;
use crate::station::{RecipeBook, item_named};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The record kind progress is stored as, its schema and its one id (the
/// player's).
pub const MISSIONS_KIND: &str = "missions";
pub const MISSIONS_SCHEMA: u32 = 1;
pub const MISSIONS_ID: u64 = 0;

/// What an objective counts. Saved in the chain's file by its lower-case
/// name, never in a save.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// A block dug, or a good turned up while digging.
    Gather,
    /// An item made, at a station or by hand; `recipe:<code>` for one recipe.
    Craft,
    /// A block placed by the player.
    Place,
    /// The float came down on water.
    Cast,
    /// A fish landed.
    Catch,
    /// A dish collected from a kitchen station, by recipe code.
    Cook,
    /// A good found in grass or picked from a wild plant.
    Find,
    /// A cell tilled.
    Till,
    /// A seed planted, by the seed's code.
    Plant,
    /// Produce harvested from a crop.
    Harvest,
    /// A wild animal tamed.
    Tame,
    /// A kept animal let go inside a pen.
    Pen,
    /// A pen closed with a gate.
    BuildPen,
    /// An egg, wool or milk collected from a kept animal.
    Collect,
    /// An ingot made.
    Smelt,
    /// A tree felled: the axe's last log from it.
    Fell,
    /// A dish eaten from the hand, by its code.
    Eat,
    /// The clock passed 20:00 with the player near a light they placed.
    Night,
}

impl Kind {
    /// The kinds a chain may name whatever the build: every kind the game
    /// raises an event for (all of them, since felling, eating and the
    /// night came in with the Homestead chain).
    pub const LIVE: [Kind; 18] = [
        Kind::Gather,
        Kind::Craft,
        Kind::Place,
        Kind::Cast,
        Kind::Catch,
        Kind::Cook,
        Kind::Find,
        Kind::Till,
        Kind::Plant,
        Kind::Harvest,
        Kind::Tame,
        Kind::Pen,
        Kind::BuildPen,
        Kind::Collect,
        Kind::Smelt,
        Kind::Fell,
        Kind::Eat,
        Kind::Night,
    ];
}

/// Something the player did, as the missions system counts it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Happened {
    pub kind: Kind,
    /// The canonical name of the thing: a good's code, `block:<name>`,
    /// `tool:<name>`, `recipe:<code>`, a roster name in lower case, `water`
    /// for a cast, `tree` for a felling.
    pub what: String,
    pub count: u32,
}

impl Happened {
    pub fn new(kind: Kind, what: impl Into<String>, count: u32) -> Self {
        Self {
            kind,
            what: what.into(),
            count,
        }
    }
}

/// One objective as the file writes it.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectiveEntry {
    pub kind: Kind,
    /// A name, names joined by `|`, `any`, or a group (`seed`, `crop`,
    /// `ingot`, `fish`).
    pub what: String,
    pub count: u32,
    /// What the HUD line calls it when a mission has several objectives.
    pub label: String,
}

/// One mission as the file writes it.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MissionEntry {
    /// Saved in the record: append-only, never renamed once shipped.
    pub id: String,
    pub title: String,
    pub help: String,
    #[serde(default)]
    pub needs: Vec<String>,
    pub objectives: Vec<ObjectiveEntry>,
    /// The recipe codes this mission's completion opens (design "Missions
    /// are a tree that unlocks the tech"). A recipe no mission names is open
    /// from the start.
    #[serde(default)]
    pub unlocks: Vec<String>,
}

/// `assets/config/missions.ron`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct MissionFile {
    /// Names another branch will bring (`tool-crafting`'s goods and
    /// recipes, `tree` for felling): an objective naming only these and
    /// missing them is pending rather than an error.
    #[serde(default)]
    pub awaited: Vec<String>,
    pub missions: Vec<MissionEntry>,
}

/// A group of names an objective can count as one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    /// Any crop's seed.
    Seed,
    /// Any crop's produce.
    Crop,
    /// A good whose code ends `-ingot`.
    Ingot,
    /// Any fish.
    Fish,
}

impl Group {
    fn named(name: &str) -> Option<Self> {
        Some(match name {
            "seed" => Group::Seed,
            "crop" => Group::Crop,
            "ingot" => Group::Ingot,
            "fish" => Group::Fish,
            _ => return None,
        })
    }

    fn holds(self, kind: Kind, what: &str) -> bool {
        match self {
            Group::Seed => CROPS.iter().any(|c| c.seed.code() == what),
            Group::Crop => CROPS.iter().any(|c| c.produce.code() == what),
            Group::Ingot => what.ends_with("-ingot"),
            Group::Fish => kind == Kind::Catch,
        }
    }
}

/// What an objective counts, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Want {
    Any,
    Group(Group),
    /// Any of these names (the ones this build knows).
    Names(Vec<String>),
}

impl Want {
    pub fn matches(&self, kind: Kind, what: &str) -> bool {
        match self {
            Want::Any => true,
            Want::Group(group) => group.holds(kind, what),
            Want::Names(names) => names.iter().any(|n| n == what),
        }
    }
}

/// One objective, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Objective {
    pub kind: Kind,
    pub want: Want,
    pub count: u32,
    pub label: String,
}

/// One mission, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mission {
    pub id: String,
    pub title: String,
    pub help: String,
    pub needs: Vec<String>,
    pub objectives: Vec<Objective>,
    /// Why it waits, when it does: the names this build lacks.
    pub pending: Option<Vec<String>>,
    /// The recipe codes its completion opens.
    pub unlocks: Vec<String>,
}

/// What the build knows, for resolving names: built by the app from its
/// tables (the recipe book, the rosters) and by tests from a few rows.
#[derive(Clone, Debug, Default)]
pub struct Catalog {
    /// Every recipe code.
    pub recipes: BTreeSet<String>,
    /// The canonical name of every item some recipe makes.
    pub made: BTreeSet<String>,
    /// Fish species, lower case.
    pub fish: BTreeSet<String>,
    /// Land animals, lower case.
    pub animals: BTreeSet<String>,
    /// The kinds the game raises events for.
    pub live: BTreeSet<Kind>,
}

impl Catalog {
    /// A catalog from the recipe book and the rosters' names, with every
    /// kind in [`Kind::LIVE`] live.
    pub fn new<'a>(
        book: &RecipeBook,
        fish: impl IntoIterator<Item = &'a str>,
        animals: impl IntoIterator<Item = &'a str>,
    ) -> Self {
        let mut catalog = Catalog {
            live: Kind::LIVE.into_iter().collect(),
            fish: fish.into_iter().map(str::to_lowercase).collect(),
            animals: animals.into_iter().map(str::to_lowercase).collect(),
            ..Default::default()
        };
        for recipe in &book.recipes {
            catalog.recipes.insert(recipe.code.clone());
            for (item, _) in &recipe.outputs {
                catalog.made.insert(item_code(*item));
            }
            for (tool, _) in &recipe.tools {
                catalog
                    .made
                    .insert(item_code(crate::inventory::Item::Tool(*tool)));
            }
        }
        catalog
    }

    /// Whether this build knows `name` for an objective of `kind`: `Ok`
    /// when it does, `Err(true)` when it knows the name but nothing makes it
    /// (a craft no recipe makes: pending), `Err(false)` when the name is
    /// unknown.
    fn knows(&self, kind: Kind, name: &str) -> Result<(), bool> {
        let item = || item_named(name).is_some();
        let known = match kind {
            Kind::Craft => {
                if let Some(code) = name.strip_prefix("recipe:") {
                    self.recipes.contains(code)
                } else if self.made.contains(name) {
                    true
                } else {
                    return Err(item());
                }
            }
            Kind::Cook => self.recipes.contains(name),
            Kind::Catch => self.fish.contains(name),
            Kind::Tame | Kind::Pen => self.animals.contains(name),
            Kind::Cast => name == "water",
            Kind::Fell => name == "tree",
            Kind::Night => name == "light",
            Kind::Eat => item(),
            Kind::Place => name.starts_with("block:") && item(),
            Kind::Till | Kind::BuildPen => false,
            Kind::Gather
            | Kind::Find
            | Kind::Plant
            | Kind::Harvest
            | Kind::Collect
            | Kind::Smelt => item(),
        };
        if known { Ok(()) } else { Err(false) }
    }
}

/// An item's canonical name: the name `recipes.ron` gives it, so a chain
/// and a recipe table name things alike. A fish is `fish`.
pub fn item_code(item: crate::inventory::Item) -> String {
    use crate::inventory::{Item, material_name};
    match item {
        Item::Good(good) => good.code().to_string(),
        Item::Block(material) => format!("block:{}", material_name(material)),
        Item::Tool(tool) => format!("tool:{}", tool.code()),
        Item::Fish(_) => "fish".into(),
    }
}

/// The chain, resolved.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Chain {
    pub missions: Vec<Mission>,
}

impl MissionFile {
    /// Resolve every mission against what the build knows: a name it lacks
    /// that the file awaits makes its mission pending; any other is an error,
    /// by mission and name.
    pub fn resolve(&self, catalog: &Catalog) -> Result<Chain, String> {
        let mut missions: Vec<Mission> = Vec::new();
        for entry in &self.missions {
            let at = &entry.id;
            if !crate::records::is_word(at) {
                return Err(format!("mission id {at:?} is not a word"));
            }
            if missions.iter().any(|m| &m.id == at) {
                return Err(format!("mission {at} twice"));
            }
            for need in &entry.needs {
                if !missions.iter().any(|m| &m.id == need) {
                    return Err(format!("{at}: needs {need:?}, which is not before it"));
                }
            }
            if entry.objectives.is_empty() {
                return Err(format!("{at}: no objectives"));
            }
            let mut missing = Vec::new();
            let mut objectives = Vec::new();
            for objective in &entry.objectives {
                if objective.count == 0 {
                    return Err(format!("{at}: a count of nought"));
                }
                let live = catalog.live.contains(&objective.kind);
                let want = match objective.what.as_str() {
                    "any" => Want::Any,
                    name if Group::named(name).is_some() => {
                        Want::Group(Group::named(name).expect("named"))
                    }
                    list => {
                        let mut known = Vec::new();
                        let mut lacking = Vec::new();
                        for name in list.split('|').map(str::trim) {
                            match catalog.knows(objective.kind, name) {
                                Ok(()) => known.push(name.to_string()),
                                // A craft of something no recipe makes.
                                Err(true) => lacking.push(format!("a recipe for {name}")),
                                Err(false) if self.awaited.iter().any(|a| a == name) => {
                                    lacking.push(name.to_string())
                                }
                                Err(false) => {
                                    return Err(format!(
                                        "{at}: {:?} names no {:?} this build knows: {name:?}",
                                        objective.kind, objective.what
                                    ));
                                }
                            }
                        }
                        if known.is_empty() {
                            missing.extend(lacking);
                        }
                        Want::Names(known)
                    }
                };
                if !live {
                    missing.push(format!("{:?} in the game", objective.kind).to_lowercase());
                }
                objectives.push(Objective {
                    kind: objective.kind,
                    want,
                    count: objective.count,
                    label: objective.label.clone(),
                });
            }
            for code in &entry.unlocks {
                if !catalog.recipes.contains(code) {
                    return Err(format!("{at}: unlocks {code:?}, which is no recipe"));
                }
            }
            missions.push(Mission {
                id: at.clone(),
                title: entry.title.clone(),
                help: entry.help.clone(),
                needs: entry.needs.clone(),
                objectives,
                pending: (!missing.is_empty()).then_some(missing),
                unlocks: entry.unlocks.clone(),
            });
        }
        Ok(Chain { missions })
    }
}

impl Chain {
    pub fn get(&self, id: &str) -> Option<&Mission> {
        self.missions.iter().find(|m| m.id == id)
    }

    /// The mission whose completion opens `recipe`, if one names it: the
    /// first in the chain's order.
    pub fn unlocked_by(&self, recipe: &str) -> Option<&Mission> {
        self.missions
            .iter()
            .find(|m| m.unlocks.iter().any(|u| u == recipe))
    }

    /// Whether `recipe` may be crafted (the one lock rule, used by every
    /// station). Open when no mission names it, or when a mission that
    /// names it is done or pending (a pending mission is met for those after
    /// it). And, so no world from before the tree loses a tool it could make
    /// (T.3): open when the world already `holds` what it makes - a tool
    /// owned, an item carried, a block placed - or when a done mission
    /// crafted it (`makes` are its outputs' canonical names).
    pub fn recipe_open(
        &self,
        progress: &Progress,
        recipe: &str,
        makes: &[String],
        holds: bool,
    ) -> bool {
        let mut naming = self
            .missions
            .iter()
            .filter(|m| m.unlocks.iter().any(|u| u == recipe))
            .peekable();
        if naming.peek().is_none() || holds {
            return true;
        }
        if naming.any(|m| progress.is_done(&m.id) || m.pending.is_some()) {
            return true;
        }
        let code = format!("recipe:{recipe}");
        self.missions
            .iter()
            .filter(|m| progress.is_done(&m.id))
            .flat_map(|m| &m.objectives)
            .filter(|o| o.kind == Kind::Craft)
            .any(|o| {
                o.want.matches(Kind::Craft, &code)
                    || makes.iter().any(|w| o.want.matches(Kind::Craft, w))
            })
    }

    /// The missions skipped in this build, with what each waits for.
    pub fn pending(&self) -> impl Iterator<Item = (&str, &[String])> {
        self.missions
            .iter()
            .filter_map(|m| m.pending.as_deref().map(|p| (m.id.as_str(), p)))
    }
}

/// Where a mission stands, for the tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    Done,
    /// Open and counting.
    Open,
    /// Waiting on a mission before it.
    Locked,
    /// Skipped: this build lacks something it names.
    Pending,
}

/// The `missions` record's body: what is done, and the counts of the open
/// missions. Ids this build's chain lacks are kept as they are.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Progress {
    pub done: BTreeSet<String>,
    /// Each open mission's count per objective, in the chain's order.
    pub counts: BTreeMap<String, Vec<u32>>,
}

impl Progress {
    pub fn is_done(&self, id: &str) -> bool {
        self.done.contains(id)
    }

    /// Where `mission` stands.
    pub fn standing(&self, chain: &Chain, mission: &Mission) -> Standing {
        if self.is_done(&mission.id) {
            Standing::Done
        } else if mission.pending.is_some() {
            Standing::Pending
        } else if mission
            .needs
            .iter()
            .all(|need| self.is_done(need) || chain.get(need).is_none_or(|m| m.pending.is_some()))
        {
            Standing::Open
        } else {
            Standing::Locked
        }
    }

    /// The open missions, in the chain's order.
    pub fn open<'a>(&'a self, chain: &'a Chain) -> impl Iterator<Item = &'a Mission> + 'a {
        chain
            .missions
            .iter()
            .filter(move |m| self.standing(chain, m) == Standing::Open)
    }

    /// The mission the HUD shows: the first open one.
    pub fn current<'a>(&'a self, chain: &'a Chain) -> Option<&'a Mission> {
        self.open(chain).next()
    }

    /// The count so far of each of `mission`'s objectives.
    pub fn counts(&self, mission: &Mission) -> Vec<u32> {
        let mut counts = self.counts.get(&mission.id).cloned().unwrap_or_default();
        counts.resize(mission.objectives.len(), 0);
        if self.is_done(&mission.id) {
            for (count, objective) in counts.iter_mut().zip(&mission.objectives) {
                *count = objective.count;
            }
        }
        counts
    }

    /// Count one thing done, then close what it finished and open what that
    /// lets open, with their state-met objectives (`state` says how many of
    /// an objective the world already holds; nought for one that is not
    /// state-shaped). The ids finished, in order; `changed` says whether the
    /// record moved at all.
    pub fn record(
        &mut self,
        chain: &Chain,
        happened: &Happened,
        state: &dyn Fn(&Objective) -> u32,
    ) -> Step {
        let mut changed = false;
        let open: Vec<Mission> = self.open(chain).cloned().collect();
        for mission in &open {
            let mut counts = self.counts(mission);
            for (count, objective) in counts.iter_mut().zip(&mission.objectives) {
                if objective.kind == happened.kind
                    && objective.want.matches(happened.kind, &happened.what)
                    && *count < objective.count
                {
                    *count = (*count + happened.count).min(objective.count);
                    changed = true;
                }
            }
            if counts.iter().any(|c| *c > 0) {
                self.counts.insert(mission.id.clone(), counts);
            }
        }
        let mut step = self.settle(chain, state);
        step.changed |= changed;
        step
    }

    /// Close every open mission whose objectives are met, counting the
    /// state-met ones, until nothing more closes: what a world does when it
    /// opens (an old world joins the chain this way) and after every count.
    pub fn settle(&mut self, chain: &Chain, state: &dyn Fn(&Objective) -> u32) -> Step {
        let mut step = Step::default();
        loop {
            let mut closed = None;
            let open: Vec<Mission> = self.open(chain).cloned().collect();
            for mission in &open {
                let mut counts = self.counts(mission);
                let had = counts.clone();
                for (count, objective) in counts.iter_mut().zip(&mission.objectives) {
                    let held = state(objective).min(objective.count);
                    *count = (*count).max(held);
                }
                let met = counts
                    .iter()
                    .zip(&mission.objectives)
                    .all(|(count, objective)| *count >= objective.count);
                if met {
                    closed = Some(mission.id.clone());
                    break;
                }
                if counts != had {
                    step.changed = true;
                    self.counts.insert(mission.id.clone(), counts);
                }
            }
            // A locked mission the world already holds whole closes too: a
            // world that owns its tools is not told to make them, whatever
            // came before them in the chain.
            if closed.is_none() {
                closed = chain
                    .missions
                    .iter()
                    .filter(|m| self.standing(chain, m) == Standing::Locked)
                    .find(|m| m.objectives.iter().all(|o| state(o) >= o.count))
                    .map(|m| m.id.clone());
            }
            let Some(id) = closed else {
                break;
            };
            // A mission done means the ones it needs were passed: they are
            // done with it, in the chain's order.
            let mut close = vec![id];
            let mut at = 0;
            while at < close.len() {
                if let Some(m) = chain.get(&close[at]) {
                    for need in &m.needs {
                        if !self.is_done(need)
                            && !close.contains(need)
                            && chain.get(need).is_some_and(|n| n.pending.is_none())
                        {
                            close.push(need.clone());
                        }
                    }
                }
                at += 1;
            }
            close.sort_by_key(|id| chain.missions.iter().position(|m| &m.id == id));
            for id in close {
                self.counts.remove(&id);
                self.done.insert(id.clone());
                step.done.push(id);
            }
            step.changed = true;
        }
        // Counts kept only for open missions: a count of nought is no record.
        let before = self.counts.len();
        let open: BTreeSet<String> = self.open(chain).map(|m| m.id.clone()).collect();
        self.counts
            .retain(|id, counts| open.contains(id) && counts.iter().any(|c| *c > 0));
        step.changed |= self.counts.len() != before;
        step
    }

    /// The HUD's line for the current mission: `Catch fish 1/3`, or the
    /// first unmet objective's label for a mission of several. `None` once
    /// the chain is done.
    pub fn line(&self, chain: &Chain) -> Option<String> {
        let mission = self.current(chain)?;
        let counts = self.counts(mission);
        let (objective, count) = mission
            .objectives
            .iter()
            .zip(&counts)
            .find(|(o, c)| **c < o.count)
            .or_else(|| mission.objectives.iter().zip(&counts).next())?;
        Some(if mission.objectives.len() == 1 {
            format!("{} {count}/{}", mission.title, objective.count)
        } else {
            format!(
                "{}: {} {count}/{}",
                mission.title, objective.label, objective.count
            )
        })
    }
}

/// What one count or settle did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Step {
    /// The missions finished, in order.
    pub done: Vec<String>,
    /// Whether the record changed and is to be stored.
    pub changed: bool,
}

#[cfg(test)]
mod tests;
