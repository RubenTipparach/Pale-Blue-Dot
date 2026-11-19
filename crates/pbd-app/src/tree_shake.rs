//! The tree being chopped or punched shakes (`tree-punch-shake` design 3, 4):
//! the tree worked now and the time of its last hit, which the planet uniform
//! carries to the near tree's vertex shader (`tree_shake` in
//! `planet_surface.wgsl`). Each hit sets the tree swaying from its base up,
//! a damped sway on top of the wind's that settles a little after the last.
//!
//! Drawn only: no rule reads it. The mid cards and far billboards do not
//! shake; the worked tree is always within reach, so always the near model.

use crate::planet::PlanetFine;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

/// How far the crown moves at the hit's peak, metres (design 3).
pub const SHAKE_M: f32 = 0.12;
/// How fast it sways, hertz.
pub const SHAKE_HZ: f32 = 9.0;
/// How long a hit's sway lasts, seconds: it is still after this.
pub const SHAKE_S: f32 = 0.6;
/// How much more the leaves move than the wood.
pub const SHAKE_LEAF: f32 = 1.3;
/// While the button is held on a tree, a hit this often, seconds (design 4).
pub const HOLD_HIT_S: f32 = 0.5;

/// The tree being worked and when it was last hit.
#[derive(Resource, Clone, Copy, Default, ExtractResource, PartialEq, Debug)]
pub struct TreeShake {
    /// The worked tree's cell key (`digging::tree_on_ray`), none before the
    /// first hit.
    pub cell: Option<u32>,
    /// Its id as the draw rolls it (the record's `spare.x`).
    pub tree: u32,
    /// The last hit, on the planet clock (`PlanetClock`, the elapsed time).
    pub hit_s: f32,
}

impl TreeShake {
    /// Whether holding the button on `cell` at `now_s` is a hit: the first
    /// moment on a tree, then every [`HOLD_HIT_S`].
    pub fn due(&self, cell: u32, now_s: f32) -> bool {
        self.cell != Some(cell) || now_s - self.hit_s >= HOLD_HIT_S
    }

    /// A hit on the tree at `cell`, whose drawn id is `tree`.
    pub fn hit(&mut self, cell: u32, tree: u32, now_s: f32) {
        *self = Self {
            cell: Some(cell),
            tree,
            hit_s: now_s,
        };
    }

    /// The uniform's lane: x the tree's id, y the hit's time as bits, z one
    /// while a tree has been hit, w spare (`shake` in both shaders' Params).
    pub fn lane(&self) -> UVec4 {
        match self.cell {
            Some(_) => UVec4::new(self.tree, self.hit_s.to_bits(), 1, 0),
            None => UVec4::ZERO,
        }
    }
}

/// The drawn id of the tree standing in `cell`: the finest record's
/// `spare.x`, which the near tree's draw rolls on.
pub fn tree_id(fine: &PlanetFine, cell: u32) -> Option<u32> {
    fine.set
        .finest_records()
        .iter()
        .find(|r| r.key() == cell)
        .map(|r| r.spare[0])
}

/// The hit's sway at `age_s` after it, -1..1 of the full shake: from rest,
/// at [`SHAKE_HZ`], falling to nothing over [`SHAKE_S`]. `tree_shake_wave`
/// in `planet_surface.wgsl`.
pub fn wave(age_s: f32) -> f32 {
    if !(0.0..SHAKE_S).contains(&age_s) {
        return 0.0;
    }
    let k = 1.0 - age_s / SHAKE_S;
    k * k * (age_s * std::f32::consts::TAU * SHAKE_HZ).sin()
}

