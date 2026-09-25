//! The rigid, textured GLB subset exported by our Blender authoring pipeline.
//! Art is cached derived state; it never feeds forces, collision or saves.
#[cfg(test)]
mod tests;
mod validate;

use bevy::{
    asset::RenderAssetUsages,
    image::{CompressedImageFormats, ImageSampler, ImageType},
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use pbd_core::vehicle::{Craft, Kind};
use std::path::PathBuf;

pub(super) const AUTHORED: &[Kind] = &[Kind::Kestrel];
pub(super) type Result<T> = std::result::Result<T, String>;

pub(super) fn folder(kind: Kind) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/models/vehicles")
        .join(kind.name().to_ascii_lowercase())
}

struct Primitive {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
    material: usize,
}

struct Node {
    name: String,
    transform: Transform,
    children: Vec<usize>,
    primitives: Vec<Primitive>,
}

struct Model {
    nodes: Vec<Node>,
    roots: Vec<usize>,
    materials: Vec<StandardMaterial>,
    png: Vec<u8>,
}

impl Model {
    fn read(kind: Kind) -> Result<Self> {
        let path = folder(kind).join(format!("{}.glb", kind.name().to_ascii_lowercase()));
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::parse(&bytes)
    }

    fn parse(bytes: &[u8]) -> Result<Self> {
        let glb = gltf::Gltf::from_slice(bytes).map_err(|e| e.to_string())?;
        let blob = glb
            .blob
            .as_deref()
            .ok_or("model needs an embedded buffer")?;
        if glb.buffers().count() != 1 || glb.animations().len() != 0 || glb.skins().len() != 0 {
            return Err("model needs one buffer and rigid, unanimated nodes".into());
        }
        if glb.images().len() != 1 || glb.textures().len() == 0 {
            return Err("model needs one embedded PNG atlas".into());
        }
        for texture in glb.textures() {
            if texture.source().index() != 0
                || texture.sampler().mag_filter() != Some(gltf::texture::MagFilter::Nearest)
                || !matches!(
                    texture.sampler().min_filter(),
                    Some(
                        gltf::texture::MinFilter::Nearest
                            | gltf::texture::MinFilter::NearestMipmapNearest
                    )
                )
            {
                return Err("model textures must use nearest sampling without a mip chain".into());
            }
        }
        let png = match glb.images().next().unwrap().source() {
            gltf::image::Source::View {
                view,
                mime_type: "image/png",
            } => blob
                .get(view.offset()..view.offset() + view.length())
                .ok_or("invalid image buffer view")?
                .to_vec(),
            _ => return Err("external or non-PNG model image".into()),
        };
        let mut materials = Vec::new();
        for material in glb.materials() {
            let pbr = material.pbr_metallic_roughness();
            if pbr.base_color_texture().is_none_or(|t| t.tex_coord() != 0)
                || material.alpha_mode() != gltf::material::AlphaMode::Opaque
                || material.normal_texture().is_some()
                || material.occlusion_texture().is_some()
                || material.emissive_texture().is_some()
                || pbr.metallic_roughness_texture().is_some()
            {
                return Err("unsupported model material: expected opaque UV0 albedo".into());
            }
            let [r, g, b, a] = pbr.base_color_factor();
            materials.push(StandardMaterial {
                base_color: Color::linear_rgba(r, g, b, a),
                perceptual_roughness: pbr.roughness_factor(),
                metallic: pbr.metallic_factor(),
                double_sided: material.double_sided(),
                cull_mode: if material.double_sided() {
                    None
                } else {
                    Some(bevy::render::render_resource::Face::Back)
                },
                ..default()
            });
        }
        let mut nodes = Vec::new();
        for node in glb.nodes() {
            let (t, r, s) = node.transform().decomposed();
            let transform = Transform {
                translation: Vec3::from(t),
                rotation: Quat::from_array(r),
                scale: Vec3::from(s),
            };
            if !transform.translation.is_finite()
                || !transform.rotation.is_finite()
                || transform.scale.distance(Vec3::ONE) > 1e-5
                || (transform.rotation.length() - 1.0).abs() > 1e-5
            {
                return Err(
                    "model transforms must be finite rigid transforms with unit scale".into(),
                );
            }
            let mut primitives = Vec::new();
            if let Some(mesh) = node.mesh() {
                for primitive in mesh.primitives() {
                    if primitive.mode() != gltf::mesh::Mode::Triangles
                        || primitive.morph_targets().len() != 0
                    {
                        return Err("model needs static triangle primitives".into());
                    }
                    let reader = primitive.reader(|buffer| (buffer.index() == 0).then_some(blob));
                    let positions = reader
                        .read_positions()
                        .ok_or("missing positions")?
                        .collect();
                    let normals = reader.read_normals().ok_or("missing normals")?.collect();
                    let uvs = reader
                        .read_tex_coords(0)
                        .ok_or("missing UV0")?
                        .into_f32()
                        .collect();
                    let indices = reader
                        .read_indices()
                        .ok_or("missing indices")?
                        .into_u32()
                        .collect();
                    primitives.push(Primitive {
                        positions,
                        normals,
                        uvs,
                        indices,
                        material: primitive.material().index().ok_or("missing material")?,
                    });
                }
            }
            nodes.push(Node {
                name: node.name().ok_or("every model node needs a name")?.into(),
                transform,
                children: node.children().map(|c| c.index()).collect(),
                primitives,
            });
        }
        let roots = glb
            .default_scene()
            .ok_or("missing default scene")?
            .nodes()
            .map(|n| n.index())
            .collect();
        let model = Self {
            nodes,
            roots,
            materials,
            png,
        };
        model.validate_structure()?;
        Ok(model)
    }

