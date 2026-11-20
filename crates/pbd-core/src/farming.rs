//! Farming's rules (`farming-and-smelting` decisions 2 to 5, after M8): what
//! tills, which crops there are and where they grow, how far along a crop is
//! at a given world time, and what a plot record holds.
//!
//! A crop's stage is DERIVED: the time it was planted and the time now say
//! everything, so nothing is written while a field grows, and a field left
//! while the game is closed has not grown, because world time does not pass.

use crate::daylight::DAY_S;
use crate::goods::Good;
use crate::planet_gen::Biome;
use crate::terrain::Material;
use serde::{Deserialize, Serialize};

/// A climate a crop grows in outdoors (M8's "climate" column).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Climate {
    Temperate,
    Hot,
    Cold,
}

impl Climate {
    /// A biome's climate, or `None` where nothing grows (the sea floor).
    pub fn of(biome: Biome) -> Option<Climate> {
        Some(match biome {
            Biome::Fields | Biome::Swamp | Biome::Beach => Climate::Temperate,
            Biome::Desert | Biome::Jungle => Climate::Hot,
            Biome::Tundra | Biome::Mountains => Climate::Cold,
            Biome::Ocean => return None,
        })
    }

    pub fn name(self) -> crate::locale::Localized {
        crate::locale::tr(match self {
            Climate::Temperate => "farming.climate.temperate",
            Climate::Hot => "farming.climate.hot",
            Climate::Cold => "farming.climate.cold",
        })
    }
}

/// One crop of the roster.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CropRow {
    /// Its saved name, in a plot record.
    pub code: &'static str,
    pub climates: &'static [Climate],
    pub seed: Good,
    pub produce: Good,
}

/// The roster: M8's base set. **Append only**; a plot names its crop by
/// code, so the order is memory and never a save.
impl CropRow {
    /// Its name: `crop.<code>.name` in the locale file.
    pub fn name(&self) -> crate::locale::Localized {
        crate::locale::tr(&format!("crop.{}.name", self.code))
    }
}

pub const CROPS: &[CropRow] = &[
    CropRow {
        code: "wheat",
        climates: &[Climate::Temperate],
        seed: Good::WHEAT_SEED,
        produce: Good::WHEAT,
    },
    CropRow {
        code: "turnip",
        climates: &[Climate::Temperate, Climate::Cold],
        seed: Good::TURNIP_SEED,
        produce: Good::TURNIP,
    },
    CropRow {
        code: "frost-kale",
        climates: &[Climate::Cold, Climate::Temperate],
        seed: Good::FROST_KALE_SEED,
        produce: Good::FROST_KALE,
    },
    CropRow {
        code: "chili",
        climates: &[Climate::Hot],
        seed: Good::CHILI_SEED,
        produce: Good::CHILI,
    },
];

/// A crop, by its row in [`CROPS`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Crop(u8);

impl Crop {
    pub const WHEAT: Crop = Crop(0);
    pub const TURNIP: Crop = Crop(1);
    pub const FROST_KALE: Crop = Crop(2);
    pub const CHILI: Crop = Crop(3);

    pub fn all() -> impl Iterator<Item = Crop> {
        (0..CROPS.len() as u8).map(Crop)
    }

    pub fn row(self) -> &'static CropRow {
        &CROPS[self.0 as usize]
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }

    pub fn by_code(code: &str) -> Option<Crop> {
        Crop::all().find(|c| c.row().code == code)
    }

    /// The crop a seed grows, or `None` for a good that is not a seed.
    pub fn of_seed(good: Good) -> Option<Crop> {
        Crop::all().find(|c| c.row().seed == good)
    }

    pub fn grows_in(self, climate: Option<Climate>) -> bool {
        climate.is_some_and(|c| self.row().climates.contains(&c))
    }
}

/// One crop's tunables, in `farming.ron`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CropTuning {
    /// Game days from sowing to ripe; a day is `DAY_S` seconds of world time.
    pub days: f32,
    /// Produce a ripe crop gives, items.
    pub produce: u16,
    /// Seeds a ripe crop gives back, items.
    pub seeds: u16,
}

