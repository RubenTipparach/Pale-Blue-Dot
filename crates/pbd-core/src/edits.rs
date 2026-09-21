//! What a player changed, kept apart from what the generator says.
//!
//! A [`Column`](crate::column::Column) is GENERATED rather than stored: the
//! app builds one per cell inside the tier's reach and throws the tier away
//! when the player walks far enough for its anchor to move. An edit written
//! into a generated column would therefore live exactly as long as the player
//! stood still.
//!
//! So an edit is a sparse override, keyed by the cell's stable ID and holding
//! only the layers that differ from what the generator produces. `generate`
//! applies them last, which keeps the property the whole tier rests on: a
//! column is a pure function of its direction, the worm field and these
//! edits. Generate it twice, from anywhere, and it is the same column.

use crate::terrain::Material;
use std::collections::HashMap;

/// One layer of one cell, changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edit {
    /// The cell's stable ID, which the LOD record has carried since it was
    /// built. Stable is the whole point: a tier rebuilt at a different anchor
    /// gives a cell a different slot and the same ID.
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

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Record one change. A second edit to the same layer REPLACES the first
    /// in place rather than appending: the list is what a column is built
    /// from, so a cell dug and refilled a hundred times is two entries, not a
    /// hundred, and its position in the order does not drift.
    pub fn set(&mut self, edit: Edit) {
        let layers = self.cells.entry(edit.cell).or_default();
        match layers.iter_mut().find(|(layer, _)| *layer == edit.layer) {
            Some(slot) => slot.1 = edit.material,
            None => layers.push((edit.layer, edit.material)),
        }
    }

    /// The changes for one cell, in the order they must be applied.
    pub fn for_cell(&self, cell: u32) -> &[(u16, Material)] {
        self.cells.get(&cell).map_or(&[], Vec::as_slice)
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
}
