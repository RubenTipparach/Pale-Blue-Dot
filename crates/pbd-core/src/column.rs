//! A cell's material stack: what makes a ceiling possible.
//!
//! The heightfield answers one surface per direction, so it cannot express a
//! cave, an overhang, or a block to remove. A column answers a material per
//! METRE over a fixed radial span, so air between two solid runs is just a
//! column, and every one of those falls out of the representation rather than
//! being a feature built on top of it.
//!
//! Ported in shape from `tenebris-core`'s `blocks[tile * WORLD_MAX_DEPTH +
//! depth]`. What is NOT ported is the carve: the reference fills every layer
//! from a per-column profile and cuts nothing, so every cave in Tenebris is one
//! a player dug. Generated cave systems are this project's own addition, and
//! they are Perlin worms: see `crate::worms`.

use crate::planet_gen::{self, TerrainConfig};
use crate::terrain::Material;
use crate::worms::{WormField, Worms};
use glam::Vec3;

/// The lowest altitude a column describes, metres against sea level. The
/// measured basin floor is -125 m, so this clears it with room for a dug one.
pub const BASE_M: i32 = -145;
/// Layers in a column, one metre each. -145 to +175 covers the measured relief
/// of -125 to +158 m at both ends.
pub const LAYERS: usize = 320;

/// The altitude of the BOTTOM of layer `index`, metres against sea level.
pub fn layer_altitude(index: usize) -> f32 {
    BASE_M as f32 + index as f32
}

/// The layer holding `altitude`, or `None` outside the span.
pub fn layer_at(altitude: f32) -> Option<usize> {
    let index = (altitude - BASE_M as f32).floor();
    (index >= 0.0 && index < LAYERS as f32).then_some(index as usize)
}

/// One cell's stack.
#[derive(Clone)]
pub struct Column {
    layers: [Material; LAYERS],
}

impl Column {
    /// A column of nothing but its bedrock floor, for a test or a caller that
    /// is about to fill it. `set` refuses layer zero, so the floor is here
    /// rather than left to the caller to remember.
    pub fn bedrock() -> Self {
        let mut layers = [Material::Air; LAYERS];
        layers[0] = Material::Stone;
        Self { layers }
    }

    pub fn material(&self, index: usize) -> Material {
        self.layers.get(index).copied().unwrap_or(Material::Air)
    }

    /// Whether a layer stops a player. Air and water do not; water is swum,
    /// and a torch is walked through.
    pub fn solid(&self, index: usize) -> bool {
        !matches!(
            self.material(index),
            Material::Air | Material::Water | Material::Torch
        )
    }

    /// The topmost solid layer, which is the surface the heightfield tiers draw.
    pub fn surface(&self) -> Option<usize> {
        (0..LAYERS).rev().find(|index| self.solid(*index))
    }

    /// Set a layer. Refuses the bedrock floor: without it a player digs through
    /// the bottom of the world and sees the inside of the planet, which is the
    /// one hole that cannot be fixed by drawing more faces.
    pub fn set(&mut self, index: usize, material: Material) -> bool {
        if index == 0 || index >= LAYERS {
            return false;
        }
        self.layers[index] = material;
        true
    }

    /// The solid run a point is in or under, and the one above it: the floor to
    /// stand on and the ceiling to hit your head on. This is the whole of
    /// walking into a cave.
    ///
    /// The floor is the top of the solid RUN, not of the solid layer the point
    /// happens to be in. The first cut answered the layer: a point 0.3 m inside
    /// a two-metre wall was told its floor was one metre up, which is within a
    /// step, and the walker climbed into the middle of the wall. Tenebris's
    /// `walkable_floor_near` insists on a passable cell above a floor for the
    /// same reason.
    pub fn contact(&self, altitude: f32) -> Contact {
        let here = (altitude - BASE_M as f32).floor();
        let start = here.clamp(0.0, LAYERS as f32 - 1.0) as usize;
        // The first solid layer at or below the sample, then up through the run
        // it belongs to.
        let mut floor = None;
        let mut top = start;
        if let Some(first) = (0..=start).rev().find(|index| self.solid(*index)) {
            top = (first..LAYERS)
                .take_while(|index| self.solid(*index))
                .last()
                .unwrap_or(first);
            floor = Some(layer_altitude(top) + 1.0);
        }
        // The ceiling is the bottom of the first solid layer above that run.
        let ceiling = ((top + 1)..LAYERS)
            .find(|index| self.solid(*index))
            .map(layer_altitude);
        // And the water standing on the floor, which is the layer just above
        // the run whether or not there is a ceiling over it.
        let water = floor.and_then(|_| self.water_surface(layer_altitude(top + 1)));
        Contact {
            floor,
            ceiling,
            water,
        }
    }