/// Farming's tunables (`assets/config/farming.ron`).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FarmSettings {
    pub wheat: CropTuning,
    pub turnip: CropTuning,
    pub frost_kale: CropTuning,
    pub chili: CropTuning,
    /// One grassy cell in this many grows a wild plant of a crop that suits
    /// its climate (decision 14), cells.
    pub wild_one_in: u32,
    /// Seeds a wild plant gives when picked, items (and one of its produce).
    pub wild_seeds: u16,
    /// One grassy cell in this many turns up a seed when it is dug
    /// (decision 14), cells.
    pub grass_seed_one_in: u32,
}

impl Default for FarmSettings {
    /// M8's table: days and yield; two seeds back each.
    fn default() -> Self {
        let tuning = |days, produce| CropTuning {
            days,
            produce,
            seeds: 2,
        };
        Self {
            wheat: tuning(6.0, 3),
            turnip: tuning(4.0, 2),
            frost_kale: tuning(5.0, 2),
            chili: tuning(5.0, 4),
            wild_one_in: 40,
            wild_seeds: 2,
            grass_seed_one_in: 4,
        }
    }
}

impl FarmSettings {
    pub fn of(&self, crop: Crop) -> &CropTuning {
        match crop.0 {
            0 => &self.wheat,
            1 => &self.turnip,
            2 => &self.frost_kale,
            _ => &self.chili,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        for crop in Crop::all() {
            let t = self.of(crop);
            if !(t.days.is_finite() && t.days > 0.0) {
                return Err(format!("{}: days must be above zero", crop.row().code));
            }
            if t.produce == 0 {
                return Err(format!("{}: a ripe crop gives something", crop.row().code));
            }
        }
        if self.wild_one_in == 0 || self.grass_seed_one_in == 0 {
            return Err("wild_one_in and grass_seed_one_in are one in N, N above zero".into());
        }
        if self.wild_seeds == 0 {
            return Err("a wild plant gives at least one seed".into());
        }
        Ok(())
    }

    /// The wild crop growing on a grassy cell of a world with seed
    /// `world_seed`, if one does (decision 14): one cell in `wild_one_in`, a
    /// crop that suits the climate. DERIVED, like the meadow's flowers, so
    /// it is never saved; picking it is (a `wild` record).
    pub fn wild_crop(&self, world_seed: u64, cell: u32, climate: Option<Climate>) -> Option<Crop> {
        let h = mix(world_seed ^ (u64::from(cell) << 1) ^ 0x0a11_d5ee_d000);
        if !h.is_multiple_of(u64::from(self.wild_one_in)) {
            return None;
        }
        let suits: Vec<Crop> = Crop::all().filter(|c| c.grows_in(climate)).collect();
        (!suits.is_empty()).then(|| suits[((h >> 32) % suits.len() as u64) as usize])
    }

    /// The seed a grassy cell turns up when it is dug (decision 14): one
    /// cell in `grass_seed_one_in`, the cell's own seed. The same cell always
    /// answers the same.
    pub fn grass_seed(&self, cell: u32, climate: Option<Climate>) -> Option<Good> {
        let h = mix(u64::from(cell) ^ 0x6a55_5eed);
        h.is_multiple_of(u64::from(self.grass_seed_one_in))
            .then(|| seed_turned_up(cell, climate))
            .flatten()
    }

    /// How far along a crop is, 0 sown to 1 ripe (and beyond, clamped).
    /// A time planted after now (a clock set back) is not started.
    pub fn progress(&self, crop: Crop, planted_s: f64, now_s: f64) -> f32 {
        let span = f64::from(self.of(crop).days) * f64::from(DAY_S);
        ((now_s - planted_s) / span).clamp(0.0, 1.0) as f32
    }

    /// The crop's stage at `now_s`: M8's thresholds on its progress.
    pub fn stage(&self, crop: Crop, planted_s: f64, now_s: f64) -> Stage {
        Stage::at(self.progress(crop, planted_s, now_s))
    }

    /// Game seconds until ripe; zero once it is.
    pub fn left_s(&self, crop: Crop, planted_s: f64, now_s: f64) -> f64 {
        let span = f64::from(self.of(crop).days) * f64::from(DAY_S);
        (planted_s + span - now_s).max(0.0)
    }
}

/// M8's four stage sprites.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stage {
    Sown = 0,
    Sprout = 1,
    Growing = 2,
    Ripe = 3,
}

