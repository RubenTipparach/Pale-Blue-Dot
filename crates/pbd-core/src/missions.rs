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

pub mod town;

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
    /// Water poured from a bucket (`water`).
    Pour,
    /// A fish landed, by its species' name: counted once a name in each
    /// mission ("Three kinds of fish", design "Closing the tree's gaps" G4).
    Species,
    /// A fish caught from the `sea` or from `fresh` water.
    Waters,
    /// A place reached: `marked` (a map mark), `reef`, `town`.
    Reach,
    /// Metres swum (`water`).
    Swim,
    /// A boat boarded (`boat`).
    Board,
    /// Metres travelled aboard a boat (`boat`).
    Travel,
    /// A screen opened by its key (`map`).
    Open,
    /// A fish landed on a baited line, by its species' name (G1).
    Baited,
    // ---- `townsfolk` design 8: the town jobs' kinds. A town's job names
    // its thing as `<site>:<thing>`, so it counts in that town only.
    /// A parcel handed to the named town's person: `<site>:parcel`.
    Deliver,
    /// A townsperson walked home or to a place: `<site>:villager`,
    /// `<site>:trader`.
    Escort,
    /// A broken piece of a town building mended: `<site>:piece`.
    Repair,
    /// A good sold to a town's trader: `<site>:<trade code>`.
    Sell,
    /// A good bought from a town's trader: `<site>:<trade code>`.
    Buy,
    /// A town lamp lit at dusk: `<site>:lamp`.
    Light,
    /// A road cell dug clear: `<site>:road` (roads bring it).
    Clear,
    // ---- `town-missions-tree` design 4: the town tree's kinds.
    /// A townsperson talked to: `<site>:<roster index>`, counted once a
    /// person in each mission.
    Talk,
    /// A town job handed in: the job's kind code (`fish`, `deliver`).
    Job,
    /// A town's goodwill reaching its next step: `<site>:step`.
    Goodwill,
    /// A building piece placed by the player: `wall`, `door`, `roof`, ...
    Build,
    /// A log come to rest after floating down a river: `log`.
    Float,
    /// A survey stake driven (`townsfolk` design 39): `stake`.
    Stake,
    /// A cell marked for paving: `cell`.
    Mark,
    /// A marked cell paved (`gravel`), or a road joined to a town (`joined`).
    Pave,
    /// Metres moved by a loaded cart: `cart`.
    Haul,
    /// A horse put in a cart's shafts: `horse`.
    Hitch,
}

impl Kind {
    /// The kinds a chain may name whatever the build: every kind the game
    /// raises an event for. Not `Clear`, `Stake`, `Mark` or `Pave`, which
    /// wait for the paving: a mission naming one of those is pending.
    /// `Haul` and `Hitch` are the carts' (`land-transport` design 11).
    pub const LIVE: [Kind; 40] = [
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
        Kind::Pour,
        Kind::Species,
        Kind::Waters,
        Kind::Reach,
        Kind::Swim,
        Kind::Board,
        Kind::Travel,
        Kind::Open,
        Kind::Baited,
        Kind::Deliver,
        Kind::Escort,
        Kind::Repair,
        Kind::Sell,
        Kind::Buy,
        Kind::Light,
        Kind::Talk,
        Kind::Job,
        Kind::Goodwill,
        Kind::Build,
        Kind::Float,
        Kind::Haul,
        Kind::Hitch,
    ];

    /// Whether this kind counts each name once in a mission (a fish's kind,
    /// a person talked to), however often it happens.
    pub fn once_a_name(self) -> bool {
        matches!(self, Kind::Species | Kind::Talk)
    }
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
    /// What the HUD line calls it when a mission has several objectives:
    /// `mission.<id>.objective.<index>` in the locale file. Set here only by
    /// a test's own missions; the shipped file names none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// One mission as the file writes it.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MissionEntry {
    /// Saved in the record: append-only, never renamed once shipped.
    pub id: String,
    /// Its title and help: `mission.<id>.title` and `.help` in the locale
    /// file. Set here only by a test's own missions; the shipped file names
    /// none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    #[serde(default)]
    pub needs: Vec<String>,
    pub objectives: Vec<ObjectiveEntry>,
    /// The recipe codes this mission's completion opens (design "Missions
    /// are a tree that unlocks the tech"). A recipe no mission names is open
    /// from the start.
    #[serde(default)]
    pub unlocks: Vec<String>,
    /// Missions that must also be done before it opens (design "One parent
    /// per mission"): a hidden gate, never drawn as a line.
    #[serde(default)]
    pub after: Vec<String>,
    /// The branch it is drawn in (`trunk`, `fishing`, ...).
    #[serde(default)]
    pub branch: String,
    /// Its icon on the mission tree (`starter-missions` TS.4): a path under
    /// `assets/items/` without `.png` (`tools/hand`); none for a plain card.
    #[serde(default)]
    pub icon: String,
    /// The coins its completion pays (`town-missions-tree` design 2); the
    /// starter tree pays none.
    #[serde(default, skip_serializing_if = "is_nought")]
    pub coins: u32,
}

