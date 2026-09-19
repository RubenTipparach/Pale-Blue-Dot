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
//! a player dug. Generated cave systems are this project's own addition.

use crate::planet_gen::{self, TerrainConfig};
use crate::terrain::Material;
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
    pub fn material(&self, index: usize) -> Material {
        self.layers.get(index).copied().unwrap_or(Material::Air)
    }

    /// Whether a layer stops a player. Air and water do not; water is swum.
    pub fn solid(&self, index: usize) -> bool {
        !matches!(self.material(index), Material::Air | Material::Water)
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
        Contact { floor, ceiling }
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
}

/// Salt so the carve is its own noise stream rather than the terrain's.
const CAVE_SEED_SALT: u64 = 0xca7e_0000_0000_0001;
/// And the mouth field its own, so mouths do not sit where the tunnels do.
const MOUTH_SEED_SALT: u64 = 0xca7e_0000_0000_0002;

/// Whether this column is in a MOUTH patch: a place where the carve's surface
/// damping is lifted so a tunnel can break the ground.
///
/// Measured before this existed: 2,530 caves in a tier and none open to the
/// surface, because `roof_m` damps the carve to nothing over the top of every
/// column. That damping is right - lifted everywhere the ground is lace - so
/// the exception is a rare seeded patch rather than a lower `roof_m`. Never
/// below the shore, where an opening would be a dry pocket under the sea.
pub fn mouth(cave: &CaveField, terrain: &TerrainConfig, direction: Vec3, surface_m: f32) -> bool {
    if surface_m < terrain.sea_level_m + terrain.beach_band_m {
        return false;
    }
    let point = direction * (terrain.radius_m / cave.mouth_scale_m.max(1e-3));
    planet_gen::noise01(terrain.seed ^ MOUTH_SEED_SALT, point, 1.0, 2) > cave.mouth_threshold
}

/// How the caves are cut. Ours, not the reference's: it carves nothing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaveField {
    /// Metres across the coarsest tunnel feature.
    pub scale_m: f32,
    /// Ridge value above which a point is hollow. Higher is fewer caves.
    pub threshold: f32,
    /// Metres below the surface where the carve reaches full strength; nearer
    /// the surface it is damped so the ground is not lace.
    pub roof_m: f32,
    /// Layers of solid the carve will not open at the very bottom.
    pub floor_layers: usize,
    /// Metres across a patch where the surface damping is LIFTED, so a tunnel
    /// under it may break the ground: a cave mouth.
    pub mouth_scale_m: f32,
    /// Share of the mouth field's unit range above which a column is in a
    /// mouth patch. Higher is rarer.
    pub mouth_threshold: f32,
    /// How far the carve threshold is lowered AT the ground inside a mouth
    /// patch, easing back to none `roof_m` down: the tunnel sheet flares as it
    /// reaches the surface, which is what turns a line where it crosses the
    /// ground into an opening a walker fits through.
    pub mouth_relax: f32,
}

impl CaveField {
    pub const DEFAULT: CaveField = CaveField {
        // A tunnel a player walks along rather than a pore: tens of metres.
        scale_m: 46.0,
        // Chosen off the measured curve rather than guessed. Sweeping it over
        // the body: 0.80 opens 14.4% of the underground, 0.85 opens 6.9%, 0.88
        // opens 3.7%, 0.94 opens 0.5%. At 0.88 about 31% of land columns carry
        // a cave somewhere in them, so there is plenty to find while the rock
        // stays rock. `carve_report` is the sweep.
        threshold: 0.88,
        roof_m: 9.0,
        floor_layers: 3,
        // Patches a few tens of metres across: at a hundred and twenty the
        // first mouth rendered as a crater thirty-five metres wide and
        // fifteen deep, a basin with tunnels off its walls rather than a hole
        // in a hillside. The field is fBm remapped to a
        // unit range and rarely reaches its ends, so the threshold reads
        // lower than it sounds: `mouth_sweep` measures 0.70 as 2% of the land
        // and 0.60 as 17%. Between them is about one patch per ninety-metre
        // tier, which is the Minecraft cadence of a cave entrance every few
        // hundred metres.
        mouth_scale_m: 48.0,
        mouth_threshold: 0.65,
        mouth_relax: 0.15,
    };
}

