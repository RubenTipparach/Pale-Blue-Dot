//! Bark dust and falling leaves when a tree is hit or felled
//! (`tree-punch-shake`, "Dust and falling leaves"; owner, 2026-11-22: "add
//! some dust fx + leaves falling fx when a tree despawns or gets hit").
//!
//! - On each stroke impact on a trunk (the moment the shake starts,
//!   `desktop/digging.rs` `shake_on_impact`): a small puff of bark dust at
//!   the trunk on the side facing the eye, and a few leaves let go from the
//!   crown that drift down, swaying, to the ground. Fewer for a fist than
//!   for an axe.
//! - On the fell (the last log, `take_log`): a bigger burst of dust round
//!   the stump and a shower of leaves from where the crown was.
//!
//! The look follows the repository's rules: small square pixel cards in the
//! tree's own colours (its species' leaf and bark means from the tree kit,
//! the same numbers `planet_surface.wgsl` pulls its near tree to), the dust
//! mixed toward a dry-earth tint; no outline; lit as the terrain is
//! (`LitByField`, `LitLikeTerrain`, with an up normal, so a card is as lit
//! as the ground it falls to); nothing glows.
//!
//! Cost: a CPU particle list, capped ([`TreeFxSettings::max_live`]), one
//! mesh per burst rebuilt each frame (a handful of quads), drawn by the
//! field-lit material in the main pass: no new render pass.
//!
//! Frames: a burst's origin is the tree's base, planet-centred in `f64`
//! (`WorldPos`-style, a stored point), its particles `f32` offsets from it
//! along the burst's own up; it is placed in render space through the
//! planet's render centre in `f64` before the cast, each frame, so a rebase
//! moves it with everything else. Drawn only: no rule reads it.

use crate::planet::{PLANET_RADIUS, PlanetFine, PlanetRenderFrame, surface_height};
use bevy::asset::RenderAssetUsages;
use bevy::camera::primitives::Aabb;
use bevy::math::DVec3;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use serde::Deserialize;

/// What happened to a tree, for the dust and leaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeFxEvent {
    /// A stroke's impact on the trunk of the tree at `cell`, by an axe or
    /// not (a fist, or another tool, which works a trunk as hands do).
    Hit { cell: u32, by_axe: bool },
    /// The last log taken: the tree at `cell` is gone.
    Felled { cell: u32 },
}

/// The hits and fells of this frame, pushed by the dig and drained here. A
/// resource rather than a message, so the dig's own test apps need nothing
/// added to run without it.
#[derive(Resource, Default, Debug)]
pub struct TreeFxQueue(pub Vec<TreeFxEvent>);

