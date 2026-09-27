//! Voxel light over a region of columns: what a cell can see of the sky, and
//! how dark a corner goes where it is wedged into stone.
//!
//! Ported from Tenebris's `world_light.rs` and the corner half of its
//! `hex_mesher.rs`, both read rather than guessed at. The two halves are what
//! the owner named - "voxel lighting algorithm combined with baked vertex
//! colors" - and they do different jobs:
//!
//! - **[`bake`] is the FIELD.** Daylight is seeded down every column and
//!   flooded outward, a level a metre (one a layer, three a cell), stopping
//!   at anything solid. A cave is dark because nothing reached it, not
//!   because a depth term was subtracted, which is why a cave MOUTH stays
//!   bright.
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

/// What a step UP or DOWN, to the layer above or below, costs, in levels.
///
/// The reference's vertical step, shipped at 1 - the comment beside it says
/// "Vertical neighbours stay at 1 per step so daylight shafts remain at full
/// strength". A layer is a metre, so a level is a metre upward.
pub const STEP: u8 = 1;

/// What a step ACROSS, to a neighbouring column, costs, in levels.
///
/// Three levels: one cell's 2.833 m, rounded, so a level is about a metre
/// whichever way light goes, for the sky and for a lamp alike. At one level
/// per cell (the reference's `voxel_sky_lateral_loss`) a brazier lit forty
/// metres of meadow at night nearly as bright as noon, where the owner's
/// towns mockup gives it eleven (`lamps-and-lanterns` decision 7), and
/// twilight reached forty metres into a tunnel. The owner chose one rule for
/// all light (survey L1, decision 11): a tunnel's mouth now lights four cells
/// in, about eleven metres.
pub const ACROSS: u8 = 3;

/// The colour a lamp's light lays over what it lights, and how strongly, in
/// the terrain shader's `albedo * TORCH_TINT * strength * TORCH_GAIN`. Warm,
/// because everything that burns is. Here so the held tool and the ship,
/// which the terrain pass does not draw, are lit by the same numbers
/// (`lamps-and-lanterns` decisions 5 and 10); a test holds the shader's copy
/// to these.
pub const TORCH_TINT: [f32; 3] = [1.00, 0.70, 0.30];
pub const TORCH_GAIN: f32 = 1.25;

/// The floor under the sky's fill: what a surface the sun and sky never reach
/// still shows, so a cave is a dark room and not a black screen. The terrain
/// shader's `AMBIENT_FLOOR`.
pub const AMBIENT_FLOOR: f32 = 0.05;

/// The sky's fill at night, as a share of the day's: the terrain shader's
/// `night` for a cap.
pub const NIGHT_FILL: f32 = 0.12;

/// How much of the day's light a surface takes from the sky, given what of
/// the sky reaches it (`sky`, 0..1) and how much it is day where it is
/// (`daylight`, 0..1): the terrain shader's
/// `max(AMBIENT_FLOOR, mix(night, 1, daylight) * skylight)`. What moves is lit
/// by it (`lamps-and-lanterns` decision 10), so a tool in a cave is as dark as
/// the cave.
pub fn sky_fill(sky: f32, daylight: f32) -> f32 {
    let day = daylight.clamp(0.0, 1.0);
    ((NIGHT_FILL + (1.0 - NIGHT_FILL) * day) * sky.clamp(0.0, 1.0)).max(AMBIENT_FLOOR)
}

/// What a block level (0..1) adds, as linear RGB over a white surface: the
/// terrain shader's `TORCH_TINT * lamp_strength(lamp) * TORCH_GAIN`.
pub fn lamp_light(block: f32) -> [f32; 3] {
    let k = lamp_strength(block) * TORCH_GAIN;
    [TORCH_TINT[0] * k, TORCH_TINT[1] * k, TORCH_TINT[2] * k]
}

