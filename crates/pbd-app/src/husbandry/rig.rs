//! An animal's body, posed and built as one mesh each frame (S11).
//!
//! Every species is one lofted, smooth-shaded form in its coat
//! (`super::forms`, `animal-husbandry` design 16): a leg swings from its hip
//! and a head dips from its neck. The pose is the behaviour's
//! (`super::Beast`): a gait phase advanced by ground distance over stride
//! (so feet never slide), a head pitch for a peck or a graze, a glance, a
//! tail flick, a weight shift, and how far it has lain down.
//!
//! Rig space is metres from the feet: X forward, Y up, Z to the right of
//! X x Y. The mesh is built in the body's frame round the animal's feet.
//! The [`Builder`] also builds the pen's textured boxes.

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

/// An animal's coat for its form: its colours with its coat and pattern
/// applied.
pub fn coat(species: &LandSpecies, genome: &Genome) -> super::forms::Coat {
    let (c, pattern) = colours(species, genome);
    super::forms::Coat {
        body: c[0],
        pale: c[1],
        legs: c[2],
        head: c[3],
        pattern,
    }
}

/// Build an animal at its pose: its form in model space (+X right, +Y up,
/// -Z forward), turned through `basis` (forward, up, right) into the mesh's
/// space round its feet.
#[allow(clippy::too_many_arguments)]
pub fn build(
    out: &mut Builder,
    species: &LandSpecies,
    genome: &Genome,
    sex: Sex,
    pose: &Pose,
    basis: [Vec3; 3],
) {
    use super::forms;
    let s = genome.size_factor();
    let coat = coat(species, genome);
    let m = Mat3::from_cols(basis[0], basis[1], basis[2]);
    let stance = forms::Stance {
        phase: pose.phase,
        walk: (pose.swing / 0.45).clamp(0.0, 1.0),
        head_down: pose.head_down,
        glance: pose.glance,
        bob: pose.bob,
        tail: pose.tail,
        shift: pose.shift,
        down: pose.down,
        saddled: false,
    };
    let mut body = Builder::default();
    match (species.plan, forms::Beast::of_id(&species.id)) {
        (Plan::Bird, _) => forms::bird(&mut body, &coat, sex, &stance, s * species.height_m / 0.42),
        (_, kind) => {
            let kind = kind.unwrap_or(if species.plan == Plan::Horse {
                forms::Beast::Horse
            } else {
                forms::Beast::Plodder
            });
            forms::beast(
                &mut body,
                kind,
                &coat,
                &stance,
                s * species.height_m / kind.withers_m(),
            );
        }
    }
    // Model space faces -Z with +X right; the rig's basis is forward, up,
    // right.
    let to = |p: [f32; 3]| m * Vec3::new(-p[2], p[1], p[0]);
    let base = out.positions.len() as u32;
    for (p, n) in body.positions.iter().zip(&body.normals) {
        out.positions.push(to(*p).to_array());
        out.normals.push(to(*n).normalize_or(Vec3::Y).to_array());
    }
    out.colours.extend(body.colours);
    out.uvs.extend(body.uvs);
    out.textured |= body.textured;
    out.indices.extend(body.indices.iter().map(|i| i + base));
}
