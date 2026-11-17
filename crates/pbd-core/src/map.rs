//! What the world map shows at a point on the planet: the ground's own top
//! block and biome, at the altitude the terrain draws.
//!
//! One function the map mockup's rasters and, later, the game's map both
//! call (`world-map` decision 1), so the map cannot show a planet the ground
//! does not have. It is the column's rule, not a second copy of it: the
//! altitude is [`column::surface_m`], floored to the layer as every column
//! is, and the biome and top block are read at that altitude, as the
//! terrain's surface code reads them.
//!
//! Colour is not decided here. What a top block looks like is the renderer's
//! business, and the map takes it from the same tiles.

use crate::column;
use crate::planet_gen::{Biome, TerrainConfig, biome_at, top_material};
use crate::terrain::Material;
use glam::Vec3;

/// One pixel of the base map.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Texel {
    /// The surface above sea level, in metres, floored to the layer as the
    /// column is. Negative is the sea floor.
    pub altitude_m: f32,
    /// The biome the terrain classifies the surface as.
    pub biome: Biome,
    /// The ground's top block, which is what the base map is coloured by
    /// (survey M3: "normal planet terrain like what it looks like on the
    /// world").
    pub top: Material,
    /// Under water: the surface is below the sea level, as a column floods.
    pub sea: bool,
    /// The sea's place here (`ocean-environments` 6.1), as the chart shows
    /// it: `None` on land and on a world made before the sea's places.
    pub place: Option<crate::fauna::Environment>,
    /// How much of the pixel a river's water covers (`map-rivers`), in
    /// thirds: 0 dry, [`RIVER_FULL`] all water. A river stands over the
    /// sea's level on its carved bed, so `sea` never says it.
    pub river: u8,
}

/// A texel's [`Texel::river`] when the river's water covers all of it.
pub const RIVER_FULL: u8 = 3;

/// Whether the column at `d` holds a river's water: the river test the
/// column itself makes (`column::generate_solid`, through
/// [`crate::rivers::water`]), which puts no river in a town's footprint.
pub fn river_water(cfg: &TerrainConfig, d: Vec3) -> bool {
    crate::settlement::ground::top(cfg, d).is_none() && crate::rivers::floods(cfg, d)
}

/// What the map shows at `direction`, a direction from the planet's centre.
/// Any length but zero; it is normalised here. Its river is the point's:
/// all or none ([`river_cover`] gives a pixel's share).
pub fn base_texel(cfg: &TerrainConfig, direction: Vec3) -> Texel {
    let d = direction.normalize_or(Vec3::Y);
    let altitude_m = column::surface_m(cfg, d);
    let sea = altitude_m < cfg.sea_level_m;
    Texel {
        altitude_m,
        biome: biome_at(cfg, d, altitude_m),
        top: top_material(cfg, d, altitude_m),
        sea,
        place: crate::ocean::places::chart_at(cfg, d, altitude_m),
        river: if !sea && river_water(cfg, d) {
            RIVER_FULL
        } else {
            0
        },
    }
}

/// How much of a map pixel centred on `direction` a river's water covers,
/// in thirds ([`Texel::river`]): [`river_water`] at a 3 x 3 grid across the
/// pixel, `east_m` wide and `north_m` tall. Only read where a river's banks
/// reach the pixel ([`crate::rivers::near`]), so dry land pays one lattice
/// read.
pub fn river_cover(cfg: &TerrainConfig, direction: Vec3, east_m: f32, north_m: f32) -> u8 {
    let d = direction.normalize_or(Vec3::Y);
    if !crate::rivers::near(cfg, d, 0.5 * east_m.hypot(north_m)) {
        return 0;
    }
    let east = Vec3::Y.cross(d).try_normalize().unwrap_or(Vec3::X);
    let north = d.cross(east);
    let wet = (-1..=1)
        .flat_map(|j| (-1..=1).map(move |i| (i, j)))
        .filter(|&(i, j)| {
            let offset = east * (i as f32 * east_m / 3.0) + north * (j as f32 * north_m / 3.0);
            river_water(cfg, (d + offset / cfg.radius_m).normalize())
        })
        .count();
    // Nine points to the nearest third: one point alone is a grazed corner,
    // two to four a third.
    ((wet as u32 * u32::from(RIVER_FULL) + 4) / 9).min(u32::from(RIVER_FULL)) as u8
}

