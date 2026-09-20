//! Voxel light over a region of columns: what a cell can see of the sky, and
//! how dark a corner goes where it is wedged into stone.
//!
//! Ported from Tenebris's `world_light.rs` and the corner half of its
//! `hex_mesher.rs`, both read rather than guessed at. The two halves are what
//! the owner named - "voxel lighting algorithm combined with baked vertex
//! colors" - and they do different jobs:
//!
//! - **[`bake`] is the FIELD.** Daylight is seeded down every column and
//!   flooded outward, one level per step, stopping at anything solid. A cave
//!   is dark because nothing reached it, not because a depth term was
//!   subtracted, which is why a cave MOUTH stays bright.
//! - **[`corner`] is the CONTACT.** A vertex takes the mean of the field over
//!   the cells that meet at its own corner, and darkens it by how many of the
//!   neighbours there are solid. That is the crease where one block sits on
//!   another, and it varies across a face because its corners were sampled
//!   apart.
//!
//! **The reference's second channel is deliberately not here.** Its byte is
//! `(sky << 4) | block`, and in the running game the block nibble is always
//! zero: `hex_mesher.rs`'s own comment says torches are NOT seeded into the
//! BFS, because a torch placement measured "~1000 ms / 17M ops" against
//! "0.4 ms" for an ordinary edit, so they became a separate per-vertex
//! proximity sum instead. Carrying a channel that the project it comes from
//! abandoned would be a field nobody fills. When a lamp exists here, it
//! follows the model that RUNS there.

use crate::column::{Column, LAYERS};

/// The brightest a cell can be, and so the number of steps light survives.
/// Tenebris's `voxel_sky_max`, shipped at 15.
pub const MAX: u8 = 15;

/// What one step between neighbouring cells costs, in levels.
///
/// The reference's `voxel_sky_lateral_loss`, shipped at 1, and its vertical
/// step is 1 too - the comment beside it says "Vertical neighbours stay at 1
/// per step so daylight shafts remain at full strength". One cost for every
/// direction, so the falloff from a cave mouth is 15 cells whichever way the
/// tunnel runs.
pub const STEP: u8 = 1;

/// How much a corner is darkened by the neighbours it is wedged between.
///
/// Notch's three-step ladder, which is the reference's own comment and its own
/// numbers. The INDEX is how many of the two side neighbours are solid at the
/// sampled layer, which the reference chose over a Minecraft-style eight
/// neighbour kernel for a stated reason: for a cap the cell itself is always
/// air and for a side face always solid, so neither says anything about how
/// exposed the corner is, and the two side neighbours do.
pub const CONTACT: [f32; 3] = [1.0, 0.85, 0.70];

/// The columns being lit and how they join.
///
/// `neighbors[i][s]` is the index of column `i`'s neighbour across side `s`,
/// or [`OFF_REGION`] where there is none. A pentagon uses five sides and
/// leaves the sixth off the region.
pub struct Region<'a> {
    pub columns: &'a [Column],
    pub neighbors: &'a [[u32; 6]],
}

/// A neighbour that is not in this region.
///
/// Treated as SOLID, never as open: light must not flood out of a region's
/// edge and come back wrong, and a wall at the edge is the same assumption the
/// column tier already makes about a cell whose neighbour has no column.
pub const OFF_REGION: u32 = u32::MAX;

/// Every cell's sky level, one array of [`LAYERS`] per column.
pub type Baked = Vec<[u8; LAYERS]>;

impl Region<'_> {
    /// Whether a cell stops light. Off the region counts as solid.
    fn opaque(&self, column: u32, layer: usize) -> bool {
        match self.columns.get(column as usize) {
            Some(col) => col.solid(layer),
            None => true,
        }
    }
}

