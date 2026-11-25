//! ACCEPT: the half of a change to the world that can refuse
//! (`edit-pipeline` design, "Accept as a core rule"; `multiplayer` task 2.2).
//!
//! Each rule here is a function of the world's state, the player and what
//! they asked for, returning the change or a refusal. None reads a frame, a
//! mouse or a camera, so a client predicting its own click and a server
//! deciding it run the same code.

use crate::column::{LAYERS, dug_sand};
use crate::edits::Edits;
use crate::inventory::{Item, Slots};
use crate::planet_gen::TerrainConfig;
use crate::terrain::Material;

/// What an edit does to the hands, settled before the edit is written: the
/// log line carries the slots after the edit, so a dig recorded without the
/// block it yielded is a world where the hole is durable and the block was
/// never picked up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hands {
    /// A dig: take what was there.
    Take,
    /// A place: spend one of the selected stack.
    Spend,
    /// A scripted edit, which has no player and no hands.
    Empty,
    /// A till (`farming-and-smelting` decision 2): the hands take nothing,
    /// but the soil may turn up a seed, which floats out as a dig's block
    /// does, on the edit's own line.
    Till(Option<Item>),
    /// Trade one of the selected stack for this item, on the same line: a
    /// bucket filled or emptied (`water-flow` E11).
    Swap(Item),
    /// A dig that also turns up finds into the hands, on the same line: a
    /// seed out of dug grass (`farming-and-smelting` decision 14), fibre out
    /// of grass and loose stone out of earth (`tool-crafting` decision 2),
    /// bait out of earth by the water (`starter-missions` G1, `dig-fixes` 2).
    TakeFinds([Option<Item>; 3]),
}

/// Why an edit was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    /// The layer is bedrock or above the world.
    OutOfWorld,
    /// The cell's layer already holds that.
    Unchanged,
    /// The column already carries a lamp: one lamp to a column, because the
    /// record carries one lamp layer.
    LampTaken,
    /// Nothing in the selected slot to place or use.
    EmptySlot,
}

/// One edit, as asked.
#[derive(Clone, Copy, Debug)]
pub struct Ask {
    pub cell: u32,
    pub layer: usize,
    /// What the layer is to become: `Air` is a dig.
    pub material: Material,
    /// What the layer holds now.
    pub was: Material,
    /// Whether the cell's column already carries a lamp.
    pub lamp_in_column: bool,
    pub hands: Hands,
}

/// An accepted edit's effect on the player.
#[derive(Clone, Debug, PartialEq)]
pub struct Taken {
    /// The slots after the edit.
    pub slots: Slots,
    /// What floats out at the cell, for the magnet to bring in.
    pub floats: Option<Item>,
}

/// Whether a layer can be edited at all: not bedrock, not above the world.
pub fn in_world(layer: usize) -> bool {
    layer > 0 && layer < LAYERS
}

/// Accept or refuse one edit (`edit-pipeline`'s ACCEPT). `yields` is what a
/// dug block gives, asked only for a dig ([`yield_of`]).
pub fn edit(ask: Ask, slots: &Slots, yields: impl FnOnce() -> Item) -> Result<Taken, Refused> {
    if !in_world(ask.layer) {
        return Err(Refused::OutOfWorld);
    }
    if ask.was == ask.material {
        return Err(Refused::Unchanged);
    }
    if ask.material.is_lamp() && ask.lamp_in_column {
        return Err(Refused::LampTaken);
    }
    let mut moved = slots.clone();
    let mut floats = None;
    match ask.hands {
        // The dug block floats where it was cut and the magnet brings it in
        // (`inventory-grid` decision 4); what does not fit of a find is not
        // given, a find is not a block the player is owed.
        Hands::Take | Hands::TakeFinds(_) | Hands::Till(Some(_)) => {
            if let Hands::TakeFinds(finds) = ask.hands {
                for find in finds.into_iter().flatten() {
                    moved.give(find, 1);
                }
            }
            floats = Some(match ask.hands {
                Hands::Till(Some(item)) => item,
                _ => yields(),
            });
        }
        Hands::Spend => {
            let index = moved.selected();
            if moved.take(index, 1) != 1 {
                return Err(Refused::EmptySlot);
            }
        }
        Hands::Swap(item) => {
            let index = moved.selected();
            if moved.take(index, 1) != 1 {
                return Err(Refused::EmptySlot);
            }
            moved.give(item, 1);
        }
        Hands::Empty | Hands::Till(None) => {}
    }
    Ok(Taken {
        slots: moved,
        floats,
    })
}