impl Default for CaveField {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Whether the carve hollows a point.
///
/// Ridged rather than plain fBm, which is the difference between tunnels and
/// bubbles: ridged noise creases along its crests, and thresholding a crest
/// gives a CONNECTED sheet a player can walk through. A plain fBm threshold
/// gives isolated pockets, which is a world with holes in it rather than caves.
pub fn hollow(
    cave: &CaveField,
    terrain: &TerrainConfig,
    direction: Vec3,
    altitude: f32,
    surface_m: f32,
    mouth: bool,
) -> bool {
    let depth = surface_m - altitude;
    if depth <= 0.0 {
        return false;
    }
    // The sample point is the real position, so the field is three-dimensional:
    // the same direction at two altitudes is two different points, which is
    // exactly what the heightfield could never express.
    // The sample point is the position in units of the tunnel scale, so a
    // feature really is `scale_m` metres across. The first cut divided by the
    // radius as well, which put every sample inside a hundredth of a noise
    // period: the field came out near-constant and the carve hollowed 96.9% of
    // the underground. A frequency is metres per feature, not a fraction of a
    // sphere.
    let radius = terrain.radius_m + altitude;
    let point = direction * (radius / cave.scale_m.max(1e-3));
    let ridge = planet_gen::ridged(terrain.seed ^ CAVE_SEED_SALT, point, 1.0, 3, 0.5, 2.1);
    // Damped toward the surface, so a cave has a roof over it rather than
    // opening the hillside into lace. In a mouth patch the opposite: no
    // damping, and the threshold LOWERED toward the ground so the tunnel
    // flares open where it meets it. Lifting the damping alone was measured
    // and was not enough - four columns in a hundred of a patch opened,
    // each a single-cell hole where the thin sheet crossed the surface.
    let near_surface = 1.0 - (depth / cave.roof_m.max(0.001)).clamp(0.0, 1.0);
    if mouth {
        ridge > cave.threshold - cave.mouth_relax * near_surface
    } else {
        ridge * (1.0 - near_surface) > cave.threshold
    }
}

/// The nearest direction to `from` whose column is an OPEN mouth: in a mouth
/// patch, and carved so its own top is below the ground the heightfield gives.
/// A spiral out to `reach_m`, stepping by `step_m`; `None` if the reach holds
/// no mouth. For putting a spawn or a camera where there is a cave to walk
/// into, since patches cover a few percent of the land and a given spot has
/// none more often than not.
pub fn nearest_mouth(
    cave: &CaveField,
    terrain: &TerrainConfig,
    from: Vec3,
    reach_m: f32,
    step_m: f32,
) -> Option<Vec3> {
    let from = from.normalize_or(Vec3::Y);
    let tangent = Vec3::Y.cross(from).normalize_or(Vec3::X);
    let bitangent = from.cross(tangent);
    let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
    let count = ((reach_m / step_m.max(0.1)).powi(2)) as usize;
    (0..count.max(1)).find_map(|i| {
        let t = (i as f32 + 0.5) / count.max(1) as f32;
        let radius = reach_m / terrain.radius_m * t.sqrt();
        let angle = golden * i as f32;
        let here = (from + (tangent * angle.cos() + bitangent * angle.sin()) * radius).normalize();
        let surface = planet_gen::surface_altitude(terrain, here);
        if !mouth(cave, terrain, here, surface) {
            return None;
        }
        let column = generate(cave, terrain, here);
        let top = layer_altitude(column.surface()?) + 1.0;
        (top < surface - 0.5).then_some(here)
    })
}

/// Build one cell's column.
///
/// The top comes from `surface_altitude` and the top material from
/// `top_material`, which is the whole point: the heightfield tiers and the
/// column tier read ONE description of where the ground is and what it is made
/// of, so they cannot drift into two worlds.
pub fn generate(cave: &CaveField, terrain: &TerrainConfig, direction: Vec3) -> Column {
    build(Some(cave), terrain, direction)
}

/// The same column with NO carve: solid from bedrock to the surface.
///
/// What this is for is the EDGE of whatever region has columns at all. Every
/// tier outside that region answers from the heightfield, which assumes the
/// ground below a cap is solid; that assumption is only safe while nobody can
/// be under a cap, and a cave is exactly being under one. A solid ring makes
/// the assumption true rather than hoping it is.
pub fn generate_solid(terrain: &TerrainConfig, direction: Vec3) -> Column {
    build(None, terrain, direction)
}

fn build(cave: Option<&CaveField>, terrain: &TerrainConfig, direction: Vec3) -> Column {
    let surface_m = planet_gen::surface_altitude(terrain, direction);
    let top = planet_gen::top_material(terrain, direction, surface_m);
    let open = cave.is_some_and(|cave| mouth(cave, terrain, direction, surface_m));
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
        let depth = surface_m - altitude;
        *layer = if depth <= 1.0 {
            top
        } else if depth <= 4.0 {
            match top {
                Material::Sand => Material::Sand,
                Material::Snow | Material::Rock | Material::Stone => Material::Stone,
                _ => Material::Soil,
            }
        } else {
            Material::Stone
        };
        if let Some(cave) = cave
            && index > cave.floor_layers
            && hollow(cave, terrain, direction, altitude, surface_m, open)
        {
            *layer = Material::Air;
        }
    }
    // Bedrock. Never mineable, and the reference's own rule.
    layers[0] = Material::Stone;
    Column { layers }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TERRAIN: TerrainConfig = TerrainConfig::TENEBRIS;

