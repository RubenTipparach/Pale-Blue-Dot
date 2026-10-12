//! Validate actual exported geometry against the engine-independent config.
use super::*;
use pbd_core::vehicle::{foil::FoilSpec, spec::VehicleSpecs};
use std::collections::BTreeSet;

const TOLERANCE: f32 = 2e-4;

fn require(ok: bool, message: impl Into<String>) -> Result<()> {
    if ok { Ok(()) } else { Err(message.into()) }
}

fn basis(foil: &FoilSpec) -> Mat3 {
    let chord = Vec3::from(foil.chord);
    let normal = Vec3::from(foil.normal);
    Mat3::from_cols(chord.cross(normal), normal, -chord)
}

impl Model {
    pub(super) fn validate_structure(&self) -> Result<()> {
        let mut names = BTreeSet::new();
        for node in &self.nodes {
            require(
                names.insert(&node.name),
                format!("duplicate part {}", node.name),
            )?;
            for p in &node.primitives {
                require(
                    p.positions.len() == p.normals.len() && p.positions.len() == p.uvs.len(),
                    "attribute count mismatch",
                )?;
                require(
                    p.indices.len().is_multiple_of(3) && !p.indices.is_empty(),
                    "invalid triangle count",
                )?;
                require(p.material < self.materials.len(), "invalid material index")?;
                require(
                    p.positions.iter().all(|v| Vec3::from(*v).is_finite())
                        && p.normals
                            .iter()
                            .all(|v| (Vec3::from(*v).length() - 1.0).abs() < 1e-4)
                        && p.uvs.iter().all(|v| Vec2::from(*v).is_finite()),
                    "nonfinite or invalid mesh attributes",
                )?;
                require(
                    p.indices.iter().all(|i| (*i as usize) < p.positions.len()),
                    "index out of range",
                )?;
                for tri in p.indices.chunks_exact(3) {
                    let [a, b, c] =
                        std::array::from_fn(|i| Vec3::from(p.positions[tri[i] as usize]));
                    require(
                        (b - a).cross(c - a).length() > 1e-10,
                        format!("degenerate triangle in {}", node.name),
                    )?;
                }
            }
        }
        fn visit(model: &Model, index: usize, seen: &mut BTreeSet<usize>) -> Result<()> {
            require(
                index < model.nodes.len() && seen.insert(index),
                "invalid or multiply parented model hierarchy",
            )?;
            for child in &model.nodes[index].children {
                visit(model, *child, seen)?;
            }
            Ok(())
        }
        let mut seen = BTreeSet::new();
        for root in &self.roots {
            visit(self, *root, &mut seen)?;
        }
        require(seen.len() == self.nodes.len(), "unreachable model parts")
    }

    pub(super) fn find(&self, name: &str) -> Result<usize> {
        self.nodes
            .iter()
            .position(|n| n.name == name)
            .ok_or_else(|| format!("missing part {name}"))
    }

    fn frame(&self, name: &str, parent: Option<&str>, at: Vec3, rotation: Mat3) -> Result<()> {
        let index = self.find(name)?;
        let parent_matches = if let Some(parent) = parent {
            self.nodes[self.find(parent)?].children.contains(&index)
        } else {
            self.roots.contains(&index)
        };
        let node = &self.nodes[index];
        require(
            parent_matches && node.transform.translation.distance(at) < TOLERANCE,
            format!("{name} parent/pivot differs from config"),
        )?;
        let actual = Mat3::from_quat(node.transform.rotation);
        require(
            actual.x_axis.distance(rotation.x_axis) < TOLERANCE
                && actual.y_axis.distance(rotation.y_axis) < TOLERANCE
                && actual.z_axis.distance(rotation.z_axis) < TOLERANCE,
            format!("{name} axes differ from config"),
        )
    }

    fn triangles(&self, root: usize) -> Vec<[Vec3; 3]> {
        fn walk(model: &Model, index: usize, transform: Mat4, out: &mut Vec<[Vec3; 3]>) {
            let node = &model.nodes[index];
            for p in &node.primitives {
                for tri in p.indices.chunks_exact(3) {
                    out.push(std::array::from_fn(|i| {
                        transform.transform_point3(Vec3::from(p.positions[tri[i] as usize]))
                    }));
                }
            }
            for child in &node.children {
                walk(
                    model,
                    *child,
                    transform * model.nodes[*child].transform.to_matrix(),
                    out,
                );
            }
        }
        let mut out = Vec::new();
        walk(self, root, Mat4::IDENTITY, &mut out);
        out
    }