impl Stage {
    /// M8's `stageOf`: ripe at one, growing from 0.6, a sprout from 0.25.
    pub fn at(progress: f32) -> Stage {
        if progress >= 1.0 {
            Stage::Ripe
        } else if progress >= 0.6 {
            Stage::Growing
        } else if progress >= 0.25 {
            Stage::Sprout
        } else {
            Stage::Sown
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

/// Whether a top block tills (decision 2): the grasses, the earths and sand.
/// Snow, stone, rock and ore do not.
pub fn tillable(material: Material) -> bool {
    matches!(
        material,
        Material::Grass
            | Material::DryGrass
            | Material::JungleGrass
            | Material::Dirt
            | Material::Soil
            | Material::Sand
    )
}

/// Whether tilling this top turns up a seed: a grassy one.
pub fn sprouts_seed(material: Material) -> bool {
    matches!(
        material,
        Material::Grass | Material::DryGrass | Material::JungleGrass
    )
}

fn mix(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

/// The seed a grassy cell turns up when it is tilled: a crop that grows in
/// its climate, picked by the cell's key, so a cell always gives the same.
/// `None` where nothing grows.
pub fn seed_turned_up(cell: u32, climate: Option<Climate>) -> Option<Good> {
    let suits: Vec<Crop> = Crop::all().filter(|c| c.grows_in(climate)).collect();
    if suits.is_empty() {
        return None;
    }
    let pick = mix(u64::from(cell) ^ 0x5eed_f1e1d) % suits.len() as u64;
    Some(suits[pick as usize].row().seed)
}

/// The record kind a picked wild plant is stored as (decision 14): its id
/// is the cell's key, and its body says when.
pub const WILD_KIND: &str = "wild";
pub const WILD_SCHEMA: u32 = 1;

/// A picked wild plant's record body.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Picked {
    pub picked_s: f64,
}

/// The record kind a planted cell is stored as.
pub const PLOT_KIND: &str = "plot";
/// Its schema.
pub const PLOT_SCHEMA: u32 = 1;

/// A plot record's body (decision 4): the tilled layer, the crop on it by
/// code (none once harvested or uprooted), and the world time it was sown.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Plot {
    pub layer: u16,
    pub crop: Option<String>,
    pub planted_s: f64,
}

impl Plot {
    pub fn sown(layer: u16, crop: Crop, planted_s: f64) -> Self {
        Self {
            layer,
            crop: Some(crop.row().code.to_string()),
            planted_s,
        }
    }

    pub fn bare(layer: u16) -> Self {
        Self {
            layer,
            crop: None,
            planted_s: 0.0,
        }
    }

    /// The crop growing here, if one is and this build knows it.
    pub fn crop(&self) -> Option<Crop> {
        self.crop.as_deref().and_then(Crop::by_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: f64 = DAY_S as f64;

    /// Wheat, six days to ripe, passes M8's thresholds at the days they
    /// fall on, and stays ripe.
    #[test]
    fn wheat_grows_through_its_stages_on_world_time() {
        let s = FarmSettings::default();
        let at = |days: f64| s.stage(Crop::WHEAT, 100.0, 100.0 + days * DAY);
        assert_eq!(at(0.0), Stage::Sown);
        assert_eq!(at(1.4), Stage::Sown);
        assert_eq!(at(1.5), Stage::Sprout);
        assert_eq!(at(3.0), Stage::Sprout);
        assert_eq!(at(3.6), Stage::Growing);
        assert_eq!(at(6.0), Stage::Ripe);
        assert_eq!(at(60.0), Stage::Ripe);
        // A clock set back before the sowing: not started, never negative.
        assert_eq!(at(-2.0), Stage::Sown);
        assert_eq!(s.left_s(Crop::WHEAT, 0.0, DAY), 5.0 * DAY);
        assert_eq!(s.left_s(Crop::WHEAT, 0.0, 9.0 * DAY), 0.0);
    }

    /// Climates: wheat is a meadow's, chili a desert's, and nothing grows
    /// on the sea floor.
    #[test]
    fn a_crop_grows_only_in_its_climates() {
        let fields = Climate::of(Biome::Fields);
        let tundra = Climate::of(Biome::Tundra);
        let desert = Climate::of(Biome::Desert);
        assert!(Crop::WHEAT.grows_in(fields));
        assert!(!Crop::WHEAT.grows_in(tundra));
        assert!(Crop::FROST_KALE.grows_in(tundra));
        assert!(Crop::CHILI.grows_in(desert) && !Crop::CHILI.grows_in(fields));
        assert!(Crop::all().all(|c| !c.grows_in(Climate::of(Biome::Ocean))));
    }

    /// A tilled meadow cell always turns up the same seed, always one that
    /// grows there, and over many cells every crop that suits shows up.
    #[test]
    fn the_seed_a_cell_turns_up_is_its_own_and_suits_it() {
        let fields = Climate::of(Biome::Fields);
        let mut seen = std::collections::BTreeSet::new();
        for cell in 0..400u32 {
            let seed = seed_turned_up(cell, fields).unwrap();
            assert_eq!(seed_turned_up(cell, fields), Some(seed));
            let crop = Crop::of_seed(seed).unwrap();
            assert!(crop.grows_in(fields));
            seen.insert(crop.index());
        }
        assert_eq!(seen.len(), 3, "wheat, turnip and kale on a meadow");
        assert_eq!(seed_turned_up(7, None), None);
    }

    #[test]
    fn grass_and_earth_till_and_stone_does_not() {
        for m in [Material::Grass, Material::Dirt, Material::Sand] {
            assert!(tillable(m));
        }
        for m in [
            Material::Snow,
            Material::Stone,
            Material::Rock,
            Material::Ore,
            Material::Tilled,
            Material::Forge,
        ] {
            assert!(!tillable(m), "{m:?}");
        }
        assert!(sprouts_seed(Material::Grass) && !sprouts_seed(Material::Dirt));
    }

    /// Every crop names a seed and a produce, each a different good, and its
    /// code reads back.
    #[test]
    fn the_roster_is_consistent() {
        for crop in Crop::all() {
            let row = crop.row();
            assert_eq!(Crop::by_code(row.code), Some(crop));
            assert_eq!(Crop::of_seed(row.seed), Some(crop));
            assert_ne!(row.seed, row.produce);
        }
        assert_eq!(Crop::of_seed(Good::COAL), None);
        FarmSettings::default().validate().unwrap();
    }

    /// Wild plants: about one grassy cell in forty, always the same cells
    /// for a seed, each a crop that suits; other seeds put them elsewhere.
    /// Grass seeds: about one cell in four.
    #[test]
    fn wild_plants_and_grass_seeds_are_derived_and_rare() {
        let s = FarmSettings::default();
        let fields = Climate::of(Biome::Fields);
        let wild: Vec<u32> = (0..40_000)
            .filter(|&c| s.wild_crop(7, c, fields).is_some())
            .collect();
        assert!((800..1200).contains(&wild.len()), "{}", wild.len());
        for &c in &wild[..20] {
            assert!(s.wild_crop(7, c, fields).unwrap().grows_in(fields));
        }
        let other = (0..40_000).filter(|&c| s.wild_crop(8, c, fields).is_some());
        assert_ne!(other.take(20).collect::<Vec<_>>(), wild[..20].to_vec());
        assert_eq!(
            s.wild_crop(7, wild[0], None),
            None,
            "nothing on the sea floor"
        );
        let seeds = (0..4000)
            .filter(|&c| s.grass_seed(c, fields).is_some())
            .count();
        assert!((800..1200).contains(&seeds), "{seeds}");
    }

    /// A plot body round-trips through RON, and a crop this build lacks
    /// reads as nothing growing rather than as another crop.
    #[test]
    fn a_plot_round_trips_and_an_unknown_crop_is_nothing() {
        let plot = Plot::sown(61, Crop::TURNIP, 1234.5);
        let text = ron::to_string(&plot).unwrap();
        let back: Plot = ron::from_str(&text).unwrap();
        assert_eq!(back, plot);
        assert_eq!(back.crop(), Some(Crop::TURNIP));
        let odd = Plot {
            crop: Some("moon-melon".into()),
            ..plot
        };
        assert_eq!(odd.crop(), None);
    }
}