/// The dust and leaves' tunables (`assets/config/tree_fx.ron`). Counts are
/// particles per event, sizes metres, times seconds, speeds metres a second,
/// colours linear RGB.
#[derive(Resource, Clone, Debug, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TreeFxSettings {
    /// The most particles alive at once, over every burst; an event that
    /// would pass it makes only what fits.
    pub max_live: u32,
    /// Dust specks on a hit by hand (and by a tool that is not an axe).
    pub punch_dust: u32,
    /// Leaves let go on a hit by hand.
    pub punch_leaves: u32,
    /// Dust specks on an axe's hit.
    pub axe_dust: u32,
    /// Leaves let go on an axe's hit.
    pub axe_leaves: u32,
    /// Dust specks round the stump on the fell.
    pub fell_dust: u32,
    /// Leaves showered from the crown on the fell.
    pub fell_leaves: u32,
    /// A dust speck's side, metres.
    pub dust_size_m: f32,
    /// How long a hit's speck lives, seconds: it puffs out and settles,
    /// shrinking to nothing.
    pub dust_life_s: f32,
    /// How long a fell's speck lives, seconds.
    pub fell_dust_life_s: f32,
    /// A hit's speck's speed out from the trunk, m/s (each takes 50..100%).
    pub dust_speed_mps: f32,
    /// A fell's speck's speed out from the stump, m/s.
    pub fell_dust_speed_mps: f32,
    /// How fast a speck slows, per second (exponential drag).
    pub dust_drag: f32,
    /// How fast a speck sinks, m/s².
    pub dust_gravity: f32,
    /// How far the dust is pulled from the bark's colour toward
    /// [`Self::dust_colour`], 0..1: the dust is the bark's and the ground's.
    pub dust_mix: f32,
    /// The dry earth the dust is mixed toward, linear RGB.
    pub dust_colour: [f32; 3],
    /// A leaf's side, metres.
    pub leaf_size_m: f32,
    /// How fast a leaf falls once it is going, m/s: from the crown's middle
    /// (about 4.5 m) this is about two seconds down.
    pub leaf_fall_mps: f32,
    /// How far a leaf sways either side as it falls, metres.
    pub leaf_sway_m: f32,
    /// How fast it sways, hertz.
    pub leaf_sway_hz: f32,
    /// How fast it turns over as it falls (its card narrowing and widening),
    /// hertz.
    pub leaf_flip_hz: f32,
    /// How long a leaf lies on the ground before it fades, seconds.
    pub leaf_rest_s: f32,
    /// How long it takes to fade (shrink away) once rested, seconds.
    pub leaf_fade_s: f32,
    /// The leaf colour against the species' leaf mean, which is the texture's
    /// mean before the near crown's root-to-tip shade.
    pub leaf_shade: f32,
    /// The crown's height band over the tree's base, metres, before the
    /// tree's size: where leaves let go.
    pub crown_low_m: f32,
    pub crown_high_m: f32,
    /// How far out from the trunk a leaf lets go, metres, before the size.
    pub crown_radius_m: f32,
}

impl Default for TreeFxSettings {
    fn default() -> Self {
        Self {
            max_live: 64,
            punch_dust: 4,
            punch_leaves: 2,
            axe_dust: 7,
            axe_leaves: 4,
            fell_dust: 18,
            fell_leaves: 22,
            dust_size_m: 0.06,
            dust_life_s: 0.5,
            fell_dust_life_s: 0.9,
            dust_speed_mps: 1.4,
            fell_dust_speed_mps: 2.2,
            dust_drag: 5.0,
            dust_gravity: 0.6,
            dust_mix: 0.45,
            dust_colour: [0.32, 0.25, 0.17],
            leaf_size_m: 0.09,
            leaf_fall_mps: 2.2,
            leaf_sway_m: 0.35,
            leaf_sway_hz: 0.8,
            leaf_flip_hz: 1.6,
            leaf_rest_s: 0.6,
            leaf_fade_s: 0.4,
            leaf_shade: 0.8,
            crown_low_m: 3.5,
            crown_high_m: 6.0,
            crown_radius_m: 1.4,
        }
    }
}

impl TreeFxSettings {
    pub fn validate(&self) -> Result<(), String> {
        let positive = [
            ("dust_size_m", self.dust_size_m),
            ("dust_life_s", self.dust_life_s),
            ("fell_dust_life_s", self.fell_dust_life_s),
            ("leaf_size_m", self.leaf_size_m),
            ("leaf_fall_mps", self.leaf_fall_mps),
            ("leaf_fade_s", self.leaf_fade_s),
        ];
        for (name, v) in positive {
            if !(v.is_finite() && v > 0.0) {
                return Err(format!("{name} must be greater than zero, got {v}"));
            }
        }
        let non_negative = [
            ("dust_speed_mps", self.dust_speed_mps),
            ("fell_dust_speed_mps", self.fell_dust_speed_mps),
            ("dust_drag", self.dust_drag),
            ("dust_gravity", self.dust_gravity),
            ("leaf_sway_m", self.leaf_sway_m),
            ("leaf_sway_hz", self.leaf_sway_hz),
            ("leaf_flip_hz", self.leaf_flip_hz),
            ("leaf_rest_s", self.leaf_rest_s),
            ("leaf_shade", self.leaf_shade),
            ("crown_low_m", self.crown_low_m),
            ("crown_radius_m", self.crown_radius_m),
        ];
        for (name, v) in non_negative {
            if !(v.is_finite() && v >= 0.0) {
                return Err(format!("{name} must not be negative, got {v}"));
            }
        }
        if !(0.0..=1.0).contains(&self.dust_mix) {
            return Err(format!(
                "dust_mix must be within 0..=1, got {}",
                self.dust_mix
            ));
        }
        if !self
            .dust_colour
            .iter()
            .all(|c| c.is_finite() && (0.0..=1.0).contains(c))
        {
            return Err("dust_colour must be linear RGB within 0..=1".into());
        }
        if !(self.crown_high_m.is_finite() && self.crown_high_m >= self.crown_low_m) {
            return Err("crown_high_m must not be under crown_low_m".into());
        }
        // Tens, not hundreds (the design's cost).
        if !(1..=256).contains(&self.max_live) {
            return Err(format!(
                "max_live must lie within 1 and 256, got {}",
                self.max_live
            ));
        }
        Ok(())
    }