/// The record kind of the player's waypoints, its schema and its one id
/// (`starter-missions` G3). Schema 3 is the list with each waypoint's number
/// (`waypoints` decision 4); schema 2 was the list without numbers and
/// schema 1 one optional mark, and both still read ([`read_marks`]).
pub const MARK_KIND: &str = "map-mark";
pub const MARK_SCHEMA: u32 = 3;
pub const MARK_ID: u64 = 0;

/// The most waypoints a player keeps; a new one past this removes the
/// oldest (owner, 2026-11-16).
pub const MARKS_MAX: usize = 4;

/// The least distance a mark is set from the player for "Read the map" to
/// count reaching it, m.
pub const MARK_FAR_M: f32 = 200.0;
/// How near the mark the walker counts as reaching it, m.
pub const MARK_NEAR_M: f32 = 8.0;

/// The player's mark on the map: one place, set and moved by a right click
/// on the map, saved as a `map-mark` record. A world without one has no
/// record.
#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Mark {
    /// The place, a unit direction from the planet's centre.
    pub at: [f32; 3],
    /// How far, along the ground, the player stood from it when it was set.
    pub set_from_m: f32,
    /// Its number, shown as "W1", "W2" ... (`waypoints` decision 3): the
    /// lowest free one when it was added, kept until it is removed. Zero
    /// before [`add_mark`] numbers it, and in a schema 1 or 2 body, which
    /// [`read_marks`] numbers in order.
    #[serde(default)]
    pub number: u8,
}

impl Mark {
    /// A mark at `at` set by a player standing at `from` (directions from
    /// the planet's centre, any length) on a planet of `radius_m`. `None`
    /// for a direction that cannot be normalised.
    pub fn set(at: Vec3, from: Vec3, radius_m: f32) -> Option<Self> {
        let at = at.try_normalize()?;
        let set_from_m = from
            .try_normalize()
            .map_or(0.0, |from| ground_m(at, from, radius_m));
        Some(Self {
            at: at.to_array(),
            set_from_m,
            number: 0,
        })
    }

    /// What the map and the compass call it: "W1", "W2" ...
    pub fn label(&self) -> String {
        format!("W{}", self.number)
    }

    pub fn direction(&self) -> Vec3 {
        Vec3::from_array(self.at)
    }

    /// Whether a walker at `feet` has reached the mark, and the mark was set
    /// far enough away for "Read the map" ([`MARK_FAR_M`]).
    pub fn reached_far(&self, feet: Vec3, radius_m: f32) -> bool {
        let Some(feet) = feet.try_normalize() else {
            return false;
        };
        self.set_from_m >= MARK_FAR_M && ground_m(self.direction(), feet, radius_m) <= MARK_NEAR_M
    }
}

/// Add a waypoint, oldest first, dropping the oldest past [`MARKS_MAX`],
/// and give it the lowest number the others leave free.
pub fn add_mark(marks: &mut Vec<Mark>, mut mark: Mark) {
    let over = (marks.len() + 1).saturating_sub(MARKS_MAX);
    marks.drain(..over.min(marks.len()));
    mark.number = lowest_free(marks);
    marks.push(mark);
}

/// The lowest number from 1 that no waypoint in `marks` holds.
fn lowest_free(marks: &[Mark]) -> u8 {
    (1..=u8::MAX)
        .find(|n| marks.iter().all(|m| m.number != *n))
        .unwrap_or(u8::MAX)
}

/// The waypoints a `map-mark` record's body holds: a schema 3 or 2 list, or
/// a schema 1 body of one optional mark, each numbered. A waypoint with no
/// number (schema 1 and 2), or with one a waypoint before it holds, takes
/// the lowest free number in list order, so an old save's come out W1, W2
/// ... oldest first. `None` for a body that is none of these.
pub fn read_marks(body: &str) -> Option<Vec<Mark>> {
    let mut marks = ron::from_str::<Vec<Mark>>(body).ok().or_else(|| {
        ron::from_str::<Option<Mark>>(body)
            .ok()
            .map(|m| m.into_iter().collect())
    })?;
    for k in 0..marks.len() {
        let n = marks[k].number;
        if marks[..k].iter().any(|m| m.number == n) {
            marks[k].number = 0;
        }
    }
    for k in 0..marks.len() {
        if marks[k].number == 0 {
            marks[k].number = lowest_free(&marks);
        }
    }
    Some(marks)
}

