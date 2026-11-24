//! What a save looks like on disk.
//!
//! Two files per slot and two shapes, because the two halves change on
//! different clocks. `edits.v1.log` is append-only and carries a line per accepted
//! edit; `world.ron` is replaced whole and carries what a timer can afford to
//! write.
//!
//! **The log line carries the HOTBAR as well as the edit**, and that is the
//! decision worth reading twice. The obvious alternative is a periodic
//! snapshot of pose and inventory together, and it is wrong in a way that only
//! shows after a crash: an edit is durable the instant it happens and a
//! five-second snapshot is not, so the hole would be dug and the block it
//! yielded never picked up. What CHANGES a stack is a dig or a place, so the
//! record of one carries both and a replay reconstructs the world and the
//! player from the same line.
//!
//! Lines rather than a serialised structure, for the reason the log is a log:
//! a torn write costs the last line, where a `ron` file with its last byte
//! missing is a file with nothing in it.
//!
//! **The slots are forty fields**, the hotbar's ten and the pack's thirty
//! (`inventory-grid` decision 3), and a line from before the pack carries ten,
//! which read as an empty pack. **A dig's line carries the drop it made**
//! (decision 4): the dug block floats in the world as the player's property,
//! so the line that records the hole records where the block went, and a
//! pickup is a line of its own.