    /// The altitude of the surface of the water this altitude is IN, or `None`
    /// where it is not in water.
    ///
    /// The surface is the top of the water run: water that reaches sea level
    /// is the sea, and water that stops lower is a pool. Nothing fills a pool
    /// yet, so today every water run in a generated column runs from the
    /// ground to sea level - but the reader asks the run rather than the sea,
    /// because the day flooding lands is the day a pool exists.
    pub fn water_surface(&self, altitude: f32) -> Option<f32> {
        let layer = layer_at(altitude)?;
        if self.material(layer) != Material::Water {
            return None;
        }
        let top = (layer..LAYERS)
            .take_while(|index| self.material(*index) == Material::Water)
            .last()
            .unwrap_or(layer);
        Some(layer_altitude(top) + 1.0)
    }

    /// Whether rain reaches `altitude`: nothing solid stands above it in this
    /// column. The ONE rule every rain effect asks, on the CPU here and on the
    /// GPU as "the topmost drawn run's top", which is the same boundary
    /// because merging runs only ever fills the gaps BETWEEN them
    /// (`drawn_runs`). How much sky LIGHT reaches a point is a different
    /// question: light spreads sideways into a cave and rain does not.
    pub fn open_to_sky(&self, altitude: f32) -> bool {
        match self.surface() {
            None => true,
            Some(top) => altitude >= layer_altitude(top) + 1.0 - 1e-3,
        }
    }

    /// Solid runs, bottom up, as half-open layer ranges. What the renderer draws
    /// a cap and walls for, instead of one cap at one height.
    pub fn runs(&self) -> Vec<(usize, usize)> {
        let mut runs = Vec::new();
        let mut start = None;
        for index in 0..LAYERS {
            match (self.solid(index), start) {
                (true, None) => start = Some(index),
                (false, Some(from)) => {
                    runs.push((from, index));
                    start = None;
                }
                _ => {}
            }
        }
        if let Some(from) = start {
            runs.push((from, LAYERS));
        }
        runs
    }
}

/// Runs a renderer carries per column. Four, because most columns are one solid
/// run from bedrock to the surface and a cave adds a second: the budget is a cap
/// on RUNS rather than on layers, and a column with more draws its largest four,
/// which is a bounded and visible failure rather than a buffer overrun.
pub const MAX_RUNS: usize = 4;

/// A solid run and the materials its faces are drawn in.
///
/// TWO materials, because a run is not made of one thing. A surface run is a
/// metre of turf over tens of metres of rock, and a flank drawn in the material
/// at its top is forty metres of wall painted like a meadow - which is exactly
/// what the first capture from inside a cave showed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    /// First solid layer.
    pub from: usize,
    /// One past the last solid layer.
    pub to: usize,
    /// The material at the run's top: its cap, and the first metre of its flank.
    pub material: Material,
    /// The material the rest of it is made of, taken at the run's bottom.
    pub body: Material,
}

impl Run {
    /// Packed for one GPU word: `from | to << 9 | code << 18`. A real run
    /// always has `to >= 1`, so a word whose `to` field is zero is an absent
    /// run - `from` cannot serve as the sentinel, because the bedrock run
    /// legitimately starts at layer zero.
    ///
    /// The codes are the RENDERER's material table rather than this crate's
    /// `Material`, because which tile and tint a material draws in is a fact
    /// about a shader and this crate does not have one.
    pub fn packed(self, code: u32, body: u32) -> u32 {
        debug_assert!(self.to >= 1 && self.to <= LAYERS && self.from < LAYERS);
        (self.from as u32 & 0x1ff)
            | (self.to as u32 & 0x1ff) << 9
            | (code & 0xf) << 18
            | (body & 0xf) << 22
    }

    /// The word for a run that is not there.
    pub const ABSENT: u32 = 0;
}

impl Column {
    /// The `MAX_RUNS` runs a renderer draws, bottom up. Bottom up rather than
    /// largest first, because the renderer asks "is this the top run" and "is
    /// this the bottom one" of the list it is given.
    ///
    /// A column with more runs than the budget has them MERGED, never dropped:
    /// the thinnest air gap is filled with rock and its two runs become one,
    /// repeatedly, until the list fits. About one column in twenty on this body
    /// has more than four runs (`column_cost` measures it; the most is eight).
    ///
    /// Merging rather than dropping, because the two failures are not
    /// comparable. A dropped run is rock that is not drawn, which from inside a
    /// cave is a WINDOW out of the world - the thing this tier exists to close.
    /// A merged gap is a cave nobody can see, and the thinnest gap in a column
    /// is the one least worth walking into. So the drawn runs always COVER
    /// every solid layer, and what the budget costs is a cave rather than a
    /// hole.
    pub fn drawn_runs(&self) -> Vec<Run> {
        let mut runs = self.runs();
        while runs.len() > MAX_RUNS {
            let thinnest = (1..runs.len())
                .min_by_key(|&i| runs[i].0 - runs[i - 1].1)
                .expect("more than MAX_RUNS runs leaves a gap to close");
            runs[thinnest - 1].1 = runs[thinnest].1;
            runs.remove(thinnest);
        }
        runs.into_iter()
            .map(|(from, to)| Run {
                from,
                to,
                material: self.material(to - 1),
                body: self.material(from),
            })
            .collect()
    }