    fn dirs(count: usize) -> Vec<Vec3> {
        let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
        (0..count)
            .map(|i| {
                let y = 1.0 - 2.0 * (i as f32 + 0.5) / count as f32;
                let r = (1.0 - y * y).max(0.0).sqrt();
                let a = golden * i as f32;
                Vec3::new(a.cos() * r, y, a.sin() * r).normalize()
            })
            .collect()
    }

    /// What a column costs to build, which is what decides how many of them a
    /// tier can hold. A report rather than an assertion: a timing is a fact
    /// about this machine, and a test that pinned it would fail on another.
    #[test]
    #[ignore = "a report: cargo test -p pbd-core column_cost -- --ignored --nocapture"]
    fn column_cost() {
        let cave = CaveField::DEFAULT;
        let sample = dirs(2_000);
        let start = std::time::Instant::now();
        let mut runs = 0usize;
        for d in &sample {
            runs += generate(&cave, &TERRAIN, *d).runs().len();
        }
        let each = start.elapsed().as_secs_f64() / sample.len() as f64;
        let mut over = 0usize;
        let mut most = 0usize;
        for d in &sample {
            let count = generate(&cave, &TERRAIN, *d).runs().len();
            most = most.max(count);
            if count > MAX_RUNS {
                over += 1;
            }
        }
        println!(
            "\none column: {:.1} us, {:.2} runs mean, most {most}, {:.1}% over the budget of {MAX_RUNS}",
            each * 1e6,
            runs as f64 / sample.len() as f64,
            100.0 * over as f64 / sample.len() as f64
        );
        for count in [4_000usize, 40_670, 65_536] {
            println!(
                "  {count} columns: {:.2} s, {:.1} MiB of layers",
                each * count as f64,
                (count * LAYERS) as f64 / 1048576.0
            );
        }
    }

