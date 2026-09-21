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
use pbd_core::inventory::{Item, SLOTS, Slots, Stack, Tool};
use pbd_core::terrain::Material;
use serde::{Deserialize, Serialize};

/// One line of the log: an edit, and what the player held once it was made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub edit: Edit,
    /// Absent on a line written before the hotbar rode the log. Such a line is
    /// still a real edit, so it is kept: an older save loses the inventory it
    /// never recorded and none of the world it did.
    pub slots: Option<Slots>,
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
            item: Item::Tool(Tool::Pick),
            count,
        }) => format!("t0,{count}"),
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
        "t" => match code {
            "0" => Item::Tool(Tool::Pick),
            _ => return None,
        },
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
    for index in 0..SLOTS {
        line.push(' ');
        line.push_str(&stack_text(slots.get(index)));
    }
    line.push('\n');
    line
}

pub fn parse_line(line: &str) -> Option<Record> {
    let mut parts = line.split_whitespace();
    let cell = parts.next()?.parse().ok()?;
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
        SLOTS => {
            let mut slots = Slots::new();
            for (index, text) in carried.iter().enumerate() {
                slots.set(index, stack_of(text)?);
            }
            Some(slots)
        }
        // Anything between is a line that was being written when the power
        // went out. Its edit looks intact and its hotbar is half there, and
        // taking the edit alone would record the hole without the block that
        // came out of it - which is the exact inconsistency the hotbar rides
        // this line to prevent.
        _ => return None,
    };
    Some(Record {
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
        slots.give(Item::Tool(Tool::Pick), 1);
        slots
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
        assert_eq!(record.edit, edit);
        assert_eq!(record.slots.as_ref(), Some(&kit()));
    }

    /// The three-field line is what the store wrote before the hotbar rode the
    /// log. It is still an edit and is still applied: an old save loses the
    /// inventory it never recorded and none of the world it did.
    #[test]
    fn a_line_from_before_the_hotbar_is_still_an_edit() {
        let record = parse_line("7 3 1").expect("three fields is the old line");
        assert_eq!(record.edit.cell, 7);
        assert_eq!(record.edit.material, Material::Stone);
        assert!(record.slots.is_none(), "and claims no inventory");
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
        let back = WorldFile::from_ron(&file.to_ron()).expect("what it just wrote");
        assert_eq!(back, file);
        assert!(WorldFile::from_ron("{ this is not ron").is_none());
    }
}
