//! Breaking a block: how long a material takes with the tool in hand, and the
//! hold that counts toward it.
//!
//! The rule is Minecraft's and Tenebris's (`tenebris-core/src/mining.rs`
//! `break_time`, `tenebris-client/src/interact.rs:578-596`): a material wants
//! one kind of tool, the right tool breaks it in its base time and any other
//! in a multiple of it, and the time counts only while the button is held on
//! the same block. The numbers are tuned against Minecraft's own
//! (`openspec/changes/fishing-and-equipment/design.md` section 3) and live in
//! `assets/config/dig.ron`.
//!
//! Engine-free: the app hands [`Breaking::step`] the target, the button and
//! the frame's time, and takes the block when it says so.

use crate::inventory::Tool;
use crate::terrain::Material;
use serde::{Deserialize, Serialize};

/// What a material is, as far as breaking it goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// Grass, soil, dirt, sand and snow: the shovel's.
    Soft,
    /// Stone: the pickaxe's.
    Stone,
    /// Rock outcrops: the pickaxe's, slower.
    Rock,
    /// Ore: the pickaxe's, slowest.
    Ore,
    /// Something placed on the ground, a torch: any tool but the rod.
    Placed,
}

impl Class {
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
            | Material::Snow => Class::Soft,
            Material::Stone => Class::Stone,
            Material::Rock => Class::Rock,
            Material::Ore => Class::Ore,
            Material::Torch => Class::Placed,
            Material::Air | Material::Water => return None,
        })
    }

    /// The tool that breaks it at its base time. `None` for a class every
    /// tool breaks alike.
    pub fn right_tool(self) -> Option<Tool> {
        match self {
            Class::Soft => Some(Tool::Shovel),
            Class::Stone | Class::Rock | Class::Ore => Some(Tool::Pickaxe),
            Class::Placed => None,
        }
    }
}

/// The break times, in `dig.ron`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DigSettings {
    /// Soft ground with the shovel, s.
    pub soft_s: f32,
    /// Stone with the pickaxe, s.
    pub stone_s: f32,
    /// Rock with the pickaxe, s.
    pub rock_s: f32,
    /// Ore with the pickaxe, s.
    pub ore_s: f32,
    /// A placed thing with any tool, s.
    pub placed_s: f32,
    /// Any tool but the right one takes this many times as long.
    pub wrong_tool: f32,
    /// With the button still held, the next block starts this long after the
    /// last one broke, s.
    pub between_s: f32,
}

impl Default for DigSettings {
    fn default() -> Self {
        Self {
            soft_s: 0.5,
            stone_s: 1.2,
            rock_s: 1.6,
            ore_s: 2.0,
            placed_s: 0.1,
            wrong_tool: 4.0,
            between_s: 0.15,
        }
    }
}

impl DigSettings {
    pub fn validate(&self) -> Result<(), String> {
        let times = [
            ("soft_s", self.soft_s),
            ("stone_s", self.stone_s),
            ("rock_s", self.rock_s),
            ("ore_s", self.ore_s),
            ("placed_s", self.placed_s),
        ];
        for (name, value) in times {
            if !(value.is_finite() && value > 0.0 && value <= 60.0) {
                return Err(format!("dig.{name} must be in (0, 60] s, got {value}"));
            }
        }
        if !(self.wrong_tool.is_finite() && self.wrong_tool >= 1.0 && self.wrong_tool <= 20.0) {
            return Err(format!(
                "dig.wrong_tool must be in [1, 20], got {}",
                self.wrong_tool
            ));
        }
        if !(self.between_s.is_finite() && (0.0..=2.0).contains(&self.between_s)) {
            return Err(format!(
                "dig.between_s must be in [0, 2] s, got {}",
                self.between_s
            ));
        }
        Ok(())
    }

    fn base(&self, class: Class) -> f32 {
        match class {
            Class::Soft => self.soft_s,
            Class::Stone => self.stone_s,
            Class::Rock => self.rock_s,
            Class::Ore => self.ore_s,
            Class::Placed => self.placed_s,
        }
    }

    /// How long `material` takes to break with `tool`, s. `None` when it
    /// never breaks: the rod breaks nothing, and air and water are not
    /// blocks.
    pub fn secs(&self, material: Material, tool: Tool) -> Option<f32> {
        if !tool.digs() {
            return None;
        }
        let class = Class::of(material)?;
        let base = self.base(class);
        Some(match class.right_tool() {
            Some(right) if right != tool => base * self.wrong_tool,
            _ => base,
        })
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

    #[test]
    fn the_right_tool_takes_the_base_time_and_any_other_takes_four() {
        let s = DigSettings::default();
        assert_eq!(s.secs(Material::Dirt, Tool::Shovel), Some(0.5));
        assert_eq!(s.secs(Material::Dirt, Tool::Pickaxe), Some(2.0));
        assert_eq!(s.secs(Material::Dirt, Tool::Axe), Some(2.0));
        assert_eq!(s.secs(Material::Stone, Tool::Pickaxe), Some(1.2));
        assert_eq!(s.secs(Material::Stone, Tool::Shovel), Some(4.8));
        assert_eq!(s.secs(Material::Rock, Tool::Pickaxe), Some(1.6));
        assert_eq!(s.secs(Material::Ore, Tool::Pickaxe), Some(2.0));
        assert_eq!(s.secs(Material::Torch, Tool::Axe), Some(0.1));
        assert_eq!(s.secs(Material::Torch, Tool::Shovel), Some(0.1));
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
            wrong_tool: 0.5,
            ..Default::default()
        };
        assert!(bad.validate().is_err());
        let bad = DigSettings {
            stone_s: 0.0,
            ..Default::default()
        };
        assert!(bad.validate().is_err());
    }
}