    /// How many specks and leaves an event makes, before the cap.
    pub fn counts(&self, event: TreeFxEvent) -> (u32, u32) {
        match event {
            TreeFxEvent::Hit { by_axe: true, .. } => (self.axe_dust, self.axe_leaves),
            TreeFxEvent::Hit { by_axe: false, .. } => (self.punch_dust, self.punch_leaves),
            TreeFxEvent::Felled { .. } => (self.fell_dust, self.fell_leaves),
        }
    }
}

/// Where a leaf is in its fall.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Leaf {
    /// Falling: the sway's direction across and its phase.
    Falling { sway: Vec3, phase: f32 },
    /// On the ground since this age, seconds.
    Rested(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Dust,
    Leaf(Leaf),
}

/// One speck or leaf, in its burst's frame: metres from the tree's base.
#[derive(Clone, Copy, Debug)]
struct Particle {
    kind: Kind,
    /// Where it is (a leaf: the line it falls down, without the sway).
    at: Vec3,
    velocity: Vec3,
    age: f32,
    /// A speck's whole life; a leaf's is its fall, its rest and its fade.
    life: f32,
    size: f32,
    colour: [f32; 4],
    /// Its height over the base, along the burst's up, where it stops.
    ground: f32,
}

impl Particle {
    /// Where it is drawn and how big, now: a falling leaf swings across its
    /// line, turns over (its card's width) and shrinks once rested; a speck
    /// shrinks over its life. None once it is gone.
    fn drawn(&self, up: Vec3, cfg: &TreeFxSettings) -> Option<(Vec3, Vec2)> {
        match self.kind {
            Kind::Dust => {
                let k = self.age / self.life;
                if k >= 1.0 {
                    return None;
                }
                let s = self.size * (1.0 - k * k);
                Some((self.at, Vec2::splat(s)))
            }
            Kind::Leaf(leaf) => {
                let flip = (self.age * std::f32::consts::TAU * cfg.leaf_flip_hz + self.colour[3])
                    .cos()
                    .abs();
                match leaf {
                    Leaf::Falling { sway, phase } => {
                        let swing = (self.age * std::f32::consts::TAU * cfg.leaf_sway_hz + phase)
                            .sin()
                            * cfg.leaf_sway_m;
                        // A swing lifts it a little at each end, as a leaf
                        // swoops.
                        let lift = swing.abs() * 0.25;
                        Some((
                            self.at + sway * swing + up * lift,
                            Vec2::new(self.size * (0.35 + 0.65 * flip), self.size),
                        ))
                    }
                    Leaf::Rested(since) => {
                        let fading =
                            (self.age - since - cfg.leaf_rest_s).max(0.0) / cfg.leaf_fade_s;
                        if fading >= 1.0 {
                            return None;
                        }
                        let s = self.size * (1.0 - fading);
                        Some((self.at, Vec2::new(s, s * 0.6)))
                    }
                }
            }
        }
    }