use pbd_core::building::site::BuildingSetting;
use pbd_core::drops::ItemDrop;
use pbd_core::edits::Edit;
use pbd_core::goods::Good;
use pbd_core::inventory::{CARRIED, Equipment, Item, SLOTS, Slots, Stack, Tier, Tool};
use pbd_core::locale::tr_with;
use pbd_core::planet_gen::TerrainConfig;
use pbd_core::records::{Author, Record as StoredRecord, split_author};
use pbd_core::terrain::{GENERATOR_VERSION, TOPOLOGY_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One line of the log: an edit, and what the player held once it was made,
/// or a kit dealt, and what the player held once it was.
#[derive(Clone, Debug, PartialEq)]
pub enum Record {
    Edit {
        edit: Edit,
        /// The drop the edit made: the dug block, floating where it was cut.
        /// Absent on a place, and on every line from before drops.
        drop: Option<ItemDrop>,
        /// Absent on a line written before the hotbar rode the log. Such a
        /// line is still a real edit, so it is kept: an older save loses the
        /// inventory it never recorded and none of the world it did.
        slots: Option<Slots>,
    },
    /// The starting kit's grants up to `version` were dealt into the hotbar,
    /// which this line carries whole. A save opened behind the kit's version
    /// is dealt what it missed, and this is what says it was, so it is never
    /// dealt twice.
    Kit { version: u32, slots: Slots },
    /// A fish was caught and went into the hotbar, which this line carries
    /// whole: the fish, and the field guide's record of it, reach the disk in
    /// one line.
    Catch {
        species: u16,
        length_cm: u32,
        slots: Slots,
    },
    /// The tool in hand changed, and which tools are owned.
    Hand { equipment: Equipment },
    /// Stacks were moved about the hotbar and the pack, which this line
    /// carries whole.
    Pack { slots: Slots },
    /// Some of drop `id` was picked up, leaving `left` on it (none: it is
    /// gone), into the slots this line carries whole.
    Pick { id: u64, left: u16, slots: Slots },
    /// A stack the player dropped (`drop-and-trash`): the drop, and the
    /// slots after it.
    Toss { drop: ItemDrop, slots: Slots },
    /// A record's new value, whole (`world-persistence` decision 11): a
    /// site, and later a settlement or a landmark. Held as text, so a kind
    /// this build does not know is kept as it came.
    Stored(StoredRecord),
    /// A record's new value and the slots after it, in ONE line
    /// (`farming-and-smelting` decision 4): a crop planted or harvested, a
    /// station's job queued or collected. Both reach the disk or neither.
    Handed { record: StoredRecord, slots: Slots },
}

/// The kit line's leading token, which no cell number can be.
const KIT: &str = "kit";
/// A catch's.
const CATCH: &str = "catch";
/// A change of tool's.
const HAND: &str = "hand";
/// A move in the pack's.
const PACK: &str = "pack";
/// A pickup's.
const PICK: &str = "pick";
/// A stack the player dropped's (`drop-and-trash`).
const TOSS: &str = "toss";
/// A record with the hands': `with s0 .. s39 rec @p kind id schema body`.
const WITH: &str = "with";

/// `with s0 .. s39 <record line>`, one line: the slots after the change,
/// then the record as a `rec` line of the player's.
pub fn with_line_of(record: &StoredRecord, slots: &Slots) -> String {
    let mut line = WITH.to_string();
    for index in 0..CARRIED {
        line.push(' ');
        line.push_str(&stack_text(slots.get(index)));
    }
    line.push(' ');
    line.push_str(&record.line(&Author::Player(0)));
    line
}

/// A tool's saved code. `t0` was the pick placeholder, which nothing ever
/// constructed, so the pickaxe keeps it.
pub fn tool_code(tool: Tool) -> u8 {
    match tool {
        Tool::Pickaxe => 0,
        Tool::Shovel => 1,
        Tool::Axe => 2,
        Tool::Rod => 3,
    }
}

pub fn tool_of(code: u8) -> Option<Tool> {
    Some(match code {
        0 => Tool::Pickaxe,
        1 => Tool::Shovel,
        2 => Tool::Axe,
        3 => Tool::Rod,
        _ => return None,
    })
}

/// The save's material codes live in the core, where the server and the
/// region digest read them too (`multiplayer` 3.1).
pub use pbd_core::terrain::{material_code, material_of};

pub fn stack_text(stack: Option<Stack>) -> String {
    match stack {
        None => "-".into(),
        Some(Stack {
            item: Item::Block(material),
            count,
        }) => format!("b{},{count}", material_code(material)),
        Some(Stack {
            item: Item::Tool(tool),
            count,
        }) => format!("t{},{count}", tool_code(tool)),
        Some(Stack {
            item: Item::Fish(species),
            count,
        }) => format!("f{species},{count}"),
        // A good by its CODE, never its index (`farming-and-smelting`
        // decision 1): `gwheat,3`.
        Some(Stack {
            item: Item::Good(good),
            count,
        }) => format!("g{},{count}", good.code()),
    }
}

pub fn stack_of(text: &str) -> Option<Option<Stack>> {
    if text == "-" {
        return Some(None);
    }
    let (kind, rest) = text.split_at_checked(1)?;
    let (code, count) = rest.split_once(',')?;
    let count: u16 = count.parse().ok()?;
    let item = match kind {
        "b" => Item::Block(material_of(code.parse().ok()?)?),
        "t" => Item::Tool(tool_of(code.parse().ok()?)?),
        "f" => Item::Fish(code.parse().ok()?),
        "g" => Item::Good(Good::by_code(code)?),
        _ => return None,
    };
    Some(Some(Stack::new(item, count)))
}

/// `cell layer material [drop] s0 s1 .. s39`, one line.
pub fn line_of(edit: Edit, drop: Option<&ItemDrop>, slots: &Slots) -> String {
    let mut line = format!(
        "{} {} {}",
        edit.cell,
        edit.layer,
        material_code(edit.material)
    );
    if let Some(drop) = drop {
        line.push(' ');
        line.push_str(&drop_text(drop));
    }
    push_slots(&mut line, slots);
    line
}

/// A drop as one field: `d<id>:<stack>:<made_s>:<x>,<y>,<z>`. It begins with
/// `d`, which no slot field does, so an edit line says whether it has one. A
/// millimetre and a millisecond are finer than anything a drop does. The
/// point is the body's frame in f64 (`world-frames` 10.4); `{:.3}` writes
/// the same text for it as it did for the f32 it was before.
fn drop_text(drop: &ItemDrop) -> String {
    let p = drop.position;
    format!(
        "d{}:{}:{:.3}:{:.3},{:.3},{:.3}",
        drop.id,
        stack_text(Some(Stack::new(drop.item, drop.count))),
        drop.made_s,
        p.x,
        p.y,
        p.z
    )
}

fn drop_of(text: &str) -> Option<ItemDrop> {
    let mut fields = text.strip_prefix('d')?.split(':');
    let id = fields.next()?.parse().ok()?;
    let stack = stack_of(fields.next()?)??;
    let made_s: f64 = fields.next()?.parse().ok()?;
    let mut xyz = fields.next()?.split(',').map(|v| v.parse::<f64>().ok());
    let position = bevy::math::DVec3::new(xyz.next()??, xyz.next()??, xyz.next()??);
    if fields.next().is_some()
        || xyz.next().is_some()
        || stack.count == 0
        || !made_s.is_finite()
        || !position.is_finite()
    {
        return None;
    }
    Some(ItemDrop {
        id,
        item: stack.item,
        count: stack.count,
        position,
        made_s,
    })
}

/// `pack s0 s1 .. s39`, one line.
pub fn pack_line_of(slots: &Slots) -> String {
    let mut line = PACK.to_string();
    push_slots(&mut line, slots);
    line
}

/// `toss d<drop> s0 s1 .. s39`, one line: a stack the player dropped and
/// the slots after it (`drop-and-trash` design 1).
pub fn toss_line_of(drop: &ItemDrop, slots: &Slots) -> String {
    let mut line = format!("{TOSS} {}", drop_text(drop));
    push_slots(&mut line, slots);
    line
}

/// `pick id left s0 s1 .. s39`, one line.
pub fn pick_line_of(id: u64, left: u16, slots: &Slots) -> String {
    let mut line = format!("{PICK} {id} {left}");
    push_slots(&mut line, slots);
    line
}

/// `kit version s0 s1 .. s39`, one line.
pub fn kit_line_of(version: u32, slots: &Slots) -> String {
    let mut line = format!("{KIT} {version}");
    push_slots(&mut line, slots);
    line
}

fn push_slots(line: &mut String, slots: &Slots) {
    for index in 0..CARRIED {
        line.push(' ');
        line.push_str(&stack_text(slots.get(index)));
    }
    line.push('\n');
}

/// `catch species length_cm s0 s1 .. s39`, one line.
pub fn catch_line_of(species: u16, length_cm: u32, slots: &Slots) -> String {
    let mut line = format!("{CATCH} {species} {length_cm}");
    push_slots(&mut line, slots);
    line
}

/// `hand tool tiers`, one line: the tool's code, or `-` for bare hands
/// (`inventory-grid` decision 8), and one character per tool in `Tool::ALL`
/// order, its tier's digit or `-` for not owned (`tool-crafting` decision
/// 8): `hand - 0-2-`. The field before tiers was a bit mask of one or two
/// digits, which [`parse_line`] still reads, as the stone tier.
pub fn hand_line_of(equipment: &Equipment) -> String {
    let tiers: String = equipment
        .tiers()
        .iter()
        .map(|tier| tier.map_or('-', |t| char::from(b'0' + t.digit())))
        .collect();
    let held = equipment
        .held()
        .map_or_else(|| BARE.to_string(), |t| tool_code(t).to_string());
    format!("{HAND} {held} {tiers}\n")
}

/// The owned field of a `hand` line: four tier characters, or the old bit
/// mask read as the stone tier.
fn tiers_of(field: &str) -> Option<[Option<Tier>; 4]> {
    if field.len() == Tool::ALL.len() && field.chars().all(|c| c == '-' || c.is_ascii_digit()) {
        let mut tiers = [None; 4];
        for (slot, c) in tiers.iter_mut().zip(field.bytes()) {
            *slot = match c {
                b'-' => None,
                digit => Some(Tier::of_digit(digit - b'0')?),
            };
        }
        return Some(tiers);
    }
    if field.len() > 2 {
        return None;
    }
    let bits: u8 = field.parse().ok()?;
    if bits >= 1 << Tool::ALL.len() {
        return None;
    }
    let mut tiers = [None; 4];
    for (i, tier) in tiers.iter_mut().enumerate() {
        *tier = (bits & (1 << i) != 0).then_some(Tier::Stone);
    }
    Some(tiers)
}

/// The `hand` line's tool for bare hands. An older build reads it as a
/// damaged line and skips it, keeping the tool it last read.
const BARE: &str = "-";

/// The slot fields of a line: forty, or ten from a line written before the
/// pack, which is an empty pack. `None` where there are neither.
fn slots_of(carried: &[&str]) -> Option<Slots> {
    if carried.len() != SLOTS && carried.len() != CARRIED {
        return None;
    }
    let mut slots = Slots::new();
    for (index, text) in carried.iter().enumerate() {
        slots.set(index, stack_of(text)?);
    }
    Some(slots)
}

/// A journal line and its author (`world-persistence` decision 11): a
/// `rec` line names its own; any other line may lead with a token, and one
/// with none is the player's, as every line before authors was. `None` for a
/// damaged line.
pub fn parse_entry(line: &str) -> Option<(Author, Record)> {
    if line.starts_with(pbd_core::records::REC) {
        let (author, record) = StoredRecord::parse_line(line)?;
        return Some((author, Record::Stored(record)));
    }
    let (author, rest) = split_author(line)?;
    Some((author, parse_line(rest)?))
}

/// A line as `author` writes it: the player's with no token, as it always
/// was, so an older build still reads what the player did; anyone else's
/// with theirs in front.
pub fn authored(author: &Author, line: String) -> String {
    match author {
        Author::Player(0) => line,
        other => format!("{} {line}", other.token()),
    }
}

/// An edit a world process or the creation made: `cell layer material`,
/// with no slots, since nobody's hands were in it.
pub fn world_edit_line(edit: Edit) -> String {
    format!(
        "{} {} {}\n",
        edit.cell,
        edit.layer,
        material_code(edit.material)
    )
}

/// One line with no author token, which [`parse_entry`] has taken off.
pub fn parse_line(line: &str) -> Option<Record> {
    let mut parts = line.split_whitespace();
    let head = parts.next()?;
    if head == CATCH {
        let species = parts.next()?.parse().ok()?;
        let length_cm = parts.next()?.parse().ok()?;
        let carried: Vec<&str> = parts.collect();
        return Some(Record::Catch {
            species,
            length_cm,
            slots: slots_of(&carried)?,
        });
    }
    if head == WITH {
        // Forty slot fields, then a whole record line, whose body may hold
        // spaces: split the slots off by count, and parse the rest as a line.
        let rest = line.trim_start().strip_prefix(WITH)?;
        let mut rest = rest.trim_start();
        let mut carried = Vec::with_capacity(CARRIED);
        for _ in 0..CARRIED {
            let (field, after) = rest.split_once(' ')?;
            carried.push(field);
            rest = after.trim_start();
        }
        let (author, record) = StoredRecord::parse_line(rest)?;
        if !author.is_player() {
            return None;
        }
        return Some(Record::Handed {
            record,
            slots: slots_of(&carried)?,
        });
    }
    if head == HAND {
        let held = match parts.next()? {
            BARE => None,
            code => Some(tool_of(code.parse().ok()?)?),
        };
        let tiers = tiers_of(parts.next()?)?;
        if parts.next().is_some() {
            return None;
        }
        return Some(Record::Hand {
            equipment: Equipment::from_tiers(tiers, held),
        });
    }
    if head == PACK {
        let carried: Vec<&str> = parts.collect();
        return Some(Record::Pack {
            slots: slots_of(&carried)?,
        });
    }
    if head == TOSS {
        let drop = drop_of(parts.next()?)?;
        let carried: Vec<&str> = parts.collect();
        return Some(Record::Toss {
            drop,
            slots: slots_of(&carried)?,
        });
    }
    if head == PICK {
        let id = parts.next()?.parse().ok()?;
        let left = parts.next()?.parse().ok()?;
        let carried: Vec<&str> = parts.collect();
        return Some(Record::Pick {
            id,
            left,
            slots: slots_of(&carried)?,
        });
    }
    if head == KIT {
        let version = parts.next()?.parse().ok()?;
        let carried: Vec<&str> = parts.collect();
        return Some(Record::Kit {
            version,
            slots: slots_of(&carried)?,
        });
    }
    let cell = head.parse().ok()?;
    let layer = parts.next()?.parse().ok()?;
    let material = material_of(parts.next()?.parse().ok()?)?;
    let mut carried: Vec<&str> = parts.collect();
    let drop = match carried.first() {
        Some(field) if field.starts_with('d') => {
            let drop = drop_of(field)?;
            carried.remove(0);
            Some(drop)
        }
        _ => None,
    };
    // A line from before the hotbar rode the log has three fields, and it is a
    // real edit. Anything else without its slots is damaged and its inventory
    // is not guessed at.
    let slots = match carried.len() {
        // Three fields: a line from before the hotbar rode the log. A real
        // edit, and it claims no inventory.
        0 if drop.is_none() => None,
        // Anything else is a line that was being written when the power went
        // out, or a drop with no slots after it. Its edit looks intact and its
        // hotbar is half there, and taking the edit alone would record the
        // hole without the block that came out of it - which is the exact
        // inconsistency the hotbar rides this line to prevent.
        _ => Some(slots_of(&carried)?),
    };
    Some(Record::Edit {
        edit: Edit {
            cell,
            layer,
            material,
        },
        drop,
        slots,
    })
}

/// How a log keys its cells: 1 is the exact key (`pbd_core::cell_key`,
/// `edits.v1.log`); 0 was the old hash (`edits.log`), which a slot is migrated
/// off when it is opened.
pub const KEY_VERSION: u32 = 1;

/// A slot's `identity.ron`: what the world IS, the seed and every version
/// that shapes it (`world-persistence` decision 3).
///
/// Its own file, written once when the world is made and again only when the
/// world is deliberately upgraded, as the key migration does. It is not in
/// `world.ron`, because that file is rewritten whole every five seconds, and
/// a build that did not know these fields would write it back without them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Identity {
    pub seed: u64,
    /// The terrain generator the world is made by (`TerrainConfig::for_version`).
    pub generator: u32,
    /// How its cells are cut ([`TOPOLOGY_VERSION`]).
    pub topology: u32,
    /// How its log keys the cells ([`KEY_VERSION`]).
    pub keys: u32,
    /// Each stored record kind's schema version, by kind. None exists yet:
    /// `city-sites` adds the first.
    #[serde(default)]
    pub records: BTreeMap<String, u32>,
    /// How the world's tools began (`tool-crafting` decision 8): 0, absent
    /// in every world made before, is the old kit of all four tools; 1 is a
    /// world made with tool crafting, which starts owning none.
    #[serde(default)]
    pub tools: u32,
    /// How the world builds (`building-planner` design 11): absent in every
    /// world made before, which reads as Free, the way those worlds were
    /// built; a world this build makes is Planned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub building: Option<BuildingSetting>,
}