/// A move in the pack (`multiplayer` design 19), as the pack screen
/// resolves a click.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackMove {
    /// Put `count` of slot `from` down on slot `to` (`Slots::shift`).
    Shift { from: usize, to: usize, count: u16 },
    /// Send a stack between the hotbar and the pack (`Slots::send`).
    Send { index: usize },
    /// Destroy `count` of a stack (the trash).
    Trash { index: usize, count: u16 },
}

/// Accept or refuse a move in the pack: the slots after it. Refused, as
/// [`Refused::EmptySlot`], when the slot it takes from is empty or holds
/// fewer than it would take, or the move names a slot past the player's.
pub fn pack(slots: &Slots, mv: PackMove) -> Result<Slots, Refused> {
    let (from, count) = match mv {
        PackMove::Shift { from, count, .. } => (from, count),
        PackMove::Send { index } => (index, 1),
        PackMove::Trash { index, count } => (index, count),
    };
    let held = slots.get(from).map_or(0, |stack| stack.count);
    if count == 0 || held < count {
        return Err(Refused::EmptySlot);
    }
    let mut moved = slots.clone();
    match mv {
        PackMove::Shift { from, to, count } => {
            if to >= crate::inventory::CARRIED {
                return Err(Refused::EmptySlot);
            }
            moved.shift(from, to, count);
        }
        PackMove::Send { index } => moved.send(index),
        PackMove::Trash { index, count } => {
            moved.take(index, count);
        }
    }
    Ok(moved)
}

/// A pickup of `drop` into `slots` by the give order: the slots after it
/// and how many are left on the drop. What does not fit stays on the drop.
pub fn pick(drop: &crate::drops::ItemDrop, slots: &Slots) -> (Slots, u16) {
    let mut taken = slots.clone();
    let left = taken.give(drop.item, drop.count);
    (taken, left)
}