    /// One step of `dt` seconds. False once it is gone.
    fn step(&mut self, dt: f32, up: Vec3, cfg: &TreeFxSettings) -> bool {
        self.age += dt;
        match self.kind {
            Kind::Dust => {
                self.velocity *= (-cfg.dust_drag * dt).exp();
                self.velocity -= up * cfg.dust_gravity * dt;
                self.at += self.velocity * dt;
                let height = self.at.dot(up);
                if height < self.ground {
                    self.at += up * (self.ground - height);
                    self.velocity = Vec3::ZERO;
                }
                self.age < self.life
            }
            Kind::Leaf(Leaf::Falling { sway, phase }) => {
                // It lets go and comes up to its falling speed in a quarter
                // second.
                let fall = cfg.leaf_fall_mps * (self.age / 0.25).min(1.0);
                self.at -= up * fall * dt;
                let height = self.at.dot(up);
                if height <= self.ground {
                    // Down where it was swinging, flat on the ground.
                    let swing = (self.age * std::f32::consts::TAU * cfg.leaf_sway_hz + phase).sin()
                        * cfg.leaf_sway_m;
                    self.at += sway * swing + up * (self.ground - height + 0.02);
                    self.kind = Kind::Leaf(Leaf::Rested(self.age));
                }
                true
            }
            Kind::Leaf(Leaf::Rested(since)) => self.age - since < cfg.leaf_rest_s + cfg.leaf_fade_s,
        }
    }
}

/// One tree's dust and leaves, and the mesh that draws them.
struct Burst {
    /// The tree's base, planet-centred, metres.
    origin: DVec3,
    /// Up at the base.
    up: Vec3,
    particles: Vec<Particle>,
    entity: Entity,
    mesh: Handle<Mesh>,
}

/// Every live burst.
#[derive(Resource, Default)]
pub struct TreeFx {
    bursts: Vec<Burst>,
    material: Option<Handle<StandardMaterial>>,
    seed: u32,
}

impl TreeFx {
    /// Particles alive now, over every burst.
    pub fn live(&self) -> usize {
        self.bursts.iter().map(|b| b.particles.len()).sum()
    }

    /// 0..1, xorshift.
    fn roll(&mut self) -> f32 {
        let mut x = self.seed.max(1);
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.seed = x;
        (x >> 8) as f32 / (1u32 << 24) as f32
    }
}

/// A tree's look and place, for its burst: the base (planet-centred), its
/// size, and its leaf and bark means, linear (None for a species with no
/// leaves).
#[derive(Clone, Copy, Debug)]
struct TreeLook {
    base: DVec3,
    size: f32,
    leaf: Option<Vec3>,
    bark: Vec3,
}

/// The tree in `cell` as the near draw shows it.
fn tree_look(fine: &PlanetFine, cell: u32) -> Option<TreeLook> {
    let record = &fine.set.finest_records()[fine.set.finest_index_of(cell)?];
    let dir = Vec3::from_slice(&record.direction_height[..3])
        .as_dvec3()
        .try_normalize()?;
    let base = dir * (f64::from(PLANET_RADIUS) + f64::from(record.direction_height[3]));
    let pick = pbd_core::tree_kit::pick_word(record.spare[0], record.metadata[1]);
    let (size, species) = pick.map_or((1.0, None), |p| (p.size, Some(p.species)));
    let (leaf, bark) = species.map_or((None, Vec3::new(0.2, 0.14, 0.09)), |s| {
        let words = &crate::planet::trees::kit().words;
        let h = (s.index() * pbd_core::tree_kit::SPECIES_WORDS) as usize;
        let f = |i: usize| f32::from_bits(words[h + i]);
        let has_leaf = words[h + 3] & 4 != 0;
        (
            has_leaf.then(|| Vec3::new(f(8), f(9), f(10))),
            Vec3::new(f(4), f(5), f(6)),
        )
    });
    Some(TreeLook {
        base,
        size,
        leaf,
        bark,
    })
}