/// Light a whole region from the sky.
///
/// Two phases, which is the reference's `rebuild`:
///
/// 1. **Seed**, per column: walk down from the top and stop at the first solid
///    layer, setting every cell above it to [`MAX`]. That is
///    `sky_seed_one_column`, and it is what gives a daylight shaft its full
///    strength all the way down.
/// 2. **Flood**, breadth first: pop a cell, hand `level - STEP` to every
///    neighbour that is not solid and not already at least that bright.
///
/// **Every seeded cell goes on the queue, not just the lowest.** That looks
/// like an easy saving and is a bug: a column open to the sky is adjacent to
/// its neighbour at EVERY layer, so a cave at layer 60 beside an open column
/// is lit by that column's layer 60 and by nothing else. Seeding only the
/// lowest open cell leaves the ones above it at [`MAX`] and never on the
/// queue, so they light nothing sideways and the cave stays black.
pub fn bake(region: &Region) -> Baked {
    let mut light = vec![[0u8; LAYERS]; region.columns.len()];
    let mut queue: std::collections::VecDeque<(u32, u16)> = std::collections::VecDeque::new();
    for (index, column) in region.columns.iter().enumerate() {
        for layer in (0..LAYERS).rev() {
            if column.solid(layer) {
                break;
            }
            light[index][layer] = MAX;
            queue.push_back((index as u32, layer as u16));
        }
    }
    while let Some((column, layer)) = queue.pop_front() {
        let level = light[column as usize][layer as usize];
        if level <= STEP {
            continue;
        }
        let next = level - STEP;
        let give =
            |light: &mut Baked, queue: &mut std::collections::VecDeque<_>, to: u32, at: usize| {
                if region.opaque(to, at) || light[to as usize][at] >= next {
                    return;
                }
                light[to as usize][at] = next;
                queue.push_back((to, at as u16));
            };
        let layer = layer as usize;
        if layer + 1 < LAYERS {
            give(&mut light, &mut queue, column, layer + 1);
        }
        if layer > 0 {
            give(&mut light, &mut queue, column, layer - 1);
        }
        for side in 0..6 {
            let Some(&neighbor) = region
                .neighbors
                .get(column as usize)
                .and_then(|sides| sides.get(side))
            else {
                continue;
            };
            if neighbor != OFF_REGION && (neighbor as usize) < region.columns.len() {
                give(&mut light, &mut queue, neighbor, layer);
            }
        }
    }
    light
}

