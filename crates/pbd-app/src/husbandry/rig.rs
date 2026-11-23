//! An animal's body, posed and built as one mesh each frame (S11).
//!
//! A rig is a handful of boxes hung from pivots - hips, a neck, a tail - so
//! a leg swings from its hip and a head dips from its neck. The pose is the
//! behaviour's (`super::Beast`): a gait phase advanced by ground distance
//! over stride (so feet never slide), a head pitch for a peck or a graze, a
//! glance, a tail flick, a weight shift, and how far it has lain down.
//!
//! Rig space is metres from the feet: X forward, Y up, Z to the right of
//! X x Y. The mesh is built in the body's frame round the animal's feet.

use bevy::prelude::*;
use pbd_core::husbandry::{Genome, LandSpecies, Plan, Sex};

/// A mesh being built: flat-shaded boxes with a colour each, or (for a pen
/// piece, `animal-husbandry` design 14) boxes wearing a texture.
#[derive(Default)]
pub struct Builder {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colours: Vec<[f32; 4]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    /// Whether any box was textured: only then are UVs written, so an
    /// animal's mesh keeps the layout its pipeline was built for.
    pub textured: bool,
}

impl Builder {
    /// A box: its centre and three half-axes (each a direction times a half
    /// extent), in the mesh's space. The colour is sRGB, as the data and a
    /// `Color::srgb` are; a vertex colour is linear, so it is converted.
    pub fn cuboid(&mut self, centre: Vec3, axes: [Vec3; 3], colour: [f32; 3]) {
        self.boxed(centre, axes, colour, None);
    }

    /// A box wearing a texture that repeats every `repeat_m` metres along
    /// each face (the settlement manifest's scale), untinted.
    pub fn textured_cuboid(&mut self, centre: Vec3, axes: [Vec3; 3], repeat_m: f32) {
        self.textured = true;
        self.boxed(centre, axes, [1.0, 1.0, 1.0], Some(repeat_m.max(0.05)));
    }

    fn boxed(&mut self, centre: Vec3, axes: [Vec3; 3], colour: [f32; 3], repeat: Option<f32>) {
        let [r, g, b, _] = Color::srgb(colour[0], colour[1], colour[2])
            .to_linear()
            .to_f32_array();
        let c = [r, g, b, 1.0];
        for k in 0..3 {
            let (a, b, d) = (axes[k], axes[(k + 1) % 3], axes[(k + 2) % 3]);
            for sign in [1.0f32, -1.0] {
                let n = (a * sign).normalize_or(Vec3::Y);
                let face = centre + a * sign;
                let (u, v) =
                    repeat.map_or((0.0, 0.0), |m| (2.0 * b.length() / m, 2.0 * d.length() / m));
                let mut quad = [
                    (face - b - d, [0.0, 0.0]),
                    (face + b - d, [u, 0.0]),
                    (face + b + d, [u, v]),
                    (face - b + d, [0.0, v]),
                ];
                if (quad[1].0 - quad[0].0).cross(quad[2].0 - quad[0].0).dot(n) < 0.0 {
                    quad.reverse();
                }
                let base = self.positions.len() as u32;
                for (p, uv) in quad {
                    self.positions.push(p.to_array());
                    self.normals.push(n.to_array());
                    self.colours.push(c);
                    self.uvs.push(uv);
                }
                self.indices.extend_from_slice(&[
                    base,
                    base + 1,
                    base + 2,
                    base,
                    base + 2,
                    base + 3,
                ]);
            }
        }
    }

    pub fn write(self, mesh: &mut Mesh) {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colours);
        if self.textured {
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        }
        mesh.insert_indices(bevy::mesh::Indices::U32(self.indices));
    }
}

/// A pivot's place and turn in rig space.
#[derive(Clone, Copy)]
struct Xf {
    r: Quat,
    t: Vec3,
}

impl Xf {
    fn at(t: Vec3) -> Self {
        Self {
            r: Quat::IDENTITY,
            t,
        }
    }

    fn turned(self, r: Quat) -> Self {
        Self {
            r: self.r * r,
            ..self
        }
    }

    /// A child pivot `t` from this one, turned by `r`.
    fn child(self, t: Vec3, r: Quat) -> Self {
        Self {
            r: self.r * r,
            t: self.t + self.r * t,
        }
    }
}