/// How much of the lamp colour a block level adds, from its share `f` of
/// [`MAX`] (`lamps-and-lanterns` decision 7): the towns mockup's
/// `(1 - (d/R)^2)^2` written in the level. The shader's `lamp_strength` is the
/// same arithmetic, and a test holds its text to this.
pub fn lamp_strength(f: f32) -> f32 {
    let f = f.clamp(0.0, 1.0);
    let g = f * (2.0 - f);
    g * g
}

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
/// 2. **Flood**, breadth first: pop a cell, hand `level - STEP` to the cells
///    above and below it and `level - ACROSS` to the columns beside it, where
///    they are not solid and not already at least that bright.
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
/// ONE implementation for both channels: two floods written apart would be
/// two answers to how light travels, and a player would learn one of them.
/// They differ only in which nibble they fill and where they are seeded.
/// With a step up ([`STEP`]) cheaper than a step across ([`ACROSS`]), a cell can
/// be reached first by a dimmer path and later by a brighter one; `give`
/// takes the brighter and queues the cell again, so the answer is still the
/// brightest path, whatever order the queue ran in.
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
        let give = |light: &mut Baked,
                    queue: &mut std::collections::VecDeque<_>,
                    to: u32,
                    at: usize,
                    next: u8| {
            if region.opaque(to, at) || channel.of(light[to as usize][at]) >= next {
                return;
            }
            light[to as usize][at] = channel.set(light[to as usize][at], next);
            queue.push_back((to, at as u16));
        };
        let layer = layer as usize;
        if layer + 1 < LAYERS {
            give(light, &mut queue, column, layer + 1, level - STEP);
        }
        if layer > 0 {
            give(light, &mut queue, column, layer - 1, level - STEP);
        }
        if level <= ACROSS {
            continue;
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
                give(light, &mut queue, neighbor, layer, level - ACROSS);
            }
        }
    }
}