/// What a dug block yields (`farming-and-smelting` decisions 2 and 10):
/// tilled soil gives dirt; a generated ore cell gives mixed ore or, in a coal
/// seam, coal; a placed one gives ore; generated sand in a desert cell gives
/// desert sand (`sand-tiles` design 5); everything else gives itself.
pub fn yield_of(
    terrain: &TerrainConfig,
    edits: &Edits,
    was: Material,
    cell: u32,
    layer: usize,
) -> Item {
    let placed = |material: Material| {
        edits
            .for_cell(cell)
            .iter()
            .any(|&(l, m)| usize::from(l) == layer && m == material)
    };
    match was {
        Material::Tilled => Item::Block(Material::Dirt),
        Material::Ore => crate::station::ore_yield(cell, layer as u16, placed(Material::Ore)),
        Material::Sand => {
            let placed = placed(Material::Sand);
            match crate::cell_key::key_direction(cell) {
                Some(direction) => Item::Block(dug_sand(terrain, direction, placed)),
                None => Item::Block(Material::Sand),
            }
        }
        other => Item::Block(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(was: Material, material: Material, hands: Hands) -> Ask {
        Ask {
            cell: 1,
            layer: 200,
            material,
            was,
            lamp_in_column: false,
            hands,
        }
    }

    fn holding(item: Item, count: u16) -> Slots {
        let mut slots = Slots::default();
        slots.give(item, count);
        slots
    }

    /// The refusals, in `apply_edit`'s order, with no Bevy app.
    #[test]
    fn an_edit_is_refused_out_of_the_world_unchanged_or_on_a_second_lamp() {
        let slots = Slots::default();
        let stone = || Item::Block(Material::Stone);
        for layer in [0, LAYERS, LAYERS + 3] {
            let a = Ask {
                layer,
                ..ask(Material::Stone, Material::Air, Hands::Take)
            };
            assert_eq!(edit(a, &slots, stone), Err(Refused::OutOfWorld));
        }
        let same = ask(Material::Stone, Material::Stone, Hands::Take);
        assert_eq!(edit(same, &slots, stone), Err(Refused::Unchanged));
        let torch = Ask {
            lamp_in_column: true,
            ..ask(Material::Air, Material::Torch, Hands::Spend)
        };
        assert_eq!(
            edit(torch, &holding(Item::Block(Material::Torch), 1), stone),
            Err(Refused::LampTaken)
        );
        let nothing = ask(Material::Air, Material::Stone, Hands::Spend);
        assert_eq!(edit(nothing, &slots, stone), Err(Refused::EmptySlot));
        let bucket = ask(
            Material::Water,
            Material::Air,
            Hands::Swap(Item::Block(Material::Water)),
        );
        assert_eq!(edit(bucket, &slots, stone), Err(Refused::EmptySlot));
    }

    /// Design 19: a pack move is the slots' own call, refused from an empty
    /// slot or for more than it holds.
    #[test]
    fn a_pack_move_is_applied_or_refused() {
        let stone = Item::Block(Material::Stone);
        let slots = holding(stone, 5);
        let mut by_hand = slots.clone();
        by_hand.shift(0, 3, 2);
        assert_eq!(
            pack(
                &slots,
                PackMove::Shift {
                    from: 0,
                    to: 3,
                    count: 2
                }
            ),
            Ok(by_hand)
        );
        let mut sent = slots.clone();
        sent.send(0);
        assert_eq!(pack(&slots, PackMove::Send { index: 0 }), Ok(sent));
        assert_eq!(
            pack(&slots, PackMove::Trash { index: 0, count: 5 }).map(|s| s.get(0)),
            Ok(None)
        );
        assert_eq!(
            pack(&slots, PackMove::Trash { index: 0, count: 6 }),
            Err(Refused::EmptySlot)
        );
        assert_eq!(
            pack(&slots, PackMove::Send { index: 4 }),
            Err(Refused::EmptySlot)
        );
        assert_eq!(
            pack(
                &slots,
                PackMove::Shift {
                    from: 0,
                    to: 999,
                    count: 1
                }
            ),
            Err(Refused::EmptySlot)
        );
    }

    /// A pickup gives by the give order and leaves what does not fit.
    #[test]
    fn a_pickup_gives_what_fits() {
        let drop = crate::drops::ItemDrop {
            id: 4,
            item: Item::Block(Material::Dirt),
            count: 3,
            position: glam::DVec3::ZERO,
            made_s: 0.0,
        };
        let (slots, left) = pick(&drop, &Slots::default());
        assert_eq!((slots, left), (holding(Item::Block(Material::Dirt), 3), 0));
    }

    /// A dig floats its yield and leaves the slots; a dig with finds gives
    /// them; a place spends one; a swap trades one; a scripted edit touches
    /// nothing.
    #[test]
    fn an_accepted_edit_settles_the_hands() {
        let dug = edit(
            ask(Material::Stone, Material::Air, Hands::Take),
            &Slots::default(),
            || Item::Block(Material::Rock),
        )
        .unwrap();
        assert_eq!(dug.floats, Some(Item::Block(Material::Rock)));
        assert_eq!(dug.slots, Slots::default());

        let find = Item::Block(Material::Sticks);
        let finds = edit(
            ask(
                Material::Grass,
                Material::Air,
                Hands::TakeFinds([Some(find), None, None]),
            ),
            &Slots::default(),
            || Item::Block(Material::Dirt),
        )
        .unwrap();
        assert_eq!(finds.slots, holding(find, 1));

        let stone = Item::Block(Material::Stone);
        let placed = edit(
            ask(Material::Air, Material::Stone, Hands::Spend),
            &holding(stone, 3),
            || unreachable!("a place yields nothing"),
        )
        .unwrap();
        assert_eq!(placed.slots, holding(stone, 2));
        assert_eq!(placed.floats, None);

        let scripted = edit(
            ask(Material::Stone, Material::Air, Hands::Empty),
            &holding(stone, 3),
            || unreachable!(),
        )
        .unwrap();
        assert_eq!(
            scripted,
            Taken {
                slots: holding(stone, 3),
                floats: None
            }
        );
    }
}