/// How an animal stands this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pose {
    /// Gait phase, radians.
    pub phase: f32,
    /// How far the legs swing, 0 (standing) to about 0.6 rad.
    pub swing: f32,
    /// Head pitch down, radians (a peck, a graze).
    pub head_down: f32,
    /// Head turn, radians left of ahead.
    pub glance: f32,
    /// Head forward and back with the steps (a bird's bob), m in rig units.
    pub bob: f32,
    /// Tail lift (bird) or swish (beast), radians.
    pub tail: f32,
    /// Side-to-side weight shift, radians of roll.
    pub shift: f32,
    /// 0 standing to 1 lying or roosting.
    pub down: f32,
}

/// An animal's colours: body, accent, legs and head, its coat and pattern
/// applied.
pub fn colours(species: &LandSpecies, genome: &Genome) -> ([[f32; 3]; 4], usize) {
    let mut c = species.colours;
    let Some(keep) = species.keep.as_ref() else {
        return (c, 0);
    };
    let tint = |name: &str| -> Option<[f32; 3]> {
        Some(match name {
            "russet" => [0.62, 0.30, 0.14],
            "black" => [0.13, 0.12, 0.11],
            "grey" => [0.60, 0.60, 0.58],
            "red" => [0.55, 0.22, 0.12],
            "dun" => [0.72, 0.60, 0.42],
            "golden" => [0.95, 0.72, 0.25],
            "melanic" => [0.08, 0.08, 0.09],
            "albino" => [0.97, 0.95, 0.93],
            "bay" => [0.45, 0.28, 0.16],
            "chestnut" => [0.58, 0.3, 0.14],
            "dapple" => [0.7, 0.68, 0.64],
            _ => return None,
        })
    };
    let coat = genome.coat();
    if genome.morph() {
        if let Some(t) = tint(&keep.morph) {
            c[0] = t;
        }
    } else if coat > 0
        && let Some(t) = keep.coats.get(coat).and_then(|n| tint(n))
    {
        c[0] = t;
        if species.plan == Plan::Bird {
            c[3] = t;
        }
    }
    (c, genome.pattern())
}

/// Build an animal at its pose: rig space mapped through `basis` (forward,
/// up, right) into the mesh's space round its feet.
#[allow(clippy::too_many_arguments)]
pub fn build(
    out: &mut Builder,
    species: &LandSpecies,
    genome: &Genome,
    sex: Sex,
    pose: &Pose,
    basis: [Vec3; 3],
) {
    let s = genome.size_factor();
    let (c, pattern) = colours(species, genome);
    let m = Mat3::from_cols(basis[0], basis[1], basis[2]);
    let mut part = |xf: Xf, centre: Vec3, half: Vec3, colour: [f32; 3]| {
        let at = xf.t + xf.r * centre;
        let axes = [
            m * (xf.r * Vec3::X * half.x),
            m * (xf.r * Vec3::Y * half.y),
            m * (xf.r * Vec3::Z * half.z),
        ];
        out.cuboid(m * at, axes, colour);
    };
    match species.plan {
        Plan::Bird => bird(
            &mut part,
            species.length_m * s / 0.42,
            &c,
            sex,
            pattern,
            pose,
        ),
        Plan::Quadruped => beast(
            &mut part,
            species.length_m * s,
            species.height_m * s,
            &c,
            pattern,
            pose,
        ),
        // The horse is one lofted body, not boxes (`land-transport` design
        // 11): its own builder, in the same basis.
        Plan::Horse => {
            let mut horse = Builder::default();
            crate::vehicles::carts::model::horse(
                &mut horse,
                &crate::vehicles::carts::model::colours_of(species, genome),
                crate::vehicles::carts::model::HorsePose {
                    phase: pose.phase,
                    walk: (pose.swing / 0.45).clamp(0.0, 1.0),
                    head_down: pose.head_down,
                    tail: pose.tail,
                },
                s * species.height_m / 1.55,
            );
            // Model space faces -Z with +X right; the rig's basis is
            // forward, up, right.
            let to = |p: [f32; 3]| m * Vec3::new(-p[2], p[1], p[0]);
            let base = out.positions.len() as u32;
            for (p, n) in horse.positions.iter().zip(&horse.normals) {
                out.positions.push(to(*p).to_array());
                out.normals.push(to(*n).normalize_or(Vec3::Y).to_array());
            }
            out.colours.extend(horse.colours);
            out.uvs.extend(horse.uvs);
            out.indices.extend(horse.indices.iter().map(|i| i + base));
        }
    }
}