/// The `tools` a world this build makes is marked with.
pub const TOOLS_VERSION: u32 = 1;

impl Identity {
    /// A world this build makes.
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            generator: GENERATOR_VERSION,
            topology: TOPOLOGY_VERSION,
            keys: KEY_VERSION,
            records: BTreeMap::new(),
            tools: TOOLS_VERSION,
            building: Some(BuildingSetting::NEW_WORLD),
        }
    }

    /// How the world builds: missing reads Free (`building-planner` 11).
    pub fn building(&self) -> BuildingSetting {
        BuildingSetting::of_saved(self.building)
    }

    /// Whether the world began with tool crafting, owning no tools.
    pub fn crafts_its_tools(&self) -> bool {
        self.tools >= TOOLS_VERSION
    }

    /// A world made before identities were written: generator 4 and topology
    /// 1, the only versions a saved world could have been made under, and
    /// the keys its log is in.
    pub fn legacy(seed: u64, keys: u32) -> Self {
        Self {
            seed,
            generator: 4,
            topology: 1,
            keys,
            records: BTreeMap::new(),
            tools: 0,
            building: None,
        }
    }

    pub fn to_ron(&self) -> String {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .unwrap_or_else(|_| String::new())
    }

    pub fn from_ron(text: &str) -> Option<Self> {
        ron::from_str(text).ok()
    }

    /// Why this build cannot open the world, naming the version it lacks, or
    /// `None` when it can. A world is never opened as some other world: its
    /// ground would be generated by rules it was not made by.
    pub fn refusal(&self) -> Option<String> {
        if TerrainConfig::for_version(self.generator).is_none() {
            return Some(
                tr_with("saves.refusal.generator", &[("version", &self.generator)]).into(),
            );
        }
        if self.topology != TOPOLOGY_VERSION {
            return Some(
                tr_with(
                    "saves.refusal.topology",
                    &[("version", &self.topology), ("have", &TOPOLOGY_VERSION)],
                )
                .into(),
            );
        }
        if self.keys > KEY_VERSION {
            return Some(
                tr_with(
                    "saves.refusal.keys",
                    &[("version", &self.keys), ("have", &KEY_VERSION)],
                )
                .into(),
            );
        }
        None
    }
}