    /// The four packed words a GPU record carries, absent runs last. `code`
    /// maps a material to the renderer's own table.
    pub fn packed_runs(&self, code: impl Fn(Material) -> u32) -> [u32; MAX_RUNS] {
        let mut words = [Run::ABSENT; MAX_RUNS];
        for (word, run) in words.iter_mut().zip(self.drawn_runs()) {
            *word = run.packed(code(run.material), code(run.body));
        }
        words
    }
}

/// What a column presents to a body at one altitude.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contact {
    /// Altitude of the top of the solid run at or below, if any.
    pub floor: Option<f32>,
    /// Altitude of the bottom of the solid run above, if any.
    pub ceiling: Option<f32>,
    /// Altitude of the surface of the water standing ON that floor, if the
    /// layer directly above it is water.
    ///
    /// The column is the authority on where water is, and this is how a
    /// walker asks. A height field can only answer "how far below sea level
    /// is the ground here", which says a cave carved under land below sea
    /// level is full of sea; the column says that pocket is air, and it is.
    pub water: Option<f32>,
}

/// Build one cell's column, carved by the region's worms.
///
/// The top comes from `surface_altitude` and the top material from
/// `top_material`, which is the whole point: the heightfield tiers and the
/// column tier read ONE description of where the ground is and what it is made
/// of, so they cannot drift into two worlds. The worms are gathered once for a
/// region (`worms::gather`) and handed to every column in it.
pub fn generate(
    worms: &Worms,
    field: &WormField,
    terrain: &TerrainConfig,
    direction: Vec3,
) -> Column {
    generate_edited(worms, field, terrain, direction, &[])
}

/// The generated column with a player's changes applied over it.
///
/// The edits come LAST, after the worms, because they are what somebody did
/// to the world the generator made. Taking `&[(u16, Material)]` rather than
/// the whole [`Edits`](crate::edits::Edits) keeps this crate's one rule about
/// order visible at the call: the slice is applied front to back, and it is
/// the caller's business that its order is stable.
///
/// Bedrock is restored after them, so no edit can open a hole through the
/// bottom of the world however it was recorded.
/// A solid column with a player's changes applied: the tier's RIM, which is
/// generated uncarved for the reason `planet_column.rs` gives, and still has
/// to carry what somebody dug out of it.
pub fn generate_edited_solid(
    terrain: &TerrainConfig,
    direction: Vec3,
    edits: &[(u16, Material)],
) -> Column {
    let mut column = generate_solid(terrain, direction);
    for &(layer, material) in edits {
        if let Some(slot) = column.layers.get_mut(layer as usize) {
            *slot = material;
        }
    }
    column.layers[0] = Material::Stone;
    column
}

pub fn generate_edited(
    worms: &Worms,
    field: &WormField,
    terrain: &TerrainConfig,
    direction: Vec3,
    edits: &[(u16, Material)],
) -> Column {
    let mut column = generate_solid(terrain, direction);
    worms.carve(terrain, direction, field.floor_layers, |index| {
        column.layers[index] = Material::Air;
    });
    for &(layer, material) in edits {
        if let Some(slot) = column.layers.get_mut(layer as usize) {
            *slot = material;
        }
    }
    column.layers[0] = Material::Stone;
    column
}

/// The same column with NO carve: solid from bedrock to the surface.
///
/// What this is for is the EDGE of whatever region has columns at all. Every
/// tier outside that region answers from the heightfield, which assumes the
/// ground below a cap is solid; that assumption is only safe while nobody can
/// be under a cap, and a cave is exactly being under one. A solid ring makes
/// the assumption true rather than hoping it is.
/// The sod: the top layer, which is the biome's own material and the only
/// layer that shows it. Tenebris's `planet_gen` gives the surface block one
/// cell and this is that cell.
pub const SOD_DEPTH_M: f32 = 1.0;

/// How deep the soil runs under the sod, from Tenebris's `altitude_m > surface
/// - 4.0`. Below it the body is stone.
pub const SOIL_DEPTH_M: f32 = 4.0;

/// What stands `depth_m` below a surface made of `top`: the sod for the first
/// metre, then the soil that belongs to that sod (sand keeps sand, and a rocky
/// or snowy top has stone directly under it, as the reference has), then
/// stone. One rule, because the stack a player digs through, the material a
/// wall shows at that depth and the wear a tool takes are the same question.
///
/// `planet_surface.wgsl` mirrors the three bands for a wall face and takes
/// their depths from `params.ground`, which is fed from the two constants
/// above: the numbers have one source even though the branch is written twice.
pub fn material_at_depth(top: Material, depth_m: f32) -> Material {
    if depth_m <= SOD_DEPTH_M {
        return top;
    }
    if depth_m <= SOIL_DEPTH_M {
        return match top {
            Material::Sand => Material::Sand,
            Material::Snow | Material::Rock | Material::Stone => Material::Stone,
            _ => Material::Soil,
        };
    }
    Material::Stone
}

