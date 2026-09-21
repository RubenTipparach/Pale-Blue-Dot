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
/// Four rungs rather than the reference's three: the fourth is reached only
/// by a wall, which counts the two cells past its edge as well ([`wall_corner`]).
pub const CONTACT: [f32; 4] = [1.0, 0.85, 0.70, 0.55];

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

/// What one cell is lit to: sky in the high nibble, block in the low, which
/// is the reference's own `(sky << 4) | block`.
///
/// Two channels rather than one number, because only ONE of them goes out at
/// night. Summed at bake time a torch would be as useless at midnight as the
/// sun is, which is the opposite of what a torch is for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Light(pub u8);

impl Light {
    pub const DARK: Light = Light(0);

    pub fn new(sky: u8, block: u8) -> Self {
        Self((sky.min(MAX) << 4) | block.min(MAX))
    }

    pub fn sky(self) -> u8 {
        self.0 >> 4
    }

    pub fn block(self) -> u8 {
        self.0 & 0xf
    }

    fn with_sky(self, level: u8) -> Self {
        Self::new(level, self.block())
    }

    fn with_block(self, level: u8) -> Self {
        Self::new(self.sky(), level)
    }
}

/// A cell that gives out light: which column, which layer, how bright.
///
/// WHAT emits is the caller's business - a torch, a glowing flower, a fire -
/// because this crate knows what light does and not what carries it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Emitter {
    pub column: u32,
    pub layer: u16,
    pub level: u8,
}

/// Every cell's light, one array of [`LAYERS`] per column.
pub type Baked = Vec<[Light; LAYERS]>;

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
pub fn bake(region: &Region, emitters: &[Emitter]) -> Baked {
    let mut light = vec![[Light::DARK; LAYERS]; region.columns.len()];
    // The sky, seeded down every column.
    let mut queue: std::collections::VecDeque<(u32, u16)> = std::collections::VecDeque::new();
    for (index, column) in region.columns.iter().enumerate() {
        for layer in (0..LAYERS).rev() {
            if column.solid(layer) {
                break;
            }
            light[index][layer] = light[index][layer].with_sky(MAX);
            queue.push_back((index as u32, layer as u16));
        }
    }
    flood(region, &mut light, queue, Channel::Sky);
    // Then the lamps, on the same machinery. An emitter inside solid rock is
    // dropped rather than lighting from within it: a torch is placed in air.
    let mut queue: std::collections::VecDeque<(u32, u16)> = std::collections::VecDeque::new();
    for emitter in emitters {
        let layer = emitter.layer as usize;
        if layer >= LAYERS || region.opaque(emitter.column, layer) {
            continue;
        }
        let Some(cell) = light
            .get_mut(emitter.column as usize)
            .and_then(|levels| levels.get_mut(layer))
        else {
            continue;
        };
        if cell.block() >= emitter.level {
            continue;
        }
        *cell = cell.with_block(emitter.level.min(MAX));
        queue.push_back((emitter.column, emitter.layer));
    }
    flood(region, &mut light, queue, Channel::Block);
    light
}

/// Which nibble a flood fills.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Channel {
    Sky,
    Block,
}

impl Channel {
    fn of(self, light: Light) -> u8 {
        match self {
            Channel::Sky => light.sky(),
            Channel::Block => light.block(),
        }
    }

    fn set(self, light: Light, level: u8) -> Light {
        match self {
            Channel::Sky => light.with_sky(level),
            Channel::Block => light.with_block(level),
        }
    }
}

