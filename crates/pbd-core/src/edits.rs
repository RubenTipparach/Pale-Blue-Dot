//! What a player changed, kept apart from what the generator says.
//!
//! A [`Column`](crate::column::Column) is GENERATED rather than stored: the
//! app builds one per cell inside the tier's reach and throws the tier away
//! when the player walks far enough for its anchor to move. An edit written
//! into a generated column would therefore live exactly as long as the player
//! stood still.
//!
//! So an edit is a sparse override, keyed by the cell's exact key
//! ([`cell_key`](crate::cell_key), unique to the cell) and holding
//! only the layers that differ from what the generator produces. `generate`
//! applies them last, which keeps the property the whole tier rests on: a
//! column is a pure function of its direction, the worm field and these
//! edits. Generate it twice, from anywhere, and it is the same column.

use crate::terrain::Material;
use std::collections::HashMap;

/// One layer of one cell, changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edit {
    /// The cell's exact key ([`cell_key`](crate::cell_key)), which the LOD
    /// record carries. Stable is the whole point: a tier rebuilt at a
    /// different anchor gives a cell a different slot and the same key. And
    /// unique: no other cell has it, so an edit lands on this cell alone.
    pub cell: u32,
    /// The layer, as `column::layer_altitude` indexes them.
    pub layer: u16,
    /// What stands there now. `Air` is a dig; anything else is a place.
    pub material: Material,
}

/// Every edit in the world, by cell.
///
/// The outer map is a keyed lookup and nothing iterates it, which is the one
/// use of a hash map this project's rules allow. What IS iterated - the layers
/// of one cell - is a `Vec` in the order the edits were made, because applying
/// two edits to one layer in a different order is a different column, and a
/// randomized order must never decide that.
#[derive(Clone, Debug, Default)]
pub struct Edits {
    cells: HashMap<u32, Vec<(u16, Material)>>,
    /// Bumped by every [`Edits::set`]: what a thing derived from the edits
    /// (the player's far lights) compares to know it is stale.
    revision: u64,
    /// The glowing mushrooms picked (`glowing-mushrooms` design 4), which the
    /// column build reads beside the layers. Not a layer change: they bump
    /// their own count, never [`Edits::revision`].
    pub glow: GlowPicks,
}

/// The glowing mushrooms players have picked, by cell key, with the world
/// day of each pick (the save's `glow-plant` records, folded in at load and
/// on each pick), and the clock the build judges their regrowth by.
///
/// A keyed lookup only: nothing iterates the map to decide anything saved.
#[derive(Clone, Debug, Default)]
pub struct GlowPicks {
    picked: HashMap<u32, f64>,
    /// The world day now, as the build reads it.
    pub day: f64,
    /// How far from the camera a due pick's cell must be built to grow
    /// back, m (`clutter_radius_m`): never in front of the player.
    pub far_m: f32,
    /// Bumped by every pick.
    count: u64,
}

impl GlowPicks {
    /// A pick of the cell `key` on world day `day`.
    pub fn pick(&mut self, key: u32, day: f64) {
        self.picked.insert(key, day);
        self.count += 1;
    }

    /// The day `key` was last picked, if it was.
    pub fn picked_on(&self, key: u32) -> Option<f64> {
        self.picked.get(&key).copied()
    }

    /// How many picks there have been: different after every [`pick`].
    ///
    /// [`pick`]: GlowPicks::pick
    pub fn count(&self) -> u64 {
        self.count
    }

    /// The picked cells, in key order.
    pub fn cells(&self) -> Vec<u32> {
        let mut keys: Vec<u32> = self.picked.keys().copied().collect();
        keys.sort_unstable();
        keys
    }

    /// Whether a cell built now, `from_camera_m` metres from the camera,
    /// stands picked: picked and not yet grown back, and a pick whose
    /// regrowth has passed grows back only where the cell is built out of
    /// sight, farther than [`GlowPicks::far_m`] (design 4).
    pub fn stands_picked(&self, key: u32, from_camera_m: f32) -> bool {
        let Some(day) = self.picked_on(key) else {
            return false;
        };
        let due = self.day - day >= crate::flora::GLOW_PLANT_REGROW_DAYS;
        !(due && from_camera_m > self.far_m)
    }
}

impl Edits {
    pub fn new() -> Self {
        Self::default()
    }

    /// How many cells carry an edit.
    pub fn cells(&self) -> usize {
        self.cells.len()
    }

    /// How many layer changes are held in total.
    pub fn len(&self) -> usize {
        self.cells.values().map(Vec::len).sum()
    }

    /// How many changes have been recorded: different after every
    /// [`Edits::set`], so anything derived from the edits can tell it is stale.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Record one change. A second edit to the same layer REPLACES the first
    /// in place rather than appending: the list is what a column is built
    /// from, so a cell dug and refilled a hundred times is two entries, not a
    /// hundred, and its position in the order does not drift.
    pub fn set(&mut self, edit: Edit) {
        self.revision += 1;
        let layers = self.cells.entry(edit.cell).or_default();
        match layers.iter_mut().find(|(layer, _)| *layer == edit.layer) {
            Some(slot) => slot.1 = edit.material,
            None => layers.push((edit.layer, edit.material)),
        }
    }

