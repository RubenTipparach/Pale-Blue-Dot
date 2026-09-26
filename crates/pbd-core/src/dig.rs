//! Breaking a block: how long a material takes with the tool in hand, and the
//! hold that counts toward it.
//!
//! The times are a MATRIX, one per tool for each class of material, so every
//! tool has its own character against dirt, rock and wood rather than one
//! right tool and a flat penalty for the rest. The time counts only while the
//! button is held on the same block, which is Minecraft's rule and Tenebris's
//! (`tenebris-client/src/interact.rs:578-596`). The numbers are tuned against
//! Minecraft's own (`openspec/changes/fishing-and-equipment/design.md`
//! section 3) and live in `assets/config/dig.ron`.
//!
//! Engine-free: the app hands [`Breaking::step`] the target, the button and
//! the frame's time, and takes the block when it says so.

use crate::inventory::Tool;
use crate::terrain::Material;
use serde::{Deserialize, Serialize};

/// What a material is, as far as breaking it goes: a row of the matrix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// Grass, soil, dirt, sand and snow.
    Dirt,
    /// Stone.
    Stone,
    /// Rock outcrops, harder than stone.
    Rock,
    /// Ore, the hardest.
    Ore,
    /// Wood. No material is wood until trees can be felled
    /// (`fishing-and-equipment` design section 5); the row is ready for it.
    Wood,
    /// Something placed on the ground, a torch.
    Placed,
}

impl Class {
    /// Every row, in the order `dig.ron` lists them.
    pub const ALL: [Class; 6] = [
        Class::Dirt,
        Class::Stone,
        Class::Rock,
        Class::Ore,
        Class::Wood,
        Class::Placed,
    ];

    /// The class of a material, or `None` for what is never broken: air and
    /// water, which the aim ray passes through. The match is exhaustive, so a
    /// new material cannot compile without being given a class or refused one.
    pub fn of(material: Material) -> Option<Self> {
        Some(match material {
            Material::Grass
            | Material::DryGrass
            | Material::JungleGrass
            | Material::Soil
            | Material::Dirt
            | Material::Sand
            | Material::Snow => Class::Dirt,
            Material::Stone => Class::Stone,
            Material::Rock => Class::Rock,
            Material::Ore => Class::Ore,
            Material::Torch => Class::Placed,
            Material::Air | Material::Water => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Class::Dirt => "dirt",
            Class::Stone => "stone",
            Class::Rock => "rock",
            Class::Ore => "ore",
            Class::Wood => "wood",
            Class::Placed => "placed",
        }
    }
}

/// One row of the matrix: how long each digging tool takes, s.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolTimes {
    pub shovel: f32,
    pub pickaxe: f32,
    pub axe: f32,
}

impl ToolTimes {
    const fn new(shovel: f32, pickaxe: f32, axe: f32) -> Self {
        Self {
            shovel,
            pickaxe,
            axe,
        }
    }

    /// This tool's time, or `None` for a tool that does not dig: the rod.
    pub fn of(&self, tool: Tool) -> Option<f32> {
        match tool {
            Tool::Shovel => Some(self.shovel),
            Tool::Pickaxe => Some(self.pickaxe),
            Tool::Axe => Some(self.axe),
            Tool::Rod => None,
        }
    }

    /// The tool that breaks this row fastest.
    pub fn best(&self) -> Tool {
        [Tool::Shovel, Tool::Pickaxe, Tool::Axe]
            .into_iter()
            .min_by(|a, b| self.of(*a).unwrap().total_cmp(&self.of(*b).unwrap()))
            .expect("three digging tools")
    }
}

/// The break-time matrix and the pause between blocks, in `dig.ron`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DigSettings {
    pub dirt: ToolTimes,
    pub stone: ToolTimes,
    pub rock: ToolTimes,
    pub ore: ToolTimes,
    pub wood: ToolTimes,
    pub placed: ToolTimes,
    /// With the button still held, the next block starts this long after the
    /// last one broke, s.
    pub between_s: f32,
}

impl Default for DigSettings {
    fn default() -> Self {
        Self {
            dirt: ToolTimes::new(0.5, 1.5, 1.2),
            stone: ToolTimes::new(4.0, 1.2, 3.0),
            rock: ToolTimes::new(5.0, 1.6, 4.0),
            ore: ToolTimes::new(6.5, 2.0, 5.5),
            wood: ToolTimes::new(2.5, 2.0, 0.6),
            placed: ToolTimes::new(0.1, 0.1, 0.1),
            between_s: 0.15,
        }
    }
}