    /// How far a line of sight actually runs inside the ground.
    ///
    /// The instrument that settled an argument about a picture. A capture from
    /// inside a cave showed open sky and a sea two kilometres off, and there
    /// were two candidate explanations: a renderer leaking, or a carve so
    /// aggressive that the sight-lines are REAL. This measures the carve alone,
    /// with no renderer in it: rays from a point in a chamber, sampled every
    /// half metre, stopped at the first solid sample.
    #[test]
    #[ignore = "a report: cargo test -p pbd-core sight_lines -- --ignored --nocapture"]
    fn sight_lines() {
        let cave = CaveField::DEFAULT;
        // The spawn, which is where the capture stands.
        let spawn = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        // A spiral of directions inside thirty metres of the spawn, which is
        // where the capture looks; `dirs` spreads over the whole sphere and a
        // thirty metre cap on a 4,800 m body catches none of it.
        let near = {
            let t0 = Vec3::Y.cross(spawn).normalize();
            let b0 = spawn.cross(t0);
            let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
            (0..4_000)
                .map(|i| {
                    let t = (i as f32 + 0.5) / 4_000.0;
                    let radius = 30.0 / TERRAIN.radius_m * t.sqrt();
                    let angle = golden * i as f32;
                    (spawn + (t0 * angle.cos() + b0 * angle.sin()) * radius).normalize()
                })
                .collect::<Vec<_>>()
        };
        let mut start = None;
        for d in near {
            let column = generate(&cave, &TERRAIN, d);
            let runs = column.drawn_runs();
            let surface = column.surface().map_or(0.0, |t| layer_altitude(t) + 1.0);
            for pair in runs.windows(2) {
                let floor = layer_altitude(pair[0].to);
                let roof = layer_altitude(pair[1].from);
                if (2.5..=12.0).contains(&(roof - floor))
                    && (4.0..=40.0).contains(&(surface - roof))
                {
                    start = Some((d, floor + 1.6));
                    break;
                }
            }
            if start.is_some() {
                break;
            }
        }
        let (direction, altitude) = start.expect("a chamber near the spawn");
        let eye = direction * (TERRAIN.radius_m + altitude);
        let tangent = Vec3::Y.cross(direction).normalize();
        let bitangent = direction.cross(tangent);
        // A fan of rays across the frame: level, and up and down a little.
        let mut reach = Vec::new();
        for i in 0..64 {
            let yaw = std::f32::consts::TAU * i as f32 / 64.0;
            for pitch in [-0.3f32, -0.1, 0.0, 0.1, 0.3] {
                let ray = (tangent * yaw.cos() + bitangent * yaw.sin()) * pitch.cos()
                    + direction * pitch.sin();
                let mut travelled = 0.0f32;
                while travelled < 300.0 {
                    travelled += 0.5;
                    let point = eye + ray * travelled;
                    let here = point.normalize();
                    let up = point.length() - TERRAIN.radius_m;
                    let surface = planet_gen::surface_altitude(&TERRAIN, here);
                    let open = mouth(&cave, &TERRAIN, here, surface);
                    if up < surface && !hollow(&cave, &TERRAIN, here, up, surface, open) {
                        break;
                    }
                }
                reach.push(travelled);
            }
        }
        reach.sort_by(f32::total_cmp);
        let pick = |q: f32| reach[((reach.len() - 1) as f32 * q) as usize];
        let escaped = reach.iter().filter(|r| **r >= 300.0).count();
        println!(
            "\nsight lines from a chamber at {altitude:.0} m, 320 rays:\n  \
             median {:.0} m, p75 {:.0} m, p90 {:.0} m, longest {:.0} m\n  \
             {escaped} of {} ran the whole 300 m without meeting rock",
            pick(0.5),
            pick(0.75),
            pick(0.90),
            reach[reach.len() - 1],
            reach.len()
        );
    }

    /// The mouth rule, swept: how much of the land is in a patch at each
    /// threshold, and how many of those columns actually OPEN - their carved
    /// column has air within a metre of its own surface, which is a tunnel
    /// breaking the ground rather than a sealed one under a lifted patch.
    #[test]
    #[ignore = "a report: cargo test -p pbd-core mouth_sweep -- --ignored --nocapture"]
    fn mouth_sweep() {
        let sample = dirs(20_000);
        let land: Vec<(Vec3, f32)> = sample
            .iter()
            .map(|d| (*d, planet_gen::surface_altitude(&TERRAIN, *d)))
            .filter(|(_, h)| *h >= TERRAIN.sea_level_m + TERRAIN.beach_band_m)
            .collect();
        println!("\n{} land columns of {}", land.len(), sample.len());
        for threshold in [0.70f32, 0.65, 0.60] {
            let cave = CaveField {
                mouth_threshold: threshold,
                ..CaveField::DEFAULT
            };
            let mut patched = 0;
            let mut open = 0;
            for (d, surface) in &land {
                if !mouth(&cave, &TERRAIN, *d, *surface) {
                    continue;
                }
                patched += 1;
                let column = generate(&cave, &TERRAIN, *d);
                let top = column.surface().unwrap();
                // Open: some air in the two metres under the ground.
                if (top.saturating_sub(2)..top).any(|i| !column.solid(i)) || {
                    let below = layer_at(*surface - 1.0).unwrap_or(0);
                    !column.solid(below)
                } {
                    open += 1;
                }
            }
            println!(
                "  threshold {threshold:.2}: {:.1}% of land in a patch, {open} of {patched} open",
                100.0 * patched as f32 / land.len().max(1) as f32
            );
        }
    }