    /// Forget every change to `cell`: the cell is as the seed made it. Only a
    /// region replaced whole from the server's snapshot does this
    /// (`multiplayer` design 29); a player's edit never takes one away.
    pub fn clear_cell(&mut self, cell: u32) {
        if self.cells.remove(&cell).is_some() {
            self.revision += 1;
        }
    }

    /// The changes for one cell, in the order they must be applied.
    pub fn for_cell(&self, cell: u32) -> &[(u16, Material)] {
        self.cells.get(&cell).map_or(&[], Vec::as_slice)
    }

    /// The cells whose changes are not what `older` holds for them, in key
    /// order: what something built from `older` has not seen (a fine set
    /// requested before them, `edit-pipeline` 1b). Changes are only ever added
    /// or replaced, never taken away, so every cell `older` names is here too
    /// and comparing this side's cells is the whole of it.
    pub fn changed_since(&self, older: &Edits) -> Vec<u32> {
        if self.revision == older.revision {
            return Vec::new();
        }
        let mut cells: Vec<u32> = self
            .cells
            .iter()
            .filter(|(cell, layers)| older.for_cell(**cell) != layers.as_slice())
            .map(|(cell, _)| *cell)
            .collect();
        cells.sort_unstable();
        cells
    }

    /// Every edit, cell by cell, for writing a save. Sorted by cell ID so the
    /// same world writes the same file: a save whose bytes depend on a hash
    /// map's order is a save that cannot be compared.
    pub fn all(&self) -> Vec<Edit> {
        let mut cells: Vec<&u32> = self.cells.keys().collect();
        cells.sort_unstable();
        cells
            .into_iter()
            .flat_map(|&cell| {
                self.cells[&cell]
                    .iter()
                    .map(move |&(layer, material)| Edit {
                        cell,
                        layer,
                        material,
                    })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `planet/flora`, "A picked glowing mushroom grows back out of sight":
    /// a pick stands until its regrowth, and after it stands still where
    /// the cell is built within `far_m` of the camera; only farther does it
    /// grow back. A cell never picked is never picked, and picks are not
    /// layer edits.
    #[test]
    fn a_pick_grows_back_only_out_of_sight() {
        let mut edits = Edits::new();
        edits.glow.far_m = 120.0;
        edits.glow.pick(42, 10.0);
        assert_eq!(edits.revision(), 0, "a pick is not a layer edit");
        assert_eq!(edits.glow.count(), 1);
        edits.glow.day = 10.0 + crate::flora::GLOW_PLANT_REGROW_DAYS * 0.5;
        assert!(edits.glow.stands_picked(42, 20.0));
        assert!(edits.glow.stands_picked(42, 500.0), "not yet due");
        edits.glow.day = 10.0 + crate::flora::GLOW_PLANT_REGROW_DAYS;
        assert!(
            edits.glow.stands_picked(42, 20.0),
            "due, but in front of the player"
        );
        assert!(!edits.glow.stands_picked(42, 121.0), "due and out of sight");
        assert!(!edits.glow.stands_picked(7, 0.0));
        assert_eq!(edits.glow.cells(), vec![42]);
    }

    #[test]
    fn an_edit_to_the_same_layer_replaces_rather_than_grows() {
        let mut edits = Edits::new();
        edits.set(Edit {
            cell: 7,
            layer: 60,
            material: Material::Air,
        });
        edits.set(Edit {
            cell: 7,
            layer: 61,
            material: Material::Stone,
        });
        edits.set(Edit {
            cell: 7,
            layer: 60,
            material: Material::Dirt,
        });
        assert_eq!(edits.len(), 2, "two layers touched, however many times");
        assert_eq!(
            edits.for_cell(7),
            &[(60, Material::Dirt), (61, Material::Stone)],
            "the order is the order they were first made in"
        );
    }

    #[test]
    fn an_untouched_cell_has_nothing_to_apply() {
        let edits = Edits::new();
        assert!(edits.for_cell(1234).is_empty());
        assert!(edits.is_empty());
    }

    #[test]
    fn every_edit_comes_back_in_a_stable_order() {
        let mut edits = Edits::new();
        for cell in [9u32, 3, 7] {
            edits.set(Edit {
                cell,
                layer: 10,
                material: Material::Air,
            });
        }
        let first = edits.all();
        assert_eq!(
            first.iter().map(|e| e.cell).collect::<Vec<_>>(),
            vec![3, 7, 9],
            "sorted by cell, so a save's bytes do not depend on a hash order"
        );
        assert_eq!(first, edits.all(), "and the same every time it is asked");
    }

    #[test]
    fn the_cells_changed_since_a_copy_are_the_ones_it_has_not_seen() {
        let edit = |cell, layer, material| Edit {
            cell,
            layer,
            material,
        };
        let mut edits = Edits::new();
        edits.set(edit(5, 10, Material::Air));
        edits.set(edit(8, 10, Material::Stone));
        let copy = edits.clone();
        assert!(edits.changed_since(&copy).is_empty(), "nothing since");
        // A new cell, a new layer of one the copy has, and one replaced.
        edits.set(edit(12, 200, Material::Torch));
        edits.set(edit(5, 11, Material::Air));
        edits.set(edit(8, 10, Material::Dirt));
        edits.set(edit(3, 40, Material::Candle));
        assert_eq!(
            edits.changed_since(&copy),
            vec![3, 5, 8, 12],
            "in key order"
        );
        assert!(copy.changed_since(&copy).is_empty());
    }
}
