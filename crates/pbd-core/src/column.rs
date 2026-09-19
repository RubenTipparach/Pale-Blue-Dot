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
    pub fn contact(&self, altitude: f32) -> Contact {
        let here = (altitude - BASE_M as f32).floor();
        let start = here.clamp(0.0, LAYERS as f32 - 1.0) as usize;
        // The floor is the top of the first solid layer at or below the sample.
        let floor = (0..=start)
            .rev()
            .find(|index| self.solid(*index))
            .map(|index| layer_altitude(index) + 1.0);
        // The ceiling is the bottom of the first solid layer strictly above the
        // air the sample stands in.
        let ceiling = ((start + 1)..LAYERS)
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
    // opening the hillside into lace.
    let roof = (depth / cave.roof_m.max(0.001)).clamp(0.0, 1.0);
    ridge * roof > cave.threshold
}

/// Build one cell's column.
///
/// The top comes from `surface_altitude` and the top material from
/// `top_material`, which is the whole point: the heightfield tiers and the
/// column tier read ONE description of where the ground is and what it is made
/// of, so they cannot drift into two worlds.
pub fn generate(cave: &CaveField, terrain: &TerrainConfig, direction: Vec3) -> Column {
    let surface_m = planet_gen::surface_altitude(terrain, direction);
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
        if index > cave.floor_layers && hollow(cave, terrain, direction, altitude, surface_m) {
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
                line.push(if hollow(&cave, &TERRAIN, d, altitude, surface_m) {
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