    /// Print a real cross-section, straight out of the generator. Proof that a
    /// cave exists before anything can draw one: a column with air between two
    /// solid runs, and a horizontal slice showing the tunnels connect rather
    /// than being isolated pockets.
    #[test]
    #[ignore = "a report: cargo test -p pbd-core cave_cross_section -- --ignored --nocapture"]
    fn cave_cross_section() {
        let cave = CaveField::DEFAULT;
        let glyph = |m: Material| match m {
            Material::Air => ' ',
            Material::Water => '~',
            Material::Stone => '#',
            Material::Soil => '+',
            Material::Sand => '.',
            _ => 'o',
        };
        // The deepest cave on a sample of the body, so the picture is of a real
        // cell rather than a lucky one.
        let mut best: Option<(Vec3, usize)> = None;
        for d in dirs(4_000) {
            if planet_gen::surface_altitude(&TERRAIN, d) < TERRAIN.sea_level_m + 10.0 {
                continue;
            }
            let column = generate(&cave, &TERRAIN, d);
            let runs = column.runs();
            if runs.len() < 2 {
                continue;
            }
            let gap = runs[1].0 - runs[0].1;
            if best.is_none_or(|(_, g)| gap > g) {
                best = Some((d, gap));
            }
        }
        let (direction, gap) = best.expect("the body must have a cave on it");
        let column = generate(&cave, &TERRAIN, direction);
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

        // And a slice across the body at one depth, so the tunnels can be seen
        // to run rather than to dot.
        println!("\nA HORIZONTAL SLICE, 30 m under the surface, 64 cells wide:");
        let (across, _) = direction.any_orthonormal_pair();
        let step = 2.833 / TERRAIN.radius_m;
        for row in 0..14 {
            let mut line = String::new();
            let up = direction.cross(across).normalize();
            for col in 0..64 {
                let d = (direction
                    + across * (step * (col as f32 - 32.0))
                    + up * (step * (row as f32 - 7.0)))
                    .normalize();
                let surface_m = planet_gen::surface_altitude(&TERRAIN, d);
                let altitude = surface_m - 30.0;
                let open = mouth(&cave, &TERRAIN, d, surface_m);
                line.push(if hollow(&cave, &TERRAIN, d, altitude, surface_m, open) {
                    ' '
                } else {
                    '#'
                });
            }
            println!("  |{line}|");
        }
    }

    #[test]
    #[ignore = "a report: cargo test -p pbd-core carve_report -- --ignored --nocapture"]
    fn carve_report() {
        let terrain = TERRAIN;
        for threshold in [0.80f32, 0.85, 0.88, 0.91, 0.94, 0.97] {
            let cave = CaveField {
                threshold,
                ..CaveField::DEFAULT
            };
            let (mut solid, mut hollowed, mut caved) = (0usize, 0usize, 0usize);
            let sample = dirs(600);
            for d in &sample {
                let surface_m = planet_gen::surface_altitude(&terrain, *d);
                if surface_m < terrain.sea_level_m + 4.0 {
                    continue;
                }
                let column = generate(&cave, &terrain, *d);
                let top = column.surface().unwrap_or(0);
                for index in (cave.floor_layers + 1)..top {
                    if column.solid(index) {
                        solid += 1;
                    } else {
                        hollowed += 1;
                    }
                }
                if column.runs().len() > 1 {
                    caved += 1;
                }
            }
            println!(
                "threshold {threshold:.2}  hollow {:.2}%  columns with a cave {:.1}%",
                100.0 * hollowed as f32 / (solid + hollowed).max(1) as f32,
                100.0 * caved as f32 / sample.len() as f32,
            );
        }
    }

    #[test]
    fn the_span_covers_the_measured_relief() {
        // The generator's own extremes, so a summit or a basin cannot fall
        // outside the column and be silently flattened.
        let (mut peak, mut floor) = (f32::MIN, f32::MAX);
        for d in dirs(20_000) {
            let h = planet_gen::surface_altitude(&TERRAIN, d);
            peak = peak.max(h);
            floor = floor.min(h);
        }
        assert!(floor > BASE_M as f32, "basin {floor} under the column");
        assert!(
            peak < layer_altitude(LAYERS - 1),
            "summit {peak} over the column"
        );
    }

    #[test]
    fn the_column_top_agrees_with_the_surface_height() {
        // One source for where the ground is. If these drift, the coarse tiers
        // and the column tier are two different worlds meeting at a band edge.
        let cave = CaveField::DEFAULT;
        for d in dirs(300) {
            let column = generate(&cave, &TERRAIN, d);
            let surface_m = planet_gen::surface_altitude(&TERRAIN, d);
            if surface_m < TERRAIN.sea_level_m {
                continue;
            }
            let top = column.surface().expect("land has a solid layer");
            let top_m = layer_altitude(top) + 1.0;
            if mouth(&cave, &TERRAIN, d, surface_m) {
                // The one exception, and it is a hole rather than a drift: a
                // mouth may take the ground DOWN, never up, and the app lowers
                // the record to match so there is still one source.
                assert!(top_m <= surface_m + 1.0, "a mouth never raises the ground");
                continue;
            }
            assert!(
                (top_m - surface_m).abs() <= 1.0,
                "column top {top_m} against surface {surface_m}"
            );
        }
    }