/// A slot's `world.ron`: what it is, and what a timer can afford to write.
///
/// Plain arrays rather than `Vec3`, because a save's shape should not depend
/// on whether a maths crate's serde feature happens to be on.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldFile {
    /// What the player called it.
    pub name: String,
    /// The world this save is OF. Checked on load: a save loaded into a
    /// different world is a save that silently became somebody else's.
    pub seed: u64,
    pub made_unix_s: u64,
    pub played_unix_s: u64,
    /// Planet-local metres. Absent until the first autosave, which is what a
    /// brand new slot looks like: it spawns where a new world spawns.
    pub position: Option<[f32; 3]>,
    pub heading: Option<[f32; 3]>,
    pub pitch: f32,
    pub selected: usize,
    /// World time, seconds since midnight of day 0: the hour and the season
    /// the world resumes in. Absent in a save from before the clock was
    /// saved, which opens at the clock's start.
    #[serde(default)]
    pub world_seconds: Option<f64>,
}

impl WorldFile {
    pub fn new(name: String, seed: u64, now: u64) -> Self {
        Self {
            name,
            seed,
            made_unix_s: now,
            played_unix_s: now,
            position: None,
            heading: None,
            pitch: 0.0,
            selected: 0,
            world_seconds: None,
        }
    }