impl DigSettings {
    /// A row of the matrix.
    pub fn row(&self, class: Class) -> &ToolTimes {
        match class {
            Class::Dirt => &self.dirt,
            Class::Stone => &self.stone,
            Class::Rock => &self.rock,
            Class::Ore => &self.ore,
            Class::Wood => &self.wood,
            Class::Placed => &self.placed,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        for class in Class::ALL {
            let row = self.row(class);
            for (tool, value) in [
                ("shovel", row.shovel),
                ("pickaxe", row.pickaxe),
                ("axe", row.axe),
            ] {
                if !(value.is_finite() && value > 0.0 && value <= 60.0) {
                    return Err(format!(
                        "dig.{}.{tool} must be in (0, 60] s, got {value}",
                        class.name()
                    ));
                }
            }
        }
        if !(self.between_s.is_finite() && (0.0..=2.0).contains(&self.between_s)) {
            return Err(format!(
                "dig.between_s must be in [0, 2] s, got {}",
                self.between_s
            ));
        }
        Ok(())
    }

    /// How long `material` takes to break with `tool`, s. `None` when it
    /// never breaks: the rod breaks nothing, and air and water are not
    /// blocks.
    pub fn secs(&self, material: Material, tool: Tool) -> Option<f32> {
        self.row(Class::of(material)?).of(tool)
    }
}

/// A block being broken: which one, how long it needs, and how long it has
/// had. `C` is whatever names a block to the caller (the app's cell and
/// layer), compared for equality only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Breaking<C> {
    target: Option<(C, f32)>,
    elapsed_s: f32,
    /// Time left before the next block may start, after one broke.
    pause_s: f32,
}

impl<C> Default for Breaking<C> {
    fn default() -> Self {
        Self {
            target: None,
            elapsed_s: 0.0,
            pause_s: 0.0,
        }
    }
}

/// What a frame of holding did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Step<C> {
    /// Nothing is being broken.
    Idle,
    /// This block is this far along, 0 to 1.
    Breaking(C, f32),
    /// This block is done: take it.
    Broken(C),
}

/// How many crack stages the overlay has: Minecraft's `destroy_stage_0..9`.
pub const STAGES: u8 = 10;

/// The crack stage to show at `progress` (0 to 1): stage `k` while progress
/// is in `[k/10, (k+1)/10)`, and the last stage at 1.
pub fn stage(progress: f32) -> u8 {
    if !progress.is_finite() || progress <= 0.0 {
        return 0;
    }
    ((progress * STAGES as f32) as u8).min(STAGES - 1)
}

impl<C: Copy + PartialEq> Breaking<C> {
    /// Progress on the current block, 0 to 1, or `None` when nothing is
    /// being broken.
    pub fn progress(&self) -> Option<(C, f32)> {
        self.target
            .map(|(block, secs)| (block, (self.elapsed_s / secs).clamp(0.0, 1.0)))
    }