    fn dimensions(&self, name: &str) -> Result<(Vec3, Vec3)> {
        let points = self.triangles(self.find(name)?).into_iter().flatten();
        let (mut lo, mut hi) = (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
        for p in points {
            lo = lo.min(p);
            hi = hi.max(p);
        }
        require(
            lo.is_finite() && hi.is_finite(),
            format!("{name} has no geometry"),
        )?;
        Ok((lo, hi))
    }

    fn foil(&self, name: &str, control: &str, spec: &FoilSpec, span: f32) -> Result<()> {
        self.frame(name, None, Vec3::from(spec.at), basis(spec))?;
        let chord = spec.area_m2 / span;
        self.frame(control, Some(name), Vec3::Z * chord / 4.0, Mat3::IDENTITY)?;
        let (lo, hi) = self.dimensions(name)?;
        let area: f32 = self
            .triangles(self.find(name)?)
            .iter()
            .map(|[a, b, c]| (*b - *a).cross(*c - *a).y.max(0.0) * 0.5)
            .sum();
        require(
            (hi.x - lo.x - span).abs() < TOLERANCE
                && (hi.z - lo.z - chord).abs() < TOLERANCE
                && (lo.x + span / 2.0).abs() < TOLERANCE
                && (lo.z + chord / 2.0).abs() < TOLERANCE
                && (area - spec.area_m2).abs() < TOLERANCE,
            format!("{name} span/chord/neutral planform area differs from config"),
        )
    }

    pub(super) fn validate_config(&self, kind: Kind, specs: &VehicleSpecs) -> Result<()> {
        match kind {
            Kind::Kestrel => {
                let s = &specs.kestrel;
                self.frame("crew", None, Vec3::from(s.seat.eye), Mat3::IDENTITY)?;
                for (side, foil) in ["left", "right"].into_iter().zip(s.wing.panels()) {
                    let span = (s.wing.panel_area_m2 * s.wing.aspect * 2.0).sqrt() / 2.0;
                    self.foil(
                        &format!("wing_{side}"),
                        &format!("flaperon_{side}"),
                        &foil,
                        span,
                    )?;
                }
                for (name, control, foil) in
                    [("tail", "elevator", &s.tail), ("fin", "rudder", &s.fin)]
                {
                    self.foil(name, control, foil, (foil.area_m2 * foil.aspect).sqrt())?;
                }
                for (side, sign) in [("left", -1.0), ("right", 1.0)] {
                    let [x, y, z] = s.rotor.at;
                    let nacelle = format!("nacelle_{side}");
                    let rotor = format!("rotor_{side}");
                    self.frame(&nacelle, None, Vec3::new(sign * x, y, z), Mat3::IDENTITY)?;
                    self.frame(&rotor, Some(&nacelle), Vec3::Y * 0.8, Mat3::IDENTITY)?;
                    let radius = self
                        .triangles(self.find(&rotor)?)
                        .into_iter()
                        .flatten()
                        .map(|p| Vec2::new(p.x, p.z).length())
                        .fold(0.0, f32::max);
                    require(
                        (radius - s.rotor.radius_m).abs() < TOLERANCE,
                        format!("{rotor} radius differs from config"),
                    )?;
                    for blade in 0..3 {
                        self.find(&format!("blade_{side}_{blade}"))?;
                    }
                }
                for (i, gear) in s.gear.iter().enumerate() {
                    let name = format!("wheel_{i}");
                    self.frame(
                        &name,
                        None,
                        Vec3::from(gear.at) + Vec3::Y * 0.23,
                        Mat3::IDENTITY,
                    )?;
                    let (lo, _) = self.dimensions(&name)?;
                    require(
                        (lo.y + 0.23).abs() < TOLERANCE,
                        format!("{name} contact differs from config"),
                    )?;
                }
                self.find("fuselage")?;
                self.find("canopy")?;
                Ok(())
            }
            _ => Err("craft export contract is not implemented".into()),
        }
    }
}