    pub fn to_ron(&self) -> String {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .unwrap_or_else(|_| String::new())
    }

    pub fn from_ron(text: &str) -> Option<Self> {
        ron::from_str(text).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::terrain::Material;

    fn kit() -> Slots {
        let mut slots = Slots::new();
        slots.give(Item::Block(Material::Grass), 64);
        slots.give(Item::Block(Material::Stone), 7);
        slots.give(Item::Tool(Tool::Pickaxe), 1);
        slots.give(Item::Fish(6), 3);
        slots
    }

    /// Every tool code comes back as the tool it was, and the pickaxe keeps
    /// the placeholder's `t0`.
    #[test]
    fn every_tool_has_one_code_and_the_pickaxe_keeps_t0() {
        assert_eq!(tool_code(Tool::Pickaxe), 0);
        for tool in Tool::ALL {
            assert_eq!(tool_of(tool_code(tool)), Some(tool));
        }
        assert_eq!(tool_of(9), None);
    }

    /// A catch line carries the species, the length and the hotbar the fish
    /// went into; a change of tool carries the tool and what is owned.
    #[test]
    fn a_catch_and_a_change_of_tool_survive_the_round_trip() {
        assert_eq!(
            parse_line(&catch_line_of(5, 44, &kit())),
            Some(Record::Catch {
                species: 5,
                length_cm: 44,
                slots: kit()
            })
        );
        let mut hand = Equipment::default();
        hand.hold(Tool::Axe);
        assert_eq!(
            parse_line(&hand_line_of(&hand)),
            Some(Record::Hand { equipment: hand })
        );
        let partial = Equipment::from_parts([true, false, true, false], Some(Tool::Pickaxe));
        assert_eq!(
            parse_line(&hand_line_of(&partial)),
            Some(Record::Hand { equipment: partial })
        );
        // Bare hands write `-`, and read back bare; a line from before tiers
        // reads as it always did, at the stone tier (`inventory-grid`
        // decision 8, `tool-crafting` decision 8).
        let bare = Equipment::default();
        assert_eq!(hand_line_of(&bare), "hand - 0000\n");
        assert_eq!(
            parse_line("hand - 15"),
            Some(Record::Hand { equipment: bare })
        );
        assert_eq!(
            parse_line("hand - 5"),
            Some(Record::Hand {
                equipment: Equipment::from_parts([true, false, true, false], None)
            })
        );
        // Tiers: a new world's nothing, and an iron pickaxe in hand.
        let none = Equipment::new_world();
        assert_eq!(hand_line_of(&none), "hand - ----\n");
        assert_eq!(
            parse_line("hand - ----"),
            Some(Record::Hand { equipment: none })
        );
        let mut iron = Equipment::new_world();
        iron.grant(Tool::Pickaxe, Tier::Iron).unwrap();
        iron.grant(Tool::Rod, Tier::Stone).unwrap();
        iron.hold(Tool::Pickaxe);
        assert_eq!(hand_line_of(&iron), "hand 0 0-2-\n");
        assert_eq!(
            parse_line(&hand_line_of(&iron)),
            Some(Record::Hand { equipment: iron })
        );
        assert_eq!(
            parse_line("hand 3 15"),
            Some(Record::Hand {
                equipment: Equipment::from_parts([true; 4], Some(Tool::Rod))
            })
        );
        for bad in [
            "catch 5",
            "catch 5 44 b3,1",
            "hand",
            "hand 3",
            "hand 9 15",
            "hand 3 16",
            "hand 3 15 x",
            "hand -- 15",
            "hand - 16",
            "hand - 0-4-",
            "hand - 0-2",
            "hand - 0-2-0",
            "hand - 015",
        ] {
            assert!(parse_line(bad).is_none(), "{bad:?} parsed");
        }
    }

    /// The save is the only thing that carries a world between two runs, so
    /// what it must never do is change one on the way through.
    #[test]
    fn an_edit_and_the_hotbar_survive_the_round_trip() {
        let edit = Edit {
            cell: 4_123_456_789,
            layer: 218,
            material: Material::Air,
        };
        let record = parse_line(&line_of(edit, None, &kit())).expect("a line it just wrote");
        assert_eq!(
            record,
            Record::Edit {
                edit,
                drop: None,
                slots: Some(kit())
            }
        );
    }

    /// A kit that reaches into the pack: every slot is written, and a line
    /// carries forty fields after its head.
    fn full() -> Slots {
        let mut slots = kit();
        for k in 0..CARRIED as u16 {
            slots.give(Item::Fish(100 + k), 1);
        }
        slots
    }

    /// The pack rides the line: forty slot fields go out and come back, and
    /// the last pack slot is not dropped on the way.
    #[test]
    fn the_pack_survives_the_round_trip() {
        let slots = full();
        assert!(slots.get(CARRIED - 1).is_some(), "the pack is full");
        let line = kit_line_of(2, &slots);
        assert_eq!(line.split_whitespace().count(), 2 + CARRIED);
        assert_eq!(parse_line(&line), Some(Record::Kit { version: 2, slots }));
    }

    /// A line from before the pack carries ten slots, and reads as that
    /// hotbar with an empty pack.
    #[test]
    fn a_ten_slot_line_reads_as_an_empty_pack() {
        let record = parse_line("kit 2 b3,64 - - - - - - - - t1,1").expect("a ten-slot line");
        let Record::Kit { slots, .. } = record else {
            panic!("a kit line is a kit");
        };
        assert_eq!(slots.get(0).map(|s| s.count), Some(64));
        assert_eq!(slots.get(9).map(|s| s.item), Some(Item::Tool(Tool::Shovel)));
        assert!((SLOTS..CARRIED).all(|k| slots.get(k).is_none()));
    }

    /// A drop's field written by the f32 build before (`world-frames`
    /// 10.4: drops are f64 now) reads and writes back to the same bytes.
    #[test]
    fn an_f32_drop_field_reads_and_writes_back_the_same() {
        for text in [
            "d41:b1,1:86400.500:-1.500,299.875,12.250",
            "d3:b2,1:120.017:4436.447,1987.819,-230.918",
            "d9:b3,1:5.000:-2354.408,1953.115,3701.418",
        ] {
            let drop = drop_of(text).expect(text);
            assert_eq!(drop_text(&drop), text);
        }
        // And what the f32 build wrote for an f32 point, this one writes for
        // the same point widened.
        let p = bevy::math::DVec3::new(4436.4468, 1987.8191, -230.91805).as_vec3();
        let old = format!("{:.3},{:.3},{:.3}", p.x, p.y, p.z);
        let wide = p.as_dvec3();
        assert_eq!(format!("{:.3},{:.3},{:.3}", wide.x, wide.y, wide.z), old);
    }

    /// A dig's line carries its drop, and a pickup and a pack move are lines
    /// of their own; each comes back exactly.
    #[test]
    fn a_drop_a_pickup_and_a_pack_move_survive_the_round_trip() {
        let edit = Edit {
            cell: 4_000_000_001,
            layer: 150,
            material: Material::Air,
        };
        let drop = ItemDrop {
            id: 41,
            item: Item::Block(Material::LanternPost),
            count: 1,
            position: bevy::math::DVec3::new(-1.5, 299.875, 12.25),
            made_s: 86_400.5,
        };
        assert_eq!(
            parse_line(&line_of(edit, Some(&drop), &full())),
            Some(Record::Edit {
                edit,
                drop: Some(drop),
                slots: Some(full())
            })
        );
        // A stack the player dropped (`drop-and-trash`).
        let tossed = ItemDrop {
            id: 42,
            item: Item::Block(Material::Sand),
            count: 99,
            ..drop
        };
        assert_eq!(
            parse_line(&toss_line_of(&tossed, &full())),
            Some(Record::Toss {
                drop: tossed,
                slots: full()
            })
        );
        assert_eq!(
            parse_line(&pick_line_of(41, 3, &full())),
            Some(Record::Pick {
                id: 41,
                left: 3,
                slots: full()
            })
        );
        assert_eq!(
            parse_line(&pack_line_of(&full())),
            Some(Record::Pack { slots: full() })
        );
    }

    /// A torn drop or pickup is refused whole, like any torn line: the hole
    /// is never recorded without the block that came out of it.
    #[test]
    fn a_torn_drop_or_pickup_is_refused() {
        let ten = " - - - - - - - - - -";
        for bad in [
            "7 3 0 d1:b3,1:5.0:1,2,3".to_string(),
            format!("7 3 0 d1:b3,1:5.0:1,2{ten}"),
            format!("7 3 0 d1:b3,0:5.0:1,2,3{ten}"),
            format!("7 3 0 d1:b3,1:NaN:1,2,3{ten}"),
            format!("7 3 0 d1:b3,1{ten}"),
            format!("7 3 0 dx:b3,1:5.0:1,2,3{ten}"),
            format!("pick 1{ten}"),
            "pick 1 0 - -".to_string(),
            "pack - - -".to_string(),
        ] {
            assert!(parse_line(&bad).is_none(), "{bad:?} parsed");
        }
        assert!(parse_line(&format!("7 3 0 d1:b3,1:5.0:1,2,3{ten}")).is_some());
    }

    /// A kit line carries the version dealt and the hotbar it was dealt
    /// into, and comes back as exactly that; a kit line missing its hotbar
    /// is a torn line and is skipped like any other.
    #[test]
    fn a_dealt_kit_survives_the_round_trip() {
        let record = parse_line(&kit_line_of(2, &kit())).expect("a line it just wrote");
        assert_eq!(
            record,
            Record::Kit {
                version: 2,
                slots: kit()
            }
        );
        assert!(parse_line("kit 2").is_none(), "no hotbar");
        assert!(parse_line("kit 2 b3,64").is_none(), "a partial hotbar");
        assert!(
            parse_line("kit x - - - - - - - - - -").is_none(),
            "no version"
        );
    }

    /// The three-field line is what the store wrote before the hotbar rode the
    /// log. It is still an edit and is still applied: an old save loses the
    /// inventory it never recorded and none of the world it did.
    #[test]
    fn a_line_from_before_the_hotbar_is_still_an_edit() {
        let record = parse_line("7 3 1").expect("three fields is the old line");
        let Record::Edit { edit, drop, slots } = record else {
            panic!("an edit line is an edit");
        };
        assert_eq!(edit.cell, 7);
        assert_eq!(edit.material, Material::Stone);
        assert!(slots.is_none(), "and claims no inventory");
        assert!(drop.is_none(), "and no drop");
    }

    /// A damaged line is skipped rather than guessed at, and the damage does
    /// not spread: a torn last line costs that line and no other.
    /// Goods save by their code (`farming-and-smelting` decision 1), and
    /// the two new blocks by codes of their own.
    #[test]
    fn goods_and_the_new_blocks_survive_the_round_trip() {
        let mut slots = kit();
        slots.give(Item::Good(Good::WHEAT), 3);
        slots.give(Item::Good(Good::COAL), 8);
        slots.give(Item::Block(Material::Forge), 1);
        let line = pack_line_of(&slots);
        assert!(
            line.contains(" gwheat,3") && line.contains(" gcoal,8"),
            "{line}"
        );
        assert!(line.contains(" b19,1"), "{line}");
        let Some(Record::Pack { slots: back }) = parse_line(line.trim_end()) else {
            panic!("{line}");
        };
        assert_eq!(back, slots);
        for material in [Material::Tilled, Material::Forge] {
            assert_eq!(material_of(material_code(material)), Some(material));
        }
        // Every material has its own code.
        let codes: std::collections::BTreeSet<u8> =
            Material::ALL.iter().map(|m| material_code(*m)).collect();
        assert_eq!(codes.len(), Material::ALL.len());
    }

    /// Flowing water saves its level (`water-flow` E4): codes 64 to 79,
    /// each one material, none taken by a block; the water's own lines are
    /// authored and carry no slots; a line from before is read as it was.
    #[test]
    fn flowing_water_saves_its_level_and_an_old_line_reads_as_before() {
        let mut codes = std::collections::BTreeSet::new();
        for level in 0..=7 {
            for falling in [false, true] {
                let m = Material::flowing(level, falling);
                let code = material_code(m);
                assert!((64..80).contains(&code), "{m:?} {code}");
                assert_eq!(material_of(code), Some(m));
                codes.insert(code);
            }
        }
        assert_eq!(codes.len(), 16);
        for block in Material::ALL {
            assert!(material_code(block) < 64, "{block:?}");
        }
        let author = crate::saves::water_author();
        let edit = Edit {
            cell: 77,
            layer: 210,
            material: Material::flowing(5, true),
        };
        let line = authored(&author, world_edit_line(edit));
        assert_eq!(line, "@wwater:flow/0 77 210 77\n");
        let (by, record) = parse_entry(line.trim_end()).expect("the water's line parses");
        assert_eq!(by, author);
        assert_eq!(
            record,
            Record::Edit {
                edit,
                drop: None,
                slots: None
            }
        );
        // A player's line from before flowing water: a source is code 4.
        let (by, record) = parse_entry("12 140 4").expect("an old line");
        assert!(by.is_player());
        let Record::Edit { edit, .. } = record else {
            panic!("an edit");
        };
        assert_eq!(edit.material, Material::Water);
    }

    /// The campfire saves as code 21 and a dish by its code (`cooking`
    /// design S7); a fish stays a fish.
    #[test]
    fn a_campfire_and_a_dish_survive_the_round_trip() {
        let mut slots = kit();
        slots.give(Item::Block(Material::Campfire), 1);
        slots.give(Item::Good(Good::by_code("grilled-fish").unwrap()), 2);
        slots.give(Item::Fish(4), 3);
        let line = pack_line_of(&slots);
        assert!(line.contains(" b21,1"), "{line}");
        assert!(line.contains(" ggrilled-fish,2"), "{line}");
        let Some(Record::Pack { slots: back }) = parse_line(line.trim_end()) else {
            panic!("{line}");
        };
        assert_eq!(back, slots);
        assert_eq!(material_of(21), Some(Material::Campfire));
        assert_eq!(material_of(20), Some(Material::Workbench));
        // The sea's cells and a beaver's sticks (`ocean-environments` B24,
        // B25), each its own number, read back as itself.
        for material in [
            Material::ReefRock,
            Material::SeaIce,
            Material::Basalt,
            Material::MusselBed,
            Material::Sticks,
            Material::DesertSand,
        ] {
            assert_eq!(material_of(material_code(material)), Some(material));
        }
        assert_eq!(material_code(Material::ReefRock), 24);
        assert_eq!(material_code(Material::Sticks), 28);
        // `sand-tiles` design 5: desert sand takes the next code, and sand
        // keeps its own, so an old save's sand is read back as sand.
        assert_eq!(material_code(Material::DesertSand), 29);
        assert_eq!(material_of(6), Some(Material::Sand));
    }

    /// A good this build does not have costs its own line and no other.
    #[test]
    fn an_unknown_good_costs_only_its_line() {
        let mut line = pack_line_of(&Slots::new());
        line = line.replacen(" -", " gdragon-egg,1", 1);
        assert!(parse_line(line.trim_end()).is_none());
    }

    /// The hands-and-record line: a plot planted, with the slots after it,
    /// reads back whole; torn, it reads as nothing at all.
    #[test]
    fn a_record_with_the_hands_survives_the_round_trip() {
        let mut slots = kit();
        slots.give(Item::Good(Good::WHEAT_SEED), 3);
        let record = StoredRecord::of(
            pbd_core::farming::PLOT_KIND,
            4242,
            pbd_core::farming::PLOT_SCHEMA,
            &pbd_core::farming::Plot::sown(61, pbd_core::farming::Crop::WHEAT, 99.5),
        );
        let line = with_line_of(&record, &slots);
        assert!(line.starts_with("with ") && line.ends_with('\n'));
        let (author, parsed) = parse_entry(&line).expect("reads");
        assert_eq!(author, Author::Player(0));
        assert_eq!(
            parsed,
            Record::Handed {
                record: record.clone(),
                slots: slots.clone()
            }
        );
        let torn = &line[..line.len() - 6];
        assert!(parse_entry(torn).is_none(), "a torn body is damaged");
        let short = line.replacen("with b3,64 ", "with ", 1);
        assert!(parse_entry(&short).is_none(), "a slot short is damaged");
    }

    #[test]
    fn a_damaged_line_costs_that_line_and_no_other() {
        for bad in [
            "",
            "7",
            "7 3",
            "7 3 99",                       // no material has that code
            "7 3 1 b3,64",                  // a partial hotbar
            "7 3 1 x9,1 - - - - - - - - -", // no item kind is x
            "not a line at all",
        ] {
            assert!(parse_line(bad).is_none(), "{bad:?} parsed");
        }
        assert!(
            parse_line(&line_of(
                Edit {
                    cell: 1,
                    layer: 2,
                    material: Material::Dirt
                },
                None,
                &kit()
            ))
            .is_some(),
            "and the next line still reads"
        );
    }

    /// Every author reads back from the line they wrote, a line with no
    /// token is the player's, and a record line comes back whole (task 3.2).
    #[test]
    fn every_author_reads_back_from_their_line() {
        let edit = Edit {
            cell: 5_000_001,
            layer: 12,
            material: Material::Stone,
        };
        let process = Author::World {
            process: "growth".into(),
            kind: "settlement".into(),
            id: 111_243,
        };
        for author in [
            Author::Player(0),
            Author::Player(3),
            Author::Creation,
            process.clone(),
        ] {
            let line = authored(&author, world_edit_line(edit));
            let (who, record) = parse_entry(&line).expect("a line it just wrote");
            assert_eq!(who, author, "{line:?}");
            assert_eq!(
                record,
                Record::Edit {
                    edit,
                    drop: None,
                    slots: None
                }
            );
        }
        assert!(
            !authored(&Author::Player(0), line_of(edit, None, &kit())).starts_with('@'),
            "the player's lines stay as older builds read them"
        );
        let old = line_of(edit, None, &kit());
        assert_eq!(
            parse_entry(&old),
            parse_line(&old).map(|r| (Author::Player(0), r))
        );
        let site = StoredRecord {
            kind: "site".into(),
            id: 111_243,
            schema: 1,
            body: "(name:\"Ashingstead\",capital:true)".into(),
        };
        assert_eq!(
            parse_entry(&site.line(&Author::Creation)),
            Some((Author::Creation, Record::Stored(site)))
        );
        for bad in ["@zz 7 3 1", "@c", "rec @c site 1 1 (name:\"Ash"] {
            assert_eq!(parse_entry(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn a_world_file_survives_the_round_trip() {
        let mut file = WorldFile::new("caves".into(), 1234, 99);
        file.position = Some([1.5, -2.0, 3.25]);
        file.heading = Some([0.0, 1.0, 0.0]);
        file.pitch = -0.3;
        file.selected = 4;
        file.world_seconds = Some(123_456.75);
        let back = WorldFile::from_ron(&file.to_ron()).expect("what it just wrote");
        assert_eq!(back, file);
        assert!(WorldFile::from_ron("{ this is not ron").is_none());
    }

    /// A save written before the clock was saved still opens, at the start.
    #[test]
    fn a_world_file_from_before_the_clock_opens() {
        let mut file = WorldFile::new("old".into(), 7, 1);
        file.world_seconds = None;
        let text = file.to_ron().replace("world_seconds: None,", "");
        assert!(!text.contains("world_seconds"), "{text}");
        let back = WorldFile::from_ron(&text).expect("an older file");
        assert_eq!(back.world_seconds, None);
    }

    /// An identity reads back as it was written, a new one names this build's
    /// versions, and one written before any record kind existed still reads.
    #[test]
    fn an_identity_round_trips_and_names_this_builds_versions() {
        let made = Identity::new(0x5eed_2026);
        assert_eq!(made.generator, GENERATOR_VERSION);
        assert_eq!(made.topology, TOPOLOGY_VERSION);
        assert_eq!(made.keys, KEY_VERSION);
        assert_eq!(Identity::from_ron(&made.to_ron()), Some(made.clone()));
        let bare = "(seed: 7, generator: 4, topology: 1, keys: 1)";
        assert_eq!(
            Identity::from_ron(bare),
            Some(Identity::legacy(7, 1)),
            "no records field reads as no record kinds"
        );
        assert_eq!(made.refusal(), None);
        assert!(made.crafts_its_tools(), "a new world crafts its tools");
        assert!(
            !Identity::from_ron(bare).unwrap().crafts_its_tools(),
            "a world from before keeps the old kit"
        );
        assert_eq!(
            Identity::legacy(7, 0).refusal(),
            None,
            "an unmigrated log is migrated, not refused"
        );
    }

    /// A world made by a version this build does not have is refused, and the
    /// reason names the version (`world/persistence`: "A version the build
    /// does not have").
    #[test]
    fn a_version_this_build_does_not_have_is_refused_by_name() {
        let mut future = Identity::new(1);
        future.generator = 99;
        let why = future.refusal().expect("refused");
        assert!(why.contains("generator version 99"), "{why}");
        let mut cut = Identity::new(1);
        cut.topology = TOPOLOGY_VERSION + 1;
        assert!(
            cut.refusal()
                .expect("refused")
                .contains(&format!("topology version {}", TOPOLOGY_VERSION + 1))
        );
        let mut keyed = Identity::new(1);
        keyed.keys = KEY_VERSION + 1;
        assert!(
            keyed
                .refusal()
                .expect("refused")
                .contains("cell key version")
        );
    }
}