/// Where the ground is, as a whole layer: the generator's altitude, floored.
///
/// ONE function, read by the column and by the heightfield record alike. The
/// column used to fill every layer whose altitude was under the UNFLOORED
/// altitude while the record was drawn at the floored one, so every cell in
/// the tier carried a metre of rock its cap was drawn a metre inside of: the
/// aim ray dug that invisible layer first, the first edit to a cell
/// reconciled its cap UP a metre, and both wall rules had to dodge a top that
/// stood proud of the cap. A surface is the layer boundary under the altitude,
/// and the cap and the column's top are the same number by construction.
pub fn surface_m(terrain: &TerrainConfig, direction: Vec3) -> f32 {
    planet_gen::surface_altitude(terrain, direction).floor()
}

pub fn generate_solid(terrain: &TerrainConfig, direction: Vec3) -> Column {
    let surface_m = surface_m(terrain, direction);
    let top = planet_gen::top_material(terrain, direction, surface_m);
    let mut layers = [Material::Air; LAYERS];
    for (index, layer) in layers.iter_mut().enumerate() {
        let altitude = layer_altitude(index);
        if altitude >= surface_m {
            // Above the ground: water up to sea level, air over that. The sea
            // is not solid, so it is swum rather than stood on, which is the
            // swimming contract this project already holds.
            *layer = if altitude < terrain.sea_level_m {
                Material::Water
            } else {
                Material::Air
            };
            continue;
        }
        *layer = material_at_depth(top, surface_m - altitude);
    }
    // Bedrock. Never mineable, and the reference's own rule.
    layers[0] = Material::Stone;
    Column { layers }
}

#[cfg(test)]
mod rain_tests {
    use super::*;

    /// Ground at `ground`, a roof from `roof_from` up to `roof_to`, air above.
    fn roofed(ground: usize, roof_from: usize, roof_to: usize) -> Column {
        let mut layers = [Material::Air; LAYERS];
        layers[..=ground].fill(Material::Stone);
        layers[roof_from..=roof_to].fill(Material::Stone);
        Column { layers }
    }

    #[test]
    fn rain_reaches_the_top_of_a_column_and_nothing_under_its_roof() {
        let column = roofed(20, 25, 27);
        let cave_floor = layer_altitude(20) + 1.0;
        let roof_top = layer_altitude(27) + 1.0;
        assert!(
            !column.open_to_sky(cave_floor),
            "a cave floor is under rock"
        );
        assert!(
            !column.open_to_sky(cave_floor + 1.6),
            "so is an eye in the cave"
        );
        assert!(
            column.open_to_sky(roof_top),
            "the ground on the roof is open"
        );
        assert!(column.open_to_sky(roof_top + 10.0));
        // A single block placed over open ground covers it.
        let mut covered = roofed(20, 0, 0);
        let ground_top = layer_altitude(20) + 1.0;
        assert!(covered.open_to_sky(ground_top));
        covered.set(22, Material::Stone);
        assert!(
            !covered.open_to_sky(ground_top),
            "a block overhead keeps the rain off"
        );
        assert!(covered.open_to_sky(layer_altitude(22) + 1.0));
        // Water and torches are not a roof.
        let mut pool = roofed(20, 0, 0);
        pool.set(21, Material::Water);
        pool.set(23, Material::Torch);
        assert!(pool.open_to_sky(ground_top));
    }

    /// The GPU asks "is this the topmost drawn run", the CPU "is this above the
    /// topmost solid layer". The packed runs are what the GPU sees, so the
    /// topmost one's top must be exactly the CPU's boundary, even in a column
    /// with more runs than the budget, whose gaps are merged.
    #[test]
    fn the_topmost_drawn_run_ends_where_the_sky_opens() {
        let mut many = roofed(20, 0, 0);
        for (i, layer) in (30..LAYERS - 2).step_by(3).enumerate().take(9) {
            many.set(
                layer,
                if i % 2 == 0 {
                    Material::Stone
                } else {
                    Material::Dirt
                },
            );
        }
        for column in [roofed(20, 25, 27), roofed(40, 0, 0), many] {
            let runs = column.drawn_runs();
            assert!(runs.len() <= MAX_RUNS);
            let top = runs.last().expect("every column has bedrock").to;
            let boundary = layer_altitude(top);
            assert!(column.open_to_sky(boundary));
            assert!(!column.open_to_sky(boundary - 0.01));
            let word = column.packed_runs(|_| 1)[runs.len() - 1];
            assert_eq!(
                (word >> 9) & 0x1ff,
                top as u32,
                "the shader reads this field"
            );
        }
    }
}