fn is_nought(n: &u32) -> bool {
    *n == 0
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
            Group::Fish => matches!(kind, Kind::Catch | Kind::Baited),
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
    /// Any name that starts with this: a town job's `<site>:`, so anything
    /// sold to that town counts (`townsfolk` design 8). Never from the
    /// starter chain's file.
    Prefix(String),
    /// `any:<thing>` in a chain's file (`town-missions-tree` design 3):
    /// what is named `<site>:<thing>`, in any town.
    AnyTown(Box<Want>),
}

impl Want {
    pub fn matches(&self, kind: Kind, what: &str) -> bool {
        match self {
            Want::Any => true,
            Want::Group(group) => group.holds(kind, what),
            Want::Names(names) => names.iter().any(|n| n == what),
            Want::Prefix(prefix) => what.starts_with(prefix.as_str()),
            Want::AnyTown(inner) => what
                .split_once(':')
                .is_some_and(|(_, thing)| inner.matches(kind, thing)),
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
    /// Hidden gates: missions that must also be done before it opens.
    pub after: Vec<String>,
    /// The branch it is drawn in.
    pub branch: String,
    /// Its icon on the mission tree, a path under `assets/items/`.
    pub icon: String,
    /// The coins its completion pays.
    pub coins: u32,
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
            Kind::Catch | Kind::Baited | Kind::Species => self.fish.contains(name),
            Kind::Tame | Kind::Pen => self.animals.contains(name),
            Kind::Cast => name == "water",
            Kind::Fell => name == "tree",
            Kind::Night => name == "light",
            Kind::Pour | Kind::Swim => name == "water",
            Kind::Waters => matches!(name, "sea" | "fresh"),
            Kind::Reach => matches!(name, "marked" | "reef" | "town"),
            Kind::Board => name == "boat",
            // Metres aboard a boat, or driven by a cart on road cells.
            Kind::Travel => matches!(name, "boat" | "cart-road"),
            Kind::Open => name == "map",
            Kind::Eat => item(),
            // A block, or a cart set down (`land-transport` design 10).
            Kind::Place => {
                (name.starts_with("block:") && item())
                    || crate::vehicle::cart::CartKind::of_code(name).is_some()
            }
            Kind::Till | Kind::BuildPen => false,
            Kind::Deliver => name == "parcel",
            Kind::Escort => matches!(name, "villager" | "trader"),
            Kind::Repair => name == "piece",
            Kind::Light => name == "lamp",
            Kind::Clear => name == "road",
            Kind::Sell => item(),
            // A good, or a horse from a town's trader (`land-transport`).
            Kind::Buy => item() || name == "horse",
            Kind::Talk => name == "person",
            Kind::Job => matches!(
                name,
                "trade"
                    | "gather"
                    | "hunt"
                    | "deliver"
                    | "cook"
                    | "fish"
                    | "mine"
                    | "herd"
                    | "harvest"
                    | "lamps"
                    | "lost"
                    | "repair"
            ),
            Kind::Goodwill => name == "step",
            Kind::Build => matches!(
                name,
                "wall" | "door" | "window" | "floor" | "roof" | "stair" | "post" | "lantern"
            ),
            Kind::Float => name == "log",
            Kind::Stake => name == "stake",
            Kind::Mark => name == "cell",
            Kind::Pave => matches!(name, "gravel" | "joined"),
            Kind::Haul => name == "cart",
            Kind::Hitch => name == "horse",
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
    use crate::inventory::Item;
    match item {
        Item::Good(good) => good.code().to_string(),
        Item::Block(material) => format!("block:{}", crate::inventory::material_id(material)),
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
        self.resolve_keyed(catalog, "mission")
    }

    /// [`MissionFile::resolve`], with the missions' words under `keys` in
    /// the locale file (`<keys>.<id>.title`): `mission` for the starter
    /// tree, `town_mission` for the town tree. An unlock may name a recipe
    /// the file awaits: it is data only until the recipe is built.
    pub fn resolve_keyed(&self, catalog: &Catalog, keys: &str) -> Result<Chain, String> {
        let mut missions: Vec<Mission> = Vec::new();
        for entry in &self.missions {
            let at = &entry.id;
            if !crate::records::is_word(at) {
                return Err(format!("mission id {at:?} is not a word"));
            }
            if missions.iter().any(|m| &m.id == at) {
                return Err(format!("mission {at} twice"));
            }
            if entry.needs.len() > 1 {
                return Err(format!("{at}: more than one parent (use `after`)"));
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
            for (index, objective) in entry.objectives.iter().enumerate() {
                if objective.count == 0 {
                    return Err(format!("{at}: a count of nought"));
                }
                let live = catalog.live.contains(&objective.kind);
                let (anywhere, what) = match objective.what.strip_prefix("any:") {
                    Some(thing) => (true, thing),
                    None => (false, objective.what.as_str()),
                };
                let want = match what {
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
                let want = if anywhere {
                    Want::AnyTown(Box::new(want))
                } else {
                    want
                };
                if !live {
                    missing.push(format!("{:?} in the game", objective.kind).to_lowercase());
                }
                objectives.push(Objective {
                    kind: objective.kind,
                    want,
                    count: objective.count,
                    label: objective.label.clone().unwrap_or_else(|| {
                        crate::locale::tr(&format!("{keys}.{at}.objective.{index}")).into()
                    }),
                });
            }
            for code in &entry.unlocks {
                if !catalog.recipes.contains(code) && !self.awaited.iter().any(|a| a == code) {
                    return Err(format!("{at}: unlocks {code:?}, which is no recipe"));
                }
            }
            missions.push(Mission {
                id: at.clone(),
                title: entry
                    .title
                    .clone()
                    .unwrap_or_else(|| crate::locale::tr(&format!("{keys}.{at}.title")).into()),
                help: entry
                    .help
                    .clone()
                    .unwrap_or_else(|| crate::locale::tr(&format!("{keys}.{at}.help")).into()),
                needs: entry.needs.clone(),
                objectives,
                pending: (!missing.is_empty()).then_some(missing),
                unlocks: entry.unlocks.clone(),
                after: entry.after.clone(),
                branch: entry.branch.clone(),
                icon: entry.icon.clone(),
                coins: entry.coins,
            });
        }
        for m in &missions {
            for gate in &m.after {
                if gate == &m.id || !missions.iter().any(|n| &n.id == gate) {
                    return Err(format!("{}: after {gate:?}, which is no mission", m.id));
                }
            }
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

    /// The coins the missions `done` pay together: what one count's
    /// completions put in the purse, in the same line as their done marks.
    pub fn coins_for(&self, done: &[String]) -> u64 {
        done.iter()
            .filter_map(|id| self.get(id))
            .map(|m| u64::from(m.coins))
            .sum()
    }

    /// Whether any mission of this chain names `recipe` as an unlock.
    pub fn names(&self, recipe: &str) -> bool {
        self.missions
            .iter()
            .any(|m| m.unlocks.iter().any(|u| u == recipe))
    }

    /// The missions skipped in this build, with what each waits for.
    pub fn pending(&self) -> impl Iterator<Item = (&str, &[String])> {
        self.missions
            .iter()
            .filter_map(|m| m.pending.as_deref().map(|p| (m.id.as_str(), p)))
    }
}

/// Whether `recipe` may be crafted, asking every tree (`town-missions-tree`
/// design 2): open when no tree names it, when the world holds what it makes,
/// or when a tree that names it holds it open by [`Chain::recipe_open`].
pub fn recipe_open_in(
    trees: &[(&Chain, &Progress)],
    recipe: &str,
    makes: &[String],
    holds: bool,
) -> bool {
    let mut naming = trees.iter().filter(|(c, _)| c.names(recipe)).peekable();
    if holds || naming.peek().is_none() {
        return true;
    }
    naming.any(|(c, p)| c.recipe_open(p, recipe, makes, holds))
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
    /// The names an open mission's `species` objectives have counted, so a
    /// kind counts once a mission (G4). Absent from a record written before
    /// it, and dropped with the mission's counts.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub seen: BTreeMap<String, BTreeSet<String>>,
    /// The one mission the player is on (`mission-playthrough` 3.5): picked
    /// on the mission tree, carried on to the next of its branch when it is
    /// done. Absent from a record written before it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<String>,
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
            .chain(&mission.after)
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

    /// Bring a record written by an older tree up to this one's objectives
    /// (`mission-playthrough` 3.9): "Go fishing" dropped its first
    /// objective, the cast, so a record still holding two counts for it
    /// keeps the second, the fish caught.
    pub fn migrate(&mut self) {
        if let Some(counts) = self.counts.get_mut("go-fishing")
            && counts.len() == 2
        {
            counts.remove(0);
        }
    }

    /// The first open mission, in the tree's order.
    pub fn current<'a>(&'a self, chain: &'a Chain) -> Option<&'a Mission> {
        self.open(chain).next()
    }

    /// The one mission the HUD shows: the focus while it is open, else the
    /// first open one.
    pub fn focused<'a>(&'a self, chain: &'a Chain) -> Option<&'a Mission> {
        self.focus
            .as_deref()
            .and_then(|id| chain.missions.iter().find(|m| m.id == id))
            .filter(|m| self.standing(chain, m) == Standing::Open)
            .or_else(|| self.current(chain))
    }

    /// Make `id` the focus, when it is open. Whether it changed.
    pub fn focus_on(&mut self, chain: &Chain, id: &str) -> bool {
        let open = chain
            .missions
            .iter()
            .any(|m| m.id == id && self.standing(chain, m) == Standing::Open);
        if !open || self.focus.as_deref() == Some(id) {
            return false;
        }
        self.focus = Some(id.to_string());
        true
    }

    /// A mission was finished: when it was the focus, the focus goes on to
    /// the first open mission that needs it (the next of its branch), or to
    /// none, and the HUD falls back to the first open one.
    pub fn focus_past(&mut self, chain: &Chain, done: &str) {
        if self.focus.as_deref() != Some(done) {
            return;
        }
        self.focus = chain
            .missions
            .iter()
            .find(|m| {
                (m.needs.iter().any(|n| n == done) || m.after.iter().any(|n| n == done))
                    && self.standing(chain, m) == Standing::Open
            })
            .map(|m| m.id.clone());
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
            let mut named = false;
            for (count, objective) in counts.iter_mut().zip(&mission.objectives) {
                if objective.kind == happened.kind
                    && objective.want.matches(happened.kind, &happened.what)
                    && *count < objective.count
                {
                    if happened.kind.once_a_name() {
                        // A kind counts once a mission, however often landed.
                        let seen = self.seen.get(&mission.id);
                        if seen.is_some_and(|s| s.contains(&happened.what)) {
                            continue;
                        }
                        named = true;
                        *count = (*count + 1).min(objective.count);
                    } else {
                        *count = (*count + happened.count).min(objective.count);
                    }
                    changed = true;
                }
            }
            if named {
                self.seen
                    .entry(mission.id.clone())
                    .or_default()
                    .insert(happened.what.clone());
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
        let before = self.seen.len();
        self.seen.retain(|id, _| open.contains(id));
        step.changed |= self.seen.len() != before;
        step
    }

    /// The HUD's line for the current mission: `Catch fish 1/3`, or the
    /// first unmet objective's label for a mission of several. `None` once
    /// the chain is done.
    pub fn line(&self, chain: &Chain) -> Option<String> {
        self.line_of(self.focused(chain)?)
    }

    /// The HUD's words for one mission: its title and every objective with
    /// its count (`mission-playthrough` 3.5).
    pub fn line_of(&self, mission: &Mission) -> Option<String> {
        let counts = self.counts(mission);
        let first = mission.objectives.first()?;
        Some(if mission.objectives.len() == 1 {
            format!("{} {}/{}", mission.title, counts[0], first.count)
        } else {
            // Every objective and how much it needs, so nothing is hidden.
            let parts: Vec<String> = mission
                .objectives
                .iter()
                .zip(&counts)
                .map(|(o, c)| format!("{} {c}/{}", o.label, o.count))
                .collect();
            format!("{}: {}", mission.title, parts.join(", "))
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