    #[test]
    fn bedrock_is_solid_and_cannot_be_dug() {
        let cave = CaveField::DEFAULT;
        let mut column = generate(&cave, &TERRAIN, Vec3::new(0.3, 0.5, 0.8).normalize());
        assert!(column.solid(0));
        assert!(!column.set(0, Material::Air), "bedrock must refuse");
        assert!(column.solid(0));
        assert!(
            column.set(40, Material::Air),
            "and everything else must not"
        );
    }

    #[test]
    fn the_carve_opens_some_of_the_underground_and_not_most_of_it() {
        // A carve that opens nothing is not caves; one that opens most of the
        // rock is not ground. Measured over the body.
        let cave = CaveField::DEFAULT;
        let (mut solid, mut hollowed) = (0usize, 0usize);
        for d in dirs(400) {
            let surface_m = planet_gen::surface_altitude(&TERRAIN, d);
            if surface_m < TERRAIN.sea_level_m + 4.0 {
                continue;
            }
            let column = generate(&cave, &TERRAIN, d);
            let top = column.surface().unwrap();
            for index in (cave.floor_layers + 1)..top {
                if column.solid(index) {
                    solid += 1;
                } else {
                    hollowed += 1;
                }
            }
        }
        let share = hollowed as f32 / (solid + hollowed).max(1) as f32;
        // A carve that opens nothing is not caves; one that opens a quarter of
        // the rock is not ground. The shipped threshold measures 3.7%.
        assert!(
            (0.01..0.10).contains(&share),
            "the carve opened {:.1}% of the underground",
            100.0 * share
        );
    }

    #[test]
    fn a_carved_column_has_a_floor_and_a_ceiling_to_stand_between() {
        // The point of the whole change: somewhere on the body there is a
        // column with air between two solid runs, and a body in that air is
        // told what is under it and what is over it.
        let cave = CaveField::DEFAULT;
        let mut found = 0;
        for d in dirs(1_500) {
            let surface_m = planet_gen::surface_altitude(&TERRAIN, d);
            if surface_m < TERRAIN.sea_level_m + 4.0 {
                continue;
            }
            let column = generate(&cave, &TERRAIN, d);
            let runs = column.runs();
            if runs.len() < 2 {
                continue;
            }
            // The gap between the two lowest runs is a cave.
            let (_, first_end) = runs[0];
            let (second_start, _) = runs[1];
            assert!(second_start > first_end);
            let inside = layer_altitude(first_end) + 0.5;
            let contact = column.contact(inside);
            assert_eq!(contact.floor, Some(layer_altitude(first_end)));
            assert_eq!(contact.ceiling, Some(layer_altitude(second_start)));
            assert!(
                contact.ceiling.unwrap() > contact.floor.unwrap(),
                "a ceiling must be over its floor"
            );
            found += 1;
        }
        assert!(found > 0, "no column on the body had a cave in it");
    }

    #[test]
    fn standing_on_open_ground_has_a_floor_and_no_ceiling() {
        let cave = CaveField::DEFAULT;
        for d in dirs(200) {
            let surface_m = planet_gen::surface_altitude(&TERRAIN, d);
            if surface_m < TERRAIN.sea_level_m + 8.0 {
                continue;
            }
            let column = generate(&cave, &TERRAIN, d);
            // Well above the ground there is nothing overhead.
            let contact = column.contact(surface_m + 20.0);
            assert!(contact.ceiling.is_none(), "open sky must have no ceiling");
            assert!(contact.floor.is_some(), "and still have ground under it");
        }
    }

    #[test]
    fn a_packed_run_survives_the_round_trip_and_an_absent_one_is_zero() {
        // Every field at its widest: the bedrock run starting at layer zero,
        // and a run reaching the top of the span.
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
        // The sentinel is the `to` field, because `from` is zero on the run
        // that holds the bedrock and that run is always drawn.
        assert_eq!((Run::ABSENT >> 9) & 0x1ff, 0);
    }