/// Two unit vectors across `up`.
fn across(up: Vec3) -> (Vec3, Vec3) {
    let a = up.any_orthonormal_vector();
    (a, up.cross(a))
}

/// The ground's height over the base at `offset` from it (along the base's
/// up), from the generator's surface there.
fn ground_at(origin: DVec3, up: Vec3, offset: Vec3) -> f32 {
    let point = origin + offset.as_dvec3();
    let Some(dir) = point.try_normalize() else {
        return 0.0;
    };
    let radius = f64::from(PLANET_RADIUS) + f64::from(surface_height(dir.as_vec3()));
    // The point lowered to the ground along its own ray, measured along the
    // base's up.
    let ground = (dir * radius - origin).as_vec3();
    ground.dot(up)
}

/// Make an event's specks and leaves into its tree's burst.
fn burst(
    fx: &mut TreeFx,
    cfg: &TreeFxSettings,
    event: TreeFxEvent,
    look: TreeLook,
    toward_eye: Vec3,
    eye_height: f32,
) -> Vec<Particle> {
    let up = look.base.normalize().as_vec3();
    let (a, b) = across(up);
    let toward = {
        let flat = toward_eye - up * toward_eye.dot(up);
        flat.try_normalize().unwrap_or(a)
    };
    let side = up.cross(toward);
    let room = (cfg.max_live as usize).saturating_sub(fx.live());
    let (dust, leaves) = cfg.counts(event);
    let dust = (dust as usize).min(room);
    let leaves = (leaves as usize).min(room - dust);
    let felled = matches!(event, TreeFxEvent::Felled { .. });
    let mut out = Vec::with_capacity(dust + leaves);
    let bark = look.bark;
    let earth = Vec3::from_array(cfg.dust_colour);
    for _ in 0..dust {
        let (at, dir, speed, life) = if felled {
            // Round the stump, out every way and up a little.
            let turn = fx.roll() * std::f32::consts::TAU;
            let out_dir = a * turn.cos() + b * turn.sin();
            (
                out_dir * 0.45 + up * (0.1 + 0.5 * fx.roll()),
                (out_dir + up * (0.2 + 0.4 * fx.roll())).normalize(),
                cfg.fell_dust_speed_mps,
                cfg.fell_dust_life_s,
            )
        } else {
            // The trunk's face toward the eye, at the stroke's height, out
            // toward the eye and to the sides.
            let h = eye_height.clamp(0.3, 2.6) + (fx.roll() - 0.5) * 0.3;
            let spread = (fx.roll() - 0.5) * 1.6;
            (
                toward * 0.45 + up * h + side * (fx.roll() - 0.5) * 0.2,
                (toward + side * spread + up * (fx.roll() - 0.3) * 0.6).normalize(),
                cfg.dust_speed_mps,
                cfg.dust_life_s,
            )
        };
        let mix = cfg.dust_mix * (0.5 + fx.roll());
        let c = bark.lerp(earth, mix.min(1.0)) * (0.85 + 0.25 * fx.roll());
        out.push(Particle {
            kind: Kind::Dust,
            at,
            velocity: dir * speed * (0.5 + 0.5 * fx.roll()),
            age: 0.0,
            life: life * (0.8 + 0.4 * fx.roll()),
            size: cfg.dust_size_m * (0.7 + 0.6 * fx.roll()),
            colour: [c.x, c.y, c.z, 1.0],
            ground: ground_at(look.base, up, at - up * at.dot(up)),
        });
    }
    if let Some(leaf) = look.leaf {
        for _ in 0..leaves {
            let turn = fx.roll() * std::f32::consts::TAU;
            let r = cfg.crown_radius_m * look.size * fx.roll().sqrt();
            let flat = (a * turn.cos() + b * turn.sin()) * r;
            let h =
                (cfg.crown_low_m + (cfg.crown_high_m - cfg.crown_low_m) * fx.roll()) * look.size;
            let at = flat + up * h;
            let sway_turn = fx.roll() * std::f32::consts::TAU;
            let sway = a * sway_turn.cos() + b * sway_turn.sin();
            let c = leaf * cfg.leaf_shade * (0.8 + 0.35 * fx.roll());
            out.push(Particle {
                kind: Kind::Leaf(Leaf::Falling {
                    sway,
                    phase: fx.roll() * std::f32::consts::TAU,
                }),
                at,
                velocity: Vec3::ZERO,
                age: 0.0,
                life: 0.0,
                size: cfg.leaf_size_m * (0.8 + 0.4 * fx.roll()),
                // w: the leaf's own turn-over phase; the card is opaque.
                colour: [c.x, c.y, c.z, fx.roll() * std::f32::consts::TAU],
                ground: ground_at(look.base, up, flat),
            });
        }
    }
    out
}