    /// One frame. `aim` is the block under the reticle and the time it needs
    /// with the tool in hand (`None` for nothing breakable, or the rod);
    /// `held` is the use button. Letting go, or aiming at another block,
    /// starts again from nought.
    pub fn step(&mut self, dt: f32, held: bool, aim: Option<(C, f32)>, between_s: f32) -> Step<C> {
        let dt = if dt.is_finite() { dt.max(0.0) } else { 0.0 };
        self.pause_s = (self.pause_s - dt).max(0.0);
        let Some((block, secs)) = aim.filter(|_| held) else {
            self.target = None;
            self.elapsed_s = 0.0;
            // The pause is only between blocks of one hold.
            if !held {
                self.pause_s = 0.0;
            }
            return Step::Idle;
        };
        if self.pause_s > 0.0 {
            self.target = None;
            self.elapsed_s = 0.0;
            return Step::Idle;
        }
        match self.target {
            Some((current, _)) if current == block => self.elapsed_s += dt,
            _ => {
                self.target = Some((block, secs));
                self.elapsed_s = 0.0;
            }
        }
        if !(secs.is_finite() && secs > 0.0) || self.elapsed_s >= secs {
            self.target = None;
            self.elapsed_s = 0.0;
            self.pause_s = between_s;
            return Step::Broken(block);
        }
        Step::Breaking(block, self.elapsed_s / secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each tool takes its own time from the material's row.
    #[test]
    fn each_tool_takes_its_own_time_from_the_row() {
        let s = DigSettings::default();
        assert_eq!(s.secs(Material::Dirt, Tool::Shovel), Some(0.5));
        assert_eq!(s.secs(Material::Dirt, Tool::Pickaxe), Some(1.5));
        assert_eq!(s.secs(Material::Dirt, Tool::Axe), Some(1.2));
        assert_eq!(s.secs(Material::Grass, Tool::Shovel), Some(0.5));
        assert_eq!(s.secs(Material::Snow, Tool::Axe), Some(1.2));
        assert_eq!(s.secs(Material::Stone, Tool::Pickaxe), Some(1.2));
        assert_eq!(s.secs(Material::Stone, Tool::Shovel), Some(4.0));
        assert_eq!(s.secs(Material::Stone, Tool::Axe), Some(3.0));
        assert_eq!(s.secs(Material::Rock, Tool::Pickaxe), Some(1.6));
        assert_eq!(s.secs(Material::Ore, Tool::Pickaxe), Some(2.0));
        assert_eq!(s.secs(Material::Ore, Tool::Shovel), Some(6.5));
        assert_eq!(s.secs(Material::Torch, Tool::Axe), Some(0.1));
        assert_eq!(s.row(Class::Wood).of(Tool::Axe), Some(0.6));
    }

    /// The design's bolded cells: the shovel for dirt, the pickaxe for stone,
    /// rock and ore, the axe for wood. A retune that made the pickaxe the
    /// best shovel fails here rather than in somebody's hands.
    #[test]
    fn each_row_has_the_best_tool_the_design_names() {
        let s = DigSettings::default();
        assert_eq!(s.row(Class::Dirt).best(), Tool::Shovel);
        for class in [Class::Stone, Class::Rock, Class::Ore] {
            assert_eq!(s.row(class).best(), Tool::Pickaxe, "{}", class.name());
        }
        assert_eq!(s.row(Class::Wood).best(), Tool::Axe);
        // And every tool is worse than the best somewhere, or it is not a
        // choice worth making.
        for tool in [Tool::Shovel, Tool::Pickaxe, Tool::Axe] {
            assert!(
                Class::ALL.iter().any(|c| s.row(*c).best() == tool),
                "{tool:?} is never the best"
            );
        }
    }

    #[test]
    fn the_rod_breaks_nothing_and_nothing_breaks_air_or_water() {
        let s = DigSettings::default();
        for m in [
            Material::Dirt,
            Material::Stone,
            Material::Torch,
            Material::Ore,
        ] {
            assert_eq!(s.secs(m, Tool::Rod), None, "{m:?}");
        }
        for tool in Tool::ALL {
            assert_eq!(s.secs(Material::Air, tool), None);
            assert_eq!(s.secs(Material::Water, tool), None);
        }
    }

    #[test]
    fn holding_on_one_block_breaks_it_at_its_time() {
        let mut b = Breaking::default();
        let dt = 1.0 / 60.0;
        let mut frames = 0;
        loop {
            frames += 1;
            match b.step(dt, true, Some((7, 0.5)), 0.15) {
                Step::Broken(block) => {
                    assert_eq!(block, 7);
                    break;
                }
                Step::Breaking(block, progress) => {
                    assert_eq!(block, 7);
                    assert!((0.0..1.0).contains(&progress));
                }
                Step::Idle => panic!("idle while held on a block"),
            }
            assert!(frames < 100);
        }
        // Half a second at 60 Hz: the first frame starts the clock, and the
        // sum of thirty sixtieths may land a hair under a half.
        assert!((31..=32).contains(&frames), "{frames} frames");
    }

    #[test]
    fn letting_go_or_looking_away_starts_again() {
        let mut b = Breaking::default();
        for _ in 0..20 {
            b.step(0.01, true, Some((1, 0.5)), 0.15);
        }
        assert!(b.progress().unwrap().1 > 0.3);
        assert_eq!(b.step(0.01, false, Some((1, 0.5)), 0.15), Step::Idle);
        assert_eq!(
            b.step(0.01, true, Some((1, 0.5)), 0.15),
            Step::Breaking(1, 0.0)
        );
        for _ in 0..20 {
            b.step(0.01, true, Some((1, 0.5)), 0.15);
        }
        assert_eq!(
            b.step(0.01, true, Some((2, 0.5)), 0.15),
            Step::Breaking(2, 0.0)
        );
        assert_eq!(b.step(0.01, true, None, 0.15), Step::Idle);
        assert_eq!(b.progress(), None);
    }

    /// With the button still down the next block waits `between_s`; a fresh
    /// press does not.
    #[test]
    fn the_next_block_waits_a_moment() {
        let mut b = Breaking::default();
        let mut broke = false;
        for _ in 0..100 {
            if let Step::Broken(_) = b.step(0.05, true, Some((1, 0.2)), 0.15) {
                broke = true;
                break;
            }
        }
        assert!(broke);
        assert_eq!(b.step(0.05, true, Some((2, 0.2)), 0.15), Step::Idle);
        assert_eq!(b.step(0.05, true, Some((2, 0.2)), 0.15), Step::Idle);
        assert!(matches!(
            b.step(0.06, true, Some((2, 0.2)), 0.15),
            Step::Breaking(2, _)
        ));
        // Let go and press again: no pause.
        let mut b = Breaking::default();
        while !matches!(b.step(0.05, true, Some((1, 0.2)), 0.15), Step::Broken(_)) {}
        b.step(0.0, false, None, 0.15);
        assert!(matches!(
            b.step(0.0, true, Some((2, 0.2)), 0.15),
            Step::Breaking(2, _)
        ));
    }

    #[test]
    fn the_stage_follows_progress_in_tenths() {
        assert_eq!(stage(0.0), 0);
        assert_eq!(stage(0.099), 0);
        assert_eq!(stage(0.1), 1);
        assert_eq!(stage(0.5), 5);
        assert_eq!(stage(0.95), 9);
        assert_eq!(stage(1.0), 9);
        assert_eq!(stage(f32::NAN), 0);
    }

    #[test]
    fn the_defaults_validate_and_nonsense_does_not() {
        DigSettings::default().validate().unwrap();
        let bad = DigSettings {
            between_s: -1.0,
            ..Default::default()
        };
        assert!(bad.validate().is_err());
        let bad = DigSettings {
            stone: ToolTimes::new(4.0, 0.0, 3.0),
            ..Default::default()
        };
        assert!(bad.validate().is_err());
    }
}
