//! What a save looks like on disk.
//!
//! Two files per slot and two shapes, because the two halves change on
//! different clocks. `edits.log` is append-only and carries a line per accepted
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

use pbd_core::edits::Edit;
use pbd_core::inventory::{Equipment, Item, SLOTS, Slots, Stack, Tool};
use pbd_core::terrain::Material;
use serde::{Deserialize, Serialize};

/// One line of the log: an edit, and what the player held once it was made,
/// or a kit dealt, and what the player held once it was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    Edit {
        edit: Edit,
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
}

/// The kit line's leading token, which no cell number can be.
const KIT: &str = "kit";
/// A catch's.
const CATCH: &str = "catch";
/// A change of tool's.
const HAND: &str = "hand";

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

/// `cell layer material s0 s1 .. s9`, one line.
pub fn line_of(edit: Edit, slots: &Slots) -> String {
    let mut line = format!(
        "{} {} {}",
        edit.cell,
        edit.layer,
        material_code(edit.material)
    );
    push_slots(&mut line, slots);
    line
}

/// `kit version s0 s1 .. s9`, one line.
pub fn kit_line_of(version: u32, slots: &Slots) -> String {
    let mut line = format!("{KIT} {version}");
    push_slots(&mut line, slots);
    line
}

fn push_slots(line: &mut String, slots: &Slots) {
    for index in 0..SLOTS {
        line.push(' ');
        line.push_str(&stack_text(slots.get(index)));
    }
    line.push('\n');
}

/// `catch species length_cm s0 s1 .. s9`, one line.
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

/// The ten slot fields of a line, or `None` where there are not exactly ten.
fn slots_of(carried: &[&str]) -> Option<Slots> {
    if carried.len() != SLOTS {
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
    let carried: Vec<&str> = parts.collect();
    // A line from before the hotbar rode the log has three fields, and it is a
    // real edit. Anything that is neither three fields nor thirteen is damaged
    // and its inventory is not guessed at.
    let slots = match carried.len() {
        // Three fields: a line from before the hotbar rode the log. A real
        // edit, and it claims no inventory.
        0 => None,
        // Anything between three and thirteen is a line that was being
        // written when the power went out. Its edit looks intact and its
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
        slots,
    })
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
        let record = parse_line(&line_of(edit, &kit())).expect("a line it just wrote");
        assert_eq!(
            record,
            Record::Edit {
                edit,
                slots: Some(kit())
            }
        );
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
        let Record::Edit { edit, slots } = record else {
            panic!("an edit line is an edit");
        };
        assert_eq!(edit.cell, 7);
        assert_eq!(edit.material, Material::Stone);
        assert!(slots.is_none(), "and claims no inventory");
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
}