/// Both channels at a point, each in 0..1: `(sky, block)`. What lights
/// anything the terrain pass does not draw - the held tool, the ship, a fish
/// (`lamps-and-lanterns` decision 10).
///
/// `column` is the column the point is in, `direction` the point's unit
/// direction from the planet's centre, `layer` its height in layers (a whole
/// number is a layer's floor), and `centres` every column's unit direction.
///
/// It blends two ways, so a thing moving through the field changes smoothly:
///
/// - **Across**, over the column and its neighbours, each weighted by how
///   near the point is to its centre, out to one neighbour's spacing. A
///   neighbour solid at that layer is left out: a point beside a wall reads
///   the air it is in, not the rock.
/// - **Up**, between the two layers the point sits between, taking each at
///   its middle.
///
/// With no column, or with the point off the column's span, it answers the
/// open sky and no lamp, which is what the far terrain is drawn with.
pub fn sample(
    region: &Region,
    light: &Baked,
    centres: &[glam::Vec3],
    column: Option<u32>,
    direction: glam::Vec3,
    layer: f32,
) -> (f32, f32) {
    const OPEN: (f32, f32) = (1.0, 0.0);
    let Some(column) = column else {
        return OPEN;
    };
    let (Some(centre), Some(_)) = (centres.get(column as usize), light.get(column as usize)) else {
        return OPEN;
    };
    if !layer.is_finite() || layer < 0.0 || layer >= LAYERS as f32 {
        return OPEN;
    }
    // The two layers the point sits between, each taken at its middle.
    let below = (layer - 0.5).floor().clamp(0.0, (LAYERS - 1) as f32);
    let above = (below + 1.0).min((LAYERS - 1) as f32);
    let up = ((layer - 0.5) - below).clamp(0.0, 1.0);
    let at = |cell: u32, level: usize| -> Option<(f32, f32)> {
        if region.opaque(cell, level) {
            return None;
        }
        let value = light.get(cell as usize)?.get(level)?;
        Some((value.sky() as f32, value.block() as f32))
    };
    let angle = |a: glam::Vec3, b: glam::Vec3| a.dot(b).clamp(-1.0, 1.0).acos();
    let neighbours: Vec<u32> = region
        .neighbors
        .get(column as usize)
        .map(|sides| {
            sides
                .iter()
                .copied()
                .filter(|&n| n != OFF_REGION && (n as usize) < centres.len())
                .collect()
        })
        .unwrap_or_default();
    // One neighbour's spacing: how far a centre's weight reaches.
    let spacing = neighbours
        .iter()
        .map(|&n| angle(*centre, centres[n as usize]))
        .fold(0.0_f32, f32::max)
        .max(1e-9);
    let weigh = |cell: u32| -> f32 {
        let d = angle(direction, centres[cell as usize]) / spacing;
        let w = (1.0 - d).max(0.0);
        w * w
    };
    let mut sum = (0.0_f32, 0.0_f32);
    let mut total = 0.0_f32;
    for cell in std::iter::once(column).chain(neighbours.iter().copied()) {
        // The home column always counts a little, so a point on its far
        // edge still reads its own cell rather than nothing.
        let w = if cell == column {
            weigh(cell).max(1e-3)
        } else {
            weigh(cell)
        };
        if w <= 0.0 {
            continue;
        }
        let low = at(cell, below as usize);
        let high = at(cell, above as usize);
        let value = match (low, high) {
            (Some(a), Some(b)) => (a.0 + (b.0 - a.0) * up, a.1 + (b.1 - a.1) * up),
            (Some(a), None) | (None, Some(a)) => a,
            (None, None) => continue,
        };
        sum.0 += value.0 * w;
        sum.1 += value.1 * w;
        total += w;
    }
    if total <= 0.0 {
        // Inside rock: nothing reaches it.
        return (0.0, 0.0);
    }
    let max = MAX as f32;
    (
        (sum.0 / total / max).clamp(0.0, 1.0),
        (sum.1 / total / max).clamp(0.0, 1.0),
    )
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
        // Three levels a cell, a metre a level (decision 11, survey L1).
        assert_eq!(light[1][51].sky(), 12, "one cell in costs three levels");
        assert_eq!(light[2][51].sky(), 9);
        assert_eq!(light[3][51].sky(), 6);
        assert_eq!(light[4][51].sky(), 3, "and it keeps falling off");
        assert_eq!(light[5][51].sky(), 0, "until, 14 m in, it is dark");
        assert_eq!(light[6][51].sky(), 0);
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
        assert_eq!(
            light[1][60].sky(),
            MAX - ACROSS,
            "and the cave beside it, one cell across"
        );
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
            MAX - ACROSS,
            "now lit only from the open column beside it, one cell away"
        );
        assert_eq!(roofed[0][60].sky(), 0, "and the rock itself holds none");
    }

    /// A lamp fills the dark: three levels a cell across, one a layer up.
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
        assert_eq!(
            lit[1][51].block(),
            14 - ACROSS,
            "a step across costs a cell's width, three levels"
        );
        assert_eq!(lit[4][51].block(), 2, "and it keeps falling off");
        assert_eq!(lit[5][51].block(), 0, "out before the fifth cell");
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

    /// Every light the city needs, at the level the design gave it, and none
    /// of them a wall (`lamps-and-lanterns` task 5.1).
    #[test]
    fn each_light_has_its_level_and_none_is_solid() {
        let levels = [
            (Material::Torch, 14),
            (Material::LanternPost, 13),
            (Material::LanternWall, 13),
            (Material::LanternHanging, 12),
            (Material::Brazier, 15),
            (Material::Candle, 8),
        ];
        assert_eq!(levels.len(), Material::LAMPS.len());
        for (material, level) in levels {
            assert_eq!(material.emission(), level, "{material:?}");
            assert!(material.is_lamp());
            let mut column = ground(40);
            column.set(41, material);
            assert!(!column.solid(41), "{material:?} is walked through");
            assert_eq!(column.surface(), Some(40), "and is not the ground");
        }
        assert!(Material::LanternPost.dusk_lit() && Material::LanternWall.dusk_lit());
        for always in [
            Material::Torch,
            Material::LanternHanging,
            Material::Brazier,
            Material::Candle,
        ] {
            assert!(!always.dusk_lit(), "{always:?} burns all day");
        }
        assert!(!Material::Stone.is_lamp() && !Material::Air.is_lamp());
    }

    /// A candle lights a room and a brazier a square: alone in the same dark
    /// tunnel, the brazier's light reaches further. Across, each lights about
    /// a metre a level, which is the towns mockup's reach for each
    /// (`lamps-and-lanterns` decision 7).
    #[test]
    fn a_brazier_reaches_further_than_a_candle() {
        let reach = |material: Material| {
            let columns: Vec<Column> = (0..24).map(|_| roofed(50, 51, 100)).collect();
            let (columns, neighbors) = line(columns);
            let light = bake(
                &Region {
                    columns: &columns,
                    neighbors: &neighbors,
                },
                &[Emitter {
                    column: 0,
                    layer: 51,
                    level: material.emission(),
                }],
            );
            (0..columns.len())
                .filter(|&c| light[c][51].block() > 0)
                .count()
        };
        let (candle, brazier) = (reach(Material::Candle), reach(Material::Brazier));
        assert_eq!(candle, 3, "a candle of 8 lights its own cell and two more");
        assert_eq!(brazier, 5, "a brazier of 15, its own and four more");
        assert!(brazier > candle);
        for (material, cells) in [
            (Material::Torch, 5),
            (Material::LanternPost, 5),
            (Material::LanternWall, 5),
            (Material::LanternHanging, 4),
        ] {
            assert_eq!(reach(material), cells, "{material:?}");
        }
    }

    /// A tier-sized patch of hex ground, `side` by `side` columns in offset
    /// rows, flat at layer `top`: the shape the column tier bakes, without
    /// the planet under it.
    fn field(side: usize, top: usize) -> (Vec<Column>, Vec<[u32; 6]>) {
        let columns = vec![ground(top); side * side];
        let at = |c: isize, r: isize| {
            if c < 0 || r < 0 || c >= side as isize || r >= side as isize {
                OFF_REGION
            } else {
                (r as usize * side + c as usize) as u32
            }
        };
        let neighbors = (0..side * side)
            .map(|i| {
                let (c, r) = ((i % side) as isize, (i / side) as isize);
                let shift = r & 1;
                [
                    at(c + 1, r),
                    at(c - 1, r),
                    at(c - 1 + shift, r - 1),
                    at(c + shift, r - 1),
                    at(c - 1 + shift, r + 1),
                    at(c + shift, r + 1),
                ]
            })
            .collect();
        (columns, neighbors)
    }

    /// Centres for a `line`: one cell's 2.833 m apart along a great circle
    /// of a 4.8 km planet, which is the finest cells' spacing.
    fn centres(count: usize) -> Vec<glam::Vec3> {
        let step = 2.833 / 4800.0;
        (0..count)
            .map(|i| glam::Vec3::new((i as f32 * step).cos(), (i as f32 * step).sin(), 0.0))
            .collect()
    }

    /// The sampler (`lamps-and-lanterns` task 3.1): a point in a sealed cave
    /// reads dark, a point beside a lamp reads the lamp and less further off,
    /// a point with no column or off the span reads the open sky, and a point
    /// in open ground reads the sky.
    #[test]
    fn the_sampler_reads_a_cave_dark_a_lamp_bright_and_off_the_tier_open() {
        let (columns, neighbors) = line((0..8).map(|_| roofed(50, 51, 100)).collect());
        let region = Region {
            columns: &columns,
            neighbors: &neighbors,
        };
        let at = centres(columns.len());
        let dark = bake(&region, &[]);
        let (sky, block) = sample(&region, &dark, &at, Some(3), at[3], 51.5);
        assert_eq!((sky, block), (0.0, 0.0), "a sealed cave, unlit");

        let lit = bake(
            &region,
            &[Emitter {
                column: 0,
                layer: 51,
                level: Material::Torch.emission(),
            }],
        );
        let (_, beside) = sample(&region, &lit, &at, Some(0), at[0], 51.5);
        assert!(
            (beside - 14.0 / 15.0).abs() < 1e-5,
            "in the torch's cell: {beside}"
        );
        let (_, near) = sample(&region, &lit, &at, Some(1), at[1], 51.5);
        let (_, far) = sample(&region, &lit, &at, Some(3), at[3], 51.5);
        assert!(beside > near && near > far, "{beside} > {near} > {far}");
        // Halfway between two centres it is between their two values.
        let half = (at[0] + at[1]).normalize();
        let (_, between) = sample(&region, &lit, &at, Some(0), half, 51.5);
        assert!(
            near < between && between < beside,
            "{near} < {between} < {beside}"
        );

        assert_eq!(
            sample(&region, &lit, &at, None, at[0], 51.5),
            (1.0, 0.0),
            "no column"
        );
        assert_eq!(
            sample(&region, &lit, &at, Some(0), at[0], LAYERS as f32 + 3.0),
            (1.0, 0.0),
            "over the span"
        );

        let (open, neighbors) = line((0..3).map(|_| ground(50)).collect());
        let region = Region {
            columns: &open,
            neighbors: &neighbors,
        };
        let day = bake(&region, &[]);
        let (sky, _) = sample(&region, &day, &centres(3), Some(1), centres(3)[1], 52.0);
        assert_eq!(sky, 1.0, "open ground reads the whole sky");
    }

    /// **A city of three hundred lanterns** (`lamps-and-lanterns` task 5.5),
    /// before any city exists: a street lantern every third cell of every
    /// third row, over a patch the size of the column tier. Each lantern's
    /// own cell holds its level, and ground more than four cells from every
    /// lantern holds none. `cargo test --release -p pbd-core city -- --nocapture`
    /// prints what the bake cost; the design's risk note records it.
    #[test]
    fn a_city_of_three_hundred_lanterns_bakes_every_lantern_and_nothing_past_its_reach() {
        let (columns, neighbors) = field(56, 100);
        let region = Region {
            columns: &columns,
            neighbors: &neighbors,
        };
        let side = 56;
        let lanterns: Vec<Emitter> = (0..side * side)
            .filter(|i| (i % side) % 3 == 1 && (i / side) % 3 == 1)
            .take(300)
            .map(|i| Emitter {
                column: i as u32,
                layer: 101,
                level: Material::LanternPost.emission(),
            })
            .collect();
        assert_eq!(lanterns.len(), 300);
        let started = std::time::Instant::now();
        let lit = bake(&region, &lanterns);
        let with = started.elapsed();
        let started = std::time::Instant::now();
        let dark = bake(&region, &[]);
        let bare = started.elapsed();
        eprintln!(
            "city bake: {} columns, {} lanterns: {:.2} ms, {:.2} ms without them",
            columns.len(),
            lanterns.len(),
            with.as_secs_f64() * 1000.0,
            bare.as_secs_f64() * 1000.0
        );
        for lantern in &lanterns {
            assert_eq!(
                lit[lantern.column as usize][101].block(),
                13,
                "a lantern's own cell"
            );
        }
        // The last lantern stands at column 1 + 3k of row 1 + 3j; the patch's
        // far corner is well over four cells from every one of them.
        let last = lanterns.last().unwrap().column as usize;
        let beyond = (side - 1) * side + (side - 1);
        assert!(
            beyond / side >= last / side + 5,
            "the corner is past the city"
        );
        assert_eq!(
            lit[beyond][101].block(),
            0,
            "nothing past a lantern's reach"
        );
        assert_eq!(dark[beyond][101].block(), 0);
        assert_eq!(lit[beyond][101].sky(), MAX, "and the sky is the sky's");
    }

    /// Up a shaft a lamp's light still falls one level a layer, as the sky's
    /// does: only the step across is dearer. A brazier at the foot of a
    /// sealed shaft lights all fifteen layers of it, and the sky, across a
    /// region with no open column, is never touched.
    #[test]
    fn a_lamp_lights_a_shaft_a_level_a_layer_and_the_sky_steps_as_before() {
        let mut shaft = ground(50);
        for layer in 51..70 {
            shaft.set(layer, Material::Air);
        }
        for layer in 70..100 {
            shaft.set(layer, Material::Stone);
        }
        let (columns, neighbors) = line(vec![shaft]);
        let lit = bake(
            &Region {
                columns: &columns,
                neighbors: &neighbors,
            },
            &[Emitter {
                column: 0,
                layer: 51,
                level: Material::Brazier.emission(),
            }],
        );
        for up in 0..15 {
            assert_eq!(
                lit[0][51 + up].block(),
                15 - up as u8,
                "{up} layers up the shaft"
            );
        }
        assert_eq!(lit[0][66].block(), 0, "fifteen layers up it is out");
        // A level is a metre both ways: a one-metre layer costs one, a
        // 2.833 m cell three.
        const { assert!(STEP == 1 && ACROSS == 3) };
    }
}