    fn image(&self) -> Result<Image> {
        Image::from_buffer(
            &self.png,
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            true,
            ImageSampler::nearest(),
            RenderAssetUsages::default(),
        )
        .map_err(|e| e.to_string())
    }
}

struct RenderModel {
    data: Model,
    meshes: Vec<Vec<Handle<Mesh>>>,
    materials: Vec<Handle<StandardMaterial>>,
}

#[derive(Resource, Default)]
struct Cache(Vec<(Kind, RenderModel)>);

pub(super) fn build(world: &mut World, owner: Entity, craft: &Craft) -> Result<()> {
    let mut cache = world.remove_resource::<Cache>().unwrap_or_default();
    let result = (|| {
        let index = if let Some(index) = cache.0.iter().position(|(kind, _)| *kind == craft.kind) {
            index
        } else {
            let data = Model::read(craft.kind)?;
            data.validate_config(craft.kind, craft.specs())?;
            let image = world.resource_mut::<Assets<Image>>().add(data.image()?);
            let materials = data
                .materials
                .iter()
                .map(|m| {
                    world
                        .resource_mut::<Assets<StandardMaterial>>()
                        .add(StandardMaterial {
                            base_color_texture: Some(image.clone()),
                            ..m.clone()
                        })
                })
                .collect();
            let meshes = data
                .nodes
                .iter()
                .map(|n| {
                    n.primitives
                        .iter()
                        .map(|p| {
                            world.resource_mut::<Assets<Mesh>>().add(
                                Mesh::new(
                                    PrimitiveTopology::TriangleList,
                                    RenderAssetUsages::default(),
                                )
                                .with_inserted_attribute(
                                    Mesh::ATTRIBUTE_POSITION,
                                    p.positions.clone(),
                                )
                                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, p.normals.clone())
                                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, p.uvs.clone())
                                .with_inserted_indices(Indices::U32(p.indices.clone())),
                            )
                        })
                        .collect()
                })
                .collect();
            cache.0.push((
                craft.kind,
                RenderModel {
                    data,
                    meshes,
                    materials,
                },
            ));
            cache.0.len() - 1
        };
        let model = &cache.0[index].1;
        model.data.validate_config(craft.kind, craft.specs())?;
        for root in &model.data.roots {
            instantiate(world, model, *root, owner, owner, craft.kind);
        }
        Ok(())
    })();
    world.insert_resource(cache);
    result.map_err(|e: String| {
        format!(
            "{} model: {e}; regenerate and re-export from Blender",
            craft.kind.name()
        )
    })
}

fn instantiate(
    world: &mut World,
    model: &RenderModel,
    index: usize,
    parent: Entity,
    owner: Entity,
    kind: Kind,
) {
    let node = &model.data.nodes[index];
    let entity = world
        .spawn((
            Name::new(node.name.clone()),
            node.transform,
            Visibility::Inherited,
        ))
        .id();
    world.entity_mut(parent).add_child(entity);
    if let Some(part) = super::draw::Part::named(owner, kind, &node.name) {
        world.entity_mut(entity).insert(part);
    }
    for (primitive, mesh) in node.primitives.iter().zip(&model.meshes[index]) {
        let child = world
            .spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(model.materials[primitive.material].clone()),
                Transform::IDENTITY,
                Visibility::Inherited,
            ))
            .id();
        world.entity_mut(entity).add_child(child);
    }
    for child in &node.children {
        instantiate(world, model, *child, entity, owner, kind);
    }
}