/// The distance along the ground between two unit directions, m.
fn ground_m(a: Vec3, b: Vec3, radius_m: f32) -> f32 {
    a.dot(b).clamp(-1.0, 1.0).acos() * radius_m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planet_gen::surface_altitude;

    /// Seeded directions spread over the sphere.
    fn directions(n: usize) -> impl Iterator<Item = Vec3> {
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        std::iter::repeat_with(move || {
            let mut next = || {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                (state >> 40) as f32 / (1u64 << 24) as f32 * 2.0 - 1.0
            };
            loop {
                let v = Vec3::new(next(), next(), next());
                let length = v.length();
                if length > 0.05 && length <= 1.0 {
                    return v / length;
                }
            }
        })
        .take(n)
    }

    /// The map shows what the ground is: the same altitude, biome, top block
    /// and water as the column the terrain builds there, on 10,000 seeded
    /// directions (`world-map` task 1.1).
    #[test]
    fn the_map_agrees_with_the_ground_on_ten_thousand_directions() {
        let cfg = TerrainConfig::TENEBRIS;
        let mut land = 0;
        for d in directions(10_000) {
            let texel = base_texel(&cfg, d);
            let surface = column::surface_m(&cfg, d);
            assert_eq!(texel.altitude_m, surface.floor(), "{d}");
            assert_eq!(texel.altitude_m, surface_altitude(&cfg, d).floor());
            assert_eq!(texel.biome, biome_at(&cfg, d, surface), "{d}");
            assert_eq!(texel.top, top_material(&cfg, d, surface), "{d}");
            // The column floods every layer from its surface up to the sea
            // level (`column::generate_solid`), so the surface is under water
            // exactly when it is below the sea level.
            assert_eq!(texel.sea, surface < cfg.sea_level_m, "{d}");
            land += usize::from(!texel.sea);
        }
        assert!(
            (2_000..8_000).contains(&land),
            "{land} of 10,000 on land: the sample is not all one thing"
        );
    }

    /// A mark counts when the walker comes within 8 m of it, and only a
    /// mark set 200 m or more away does (`starter-missions` G3); it reads
    /// back from its record.
    #[test]
    fn a_far_mark_is_reached_within_eight_metres() {
        let r = 4_800.0;
        let home = Vec3::Y;
        // 250 m along the ground to the east.
        let east = |m: f32| Vec3::new((m / r).sin(), (m / r).cos(), 0.0);
        let far = Mark::set(east(250.0) * r, home * r, r).unwrap();
        assert!((far.set_from_m - 250.0).abs() < 0.5, "{}", far.set_from_m);
        assert!(!far.reached_far(home, r), "not yet");
        assert!(!far.reached_far(east(240.0), r), "10 m short");
        assert!(far.reached_far(east(244.0), r), "6 m short");
        assert!(far.reached_far(east(250.0) * 3.0, r), "any length");
        let near = Mark::set(east(150.0), home, r).unwrap();
        assert!(!near.reached_far(east(150.0), r), "set too near");
        assert!(Mark::set(Vec3::ZERO, home, r).is_none());
        let body = ron::to_string(&far).unwrap();
        assert_eq!(ron::from_str::<Mark>(&body).unwrap(), far);
    }

    /// Four waypoints at most, oldest dropped first; a schema 1 body reads
    /// as none or one (owner, 2026-11-16).
    #[test]
    fn a_fifth_waypoint_drops_the_first() {
        let mark = |x: f32| Mark::set(Vec3::new(x, 1.0, 0.0), Vec3::Y, 4_800.0).unwrap();
        let mut marks = Vec::new();
        for k in 0..5 {
            add_mark(&mut marks, mark(k as f32 * 0.01));
        }
        assert_eq!(marks.len(), MARKS_MAX);
        assert_eq!(marks[0].at, mark(0.01).at, "the first is gone");
        assert_eq!(marks[3].at, mark(0.04).at);
        let body = ron::to_string(&marks).unwrap();
        assert_eq!(read_marks(&body), Some(marks));
        let one = ron::to_string(&Some(mark(0.0))).unwrap();
        let first = Mark {
            number: 1,
            ..mark(0.0)
        };
        assert_eq!(read_marks(&one), Some(vec![first]));
        assert_eq!(
            read_marks(&ron::to_string(&None::<Mark>).unwrap()),
            Some(vec![])
        );
    }

    /// `map-rivers`: a texel's river is the column's river water, the
    /// generator's own test, on seeded directions near rivers and off them;
    /// never on the sea; and a pixel's cover is all of it where the point
    /// and its neighbours are all river, none where the banks are far.
    #[test]
    fn the_map_river_is_the_column_river() {
        let cfg = TerrainConfig::for_version(10).expect("version 10");
        let settings = cfg.rivers.expect("rivers");
        let rivers = crate::rivers::map(&cfg, &settings);
        // Points along the rivers' lines as well as anywhere, so the sample
        // holds water.
        let along = rivers
            .rivers()
            .step_by(7)
            .take(600)
            .map(|k| rivers.describe(k).0);
        let mut wet = 0;
        for d in directions(2_000).chain(along) {
            let texel = base_texel(&cfg, d);
            let water = crate::rivers::water(&cfg, d);
            assert_eq!(
                texel.river == RIVER_FULL,
                !texel.sea && river_water(&cfg, d),
                "{d}"
            );
            if crate::settlement::ground::top(&cfg, d).is_none() && !texel.sea {
                assert_eq!(texel.river == RIVER_FULL, !water.is_empty(), "{d}");
            }
            assert!(!(texel.sea && texel.river > 0), "{d}");
            wet += usize::from(texel.river > 0);
            let cover = river_cover(&cfg, d, 2.833, 2.833);
            if texel.river == 0 && !crate::rivers::near(&cfg, d, 3.0) {
                assert_eq!(cover, 0, "{d}");
            }
        }
        assert!(wet > 100, "{wet} wet points: the sample holds rivers");
    }

    /// Waypoints are numbered W1, W2 ... in the order placed by the lowest
    /// free number: a removal leaves the others' numbers, the next one takes
    /// the gap, and a fifth takes the number of the oldest it drops
    /// (`waypoints` decision 3).
    #[test]
    fn waypoints_take_the_lowest_free_number() {
        let mark = |x: f32| Mark::set(Vec3::new(x, 1.0, 0.0), Vec3::Y, 4_800.0).unwrap();
        let numbers = |marks: &[Mark]| marks.iter().map(|m| m.number).collect::<Vec<_>>();
        let mut marks = Vec::new();
        for k in 0..3 {
            add_mark(&mut marks, mark(k as f32 * 0.01));
        }
        assert_eq!(numbers(&marks), [1, 2, 3]);
        assert_eq!(marks[0].label(), "W1");
        marks.remove(1);
        assert_eq!(numbers(&marks), [1, 3], "W3 keeps its number");
        add_mark(&mut marks, mark(0.1));
        assert_eq!(numbers(&marks), [1, 3, 2], "the next is W2");
        add_mark(&mut marks, mark(0.2));
        assert_eq!(numbers(&marks), [1, 3, 2, 4]);
        add_mark(&mut marks, mark(0.3));
        assert_eq!(numbers(&marks), [3, 2, 4, 1], "the fifth takes W1's");
        assert_eq!(marks[3].label(), "W1");
    }

    /// A schema 2 body has no numbers: it reads numbered in order, oldest
    /// W1. A schema 3 body reads back as written, and an older build that
    /// knows no number still reads its waypoints (`waypoints` decision 4).
    #[test]
    fn old_waypoint_records_read_numbered_in_order() {
        /// The schema 2 shape, as the build before numbers wrote it.
        #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
        struct OldMark {
            at: [f32; 3],
            set_from_m: f32,
        }
        let old = vec![
            OldMark {
                at: [1.0, 0.0, 0.0],
                set_from_m: 250.0,
            },
            OldMark {
                at: [0.0, 1.0, 0.0],
                set_from_m: 30.0,
            },
        ];
        let body = ron::to_string(&old).unwrap();
        let read = read_marks(&body).expect("a schema 2 body reads");
        assert_eq!(read.iter().map(|m| m.number).collect::<Vec<_>>(), [1, 2]);
        assert_eq!(read[0].at, old[0].at);
        assert_eq!(read[1].set_from_m, 30.0);
        // Two with one number are numbered again in order.
        let mut clash = read.clone();
        clash[1].number = 1;
        let clash = read_marks(&ron::to_string(&clash).unwrap()).unwrap();
        assert_eq!(clash.iter().map(|m| m.number).collect::<Vec<_>>(), [1, 2]);
        // A schema 3 body keeps a gap in the numbers.
        let mut gap = read.clone();
        gap[0].number = 3;
        let three = ron::to_string(&gap).unwrap();
        assert_eq!(read_marks(&three), Some(gap));
        // An older build reading schema 3 passes over the number.
        let older: Vec<OldMark> = ron::from_str(&three).expect("an older build reads it");
        assert_eq!(older, old);
    }

    /// A direction's length does not matter.
    #[test]
    fn any_length_of_direction_is_the_same_place() {
        let cfg = TerrainConfig::TENEBRIS;
        for d in directions(100) {
            assert_eq!(base_texel(&cfg, d), base_texel(&cfg, d * 4_800.0));
        }
    }
}