fn bird(
    part: &mut impl FnMut(Xf, Vec3, Vec3, [f32; 3]),
    s: f32,
    c: &[[f32; 3]; 4],
    sex: Sex,
    pattern: usize,
    pose: &Pose,
) {
    let v = |x: f32, y: f32, z: f32| Vec3::new(x, y, z) * s;
    let down = pose.down.clamp(0.0, 1.0);
    // Roosting, the body settles onto the legs folded under it.
    let lift = 0.11 * (1.0 - down);
    let body = Xf::at(v(0.0, lift, 0.0)).turned(Quat::from_rotation_x(pose.shift));
    part(body, v(0.0, 0.09, 0.0), v(0.12, 0.085, 0.085), c[0]);
    // A speckled hen carries darker flecks along her sides.
    if pattern > 0 {
        for (x, y) in [(-0.05, 0.12), (0.03, 0.07), (-0.02, 0.04), (0.06, 0.12)] {
            for z in [-0.086, 0.086] {
                part(
                    body,
                    v(x, y, z),
                    v(0.012, 0.012, 0.004),
                    c[2].map(|k| k * 0.5),
                );
            }
        }
    }
    // Tail feathers, lifted and flicked.
    let tail = body.child(v(-0.10, 0.13, 0.0), Quat::from_rotation_z(0.6 + pose.tail));
    let tail_colour = if sex == Sex::M {
        [0.12, 0.25, 0.2]
    } else {
        c[0]
    };
    let tail_size = if sex == Sex::M { 1.4 } else { 1.0 };
    part(
        tail,
        v(-0.03, 0.05 * tail_size, 0.0),
        v(0.03, 0.07 * tail_size, 0.05),
        tail_colour,
    );
    // The neck: it turns to glance, dips to peck, and bobs with the steps.
    let neck = body.child(
        v(0.08 + pose.bob, 0.14, 0.0),
        Quat::from_rotation_y(pose.glance) * Quat::from_rotation_z(-pose.head_down * (1.0 - down)),
    );
    part(neck, v(0.02, 0.04, 0.0), v(0.03, 0.05, 0.035), c[0]);
    part(neck, v(0.04, 0.10, 0.0), v(0.045, 0.04, 0.035), c[3]);
    part(neck, v(0.095, 0.095, 0.0), v(0.022, 0.01, 0.012), c[2]);
    let comb = if sex == Sex::M { 1.7 } else { 1.0 };
    part(
        neck,
        v(0.04, 0.14 + 0.01 * comb, 0.0),
        v(0.035 * comb.sqrt(), 0.017 * comb, 0.008),
        c[1],
    );
    part(
        neck,
        v(0.075, 0.065, 0.0),
        v(0.012, 0.02 * comb, 0.008),
        c[1],
    );
    for z in [-0.036, 0.036] {
        part(neck, v(0.06, 0.11, z), v(0.008, 0.008, 0.004), [0.04; 3]);
    }
    // Legs from the hips, swinging opposite; folded away when roosting.
    if down < 0.9 {
        for (side, phase) in [(-1.0, 0.0), (1.0, std::f32::consts::PI)] {
            let swing = (pose.phase + phase).sin() * pose.swing;
            let hip = Xf::at(v(0.0, lift + 0.02, 0.04 * side))
                .child(Vec3::ZERO, Quat::from_rotation_z(swing));
            let k = 1.0 - down;
            part(hip, v(0.0, -0.055 * k, 0.0), v(0.01, 0.055 * k, 0.01), c[2]);
            part(hip, v(0.02, -0.11 * k, 0.0), v(0.03, 0.005, 0.015), c[2]);
        }
    }
}