/// The cards of a burst, facing the eye (`right` and `upward` the camera's
/// axes), with an up normal: lit as the ground they fall to.
fn build_mesh(
    mesh: &mut Mesh,
    particles: &[Particle],
    up: Vec3,
    right: Vec3,
    upward: Vec3,
    cfg: &TreeFxSettings,
) -> Option<Aabb> {
    let mut positions = Vec::with_capacity(particles.len() * 4);
    let mut colours = Vec::with_capacity(particles.len() * 4);
    let mut indices = Vec::with_capacity(particles.len() * 6);
    for p in particles {
        let Some((at, half)) = p.drawn(up, cfg) else {
            continue;
        };
        let (w, h) = (right * half.x * 0.5, upward * half.y * 0.5);
        let base = positions.len() as u32;
        for corner in [at - w - h, at + w - h, at + w + h, at - w + h] {
            positions.push(corner.to_array());
        }
        let c = [p.colour[0], p.colour[1], p.colour[2], 1.0];
        colours.extend([c; 4]);
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    if positions.is_empty() {
        return None;
    }
    let (mut low, mut high) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for p in &positions {
        low = low.min(Vec3::from_array(*p));
        high = high.max(Vec3::from_array(*p));
    }
    let n = positions.len();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![up.to_array(); n]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.5f32, 0.5]; n]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
    mesh.insert_indices(Indices::U32(indices));
    Some(Aabb::from_min_max(low, high))
}

fn empty_mesh() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new())
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new())
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<[f32; 4]>::new())
    .with_inserted_indices(Indices::U32(Vec::new()))
}