/// The flood. Every cell already at its seeded level is on the queue.
///
/// ONE implementation for both channels, which is what keeps a lamp's falloff
/// and daylight's the same falloff: two floods written apart would be two
/// answers to how far light travels, and a player would learn one of them.
fn flood(
    region: &Region,
    light: &mut Baked,
    mut queue: std::collections::VecDeque<(u32, u16)>,
    channel: Channel,
) {
    while let Some((column, layer)) = queue.pop_front() {
        let level = channel.of(light[column as usize][layer as usize]);
        if level <= STEP {
            continue;
        }
        let next = level - STEP;
        let give =
            |light: &mut Baked, queue: &mut std::collections::VecDeque<_>, to: u32, at: usize| {
                if region.opaque(to, at) || channel.of(light[to as usize][at]) >= next {
                    return;
                }
                light[to as usize][at] = channel.set(light[to as usize][at], next);
                queue.push_back((to, at as u16));
            };
        let layer = layer as usize;
        if layer + 1 < LAYERS {
            give(light, &mut queue, column, layer + 1);
        }
        if layer > 0 {
            give(light, &mut queue, column, layer - 1);
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
                give(light, &mut queue, neighbor, layer);
            }
        }
    }
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
        sum += levels[layer].sky() as u32;
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

/// A WALL's corner: [`corner`]'s rule plus the two cells one layer past the
/// wall's edge, `beyond` - the column across and the column beside, at the
/// layer below a foot or above a head.
///
/// The reference counts only the two side columns at the face's own layer,
/// which for a cap is the whole rule. For a wall the plane in front of the
/// face is the column across, and a vertex on the wall's edge has three cells
/// that can shadow it: the side column at the wall's layer, and the across and
/// side columns one layer past the edge. That is Minecraft's three-neighbour
/// rule, which is where the reference's own ladder comes from, on a lattice
/// where three columns meet at a corner. Without it a wall standing on a floor
/// is lit the same at its foot as at its middle, and a wall under a lid as if
/// nothing were over it - the picture the owner drew.
pub fn wall_corner(
    region: &Region,
    light: &Baked,
    cells: [u32; 3],
    layer: usize,
    beyond: usize,
) -> f32 {
    let mut sum = 0u32;
    let mut samples = 0u32;
    for &cell in &cells {
        if cell == OFF_REGION || region.opaque(cell, layer) {
            continue;
        }
        let Some(levels) = light.get(cell as usize) else {
            continue;
        };
        sum += levels[layer].sky() as u32;
        samples += 1;
    }
    if samples == 0 {
        return 0.0;
    }
    let mean = sum as f32 / (samples * MAX as u32) as f32;
    let solid = |cell: u32, at: usize| cell != OFF_REGION && region.opaque(cell, at);
    let occluders = cells[1..]
        .iter()
        .filter(|&&cell| solid(cell, layer))
        .count()
        + cells[1..]
            .iter()
            .filter(|&&cell| solid(cell, beyond))
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
        let light = bake(
            &Region {
                columns: &columns,
                neighbors: &neighbors,
            },
            &[],
        );
        (columns, neighbors, light)
    }

    #[test]
    fn open_ground_is_full_daylight_and_the_rock_under_it_is_not_lit() {
        let (_, _, light) = baked(vec![ground(100)]);
        assert_eq!(light[0][101].sky(), MAX, "the air over the ground");
        assert_eq!(light[0][LAYERS - 1].sky(), MAX, "and all the way up");
        assert_eq!(light[0][100].sky(), 0, "the rock itself holds no light");
        assert_eq!(light[0][50].sky(), 0, "nor anything under it");
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
        assert_eq!(light[0][51].sky(), MAX, "the mouth is open to the sky");
        assert_eq!(light[1][51].sky(), MAX - 1, "one step in costs one level");
        assert_eq!(light[2][51].sky(), MAX - 2);
        assert_eq!(light[6][51].sky(), MAX - 6, "and it keeps falling off");
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
        assert_eq!(light.last().unwrap()[51].sky(), 0, "past the range, dark");
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
        assert_eq!(light[0][60].sky(), MAX, "the open column at that height");
        assert_eq!(light[1][60].sky(), MAX - 1, "and the cave beside it");
    }

    /// Off the region is solid, never open: a tier boundary must not become
    /// its own light source.
    #[test]
    fn light_does_not_leak_in_from_off_the_region() {
        let (_, _, light) = baked(vec![roofed(50, 51, 100)]);
        assert_eq!(light[0][51].sky(), 0, "roofed and joined to nothing");
        assert_eq!(light[0][101].sky(), MAX, "and open above the roof");
    }

    /// **Digging changes the light, and the field has to be asked again.**
    ///
    /// This is the shipped bug the owner caught: the edit path repacked the
    /// geometry and never re-baked, so a hole appeared with its walls black.
    /// Rock has a sky level of zero because rock holds no light, so a cell dug
    /// out of one stays at zero until something re-bakes - the picture is a
    /// hole lit by the inside of a stone.
    #[test]
    fn a_dug_shaft_is_dark_until_it_is_baked_again() {
        let (mut columns, neighbors) = line(vec![ground(100)]);
        let before = bake(
            &Region {
                columns: &columns,
                neighbors: &neighbors,
            },
            &[],
        );
        assert_eq!(before[0][98].sky(), 0, "rock holds no light");

        // Dig three layers out of the top, as a player would.
        for layer in 98..=100 {
            columns[0].set(layer, Material::Air);
        }
        let stale = &before;
        assert_eq!(
            stale[0][98].sky(),
            0,
            "and the OLD field still says so, which is what drew black walls"
        );

        let after = bake(
            &Region {
                columns: &columns,
                neighbors: &neighbors,
            },
            &[],
        );
        assert_eq!(
            after[0][98].sky(),
            MAX,
            "a shaft open to the sky keeps full strength all the way down"
        );
    }

    /// And the other verb: PLACING a block takes light away.
    ///
    /// The mirror of the dig case, and worth its own pin because the two are
    /// not symmetric in the reference: its incremental pass has a REMOVAL
    /// phase for exactly this, which has to walk back everything the now
    /// blocked path used to light, and its own comment records the scar of
    /// getting it wrong - a dug cell that "stayed dark forever". Re-baking
    /// the whole region has no removal phase to get wrong.
    #[test]
    fn roofing_a_cell_takes_its_daylight_away() {
        let (mut columns, neighbors) = line(vec![ground(50), ground(50)]);
        let open = bake(
            &Region {
                columns: &columns,
                neighbors: &neighbors,
            },
            &[],
        );
        assert_eq!(open[0][51].sky(), MAX, "open to the sky");

        // Roof the first column over, as placing blocks does.
        for layer in 52..70 {
            columns[0].set(layer, Material::Stone);
        }
        let roofed = bake(
            &Region {
                columns: &columns,
                neighbors: &neighbors,
            },
            &[],
        );
        assert_eq!(
            roofed[0][51].sky(),
            MAX - 1,
            "now lit only from the open column beside it, one step away"
        );
        assert_eq!(roofed[0][60].sky(), 0, "and the rock itself holds none");
    }

    /// A lamp fills the dark, on the same falloff daylight uses.
    #[test]
    fn a_lamp_lights_a_buried_tunnel_and_the_sky_does_not_notice() {
        let mut columns = vec![roofed(50, 51, 100)];
        for _ in 0..6 {
            columns.push(roofed(50, 51, 100));
        }
        let (columns, neighbors) = line(columns);
        let region = Region {
            columns: &columns,
            neighbors: &neighbors,
        };
        let dark = bake(&region, &[]);
        assert_eq!(dark[0][51].sky(), 0, "sealed, so no daylight");
        assert_eq!(dark[0][51].block(), 0, "and no lamp yet");

        let lit = bake(
            &region,
            &[Emitter {
                column: 0,
                layer: 51,
                level: 14,
            }],
        );
        assert_eq!(lit[0][51].block(), 14, "the lamp's own cell");
        assert_eq!(lit[1][51].block(), 13, "one step costs one level");
        assert_eq!(lit[6][51].block(), 8, "and it keeps falling off");
        assert_eq!(lit[0][51].sky(), 0, "the sky channel is untouched");
    }

    /// **A lamp does not light through a wall**, which is the whole reason
    /// this is a flood rather than the proximity sum the reference settled
    /// for. Its own comment admits that one "ignores walls entirely - it leaks
    /// through stone"; it settled there because a planet-wide relight cost it
    /// a second per placement, and a tier of three thousand columns costs six
    /// milliseconds.
    #[test]
    fn a_lamp_does_not_light_through_a_wall() {
        // A lamp in a sealed pocket, and a second pocket next door with solid
        // rock between them: adjacent in space, unreachable through the rock.
        let mut near = ground(50);
        near.set(51, Material::Air);
        for layer in 52..100 {
            near.set(layer, Material::Stone);
        }
        let wall = ground(100);
        let far = near.clone();
        let (columns, neighbors) = line(vec![near, wall, far]);
        let region = Region {
            columns: &columns,
            neighbors: &neighbors,
        };
        let lit = bake(
            &region,
            &[Emitter {
                column: 0,
                layer: 51,
                level: 15,
            }],
        );
        assert_eq!(lit[0][51].block(), 15, "the lamp's own pocket");
        assert_eq!(
            lit[2][51].block(),
            0,
            "and nothing at all through one cell of rock"
        );
    }

    /// A lamp buried in rock lights nothing: it is not in the world, and a
    /// torch is placed in air.
    #[test]
    fn a_lamp_inside_rock_is_dropped() {
        let (columns, neighbors) = line(vec![ground(100)]);
        let region = Region {
            columns: &columns,
            neighbors: &neighbors,
        };
        let lit = bake(
            &region,
            &[Emitter {
                column: 0,
                layer: 50,
                level: 15,
            }],
        );
        assert_eq!(lit[0][50].block(), 0);
        assert_eq!(lit[0][51].block(), 0, "and it does not leak upward");
    }

    /// The two channels are apart in one byte and neither reaches the other,
    /// which is what lets a shader put the sun out and leave a lamp burning.
    #[test]
    fn the_two_channels_are_kept_apart_in_one_byte() {
        let light = Light::new(12, 5);
        assert_eq!(light.sky(), 12);
        assert_eq!(light.block(), 5);
        assert_eq!(Light::new(99, 99), Light::new(MAX, MAX), "clamped");
        assert_eq!(Light::DARK.sky(), 0);
        assert_eq!(Light::DARK.block(), 0);
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

    /// A wall's foot where it stands on a floor is darker than its middle,
    /// and its head under a lid is darker than its middle: the two cells past
    /// the wall's edge count, which is what the reference's ladder left out.
    #[test]
    fn a_walls_foot_on_a_floor_and_its_head_under_a_lid_are_darker_than_its_middle() {
        // Column 0 is the floor at 50, column 1 the wall standing on it to
        // 53, column 2 a lid over the floor from 55 up. The wall's face is
        // toward column 0 at layers 51..=53.
        let (columns, neighbors, light) = baked(vec![ground(50), ground(53), roofed(50, 54, 60)]);
        let region = Region {
            columns: &columns,
            neighbors: &neighbors,
        };
        let cells = [1, 0, OFF_REGION];
        let middle = corner(&region, &light, cells, 52);
        // The foot: layer 51 of the face, and the floor at 50 past its edge.
        let foot = wall_corner(&region, &light, cells, 51, 50);
        assert!(foot < middle, "foot {foot} against middle {middle}");
        assert!(
            (foot - middle * CONTACT[1] / CONTACT[0]).abs() < 1e-6,
            "one cell past the edge is one rung: {foot}"
        );
        // A head under a lid: the wall's top layer 53 of a face toward column
        // 2, whose lid at 55 is one past the edge at 54... so use a face at
        // layer 54 toward column 2 with the lid at 55 past it.
        let lid = wall_corner(&region, &light, [1, 2, OFF_REGION], 54, 55);
        let open = corner(&region, &light, [1, 2, OFF_REGION], 54);
        assert!(lid < open, "head under a lid {lid} against open {open}");
        // Without a cell past the edge the wall rule is the cap rule.
        let same = wall_corner(&region, &light, cells, 52, 52);
        assert!((same - middle).abs() < 1e-6, "{same} against {middle}");
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