/// What one corner of a face is lit to, in 0..1.
///
/// `cells` are the columns that MEET at this corner - the face's own and the
/// two that share it - with [`OFF_REGION`] for any that is not in the region.
/// `layer` is the AIR layer the face opens onto, which is the one above a top
/// cap, below a bottom cap, and beside a flank. Sampling the solid cell's own
/// layer instead is the reference's recorded bug: it made a sealed room's
/// ceiling read pitch black.
///
/// Two steps, both the reference's:
///
/// 1. **Smooth**: the mean level over the cells there that are NOT solid. A
///    solid cell holds no light and is not counted, so a corner against a wall
///    averages what air there is rather than being dragged to zero by rock.
/// 2. **Contact**: multiply by [`CONTACT`] indexed on how many of the two SIDE
///    neighbours are solid at that layer. The face's own cell is not counted,
///    because it says nothing: a cap's is always air and a flank's always rock.
///
/// With no air at all to sample the corner is dark, which the caller may want
/// to override: a wall standing on flat ground has every neighbour solid at
/// its foot, and the reference copies the corner above rather than drawing a
/// black line along every floor.
pub fn corner(region: &Region, light: &Baked, cells: [u32; 3], layer: usize) -> f32 {
    let mut sum = 0u32;
    let mut samples = 0u32;
    for &cell in &cells {
        if cell == OFF_REGION || region.opaque(cell, layer) {
            continue;
        }
        let Some(levels) = light.get(cell as usize) else {
            continue;
        };
        sum += levels[layer] as u32;
        samples += 1;
    }
    if samples == 0 {
        return 0.0;
    }
    let mean = sum as f32 / (samples * MAX as u32) as f32;
    let occluders = cells[1..]
        .iter()
        .filter(|&&cell| cell != OFF_REGION && region.opaque(cell, layer))
        .count();
    (mean * CONTACT[occluders.min(CONTACT.len() - 1)]).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::Material;

    /// Solid to `top` inclusive, air above.
    fn ground(top: usize) -> Column {
        let mut column = Column::bedrock();
        for layer in 1..=top {
            column.set(layer, Material::Stone);
        }
        column
    }

    /// Solid to `top`, then a one-layer gap at `gap`, then solid again to
    /// `roof`: a tunnel with a lid on it.
    fn roofed(top: usize, gap: usize, roof: usize) -> Column {
        let mut column = ground(top);
        for layer in gap + 1..=roof {
            column.set(layer, Material::Stone);
        }
        column
    }

    /// A line of columns, each joined to the next on side 0 and the previous
    /// on side 1: the simplest region a flood can cross.
    fn line(columns: Vec<Column>) -> (Vec<Column>, Vec<[u32; 6]>) {
        let last = columns.len() - 1;
        let neighbors = (0..columns.len())
            .map(|i| {
                let mut sides = [OFF_REGION; 6];
                if i < last {
                    sides[0] = i as u32 + 1;
                }
                if i > 0 {
                    sides[1] = i as u32 - 1;
                }
                sides
            })
            .collect();
        (columns, neighbors)
    }

    fn baked(columns: Vec<Column>) -> (Vec<Column>, Vec<[u32; 6]>, Baked) {
        let (columns, neighbors) = line(columns);
        let light = bake(&Region {
            columns: &columns,
            neighbors: &neighbors,
        });
        (columns, neighbors, light)
    }

    #[test]
    fn open_ground_is_full_daylight_and_the_rock_under_it_is_not_lit() {
        let (_, _, light) = baked(vec![ground(100)]);
        assert_eq!(light[0][101], MAX, "the air over the ground");
        assert_eq!(light[0][LAYERS - 1], MAX, "and all the way up");
        assert_eq!(light[0][100], 0, "the rock itself holds no light");
        assert_eq!(light[0][50], 0, "nor anything under it");
    }

    /// The point of the field. A tunnel under rock is dark and its mouth is
    /// not, and what makes the difference is the number of steps rather than
    /// the depth: both ends of this tunnel are at the same altitude.
    #[test]
    fn a_tunnel_darkens_with_distance_from_its_mouth_not_with_depth() {
        let mut columns = vec![ground(50)];
        for _ in 0..6 {
            columns.push(roofed(50, 51, 100));
        }
        let (_, _, light) = baked(columns);
        assert_eq!(light[0][51], MAX, "the mouth is open to the sky");
        assert_eq!(light[1][51], MAX - 1, "one step in costs one level");
        assert_eq!(light[2][51], MAX - 2);
        assert_eq!(light[6][51], MAX - 6, "and it keeps falling off");
    }

    /// Past the range it is BLACK rather than dim, which is what makes a deep
    /// cave a place you need a light to be in.
    #[test]
    fn a_tunnel_longer_than_the_range_goes_out() {
        let mut columns = vec![ground(50)];
        for _ in 0..(MAX as usize + 4) {
            columns.push(roofed(50, 51, 100));
        }
        let (_, _, light) = baked(columns);
        assert_eq!(light.last().unwrap()[51], 0, "past the range, dark");
    }

    /// The bug the reference's own seed rule avoids, pinned. A column open to
    /// the sky lights its neighbour at EVERY layer, so a cave beside one is
    /// lit by the open cell at its own height. Seeding only the lowest open
    /// cell leaves this at zero.
    #[test]
    fn an_open_column_lights_a_cave_beside_it_at_the_caves_own_height() {
        // Open to bedrock beside a column roofed over a gap at layer 60.
        let open = Column::bedrock();
        let cave = roofed(59, 60, 100);
        let (_, _, light) = baked(vec![open, cave]);
        assert_eq!(light[0][60], MAX, "the open column at that height");
        assert_eq!(light[1][60], MAX - 1, "and the cave beside it");
    }

    /// Off the region is solid, never open: a tier boundary must not become
    /// its own light source.
    #[test]
    fn light_does_not_leak_in_from_off_the_region() {
        let (_, _, light) = baked(vec![roofed(50, 51, 100)]);
        assert_eq!(light[0][51], 0, "roofed and joined to nothing");
        assert_eq!(light[0][101], MAX, "and open above the roof");
    }

    /// The contact ladder, which is the crease where one block sits on
    /// another. Counted over the two SIDE neighbours only.
    #[test]
    fn a_corner_darkens_by_how_much_stone_it_is_wedged_between() {
        let (columns, neighbors, light) = baked(vec![ground(50), ground(50), ground(60)]);
        let region = Region {
            columns: &columns,
            neighbors: &neighbors,
        };
        // Layer 51 is air over columns 0 and 1 and rock inside column 2.
        let open = corner(&region, &light, [0, 1, OFF_REGION], 51);
        assert!((open - 1.0).abs() < 1e-6, "nothing beside it: full light");
        let against_one = corner(&region, &light, [0, 1, 2], 51);
        assert!(
            (against_one - CONTACT[1]).abs() < 1e-6,
            "one solid neighbour: {against_one}"
        );
        let wedged = corner(&region, &light, [0, 2, 2], 51);
        assert!(
            (wedged - CONTACT[2]).abs() < 1e-6,
            "wedged between two: {wedged}"
        );
        assert!(
            against_one < open && wedged < against_one,
            "and it is a ladder"
        );
    }

    /// A corner with no air to sample answers dark, and says so plainly, so
    /// the caller can do what the reference does and take the corner above
    /// instead of drawing a black line along every floor.
    #[test]
    fn a_corner_with_nothing_but_rock_around_it_is_dark() {
        let (columns, neighbors, light) = baked(vec![ground(100)]);
        let region = Region {
            columns: &columns,
            neighbors: &neighbors,
        };
        assert_eq!(
            corner(&region, &light, [0, OFF_REGION, OFF_REGION], 50),
            0.0
        );
    }
}
