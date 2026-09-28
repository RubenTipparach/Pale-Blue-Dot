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

use bevy::math::Vec3;
use pbd_core::drops::ItemDrop;
use pbd_core::edits::Edit;
use pbd_core::inventory::{CARRIED, Equipment, Item, SLOTS, Slots, Stack, Tool};
use pbd_core::planet_gen::TerrainConfig;
use pbd_core::terrain::{GENERATOR_VERSION, Material, TOPOLOGY_VERSION};
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

/// The material a saved code names, and the code it is saved as.
///
/// Written out both ways rather than transmuted: a code nothing names must not
/// become a material, and a material added tomorrow must not silently take
/// another's number.
pub fn material_code(material: Material) -> u8 {
    match material {
        Material::Air => 0,
        Material::Stone => 1,
        Material::Soil => 2,
        Material::Grass => 3,
        Material::Water => 4,
        Material::Ore => 5,
        Material::Sand => 6,
        Material::DryGrass => 7,
        Material::JungleGrass => 8,
        Material::Snow => 9,
        Material::Rock => 10,
        Material::Dirt => 11,
        Material::Torch => 12,
        Material::LanternPost => 13,
        Material::LanternWall => 14,
        Material::LanternHanging => 15,
        Material::Brazier => 16,
        Material::Candle => 17,
    }
}

pub fn material_of(code: u8) -> Option<Material> {
    Some(match code {
        0 => Material::Air,
        1 => Material::Stone,
        2 => Material::Soil,
        3 => Material::Grass,
        4 => Material::Water,
        5 => Material::Ore,
        6 => Material::Sand,
        7 => Material::DryGrass,
        8 => Material::JungleGrass,
        9 => Material::Snow,
        10 => Material::Rock,
        11 => Material::Dirt,
        12 => Material::Torch,
        13 => Material::LanternPost,
        14 => Material::LanternWall,
        15 => Material::LanternHanging,
        16 => Material::Brazier,
        17 => Material::Candle,
        _ => return None,
    })
}

fn stack_text(stack: Option<Stack>) -> String {
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
    }
}

fn stack_of(text: &str) -> Option<Option<Stack>> {
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
/// millimetre and a millisecond are finer than anything a drop does.
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
    let mut xyz = fields.next()?.split(',').map(|v| v.parse::<f32>().ok());
    let position = Vec3::new(xyz.next()??, xyz.next()??, xyz.next()??);
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

/// `hand tool owned`, one line: the tool's code and one bit per tool owned,
/// in `Tool::ALL` order.
pub fn hand_line_of(equipment: &Equipment) -> String {
    let owned = equipment
        .owned()
        .iter()
        .enumerate()
        .fold(0u8, |bits, (i, owns)| bits | (u8::from(*owns) << i));
    format!("{HAND} {} {owned}\n", tool_code(equipment.held()))
}

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
    if head == HAND {
        let held = tool_of(parts.next()?.parse().ok()?)?;
        let bits: u8 = parts.next()?.parse().ok()?;
        if parts.next().is_some() || bits >= 1 << Tool::ALL.len() {
            return None;
        }
        let mut owned = [false; 4];
        for (i, owns) in owned.iter_mut().enumerate() {
            *owns = bits & (1 << i) != 0;
        }
        return Some(Record::Hand {
            equipment: Equipment::from_parts(owned, held),
        });
    }
    if head == PACK {
        let carried: Vec<&str> = parts.collect();
        return Some(Record::Pack {
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
}

impl Identity {
    /// A world this build makes.
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            generator: GENERATOR_VERSION,
            topology: TOPOLOGY_VERSION,
            keys: KEY_VERSION,
            records: BTreeMap::new(),
        }
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
            return Some(format!(
                "terrain generator version {}, which this build does not have",
                self.generator
            ));
        }
        if self.topology != TOPOLOGY_VERSION {
            return Some(format!(
                "topology version {}, where this build has {TOPOLOGY_VERSION}",
                self.topology
            ));
        }
        if self.keys > KEY_VERSION {
            return Some(format!(
                "cell key version {}, where this build has {KEY_VERSION}",
                self.keys
            ));
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
        let partial = Equipment::from_parts([true, false, true, false], Tool::Pickaxe);
        assert_eq!(
            parse_line(&hand_line_of(&partial)),
            Some(Record::Hand { equipment: partial })
        );
        for bad in [
            "catch 5",
            "catch 5 44 b3,1",
            "hand",
            "hand 3",
            "hand 9 15",
            "hand 3 16",
            "hand 3 15 x",
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
            position: Vec3::new(-1.5, 299.875, 12.25),
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