/// How far a point of tree `id` moves along the push, metres, at `now_s`:
/// `height` its fraction of the tree's height over the base (zero at the
/// foot, one at the top), growing as its square, as the wind's bend does.
/// Zero for any tree but the worked one. The shader's `tree_shake`.
pub fn offset_m(lane: UVec4, id: u32, now_s: f32, height: f32, leaf: bool) -> f32 {
    if lane.z == 0 || lane.x != id {
        return 0.0;
    }
    let h = height.clamp(0.0, 1.2);
    let leaf = if leaf { SHAKE_LEAF } else { 1.0 };
    SHAKE_M * h * h * leaf * wave(now_s - f32::from_bits(lane.y))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SURFACE: &str = include_str!("../../../assets/shaders/planet_surface.wgsl");

    fn wgsl(name: &str) -> f32 {
        SURFACE
            .lines()
            .find_map(|l| l.strip_prefix(&format!("const {name}: f32 = ")))
            .and_then(|v| v.trim_end_matches(';').parse().ok())
            .unwrap_or_else(|| panic!("no const {name} in planet_surface.wgsl"))
    }

    #[test]
    fn the_shader_holds_the_same_shake() {
        assert_eq!(wgsl("TREE_SHAKE_M"), SHAKE_M);
        assert_eq!(wgsl("TREE_SHAKE_HZ"), SHAKE_HZ);
        assert_eq!(wgsl("TREE_SHAKE_S"), SHAKE_S);
        assert_eq!(wgsl("TREE_SHAKE_LEAF"), SHAKE_LEAF);
        // The same sum, line for line.
        for line in [
            "if age < 0.0 || age >= TREE_SHAKE_S { return 0.0; }",
            "let k = 1.0 - age/TREE_SHAKE_S;",
            "return k*k*sin(age*TAU*TREE_SHAKE_HZ);",
            "if params.shake.z == 0u || params.shake.x != id { return w; }",
            "let age = params.settings.z - bitcast<f32>(params.shake.y);",
            "let h = clamp(dot(w - t.base, t.up)/max(kit_f(t.header + 4u)*t.size, 1.0), 0.0, 1.2);",
            "let s = TREE_SHAKE_M*h*h*select(1.0, TREE_SHAKE_LEAF, leaf)*tree_shake_wave(age);",
        ] {
            assert!(SURFACE.contains(line), "planet_surface.wgsl lacks `{line}`");
        }
    }

    #[test]
    fn only_the_worked_tree_shakes_and_it_settles() {
        let mut shake = TreeShake::default();
        assert_eq!(shake.lane(), UVec4::ZERO, "no tree before the first hit");
        assert_eq!(offset_m(shake.lane(), 0, 1.0, 1.0, false), 0.0);
        shake.hit(500, 77, 10.0);
        let lane = shake.lane();
        // Mid-shake the crown moves, about SHAKE_M at its peak.
        let peak = (0..60)
            .map(|i| offset_m(lane, 77, 10.0 + i as f32 * 0.01, 1.0, false).abs())
            .fold(0.0, f32::max);
        assert!(peak > 0.8 * SHAKE_M && peak <= SHAKE_M, "peak {peak}");
        let leaves = (0..60)
            .map(|i| offset_m(lane, 77, 10.0 + i as f32 * 0.01, 1.0, true).abs())
            .fold(0.0, f32::max);
        assert!(leaves > peak, "the leaves move a little more");
        // The foot stays put, and the crown moves more than the middle.
        assert_eq!(offset_m(lane, 77, 10.03, 0.0, false), 0.0);
        assert!(
            offset_m(lane, 77, 10.03, 0.5, false).abs()
                < offset_m(lane, 77, 10.03, 1.0, false).abs()
        );
        // Another tree, at any time, never.
        for i in 0..100 {
            assert_eq!(offset_m(lane, 78, 10.0 + i as f32 * 0.01, 1.0, true), 0.0);
        }
        // Still before the hit, and still once it has settled.
        assert_eq!(offset_m(lane, 77, 9.9, 1.0, true), 0.0);
        for i in 0..100 {
            let t = 10.0 + SHAKE_S + i as f32 * 0.05;
            assert_eq!(offset_m(lane, 77, t, 1.0, true), 0.0, "at {t}");
        }
        // It dies away: the last tenth is a small part of the first.
        let swing = |from: f32| {
            (0..12)
                .map(|i| wave(from + i as f32 * 0.005).abs())
                .fold(0.0, f32::max)
        };
        assert!(swing(0.54) < 0.05 * swing(0.0));
    }

    #[test]
    fn holding_the_button_hits_every_half_second() {
        let mut shake = TreeShake::default();
        assert!(shake.due(5, 0.0), "the first moment on a tree is a hit");
        shake.hit(5, 9, 0.0);
        assert!(!shake.due(5, 0.2));
        assert!(!shake.due(5, 0.49));
        assert!(shake.due(5, 0.5));
        assert!(shake.due(6, 0.2), "a new tree is hit at once");
        // A hit keeps the sway going past the settling time.
        shake.hit(5, 9, 0.5);
        assert_ne!(offset_m(shake.lane(), 9, 0.53, 1.0, false), 0.0);
    }
}