/// Turn this frame's hits and fells into bursts, step every particle, and
/// redraw each burst's cards; a burst whose last particle is gone goes.
#[allow(clippy::too_many_arguments)]
fn run(
    mut commands: Commands,
    mut queue: ResMut<TreeFxQueue>,
    mut fx: ResMut<TreeFx>,
    cfg: Res<TreeFxSettings>,
    fine: Option<Res<PlanetFine>>,
    frame: Option<Res<PlanetRenderFrame>>,
    time: Res<Time>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let fx = &mut *fx;
    let camera = cameras
        .iter()
        .find(|(_, camera)| camera.is_active)
        .map(|(transform, _)| *transform);
    let centre = frame.as_deref().map_or(DVec3::ZERO, |f| f.center);
    // New bursts.
    let events: Vec<_> = queue.0.drain(..).collect();
    if let (Some(fine), Some(camera)) = (fine.as_deref(), camera) {
        for event in events {
            let cell = match event {
                TreeFxEvent::Hit { cell, .. } | TreeFxEvent::Felled { cell } => cell,
            };
            let Some(look) = tree_look(fine, cell) else {
                continue;
            };
            // The eye from the tree's base, in f64 before the cast.
            let eye = camera.translation().as_dvec3() - centre;
            let rel = (eye - look.base).as_vec3();
            let up = look.base.normalize().as_vec3();
            let particles = burst(fx, &cfg, event, look, rel, rel.dot(up) - 0.3);
            if particles.is_empty() {
                continue;
            }
            if fx.seed == 0 {
                fx.seed = cell | 1;
            }
            // One burst per tree: a hit adds to the tree's own.
            if let Some(b) = fx
                .bursts
                .iter_mut()
                .find(|b| b.origin.distance_squared(look.base) < 0.01)
            {
                b.particles.extend(particles);
                continue;
            }
            let material = fx
                .material
                .get_or_insert_with(|| {
                    materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        perceptual_roughness: 1.0,
                        reflectance: 0.0,
                        cull_mode: None,
                        double_sided: true,
                        ..default()
                    })
                })
                .clone();
            let mesh = meshes.add(empty_mesh());
            let entity = commands
                .spawn((
                    Name::new("tree fx"),
                    crate::field_light::LitByField,
                    crate::field_light::LitLikeTerrain,
                    bevy::light::NotShadowCaster,
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation((centre + look.base).as_vec3()),
                    Visibility::default(),
                    Aabb::from_min_max(Vec3::splat(-0.5), Vec3::splat(0.5)),
                ))
                .id();
            fx.bursts.push(Burst {
                origin: look.base,
                up,
                particles,
                entity,
                mesh,
            });
        }
    }
    if fx.bursts.is_empty() {
        return;
    }
    // Step and draw.
    let dt = time.delta_secs().min(0.1);
    let (right, upward) = camera.map_or((Vec3::X, Vec3::Y), |c| {
        (c.right().as_vec3(), c.up().as_vec3())
    });
    let cfg = &*cfg;
    fx.bursts.retain_mut(|b| {
        let up = b.up;
        b.particles.retain_mut(|p| p.step(dt, up, cfg));
        let aabb = meshes
            .get_mut(&b.mesh)
            .and_then(|mut mesh| build_mesh(&mut mesh, &b.particles, up, right, upward, cfg));
        let Some(aabb) = aabb else {
            commands.entity(b.entity).despawn();
            meshes.remove(&b.mesh);
            return false;
        };
        commands.entity(b.entity).insert((
            aabb,
            Transform::from_translation((centre + b.origin).as_vec3()),
        ));
        true
    });
}

/// The dust and leaves: the queue the dig fills, and the system that draws
/// them, while a world is open.
pub struct TreeFxPlugin;