    #[test]
    fn the_drawn_runs_cover_every_solid_layer_within_the_budget() {
        let mut column = generate(&CaveField::DEFAULT, &TERRAIN, Vec3::X);
        // Cut four one-metre holes, which makes five runs of very different
        // sizes out of whatever the carve left.
        for index in [40, 60, 80, 100] {
            column.set(index, Material::Air);
        }
        let runs = column.drawn_runs();
        assert!(runs.len() <= MAX_RUNS, "the budget is four runs");
        assert!(
            runs.windows(2).all(|pair| pair[0].to <= pair[1].from),
            "runs stay bottom up and disjoint: {runs:?}"
        );
        // The drawn runs COVER every solid layer. That is the invariant that
        // matters: a solid layer left out of the list is rock nobody draws,
        // which from inside a cave is a window out of the world.
        for index in 0..LAYERS {
            if column.solid(index) {
                assert!(
                    runs.iter().any(|r| (r.from..r.to).contains(&index)),
                    "layer {index} is solid and is in no drawn run"
                );
            }
        }
        // The top of the column is still the top of the last run.
        assert_eq!(
            runs.last().map(|r| r.to),
            column.runs().last().map(|&(_, to)| to),
            "the ground underfoot is the top run's own top"
        );
        for run in &runs {
            assert!(column.solid(run.to - 1), "a run's top layer is solid");
            assert!(column.solid(run.from), "a run's bottom layer is solid");
            assert_eq!(run.material, column.material(run.to - 1));
            assert_eq!(run.body, column.material(run.from));
        }
    }

    /// The floor is the top of the RUN. A point inside a three-layer wall is
    /// told the wall's top, not the top of the layer it is in: the difference
    /// is a walker stepping onto a ledge and a walker climbing into a wall.
    #[test]
    fn a_point_inside_a_wall_is_told_the_top_of_the_wall() {
        let mut column = generate_solid(&TERRAIN, Vec3::X);
        let top = column.surface().unwrap();
        // A chamber under three layers of wall under the surface.
        for index in (top - 6)..(top - 3) {
            column.set(index, Material::Air);
        }
        let wall_bottom = layer_altitude(top - 3);
        let wall_top = layer_altitude(top) + 1.0;
        // Inside the wall's lowest layer: the floor is the WALL's top.
        let inside = column.contact(wall_bottom + 0.3);
        assert_eq!(inside.floor, Some(wall_top));
        assert_eq!(inside.ceiling, None, "nothing over the surface");
        // In the chamber: floor and ceiling bound the chamber.
        let chamber = column.contact(wall_bottom - 1.5);
        assert_eq!(chamber.floor, Some(layer_altitude(top - 6)));
        assert_eq!(chamber.ceiling, Some(wall_bottom));
    }

    #[test]
    fn a_solid_column_has_one_run_and_the_same_top_as_the_carved_one() {
        let cave = CaveField::DEFAULT;
        let mut carved_somewhere = false;
        for d in dirs(400) {
            let solid = generate_solid(&TERRAIN, d);
            let carved = generate(&cave, &TERRAIN, d);
            let surface_m = planet_gen::surface_altitude(&TERRAIN, d);
            if !mouth(&cave, &TERRAIN, d, surface_m) {
                assert_eq!(
                    solid.surface(),
                    carved.surface(),
                    "outside a mouth the carve never moves the ground underfoot"
                );
            }
            assert_eq!(solid.runs().len(), 1, "solid rock is one run");
            carved_somewhere |= carved.runs().len() > 1;
        }
        assert!(carved_somewhere, "the sample must include a carved column");
    }

    #[test]
    fn the_same_cell_generates_the_same_column() {
        let cave = CaveField::DEFAULT;
        let d = Vec3::new(0.21, -0.44, 0.87).normalize();
        let a = generate(&cave, &TERRAIN, d);
        let b = generate(&cave, &TERRAIN, d);
        for index in 0..LAYERS {
            assert_eq!(a.material(index), b.material(index), "layer {index}");
        }
    }

    #[test]
    fn runs_and_contact_agree_about_what_is_solid() {
        let cave = CaveField::DEFAULT;
        for d in dirs(120) {
            let column = generate(&cave, &TERRAIN, d);
            for (from, to) in column.runs() {
                assert!(column.solid(from) && column.solid(to - 1));
                if to < LAYERS {
                    assert!(!column.solid(to), "a run must end at air");
                }
            }
        }
    }
}