#[cfg(test)]
mod water_tests {
    use super::*;

    /// A column with rock to `ground` and water over it to `sea`.
    fn flooded(ground: usize, sea: usize) -> Column {
        let mut layers = [Material::Air; LAYERS];
        layers[..=ground].fill(Material::Stone);
        layers[ground + 1..=sea].fill(Material::Water);
        Column { layers }
    }

    /// The sea: a walker on the seabed stands in water as deep as the sea is
    /// over it, and the surface is the top of the water RUN rather than a
    /// constant, so a pool that stops short reads as a pool.
    #[test]
    fn a_column_reports_the_water_standing_on_its_floor() {
        let column = flooded(20, 40);
        let floor = layer_altitude(20) + 1.0;
        let surface = layer_altitude(40) + 1.0;
        let contact = column.contact(floor + 0.5);
        assert_eq!(contact.floor, Some(floor));
        assert_eq!(contact.water, Some(surface));
        assert_eq!(surface - floor, 20.0, "twenty layers of water");
        // In it, over it, and in the rock under it.
        assert_eq!(column.water_surface(floor + 0.5), Some(surface));
        assert_eq!(column.water_surface(surface + 0.5), None);
        assert_eq!(column.water_surface(floor - 0.5), None);
    }

    /// The owner's picture: a cave carved under land below sea level. There
    /// is no water in the column, so there is none in the cave, whatever a
    /// height field would say about its depth.
    #[test]
    fn a_cave_under_land_below_sea_level_holds_no_water() {
        let mut layers = [Material::Stone; LAYERS];
        // A pocket of air well under the ground, and nothing above sea level.
        layers[30..36].fill(Material::Air);
        layers[100..].fill(Material::Air);
        let column = Column { layers };
        let floor = layer_altitude(29) + 1.0;
        let contact = column.contact(floor + 0.5);
        assert_eq!(contact.floor, Some(floor));
        assert_eq!(contact.ceiling, Some(layer_altitude(36)));
        assert_eq!(contact.water, None, "a sealed pocket is dry");
        assert_eq!(column.water_surface(floor + 0.5), None);
        // And the same column with the pocket flooded says so, which is what
        // the flooding phase will write.
        let mut flooded = column;
        flooded.layers[30..36].fill(Material::Water);
        assert_eq!(
            flooded.contact(floor + 0.5).water,
            Some(layer_altitude(35) + 1.0)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worms;

    const TERRAIN: TerrainConfig = TerrainConfig::TENEBRIS;
    const FIELD: WormField = WormField::DEFAULT;

    /// A spiral of directions inside `reach_m` of `from`, nearest first.
    fn near(from: Vec3, reach_m: f32, count: usize) -> Vec<Vec3> {
        let t0 = Vec3::Y.cross(from).normalize();
        let b0 = from.cross(t0);
        let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
        (0..count)
            .map(|i| {
                let t = (i as f32 + 0.5) / count as f32;
                let radius = reach_m / TERRAIN.radius_m * t.sqrt();
                let angle = golden * i as f32;
                (from + (t0 * angle.cos() + b0 * angle.sin()) * radius).normalize()
            })
            .collect()
    }

    /// The spawn, and the region's worms around it.
    fn spawn() -> (Vec3, Worms) {
        let spawn = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        (spawn, worms::gather(&FIELD, &TERRAIN, spawn, 120.0))
    }

    #[test]
    fn the_span_covers_the_measured_relief() {
        assert!(layer_altitude(0) <= -125.0);
        assert!(layer_altitude(LAYERS - 1) + 1.0 >= 158.0);
        assert_eq!(layer_at(BASE_M as f32), Some(0));
        assert_eq!(layer_at(BASE_M as f32 - 0.01), None);
        assert_eq!(layer_at(layer_altitude(LAYERS - 1) + 0.5), Some(LAYERS - 1));
    }

    /// One source for where the ground is. Where no worm touched a column its
    /// top is the surface to the metre; a worm that starts at the surface may
    /// take the ground DOWN, never up, and the app lowers the record to match.
    #[test]
    fn the_column_top_agrees_with_the_surface_height() {
        let (spawn, worms) = spawn();
        for d in near(spawn, 100.0, 300) {
            let surface_m = planet_gen::surface_altitude(&TERRAIN, d);
            if surface_m < TERRAIN.sea_level_m {
                continue;
            }
            let column = generate(&worms, &FIELD, &TERRAIN, d);
            let top = column.surface().expect("land has a solid layer");
            let top_m = layer_altitude(top) + 1.0;
            assert!(
                top_m <= surface_m + 1.0,
                "the carve never raises the ground"
            );
            if generate_solid(&TERRAIN, d).surface() == column.surface() {
                assert!(
                    (top_m - surface_m).abs() <= 1.0,
                    "column top {top_m} against surface {surface_m}"
                );
            }
        }
    }

    #[test]
    fn bedrock_is_solid_and_cannot_be_dug() {
        let (spawn, worms) = spawn();
        for d in near(spawn, 100.0, 50) {
            let mut column = generate(&worms, &FIELD, &TERRAIN, d);
            assert!(column.solid(0));
            assert!(!column.set(0, Material::Air));
            assert!(column.solid(0));
            assert!(column.set(1, Material::Air));
        }
    }

    /// The worms open some of the underground and not most of it. Tubes are
    /// sparse by volume - a Minecraft-like fraction of a percent - and what
    /// matters more is how many columns they cross, which is what a player
    /// meets.
    #[test]
    fn the_worms_open_some_columns_and_not_most_of_the_rock() {
        let (spawn, worms) = spawn();
        let mut hollow = 0usize;
        let mut underground = 0usize;
        let mut crossed = 0usize;
        let mut land = 0usize;
        for d in near(spawn, 100.0, 1_500) {
            let surface_m = planet_gen::surface_altitude(&TERRAIN, d);
            if surface_m < TERRAIN.sea_level_m + TERRAIN.beach_band_m {
                continue;
            }
            land += 1;
            let column = generate(&worms, &FIELD, &TERRAIN, d);
            let top = column.surface().unwrap();
            let air = (1..top).filter(|&i| !column.solid(i)).count();
            hollow += air;
            underground += top;
            crossed += (air > 0) as usize;
        }
        let share = hollow as f32 / underground.max(1) as f32;
        let crossed_share = crossed as f32 / land.max(1) as f32;
        eprintln!(
            "worms: {:.2}% of the underground is hollow, {:.1}% of {land} land columns crossed",
            100.0 * share,
            100.0 * crossed_share
        );
        assert!(
            (0.0005..0.05).contains(&share),
            "{:.2}% of the underground is hollow",
            100.0 * share
        );
        assert!(
            (0.03..0.60).contains(&crossed_share),
            "{:.1}% of land columns are crossed by a worm",
            100.0 * crossed_share
        );
    }

    /// Tenebris's own stack, from `planet_gen::sample_with_profile`: the sod
    /// is one cell, dirt runs to four metres (sand under a desert), stone
    /// below. A rocky or snowy top has no soil under it at all.
    #[test]
    fn the_stack_under_a_cap_is_sod_then_soil_then_stone() {
        for (top, soil) in [
            (Material::Grass, Material::Soil),
            (Material::DryGrass, Material::Soil),
            (Material::JungleGrass, Material::Soil),
            (Material::Dirt, Material::Soil),
            (Material::Sand, Material::Sand),
            (Material::Snow, Material::Stone),
            (Material::Rock, Material::Stone),
            (Material::Stone, Material::Stone),
        ] {
            assert_eq!(material_at_depth(top, 0.0), top);
            assert_eq!(material_at_depth(top, SOD_DEPTH_M), top);
            assert_eq!(material_at_depth(top, SOD_DEPTH_M + 0.001), soil);
            assert_eq!(material_at_depth(top, SOIL_DEPTH_M), soil);
            assert_eq!(
                material_at_depth(top, SOIL_DEPTH_M + 0.001),
                Material::Stone
            );
            assert_eq!(material_at_depth(top, 200.0), Material::Stone);
        }
    }

    /// The generated column IS that rule: one source, so a wall drawn from the
    /// rule and a cell dug out of the column can never disagree.
    #[test]
    fn a_generated_column_agrees_with_the_depth_rule() {
        let (spawn, _) = spawn();
        for d in near(spawn, 60.0, 200) {
            let column = generate_solid(&TERRAIN, d);
            let surface_m = surface_m(&TERRAIN, d);
            let top = planet_gen::top_material(&TERRAIN, d, surface_m);
            for index in 1..LAYERS {
                let altitude = layer_altitude(index);
                if altitude >= surface_m {
                    continue;
                }
                assert_eq!(
                    column.layers[index],
                    material_at_depth(top, surface_m - altitude),
                    "layer {index} at {altitude} m under a {surface_m} m {top:?} surface"
                );
            }
        }
    }

    #[test]
    fn a_carved_column_has_a_floor_and_a_ceiling_to_stand_between() {
        let (spawn, worms) = spawn();
        let mut found = false;
        for d in near(spawn, 100.0, 2_000) {
            let column = generate(&worms, &FIELD, &TERRAIN, d);
            let runs = column.runs();
            if runs.len() < 2 {
                continue;
            }
            let (_, floor_top) = runs[0];
            let (roof_bottom, _) = runs[1];
            if roof_bottom - floor_top < 2 {
                continue;
            }
            let inside = layer_altitude(floor_top) + 0.5;
            let contact = column.contact(inside);
            assert_eq!(contact.floor, Some(layer_altitude(floor_top)));
            assert_eq!(contact.ceiling, Some(layer_altitude(roof_bottom)));
            found = true;
            break;
        }
        assert!(found, "the spawn's region has a cave with room to stand in");
    }

    #[test]
    fn standing_on_open_ground_has_a_floor_and_no_ceiling() {
        let column = generate_solid(&TERRAIN, Vec3::new(0.8776, 0.4794, 0.0).normalize());
        let top = column.surface().unwrap();
        let contact = column.contact(layer_altitude(top) + 1.5);
        assert_eq!(contact.floor, Some(layer_altitude(top) + 1.0));
        assert_eq!(contact.ceiling, None);
    }

    /// The floor is the top of the RUN. A point inside a three-layer wall is
    /// told the wall's top, not the top of the layer it is in: the difference
    /// is a walker stepping onto a ledge and a walker climbing into a wall.
    #[test]
    fn a_point_inside_a_wall_is_told_the_top_of_the_wall() {
        let mut column = generate_solid(&TERRAIN, Vec3::X);
        let top = column.surface().unwrap();
        for index in (top - 6)..(top - 3) {
            column.set(index, Material::Air);
        }
        let wall_bottom = layer_altitude(top - 3);
        let wall_top = layer_altitude(top) + 1.0;
        let inside = column.contact(wall_bottom + 0.3);
        assert_eq!(inside.floor, Some(wall_top));
        assert_eq!(inside.ceiling, None, "nothing over the surface");
        let chamber = column.contact(wall_bottom - 1.5);
        assert_eq!(chamber.floor, Some(layer_altitude(top - 6)));
        assert_eq!(chamber.ceiling, Some(wall_bottom));
    }

    #[test]
    fn a_packed_run_survives_the_round_trip_and_an_absent_one_is_zero() {
        for run in [
            Run {
                from: 0,
                to: LAYERS,
                material: Material::Stone,
                body: Material::Stone,
            },
            Run {
                from: LAYERS - 1,
                to: LAYERS,
                material: Material::Dirt,
                body: Material::Dirt,
            },
        ] {
            let word = run.packed(7, 5);
            assert_ne!(word, Run::ABSENT, "a real run is never the absent word");
            assert_eq!(word & 0x1ff, run.from as u32);
            assert_eq!((word >> 9) & 0x1ff, run.to as u32);
            assert_eq!((word >> 18) & 0xf, 7);
            assert_eq!(word >> 22, 5);
        }
        assert_eq!((Run::ABSENT >> 9) & 0x1ff, 0);
    }

    #[test]
    fn the_drawn_runs_cover_every_solid_layer_within_the_budget() {
        let mut column = generate_solid(&TERRAIN, Vec3::X);
        for index in [40, 60, 80, 100, 120] {
            column.set(index, Material::Air);
        }
        let runs = column.drawn_runs();
        assert!(runs.len() <= MAX_RUNS, "the budget is four runs");
        assert!(
            runs.windows(2).all(|pair| pair[0].to <= pair[1].from),
            "runs stay bottom up and disjoint: {runs:?}"
        );
        for index in 0..LAYERS {
            if column.solid(index) {
                assert!(
                    runs.iter().any(|r| (r.from..r.to).contains(&index)),
                    "layer {index} is solid and is in no drawn run"
                );
            }
        }
        assert_eq!(
            runs.last().map(|r| r.to),
            column.runs().last().map(|&(_, to)| to),
            "the ground underfoot is the top run's own top"
        );
        for run in &runs {
            assert!(column.solid(run.to - 1));
            assert!(column.solid(run.from));
            assert_eq!(run.material, column.material(run.to - 1));
            assert_eq!(run.body, column.material(run.from));
        }
    }

    #[test]
    fn the_same_cell_generates_the_same_column() {
        let (spawn, worms) = spawn();
        let d = near(spawn, 30.0, 8)[5];
        let a = generate(&worms, &FIELD, &TERRAIN, d);
        let b = generate(&worms, &FIELD, &TERRAIN, d);
        assert!((0..LAYERS).all(|i| a.material(i) == b.material(i)));
    }

    #[test]
    fn runs_and_contact_agree_about_what_is_solid() {
        let (spawn, worms) = spawn();
        for d in near(spawn, 60.0, 40) {
            let column = generate(&worms, &FIELD, &TERRAIN, d);
            for (from, to) in column.runs() {
                assert!((from..to).all(|i| column.solid(i)));
                if to < LAYERS {
                    assert!(!column.solid(to));
                }
                let contact = column.contact(layer_altitude(to) - 0.5);
                assert_eq!(contact.floor, Some(layer_altitude(to)));
            }
        }
    }

    /// How far a line of sight runs inside a cave. The sheet carve this
    /// replaced measured a median of 2 m and nothing past 12 m: a slab seen
    /// edge-on is a wall. A tube is a tube.
    #[test]
    #[ignore = "a report: cargo test -p pbd-core sight_lines -- --ignored --nocapture"]
    fn sight_lines() {
        let (_, worms) = spawn();
        // A point on a buried worm's own axis, mid-way along.
        let worm = worms
            .worms
            .iter()
            .filter(|w| !w.surface_start)
            .max_by_key(|w| w.capsules.len())
            .expect("a buried worm near the spawn");
        let mid = worm.capsules[worm.capsules.len() / 2];
        let eye = mid.a;
        let along = (mid.b - mid.a).normalize();
        let up = eye.normalize();
        let side = up.cross(along).normalize();
        let mut reach = Vec::new();
        for i in 0..64 {
            let yaw = std::f32::consts::TAU * i as f32 / 64.0;
            for pitch in [-0.3f32, -0.1, 0.0, 0.1, 0.3] {
                let ray = (along * yaw.cos() + side * yaw.sin()) * pitch.cos() + up * pitch.sin();
                let mut travelled = 0.0f32;
                while travelled < 300.0 {
                    travelled += 0.5;
                    let point = eye + ray * travelled;
                    let here = point.normalize();
                    let altitude = point.length() - TERRAIN.radius_m;
                    let surface = planet_gen::surface_altitude(&TERRAIN, here);
                    if altitude < surface && !worms.hollow(&TERRAIN, here, altitude) {
                        break;
                    }
                }
                reach.push(travelled);
            }
        }
        reach.sort_by(f32::total_cmp);
        let pick = |q: f32| reach[((reach.len() - 1) as f32 * q) as usize];
        println!(
            "\nsight lines from inside a worm, {} rays: median {:.0} m, p75 {:.0} m, p90 {:.0} m, longest {:.0} m; along the tube {:.0} m",
            reach.len(),
            pick(0.5),
            pick(0.75),
            pick(0.90),
            reach[reach.len() - 1],
            reach[reach.len() - 1]
        );
    }

    /// What a column costs to build, which is what decides how many of them a
    /// tier can hold.
    #[test]
    #[ignore = "a report: cargo test -p pbd-core column_cost -- --ignored --nocapture"]
    fn column_cost() {
        let (spawn, _) = spawn();
        let start = std::time::Instant::now();
        let worms = worms::gather(&FIELD, &TERRAIN, spawn, 90.0);
        let gathered = start.elapsed().as_secs_f64();
        let sample = near(spawn, 90.0, 3_000);
        let start = std::time::Instant::now();
        let mut runs = 0usize;
        for d in &sample {
            runs += generate(&worms, &FIELD, &TERRAIN, *d).runs().len();
        }
        let each = start.elapsed().as_secs_f64() / sample.len() as f64;
        println!(
            "\ngather for a 90 m tier: {:.1} ms for {} worms, {} capsules\none column: {:.1} us, {:.2} runs mean",
            gathered * 1e3,
            worms.worms.len(),
            worms.worms.iter().map(|w| w.capsules.len()).sum::<usize>(),
            each * 1e6,
            runs as f64 / sample.len() as f64
        );
    }

    /// Print a real cross-section: a column with air between two solid runs,
    /// straight out of the generator.
    #[test]
    #[ignore = "a report: cargo test -p pbd-core cave_cross_section -- --ignored --nocapture"]
    fn cave_cross_section() {
        let (spawn, worms) = spawn();
        let glyph = |m: Material| match m {
            Material::Air => ' ',
            Material::Water => '~',
            Material::Stone => '#',
            Material::Soil => '+',
            Material::Sand => '.',
            _ => 'o',
        };
        let mut best: Option<(Vec3, usize)> = None;
        for d in near(spawn, 100.0, 2_000) {
            let column = generate(&worms, &FIELD, &TERRAIN, d);
            let runs = column.runs();
            if runs.len() < 2 {
                continue;
            }
            let gap = runs[1].0 - runs[0].1;
            if best.is_none_or(|(_, g)| gap > g) {
                best = Some((d, gap));
            }
        }
        let (direction, gap) = best.expect("a cave near the spawn");
        let column = generate(&worms, &FIELD, &TERRAIN, direction);
        let top = column.surface().unwrap();
        println!("\nONE COLUMN, bottom of the cave upward ({gap} m of air):");
        let runs = column.runs();
        let from = runs[0].1.saturating_sub(4);
        for index in (from..=(top + 2).min(LAYERS - 1)).rev() {
            let contact = column.contact(layer_altitude(index) + 0.5);
            let standing = contact.floor == Some(layer_altitude(index));
            println!(
                "  {:>5.0} m |{}|{}",
                layer_altitude(index),
                glyph(column.material(index)),
                if standing {
                    "  <- a floor to stand on"
                } else {
                    ""
                }
            );
        }
    }
}