impl Plugin for TreeFxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TreeFxQueue>()
            .init_resource::<TreeFx>()
            .init_resource::<TreeFxSettings>()
            .add_systems(Update, run.run_if(crate::world_open));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RON: &str = include_str!("../../../assets/config/tree_fx.ron");

    #[test]
    fn the_shipped_file_is_the_defaults() {
        let shipped: TreeFxSettings = ron::from_str(RON).unwrap();
        shipped.validate().unwrap();
        assert_eq!(shipped, TreeFxSettings::default());
        assert!(ron::from_str::<TreeFxSettings>("(max_lives: 3)").is_err());
        let bad = TreeFxSettings {
            max_live: 1000,
            ..default()
        };
        assert!(bad.validate().is_err());
    }

    fn look() -> TreeLook {
        TreeLook {
            base: DVec3::new(0.0, f64::from(PLANET_RADIUS), 0.0),
            size: 1.0,
            leaf: Some(Vec3::new(0.1, 0.3, 0.05)),
            bark: Vec3::new(0.2, 0.14, 0.09),
        }
    }

    /// A fist lets go fewer than an axe, the fell more than either, and the
    /// cap holds over every burst.
    #[test]
    fn counts_grow_with_the_blow_and_the_cap_holds() {
        let cfg = TreeFxSettings::default();
        let punch = cfg.counts(TreeFxEvent::Hit {
            cell: 1,
            by_axe: false,
        });
        let axe = cfg.counts(TreeFxEvent::Hit {
            cell: 1,
            by_axe: true,
        });
        let fell = cfg.counts(TreeFxEvent::Felled { cell: 1 });
        assert!(punch.0 < axe.0 && punch.1 < axe.1);
        assert!(axe.0 < fell.0 && axe.1 < fell.1);
        let mut fx = TreeFx {
            seed: 7,
            ..default()
        };
        let mut live = 0;
        for _ in 0..10 {
            let made = burst(
                &mut fx,
                &cfg,
                TreeFxEvent::Felled { cell: 1 },
                look(),
                Vec3::X * 2.0,
                1.4,
            );
            live += made.len();
            // Stand in for the bursts kept, without an app.
            fx.bursts.push(Burst {
                origin: DVec3::ZERO,
                up: Vec3::Y,
                particles: made,
                entity: Entity::PLACEHOLDER,
                mesh: Handle::default(),
            });
        }
        assert_eq!(live, cfg.max_live as usize);
        assert_eq!(fx.live(), cfg.max_live as usize);
    }

    /// A leaf falls from the crown, swaying, reaches the ground in about two
    /// seconds, lies there and fades; a speck puffs out and is gone in about
    /// half a second.
    #[test]
    fn leaves_fall_and_rest_and_dust_settles() {
        let cfg = TreeFxSettings::default();
        let up = Vec3::Y;
        let mut leaf = Particle {
            kind: Kind::Leaf(Leaf::Falling {
                sway: Vec3::X,
                phase: 0.0,
            }),
            at: up * 4.5,
            velocity: Vec3::ZERO,
            age: 0.0,
            life: 0.0,
            size: cfg.leaf_size_m,
            colour: [0.1, 0.3, 0.05, 0.0],
            ground: 0.0,
        };
        let dt = 1.0 / 60.0;
        let mut landed = None;
        let mut swayed = 0.0f32;
        let mut t = 0.0;
        while leaf.step(dt, up, &cfg) {
            t += dt;
            if let Some((at, _)) = leaf.drawn(up, &cfg) {
                swayed = swayed.max(at.x.abs());
                assert!(at.y >= -1e-3, "never under the ground");
            }
            if landed.is_none() && matches!(leaf.kind, Kind::Leaf(Leaf::Rested(_))) {
                landed = Some(t);
            }
            assert!(t < 10.0);
        }
        let landed = landed.expect("it lands");
        assert!((1.6..2.6).contains(&landed), "down in {landed} s");
        assert!(swayed > 0.5 * cfg.leaf_sway_m, "it sways");
        assert!(t >= landed + cfg.leaf_rest_s, "it lies a while");

        let mut speck = Particle {
            kind: Kind::Dust,
            at: up * 1.2,
            velocity: Vec3::X * cfg.dust_speed_mps,
            age: 0.0,
            life: cfg.dust_life_s,
            size: cfg.dust_size_m,
            colour: [0.3; 4],
            ground: 0.0,
        };
        let mut t = 0.0;
        while speck.step(dt, up, &cfg) {
            t += dt;
        }
        assert!((t - cfg.dust_life_s).abs() < 0.05, "gone in {t} s");
        assert!(speck.at.x > 0.1, "it puffed out");
    }

    /// The cards take the tree's colours: leaves its leaf, the dust between
    /// its bark and the earth; and a leafless species lets no leaves go.
    #[test]
    fn the_cards_take_the_tree_s_colours() {
        let cfg = TreeFxSettings::default();
        let mut fx = TreeFx {
            seed: 3,
            ..default()
        };
        let made = burst(
            &mut fx,
            &cfg,
            TreeFxEvent::Hit {
                cell: 1,
                by_axe: true,
            },
            look(),
            Vec3::X * 2.0,
            1.4,
        );
        for p in &made {
            match p.kind {
                Kind::Leaf(_) => assert!(p.colour[1] > p.colour[0] && p.colour[1] > p.colour[2]),
                Kind::Dust => assert!(p.colour[0] > p.colour[2], "an earth brown"),
            }
        }
        let bare = TreeLook {
            leaf: None,
            ..look()
        };
        let made = burst(
            &mut fx,
            &cfg,
            TreeFxEvent::Felled { cell: 1 },
            bare,
            Vec3::X,
            1.0,
        );
        assert!(made.iter().all(|p| p.kind == Kind::Dust));
        assert_eq!(made.len(), cfg.fell_dust as usize);
    }
}
