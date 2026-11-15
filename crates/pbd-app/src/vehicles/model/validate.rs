//! Validate actual exported geometry against the engine-independent config.
use super::*;
use pbd_core::vehicle::spec::VehicleSpecs;
use std::collections::BTreeSet;

fn require(ok: bool, message: impl Into<String>) -> Result<()> {
    if ok { Ok(()) } else { Err(message.into()) }
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

    #[cfg(test)]
    pub(super) fn find(&self, name: &str) -> Result<usize> {
        self.nodes
            .iter()
            .position(|n| n.name == name)
            .ok_or_else(|| format!("missing part {name}"))
    }

    /// The export contract of a craft's model against its config. Each
    /// authored craft has its own; none is registered since the Kestrel,
    /// the only one, was retired (`retire-kestrel`), so every kind is refused
    /// until a craft adds its contract here.
    pub(super) fn validate_config(&self, kind: Kind, _specs: &VehicleSpecs) -> Result<()> {
        Err(format!("the {} has no export contract", kind.name()))
    }
}