fn beast(
    part: &mut impl FnMut(Xf, Vec3, Vec3, [f32; 3]),
    l: f32,
    h: f32,
    c: &[[f32; 3]; 4],
    pattern: usize,
    pose: &Pose,
) {
    let down = pose.down.clamp(0.0, 1.0);
    let leg = 0.5 * h;
    let w = 0.17 * l;
    // Lying, the body sinks to the ground with the legs folded under.
    let lift = leg * (1.0 - 0.75 * down);
    let body = Xf::at(Vec3::new(0.0, lift, 0.0)).turned(Quat::from_rotation_x(pose.shift));
    part(
        body,
        Vec3::new(0.0, 0.2 * h, 0.0),
        Vec3::new(0.36 * l, 0.2 * h, w),
        c[0],
    );
    // The belly and the pattern: a belt round the middle, or a patch.
    part(
        body,
        Vec3::new(0.0, 0.03 * h, 0.0),
        Vec3::new(0.26 * l, 0.04 * h, w * 0.9),
        c[1],
    );
    if pattern > 0 {
        part(
            body,
            Vec3::new(-0.02 * l, 0.2 * h, 0.0),
            Vec3::new(0.08 * l, 0.205 * h, w * 1.02),
            c[1],
        );
    }
    // The tail, swishing.
    let tail = body.child(
        Vec3::new(-0.36 * l, 0.34 * h, 0.0),
        Quat::from_rotation_x(pose.tail) * Quat::from_rotation_z(-0.3),
    );
    part(
        tail,
        Vec3::new(0.0, -0.14 * h, 0.0),
        Vec3::new(0.025 * l, 0.14 * h, 0.025 * l),
        c[0],
    );
    // The neck and head: down to graze, up to look about.
    let neck = body.child(
        Vec3::new(0.32 * l, 0.28 * h, 0.0),
        Quat::from_rotation_y(pose.glance) * Quat::from_rotation_z(0.35 - pose.head_down),
    );
    part(
        neck,
        Vec3::new(0.12 * l, 0.0, 0.0),
        Vec3::new(0.13 * l, 0.1 * h, 0.6 * w),
        c[0],
    );
    let head = neck.child(Vec3::new(0.27 * l, 0.0, 0.0), Quat::from_rotation_z(-0.5));
    part(
        head,
        Vec3::new(0.06 * l, 0.0, 0.0),
        Vec3::new(0.1 * l, 0.09 * h, 0.5 * w),
        c[3],
    );
    part(
        head,
        Vec3::new(0.15 * l, -0.03 * h, 0.0),
        Vec3::new(0.03 * l, 0.06 * h, 0.42 * w),
        c[1],
    );
    for z in [-1.0, 1.0] {
        part(
            head,
            Vec3::new(0.0, 0.08 * h, 0.55 * w * z),
            Vec3::new(0.03 * l, 0.015 * h, 0.12 * w),
            c[3],
        );
        part(
            head,
            Vec3::new(0.09 * l, 0.04 * h, 0.5 * w * z),
            Vec3::new(0.01 * l, 0.012 * h, 0.01 * l),
            [0.04; 3],
        );
    }
    // Four legs from the hips, the diagonal pairs together.
    for (x, z, phase) in [
        (0.27, -1.0, 0.0),
        (0.27, 1.0, std::f32::consts::PI),
        (-0.27, -1.0, std::f32::consts::PI),
        (-0.27, 1.0, 0.0),
    ] {
        let swing = (pose.phase + phase).sin() * pose.swing;
        let fold = down * 1.45 * if x > 0.0 { -1.0 } else { 1.0 };
        let hip = Xf::at(Vec3::new(x * l, lift, 0.6 * w * z))
            .child(Vec3::ZERO, Quat::from_rotation_z(swing + fold));
        let t = 0.045 * l;
        let k = 1.0 - 0.4 * down;
        part(
            hip,
            Vec3::new(0.0, -0.5 * leg * k, 0.0),
            Vec3::new(t, 0.5 * leg * k, t),
            c[2],
        );
        part(
            hip,
            Vec3::new(0.0, -leg * k + 0.03 * h, 0.0),
            Vec3::new(t * 1.1, 0.03 * h, t * 1.1),
            [c[2][0] * 0.6, c[2][1] * 0.6, c[2][2] * 0.6],
        );
    }
}
